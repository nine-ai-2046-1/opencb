## 1. Config Struct Rewrite

- [x] 1.1 Rewrite `Config` struct in `src/config.rs`: remove `profiles: HashMap`, add `config_path: PathBuf`, flatten Profile fields into Config
- [x] 1.2 Remove `Profile` struct (merge into Config)
- [x] 1.3 Update `load_config()` to support two-tier resolution: global default vs `--profile <id>` directory
- [x] 1.4 Add auto-create logic: when `--profile <id>` dir missing, create it, copy default, print path, return error
- [x] 1.5 Update `render_default_toml()` to generate flat format (no `[profiles]` sections)
- [x] 1.6 Update `validate_profile()` to validate Config directly

## 2. CLI Changes

- [x] 2.1 Add `Commands::Profiles` variant to `src/cli.rs`
- [x] 2.2 Add `--profile` flag to `Commands::Profiles` (for custom config path scanning)
- [x] 2.3 Ensure `--profile` flag is available on all commands (serve, send, profiles)

## 3. Main.rs Updates

- [x] 3.1 Update config loading in `main()` to use new `load_config()` signature
- [x] 3.2 Update serve command: remove `config.profiles.get()`, use Config directly
- [x] 3.3 Update send command: remove `config.profiles.get()`, use Config directly
- [x] 3.4 Implement `profiles` command: scan `~/.config/opencb/*/config.toml`, list names + paths

## 4. Handler Updates

- [x] 4.1 Update `ServeHandler` to hold `Config` directly instead of `Profile`
- [x] 4.2 Update all `self.profile.*` references to `self.config.*`

## 5. Outbound/Send Updates

- [x] 5.1 Update `send_message_to_channel` and `send_split_message` calls to use Config directly

## 6. Tests

- [x] 6.1 Update all tests that construct `Profile` or `Config` structs

## 7. Verify

- [x] 7.1 Run `cargo build` to confirm compilation
- [x] 7.2 Run `cargo test` to confirm all tests pass
- [x] 7.3 Run `cargo clippy` to check for warnings
