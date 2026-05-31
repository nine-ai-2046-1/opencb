## Why

When users send messages with line breaks or markdown formatting (like `# headings`) to Discord, the bot loses these formatting elements. The current implementation collapses all whitespace into single spaces and doesn't preserve markdown in CLI output replies. This degrades the readability of multi-line responses and formatted text.

## What Changes

- Remove aggressive whitespace normalization that collapses line breaks in incoming messages
- Preserve markdown formatting in CLI output sent back to Discord
- Ensure Discord's markdown rendering works for bot replies

## Capabilities

### New Capabilities

- `message-formatting`: Preserve line breaks and markdown in Discord message handling

### Modified Capabilities

(none - this is new functionality)

## Impact

- `src/handler.rs`: Modify whitespace normalization logic (lines 64-69)
- `src/outbound.rs`: May need to adjust message content handling for markdown
- Discord message rendering will now show formatted text properly
