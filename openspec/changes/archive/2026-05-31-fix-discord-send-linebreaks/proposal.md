## Why

When sending messages via `opencb send`, line breaks (`\n`, `\n\n`, etc.) are lost and appear as a single line. This is because the shell passes literal backslash-n characters, which the code joins with spaces and passes directly to Discord. Discord only renders actual newline characters (`\n` in Unicode) as line breaks, not the literal two-character sequence `\n`. Additionally, markdown formatting (bold, italic, code blocks) may not render as expected depending on how the message is constructed.

## What Changes

- Add `\n` literal-to-newline conversion in the send path so that `opencb send "Hello\nWorld"` produces a two-line Discord message
- Ensure markdown content passes through to Discord correctly (it already should via `CreateMessage::new().content()`, but verify no escaping occurs)
- Apply the same fix to all send paths: CLI immediate send, CLI DM send, scheduled job send, and scheduled job DM send

## Capabilities

### New Capabilities
- `send-message-formatting`: Handles conversion of literal escape sequences (`\n`) to actual newlines in outbound Discord messages, ensuring multi-line messages and markdown render correctly

### Modified Capabilities

## Impact

- **Files**: `src/main.rs` (send command handler, `extract_time_date_message` function, scheduled send paths), `src/outbound.rs` (serve mode send)
- **APIs**: Discord message content field - messages will now contain actual newlines instead of literal `\n`
- **Behavior change**: Existing scripts or habits using `\n` in send arguments will start working as expected (multi-line). No breaking change - previously `\n` was a no-op, now it becomes a newline.
