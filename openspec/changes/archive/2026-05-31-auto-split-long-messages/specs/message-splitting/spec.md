## ADDED Requirements

### Requirement: Character counting uses Unicode codepoints
The system SHALL count message characters using Unicode codepoints (`.chars().count()` in Rust), not byte length (`.len()`). The Discord 2000-character limit SHALL be enforced using character count.

#### Scenario: CJK content within limit
- **WHEN** a message contains 1900 CJK characters (5700 bytes in UTF-8)
- **THEN** the system SHALL treat it as within the 2000-character limit and send it as a single message

#### Scenario: CJK content exceeds limit
- **WHEN** a message contains 2100 CJK characters (6300 bytes in UTF-8)
- **THEN** the system SHALL split it into multiple messages, each ≤2000 characters

#### Scenario: Mixed language content
- **WHEN** a message contains 1000 English characters + 1000 Japanese characters
- **THEN** the system SHALL treat it as exactly 2000 characters and send as a single message

### Requirement: Content parsed into atomic and splittable segments
The system SHALL parse message content into typed segments. Each segment type has a distinct splitting behavior.

**Atomic segments** (never split internally, sent whole in one message):
- URL (`http://` or `https://` prefix)
- Base64 data (continuous alphanumeric + `+/=` characters, no spaces/newlines)
- Fenced code block (triple backtick pair)
- Inline code (single backtick pair)

**Splittable segments** (can be split at internal boundaries):
- Continuous string (no spaces or newlines)
- Normal text (contains spaces or newlines)

#### Scenario: URL preserved intact across messages
- **WHEN** a message contains a 2500-character URL that exceeds the Discord limit
- **THEN** the URL SHALL be sent as a single message, intact, not split

#### Scenario: URL in mixed content sent whole
- **WHEN** a message is 3000 characters total, containing a 500-character URL plus other text
- **THEN** the URL SHALL be kept intact, and the split SHALL occur at text boundaries around the URL

#### Scenario: Base64 data preserved intact
- **WHEN** a message contains a 1800-character base64 string
- **THEN** the base64 string SHALL be sent intact in one message (under 2000 limit)

#### Scenario: Base64 data exceeds limit sent whole
- **WHEN** a message contains a 2500-character base64 string
- **THEN** the base64 string SHALL be sent as a single message, intact, not split

#### Scenario: Code block preserved intact
- **WHEN** a message contains a fenced code block (````...````) that would be split mid-block
- **THEN** the entire code block SHALL be kept intact in one message

#### Scenario: Inline code preserved intact
- **WHEN** a message contains inline code (`` `...` ``) that would be split mid-code
- **THEN** the inline code SHALL be kept intact in one message

### Requirement: Continuous string split at 164 characters
A continuous string is a sequence of characters with no spaces or newlines. If it exceeds 164 characters, it SHALL be split. The split point SHALL be near 164, preferring a punctuation or dash boundary within the last 20 characters (range 144-164). If no punctuation is found, hard-cut at 164.

#### Scenario: Short continuous string kept intact
- **WHEN** a message contains a continuous string of 150 characters (no spaces/newlines)
- **THEN** the string SHALL NOT be split and remains in one segment

#### Scenario: Long continuous string split near punctuation
- **WHEN** a continuous string is 200 characters and contains a comma at position 158
- **THEN** the system SHALL split at position 158 (the comma), producing segments of 158 and 42 characters

#### Scenario: Long continuous string hard split
- **WHEN** a continuous string is 200 characters with no punctuation in positions 144-164
- **THEN** the system SHALL split at position 164, producing segments of 164 and 36 characters

#### Scenario: Very long continuous string split into multiple parts
- **WHEN** a continuous string is 500 characters
- **THEN** the system SHALL split it into multiple segments, each ≤164 characters

### Requirement: Split strategy cascade for text segments
Normal text segments (containing spaces or newlines) SHALL be split in the following order of preference:
1. At newline (`\n`) boundaries — best for readability
2. At space boundaries — fallback for single long lines

#### Scenario: Split at newline preferred
- **WHEN** a text segment contains newlines every 200 characters
- **THEN** the system SHALL split at newline boundaries

#### Scenario: Split at space for single long line
- **WHEN** a text segment is a single line with spaces but no newlines
- **THEN** the system SHALL split at space boundaries

### Requirement: Each split message within Discord limit
Every message produced by the splitter SHALL contain ≤2000 Unicode codepoints.

#### Scenario: Two messages for 3000-char content
- **WHEN** a 3000-character message is split
- **THEN** exactly 2 messages SHALL be produced, each ≤2000 characters, totaling the original content

#### Scenario: Many messages for very long content
- **WHEN** a 20000-character message is split
- **THEN** the system SHALL produce 10+ messages, each ≤2000 characters, with no content lost

### Requirement: Both send paths use splitting
The system SHALL apply message splitting to both the CLI send command path and the serve mode reply path.

#### Scenario: Send command splits long message
- **WHEN** user runs `opencb send` with a 2500-character message
- **THEN** the message SHALL be split into 2 messages and both SHALL be sent to Discord

#### Scenario: Serve mode reply splits long output
- **WHEN** a bot reply to a mention exceeds 2000 characters
- **THEN** the reply SHALL be split into multiple messages and all SHALL be sent to Discord

### Requirement: Sequential sending with rate-limit delay
The system SHALL send split messages sequentially with a small delay between each to avoid Discord rate limits.

#### Scenario: Delay between messages
- **WHEN** a message is split into 3 parts
- **THEN** the system SHALL send part 1, wait ~100ms, send part 2, wait ~100ms, send part 3

#### Scenario: No delay for single message
- **WHEN** a message does not need splitting
- **THEN** the system SHALL send it immediately without any delay

### Requirement: No content loss
The system SHALL preserve 100% of the original message content across all split messages. No characters SHALL be dropped, truncated, or replaced with ellipsis.

#### Scenario: Complete content delivery
- **WHEN** a 5000-character message containing mixed CJK, English, URLs, and code blocks is split
- **THEN** the concatenation of all split messages SHALL equal the original 5000-character message exactly
