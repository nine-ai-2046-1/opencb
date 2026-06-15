## Context

The profile management commands were implemented with a nested structure (`profiles config show/set`). This needs to be flattened to `profiles show/set` for better UX. Additionally, the default profile should auto-setup interactively when missing or incomplete.

## Goals / Non-Goals

**Goals:**
- Flatten CLI: `profiles new/rm/show/set` (one level, not two)
- Auto-setup default profile when incomplete (interactive wizard)
- Keep existing functionality intact

**Non-Goals:**
- Changing the config file format
- Adding new config fields

## Decisions

### Decision: Flatten commands

**Chosen approach**: Remove `Profiles::Config` subcommand. Move `Show` and `Set` to be direct subcommands of `Profiles`.

**Before:**
```
opencb profiles config show "id"
opencb profiles config set "id" k v
```

**After:**
```
opencb profiles show "id"
opencb profiles set "id" k v
```

**Rationale**: One level of subcommands is easier to remember and type.

### Decision: Auto-setup trigger

**Chosen approach**: In `load_config()`, after loading the default profile, check if required fields are present. If `bot_token` is empty or placeholder, or `channel_ids` is empty, trigger interactive setup.

**Rationale**: Proactive setup prevents confusing errors later. The user gets guided through configuration on first use.

### Decision: Auto-setup flow

**Chosen approach**: When auto-setup triggers:
1. Create directory + config file with defaults
2. Call `add_profile()` in interactive mode (reuse existing function)
3. After setup completes, return the loaded config
4. If user cancels setup, exit with error

**Rationale**: Reuse existing interactive logic rather than duplicating it.

## Risks / Trade-offs

- **[Risk] User cancels default setup → can't proceed** → Mitigation: Exit with clear message explaining they need to set up config
- **[Trade-off] Auto-setup on every serve if config is incomplete** → Acceptable — only triggers when fields are actually missing
