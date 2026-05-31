## 1. Core Helper Function

- [x] 1.1 Add `process_message_content(s: &str) -> String` function in `src/main.rs` that converts literal escape sequences to real characters: `\\` → `\` first, then `\r\n` → CR+LF, then `\n` → LF
- [x] 1.2 Add unit tests for `process_message_content` covering: single `\n`, double `\n\n`, `\r\n`, `\\n` (escaped), mixed content, and plain text without escapes

## 2. Apply to CLI Send Paths

- [x] 2.1 In `main.rs` CLI immediate channel send (line ~406), apply `process_message_content` to `full_msg` before passing to `send_message`
- [x] 2.2 In `main.rs` CLI DM send (line ~386), apply `process_message_content` to `full_msg` before passing to `send_message`
- [x] 2.3 In `main.rs` scheduled job channel send (line ~485), apply `process_message_content` to `full_msg` before passing to `send_message`
- [x] 2.4 In `main.rs` scheduled job DM send (line ~510), apply `process_message_content` to `full_msg` before passing to `send_message`

## 3. Apply to Serve Mode

- [x] 3.1 In `outbound.rs`, apply `process_message_content` to `content` in `send_message_to_channel` before passing to `send_message`

## 4. Verify

- [x] 4.1 Run `cargo build` to confirm compilation
- [x] 4.2 Run `cargo test` to confirm all tests pass
- [x] 4.3 Run `cargo clippy` to check for warnings
