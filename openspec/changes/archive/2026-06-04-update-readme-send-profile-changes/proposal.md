## Why

The README (English and Chinese) documents an outdated config format and send command usage that no longer matches the current codebase. After introducing profile-based config, `default_send_to_channel_ids`, `--profile` / `--rc` / `--ru` / `--mu` flags, and native slash commands, users following the README will get incorrect results or errors.

## What Changes

- Remove old flat `bot_token` / `channel_id` config example from the Usage section
- Document the `[profiles.<name>]` config format as the canonical format
- Document `default_send_to_channel_ids` field and its purpose
- Update `send` command examples to include `--profile`, `--rc`, `--ru`, `--mu` flags
- Clarify that multi-word messages work without quotes and `--rc` is an override (not the message content)
- Document native Discord slash commands (`/echo`) and how they are auto-registered on bot start
- Update the `serve` command docs to mention `--profile` flag
- Update module table (add `slash_commands/` module, `splitter.rs`, `scheduler.rs`)
- Update test count (49 tests, not 4)
- Update `config.sample.toml` inline example to reflect profiles format
- Apply all of the above identically to `README-ZH.md` (Cantonese)

## Capabilities

### New Capabilities

- `readme-send-profile`: Documents the profile-based config, send command flags, and slash commands in both README.md and README-ZH.md

### Modified Capabilities

- `profiles-config`: Delta spec update — add `default_send_to_channel_ids` field requirement
- `send-message-formatting`: Delta spec update — document `--profile`, `--rc`, `--ru`, `--mu` flags and multi-word message parsing fix

## Impact

- `README.md` — full rewrite of Configuration, Usage, and Send/Serve command sections
- `README-ZH.md` — same sections rewritten in Cantonese
- `config.sample.toml` — already up to date (no change needed)
- No code changes; documentation only
