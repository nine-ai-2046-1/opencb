## Why

Currently, `opencb serve` rejects ALL messages that don't start with `/` (hardcoded at `handler.rs:89`). This means the bot can only respond to slash-style commands like `/cli`, `/echo`, etc. Users want the flexibility to have profiles where ALL messages (not just `/`-prefixed) are passed through to the target CLI — useful for chatbot-style deployments where the bot processes every message as input.

## What Changes

- Add `cli_only: bool` field to `Profile` struct (default `true` for backward compatibility)
- When `cli_only = true`: current behavior — reject non-`/` messages
- When `cli_only = false`: pass all messages through to the target CLI as input
- Update `config.sample.toml` with the new field
- Add config loading support for the new field

## Capabilities

### New Capabilities
- `profile-cli-only`: Configurable per-profile control over whether non-`/` messages are rejected or passed to the target CLI

### Modified Capabilities

## Impact

- **Files**: `src/config.rs` (add `cli_only` field to `Profile`), `src/handler.rs` (conditional rejection logic), `config.sample.toml`
- **Config**: New optional field `cli_only` in `[profiles.<name>]` — backward compatible (defaults to `true`)
- **Behavior change**: Profiles with `cli_only = false` will process all messages, not just `/`-prefixed ones
