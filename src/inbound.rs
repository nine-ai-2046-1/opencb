//! 📥 入站訊息模組
//! 負責從 Discord Message 提取元數據並轉換成結構化格式 ✨

use serenity::all::{Context, Message};

use crate::types::{
    AttachmentMetadata, AuthorMetadata, ChannelMetadata, GuildMetadata, MentionsMetadata,
    MessageMetadata, UserMention,
};

pub fn extract_message_metadata(ctx: &Context, msg: &Message) -> MessageMetadata {
    let author = AuthorMetadata {
        id: msg.author.id.to_string(),
        name: msg.author.name.clone(),
        bot: msg.author.bot,
    };

    let channel = ChannelMetadata {
        id: msg.channel_id.to_string(),
        name: msg.guild_id.and_then(|guild_id| {
            ctx.cache.guild(guild_id).and_then(|guild| {
                guild
                    .channels
                    .get(&msg.channel_id)
                    .map(|channel| channel.name.clone())
            })
        }),
        channel_type: msg
            .guild_id
            .and_then(|guild_id| ctx.cache.guild(guild_id))
            .and_then(|guild| {
                guild
                    .channels
                    .get(&msg.channel_id)
                    .map(|channel| format!("{:?}", channel.kind))
            })
            .unwrap_or_else(|| "Unknown".to_string()),
    };

    let guild = msg.guild_id.and_then(|gid| {
        ctx.cache.guild(gid).map(|g| GuildMetadata {
            id: g.id.to_string(),
            name: g.name.clone(),
        })
    });

    let users = msg
        .mentions
        .iter()
        .map(|u| UserMention {
            id: u.id.to_string(),
            name: u.name.clone(),
        })
        .collect();

    let attachments = msg
        .attachments
        .iter()
        .map(|a| AttachmentMetadata {
            id: a.id.to_string(),
            filename: a.filename.clone(),
            size: a.size as u64,
            url: a.url.clone(),
        })
        .collect();

    MessageMetadata {
        id: msg.id.to_string(),
        content: msg.content.clone(),
        created_at: msg.timestamp.to_rfc3339(),
        author,
        channel,
        guild,
        mentions: MentionsMetadata {
            users,
            everyone: msg.mention_everyone,
        },
        attachments,
        embeds_count: msg.embeds.len(),
        pinned: msg.pinned,
        webhook_id: msg.webhook_id.map(|id| id.to_string()),
    }
}
