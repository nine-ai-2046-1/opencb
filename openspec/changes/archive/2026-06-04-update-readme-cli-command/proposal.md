## Why

The `add-cli-slash-command` change introduced the `/cli` command, `ResponseHandle`, an async `SlashCommand` trait, `CommandDispatch` enum dispatch, and the `libs/argv-parser` library. Neither `README.md` nor `README-ZH.md` reflects any of these additions — the slash commands table, module structure diagram, module description table, and test count are all stale.

## What Changes

- Add `/cli` command entry to the slash commands table in both READMEs, including a description of streaming behaviour.
- Update the module directory tree to include `libs/argv-parser/mod.rs` and `src/slash_commands/cli.rs`.
- Update the module description table rows for `slash_commands/mod.rs` (now includes `ResponseHandle`, async trait, `CommandDispatch`) and add rows for `slash_commands/cli.rs` and `argv-parser`.
- Update the test count from `49` to `72` in both READMEs.
- Add a brief note about the deferred-response / streaming update pattern under the slash commands section.

## Capabilities

### New Capabilities

- `readme-documentation`: Requirements for what `README.md` and `README-ZH.md` must document — slash commands table, module structure, test count, and streaming behaviour description.

### Modified Capabilities

*(none — no spec-level behaviour changes)*

## Impact

- `README.md` — slash commands table, module tree, module table, test count.
- `README-ZH.md` — same sections, Chinese content.
- No code changes. No dependency changes.
