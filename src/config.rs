//! ⚙️ 配置處理模組
//! 負責讀取同驗證 config.toml 配置檔 ✨
//!
//! 新架構：每個 profile 有獨立目錄
//!   ~/.config/opencb/config.toml          ← 全域預設
//!   ~/.config/opencb/<profile_id>/config.toml ← 個別 profile

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::profile_manager;

/// Regex for validating profile names and command names: ^[a-z0-9_-]+$
#[allow(dead_code)]
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

/// 🎯 Target CLI 規格（用嚟描述要叫邊個外部 CLI）
/// 例如 [target.opencode] cmd="opencode" argv=["run","#INPUT#"] work_dir="/tmp"
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TargetSpec {
    /// 🛠️ 執行檔（例如 "opencode"、"bash"）
    pub cmd: String,
    /// 📝 argv 列表，當中 `#INPUT#` 會被訊息內容取代
    pub argv: Vec<String>,
    /// 📂 可選工作目錄
    #[serde(default)]
    pub work_dir: Option<String>,
}

/// 🤖 Bot 配置結構體（對應 config.toml 嘅欄位）
/// 每個 config.toml 就是一個 profile，格式扁平無 sections
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    /// 配置檔嘅完整路徑（運行時設定，唔從 TOML 讀取）
    #[serde(default, skip)]
    pub config_path: PathBuf,
    /// Discord bot token
    pub bot_token: String,
    /// 目標頻道嘅 channel ID 列表，支援 ["*"] 表示接受所有頻道
    #[serde(default)]
    pub channel_ids: Vec<String>,
    /// Bot owner 嘅 user ID 列表（字串陣列）
    #[serde(default)]
    pub owner_id: Vec<String>,
    /// 係咪開 debug 模式（可選）。預設 false
    pub debug: Option<bool>,
    /// 🎯 Target CLI map
    #[serde(default)]
    pub targets: HashMap<String, TargetSpec>,
    /// Admin HTTP server bind address for scheduling admin endpoint
    #[serde(default)]
    pub scheduled_admin_bind: Option<String>,
    /// 🎛️ If true, only process messages starting with "/" (slash commands).
    /// If false, process ALL messages and pass them to the target CLI.
    #[serde(default = "default_cli_only")]
    pub cli_only: bool,
}

fn default_cli_only() -> bool {
    true
}

impl Config {
    /// Parse channel_ids into u64 values. Invalid entries are ignored.
    /// Returns empty Vec if wildcard "*" is present.
    pub fn channel_ids_u64(&self) -> Vec<u64> {
        if self.channel_ids.iter().any(|s| s == "*") {
            return Vec::new();
        }
        self.channel_ids
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    }

    /// Check if this config accepts all channels (wildcard mode).
    pub fn is_wildcard(&self) -> bool {
        self.channel_ids.iter().any(|s| s == "*")
    }

    /// Get default send channel IDs for the send command.
    pub fn default_send_channel_ids_u64(&self) -> Vec<u64> {
        self.channel_ids_u64()
    }

    /// Get the profile ID derived from the config file path.
    /// e.g. ~/.config/opencb/work/config.toml → "work"
    /// e.g. ~/.config/opencb/config.toml → "default"
    pub fn profile_id(&self) -> String {
        self.config_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_string()
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut targets = HashMap::new();
        targets.insert(
            "opencode".to_string(),
            TargetSpec {
                cmd: "opencode".to_string(),
                argv: vec!["run".to_string(), "#INPUT#".to_string()],
                work_dir: None,
            },
        );
        Self {
            config_path: PathBuf::new(),
            bot_token: "YOUR_BOT_TOKEN_HERE".to_string(),
            channel_ids: Vec::new(),
            owner_id: Vec::new(),
            debug: Some(false),
            targets,
            scheduled_admin_bind: Some("127.0.0.1:19001".to_string()),
            cli_only: true,
        }
    }
}

/// Parse channel_ids from a TOML value (array of strings, single string, or single integer).
fn parse_channel_ids(v: &toml::Value) -> Vec<String> {
    if let Some(arr) = v.as_array() {
        arr.iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect()
    } else if let Some(s) = v.as_str() {
        vec![s.to_string()]
    } else if let Some(n) = v.as_integer() {
        vec![n.to_string()]
    } else {
        Vec::new()
    }
}

