## 1. README.md — Configuration Section

- [x] 1.1 Replace flat config example (`bot_token`, `channel_id`) with profiles format (`[profiles.default]`)
- [x] 1.2 Add `default_send_to_channel_ids` field to the config example with explanatory comment
- [x] 1.3 Add note explaining `channel_ids = ["*"]` is for serve mode; `default_send_to_channel_ids` is for send mode
- [x] 1.4 Add brief legacy compatibility note for flat config format

## 2. README.md — Send Command Section

- [x] 2.1 Update basic send example (no flags) to show it uses `default_send_to_channel_ids`
- [x] 2.2 Add `--rc` example showing single-channel override
- [x] 2.3 Add `--profile` example showing non-default profile usage
- [x] 2.4 Add `--ru` example showing DM send
- [x] 2.5 Add `--mu` example showing mention append
- [x] 2.6 Clarify multi-word message usage (with and without quotes)

## 3. README.md — Serve Command Section

- [x] 3.1 Add `--profile` flag to serve command examples
- [x] 3.2 Add slash commands section: explain `/echo` is auto-registered on bot start

## 4. README.md — Project Structure & Modules

- [x] 4.1 Add `slash_commands/` module to the directory tree
- [x] 4.2 Add `splitter.rs` and `scheduler.rs` to the directory tree
- [x] 4.3 Update modules table to include `slash_commands/mod.rs`, `splitter.rs`, `scheduler.rs`
- [x] 4.4 Update test count from 4 to 49

## 5. README-ZH.md — Configuration Section

- [x] 5.1 Replace flat config example with profiles format (Cantonese)
- [x] 5.2 Add `default_send_to_channel_ids` field with Cantonese comment
- [x] 5.3 Add wildcard vs send-mode explanation in Cantonese
- [x] 5.4 Add legacy compatibility note in Cantonese

## 6. README-ZH.md — Send Command Section

- [x] 6.1 Update basic send example (Cantonese)
- [x] 6.2 Add `--rc`, `--profile`, `--ru`, `--mu` examples in Cantonese
- [x] 6.3 Clarify multi-word message usage in Cantonese

## 7. README-ZH.md — Serve Command Section

- [x] 7.1 Add `--profile` flag to serve examples (Cantonese)
- [x] 7.2 Add slash commands section in Cantonese

## 8. README-ZH.md — Project Structure & Modules

- [x] 8.1 Add `slash_commands/`, `splitter.rs`, `scheduler.rs` to directory tree (Cantonese)
- [x] 8.2 Update modules table (Cantonese)
- [x] 8.3 Update test count from 4 to 49 (Cantonese)

## 9. Verification

- [x] 9.1 Cross-check README.md config block matches `config.sample.toml`
- [x] 9.2 Verify all send flag examples are syntactically correct
- [x] 9.3 Confirm README.md and README-ZH.md cover the same topics
