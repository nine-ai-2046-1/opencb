//! 🤖 Discord 事件處理模組
//! 實作 EventHandler trait，處理訊息同就緒事件 ✨

use serenity::all::{
    Context, CreateInteractionResponse, CreateInteractionResponseMessage,
    EventHandler, Interaction, Message, Ready,
};
use serenity::async_trait;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::inbound::extract_message_metadata;
use crate::outbound::send_message_to_channel;
use crate::slash_commands::{self, CommandContext, ResponseHandle};

/// 🤖 Serve 模式 handler
/// 內含 config（用嚟攞 bot_token, channel_ids, targets）
pub struct ServeHandler {
    pub config: Config,
}

/// Validate command name matches ^[a-z0-9_-]+$
pub fn is_valid_command_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

/// Build a CommandContext from a text-based message and parsed args.
fn build_context_from_message(msg_metadata: crate::types::MessageMetadata, args: &str) -> CommandContext {
    CommandContext {
        args: args.to_string(),
        message: msg_metadata,
    }
}

/// Send an ephemeral "Invalid command" response to an interaction.
async fn respond_invalid_command(ctx: &Context, interaction: &serenity::all::CommandInteraction) {
    let resp = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content("Invalid command"),
    );
    let _ = interaction.create_response(&ctx.http, resp).await;
}

#[async_trait]
impl EventHandler for ServeHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // 🚫 唔處理自己發嘅訊息，避免無限循環
        let current_user_id = ctx.cache.current_user().id;
        if msg.author.id == current_user_id {
            info!("🚫 Ignored: message from bot itself (id={})", msg.id);
            return;
        }

        // 📡 Channel filtering: DM always accepted; Guild check channel_ids
        if msg.guild_id.is_some() {
            // Guild message — check channel_ids unless wildcard
            if !self.config.is_wildcard() {
                let channel_id_str = msg.channel_id.get().to_string();
                if !self.config.channel_ids.contains(&channel_id_str) {
                    info!(
                        "🚫 Ignored: channel {} not in allowed list {:?} (msg={})",
                        channel_id_str, self.config.channel_ids, msg.id
                    );
                    return;
                }
            }
        }

        // 📝 Extract metadata from the message
        let metadata = extract_message_metadata(&ctx, &msg);

        // 📝 Output JSON metadata to stdout
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

        // 🔍 Only process messages starting with "/" when cli_only is true
        let content = metadata.content.clone();
        let content = content.trim();
        if self.config.cli_only && !content.starts_with('/') {
            info!(
                "🚫 Ignored: message without '/' prefix (msg={}, author={})",
                msg.id, metadata.author.name
            );
            return;
        }

        // 🔍 Validate command name format
        let after_slash = &content[1..]; // remove leading '/'
        let (cmd_name, args) = match after_slash.split_once(char::is_whitespace) {
            Some((name, rest)) => (name, rest.trim()),
            None => (after_slash, ""),
        };

        if !is_valid_command_name(cmd_name) {
            info!("🚫 Invalid command name format: '{}'", cmd_name);
            let _ = send_message_to_channel(&ctx, msg.channel_id.get(), "Invalid command").await;
            return;
        }

        // 🔎 Find and execute command with context
        match slash_commands::find(cmd_name) {
            Some(command) => {
                let cmd_ctx = build_context_from_message(metadata, args);
                let output = command.execute(&cmd_ctx).await;
                if output.is_empty() {
                    warn!("⚠️ Slash command '{}' returned empty output", cmd_name);
                    return;
                }
                let channel_id = msg.channel_id.get();
                info!("✅ Slash command '{}' executed, replying to channel {}", cmd_name, channel_id);
                send_message_to_channel(&ctx, channel_id, &output).await;
            }
            None => {
                info!("🚫 Unknown slash command: '{}'", cmd_name);
                let _ = send_message_to_channel(&ctx, msg.channel_id.get(), "Invalid command").await;
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // Only handle chat input (slash) commands
        let command = match interaction {
            Interaction::Command(cmd) => cmd,
            _ => return,
        };

        let cmd_name = command.data.name.clone();
        info!("📨 Native slash command interaction: /{} from user {}", cmd_name, command.user.id);

        // Validate command name format
        if !is_valid_command_name(&cmd_name) {
            info!("🚫 Invalid command name format: '{}'", cmd_name);
            respond_invalid_command(&ctx, &command).await;
            return;
        }

        // Build args string from interaction options
        let args = command
            .data
            .options
            .iter()
            .filter_map(|opt| match &opt.value {
                serenity::all::CommandDataOptionValue::String(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");

        // 📝 Print message body to console — interaction accepted, not rejected
        println!("\n========== 📨 Slash Command Interaction ==========");
        println!("command : /{}", cmd_name);
        println!("text    : {}", args);
        println!("user    : {} ({})", command.user.name, command.user.id);
        println!("channel : {}", command.channel_id);
        if let Some(gid) = command.guild_id {
            println!("guild   : {}", gid);
        }
        println!("==================================================\n");

        // Find and execute command
        match slash_commands::find(&cmd_name) {
            Some(command_impl) => {
                // Build minimal MessageMetadata for context
                let msg_metadata = crate::types::MessageMetadata {
                    id: command.id.to_string(),
                    content: format!("/{} {}", cmd_name, args),
                    created_at: None,
                    author: crate::types::AuthorMetadata {
                        id: command.user.id.to_string(),
                        name: command.user.name.clone(),
                        bot: false,
                    },
                    channel: crate::types::ChannelMetadata {
                        id: command.channel_id.to_string(),
                        name: None,
                        channel_type: "Unknown".to_string(),
                    },
                    guild: command.guild_id.map(|g| crate::types::GuildMetadata {
                        id: g.to_string(),
                        name: "Unknown".to_string(),
                    }),
                    mentions: crate::types::MentionsMetadata {
                        users: vec![],
                        everyone: false,
                    },
                    attachments: vec![],
                    embeds_count: 0,
                    pinned: false,
                    webhook_id: None,
                };

                let cmd_ctx = CommandContext {
                    args: args.clone(),
                    message: msg_metadata,
                };

                // 5.1 Defer the response immediately so Discord shows "thinking..."
                // This avoids the 3-second interaction timeout for long-running commands.
                let defer_resp = CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new(),
                );
                if let Err(e) = command.create_response(&ctx.http, defer_resp).await {
                    error!("❌ Failed to defer interaction for /{}: {}", cmd_name, e);
                    return;
                }

                // 5.2 Build ResponseHandle so the command can push updates to Discord
                let handle = ResponseHandle {
                    http: Arc::clone(&ctx.http),
                    application_id: command.application_id,
                    interaction_token: command.token.clone(),
                };

                // 5.3 Execute with streaming update support (replaces old create_response)
                // 5.4 The old create_response call is removed — ResponseHandle handles all edits
                info!("▶️  Dispatching /{} via execute_with_updates", cmd_name);
                command_impl.execute_with_updates(&cmd_ctx, &handle).await;
                info!("✅ Slash command '{}' interaction complete", cmd_name);
            }
            None => {
                info!("🚫 Unknown slash command: '{}'", cmd_name);
                respond_invalid_command(&ctx, &command).await;
            }
        }
    }

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        info!("✅ Bot is ready! Logged in as {}", data_about_bot.user.name);
        info!("🎯 Active profile: {}", self.config.profile_id());

        // Register slash commands with Discord API
        slash_commands::register_all_commands(&ctx.http, data_about_bot.user.id).await;
    }
}
