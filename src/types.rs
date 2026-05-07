//! 📊 訊息元數據類型定義
//! 呢個模組包含所有用嚟序列化 Discord 訊息嘅 struct ✨

use serde::Serialize;

// 📨 訊息元數據主體 - 包晒成條訊息嘅資料
#[derive(Serialize)]
pub struct MessageMetadata {
    pub id: String,
    pub content: String,
    pub created_at: Option<String>,
    pub author: AuthorMetadata,
    pub channel: ChannelMetadata,
    pub guild: Option<GuildMetadata>,
    pub mentions: MentionsMetadata,
    pub attachments: Vec<AttachmentMetadata>,
    pub embeds_count: usize,
    pub pinned: bool,
    pub webhook_id: Option<String>,
}

// 👤 作者資料 - 邊個發嘅
#[derive(Serialize)]
pub struct AuthorMetadata {
    pub id: String,
    pub name: String,
    pub bot: bool,
}

// 📡 頻道資料 - 喺邊度發嘅
#[derive(Serialize)]
pub struct ChannelMetadata {
    pub id: String,
    pub name: Option<String>,
    pub channel_type: String,
}

// 🏰 伺服器資料 - 邊個 guild
#[derive(Serialize)]
pub struct GuildMetadata {
    pub id: String,
    pub name: String,
}

// 🔔 提及資料 - tag 咗邊個
#[derive(Serialize)]
pub struct MentionsMetadata {
    pub users: Vec<UserMention>,
    pub everyone: bool,
}

// 👥 用戶提及 - 個別被 tag 嘅人
#[derive(Serialize)]
pub struct UserMention {
    pub id: String,
    pub name: String,
}

// 📎 附件資料 - 跟住訊息嘅檔案
#[derive(Serialize)]
pub struct AttachmentMetadata {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub url: String,
}
