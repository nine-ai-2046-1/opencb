//! 🎯 Slash Commands 模組
//! 提供命令註冊、查找同路由功能 ✨

use serenity::all::Http;
use serenity::all::UserId;
use serenity::builder::Builder;
use tracing::info;

use crate::types::MessageMetadata;

mod echo;

/// Context passed to every slash command execution.
/// Contains both the parsed args and the full message metadata.
pub struct CommandContext {
    /// The raw arguments string after the command name.
    pub args: String,
    /// Full message metadata (author, channel, guild, attachments, etc.).
    #[allow(dead_code)]
    pub message: MessageMetadata,
}

/// Trait that all slash commands must implement.
pub trait SlashCommand: Send + Sync {
    /// Returns the command name (without `/` prefix).
    fn name(&self) -> &str;

    /// Returns a human-readable description of what the command does.
    /// Used for Discord slash command registration.
    fn description(&self) -> &str;

    /// Execute the command with the given context.
    /// `ctx.args` contains the arguments text, `ctx.message` contains full metadata.
    fn execute(&self, ctx: &CommandContext) -> String;
}

/// Find a slash command by name.
/// Returns `Some(Box<dyn SlashCommand>)` if found, `None` otherwise.
pub fn find(command_name: &str) -> Option<Box<dyn SlashCommand>> {
    match command_name {
        "echo" => Some(Box::new(echo::EchoCommand)),
        // Future commands: register here
        _ => None,
    }
}

/// Returns all registered slash commands for Discord API registration.
pub fn all_commands() -> Vec<Box<dyn SlashCommand>> {
    vec![
        Box::new(echo::EchoCommand),
        // Future commands: add here
    ]
}

/// Register all slash commands with Discord's API.
/// Called once on bot startup from `ready()` event.
pub async fn register_all_commands(http: &Http, _app_id: UserId) {
    for cmd in all_commands() {
        let builder = serenity::builder::CreateCommand::new(cmd.name())
            .description(cmd.description());
        match builder.execute(http, (None, None)).await {
            Ok(_) => info!("✅ Registered slash command: /{}", cmd.name()),
            Err(e) => tracing::error!("❌ Failed to register /{}: {}", cmd.name(), e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AuthorMetadata, ChannelMetadata, MentionsMetadata};

    fn make_test_context(args: &str) -> CommandContext {
        CommandContext {
            args: args.to_string(),
            message: MessageMetadata {
                id: "123".to_string(),
                content: format!("/echo {}", args),
                created_at: None,
                author: AuthorMetadata {
                    id: "456".to_string(),
                    name: "test_user".to_string(),
                    bot: false,
                },
                channel: ChannelMetadata {
                    id: "789".to_string(),
                    name: Some("general".to_string()),
                    channel_type: "GuildText".to_string(),
                },
                guild: None,
                mentions: MentionsMetadata {
                    users: vec![],
                    everyone: false,
                },
                attachments: vec![],
                embeds_count: 0,
                pinned: false,
                webhook_id: None,
            },
        }
    }

    #[test]
    fn test_find_echo() {
        assert!(find("echo").is_some());
    }

    #[test]
    fn test_find_nonexistent() {
        assert!(find("nonexistent").is_none());
    }

    #[test]
    fn test_all_commands_returns_all() {
        let cmds = all_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].name(), "echo");
    }

    #[test]
    fn test_echo_command_name() {
        let cmd = find("echo").unwrap();
        assert_eq!(cmd.name(), "echo");
    }

    #[test]
    fn test_echo_command_description() {
        let cmd = find("echo").unwrap();
        assert!(!cmd.description().is_empty());
    }

    #[test]
    fn test_echo_execute_with_context() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("Hello World");
        assert_eq!(cmd.execute(&ctx), "Hello World");
    }

    #[test]
    fn test_echo_preserves_spacing() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("Hello   World");
        assert_eq!(cmd.execute(&ctx), "Hello   World");
    }

    #[test]
    fn test_echo_preserves_newlines() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("Hello\nWorld");
        assert_eq!(cmd.execute(&ctx), "Hello\nWorld");
    }

    #[test]
    fn test_echo_preserves_markdown() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("**bold** and _italic_");
        assert_eq!(cmd.execute(&ctx), "**bold** and _italic_");
    }

    #[test]
    fn test_echo_empty_args() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("");
        assert_eq!(cmd.execute(&ctx), "");
    }

    #[test]
    fn test_context_has_message_metadata() {
        let ctx = make_test_context("test");
        assert_eq!(ctx.message.author.name, "test_user");
        assert_eq!(ctx.message.channel.id, "789");
        assert_eq!(ctx.message.id, "123");
    }
}
