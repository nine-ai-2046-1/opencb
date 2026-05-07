//! ⚙️ 配置處理模組
//! 負責讀取同驗證 config.toml 配置檔 ✨

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

/// 🤖 Bot 配置結構體（對應 config.toml 嘅欄位）
#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    /// Discord bot token，去 Developer Portal 攞
    pub bot_token: String,
    /// 目標頻道嘅 channel ID
    pub channel_id: u64,
    /// Bot owner 嘅 user ID（可選）
    pub owner_id: Option<String>,
    /// 係咪開 debug 模式（可選）
    pub debug: Option<bool>,
    /// 🎯 Target CLI map（key 係 target 名，例如 "opencode"）
    /// ⚠️ 唔會直接由 serde derive 解析，係由 load_config() 額外掃 TOML table
    #[serde(default, skip)]
    pub targets: HashMap<String, TargetSpec>,
}

impl Default for Config {
    /// 🆕 生成預設 config，畀用戶填返自己嘅資料
    fn default() -> Self {
        let mut targets = HashMap::new();
        // 📌 預設範例：opencode target，用戶可以照住改
        targets.insert(
            "opencode".to_string(),
            TargetSpec {
                // 預設用 opencode CLI，argv 示範用 run "#INPUT#"
                cmd: "opencode".to_string(),
                argv: vec!["run".to_string(), "#INPUT#".to_string()],
                work_dir: None,
            },
        );
        Self {
            bot_token: "YOUR_BOT_TOKEN_HERE".to_string(),
            channel_id: 123456789012345678,
            owner_id: None,
            debug: Some(true),
            targets,
        }
    }
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
        None => std::env::current_dir()?.join("config.toml"),
    };

    if !path.exists() {
        if config_path.is_none() {
            let default_config = Config::default();
            // 🖊️ 手動拼出預設 config 文字（包含 [opencode] 範例 + 註解）
            let toml_str = render_default_toml(&default_config);
            fs::write(&path, toml_str)?;
            return Err(format!(
                "Config file '{}' not found. Default config created. Please fill in your values before running again.",
                path.display()
            ).into());
        } else {
            return Err(format!(
                "Config file '{}' does not exist",
                path.display()
            ).into());
        }
    }

    let metadata = fs::metadata(&path)?;
    if metadata.permissions().readonly() {
        return Err(format!(
            "Config file '{}' is not readable (read-only or no permission)",
            path.display()
        ).into());
    }

    let content = fs::read_to_string(&path)?;
    let mut config: Config = toml::from_str(&content)?;

    if config.bot_token == "YOUR_BOT_TOKEN_HERE" {
        return Err("Please set your BOT_TOKEN in config.toml".into());
    }

    // 🔍 額外掃 TOML 入面所有 top-level table，凡係有 cmd + argv 嘅就當 TargetSpec
    let raw: toml::Value = toml::from_str(&content)?;
    if let toml::Value::Table(tbl) = raw {
        for (key, val) in tbl {
            if let toml::Value::Table(inner) = val {
                if inner.contains_key("cmd") && inner.contains_key("argv") {
                    match inner.try_into::<TargetSpec>() {
                        Ok(spec) => {
                            config.targets.insert(key, spec);
                        }
                        Err(e) => {
                            return Err(format!(
                                "Invalid target spec for [{}]: {}",
                                key, e
                            ).into());
                        }
                    }
                }
            }
        }
    }

    Ok(config)
}

/// 🖊️ 整出預設 config.toml 內容（含粵語註解 + opencode 範例）
fn render_default_toml(cfg: &Config) -> String {
    let mut s = String::new();
    s.push_str("# 🤖 OpenCB 配置檔 - 請填入你嘅資料\n");
    s.push_str(&format!("bot_token = \"{}\"\n", cfg.bot_token));
    s.push_str(&format!("channel_id = {}\n", cfg.channel_id));
    s.push_str("# owner_id = \"123456789\"\n");
    s.push_str(&format!("debug = {}\n", cfg.debug.unwrap_or(true)));
    s.push_str("\n");
    s.push_str("# 🎯 Target CLI 範例（可以加多個 [xxx] table）\n");
    s.push_str("# 收到 message 時會用 cmd + argv 執行，#INPUT# 會被訊息內容取代\n");
    s.push_str("[opencode]\n");
    s.push_str("cmd = \"opencode\"\n");
    s.push_str("argv = [\"run\", \"#INPUT#\"]\n");
    s.push_str("# work_dir = \"/tmp\"\n");
    s
}
