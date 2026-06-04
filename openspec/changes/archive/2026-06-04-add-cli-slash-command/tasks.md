## 1. Branch Setup

- [x] 1.1 Create git branch `dev/add-cli-slash-command`

## 2. argv-parser Library

- [x] 2.1 Create `libs/argv-parser/mod.rs` with `tokenize_argv(input: &str) -> Vec<String>` state-machine implementation (Normal / InDoubleQuote / InSingleQuote states)
- [x] 2.2 Add module declaration so `libs/argv-parser` is reachable from `src/` (add `mod` path or adjust `Cargo.toml`)
- [x] 2.3 Write unit tests in `libs/argv-parser/mod.rs` covering: empty input, bare tokens, double-quoted, single-quoted, mixed, leading/trailing whitespace, adjacent quoted tokens, unclosed quote
- [x] 2.4 Run `cargo test` — all argv-parser tests pass

## 3. SlashCommand Trait Upgrade

- [x] 3.1 Add `ResponseHandle` struct to `src/slash_commands/mod.rs` with fields `http: Arc<Http>`, `application_id: ApplicationId`, `interaction_token: String`
- [x] 3.2 Implement `ResponseHandle::update(&self, content: &str)` — calls `edit_original_interaction_response`
- [x] 3.3 Implement `ResponseHandle::finalize(&self, content: &str)` — calls `edit_original_interaction_response` (final edit)
- [x] 3.4 Change `fn execute(&self, ctx: &CommandContext) -> String` to `async fn execute(&self, ctx: &CommandContext) -> String` in the trait
- [x] 3.5 Add `async fn execute_with_updates(&self, ctx: &CommandContext, handle: &ResponseHandle)` with default impl: calls `self.execute(ctx).await` then `handle.finalize(&result).await`
- [x] 3.6 Run `cargo build` — confirm trait changes compile (will break EchoCommand, fix next)

## 4. Update EchoCommand

- [x] 4.1 Add `async` keyword to `EchoCommand::execute()` in `src/slash_commands/echo.rs` — no logic change
- [x] 4.2 Run `cargo test` — all 49 existing tests still pass

## 5. Update Handler

- [x] 5.1 In `handler.rs` `interaction_create`, replace immediate `CreateInteractionResponse::Message(...)` with `defer_response()` call before command dispatch
- [x] 5.2 Construct `ResponseHandle` from `ctx.http`, `command.application_id`, and `command.token` after deferring
- [x] 5.3 Replace `command_impl.execute(&cmd_ctx)` call with `command_impl.execute_with_updates(&cmd_ctx, &handle).await`
- [x] 5.4 Remove the post-execute `command.create_response()` call (now handled by ResponseHandle inside the command)
- [x] 5.5 Run `cargo build` — handler compiles cleanly

## 6. CliCommand Implementation

- [x] 6.1 Create `src/slash_commands/cli.rs` with `CliCommand` struct
- [x] 6.2 Implement `SlashCommand::name()` → `"cli"`, `description()` → description string, `options()` → one required String option named `"args"`
- [x] 6.3 Implement `async fn execute()` — delegate to `execute_with_updates` (or keep minimal for simple path)
- [x] 6.4 Override `async fn execute_with_updates()`:
  - Tokenize `ctx.args` via `tokenize_argv()`
  - Send initial `handle.update()` with `🔄  nine-cli <args>\n────\n...`
  - Spawn `tokio::process::Command::new("nine-cli").args(&tokens).stdout(Stdio::piped()).stderr(Stdio::piped())`
  - Read stdout line-by-line with `tokio::io::BufReader` + `AsyncBufReadExt::lines()`
  - Accumulate output; edit Discord at most every 2 seconds via `handle.update()`
  - Apply rolling 1800-char window truncation when needed
  - On process exit: call `handle.finalize()` with success/failure header + output
- [x] 6.5 Handle `nine-cli` not found in PATH — catch spawn error and call `handle.finalize("❌  nine-cli not found in PATH")`
- [x] 6.6 Handle non-zero exit code — include exit code in final message
- [x] 6.7 Wrap entire spawn+stream block in `tokio::time::timeout(Duration::from_secs(600), ...)` — on expiry kill child and call `handle.finalize("⏱️  nine-cli ... timed out after 10 minutes")`

## 7. Command Registration

- [x] 7.1 Add `mod cli;` to `src/slash_commands/mod.rs`
- [x] 7.2 Add `"cli" => Some(Box::new(cli::CliCommand))` to `find()`
- [x] 7.3 Add `Box::new(cli::CliCommand)` to `all_commands()`
- [x] 7.4 Run `cargo build` — full project compiles with zero warnings

## 8. Tests

- [x] 8.1 Write unit tests for `CliCommand::name()`, `description()`, `options()` in `src/slash_commands/cli.rs`
- [x] 8.2 Write unit test: `find("cli")` returns `Some`
- [x] 8.3 Write unit test: `all_commands()` length is now 2
- [x] 8.4 Run `cargo test` — all tests pass (including all 49 pre-existing tests)

## 9. Verification

- [x] 9.1 Run `cargo clippy` — zero warnings or errors
- [x] 9.2 Start bot locally; verify `/cli` appears in Discord command picker
- [x] 9.3 Invoke `/cli` with a fast skill; confirm deferred response + final output appears in Discord
- [x] 9.4 Invoke `/cli` with a skill that produces multiple lines; confirm status updates appear during execution
- [x] 9.5 Invoke `/echo` — confirm it still works correctly (no regression)
