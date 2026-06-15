use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config;

/// Get the opencb config base directory (~/.config/opencb/)
fn opencb_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Cannot determine home directory ($HOME or $USERPROFILE)")?;
    Ok(PathBuf::from(home).join(".config").join("opencb"))
}

fn read_line(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

/// Prompt for a value. Returns None if user enters empty string (use default).
fn prompt_value(prompt: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let val = read_line(prompt)?;
    if val.is_empty() {
        Ok(None)
    } else {
        Ok(Some(val))
    }
}

/// Create a new profile with interactive prompts and CLI flag prefill.
pub fn add_profile(
    id: &str,
    bot_token: Option<&str>,
    channel_ids: Option<&[String]>,
    cli_only: Option<bool>,
    debug: Option<bool>,
) -> Result<config::Config, Box<dyn std::error::Error>> {
    let base = opencb_config_dir()?;
    let profile_dir = base.join(id);
    let config_file = profile_dir.join("config.toml");

    if config_file.exists() {
        return Err(format!("Profile '{}' already exists at:\n  {}", id, config_file.display()).into());
    }

    if !config::is_valid_name(id) {
        return Err(format!(
            "Invalid profile name '{}'. Use lowercase letters, digits, hyphens, or underscores.",
            id
        )
        .into());
    }

    // Collect values — use flags as prefill, prompt if missing
    let mut final_bot_token = bot_token.map(|s| s.to_string());
    let mut final_channel_ids = channel_ids.map(|s| s.to_vec());
    let mut final_cli_only = cli_only;
    let mut final_debug = debug;

    let needs_interactive = final_bot_token.is_none() || final_channel_ids.is_none();

    if needs_interactive {
        println!("Creating profile '{}'. Press Enter to accept defaults.\n", id);

        // Bot token
        if final_bot_token.is_none() {
            loop {
                match prompt_value("Bot token: ")? {
                    Some(val) => {
                        if !val.is_empty() {
                            final_bot_token = Some(val);
                            break;
                        }
                    }
                    None => {
                        eprintln!("Error: bot_token is required.");
                    }
                }
            }
        }

        // Channel IDs
        if final_channel_ids.is_none() {
            loop {
                match prompt_value("Channel IDs (comma-separated, or * for all): ")? {
                    Some(val) => {
                        if val == "*" {
                            final_channel_ids = Some(vec!["*".to_string()]);
                            break;
                        } else if !val.is_empty() {
                            let ids: Vec<String> =
                                val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                            if !ids.is_empty() {
                                final_channel_ids = Some(ids);
                                break;
                            }
                        }
                    }
                    None => {
                        // empty = use wildcard
                        final_channel_ids = Some(vec!["*".to_string()]);
                        break;
                    }
                }
            }
        }

        // Optional fields
        if final_cli_only.is_none() {
            let val = read_line("CLI only (true/false) [true]: ")?;
            final_cli_only = Some(!matches!(val.as_str(), "false" | "no"));
        }

        if final_debug.is_none() {
            let val = read_line("Debug mode (true/false) [false]: ")?;
            final_debug = Some(matches!(val.as_str(), "true" | "yes"));
        }
    }

    let bot_token_str = final_bot_token.unwrap_or_default();
    let channel_ids_vec = final_channel_ids.unwrap_or_else(|| vec!["*".to_string()]);
    let cli_only_val = final_cli_only.unwrap_or(true);
    let debug_val = final_debug.unwrap_or(false);

    // Show summary
    println!("\n--- Profile Summary ---");
    println!("ID:         {}", id);
    println!("Bot token:  {}", mask_token(&bot_token_str));
    println!("Channel IDs: {:?}", channel_ids_vec);
    println!("CLI only:   {}", cli_only_val);
    println!("Debug:      {}", debug_val);
    println!("------------------------\n");

    // Confirmation
    let confirm = read_line("Create this profile? [y/N]: ")?;
    if confirm.to_lowercase() != "y" {
        return Err("Profile creation cancelled by user.".into());
    }

    // Write config
    fs::create_dir_all(&profile_dir)?;

    let mut toml = String::new();
    toml.push_str(&format!("bot_token = \"{}\"\n", bot_token_str));
    toml.push_str(&format!("channel_ids = {:?}\n", channel_ids_vec));
    toml.push_str(&format!("cli_only = {}\n", cli_only_val));
    if debug_val {
        toml.push_str("debug = true\n");
    }
    toml.push_str("\n[target.opencode]\n");
    toml.push_str("cmd = \"opencode\"\n");
    toml.push_str("argv = [\"run\", \"#INPUT#\"]\n");

    fs::write(&config_file, &toml)?;

    println!("Profile '{}' created at:\n  {}", id, config_file.display());
    println!(
        "Tip: You can modify the config anytime via: opencb profiles set \"{}\" \"key\" \"value\"",
        id
    );

    let created_config = config::Config {
        config_path: config_file,
        bot_token: bot_token_str,
        channel_ids: channel_ids_vec,
        owner_id: Vec::new(),
        debug: Some(debug_val),
        targets: {
            let mut m = std::collections::HashMap::new();
            m.insert(
                "opencode".to_string(),
                config::TargetSpec {
                    cmd: "opencode".to_string(),
                    argv: vec!["run".to_string(), "#INPUT#".to_string()],
                    work_dir: None,
                },
            );
            m
        },
        scheduled_admin_bind: Some("127.0.0.1:19001".to_string()),
        cli_only: cli_only_val,
    };

    Ok(created_config)
}

fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "***".to_string();
    }
    format!("{}...{}", &token[..4], &token[token.len() - 4..])
}

