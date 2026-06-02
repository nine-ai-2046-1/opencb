## Context

OpenCB's slash command system currently uses text-based parsing: the `message()` handler detects messages starting with `/`, extracts the command name, and calls `SlashCommand::execute(args: &str)`. This works but:

- Users don't see autocomplete or command descriptions in Discord's UI
- Commands only receive the args string, not the full message context (author, channel, attachments, etc.)
- serenity (the Discord library) has built-in support for native slash commands via `CreateSlashCommand` and `interaction_create`

**Current state:**
- `SlashCommand` trait: `name() -> &str`, `execute(args: &str) -> String`
- `find()` function returns `Box<dyn SlashCommand>`
- `ServeHandler::message()` parses `/cmd args` text
- One command: `/echo` (returns args verbatim)

**Constraints:**
- Must keep text-based `/command` as fallback (some users may type it manually)
- No new crate dependencies (serenity already supports slash commands)
- Command registration happens once on bot startup

## Goals / Non-Goals

**Goals:**
- Register all slash commands with Discord API on bot startup
- Add `description()` to `SlashCommand` trait for Discord UI display
- Pass full message/interaction context to commands (author, channel, guild, attachments, message ID)
- Handle both native interactions and text-based fallback
- Easy for developers to add new commands (just implement trait + register)

**Non-Goals:**
- Discord options/parameters system (keep simple string args for now)
- Removing text-based fallback
- Subcommands or command groups
- Ephemeral responses or interaction-only modes

## Decisions

### D1: Trait Signature — `execute(ctx, args)`

**Decision:** Change `SlashCommand::execute` to receive a `CommandContext` struct containing the `MessageMetadata` plus the raw args string.

```rust
pub struct CommandContext {
    pub args: String,
    pub message: MessageMetadata,
}

pub trait SlashCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, ctx: &CommandContext) -> String;
}
```

**Why not just pass `MessageMetadata` and let commands split args themselves?**
- The `args` field is pre-parsed convenience — every command needs it
- Avoids duplicating the split logic in every command

**Why a struct instead of multiple parameters?**
- Extensible — can add fields later without breaking trait signature
- Clean API: `ctx.args`, `ctx.message.author.name`

### D2: Registration — `register_all_commands()` on startup

**Decision:** Add a `register_all_commands(http: &Http, app_id: UserId)` function in `slash_commands::mod` that iterates all commands and calls serenity's `CreateSlashCommand::create_global_application_command`.

Called once in `ServeHandler::ready()`.

**Why global commands instead of guild-specific?**
- Simpler — one registration, works everywhere
- Guild commands update instantly but are limited to one guild
- Global commands take ~1 hour to propagate but work everywhere

### D3: Interaction Handler — `interaction_create` event

**Decision:** Add `interaction_create` to `ServeHandler` that:
1. Checks if the interaction is a `ChatInput` command
2. Extracts the command name and options
3. Builds a `CommandContext` from the interaction data
4. Routes to `find()` and executes

For text-based fallback: the existing `message()` handler continues to parse `/command args`.

### D4: CommandContext Construction

**Decision:** For native interactions, `CommandContext` is built from `Interaction` data (command name, options, user, channel, guild). For text-based fallback, it's built from `MessageMetadata` + parsed args.

```rust
// From interaction
let ctx = CommandContext {
    args: interaction.data.options...,
    message: MessageMetadata { /* from interaction */ },
};

// From text message
let ctx = CommandContext {
    args: parsed_args.to_string(),
    message: extract_message_metadata(&ctx, &msg),
};
```

### D5: Error Handling for Registration

**Decision:** Log errors during command registration but don't crash the bot. If registration fails (e.g., rate limited), the bot still starts and text-based fallback works.

## Risks / Trade-offs

- **[Risk] Registration rate limits** → Discord limits global command creation to 200 per hour. With <10 commands, this is fine.
- **[Risk] 1-hour propagation delay** → Global commands take up to 1 hour to appear. Mitigated by text-based fallback working immediately.
- **[Trade-off] No options/parameters** → Discord supports typed options (string, integer, etc.). We keep simple string args for simplicity. Can add later.
- **[Trade-off] Two code paths** → Native interactions and text-based parsing both route to the same `execute()`. This is intentional for backward compatibility.
