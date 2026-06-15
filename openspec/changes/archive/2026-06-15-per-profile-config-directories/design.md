## Context

The current config system (`src/config.rs`) loads a single `config.toml` file that contains all profiles under `[profiles.<name>]` sections. The `Config` struct wraps a `profiles: HashMap<String, Profile>` and every command does `config.profiles.get(&name)` to resolve a profile.

The user wants to restructure this so each profile lives in its own directory with its own flat config file. The global config becomes a shared default.

## Goals / Non-Goals

**Goals:**
- Each profile has its own directory: `~/.config/opencb/<profile_id>/config.toml`
- Global default: `~/.config/opencb/config.toml` (flat format, no `[profiles]` section)
- When `--profile <id>` is used and directory doesn't exist: create it, copy default, print path, exit
- `opencb profiles` command lists all profile directories
- Config struct becomes flat — no more `profiles` HashMap

**Non-Goals:**
- Migrating old config files automatically (user must restructure manually)
- Supporting both old and new formats simultaneously

## Decisions

### Decision: Config struct simplification

**Chosen approach**: Remove `profiles: HashMap<String, Profile>` from `Config`. Config becomes the profile itself — flat fields directly on `Config`.

**Rationale**: Each config file is now a single profile. The HashMap indirection is no longer needed.

**Before:**
```rust
struct Config {
    profiles: HashMap<String, Profile>,
    bot_token: String,       // fallback
    channel_id: Vec<String>, // fallback
    ...
}
```

**After:**
```rust
struct Config {
    config_path: PathBuf,
    bot_token: String,
    channel_ids: Vec<String>,
    cli_only: bool,
    default_send_to_channel_ids: Vec<String>,
    targets: HashMap<String, TargetSpec>,
    owner_id: Vec<String>,
    debug: Option<bool>,
    scheduled_admin_bind: Option<String>,
}
```

### Decision: Config path resolution

**Chosen approach**: Two-tier resolution in `load_config()`:

1. No `--profile`: use `~/.config/opencb/config.toml`
2. With `--profile <id>`: check `~/.config/opencb/<id>/config.toml`
   - Missing → create dir, copy default, print path, return error to exit
   - Exists → load it

**Rationale**: Simple, predictable. The profile directory is always at a known location.

### Decision: Default config generation

**Chosen approach**: `render_default_toml()` generates flat format without `[profiles]` sections.

**Rationale**: The new format is flat per-profile. The default template should match.

### Decision: `profiles` command implementation

**Chosen approach**: Scan `~/.config/opencb/*/config.toml` using `std::fs::read_dir`. List directory names as profile IDs.

**Rationale**: Simple filesystem scan, no parsing needed. Only directories containing `config.toml` are listed.

## Risks / Trade-offs

- **[Risk] Breaking change for existing users** → Mitigation: Document in release notes. Users must restructure their config manually.
- **[Risk] Race condition on first `--profile` use** → Mitigation: `create_dir_all` + atomic write. Low risk for CLI tool.
- **[Trade-off] No backward compatibility with old format** → Acceptable since no known users yet.
