## Why

Long messages sent via `opencb send` or bot replies in serve mode are either truncated (losing content) or rejected by Discord's 2000-character API limit. The current `truncate()` function also uses byte length instead of character count, which is especially broken for CJK/Japanese/emoji content. Users need full message delivery without content loss, with intelligent splitting that preserves readability across mixed languages, URLs, code blocks, and base64 data.

## What Changes

- Replace the `truncate()` function with a new `split_message()` utility that splits long content into multiple Discord-compliant messages (≤2000 chars each)
- Fix character counting to use Unicode codepoints (`.chars().count()`) instead of byte length (`.len()`)
- Parse content into segments (text, URLs, code blocks, inline code) and split at safe boundaries between segments, never inside them
- Apply splitting to both the send command path and serve mode reply path
- Add rate-limit-aware sequential sending with small delays between split messages

## Capabilities

### New Capabilities
- `message-splitting`: Intelligent splitting of long messages into multiple Discord-compliant messages, preserving readability across multi-language content, URLs, code blocks, and structured data

### Modified Capabilities

## Impact

- **Files**: New utility module (e.g., `src/splitter.rs`), `src/handler.rs` (replace truncation), `src/main.rs` (apply splitting to send path), `src/outbound.rs` (shared send with splitting)
- **APIs**: Discord message sending — messages will now be delivered as multiple sequential messages instead of being truncated or rejected
- **Behavior change**: Long messages that previously failed or lost content will now be delivered in full across multiple messages
