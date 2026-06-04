//! ⚙️ 配置處理模組
//! 負責讀取同驗證 config.toml 配置檔 ✨

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Regex for validating profile names and command names: ^[a-z0-9_-]+$
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

/// 🎯 Target CLI 規格（用嚟描述要叫邊個外部 CLI）
/// 例如 [opencode] cmd="opencode" argv=["run","#INPUT#"] work_dir="/tmp"
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

/// 🎯 Bot Profile 配置結構體（對應 [profiles.<name>] table）
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Profile {
    /// Profile ID（須同 TOML table key 一致）
    pub profile_id: String,
    /// Channel type（預設 "discord"）
    #[serde(default = "default_channel_type")]
    pub channel_type: String,
    /// 目標頻道 ID 列表，支援 ["*"] 表示接受所有頻道
    pub channel_ids: Vec<String>,
    /// Discord bot token
    pub bot_token: String,
    /// 🎯 用嚟 send 嘅預設頻道 ID 列表（唔可以係 "*" 或空）
    #[serde(default)]
    pub default_send_to_channel_ids: Vec<String>,
    /// Profile 獨立嘅 Target CLI map
    #[serde(default, skip)]
    pub targets: HashMap<String, TargetSpec>,
}

fn default_channel_type() -> String {
    "discord".to_string()
}

impl Profile {
    /// Parse channel_ids into u64 values. Invalid entries are ignored.
    /// Returns empty Vec if wildcard "*" is present.
    #[allow(dead_code)]
    pub fn channel_ids_u64(&self) -> Vec<u64> {
        if self.channel_ids.iter().any(|s| s == "*") {
            return Vec::new();
        }
        self.channel_ids
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    }

    /// Check if this profile accepts all channels (wildcard mode).
    pub fn is_wildcard(&self) -> bool {
        self.channel_ids.iter().any(|s| s == "*")
    }

    /// Get default send channel IDs for the send command.
    /// Returns parsed u64 IDs if configured, empty Vec otherwise.
    pub fn default_send_channel_ids_u64(&self) -> Vec<u64> {
        self.default_send_to_channel_ids
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    }
}

/// 🤖 Bot 配置結構體（對應 config.toml 嘅欄位）
#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    /// Profiles map（key 係 profile name，例如 "default"）
    #[serde(default, skip)]
    pub profiles: HashMap<String, Profile>,
    /// Discord bot token（fallback，向後兼容舊格式）
    pub bot_token: String,
    /// 目標頻道嘅 channel ID 列表（fallback，向後兼容舊格式）
    pub channel_id: Vec<String>,
    /// Bot owner 嘅 user ID 列表（字串陣列）
    pub owner_id: Vec<String>,
    /// 係咪開 debug 模式（可選）。預設 false
    pub debug: Option<bool>,
    /// 🎯 Target CLI map（fallback，向後兼容舊格式）
    #[serde(default, skip)]
    #[allow(dead_code)]
    pub targets: HashMap<String, TargetSpec>,
    /// Admin HTTP server bind address for scheduling admin endpoint (e.g. "127.0.0.1:19001")
    #[serde(default)]
    pub scheduled_admin_bind: Option<String>,
}

impl Config {
    /// Try to parse channel_id strings into u64 values. Invalid entries are ignored.
    #[allow(dead_code)]
    pub fn channel_ids_u64(&self) -> Vec<u64> {
        self.channel_id
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    }
}

