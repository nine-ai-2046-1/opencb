//! Slash Commands module.
//! Provides command trait, ResponseHandle for streaming updates,
//! enum-based dispatch, registration, and routing.

use std::sync::Arc;
use serenity::all::{ApplicationId, Http, UserId};
use serenity::builder::{Builder, CreateCommandOption, EditInteractionResponse};
use tracing::info;

use crate::types::MessageMetadata;

mod echo;
pub mod cli;

// ────────────────────────────────────────────────────────────────
// CommandContext
// ────────────────────────────────────────────────────────────────

/// Context passed to every slash command execution.
/// Contains both the parsed args and the full message metadata.
pub struct CommandContext {
    /// The raw arguments string after the command name.
    pub args: String,
    /// Full message metadata (author, channel, guild, attachments, etc.).
    #[allow(dead_code)]
    pub message: MessageMetadata,
}

// ────────────────────────────────────────────────────────────────
// ResponseHandle — allows commands to push streaming updates to Discord
// ────────────────────────────────────────────────────────────────

/// Carries the Discord HTTP client and interaction token so that commands
/// can push intermediate status updates and a final result to Discord
/// without knowing about the interaction protocol.
pub struct ResponseHandle {
    /// Serenity HTTP client for Discord API calls.
    pub http: Arc<Http>,
    /// The Discord application ID (needed for editing interaction responses).
    #[allow(dead_code)]
    pub application_id: ApplicationId,
    /// The interaction token used to edit the original deferred response.
    pub interaction_token: String,
}

impl ResponseHandle {
    /// Edit the deferred interaction response with an in-progress status.
    /// Call this periodically during long-running execution.
    pub async fn update(&self, content: &str) {
        let builder = EditInteractionResponse::new().content(content);
        if let Err(e) = builder
            .execute(self.http.as_ref(), &self.interaction_token)
            .await
        {
            tracing::warn!("⚠️ ResponseHandle::update failed: {}", e);
        }
    }

    /// Edit the deferred interaction response with the final result.
    /// Should be called exactly once at the end of execution.
    pub async fn finalize(&self, content: &str) {
        let builder = EditInteractionResponse::new().content(content);
        if let Err(e) = builder
            .execute(self.http.as_ref(), &self.interaction_token)
            .await
        {
            tracing::error!("❌ ResponseHandle::finalize failed: {}", e);
        }
    }
}

// ────────────────────────────────────────────────────────────────
// SlashCommand trait
// ────────────────────────────────────────────────────────────────

/// Trait implemented by every slash command.
///
/// Simple commands implement only `execute()`.
/// Streaming commands override `execute_with_updates()` to push incremental
/// status messages to Discord via `ResponseHandle`.
///
/// Note: this trait uses `async fn` (Rust 1.75+) and is NOT used as a
/// trait object (`dyn SlashCommand`). Use `CommandDispatch` for routing.
pub trait SlashCommand: Send + Sync {
    /// Returns the command name (without `/` prefix).
    fn name(&self) -> &str;

    /// Returns a human-readable description shown in the Discord command picker.
    fn description(&self) -> &str;

    /// Returns the Discord option definitions for this command.
    /// Default: no options. Override to declare typed parameters.
    fn options(&self) -> Vec<CreateCommandOption> {
        vec![]
    }

    /// Execute the command and return the response string.
    /// Used by simple commands and as the fallback for `execute_with_updates`.
    async fn execute(&self, ctx: &CommandContext) -> String;

    /// Execute the command with streaming Discord update support.
    ///
    /// Default implementation calls `execute()` once and passes the result to
    /// `handle.finalize()`. Streaming commands override this to call
    /// `handle.update()` periodically during execution.
    async fn execute_with_updates(&self, ctx: &CommandContext, handle: &ResponseHandle) {
        let result = self.execute(ctx).await;
        handle.finalize(&result).await;
    }
}

// ────────────────────────────────────────────────────────────────
// CommandDispatch — concrete enum router (avoids dyn + async incompatibility)
// ────────────────────────────────────────────────────────────────

/// Enum-based dispatcher for all registered slash commands.
///
/// Rust's `async fn in trait` is not yet `dyn`-compatible, so `find()` and
/// `all_commands()` return `CommandDispatch` instead of `Box<dyn SlashCommand>`.
/// Each variant delegates to the corresponding concrete command struct.
pub enum CommandDispatch {
    /// The `/echo` command — echoes args back verbatim.
    Echo,
    /// The `/cli` command — forwards args to nine-cli with streaming output.
    Cli,
}

impl CommandDispatch {
    /// Returns the command name (without `/` prefix).
    pub fn name(&self) -> &str {
        match self {
            CommandDispatch::Echo => echo::EchoCommand.name(),
            CommandDispatch::Cli => cli::CliCommand.name(),
        }
    }