/// Remove a profile directory.
pub fn remove_profile(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base = opencb_config_dir()?;
    let profile_dir = base.join(id);

    if !profile_dir.exists() {
        return Err(format!("Profile '{}' does not exist.", id).into());
    }

    fs::remove_dir_all(&profile_dir)?;
    println!("Profile '{}' removed.", id);
    Ok(())
}

/// Show config key-value pairs for a profile.
pub fn show_config(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base = opencb_config_dir()?;
    let config_file = base.join(id).join("config.toml");

    if !config_file.exists() {
        return Err(format!(
            "Profile '{}' does not exist. Create it with: opencb profiles add \"{}\"",
            id, id
        )
        .into());
    }

    let content = fs::read_to_string(&config_file)?;
    let raw: toml::Value = toml::from_str(&content)?;

    println!("Profile '{}':\n", id);
    print_value(&raw, "");
    Ok(())
}

fn print_value(val: &toml::Value, prefix: &str) {
    match val {
        toml::Value::Table(map) => {
            for (key, v) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                match v {
                    toml::Value::Table(_) => {
                        println!("[{}]", new_prefix);
                        print_value(v, &new_prefix);
                    }
                    _ => {
                        println!("{} = {}", new_prefix, format_toml_value(v));
                    }
                }
            }
        }
        _ => {
            println!("{} = {}", prefix, format_toml_value(val));
        }
    }
}

fn format_toml_value(val: &toml::Value) -> String {
    match val {
        toml::Value::String(s) => format!("\"{}\"", s),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_toml_value).collect();
            format!("[{}]", items.join(", "))
        }
        toml::Value::Datetime(dt) => dt.to_string(),
        toml::Value::Table(_) => "<table>".to_string(),
    }
}

/// Set a config key for a profile.
pub fn set_config(id: &str, key: &str, values: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let base = opencb_config_dir()?;
    let config_file = base.join(id).join("config.toml");

    if !config_file.exists() {
        return Err(format!(
            "Profile '{}' does not exist. Create it with: opencb profiles add \"{}\"",
            id, id
        )
        .into());
    }

    // Skip target keys
    if key.starts_with("target.") {
        println!(
            "Info: Target keys (e.g. '{}') cannot be set via CLI. Edit the config file directly:\n  {}",
            key,
            config_file.display()
        );
        return Ok(());
    }

    let valid_keys = [
        "bot_token",
        "channel_ids",
        "owner_id",
        "debug",
        "cli_only",
        "scheduled_admin_bind",
    ];
    if !valid_keys.contains(&key) {
        return Err(format!(
            "Invalid key '{}'. Valid keys: {}",
            key,
            valid_keys.join(", ")
        )
        .into());
    }

    let content = fs::read_to_string(&config_file)?;
    let mut raw: toml::Value = toml::from_str(&content)?;

    // Determine the TOML value type
    match key {
        "bot_token" | "scheduled_admin_bind" => {
            if values.len() != 1 {
                return Err(format!("Key '{}' requires exactly one value.", key).into());
            }
            raw[key] = toml::Value::String(values[0].clone());
        }
        "debug" | "cli_only" => {
            if values.len() != 1 {
                return Err(format!("Key '{}' requires exactly one value (true/false).", key).into());
            }
            let b = match values[0].as_str() {
                "true" | "yes" => true,
                "false" | "no" => false,
                _ => return Err(format!("Invalid boolean value '{}'. Use true or false.", values[0]).into()),
            };
            raw[key] = toml::Value::Boolean(b);
        }
        "channel_ids" => {
            if values.len() == 1 && values[0] == "*" {
                raw[key] = toml::Value::Array(vec![toml::Value::String("*".to_string())]);
            } else {
                let arr: Vec<toml::Value> =
                    values.iter().map(|v| toml::Value::String(v.clone())).collect();
                raw[key] = toml::Value::Array(arr);
            }
        }
        "owner_id" => {
            let arr: Vec<toml::Value> =
                values.iter().map(|v| toml::Value::String(v.clone())).collect();
            raw[key] = toml::Value::Array(arr);
        }
        _ => unreachable!(),
    }

    let new_content = toml::to_string_pretty(&raw)?;
    fs::write(&config_file, new_content)?;

    println!("Updated '{}' in profile '{}'.", key, id);
    Ok(())
}
