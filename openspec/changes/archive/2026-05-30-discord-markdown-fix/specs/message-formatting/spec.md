## ADDED Requirements

### Requirement: Preserve line breaks in incoming messages
The system SHALL preserve line breaks when processing incoming Discord messages for CLI input.

#### Scenario: Message with line breaks
- **WHEN** user sends a message containing line breaks to Discord
- **THEN** the bot passes the message content with line breaks preserved to the CLI

### Requirement: Preserve markdown in CLI output
The system SHALL preserve markdown formatting in CLI output when sending replies to Discord.

#### Scenario: CLI outputs markdown
- **WHEN** CLI execution produces output containing markdown (e.g., `# Heading`, `**bold**`)
- **THEN** the bot sends the markdown content to Discord without stripping formatting

### Requirement: Discord markdown rendering works
The system SHALL ensure markdown in bot replies renders properly in Discord.

#### Scenario: Formatted reply displayed
- **WHEN** bot sends a reply containing markdown to Discord
- **THEN** Discord renders the markdown (headings, bold, lists, etc.) correctly