impl Default for Config {
    /// 🆕 生成預設 config，畀用戶填返自己嘅資料
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
            profiles: HashMap::new(),
            bot_token: "YOUR_BOT_TOKEN_HERE".to_string(),
            channel_id: Vec::new(),
            owner_id: Vec::new(),
            debug: Some(false),
            targets,
            scheduled_admin_bind: Some("127.0.0.1:19001".to_string()),
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

/// Validate a profile's fields: bot_token required, channel_ids non-empty.
fn validate_profile(name: &str, profile: &Profile) -> Result<(), Box<dyn std::error::Error>> {
    if !is_valid_name(name) {
        return Err(format!(
            "Invalid profile name '{}' — must match [a-z0-9_-]+",
            name
        ).into());
    }
    if profile.profile_id != name {
        return Err(format!(
            "Profile table key '{}' does not match profile_id '{}'",
            name, profile.profile_id
        ).into());
    }
    if profile.bot_token.is_empty() {
        return Err(format!("Profile '{}' is missing bot_token", name).into());
    }
    if profile.bot_token == "YOUR_BOT_TOKEN_HERE" {
        return Err(format!(
            "Profile '{}' has placeholder bot_token — please set a real token",
            name
        ).into());
    }
    if profile.channel_ids.is_empty() {
        return Err(format!("Profile '{}' channel_ids must not be empty", name).into());
    }
    // Validate wildcard is alone
    if profile.channel_ids.iter().any(|s| s == "*") && profile.channel_ids.len() > 1 {
        return Err(format!(
            "Profile '{}' channel_ids: '*' must be the only element when used",
            name
        ).into());
    }
    // Validate default_send_to_channel_ids: must not contain "*" or be empty if present
    if !profile.default_send_to_channel_ids.is_empty() {
        if profile.default_send_to_channel_ids.iter().any(|s| s == "*") {
            return Err(format!(
                "Profile '{}' default_send_to_channel_ids must not contain '*'",
                name
            ).into());
        }
        for id in &profile.default_send_to_channel_ids {
            if id.parse::<u64>().is_err() {
                return Err(format!(
                    "Profile '{}' default_send_to_channel_ids contains invalid ID: '{}'",
                    name, id
                ).into());
            }
        }
    }
    Ok(())
}

/// 📂 讀取 config.toml，唔存在就創建預設檔
pub fn load_config(config_path: Option<&str>) -> Result<Config, Box<dyn std::error::Error>> {
    let path = match config_path {
        Some(p) => {
            let p = Path::new(p);
            if p.is_relative() {
                std::env::current_dir()?.join(p)
            } else {
                p.to_path_buf()
            }
        }
        None => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| "Cannot determine home directory ($HOME or $USERPROFILE)")?;
            PathBuf::from(home).join(".config").join("opencb").join("config.toml")
        }
    };

    if !path.exists() {
        if config_path.is_none() {
            let default_config = Config::default();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_str = render_default_toml(&default_config);
            fs::write(&path, toml_str)?;
            return Err(format!(
                "Config file '{}' not found. Default config created. Please fill in your values before running again.",
                path.display()
            ).into());
        } else {
            return Err(format!("Config file '{}' does not exist", path.display()).into());
        }
    }

    let metadata = fs::metadata(&path)?;
    if metadata.permissions().readonly() {
        return Err(format!(
            "Config file '{}' is not readable (read-only or no permission)",
            path.display()
        )
        .into());
    }

    let content = fs::read_to_string(&path)?;
    let raw: toml::Value = toml::from_str(&content)?;

    // Parse top-level fields (fallback / legacy format)
    let bot_token = raw
        .get("bot_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let channel_id_vec = match raw.get("channel_id") {
        Some(v) => parse_channel_ids(v),
        None => Vec::new(),
    };

    let owner_id_vec: Vec<String> = match raw.get("owner_id") {
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
    let scheduled_admin_bind = raw
        .get("scheduled_admin_bind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // 🔍 Check for [profiles] section
    let has_profiles = raw.get("profiles").and_then(|v| v.as_table()).is_some();

    let mut profiles = HashMap::new();

    if has_profiles {
        // Parse [profiles.<name>] tables
        if let Some(profiles_tbl) = raw.get("profiles").and_then(|v| v.as_table()) {
            for (name, val) in profiles_tbl {
                if let toml::Value::Table(tbl) = val {
                    let profile_id = tbl
                        .get("profile_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(name)
                        .to_string();
                    let channel_type = tbl
                        .get("channel_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("discord")
                        .to_string();
                    let channel_ids = match tbl.get("channel_ids") {
                        Some(v) => parse_channel_ids(v),
                        None => Vec::new(),
                    };
                    let profile_bot_token = tbl
                        .get("bot_token")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    // Parse default_send_to_channel_ids from the profile's table
                    let default_send_to_channel_ids = match tbl.get("default_send_to_channel_ids") {
                        Some(v) => parse_channel_ids(v),
                        None => Vec::new(),
                    };

                    // Parse targets from the profile's table
                    let targets = parse_targets(&toml::Value::Table(tbl.clone()))?;

                    let profile = Profile {
                        profile_id: profile_id.clone(),
                        channel_type,
                        channel_ids,
                        bot_token: profile_bot_token,
                        default_send_to_channel_ids,
                        targets,
                    };

                    validate_profile(name, &profile)?;
                    profiles.insert(name.clone(), profile);
                }
            }
        }

        Ok(Config {
            profiles,
            bot_token,
            channel_id: channel_id_vec,
            owner_id: owner_id_vec,
            debug,
            targets: HashMap::new(),
            scheduled_admin_bind,
        })
    } else {
        // Fallback: build synthetic "default" profile from top-level fields
        if bot_token.is_empty() {
            return Err("bot_token missing or invalid in config.toml".into());
        }
        if bot_token == "YOUR_BOT_TOKEN_HERE" {
            return Err("Please set your BOT_TOKEN in config.toml".into());
        }

        let fallback_targets = parse_targets(&raw)?;

        let default_profile = Profile {
            profile_id: "default".to_string(),
            channel_type: "discord".to_string(),
            channel_ids: if channel_id_vec.is_empty() {
                vec!["*".to_string()]
            } else {
                channel_id_vec.clone()
            },
            bot_token: bot_token.clone(),
            default_send_to_channel_ids: Vec::new(),
            targets: fallback_targets.clone(),
        };

        profiles.insert("default".to_string(), default_profile);

        Ok(Config {
            profiles,
            bot_token,
            channel_id: channel_id_vec,
            owner_id: owner_id_vec,
            debug,
            targets: fallback_targets,
            scheduled_admin_bind,
        })
    }
}

/// 🖊️ 整出預設 config.toml 內容（profiles 格式）
fn render_default_toml(_cfg: &Config) -> String {
    let mut s = String::new();
    s.push_str("# 🤖 OpenCB 配置檔 - 請填入你嘅資料\n");
    s.push_str("debug = false\n");
    s.push('\n');
    s.push_str("# ── Profiles ──\n");
    s.push_str("[profiles.default]\n");
    s.push_str("profile_id = \"default\"\n");
    s.push_str("channel_type = \"discord\"\n");
    s.push_str("channel_ids = [\"*\"]  # or specific IDs: [\"123\", \"456\"]\n");
    s.push_str("bot_token = \"YOUR_BOT_TOKEN_HERE\"\n");
    s.push('\n');
    s.push_str("[profiles.default.targets.opencode]\n");
    s.push_str("cmd = \"opencode\"\n");
    s.push_str("argv = [\"run\", \"#INPUT#\"]\n");
    s.push_str("# work_dir = \"/tmp\"\n");
    s
}
