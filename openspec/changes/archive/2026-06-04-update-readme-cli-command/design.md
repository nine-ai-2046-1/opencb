## Context

This is a pure documentation change following the `add-cli-slash-command` implementation. Both `README.md` and `README-ZH.md` were last updated after the `update-readme-send-profile-changes` change (2026-06-04) and do not reflect the `/cli` command, `libs/argv-parser`, async trait upgrade, `ResponseHandle`, or the new test count of 72.

## Goals / Non-Goals

**Goals:**
- Both READMEs accurately reflect the current state of the codebase after `add-cli-slash-command`.
- A reader can understand what `/cli` does and how to use it from the README alone.
- Module tables and directory trees match the actual file structure.

**Non-Goals:**
- Rewriting or restructuring the README beyond the stale sections.
- Documenting internal implementation details (async trait internals, `CommandDispatch` enum) — these are developer concerns, not user-facing.
- Updating `TECH.md` (scope limited to the two READMEs).

## Decisions

### D1 — Update only the stale sections, not a full README rewrite

Four discrete sections need updating in each file: slash commands table, directory tree, module table, test count. Editing only these sections minimises diff noise and reduces risk of accidentally breaking other content.

### D2 — Describe `/cli` streaming behaviour briefly in the slash commands section

Users need to know that `/cli` shows live progress updates and may run for several minutes — this is functionally different from `/echo` and worth calling out. One or two sentences under the table is sufficient; full technical detail belongs in the code.

### D3 — Keep README-ZH.md in sync with README.md section-for-section

The Chinese README mirrors the English one. All changes applied to `README.md` SHALL have equivalent Chinese content applied to `README-ZH.md`.

## Risks / Trade-offs

- **Directory tree drift** → The tree is manually maintained. Risk of it drifting again after future changes. No mitigation beyond discipline; a future change could automate this.
- **Test count becoming stale again** → Hardcoded number. Acceptable for now; a CI badge would be a better long-term solution.
