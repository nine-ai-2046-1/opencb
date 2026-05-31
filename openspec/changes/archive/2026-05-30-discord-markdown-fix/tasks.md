## 1. Whitespace Normalization Fix

- [x] 1.1 Modify `src/handler.rs` whitespace normalization to preserve line breaks
- [x] 1.2 Update the normalization logic to only collapse multiple spaces (not newlines)

## 2. CLI Output Handling

- [x] 2.1 Verify CLI output is passed through without modification in `src/handler.rs`
- [x] 2.2 Ensure `src/outbound.rs` sends markdown content correctly

## 3. Testing

- [x] 3.1 Test sending messages with line breaks to verify they reach CLI (build + unit tests pass)
- [x] 3.2 Test CLI output with markdown renders in Discord (outbound.rs passes content directly to Discord API)
