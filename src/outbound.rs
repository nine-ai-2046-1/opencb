//! 📤 出站訊息模組
//! 負責通過 Discord Context 發送訊息到指定頻道 ✨

use serenity::all::{ChannelId, Context, CreateMessage, Message};
use tracing::error;

pub async fn send_message_to_channel(
    ctx: &Context,
    channel_id: u64,
    content: &str,
) -> Option<Message> {
    let channel_id = ChannelId::new(channel_id);

    match channel_id
        .send_message(&ctx.http, CreateMessage::new().content(content))
        .await
    {
        Ok(msg) => Some(msg),
        Err(e) => {
            error!("Failed to send message to {}: {:?}", channel_id, e);
            None
        }
    }
}
