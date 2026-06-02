## MODIFIED Requirements

### Requirement: SlashCommand trait definition
The system SHALL define a `SlashCommand` trait in `src/slash_commands/mod.rs` with methods `name() -> &str`, `description() -> &str`, and `execute(ctx: &CommandContext) -> String`. All slash commands SHALL implement this trait.

#### Scenario: Trait implementation
- **WHEN** a new command struct implements `SlashCommand`
- **THEN** it SHALL provide `name()`, `description()`, and `execute(ctx)` methods

### Requirement: Command registration and lookup
The system SHALL provide a `find(command_name: &str) -> Option<Box<dyn SlashCommand>>` function and a `all_commands() -> Vec<Box<dyn SlashCommand>>` function that returns all registered commands for registration purposes.

#### Scenario: Find registered command
- **WHEN** `find("echo")` is called
- **THEN** it SHALL return `Some(Box<EchoCommand>)`

#### Scenario: Get all commands
- **WHEN** `all_commands()` is called
- **THEN** it SHALL return a vector containing all registered command instances

### Requirement: Echo command implementation
The system SHALL implement an `/echo` command that returns the args string exactly as received, with description "Echoes back the input message".

#### Scenario: Echo with context
- **WHEN** user sends `/echo Hello World`
- **THEN** the system SHALL reply `Hello World`

#### Scenario: Echo command description
- **WHEN** `EchoCommand.description()` is called
- **THEN** it SHALL return `"Echoes back the input message"`
