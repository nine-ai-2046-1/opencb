## 1. CLI Flatten

- [x] 1.1 Remove `ConfigSubcommand` enum from `src/cli.rs`
- [x] 1.2 Remove `Profiles::Config` variant — move `Show` and `Set` to be direct subcommands of `Profiles`
- [x] 1.3 Rename `Profiles::Add` to `Profiles::New`
- [x] 1.4 Rename `Profiles::Remove` to `Profiles::Rm`

## 2. Default Profile Auto-Setup

- [x] 2.1 In `load_config()`, after loading default profile, check if `bot_token` is empty/placeholder or `channel_ids` is empty
- [x] 2.2 If incomplete, call `profile_manager::add_profile()` in interactive mode
- [x] 2.3 If user completes setup, write config and return loaded Config
- [x] 2.4 If user cancels, exit with clear error message

## 3. Profile Manager Updates

- [x] 3.1 Update `add_profile()` to return `Result<Config>` instead of `Result<()>`
- [x] 3.2 Update `remove_profile()` call sites for renamed command
- [x] 3.3 Update `show_config()` call sites for renamed command
- [x] 3.4 Update `set_config()` call sites for renamed command

## 4. Main.rs Updates

- [x] 4.1 Update match arms for `Profiles::New` (was `Add`)
- [x] 4.2 Update match arms for `Profiles::Rm` (was `Remove`)
- [x] 4.3 Update match arms for `Profiles::Show` and `Profiles::Set` (was nested under Config)
- [x] 4.4 Update default `profiles` list to work with new structure

## 5. Verify

- [x] 5.1 Run `cargo build` to confirm compilation
- [x] 5.2 Run `cargo test` to confirm all tests pass
- [x] 5.3 Run `cargo clippy` to check for warnings
