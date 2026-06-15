## 1. Config Changes

- [x] 1.1 Add `cli_only: bool` field to `Profile` struct in `src/config.rs` with `#[serde(default = "default_cli_only")]`
- [x] 1.2 Add `default_cli_only()` function that returns `true`
- [x] 1.3 Update `config.sample.toml` with `cli_only` example and comment

## 2. Handler Logic

- [x] 2.1 In `src/handler.rs`, change rejection condition from `!content.starts_with('/')` to `self.profile.cli_only && !content.starts_with('/')`

## 3. Tests

- [x] 3.1 Add test: profile without `cli_only` defaults to `true`
- [x] 3.2 Add test: profile with `cli_only = true` rejects non-slash messages
- [x] 3.3 Add test: profile with `cli_only = false` accepts non-slash messages

## 4. Verify

- [x] 4.1 Run `cargo build` to confirm compilation
- [x] 4.2 Run `cargo test` to confirm all tests pass
- [x] 4.3 Run `cargo clippy` to check for warnings
