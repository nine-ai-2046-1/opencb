//! /cli slash command.
//! Forwards free-form user input to nine-cli, streams stdout back to Discord
//! via ResponseHandle with periodic status updates.

use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;
use tracing::{error, info, warn};

use serenity::all::CommandOptionType;
use serenity::builder::CreateCommandOption;

use crate::argv_parser::tokenize_argv;

use super::{CommandContext, ResponseHandle, SlashCommand};

/// Maximum execution time for a nine-cli skill (10 minutes).
const TIMEOUT_SECS: u64 = 600;

/// Minimum interval between Discord message edits to avoid rate limits.
const UPDATE_INTERVAL_SECS: u64 = 2;

/// Maximum characters of output to display in a single Discord message.
/// Discord cap is 2000; we reserve ~200 for the header.
const MAX_DISPLAY_CHARS: usize = 1800;

/// Characters to keep when the rolling window kicks in.
const ROLLING_WINDOW_CHARS: usize = 1600;

pub struct CliCommand;

impl SlashCommand for CliCommand {
    fn name(&self) -> &str {
        "cli"
    }

    fn description(&self) -> &str {
        "Run a nine-cli skill. Usage: /cli <skill-name> [args...]"
    }

    /// Declares one required String option named "args".
    /// Discord will show a text input field when the user types /cli.
    fn options(&self) -> Vec<CreateCommandOption> {
        vec![
            CreateCommandOption::new(
                CommandOptionType::String,
                "args",
                "Arguments to pass to nine-cli (e.g. skill-name arg1 \"quoted arg\")",
            )
            .required(true),
        ]
    }

    /// Simple execute — not used directly; execute_with_updates handles /cli.
    /// Returns a plain string for text-based message invocations (no streaming).
    async fn execute(&self, ctx: &CommandContext) -> String {
        // For text-based invocations, run nine-cli and capture output synchronously
        let tokens = tokenize_argv(&ctx.args);
        if tokens.is_empty() {
            return "❌  Usage: /cli <skill-name> [args...]".to_string();
        }

        let skill = &tokens[0];
        let skill_args = &tokens[1..];

        match std::process::Command::new("nine-cli")
            .args(skill_args.iter().fold(vec![skill.as_str()], |mut acc, a| {
                acc.push(a.as_str());
                acc
            }))
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let combined = format!("{}{}", stdout, stderr);
                if combined.is_empty() {
                    format!("✅  nine-cli {} — no output", ctx.args)
                } else {
                    apply_rolling_window(&combined)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                "❌  nine-cli not found in PATH".to_string()
            }
            Err(e) => format!("❌  Failed to spawn nine-cli: {}", e),
        }
    }

    /// Streaming execute — spawns nine-cli asynchronously, edits Discord
    /// message with accumulated output every UPDATE_INTERVAL_SECS seconds.
    async fn execute_with_updates(&self, ctx: &CommandContext, handle: &ResponseHandle) {
        let tokens = tokenize_argv(&ctx.args);
        if tokens.is_empty() {
            handle
                .finalize("❌  Usage: /cli <skill-name> [args...]")
                .await;
            return;
        }

        // Show immediate "running" status before spawning
        let header_running = format_header(false, &ctx.args, None);
        handle.update(&header_running).await;

        let start = Instant::now();

        // Wrap the entire operation in a timeout
        let result = timeout(
            Duration::from_secs(TIMEOUT_SECS),
            run_nine_cli(tokens, ctx.args.clone(), handle),
        )
        .await;

        match result {
            Ok(()) => {
                // Execution completed — finalize was called inside run_nine_cli
                info!("✅ /cli completed in {:.1}s", start.elapsed().as_secs_f64());
            }
            Err(_) => {
                // Timeout elapsed — report to Discord
                warn!("⏱️ /cli timed out after {} minutes", TIMEOUT_SECS / 60);
                handle
                    .finalize(&format!(
                        "⏱️  `nine-cli {}` timed out after {} minutes",
                        ctx.args,
                        TIMEOUT_SECS / 60
                    ))
                    .await;
            }
        }
    }
}