/// Parse targets from a TOML table (scanning for tables with cmd + argv).
fn parse_targets(tbl: &toml::Value) -> Result<HashMap<String, TargetSpec>, Box<dyn std::error::Error>> {
    let mut targets = HashMap::new();
    if let toml::Value::Table(top) = tbl {
        for (key, val) in top {
            if let toml::Value::Table(inner) = val {
                if inner.contains_key("cmd") && inner.contains_key("argv") {
                    match inner.clone().try_into::<TargetSpec>() {
                        Ok(spec) => {
                            targets.insert(key.clone(), spec);
                        }
                        Err(e) => {
                            return Err(format!("Invalid target spec for [{}]: {}", key, e).into());
                        }
                    }
                }
            }
        }
    }
    Ok(targets)
}

/// Validate config fields: bot_token required, channel_ids non-empty.
fn validate_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if config.bot_token.is_empty() {
        return Err("bot_token is missing or empty".into());
    }
    if config.bot_token == "YOUR_BOT_TOKEN_HERE" {
        return Err("Please set your bot_token in config.toml".into());
    }
    if config.channel_ids.is_empty() {
        return Err("channel_ids must not be empty".into());
    }
    // Validate wildcard is alone
    if config.channel_ids.iter().any(|s| s == "*") && config.channel_ids.len() > 1 {
        return Err("channel_ids: '*' must be the only element when used".into());
    }
    // Validate default_send_to_channel_ids via channel_ids field
    Ok(())
}

/// Get the opencb config base directory (~/.config/opencb/)
fn opencb_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Cannot determine home directory ($HOME or $USERPROFILE)")?;
    Ok(PathBuf::from(home).join(".config").join("opencb"))
}