    /// Returns the command description for Discord registration.
    pub fn description(&self) -> &str {
        match self {
            CommandDispatch::Echo => echo::EchoCommand.description(),
            CommandDispatch::Cli => cli::CliCommand.description(),
        }
    }

    /// Returns the Discord option definitions for this command.
    pub fn options(&self) -> Vec<CreateCommandOption> {
        match self {
            CommandDispatch::Echo => echo::EchoCommand.options(),
            CommandDispatch::Cli => cli::CliCommand.options(),
        }
    }

    /// Execute and return the output string.
    /// Used by the text-based message handler (no ResponseHandle available).
    pub async fn execute(&self, ctx: &CommandContext) -> String {
        match self {
            CommandDispatch::Echo => echo::EchoCommand.execute(ctx).await,
            CommandDispatch::Cli => cli::CliCommand.execute(ctx).await,
        }
    }

    /// Execute with streaming Discord update support.
    /// Used by the interaction handler after deferring the response.
    pub async fn execute_with_updates(&self, ctx: &CommandContext, handle: &ResponseHandle) {
        match self {
            CommandDispatch::Echo => echo::EchoCommand.execute_with_updates(ctx, handle).await,
            CommandDispatch::Cli => cli::CliCommand.execute_with_updates(ctx, handle).await,
        }
    }
}

// ────────────────────────────────────────────────────────────────
// Public routing functions
// ────────────────────────────────────────────────────────────────

/// Find a slash command by name.
/// Returns `Some(CommandDispatch)` if found, `None` otherwise.
pub fn find(command_name: &str) -> Option<CommandDispatch> {
    match command_name {
        "echo" => Some(CommandDispatch::Echo),
        "cli" => Some(CommandDispatch::Cli),
        _ => None,
    }
}

/// Returns all registered slash commands for Discord API registration.
pub fn all_commands() -> Vec<CommandDispatch> {
    vec![CommandDispatch::Echo, CommandDispatch::Cli]
}

/// Register all slash commands with Discord's API.
/// Called once on bot startup from the `ready()` event.
pub async fn register_all_commands(http: &Http, _app_id: UserId) {
    for cmd in all_commands() {
        // Build the command with its declared options so Discord shows correct input fields
        let mut builder = serenity::builder::CreateCommand::new(cmd.name())
            .description(cmd.description());
        for opt in cmd.options() {
            builder = builder.add_option(opt);
        }
        match builder.execute(http, (None, None)).await {
            Ok(_) => info!("✅ Registered slash command: /{}", cmd.name()),
            Err(e) => tracing::error!("❌ Failed to register /{}: {}", cmd.name(), e),
        }
    }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AuthorMetadata, ChannelMetadata, MentionsMetadata};

    /// Build a minimal CommandContext for testing.
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

    // ── find() / all_commands() ──

    #[test]
    fn test_find_echo() {
        assert!(find("echo").is_some());
    }

    #[test]
    fn test_find_cli() {
        assert!(find("cli").is_some());
    }

    #[test]
    fn test_find_nonexistent() {
        assert!(find("nonexistent").is_none());
    }

    #[test]
    fn test_all_commands_returns_all() {
        let cmds = all_commands();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].name(), "echo");
        assert_eq!(cmds[1].name(), "cli");
    }

    // ── EchoCommand via dispatch ──

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

    #[tokio::test]
    async fn test_echo_execute_with_context() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("Hello World");
        assert_eq!(cmd.execute(&ctx).await, "Hello World");
    }

    #[tokio::test]
    async fn test_echo_preserves_spacing() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("Hello   World");
        assert_eq!(cmd.execute(&ctx).await, "Hello   World");
    }

    #[tokio::test]
    async fn test_echo_preserves_newlines() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("Hello\nWorld");
        assert_eq!(cmd.execute(&ctx).await, "Hello\nWorld");
    }

    #[tokio::test]
    async fn test_echo_preserves_markdown() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("**bold** and _italic_");
        assert_eq!(cmd.execute(&ctx).await, "**bold** and _italic_");
    }

    #[tokio::test]
    async fn test_echo_empty_args() {
        let cmd = find("echo").unwrap();
        let ctx = make_test_context("");
        assert_eq!(cmd.execute(&ctx).await, "");
    }

    #[test]
    fn test_context_has_message_metadata() {
        let ctx = make_test_context("test");
        assert_eq!(ctx.message.author.name, "test_user");
        assert_eq!(ctx.message.channel.id, "789");
        assert_eq!(ctx.message.id, "123");
    }

    // ── CliCommand via dispatch ──

    #[test]
    fn test_cli_command_name() {
        let cmd = find("cli").unwrap();
        assert_eq!(cmd.name(), "cli");
    }

    #[test]
    fn test_cli_command_description() {
        let cmd = find("cli").unwrap();
        assert!(!cmd.description().is_empty());
    }

    #[test]
    fn test_cli_command_has_args_option() {
        let cmd = find("cli").unwrap();
        let opts = cmd.options();
        assert_eq!(opts.len(), 1);
    }
}
