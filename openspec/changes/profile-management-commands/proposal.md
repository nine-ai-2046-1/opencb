## Why

Users need CLI commands to create, view, modify, and delete profiles without manually editing config files. Additionally, when the default profile is missing or incomplete, the system should guide users through setup interactively instead of failing with cryptic errors.

## What Changes

- **Flatten profiles commands**: `opencb profiles new/rm/show/set` instead of `opencb profiles config show/set`
- **Default profile auto-setup**: When `opencb serve` runs without `--profile` and the default config is missing/incomplete, start interactive wizard to fill required fields
- **`opencb profiles new "id"`**: Interactive profile creation with CLI flag prefill
- **`opencb profiles rm "id"`**: Delete a profile directory
- **`opencb profiles show "id"`**: Display config key-value pairs
- **`opencb profiles set "id" "key" "values..."`**: Set config values. Arrays accept multiple args. Targets skipped with info message
- **Update `load_config()`**: Detect incomplete config and trigger interactive setup

## Capabilities

### New Capabilities
- `profile-management`: CLI commands for creating, viewing, modifying, and deleting profiles with interactive mode support
- `default-profile-setup`: Auto-setup wizard when default profile is missing or incomplete

### Modified Capabilities
- `per-profile-config`: Config loading triggers interactive setup when default profile is incomplete

## Impact

- **Files**: `src/cli.rs` (flatten subcommands), `src/config.rs` (auto-setup logic), `src/main.rs` (handle new commands), `src/profile_manager.rs` (update function names)
- **Behavior**: `opencb serve` without `--profile` auto-creates and guides setup of default profile
