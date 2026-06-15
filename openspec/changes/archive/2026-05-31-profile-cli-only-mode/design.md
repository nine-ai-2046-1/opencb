## Context

The `opencb serve` command runs a Discord bot that listens for messages. Currently, `handler.rs:89` hardcodes rejection of all messages not starting with `/`:

```rust
if !content.starts_with('/') {
    return;
}
```

The bot uses a `Profile` struct (from `config.toml`) that contains `profile_id`, `channel_ids`, `bot_token`, and `targets`. There is no per-profile control over message filtering behavior.

The user wants profiles where the bot processes ALL messages (not just `/`-prefixed) as input to the target CLI — useful for chatbot-style deployments.

## Goals / Non-Goals

**Goals:**
- Add `cli_only: bool` field to `Profile` struct with default `true` (backward compatible)
- Make the `/`-prefix rejection conditional on `profile.cli_only`
- When `cli_only = false`, all messages are passed to the target CLI

**Non-Goals:**
- Adding new commands or slash commands
- Changing how targets are resolved
- Adding `opencb spy` command (future work)

## Decisions

### Decision: Field default value

**Chosen approach**: `cli_only` defaults to `true` when not present in config.

**Rationale**: Backward compatible — existing configs without `cli_only` behave exactly as before (reject non-`/` messages). Users must explicitly opt into the new behavior.

**Alternative considered**: Default `false` — rejected because it would change behavior for existing users without config changes.

### Decision: Config location

**Chosen approach**: `cli_only` goes in `[profiles.<name>]` table, not at the top level.

**Rationale**: Per-profile control allows different profiles to have different behaviors. A "chatbot" profile can have `cli_only = false` while a "command" profile keeps `cli_only = true`.

### Decision: Conditional in handler

**Chosen approach**: Single conditional check at `handler.rs:89`:

```rust
if self.profile.cli_only && !content.starts_with('/') {
    return;
}
```

**Rationale**: Minimal change, single line. The `self.profile` is already available in `ServeHandler`.

## Risks / Trade-offs

- **[Risk] Users might set `cli_only = false` without understanding implications** → Mitigation: Document in config.sample.toml that this passes ALL messages to the target CLI
- **[Trade-off] No gradual filtering (e.g., regex patterns)** → Acceptable for now; can be extended later if needed
