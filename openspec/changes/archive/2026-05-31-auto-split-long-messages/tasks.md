## 1. Core Splitter Module

- [x] 1.1 Create `src/splitter.rs` module with `pub fn split_message(content: &str, max_chars: usize) -> Vec<String>` signature
- [x] 1.2 Implement content segment parser: scan content into typed segments — atomic (URL, base64, code block, inline code) and splittable (continuous string, normal text)
- [x] 1.3 Implement continuous string splitter: split at 164 chars, preferring punctuation/dash boundary within positions 144-164, else hard cut at 164
- [x] 1.4 Implement text segment splitter: split at `\n` boundaries first, then space boundaries
- [x] 1.5 Implement message assembler: combine segments into messages ≤2000 chars, keeping atomic segments intact
- [x] 1.6 Implement `pub fn send_split_message(http: &Http, channel_id: ChannelId, content: &str, max_chars: usize)` with 100ms delay between sends

## 2. Unit Tests

- [x] 2.1 Test character counting: CJK content within limit sends as single message
- [x] 2.2 Test atomic segments: URLs preserved intact across splits
- [x] 2.3 Test atomic segments: code blocks preserved intact across splits
- [x] 2.4 Test atomic segments: base64 preserved intact across splits
- [x] 2.5 Test continuous string ≤164 chars kept intact
- [x] 2.6 Test continuous string >164 chars split near punctuation (144-164 range)
- [x] 2.7 Test continuous string >164 chars with no punctuation hard split at 164
- [x] 2.8 Test continuous string >164 chars split into multiple parts
- [x] 2.9 Test text split strategy: prefers newlines, falls back to spaces
- [x] 2.10 Test mixed content: CJK + English + URLs + code blocks split correctly
- [x] 2.11 Test no content loss: concatenation of splits equals original content
- [x] 2.12 Test within-limit message returns single-element vec unchanged

## 3. Integration: Send Command Path

- [x] 3.1 In `main.rs`, replace `process_message_content` send with `send_split_message` for CLI immediate channel send
- [x] 3.2 In `main.rs`, replace `process_message_content` send with `send_split_message` for CLI DM send
- [x] 3.3 In `main.rs`, replace `process_message_content` send with `send_split_message` for scheduled job channel send
- [x] 3.4 In `main.rs`, replace `process_message_content` send with `send_split_message` for scheduled job DM send

## 4. Integration: Serve Mode Path

- [x] 4.1 In `outbound.rs`, replace `send_message_to_channel` with `send_split_message` (or update `send_message_to_channel` to use splitting internally)
- [x] 4.2 In `handler.rs`, remove `truncate()` call and `DISCORD_MSG_LIMIT` constant — splitting handles the limit now

## 5. Verify

- [x] 5.1 Run `cargo build` to confirm compilation
- [x] 5.2 Run `cargo test` to confirm all tests pass
- [x] 5.3 Run `cargo clippy` to check for warnings