/// 📂 讀取 config.toml
///
/// 嘗試順序：
///   1. --config <path> → 直接用指定路徑
///   2. --profile <id> → ~/.config/opencb/<id>/config.toml
///      - 不存在 → 自動建立目錄、複製預設檔、提示用戶、return error
///   3. (無參數) → ~/.config/opencb/default/config.toml
pub fn load_config(
    config_path: Option<&str>,
    profile: Option<&str>,
) -> Result<Config, Box<dyn std::error::Error>> {
    let path = if let Some(p) = config_path {
        // --config 旗標：直接用指定路徑
        let p = Path::new(p);
        if p.is_relative() {
            std::env::current_dir()?.join(p)
        } else {
            p.to_path_buf()
        }
    } else if let Some(profile_id) = profile {
        // --profile 旗標：去 ~/.config/opencb/<profile_id>/config.toml
        let base = opencb_config_dir()?;
        let profile_path = base.join(profile_id).join("config.toml");

        if !profile_path.exists() {
            // 建立 profile 目錄並複製預設檔
            let default_config = base.join("default").join("config.toml");
            if !default_config.exists() {
                // default profile 冇，先建立
                if let Some(parent) = default_config.parent() {
                    fs::create_dir_all(parent)?;
                }
                let toml_str = render_default_toml();
                fs::write(&default_config, &toml_str)?;
            }

            if let Some(parent) = profile_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&default_config, &profile_path)?;

            return Err(format!(
                "Profile '{}' config created at:\n  {}\n\nPlease edit this file with your settings, then run the command again.",
                profile_id,
                profile_path.display()
            ).into());
        }

        profile_path
    } else {
        // 無參數：用 default profile
        let base = opencb_config_dir()?;
        base.join("default").join("config.toml")
    };

    // 檔案唔存在 → 自動建立預設
    if !path.exists() {
        if config_path.is_some() {
            return Err(format!("Config file '{}' does not exist", path.display()).into());
        }
        // Default profile 唔存在 → 啟動互動設定
        if profile.is_none() {
            eprintln!("Default profile not found. Launching interactive setup...");
            let profile_id = "default".to_string();
            match profile_manager::add_profile(
                &profile_id,
                None, None, None, None,
            ) {
                Ok(setup_config) => {
                    return Ok(setup_config);
                }
                Err(e) => {
                    eprintln!("Setup cancelled or failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        // --profile specified but dir missing → create dir + copy default + error
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = render_default_toml();
        fs::write(&path, toml_str)?;
        return Err(format!(
            "Config file '{}' not found. Default config created. Please fill in your values before running again.",
            path.display()
        ).into());
    }

    let metadata = fs::metadata(&path)?;
    if metadata.permissions().readonly() {
        return Err(format!(
            "Config file '{}' is not readable (read-only or no permission)",
            path.display()
        ).into());
    }

    let content = fs::read_to_string(&path)?;
    let raw: toml::Value = toml::from_str(&content)?;

    // 解析扁平 config 欄位
    let bot_token = raw
        .get("bot_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let channel_ids = match raw.get("channel_ids") {
        Some(v) => parse_channel_ids(v),
        None => Vec::new(),
    };

    let owner_id: Vec<String> = match raw.get("owner_id") {
        Some(v) => {
            if let Some(arr) = v.as_array() {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            } else if let Some(s) = v.as_str() {
                vec![s.to_string()]
            } else {
                Vec::new()
            }
        }
        None => Vec::new(),
    };

    let debug = raw.get("debug").and_then(|v| v.as_bool());
    let cli_only = raw.get("cli_only").and_then(|v| v.as_bool()).unwrap_or(true);
    let scheduled_admin_bind = raw
        .get("scheduled_admin_bind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let targets = parse_targets(&raw)?;

    let mut config = Config {
        config_path: path.clone(),
        bot_token,
        channel_ids,
        owner_id,
        debug,
        targets,
        scheduled_admin_bind,
        cli_only,
    };

    // 如果 channel_ids 為空，預設為 wildcard
    if config.channel_ids.is_empty() {
        config.channel_ids = vec!["*".to_string()];
    }

    // Auto-setup: if this is the default profile and key fields are incomplete, launch interactive setup
    if config_path.is_none() && profile.is_none() {
        let needs_setup = config.bot_token.is_empty()
            || config.bot_token == "YOUR_BOT_TOKEN_HERE"
            || config.channel_ids.iter().all(|s| s == "*");

        if needs_setup {
            eprintln!("Default profile is not configured. Launching interactive setup...");
            let profile_id = config.profile_id();
            match profile_manager::add_profile(
                &profile_id,
                None, // no prefill — force interactive prompts
                None,
                None,
                None,
            ) {
                Ok(setup_config) => {
                    return Ok(setup_config);
                }
                Err(e) => {
                    eprintln!("Setup cancelled or failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    validate_config(&config)?;

    Ok(config)
}

/// 列出所有可用嘅 profile（掃描 ~/.config/opencb/*/config.toml）
pub fn list_profiles(config_path: Option<&str>) -> Result<Vec<(String, PathBuf)>, Box<dyn std::error::Error>> {
    let base = if let Some(p) = config_path {
        let p = Path::new(p);
        if p.is_relative() {
            std::env::current_dir()?.join(p)
        } else {
            p.to_path_buf()
        }
    } else {
        opencb_config_dir()?
    };

    let base_dir = if base.is_file() {
        base.parent().unwrap_or(&base).to_path_buf()
    } else {
        base
    };

    let mut profiles = Vec::new();

    // 掃描子目錄（包括 default）
    if base_dir.is_dir() {
        let entries = fs::read_dir(&base_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let config_file = path.join("config.toml");
                if config_file.exists() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    profiles.push((name, config_file));
                }
            }
        }
    }

    profiles.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(profiles)
}

/// 🖊️ 整出預設 config.toml 內容（扁平格式）
fn render_default_toml() -> String {
    let mut s = String::new();
    s.push_str("# 🤖 OpenCB Config\n");
    s.push_str("# Edit this file with your settings.\n");
    s.push('\n');
    s.push_str("bot_token = \"YOUR_BOT_TOKEN_HERE\"\n");
    s.push_str("channel_ids = [\"*\"]  # or specific IDs: [\"123\", \"456\"]\n");
    s.push_str("cli_only = true  # true: only /commands, false: all messages\n");
    s.push('\n');
    s.push_str("# Target CLIs\n");
    s.push_str("[target.opencode]\n");
    s.push_str("cmd = \"opencode\"\n");
    s.push_str("argv = [\"run\", \"#INPUT#\"]\n");
    s.push_str("# work_dir = \"/tmp\"\n");
    s
}
