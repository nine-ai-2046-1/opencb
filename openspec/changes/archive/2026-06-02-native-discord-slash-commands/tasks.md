## 1. SlashCommand Trait Update

- [x] 1.1 Add `CommandContext` struct to `src/slash_commands/mod.rs` with `args: String` and `message: MessageMetadata`
- [x] 1.2 Add `description() -> &str` method to `SlashCommand` trait
- [x] 1.3 Change `SlashCommand::execute()` signature to `execute(&self, ctx: &CommandContext) -> String`
- [x] 1.4 Add `all_commands() -> Vec<Box<dyn SlashCommand>>` function for registration

## 2. Echo Command Update

- [x] 2.1 Update `EchoCommand` to implement new trait signature with `description()` and `execute(ctx)`
- [x] 2.2 Echo returns `ctx.args` verbatim (same behavior, new interface)

## 3. Native Discord Registration

- [x] 3.1 Add `register_all_commands(http: &Http, app_id: UserId)` function in `slash_commands::mod.rs`
- [x] 3.2 In `ServeHandler::ready()`, call `register_all_commands()` with bot's user ID
- [x] 3.3 Handle registration errors gracefully (log + continue, don't crash)

## 4. Interaction Handler

- [x] 4.1 Add `interaction_create` event handler to `ServeHandler`
- [x] 4.2 Extract command name and options from `ChatInput` interaction
- [x] 4.3 Build `CommandContext` from interaction data (user, channel, guild, options as args)
- [x] 4.4 Route to `find()` and execute, respond with `CreateInteractionResponse`
- [x] 4.5 Handle unknown commands with "Invalid command" response

## 5. Text-Based Fallback Update

- [x] 5.1 Update `message()` handler to build `CommandContext` from `MessageMetadata` + parsed args
- [x] 5.2 Pass `CommandContext` to `command.execute()` instead of raw args string

## 6. Testing

- [x] 6.1 Test: `all_commands()` returns all registered commands
- [x] 6.2 Test: `EchoCommand.description()` returns expected string
- [x] 6.3 Test: `EchoCommand.execute(ctx)` returns `ctx.args` verbatim
- [x] 6.4 Test: `CommandContext` fields are accessible from command
- [x] 6.5 Test: existing unit tests still pass