/// Spawn nine-cli, stream stdout line-by-line, and push periodic Discord updates.
/// Calls `handle.finalize()` on completion.
async fn run_nine_cli(tokens: Vec<String>, raw_args: String, handle: &ResponseHandle) {
    // Spawn nine-cli with piped stdout and stderr
    let mut child = match TokioCommand::new("nine-cli")
        .args(&tokens)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            handle.finalize("❌  nine-cli not found in PATH").await;
            return;
        }
        Err(e) => {
            handle
                .finalize(&format!("❌  Failed to spawn nine-cli: {}", e))
                .await;
            return;
        }
    };

    // Attach async readers to stdout and stderr
    let stdout = child.stdout.take().expect("stdout pipe missing");
    let stderr = child.stderr.take().expect("stderr pipe missing");

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    let mut accumulated = String::new();
    let mut last_update = Instant::now();

    // Read stdout and stderr concurrently until both are exhausted
    loop {
        tokio::select! {
            // Read next stdout line
            line = stdout_lines.next_line() => {
                match line {
                    Ok(Some(l)) => {
                        accumulated.push_str(&l);
                        accumulated.push('\n');
                    }
                    Ok(None) => break, // stdout EOF
                    Err(e) => {
                        error!("Error reading nine-cli stdout: {}", e);
                        break;
                    }
                }
            }
            // Read next stderr line (interleaved)
            line = stderr_lines.next_line() => {
                match line {
                    Ok(Some(l)) => {
                        accumulated.push_str(&l);
                        accumulated.push('\n');
                    }
                    Ok(None) => {} // stderr EOF — keep reading stdout
                    Err(e) => {
                        error!("Error reading nine-cli stderr: {}", e);
                    }
                }
            }
        }

        // Rate-limited Discord edit — at most once every UPDATE_INTERVAL_SECS
        if last_update.elapsed() >= Duration::from_secs(UPDATE_INTERVAL_SECS) {
            let display = format_running_message(&raw_args, &accumulated);
            handle.update(&display).await;
            last_update = Instant::now();
        }
    }

    // Drain remaining stderr after stdout EOF
    while let Ok(Some(l)) = stderr_lines.next_line().await {
        accumulated.push_str(&l);
        accumulated.push('\n');
    }

    // Wait for process to exit and get status code
    let exit_status = child.wait().await;

    // Build final message
    let final_msg = match exit_status {
        Ok(status) if status.success() => {
            let output_display = apply_rolling_window(&accumulated);
            format!(
                "{}{}",
                format_header(true, &raw_args, Some("✅")),
                output_display
            )
        }
        Ok(status) => {
            let code = status.code().unwrap_or(-1);
            let output_display = apply_rolling_window(&accumulated);
            format!(
                "❌  `nine-cli {}` exited with code {}\n────────────────────────\n{}",
                raw_args, code, output_display
            )
        }
        Err(e) => format!("❌  nine-cli wait() failed: {}", e),
    };

    handle.finalize(&final_msg).await;
}

/// Format the "in progress" Discord message with accumulated output.
fn format_running_message(raw_args: &str, accumulated: &str) -> String {
    let header = format!("🔄  `nine-cli {}`\n────────────────────────\n", raw_args);
    let body = apply_rolling_window(accumulated);
    // Ensure total stays within Discord's 2000-char limit
    let combined = format!("{}{}", header, body);
    if combined.len() > 1990 {
        combined[..1990].to_string()
    } else {
        combined
    }
}

/// Format a completion header line.
fn format_header(done: bool, raw_args: &str, icon: Option<&str>) -> String {
    let icon = icon.unwrap_or(if done { "✅" } else { "🔄" });
    format!("{}  `nine-cli {}`\n────────────────────────\n", icon, raw_args)
}

/// Apply a rolling display window: if output exceeds MAX_DISPLAY_CHARS,
/// show a truncation notice followed by the last ROLLING_WINDOW_CHARS characters.
fn apply_rolling_window(output: &str) -> String {
    let char_count = output.chars().count();
    if char_count <= MAX_DISPLAY_CHARS {
        output.to_string()
    } else {
        // Take the last ROLLING_WINDOW_CHARS characters (char-based, not byte-based)
        let skip = char_count - ROLLING_WINDOW_CHARS;
        let tail: String = output.chars().skip(skip).collect();
        format!("[...earlier output truncated...]\n{}", tail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AuthorMetadata, ChannelMetadata, MentionsMetadata};

    /// Build a minimal CommandContext for testing.
    fn make_ctx(args: &str) -> CommandContext {
        CommandContext {
            args: args.to_string(),
            message: crate::types::MessageMetadata {
                id: "1".to_string(),
                content: format!("/cli {}", args),
                created_at: None,
                author: AuthorMetadata {
                    id: "2".to_string(),
                    name: "tester".to_string(),
                    bot: false,
                },
                channel: ChannelMetadata {
                    id: "3".to_string(),
                    name: None,
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
    fn test_cli_name() {
        assert_eq!(CliCommand.name(), "cli");
    }

    #[test]
    fn test_cli_description_nonempty() {
        assert!(!CliCommand.description().is_empty());
    }

    #[test]
    fn test_cli_options_has_args_field() {
        let opts = CliCommand.options();
        assert_eq!(opts.len(), 1);
    }

    #[test]
    fn test_apply_rolling_window_short() {
        // Output within limit — returned as-is
        let short = "hello world";
        assert_eq!(apply_rolling_window(short), short);
    }

    #[test]
    fn test_apply_rolling_window_long() {
        // Output exceeding MAX_DISPLAY_CHARS gets truncated notice
        let long = "x".repeat(MAX_DISPLAY_CHARS + 100);
        let result = apply_rolling_window(&long);
        assert!(result.starts_with("[...earlier output truncated...]"));
        assert!(result.chars().count() <= MAX_DISPLAY_CHARS + 50); // well within Discord limit
    }

    #[tokio::test]
    async fn test_execute_empty_args() {
        // Empty args returns usage error without spawning anything
        let ctx = make_ctx("");
        let result = CliCommand.execute(&ctx).await;
        assert!(result.contains("Usage"));
    }
}
