## Why

The current slash command system is text-based only — users type `/echo hello` as a regular message, and the handler parses it. This works but misses Discord's native slash command UX: autocomplete, command descriptions in the UI, structured option parsing, and the familiar `/` popup menu.

Additionally, the current `SlashCommand` trait only receives `args: &str`, limiting what commands can do. Commands like `/search` or `/news` need access to the message author, channel, attachments, and other metadata to provide richer functionality.

## What Changes

- **Upgrade `SlashCommand` trait**: Add `description()` method and change `execute()` to receive full `MessageMetadata` alongside the args string
- **Register commands with Discord API**: On bot startup (`ready` event), register all slash commands as Discord native slash commands using serenity's `CreateInteractionResponse` and command builder
- **Handle Discord interaction events**: Add `interaction_create` handler to intercept native slash command interactions and route them to the command modules
- **Keep backward compatibility**: The text-based `/command` parsing in `message()` handler continues to work as a fallback

## Capabilities

### New Capabilities
- `native-slash-registration`: Discord API registration of slash commands on bot startup, with per-command description metadata
- `command-context-passing`: Passing full message/interaction context to slash command modules (author, channel, guild, attachments, etc.)

### Modified Capabilities
- `slash-commands`: Trait updated with `description()` method and new `execute()` signature receiving context

## Impact

- **Code**: `src/slash_commands/mod.rs` (trait change + registration fn), `src/slash_commands/echo.rs` (updated impl), `src/handler.rs` (add `interaction_create` + update `message()` handler), `src/main.rs` (register commands on startup)
- **Dependencies**: No new crates — serenity already supports slash commands
- **Breaking changes**: The `SlashCommand` trait signature changes; all command implementations must update
