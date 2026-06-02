## Why

The current `opencb serve` has a flat config structure with a single bot token and channel list, limiting it to one Discord bot instance per config file. Additionally, the message handler processes all messages as raw text input to external CLI targets, with no built-in command system. We need:

1. **Multi-bot support** via profiles — run different bots with different tokens/channels from one config
2. **Built-in slash command system** — structured `/command args` parsing with modular command registration, replacing the "everything is a CLI target" approach

## What Changes

- **Config restructure**: Add `[profiles]` section where each profile has `profile_id`, `channel_type`, `channel_ids`, `bot_token`, and its own `[targets]` map
- **Backward compatible**: Old top-level `bot_token` / `channel_id` / `[targets]` kept as fallback when no profiles exist
- **CLI `--profile` flag**: `opencb serve --profile <id>` selects a profile; defaults to `"default"`
- **Message handler rewrite**: Ignore bot's own messages; filter by channel_ids (unless `*`); only accept messages starting with `/`; route to slash command system
- **New slash commands module**: `src/slash_commands/` with `SlashCommand` trait, command registry, and `/echo` as first implementation
- **Naming validation**: Profile names and command names must match `^[a-z0-9_-]+$`

## Capabilities

### New Capabilities
- `profiles-config`: Profile-based configuration with per-profile bot token, channel IDs, targets, and naming validation
- `slash-commands`: Modular slash command system with trait-based registration, `/command args` parsing, and routing
- `message-handler`: Updated Discord message handler with bot-ignore, channel filtering, slash-prefix detection, and command dispatch

### Modified Capabilities
- `message-formatting`: No changes to requirements (escape sequence handling preserved)
- `message-splitting`: No changes to requirements
- `send-message-formatting`: No changes to requirements

## Impact

- **Code**: `src/config.rs` (Config struct + load_config rewrite), `src/cli.rs` (--profile arg), `src/handler.rs` (message handler rewrite), `src/main.rs` (profile selection logic)
- **New code**: `src/slash_commands/mod.rs`, `src/slash_commands/echo.rs`
- **Config format**: New `[profiles]` section in `config.toml`; old format still works as fallback
- **Dependencies**: No new crate dependencies needed
- **Breaking changes**: None — old configs continue to work via fallback path
