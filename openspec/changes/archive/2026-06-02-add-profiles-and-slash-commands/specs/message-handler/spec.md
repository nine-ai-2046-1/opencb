## ADDED Requirements

### Requirement: Ignore bot's own messages
The system SHALL ignore all messages where `msg.author.id` matches the bot's own user ID. This prevents infinite loops when the bot sends messages to Discord channels.

#### Scenario: Bot receives its own message
- **WHEN** a message is sent by the bot itself
- **THEN** the handler SHALL return immediately without processing

### Requirement: Channel ID filtering
The system SHALL filter guild messages by channel ID. If the profile's `channel_ids` is `["*"]`, all channels are accepted. Otherwise, only messages from channel IDs present in the array SHALL be processed.

#### Scenario: Wildcard channel accepts all
- **WHEN** profile has `channel_ids = ["*"]` and a guild message arrives
- **THEN** the message SHALL be processed regardless of channel

#### Scenario: Specific channels filter
- **WHEN** profile has `channel_ids = ["123", "456"]` and a message arrives in channel `"789"`
- **THEN** the message SHALL be ignored

#### Scenario: Specific channels accept matching
- **WHEN** profile has `channel_ids = ["123", "456"]` and a message arrives in channel `"123"`
- **THEN** the message SHALL be processed

### Requirement: DM messages always accepted
The system SHALL accept Direct Messages regardless of channel_ids configuration. DM messages SHALL bypass the channel filter entirely.

#### Scenario: DM with wildcard channels
- **WHEN** a DM is received and profile has `channel_ids = ["*"]`
- **THEN** the DM SHALL be processed

#### Scenario: DM with specific channels
- **WHEN** a DM is received and profile has `channel_ids = ["123"]`
- **THEN** the DM SHALL still be processed (channel filter does not apply)

### Requirement: Only slash-prefixed messages processed
The system SHALL only process messages that start with `/`. Messages not starting with `/` SHALL be ignored (no CLI target execution, no output).

#### Scenario: Message with slash prefix
- **WHEN** a message starts with `/` (e.g., `/echo hello`)
- **THEN** it SHALL be parsed as a slash command

#### Scenario: Message without slash prefix
- **WHEN** a message does not start with `/` (e.g., `search what is wiki`)
- **THEN** the handler SHALL ignore it completely

### Requirement: Command parsing and routing
The system SHALL parse the first word after `/` as the command name and the remainder as args. The command name SHALL be validated, then routed to `slash_commands::find()`. If not found, the system SHALL reply "Invalid command".

#### Scenario: Valid command found
- **WHEN** message is `/echo hello world`
- **THEN** command `"echo"` SHALL be looked up and executed with args `"hello world"`

#### Scenario: Command not found
- **WHEN** message is `/nonexistent hello`
- **THEN** the system SHALL reply `Invalid command`

#### Scenario: Command with no args
- **WHEN** message is `/echo`
- **THEN** command `"echo"` SHALL be executed with args `""`

### Requirement: Reply to Discord channel
The system SHALL send command output back to the same Discord channel where the message was received. DM commands SHALL reply via DM.

#### Scenario: Guild command reply
- **WHEN** a command is executed from a guild channel
- **THEN** the output SHALL be sent to that same channel

#### Scenario: DM command reply
- **WHEN** a command is executed from a DM
- **THEN** the output SHALL be sent as a DM reply
