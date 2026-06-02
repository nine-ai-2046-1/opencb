## ADDED Requirements

### Requirement: CommandContext struct
The system SHALL define a `CommandContext` struct containing:
- `args: String` — the raw arguments string after the command name
- `message: MessageMetadata` — full message metadata (id, content, author, channel, guild, mentions, attachments, etc.)

This struct SHALL be passed to `SlashCommand::execute()`.

#### Scenario: Context contains args and message
- **WHEN** a command is executed
- **THEN** `ctx.args` SHALL contain the arguments text and `ctx.message` SHALL contain the full message metadata

### Requirement: MessageMetadata accessible from command
Each slash command SHALL have access to `ctx.message.author.id`, `ctx.message.author.name`, `ctx.message.channel.id`, `ctx.message.guild`, `ctx.message.attachments`, and all other `MessageMetadata` fields.

#### Scenario: Command reads author info
- **WHEN** a command accesses `ctx.message.author.name`
- **THEN** it SHALL return the display name of the user who invoked the command

#### Scenario: Command reads channel info
- **WHEN** a command accesses `ctx.message.channel.id`
- **THEN** it SHALL return the channel ID where the command was invoked

#### Scenario: Command reads attachments
- **WHEN** a message has file attachments and the user invokes a command
- **THEN** `ctx.message.attachments` SHALL contain the attachment metadata (id, filename, size, url)

### Requirement: Context built from native interaction
When handling a native Discord interaction, the system SHALL construct `CommandContext` from the interaction's data: command options become `args`, and interaction metadata (user, channel, guild) populate `message`.

#### Scenario: Native interaction context
- **WHEN** a native `/echo hello` interaction is received
- **THEN** `ctx.args` SHALL be `"hello"` and `ctx.message.author.id` SHALL match the interacting user's ID

### Requirement: Context built from text message
When handling a text-based `/command args` message, the system SHALL construct `CommandContext` from the message's `MessageMetadata` and the parsed args string.

#### Scenario: Text message context
- **WHEN** a text message `/echo hello world` is received
- **THEN** `ctx.args` SHALL be `"hello world"` and `ctx.message` SHALL be the full `MessageMetadata` of that message
