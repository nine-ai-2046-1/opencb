## Context

The `opencb send` command sends messages to Discord channels. When users include `\n` in their message arguments (e.g., `opencb send "Hello\nWorld"`), the shell passes the literal two-character sequence `\n` rather than an actual newline character. Discord's API only renders actual newline characters as line breaks, so messages appear on a single line. The code at `src/main.rs:228` joins parts with spaces via `parts.join(" ")` and passes the result directly to `CreateMessage::new().content()`.

There are five send paths in the codebase:
- CLI immediate channel send (`main.rs:404-407`)
- CLI DM send (`main.rs:385-389`)
- Scheduled job channel send (`main.rs:484-486`)
- Scheduled job DM send (`main.rs:508-511`)
- Serve mode outbound send (`outbound.rs:14-16`)

All paths pass content directly to Discord without any text transformation.

## Goals / Non-Goals

**Goals:**
- Convert literal `\n` sequences in message content to actual newline characters before sending to Discord
- Support `\r\n` (Windows-style) and `\n\n` (double newline / paragraph break)
- Ensure the fix applies to all five send paths
- Preserve existing behavior for messages that don't contain literal escape sequences

**Non-Goals:**
- Adding markdown escaping or sanitization (Discord already interprets markdown in the `content` field)
- Adding embed support or rich message formatting
- Changing how the shell or clap parses arguments
- Adding a custom markdown-to-Discord converter

## Decisions

### Decision: Where to perform the escape sequence conversion

**Chosen approach**: Create a helper function `fn process_message_content(s: &str) -> String` that converts literal escape sequences to their real characters. Apply it at the point where the final message string is constructed, right before passing to `send_message`.

**Rationale**: Centralizing the conversion in one function makes it testable and ensures all send paths use the same logic. Applying it at the last moment (before send) means the cleaned message string is used consistently everywhere.

**Alternative considered**: Converting in `extract_time_date_message` — rejected because that function is about flag parsing, not message formatting, and scheduled messages would bypass it.

### Decision: Which escape sequences to support

**Chosen approach**: Convert `\n` to newline and `\r\n` to carriage-return-newline. Also handle `\\n` as a literal backslash-n (escaped escape).

**Rationale**: These are the most common escape sequences users will type. Supporting `\\n` as an escape hatch prevents unwanted line breaks when a user literally wants to show `\n` in the message. The conversion should process `\\` first (to `\\`) then `\n` to avoid double-conversion.

### Decision: No changes to outbound.rs for serve mode

**Chosen approach**: The serve mode send (`outbound.rs`) receives its content from the handler module, which already normalizes whitespace. The `process_message_content` function should also be applied there for consistency, but serve mode messages are typically bot replies, not user-typed escape sequences. Apply the function defensively.

## Risks / Trade-offs

- **[Risk] Double-conversion of already-converted content** → Mitigation: Process `\\` → `\` first, then `\n` → newline, so `\\n` becomes literal `\n` correctly
- **[Risk] Users who intentionally want literal `\n` displayed** → Mitigation: Support `\\n` as the escape for literal `\n`
- **[Trade-off] Only `\n` and `\r\n` supported, not arbitrary `\t`, `\0`, etc.** → Acceptable because newline is the primary pain point; other escapes are rarely needed in Discord messages
