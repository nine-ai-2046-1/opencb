## ADDED Requirements

### Requirement: Literal escape sequences converted to real characters
The system SHALL convert literal escape sequences in message content to their actual characters before sending to Discord. Specifically, `\n` SHALL be converted to a newline character (U+000A), and `\r\n` SHALL be converted to a carriage-return + newline (U+000D U+000A).

#### Scenario: Single newline in message
- **WHEN** user sends `opencb send "Line1\nLine2"`
- **THEN** the Discord message SHALL contain two lines: "Line1" on the first line and "Line2" on the second line

#### Scenario: Double newline in message
- **WHEN** user sends `opencb send "Para1\n\nPara2"`
- **THEN** the Discord message SHALL contain two paragraphs separated by a blank line

#### Scenario: Windows-style newline
- **WHEN** user sends `opencb send "Line1\r\nLine2"`
- **THEN** the Discord message SHALL contain two lines: "Line1" on the first line and "Line2" on the second line

### Requirement: Escaped escape sequences preserved as literals
The system SHALL convert `\\n` to the literal two-character sequence `\n` (backslash followed by n) in the Discord message, allowing users to display escape sequences literally.

#### Scenario: Escaped newline displayed literally
- **WHEN** user sends `opencb send "Show me \\n literally"`
- **THEN** the Discord message SHALL contain the text "Show me \n literally" on a single line

#### Scenario: Mixed escaped and real newlines
- **WHEN** user sends `opencb send "Line1\nLine2\\nLine3"`
- **THEN** the Discord message SHALL contain "Line1" on line one, "Line2\nLine3" on line two

### Requirement: All send paths apply escape conversion
The system SHALL apply escape sequence conversion to all outbound message paths: CLI immediate send, CLI DM send, scheduled job send, scheduled job DM send, and serve mode outbound send.

#### Scenario: CLI channel send
- **WHEN** a message is sent via `opencb send "A\nB"` to a channel
- **THEN** the message content sent to Discord SHALL have actual newlines

#### Scenario: CLI DM send
- **WHEN** a message is sent via `opencb send "A\nB" --ru "12345"`
- **THEN** the DM content sent to Discord SHALL have actual newlines

#### Scenario: Scheduled send
- **WHEN** a scheduled job fires with message content containing `\n`
- **THEN** the message content sent to Discord SHALL have actual newlines

#### Scenario: Serve mode reply
- **WHEN** the bot replies to a Discord mention with content containing `\n`
- **THEN** the reply message sent to Discord SHALL have actual newlines

### Requirement: Messages without escape sequences unchanged
The system SHALL NOT alter message content that does not contain literal escape sequences.

#### Scenario: Plain text message
- **WHEN** user sends `opencb send "Hello World"`
- **THEN** the Discord message SHALL contain exactly "Hello World" with no modifications

#### Scenario: Message with markdown only
- **WHEN** user sends `opencb send "**bold** and _italic_"`
- **THEN** the Discord message SHALL contain exactly "**bold** and _italic_" (Discord renders as bold and italic)
