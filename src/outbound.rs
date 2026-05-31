//! 📤 出站訊息模組
//! 負責通過 Discord Context 發送訊息到指定頻道 ✨

use crate::process_message_content;
use crate::splitter::send_split_message;
use serenity::all::{ChannelId, Context};

pub async fn send_message_to_channel(
    ctx: &Context,
    channel_id: u64,
    content: &str,
) {
    let channel_id = ChannelId::new(channel_id);
    let processed = process_message_content(content);
    send_split_message(&ctx.http, channel_id, &processed, 2000).await;
}
