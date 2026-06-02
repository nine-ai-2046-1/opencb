## ADDED Requirements

### Requirement: SlashCommand trait definition
The system SHALL define a `SlashCommand` trait in `src/slash_commands/mod.rs` with methods `name() -> &str` and `execute(args: &str) -> String`. All slash commands SHALL implement this trait.

#### Scenario: Trait implementation
- **WHEN** a new command struct implements `SlashCommand`
- **THEN** it SHALL be callable via `name()` to get its name and `execute(args)` to run it

### Requirement: Command registration and lookup
The system SHALL provide a `find(command_name: &str) -> Option<Box<dyn SlashCommand>>` function in `slash_commands::mod` that routes command names to their implementations.

#### Scenario: Find registered command
- **WHEN** `find("echo")` is called
- **THEN** it SHALL return `Some(Box<EchoCommand>)`

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
The system SHALL organize slash commands in `src/slash_commands/` with one file per command. Adding a new command SHALL require: (1) creating a new file implementing `SlashCommand`, (2) adding a match arm in `find()`.

#### Scenario: New command file structure
- **WHEN** developer creates `src/slash_commands/news.rs` with `NewsCommand` struct
- **THEN** it SHALL be importable from `slash_commands::mod` and registerable in `find()`
