## Context

The bot currently implements slash commands via a synchronous `SlashCommand` trait (`fn execute() -> String`). All existing commands (`/echo`) return immediately with no I/O. The proposal introduces `/cli`, which shells out to `nine-cli` — a local skill runner that can execute arbitrary installed skills, potentially running for several minutes and producing streaming output.

Discord slash command interactions have a hard 3-second timeout before the "This interaction failed" message appears. Long-running commands therefore require a deferred response pattern: acknowledge within 3 seconds, then edit the message repeatedly as output arrives.

Current Rust toolchain is 1.94.1, which natively supports `async fn` in traits (stabilised in 1.75). No new crate dependencies are required.

## Goals / Non-Goals

**Goals:**
- Upgrade `SlashCommand` trait to `async fn execute()` with zero behaviour change for existing commands.
- Introduce `ResponseHandle` so commands can push incremental updates to Discord during execution.
- Implement `CliCommand` that tokenizes free-form input (quote-aware) and streams `nine-cli` stdout back to Discord with ~2-second edit intervals.
- Extract a standalone `libs/argv-parser` module — quote-aware tokenizer, independently testable.
- Keep `/echo` and all future simple commands unaware of streaming concerns.

**Non-Goals:**
- Shell-level piping, redirection, or glob expansion in user-supplied arguments.
- Per-user access control or skill whitelisting (private server, open to all members).
- Persisting command output to disk or database.
- Supporting more than one concurrent `/cli` invocation guard (rate limiting is a future concern).

## Decisions

### D1 — Native `async fn in trait` (no `async-trait` crate)

Rust 1.75+ supports `async fn` directly in traits without proc macros. Since the project is on 1.94.1, we use this natively. The `async-trait` crate is not added.

*Alternative considered*: `async-trait` proc macro — rejected because it adds a dependency and is no longer necessary on this toolchain.

### D2 — Two-method trait: `execute()` + `execute_with_updates()`

Simple commands implement only `async fn execute() -> String`. The trait provides a default `execute_with_updates(ctx, handle)` that calls `execute()` once and calls `handle.finalize()`. Streaming commands (e.g., `CliCommand`) override `execute_with_updates()` directly.

```
SlashCommand trait
  async fn execute(ctx) -> String              ← required; simple commands implement this
  async fn execute_with_updates(ctx, handle)   ← default: calls execute() + finalize()
                                                 streaming commands override this
```

*Alternative considered*: A single `async fn execute(ctx, handle)` for all commands — rejected because it forces every simple command to be aware of `ResponseHandle`, increasing coupling unnecessarily.

### D3 — `ResponseHandle` carries HTTP + interaction token

`ResponseHandle` wraps `Arc<serenity::all::Http>` and the interaction token string. It exposes two async methods: `update(content)` (edit in progress) and `finalize(content)` (final edit). The handler constructs it after deferring and passes it into `execute_with_updates()`.

```
ResponseHandle {
    http: Arc<Http>,
    application_id: ApplicationId,
    interaction_token: String,
}
```

Discord's `edit_original_interaction_response` endpoint is used for both `update()` and `finalize()`.

### D4 — Deferred response in handler, not in commands

The handler calls `defer_response()` before constructing `ResponseHandle`. Commands never need to know about deferral — they only call `handle.update()` / `handle.finalize()`. This keeps the streaming abstraction clean and keeps Discord protocol concerns out of command implementations.

### D5 — Rate-limited Discord edits (2-second interval)

Discord's global rate limit for editing messages is lenient, but editing on every stdout line would be wasteful and could trigger client-side spam warnings. The `CliCommand` accumulates stdout lines and edits only when ≥2 seconds have elapsed since the last edit.

*Alternative considered*: Edit on every line — rejected due to potential rate limit issues and noisy updates for fast-output skills.

### D6 — Rolling 1800-character output window

Discord messages are capped at 2000 characters. The status header ("🔄 `nine-cli ...`\n────\n") uses ~100 characters. When accumulated output exceeds ~1800 characters, the display shows `[...earlier output truncated...]\n` followed by the most recent 1600 characters. The full output is never lost — only the Discord display is windowed.

### D7 — `libs/argv-parser` as an internal Rust module (not a workspace crate)

A simple state-machine tokenizer does not justify a separate Cargo workspace member at this stage. It lives at `libs/argv-parser/mod.rs` and is declared as a module path in `Cargo.toml` or included via `src/lib.rs`. It is independently unit-tested. If it grows into a reusable crate it can be extracted later.

*Alternative considered*: Cargo workspace crate — over-engineering for a ~60-line tokenizer; deferred.

### D8 — `nine-cli` resolved via `PATH`

No configuration entry is added for the binary path. The bot is deployed on the same machine where `nine-cli` is installed globally. `tokio::process::Command::new("nine-cli")` resolves via PATH at runtime. If the binary is missing, the error is surfaced to Discord as a command failure message.

## Risks / Trade-offs

- **Long-running skill hangs forever** → `CliCommand` sets a configurable timeout (default 10 minutes) via `tokio::time::timeout`. On expiry it kills the child process and replies with a timeout error message.
- **nine-cli not in PATH at bot startup** → Failure surfaces at invocation time, not startup. A future enhancement could validate at `ready()`.
- **Concurrent `/cli` invocations** → Multiple users can trigger simultaneous subprocesses. No guard is added now (private server, low risk). A semaphore can be added later.
- **Deferred response token expiry** → Discord interaction follow-up tokens are valid for 15 minutes. Skills running beyond 15 minutes will fail to edit the message. The 10-minute timeout mitigates this.
- **`async fn in trait` object safety** → Rust's current implementation of `async fn in trait` is not object-safe (cannot be used as `dyn SlashCommand` with async methods directly). The existing `find()` returns `Box<dyn SlashCommand>`. This requires using `async_trait`-style workaround or switching to an enum dispatch. **Decision**: Use an explicit `enum Dispatch` or keep `Box<dyn SlashCommand>` with a concrete `execute_with_updates` that is NOT part of the trait's dyn interface — instead, the handler matches on command name and calls a concrete type. See Open Questions.

## Migration Plan

1. Upgrade `SlashCommand` trait to `async fn execute()` and add `execute_with_updates()` with default impl.
2. Update `EchoCommand::execute()` — add `async` keyword only, no logic change.
3. Add `ResponseHandle` struct to `slash_commands/mod.rs`.
4. Update `handler.rs` interaction path to defer + construct `ResponseHandle` + call `execute_with_updates()`.
5. Add `libs/argv-parser/mod.rs` with tokenizer + unit tests.
6. Add `CliCommand` in `src/slash_commands/cli.rs`.
7. Register `/cli` in `find()` and `all_commands()`.
8. Run all tests; verify `/echo` still passes all 49 existing tests.

Rollback: revert the trait change and remove `cli.rs`. `/echo` is unaffected by removing `/cli` registration.

## Open Questions

- **`dyn SlashCommand` + async**: Rust async fn in trait is not yet `dyn`-compatible without a shim. The current `find()` returns `Box<dyn SlashCommand>`. We need to decide: (a) use a concrete enum for dispatch instead of `dyn`, or (b) split the trait into a sync registry trait and an async execution trait. This must be resolved before implementation starts.
