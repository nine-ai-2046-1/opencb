## Context

OpenCB is a Discord bot that processes messages and routes them to external CLI targets. Currently, the config uses a flat structure with a single `bot_token`, `channel_id` list, and a global `targets` map. This limits the bot to one Discord identity per config file.

The message handler currently treats all incoming messages as raw input for CLI execution, with no structured command system.

**Current state:**
- `Config` struct: flat `bot_token`, `channel_id`, `targets` fields
- `load_config()`: parses TOML manually via `toml::Value`, discovers targets by scanning tables with `cmd` + `argv`
- `ServeHandler`: ignores self-messages, requires @mention for guild messages, passes content to `run_target_and_reply()`
- CLI: `opencb serve` or `opencb <target> serve`

**Constraints:**
- Must remain backward-compatible with existing flat config format
- No new crate dependencies
- Existing `message-formatting`, `message-splitting`, `send-message-formatting` specs unchanged

## Goals / Non-Goals

**Goals:**
- Support multiple bot profiles in one config file
- Each profile has isolated bot_token, channel_ids, and targets
- Built-in slash command system with modular command registration
- Only process `/` prefixed messages as commands
- Channel filtering: accept only configured channels (or all with `*`)
- DM messages always accepted (bypass channel filter)
- Naming validation: `^[a-z0-9_-]+$` for profile and command names

**Non-Goals:**
- Discord native slash commands (this is text-based `/command` parsing)
- Removing the old flat config format (fallback preserved)
- Changing existing message formatting/splitting behavior
- Implementing all slash commands (only `/echo` in this change)
- Target-per-profile inheritance from global targets

## Decisions

### D1: Config Structure — Nested Profiles with Per-Profile Targets

**Decision:** Add `[profiles.<name>]` TOML tables. Each profile is self-contained with its own `bot_token`, `channel_ids`, `channel_type`, and `[profiles.<name>.targets]` sub-table.

**Why not shared/global targets:**
- Profiles represent isolated bot identities; sharing targets creates coupling
- A "work" bot and a "personal" bot likely need different CLI targets
- Simpler mental model: each profile is a complete bot configuration

**Fallback:** If no `[profiles]` section exists, `load_config()` builds a single synthetic profile `"default"` from top-level `bot_token`, `channel_id`, and `[targets]`.

### D2: Profile Selection — CLI Flag with Default Fallback

**Decision:** Add `--profile <id>` flag to `Commands::Serve`. When omitted, look for `"default"` profile. If no profiles exist at all, use fallback path.

**Alternatives considered:**
- Positional arg `opencb serve work` — rejected because `target` is already a positional on `Cli`, would be confusing
- Environment variable `OPENCB_PROFILE` — rejected, CLI flag is more explicit

### D3: Message Handler — Slash Prefix + Command Routing

**Decision:** The handler filters messages through a pipeline:
1. Ignore self (bot's own messages)
2. DM: always proceed (skip channel filter)
3. Guild: check channel_ids (skip if `*`)
4. Require `/` prefix
5. Parse command name + args
6. Validate command name format (`^[a-z0-9_-]+$`)
7. Route to `slash_commands::find()`
8. Reply "Invalid command" if not found

**Why not @mention-based:**
- User explicitly asked for `/` prefix
- Slash commands are a familiar pattern (Discord, Slack, CLI)
- @mention is removed from content before this pipeline

### D4: Slash Command Trait — Simple String In/Out

**Decision:** `SlashCommand` trait with `name() -> &str` and `execute(args: &str) -> String`. Args is the raw string after the command name, preserving original formatting.

**Why not typed args:**
- Each command does its own parsing (user confirmed this approach)
- Keeps the trait minimal and universal
- Commands like `/echo` need no parsing; others can use regex/struct parsing internally

### D5: Command Registration — Static Match Statement

**Decision:** `find()` function uses a `match` on command name string to return `Box<dyn SlashCommand>`.

**Why not inventory/linkme:**
- No new dependencies
- Explicit registration is easier to audit
- Only a handful of commands expected

### D6: Naming Validation — Regex at Config Load + Command Dispatch

**Decision:** Profile names validated in `load_config()` against `^[a-z0-9_-]+$`. Command names validated in handler before calling `find()`.

**Error behavior:**
- Invalid profile name: `load_config()` returns error, process exits
- Invalid command name: handler replies "Invalid command" to Discord

## Risks / Trade-offs

- **[Risk] Config format confusion** → Mitigated by keeping fallback; sample config shows both formats
- **[Risk] Command name collisions with Discord** → `/` prefix is common; no conflict with Discord's native `/` commands since this is text-based, not API-based
- **[Risk] Performance of match statement** → Negligible; linear scan over <20 commands
- **[Trade-off] No command help system** → Acceptable for initial implementation; can add `/help` later
- **[Trade-off] Per-profile targets not inheritable** → Intentional; keeps profiles isolated. Users duplicate if needed.
