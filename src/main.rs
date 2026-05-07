//! 🚀 Discord Bot 主程序入口
//! 組裝各模組，提供清晰嘅程序入口點 ✨

// 模組聲明
mod types;
mod config;
mod cli;
mod error;
mod outbound;
mod inbound;
mod handler;

// 引入所需類型
use clap::Parser;
use serenity::all::*;
use serenity::builder::CreateMessage;
use serenity::model::id::ChannelId;
use tracing::info;
use cli::{Cli, Commands};
use handler::ServeHandler;
use error::handle_discord_error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = match config::load_config(cli.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Config error: {}", e);
            std::process::exit(1);
        }
    };

    // ✅ 及早檢查：如果 CLI 啟動時指定咗 target，立即驗證 config 裡面有無該 target
    if let Some(ref target_name) = cli.target {
        if !config.targets.contains_key(target_name) {
            eprintln!(
                "❌ Target '{}' 喺 config.toml 入面搵唔到，請檢查 [<target>] table 同名稱是否一致",
                target_name
            );
            std::process::exit(1);
        }
    }

    // Set tracing subscriber ONCE based on debug flag 📊
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if config.debug.unwrap_or(false) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        });
    subscriber.init();

    match cli.command {
        Some(Commands::Send { message }) => {
            let msg = message.join(" ");
            info!("Sending message via HTTP API (no Gateway needed)");

            // 直接用 HTTP API - 唔需要 Gateway/event loop 🚀
            let http = serenity::http::Http::new(&config.bot_token);
            let channel_id = ChannelId::new(config.channel_id);

            match channel_id.send_message(&http, CreateMessage::new().content(&msg)).await {
                Ok(sent_msg) => {
                    info!("✅ Message sent to channel {} (msg id: {})", config.channel_id, sent_msg.id);
                }
                Err(e) => {
                    eprintln!("❌ 发送失败！");
                    eprintln!("HTTP 错误：{}", e);
                    eprintln!();
                    eprintln!("可能原因：");
                    eprintln!("  1. bot_token 无效或已过期");
                    eprintln!("  2. channel_id 不正确或 bot 无权限");
                    eprintln!("  3. Token 格式有问题（多余空格等）");
                }
            }

            // 強制退出 - 冇 event loop，冇 Gateway，直接 quit ✨
            info!("Send command completed, exiting.");
            std::process::exit(0);
        }
        None | Some(Commands::Serve { .. }) => {
            let intents = GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT;

            let mut client = Client::builder(&config.bot_token, intents)
                .event_handler(ServeHandler {
                    config: config.clone(),
                    target: cli.target.clone(),
                })
                .await?;

            info!("Starting bot in serve mode");
            if let Err(e) = client.start().await {
                handle_discord_error(e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{
        MessageMetadata, AuthorMetadata, ChannelMetadata,
        GuildMetadata, MentionsMetadata,
    };

    #[test]
    fn test_cli_parsing_serve() {
        let cli = Cli::try_parse_from(["opencb", "serve"]).unwrap();
        match cli.command {
            Some(Commands::Serve) => (),
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_cli_parsing_send() {
        let cli = Cli::try_parse_from(["opencb", "send", "Hello", "World"]).unwrap();
        match cli.command {
            Some(Commands::Send { message }) => {
                assert_eq!(message.join(" "), "Hello World");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_cli_parsing_default() {
        let cli = Cli::try_parse_from(["opencb"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_message_metadata_serialization() {
        let metadata = MessageMetadata {
            id: "123".to_string(),
            content: "test".to_string(),
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
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
            guild: Some(GuildMetadata {
                id: "999".to_string(),
                name: "Test Guild".to_string(),
            }),
            mentions: MentionsMetadata {
                users: vec![],
                everyone: false,
            },
            attachments: vec![],
            embeds_count: 0,
            pinned: false,
            webhook_id: None,
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("\"id\": \"123\""));
        assert!(json.contains("\"content\": \"test\""));
    }
}
