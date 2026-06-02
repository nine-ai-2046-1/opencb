## ADDED Requirements

### Requirement: SlashCommand trait includes description
The `SlashCommand` trait SHALL include a `description() -> &str` method that returns a human-readable description of what the command does. This description SHALL be used when registering the command with Discord's API.

#### Scenario: Command provides description
- **WHEN** a command implements `SlashCommand`
- **THEN** `description()` SHALL return a non-empty string describing the command's purpose

### Requirement: Command registration on bot startup
The system SHALL register all slash commands with Discord's API during the `ready` event. Registration SHALL use serenity's `CreateSlashCommand` builder with the command name and description from each `SlashCommand` implementation.

#### Scenario: All commands registered on startup
- **WHEN** the bot connects and fires the `ready` event
- **THEN** the system SHALL call `CreateSlashCommand::create_global_application_command` for each registered command

#### Scenario: Registration error does not crash bot
- **WHEN** a command registration fails (e.g., rate limit, API error)
- **THEN** the system SHALL log the error and continue starting the bot (text-based fallback remains available)

### Requirement: Commands visible in Discord UI
After registration, slash commands SHALL appear in Discord's autocomplete popup when users type `/` in a chat input.

#### Scenario: User sees command in autocomplete
- **WHEN** a user types `/` in Discord chat
- **THEN** registered commands (e.g., "echo") SHALL appear in the popup with their description

### Requirement: Interaction handler routes to commands
The system SHALL handle `interaction_create` events from Discord. When a `ChatInput` interaction matches a registered command name, the system SHALL execute the command and respond via `CreateInteractionResponse`.

#### Scenario: Native slash command interaction
- **WHEN** a user selects `/echo` from Discord's autocomplete and submits
- **THEN** the system SHALL route to `EchoCommand::execute()` and respond with the command output

#### Scenario: Unknown interaction
- **WHEN** an interaction references an unregistered command
- **THEN** the system SHALL respond with "Invalid command"
