## Why

The current config system stores all profiles in a single `~/.config/opencb/config.toml` with `[profiles.<name>]` sections. This couples profiles together and makes it hard to manage per-profile configurations independently. Users need a cleaner separation where each profile has its own config file in its own directory, and the global config serves as a shared default.

## What Changes

- **New directory structure**: `~/.config/opencb/<profile_id>/config.toml` per profile, with `~/.config/opencb/config.toml` as global default
- **Flat config format**: Remove `[profiles]` and `[profiles.<name>]` sections — each config file is a flat single-profile config
- **Profile resolution**: When `--profile <id>` is used, check `~/.config/opencb/<id>/config.toml`. If missing, create directory, copy default config, print path, and exit
- **New `profiles` command**: List all profile directories under `~/.config/opencb/` with their paths
- **Config struct simplification**: Remove `profiles: HashMap` from `Config` — Config becomes the profile itself

## Capabilities

### New Capabilities
- `per-profile-config`: Per-profile config directory structure with automatic setup on first use
- `profiles-command`: CLI command to list available profiles and their config paths

### Modified Capabilities

## Impact

- **Files**: `src/config.rs` (major rewrite), `src/cli.rs` (add `Profiles` command), `src/main.rs` (update all profile access), `src/handler.rs` (use Config directly)
- **Config format**: Breaking change — old `[profiles]` format no longer supported
- **Directory structure**: New `~/.config/opencb/<profile>/` directories created on first `--profile` use
