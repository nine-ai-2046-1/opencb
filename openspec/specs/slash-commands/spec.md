## ADDED Requirements

### Requirement: SlashCommand trait definition
The system SHALL define a `SlashCommand` trait in `src/slash_commands/mod.rs` with:
- `fn name(&self) -> &str` (sync)
- `fn description(&self) -> &str` (sync)
- `fn options(&self) -> Vec<CreateCommandOption>` (sync, default `vec![]`)
- `async fn execute(&self, ctx: &CommandContext) -> String` — all commands MUST implement this
- `async fn execute_with_updates(&self, ctx: &CommandContext, handle: &ResponseHandle)` — default implementation calls `self.execute(ctx).await` and then `handle.finalize(&result).await`; streaming commands MAY override this method

#### Scenario: Trait implementation
- **WHEN** a new command struct implements `SlashCommand`
- **THEN** it SHALL be callable via `name()` to get its name and `execute(ctx).await` to run it

#### Scenario: Simple command uses default execute_with_updates
- **WHEN** a command implements only `async fn execute()` and `execute_with_updates()` is called
- **THEN** the default implementation SHALL call `execute()` once and pass the result to `handle.finalize()`

#### Scenario: Streaming command overrides execute_with_updates
- **WHEN** a command overrides `execute_with_updates()` and the handler calls it
- **THEN** the command MAY call `handle.update()` multiple times before calling `handle.finalize()`

### Requirement: Command registration and lookup
The system SHALL provide a `find(command_name: &str) -> Option<CommandDispatch>` function in `slash_commands::mod` that routes command names to their implementations via enum dispatch.

#### Scenario: Find registered command
- **WHEN** `find("echo")` is called
- **THEN** it SHALL return `Some(CommandDispatch::Echo)`

#### Scenario: Find registered cli command
- **WHEN** `find("cli")` is called
- **THEN** it SHALL return `Some(CommandDispatch::Cli)`

#### Scenario: Find unregistered command
- **WHEN** `find("nonexistent")` is called
- **THEN** it SHALL return `None`

### Requirement: Command name validation
The system SHALL validate command names match `^[a-z0-9_-]+$` before calling `find()`. Invalid names SHALL result in an "Invalid command" reply.

#### Scenario: Valid command name
- **WHEN** message is `/echo hello`
- **THEN** command name `"echo"` SHALL be validated and passed to `find()`

#### Scenario: Invalid command name with space
- **WHEN** message is `/my command hello`
- **THEN** the system SHALL reply "Invalid command" without calling `find()`

#### Scenario: Invalid command name with special character
- **WHEN** message is `/cmd! hello`
- **THEN** the system SHALL reply "Invalid command" without calling `find()`

### Requirement: Echo command implementation
The system SHALL implement an `/echo` command in `src/slash_commands/echo.rs` that returns the args string exactly as received, preserving all spacing, line breaks, markdown, and formatting.

#### Scenario: Echo with simple text
- **WHEN** user sends `/echo Hello World`
- **THEN** the system SHALL reply `Hello World`

#### Scenario: Echo preserves spacing
- **WHEN** user sends `/echo Hello   World` (multiple spaces)
- **THEN** the system SHALL reply `Hello   World` (spaces preserved)

#### Scenario: Echo preserves line breaks
- **WHEN** user sends `/echo Hello\nWorld`
- **THEN** the system SHALL reply with the line break preserved

#### Scenario: Echo preserves markdown
- **WHEN** user sends `/echo **bold** and _italic_`
- **THEN** the system SHALL reply `**bold** and _italic_`

#### Scenario: Echo with no args
- **WHEN** user sends `/echo`
- **THEN** the system SHALL reply with an empty string

### Requirement: Modular command structure
The system SHALL organize slash commands in `src/slash_commands/` with one file per command. Adding a new command SHALL require: (1) creating a new file implementing `SlashCommand`, (2) adding a variant to `CommandDispatch` and match arms in `find()`, `all_commands()`, and `CommandDispatch` delegation methods.

#### Scenario: New command file structure
- **WHEN** developer creates `src/slash_commands/news.rs` with `NewsCommand` struct
- **THEN** it SHALL be importable from `slash_commands::mod` and registerable via `CommandDispatch`

### Requirement: ResponseHandle for interaction updates
The system SHALL provide a `ResponseHandle` struct in `src/slash_commands/mod.rs` that wraps `Arc<serenity::all::Http>` and an interaction token string. It SHALL expose:
- `async fn update(&self, content: &str)` — edits the deferred interaction response with new content
- `async fn finalize(&self, content: &str)` — makes the final edit to the interaction response

#### Scenario: Intermediate update via handle
- **WHEN** `handle.update("step 1 done")` is called
- **THEN** the Discord interaction message SHALL be edited to show `"step 1 done"`

#### Scenario: Final update via handle
- **WHEN** `handle.finalize("all done")` is called
- **THEN** the Discord interaction message SHALL be edited to show `"all done"` as the final response
