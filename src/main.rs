//! 🚀 Discord Bot 主程序入口
//! 組裝各模組，提供清晰嘅程序入口點 ✨

// 模組聲明
mod cli;
mod config;
mod error;
mod handler;
mod inbound;
mod outbound;
mod scheduler;
mod sqlite_job_store;
mod types;

// 引入所需類型
// sqlite and file persistence intentionally unused in pure in-memory mode
use clap::Parser;
use cli::{Cli, Commands};
use error::handle_discord_error;
use handler::ServeHandler;
use serenity::all::*;
use serenity::builder::CreateMessage;
use serenity::model::id::{ChannelId, UserId};
use chrono::Local;
use std::sync::Arc;
use std::time::Duration;
use crate::scheduler::JobStore;
use serde_json::json;
use tracing::info;
// admin server will capture store/token in closure captures

// Extract time/date flags from a message Vec and from raw argv fallback.
// Returns (effective_time, effective_date, cleaned_message)
fn extract_time_date_message(
    message: Vec<String>,
    time: Option<String>,
    date: Option<String>,
) -> (Option<String>, Option<String>, String) {
    let mut parts = message.clone();
    let mut effective_time = time;
    let mut effective_date = date;
    let mut i = 0usize;
    while i < parts.len() {
        let token = parts[i].clone();

        // If a token contains whitespace (user quoted a phrase that includes flags)
        // split it and process subparts, then rebuild remaining text.
        if token.contains(' ') {
            let mut subparts: Vec<String> =
                token.split_whitespace().map(|s| s.to_string()).collect();
            let mut j = 0usize;
            while j < subparts.len() {
                let sub = subparts[j].clone();
                if sub == "-t" || sub == "--time" {
                    if j + 1 < subparts.len() {
                        if effective_time.is_none() {
                            effective_time = Some(subparts[j + 1].clone());
                        }
                        subparts.remove(j);
                        subparts.remove(j);
                        continue;
                    } else {
                        subparts.remove(j);
                        continue;
                    }
                }
                if sub == "-d" || sub == "--date" {
                    if j + 1 < subparts.len() {
                        if effective_date.is_none() {
                            effective_date = Some(subparts[j + 1].clone());
                        }
                        subparts.remove(j);
                        subparts.remove(j);
                        continue;
                    } else {
                        subparts.remove(j);
                        continue;
                    }
                }
                if sub.starts_with("--time=") || sub.starts_with("-t=") {
                    if effective_time.is_none() {
                        if let Some(v) = sub.splitn(2, '=').nth(1) {
                            effective_time = Some(v.to_string());
                        }
                    }
                    subparts.remove(j);
                    continue;
                }
                if sub.starts_with("--date=") || sub.starts_with("-d=") {
                    if effective_date.is_none() {
                        if let Some(v) = sub.splitn(2, '=').nth(1) {
                            effective_date = Some(v.to_string());
                        }
                    }
                    subparts.remove(j);
                    continue;
                }
                j += 1;
            }
            if subparts.is_empty() {
                parts.remove(i);
                continue;
            } else {
                parts[i] = subparts.join(" ");
                i += 1;
                continue;
            }
        }

        // Normal token handling
        match token.as_str() {
            "-t" | "--time" => {
                if i + 1 < parts.len() {
                    if effective_time.is_none() {
                        effective_time = Some(parts[i + 1].clone());
                    }
                    parts.remove(i);
                    parts.remove(i);
                    continue;
                } else {
                    parts.remove(i);
                    continue;
                }
            }
            "-d" | "--date" => {
                if i + 1 < parts.len() {
                    if effective_date.is_none() {
                        effective_date = Some(parts[i + 1].clone());
                    }
                    parts.remove(i);
                    parts.remove(i);
                    continue;
                } else {
                    parts.remove(i);
                    continue;
                }
            }
            _ => {}
        }

        // handle equals-style tokens inside parts (e.g., --time=23:59)
        if token.starts_with("--time=") || token.starts_with("-t=") {
            if effective_time.is_none() {
                if let Some(v) = token.splitn(2, '=').nth(1) {
                    effective_time = Some(v.to_string());
                }
            }
            parts.remove(i);
            continue;
        }
        if token.starts_with("--date=") || token.starts_with("-d=") {
            if effective_date.is_none() {
                if let Some(v) = token.splitn(2, '=').nth(1) {
                    effective_date = Some(v.to_string());
                }
            }
            parts.remove(i);
            continue;
        }

        // handle compact flags like -t23:59 or --time23:59 (no '='), accept when trailing part looks like time/date
        if token.starts_with("-t") && token.len() > 2 {
            let tail = &token[2..];
            if effective_time.is_none() && tail.chars().any(|c| c == ':') {
                effective_time = Some(tail.to_string());
                parts.remove(i);
                continue;
            }
        }
        if token.starts_with("--time") && token.len() > 6 {
            let tail = &token[6..];
            if effective_time.is_none() && tail.chars().any(|c| c == ':') {
                effective_time = Some(tail.to_string());
                parts.remove(i);
                continue;
            }
        }
        if token.starts_with("-d") && token.len() > 2 {
            let tail = &token[2..];
            if effective_date.is_none() && tail.chars().any(|c| c == '-') {
                effective_date = Some(tail.to_string());
                parts.remove(i);
                continue;
            }
        }
        if token.starts_with("--date") && token.len() > 6 {
            let tail = &token[6..];
            if effective_date.is_none() && tail.chars().any(|c| c == '-') {
                effective_date = Some(tail.to_string());
                parts.remove(i);
                continue;
            }
        }

        i += 1;
    }

    // raw argv fallback for flags that may appear elsewhere on argv
    if effective_time.is_none() || effective_date.is_none() {
        let raw_args: Vec<String> = std::env::args().collect();
        for i in 0..raw_args.len() {
            if (raw_args[i] == "-t" || raw_args[i] == "--time") && i + 1 < raw_args.len() {
                if effective_time.is_none() {
                    effective_time = Some(raw_args[i + 1].clone());
                }
            }
            if (raw_args[i] == "-d" || raw_args[i] == "--date") && i + 1 < raw_args.len() {
                if effective_date.is_none() {
                    effective_date = Some(raw_args[i + 1].clone());
                }
            }
            if raw_args[i].starts_with("--time=") || raw_args[i].starts_with("-t=") {
                if effective_time.is_none() {
                    if let Some(v) = raw_args[i].splitn(2, '=').nth(1) {
                        effective_time = Some(v.to_string());
                    }
                }
            }
            if raw_args[i].starts_with("--date=") || raw_args[i].starts_with("-d=") {
                if effective_date.is_none() {
                    if let Some(v) = raw_args[i].splitn(2, '=').nth(1) {
                        effective_date = Some(v.to_string());
                    }
                }
            }
        }
    }

    let cleaned = parts.join(" ");
    (effective_time, effective_date, cleaned)
}

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
    let subscriber = tracing_subscriber::fmt().with_max_level(if config.debug.unwrap_or(false) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    });
    subscriber.init();

    match cli.command {
        Some(Commands::Send {
            message,
            time,
            date,
            rc,
            ru,
            mu,
        }) => {
            // Extract flags and cleaned message
            let (effective_time, effective_date, msg) =
                extract_time_date_message(message, time, date);

            // If time provided, schedule instead of sending immediately
            tracing::info!(
                "scheduling check: effective_time={:?}, effective_date={:?}",
                effective_time,
                effective_date
            );
            if let Some(time_str) = effective_time {
                // build ScheduledJob
                match crate::scheduler::build_job(msg.clone(), effective_date.clone(), &time_str) {
                    Ok(mut job) => {
                        // attach CLI overrides (rc, ru, mu) into job.meta so serve can honor them later
                        let mut meta_obj = serde_json::Map::new();
                        if let Some(rc_str) = rc.as_ref() {
                            let rc_ids: Vec<u64> = rc_str.split(',').filter_map(|s| s.trim().parse::<u64>().ok()).collect();
                            if !rc_ids.is_empty() {
                                meta_obj.insert("rc".to_string(), json!(rc_ids));
                            }
                        }
                        if let Some(ru_str) = ru.as_ref() {
                            let ru_ids: Vec<u64> = ru_str.split(',').filter_map(|s| s.trim().parse::<u64>().ok()).collect();
                            if !ru_ids.is_empty() {
                                meta_obj.insert("ru".to_string(), json!(ru_ids));
                            }
                        }
                        if let Some(mu_str) = mu.as_ref() {
                            let mu_ids: Vec<u64> = mu_str.split(',').filter_map(|s| s.trim().parse::<u64>().ok()).collect();
                            if !mu_ids.is_empty() {
                                meta_obj.insert("mu".to_string(), json!(mu_ids));
                            }
                        }
                        if !meta_obj.is_empty() {
                            job.meta = Some(serde_json::Value::Object(meta_obj));
                        }
                        // Pure in-memory approach: try admin URL first; if not set, fall back to attempting to contact localhost admin endpoint
                        // so local `opencb send -t` can schedule into the same host without requiring SCHEDULED_ADMIN_URL explicitly.
                        let admin_url = std::env::var("SCHEDULED_ADMIN_URL").unwrap_or_else(|_| "http://127.0.0.1:9001".to_string());
                        let client = reqwest::Client::new();
                        let url = if admin_url.ends_with('/') { format!("{}schedule", admin_url) } else { format!("{}/schedule", admin_url) };
                        let mut req = client.post(&url).json(&job);
                        if let Ok(token) = std::env::var("SCHEDULED_ADMIN_TOKEN") {
                            req = req.bearer_auth(token);
                        }
                        match req.send().await {
                            Ok(resp) if resp.status().is_success() => {
                                println!("Scheduled via admin: id={}", job.id);
                                std::process::exit(0);
                            }
                            Ok(resp) => {
                                eprintln!("Admin schedule failed: status={}", resp.status());
                                std::process::exit(1);
                            }
                            Err(e) => {
                                eprintln!("Admin schedule request failed: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Invalid time/date provided: {}", e);
                        // User requested scheduling but provided invalid time/date.
                        // Do not fall through to immediate send; exit non-zero.
                        std::process::exit(1);
                    }
                }
            }
            info!("Sending message via HTTP API (no Gateway needed)");

            // 直接用 HTTP API - 唔需要 Gateway/event loop 🚀
            let http = serenity::http::Http::new(&config.bot_token);
            // For CLI 'send' allow overriding target channels via -c; accepts comma-separated ids.
            // Resolve final channel list (rc overrides config channel list)
            let override_channel_ids: Vec<u64> = rc
                .as_ref()
                .map(|s| {
                    s.split(',')
                        .filter_map(|x| x.trim().parse::<u64>().ok())
                        .collect()
                })
                .unwrap_or_else(|| config.channel_ids_u64());

            // Append mentions if provided (mu)
            let full_msg = if let Some(mu_str) = mu {
                let mentions: Vec<String> = mu_str
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u64>().ok().map(|id| format!("<@{}>", id)))
                    .collect();
                if mentions.is_empty() {
                    msg.clone()
                } else {
                    format!("{} {}", msg, mentions.join(" "))
                }
            } else {
                msg.clone()
            };

            // If ru (user DM recipients) provided, send DMs to each user id
            if let Some(ru_str) = ru {
                let user_ids: Vec<u64> = ru_str
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u64>().ok())
                    .collect();
                for uid in user_ids {
                    let http = serenity::http::Http::new(&config.bot_token);
                    let user = UserId::new(uid);
                    match user.create_dm_channel(&http).await {
                        Ok(pm) => {
                            let cid = pm.id;
                            // Use the low-level send helper from outbound.rs to avoid type headaches
                            let _ctx_stub = None::<&()>; // placeholder to keep types local; we'll call HTTP API directly
                            let _ = cid
                                .send_message(
                                    &http,
                                    serenity::builder::CreateMessage::new().content(&full_msg),
                                )
                                .await;
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to open DM channel for user {}: {}", uid, e);
                        }
                    }
                }
            }

            // Send to all resolved channels
            if override_channel_ids.is_empty() {
                eprintln!("❌ No channel configured in config.toml (channel_id is empty) and no --rc provided. Set channel_id to a list like [\"123\"] or pass --rc.");
            } else {
                for chid in override_channel_ids.into_iter() {
                    let channel_id = ChannelId::new(chid);
                    match channel_id
                        .send_message(&http, CreateMessage::new().content(&full_msg))
                        .await
                    {
                        Ok(sent_msg) => {
                            info!(
                                "✅ Message sent to channel {} (msg id: {})",
                                chid, sent_msg.id
                            );
                        }
                        Err(e) => {
                            eprintln!("❌ 发送失败 to {}: {}", chid, e);
                        }
                    }
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

            // Initialize job store used by the serve process.
            // Prefer Sqlite if configured, otherwise use an in-memory store but import any scheduled_jobs.json so jobs survive CLI scheduling.
            let in_memory_store = Arc::new(crate::scheduler::InMemoryJobStore::new());

            // Pure in-memory mode: do NOT import from disk or initialize persistent DB.
            // This process will only hold scheduled jobs while running in memory.
            info!("Using pure in-memory job store; scheduled jobs are NOT persisted to disk or DB.");

            // Intentionally not initializing any persistent DB.

            // Spawn a background worker that polls the in-memory job store and sends due jobs
            {
                let store = Arc::clone(&in_memory_store);
                let config_for_task = config.clone();
                tokio::spawn(async move {
                    let http = serenity::http::Http::new(&config_for_task.bot_token);
                    loop {
                        let minute = Local::now().format("%Y-%m-%dT%H:%M").to_string();
                        let due = store.fetch_and_remove_due_jobs(&minute);
                        if !due.is_empty() {
                            // send each job's message to configured channels
                            for job in due.into_iter() {
                                info!("Scheduled job claimed for send: id={} run_at={}", job.id, job.run_at_local_minute);
                                // Determine mentions (mu) appended to the message
                                let mut full_msg = job.message.clone();
                                if let Some(meta) = job.meta.as_ref() {
                                    if let Some(mu_val) = meta.get("mu") {
                                        if let Some(arr) = mu_val.as_array() {
                                            let mentions: Vec<String> = arr.iter().filter_map(|v| v.as_u64().map(|id| format!("<@{}>", id))).collect();
                                            if !mentions.is_empty() {
                                                full_msg = format!("{} {}", full_msg, mentions.join(" "));
                                            }
                                        }
                                    }
                                }

                                // Channel recipients: per-job override (rc) else config defaults
                                let mut channel_ids: Vec<u64> = Vec::new();
                                if let Some(meta) = job.meta.as_ref() {
                                    if let Some(rc_val) = meta.get("rc") {
                                        if let Some(arr) = rc_val.as_array() {
                                            channel_ids = arr.iter().filter_map(|v| v.as_u64()).collect();
                                        }
                                    }
                                }
                                if channel_ids.is_empty() {
                                    channel_ids = config_for_task.channel_ids_u64();
                                }

                                if !channel_ids.is_empty() {
                                    for cid in channel_ids.iter() {
                                        let channel = ChannelId::new(*cid);
                                        info!("Attempting to send scheduled job {} to channel {}", job.id, cid);
                                        match channel
                                            .send_message(&http, CreateMessage::new().content(&full_msg))
                                            .await
                                        {
                                            Ok(sent_msg) => {
                                                info!("Scheduled job {} sent to channel {} msg_id={}", job.id, cid, sent_msg.id);
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to send scheduled job {} to channel {}: {}", job.id, cid, e);
                                            }
                                        }
                                    }
                                } else {
                                    // no channels configured and no per-job override — log warning
                                    eprintln!("No channel configured for scheduled job {}: skipping send", job.id);
                                }

                                // Handle per-job user DM overrides (ru)
                                if let Some(meta) = job.meta.as_ref() {
                                    if let Some(ru_val) = meta.get("ru") {
                                        if let Some(arr) = ru_val.as_array() {
                                            for u in arr.iter().filter_map(|v| v.as_u64()) {
                                                let user = UserId::new(u);
                                                if let Ok(pm) = user.create_dm_channel(&http).await {
                                                    match pm
                                                        .id
                                                        .send_message(&http, CreateMessage::new().content(&full_msg))
                                                        .await
                                                    {
                                                        Ok(sent_msg) => {
                                                            info!("Scheduled job {} sent via DM to user {} msg_id={}", job.id, u, sent_msg.id);
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Failed to send scheduled job {} via DM to user {}: {}", job.id, u, e);
                                                        }
                                                    }
                                                } else {
                                                    eprintln!("Failed to open DM for user {} when sending scheduled job {}", u, job.id);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        tokio::time::sleep(Duration::from_secs(30)).await;
                    }
                });
            }

            // Admin HTTP endpoint: start unless explicitly disabled. Bind/address may be set via SCHEDULED_ADMIN_BIND.
            if std::env::var("SCHEDULED_ADMIN_DISABLE").unwrap_or_default() != "true" {
                // Use a small Hyper server instead of axum to avoid handler type issues.
                use hyper::{Server, Request, Body, Response, Method, StatusCode};
                use hyper::service::{make_service_fn, service_fn};

                // Clone values for move into server
                let store_for_admin = Arc::clone(&in_memory_store);
                let admin_token = std::env::var("SCHEDULED_ADMIN_TOKEN").ok();
                // prefer config.toml scheduled_admin_bind, then env, then default
                let bind_addr = config
                    .scheduled_admin_bind
                    .clone()
                    .or_else(|| std::env::var("SCHEDULED_ADMIN_BIND").ok())
                    .unwrap_or_else(|| "127.0.0.1:19001".to_string());

                let handler_store = store_for_admin.clone();
                let handler_token = admin_token.clone();

                let bind = bind_addr.parse().expect("invalid bind addr");
                tokio::spawn(async move {
                    let make_svc = make_service_fn(move |_conn| {
                        let store = handler_store.clone();
                        let token = handler_token.clone();
                        async move {
                            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                                let store = store.clone();
                                let token = token.clone();
                                async move {
                                    if req.method() == Method::POST && req.uri().path() == "/schedule" {
                                        // auth if token configured
                                        if let Some(expected) = token.as_ref() {
                                            match req.headers().get(hyper::header::AUTHORIZATION) {
                                                Some(hv) => {
                                                    if let Ok(s) = hv.to_str() {
                                                        if !s.starts_with("Bearer ") || &s[7..] != expected {
                                                            let res = Response::builder()
                                                                .status(StatusCode::UNAUTHORIZED)
                                                                .body(Body::from("unauthorized"))
                                                                .unwrap();
                                                            return Ok::<_, hyper::Error>(res);
                                                        }
                                                    } else {
                                                        let res = Response::builder()
                                                            .status(StatusCode::UNAUTHORIZED)
                                                            .body(Body::from("unauthorized"))
                                                            .unwrap();
                                                        return Ok::<_, hyper::Error>(res);
                                                    }
                                                }
                                                None => {
                                                    let res = Response::builder()
                                                        .status(StatusCode::UNAUTHORIZED)
                                                        .body(Body::from("unauthorized"))
                                                        .unwrap();
                                                    return Ok::<_, hyper::Error>(res);
                                                }
                                            }
                                        }

                                        let whole = hyper::body::to_bytes(req.into_body()).await?;
                                        match serde_json::from_slice::<crate::scheduler::ScheduledJob>(&whole) {
                                            Ok(job) => match store.add_job(&job) {
                                                Ok(()) => {
                                                    info!("Admin scheduled job received: id={} run_at={}", job.id, job.run_at_local_minute);
                                                    let body = serde_json::to_string(&serde_json::json!({"id": job.id})).unwrap();
                                                    let res = Response::builder()
                                                        .status(StatusCode::CREATED)
                                                        .body(Body::from(body))
                                                        .unwrap();
                                                    Ok::<_, hyper::Error>(res)
                                                }
                                                Err(e) => {
                                                    let res = Response::builder()
                                                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                        .body(Body::from(format!("fail: {}", e)))
                                                        .unwrap();
                                                    Ok::<_, hyper::Error>(res)
                                                }
                                            },
                                            Err(_) => {
                                                let res = Response::builder()
                                                    .status(StatusCode::BAD_REQUEST)
                                                    .body(Body::from("bad json"))
                                                    .unwrap();
                                                Ok::<_, hyper::Error>(res)
                                            }
                                        }
                                    } else {
                                        let res = Response::builder()
                                            .status(StatusCode::NOT_FOUND)
                                            .body(Body::from("not found"))
                                            .unwrap();
                                        Ok::<_, hyper::Error>(res)
                                    }
                                }
                            }))
                        }
                    });

                    let server = Server::bind(&bind).serve(make_svc);
                    if let Err(e) = server.await {
                        eprintln!("Admin server error: {}", e);
                    }
                });
            }

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
        AuthorMetadata, ChannelMetadata, GuildMetadata, MentionsMetadata, MessageMetadata,
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
            Some(Commands::Send { message, .. }) => {
                assert_eq!(message.join(" "), "Hello World");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_extract_time_after_positional() {
        let parts = vec!["hello".to_string(), "-t".to_string(), "23:59".to_string()];
        let (t, d, msg) = extract_time_date_message(parts, None, None);
        assert_eq!(t.unwrap(), "23:59");
        assert!(d.is_none());
        assert_eq!(msg, "hello");
    }

    #[test]
    fn test_extract_time_equals_style() {
        let parts = vec!["notify".to_string(), "--time=07:30".to_string()];
        let (t, d, msg) = extract_time_date_message(parts, None, None);
        assert_eq!(t.unwrap(), "07:30");
        assert!(d.is_none());
        assert_eq!(msg, "notify");
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
