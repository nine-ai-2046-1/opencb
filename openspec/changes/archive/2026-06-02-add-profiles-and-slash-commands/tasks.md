## 1. Config Restructure — Profiles

- [x] 1.1 Add `Profile` struct to `src/config.rs` with fields: `profile_id`, `channel_type`, `channel_ids`, `bot_token`, `targets`
- [x] 1.2 Add `profiles: HashMap<String, Profile>` field to `Config` struct
- [x] 1.3 Update `load_config()` to parse `[profiles.<name>]` TOML tables into `Config.profiles`
- [x] 1.4 Implement profile name validation (`^[a-z0-9_-]+$`) in `load_config()`
- [x] 1.5 Implement per-profile field validation: bot_token required, not placeholder; channel_ids non-empty
- [x] 1.6 Implement channel_ids wildcard support (`["*"]`)
- [x] 1.7 Implement fallback: when no `[profiles]` section, build synthetic `"default"` profile from top-level fields
- [x] 1.8 Update `render_default_toml()` to generate new `[profiles]` format in default config
- [x] 1.9 Update `config.sample.toml` to show new profiles format

## 2. CLI — Profile Selection

- [x] 2.1 Add `--profile <id>` flag to `Commands::Serve` in `src/cli.rs`
- [x] 2.2 In `main.rs`, resolve profile from CLI flag (default to `"default"`)
- [x] 2.3 Validate selected profile exists in `config.profiles`, exit with error if not found
- [x] 2.4 Pass resolved profile's bot_token, channel_ids, and targets to `ServeHandler`

## 3. Slash Commands Module

- [x] 3.1 Create `src/slash_commands/mod.rs` with `SlashCommand` trait definition
- [x] 3.2 Implement `find(command_name: &str) -> Option<Box<dyn SlashCommand>>` function
- [x] 3.3 Create `src/slash_commands/echo.rs` implementing `EchoCommand`
- [x] 3.4 Register `echo` in `find()` match statement
- [x] 3.5 Add `mod slash_commands;` to `src/main.rs`

## 4. Message Handler Rewrite

- [x] 4.1 Update `ServeHandler` struct to hold resolved profile data (bot_token, channel_ids, targets)
- [x] 4.2 Implement bot-self ignore: compare `msg.author.id` with bot user ID
- [x] 4.3 Implement channel filter: check `msg.channel_id` against profile's channel_ids (skip for DMs)
- [x] 4.4 Implement slash-prefix check: only process messages starting with `/`
- [x] 4.5 Implement command name parsing: extract first word after `/`, validate with `^[a-z0-9_-]+$`
- [x] 4.6 Implement args extraction: remainder after command name, passed as raw string
- [x] 4.7 Route valid commands to `slash_commands::find()`, reply "Invalid command" if not found
- [x] 4.8 Execute found command and send output to same channel (guild) or DM
- [x] 4.9 Remove old target-based execution path from `message()` handler

## 5. Integration and Testing

- [x] 5.1 Update `main.rs` to wire profile selection into `ServeHandler` construction
- [x] 5.2 Test: config with profiles loads correctly
- [x] 5.3 Test: fallback to top-level config works
- [x] 5.4 Test: invalid profile name rejected
- [x] 5.5 Test: `/echo` command returns args verbatim
- [x] 5.6 Test: messages without `/` prefix are ignored
- [x] 5.7 Test: bot's own messages are ignored
- [x] 5.8 Test: channel filtering works with specific IDs and wildcard
