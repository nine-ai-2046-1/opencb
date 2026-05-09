//! 🎯 命令行參數模組
//! 使用 clap 解析 CLI 參數，支援 serve 同 send 命令 ✨

use clap::{Parser, Subcommand};

/// 🚀 CLI 主結構，畀 main.rs 用 Cli::parse() 解析參數
#[derive(Parser, Debug)]
#[command(name = "opencb", version, about = "OpenCB Discord bot 🚀")]
pub struct Cli {
    /// 📝 配置文件路徑（全局參數，serve 同 send 都用得）
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<String>,

    /// 🎯 可選 target 名（例如 `opencb opencode`）
    /// 🔁 收到訊息時會用呢個 target 執行外部 CLI，再回覆 stdout
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// 🎮 子命令（serve 啟動服務，send 發送消息）
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 🎮 子命令枚舉，分 serve 同 send 兩種模式
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 🌐 啟動 Discord bot 服務
    Serve,

    /// 💬 發送一條消息（支援多個詞語自動拼接）
    Send {
        /// 💌 要發送嘅消息內容（至少一個詞）
        #[arg(num_args = 1.., required = true, trailing_var_arg = true)]
        message: Vec<String>,
        /// ⏱ 可選排程時間 HH:MM（例如 -t "10:15"）
        #[arg(short = 't', long = "time")]
        time: Option<String>,
        /// 📅 可選排程日期 YYYY-MM-DD（例如 -d "2026-12-11"）
        #[arg(short = 'd', long = "date")]
        date: Option<String>,
        /// 📣 覆蓋發送目標頻道，comma-separated list or single id (e.g. --rc "123,456")
        #[arg(long = "rc")]
        rc: Option<String>,
        /// 📨 覆蓋接收者 user id 列表（將以 DM 方式發送），comma-separated (e.g. --ru "111,222")
        #[arg(long = "ru")]
        ru: Option<String>,
        /// 🏷 要在訊息尾部 mention 嘅 user id 列表，comma-separated (e.g. --mu "111,222")
        #[arg(long = "mu")]
        mu: Option<String>,
    },
}
