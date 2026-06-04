## Why

The bot has no way for Discord users to invoke nine-cli skills directly. Adding a `/cli` slash command bridges Discord and the nine-cli skill runtime, enabling users on a private server to run any installed skill and receive streaming output updates as the skill executes — including long-running skills that may take several minutes.

## What Changes

- Add a new `/cli` slash command that accepts free-form arguments, tokenizes them (quote-aware), and spawns `nine-cli` with the resulting argv.
- Upgrade the `SlashCommand` trait from synchronous to `async fn execute()` so commands can perform I/O.
- Introduce a `ResponseHandle` abstraction that carries a Discord HTTP client and interaction token, allowing commands to push incremental status updates to Discord during execution.
- Add `execute_with_updates()` as a second trait method with a default implementation that calls `execute()` once — simple commands like `/echo` require no changes beyond the async upgrade.
- Add a `libs/argv-parser` library that implements a quote-aware argument tokenizer (state machine, no new crate dependencies).
- Update the interaction handler to defer the Discord response immediately, then drive `execute_with_updates()` for up to several minutes.

## Capabilities

### New Capabilities

- `cli-slash-command`: The `/cli` command — accepts a free-form string, tokenizes it (preserving quoted groups), spawns `nine-cli`, streams stdout line-by-line, edits the Discord message every ~2 seconds with accumulated output, and finalises with the completed result.
- `argv-parser`: A standalone quote-aware argument tokenizer library. Splits on whitespace while respecting single- and double-quoted groups. Returns tokens with quotes stripped and content preserved. Lives in `libs/argv-parser/`.

### Modified Capabilities

- `slash-commands`: The `SlashCommand` trait gains `async fn execute()` and a new default method `async fn execute_with_updates(ctx, handle)`. The `CommandContext` struct is unchanged. The `find()` and `register_all_commands()` functions are unchanged.

## Impact

- `src/slash_commands/mod.rs` — trait signature change (async), new `ResponseHandle` struct, new `execute_with_updates()` default method.
- `src/slash_commands/echo.rs` — add `async` keyword to `execute()`; no logic change.
- `src/slash_commands/cli.rs` — new file implementing `CliCommand`.
- `src/slash_commands/mod.rs` `find()` / `all_commands()` — add `"cli"` entry.
- `src/handler.rs` — interaction handler uses `defer_response()` before calling `execute_with_updates()`; creates `ResponseHandle` with HTTP + interaction token.
- `libs/argv-parser/mod.rs` — new library; no external crate dependencies added.
- Rust 1.75+ `async fn in trait` used natively (current toolchain: 1.94.1 ✓).
