## Context

Both `README.md` (English) and `README-ZH.md` (Cantonese) were written when the codebase used a flat config format (`bot_token`, `channel_id` at top level). Since then, three significant changes landed:

1. **Profile-based config** (`[profiles.<name>]`) — the canonical format; flat config is now legacy fallback only
2. **`default_send_to_channel_ids`** — a new required field for the `send` command when `channel_ids = ["*"]`
3. **Send command flag fixes** — `--profile`, `--rc`, `--ru`, `--mu` now work correctly (removed `trailing_var_arg`); multi-word messages no longer consume flags as message text
4. **Native Discord slash commands** — `/echo` auto-registered on bot start via `interaction_create` handler

Users reading the old README will either get config parse errors or use the wrong send syntax.

## Goals / Non-Goals

**Goals:**
- Rewrite Configuration section in both READMEs to show profiles format
- Document `default_send_to_channel_ids` with explanation of wildcard vs send context
- Update all `send` command examples to show correct flag usage
- Add slash commands section
- Update module table and test count
- Keep both files in sync (README.md English, README-ZH.md Cantonese)

**Non-Goals:**
- Rewriting unrelated sections (build, install, FAQ structure)
- Adding new features or changing any code
- Generating API documentation

## Decisions

### Decision 1: Profiles format as the only documented format

The flat config (`bot_token` / `channel_id` at top level) still works as a fallback, but documenting both formats would confuse users. Document only the profiles format as the canonical approach; mention legacy flat format briefly as a compatibility note.

**Alternative considered:** Document both formats side by side → Rejected: doubles the config section length and implies both are equal options.

### Decision 2: Show `--rc` as a per-send override, not the primary channel config

Users should configure `default_send_to_channel_ids` in their profile and only use `--rc` for one-off overrides. README examples will reflect this hierarchy.

**Alternative considered:** Show `--rc` as the primary mechanism → Rejected: encourages users to skip `default_send_to_channel_ids` and always type the channel ID manually.

### Decision 3: Update both README.md and README-ZH.md in the same change

Both files must stay in sync. Doing them separately risks one falling behind. Treat them as a single atomic update.

### Decision 4: Preserve README structure (sections, headings order)

Rewrite only the sections that are outdated (Configuration, Usage/Send, Usage/Serve, Project Structure, Testing). Keep Build, Install, FAQ, and License sections intact to minimise diff noise.

## Risks / Trade-offs

- [Risk] README diverges from code again in future → Mitigation: openspec change process makes documentation updates explicit; any future code change to send/config should include a README update task
- [Risk] Cantonese README falls behind English → Mitigation: tasks list both files explicitly so neither can be skipped

## Migration Plan

1. Update `README.md` sections in order: Configuration → Send command → Serve command → Modules table → Test count
2. Mirror all changes in `README-ZH.md`
3. Verify config sample block matches `config.sample.toml`
4. No code changes; no deployment steps required
