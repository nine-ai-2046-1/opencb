//! 🤖 Discord 事件處理模組
//! 實作 EventHandler trait，處理訊息同就緒事件 ✨

use serenity::all::{Context, EventHandler, Message, Ready};
use serenity::async_trait;
use std::env;
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::config::{Config, TargetSpec};
use crate::inbound::extract_message_metadata;
use crate::outbound::send_message_to_channel;
use crate::types::MessageMetadata;

/// 📏 Discord 單則訊息上限係 2000 字元，留少少 buffer
const DISCORD_MSG_LIMIT: usize = 1900;

/// 🤖 Serve 模式 handler
/// 內含 config（用嚟攞 targets）同 target name（CLI 啟動時指定）
pub struct ServeHandler {
    pub config: Config,
    pub target: Option<String>,
}

#[async_trait]
impl EventHandler for ServeHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // 🚫 唔處理自己發嘅訊息，避免無限循環
        let current_user_id = ctx.cache.current_user().id;
        if msg.author.id == current_user_id {
            return;
        }

        // ➕ 只處理有提及 bot id 嘅 Guild 訊息；若冇提及就忽略
        // For guild messages require an explicit mention of the bot.
        if msg.guild_id.is_some() {
            let mentioned_bot = msg.mentions.iter().any(|u| u.id == current_user_id);
            if !mentioned_bot {
                // Not mentioning the bot -> ignore
                return;
            }
        } else {
            // 🔒 Direct message (DM) path: verify we can open a DM channel with the user
            // If we cannot create/open a DM channel (e.g. blocked or privacy settings), skip processing.
            if let Err(e) = msg.author.create_dm_channel(&ctx.http).await {
                warn!(
                    "⚠️ Cannot open DM channel with user {}: {}",
                    msg.author.id, e
                );
                return;
            }
        }

        // Extract metadata and then strip the explicit bot mention from the message content
        let mut metadata: MessageMetadata = extract_message_metadata(&ctx, &msg);

        // Remove explicit mention tokens for this bot: <@123...> and <@!123...>
        let bot_id_str = current_user_id.to_string();
        metadata.content = metadata
            .content
            .replace(&format!("<@{}>", bot_id_str), "")
            .replace(&format!("<@!{}>", bot_id_str), "");

        // Normalize whitespace (collapse multiple spaces/newlines into single spaces, trim ends)
        metadata.content = metadata
            .content
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        // 📝 照常輸出 JSON metadata 到 stdout
        match serde_json::to_string_pretty(&metadata) {
            Ok(json) => {
                println!("\n========== 📨 New Message ==========");
                println!("{}", json);
                println!("====================================\n");
            }
            Err(e) => {
                error!("❌ Failed to serialize message metadata: {}", e);
            }
        }

        // 🎯 如果有指定 target，就調用外部 CLI
        let target_name = match &self.target {
            Some(t) => t,
            None => return, // 冇 target 就保持原行為（淨係輸出 JSON）
        };

        let spec = match self.config.targets.get(target_name) {
            Some(s) => s.clone(),
            None => {
                error!(
                    "❌ Target '{}' 喺 config.toml 入面搵唔到，請檢查 [target] table",
                    target_name
                );
                return;
            }
        };

        let channel_id = msg.channel_id.get();
        let input = metadata.content.clone();
        let label = target_name.clone();

        // 🚀 喺 spawn 入面執行，避免阻塞 event loop
        tokio::spawn(async move {
            run_target_and_reply(ctx, channel_id, label, spec, input).await;
        });
    }

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        info!("✅ Bot is ready! Logged in as {}", data_about_bot.user.name);
        if let Some(t) = &self.target {
            info!("🎯 Active target: {}", t);
        }

        if let Ok(channel_id_str) = env::var("CHANNEL_ID") {
            // Accept comma-separated list or single ID; prefer first valid u64
            let first_valid = channel_id_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u64>().ok())
                .next();
            if let Some(channel_id) = first_valid {
                let test_message =
                    format!("🚀 Bot {} is online and ready!", data_about_bot.user.name);
                match send_message_to_channel(&ctx, channel_id, &test_message).await {
                    Some(_) => info!("✅ Startup message sent to channel {}", channel_id),
                    None => error!("❌ Failed to send startup message"),
                }
            } else {
                error!("❌ Invalid CHANNEL_ID format: {}", channel_id_str);
            }
        }
    }
}

/// 🛠️ 執行 target CLI 並將 stdout 回覆到 Discord channel
async fn run_target_and_reply(
    ctx: Context,
    channel_id: u64,
    target_label: String,
    spec: TargetSpec,
    input: String,
) {
    info!("🎯 Target '{}' 接收到訊息，準備執行 CLI", target_label);
    // 🔁 將 #INPUT# 取代成訊息內容
    let args: Vec<String> = spec
        .argv
        .iter()
        .map(|a| a.replace("#INPUT#", &input))
        .collect();

    info!(
        "🚀 執行 target CLI: cmd={} argv={:?} work_dir={:?}",
        spec.cmd, args, spec.work_dir
    );

    let mut cmd = Command::new(&spec.cmd);
    cmd.args(&args);
    if let Some(dir) = &spec.work_dir {
        cmd.current_dir(dir);
    }

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(e) => {
            error!("❌ 執行 target CLI 失敗: {}", e);
            let _ = send_message_to_channel(&ctx, channel_id, &format!("⚠️ 執行 CLI 失敗: {}", e))
                .await;
            return;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        error!(
            "❌ Target CLI 執行失敗 exit={:?} stderr={}",
            output.status.code(),
            stderr
        );
        let reply = format!(
            "⚠️ CLI exit={:?}\n```\n{}\n```",
            output.status.code(),
            truncate(&stderr, DISCORD_MSG_LIMIT - 40)
        );
        let _ = send_message_to_channel(&ctx, channel_id, &reply).await;
        return;
    }

    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        warn!("⚠️ Target CLI stdout 為空，唔回覆");
        return;
    }

    let reply = truncate(trimmed, DISCORD_MSG_LIMIT);
    info!(
        "✅ Target CLI 完成，回覆 {} 字元到 channel {}",
        reply.len(),
        channel_id
    );
    let _ = send_message_to_channel(&ctx, channel_id, &reply).await;
}

/// ✂️ 截斷字串到指定長度（按 char 邊界）
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    format!("{}…", &s[..end])
}
