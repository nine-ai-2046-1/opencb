## ADDED Requirements

### Requirement: Global default config path
The system SHALL use `~/.config/opencb/config.toml` as the global default config when no `--profile` flag is provided.

#### Scenario: No --profile flag
- **WHEN** user runs `opencb serve` without `--profile`
- **THEN** the system SHALL load `~/.config/opencb/config.toml`

#### Scenario: --config flag overrides
- **WHEN** user runs `opencb serve --config /path/to/config.toml`
- **THEN** the system SHALL load the specified file instead of the default

### Requirement: Per-profile config directory
When `--profile <id>` is provided, the system SHALL look for config at `~/.config/opencb/<id>/config.toml`.

#### Scenario: Profile config exists
- **WHEN** user runs `opencb serve --profile work` and `~/.config/opencb/work/config.toml` exists
- **THEN** the system SHALL load that file

#### Scenario: Profile config missing — auto-create
- **WHEN** user runs `opencb serve --profile work` and `~/.config/opencb/work/config.toml` does not exist
- **THEN** the system SHALL create `~/.config/opencb/work/` directory
- **AND** copy `~/.config/opencb/config.toml` to `~/.config/opencb/work/config.toml`
- **AND** print the path to the new config file
- **AND** exit without processing the command

### Requirement: Flat config format
Each config file SHALL use flat format without `[profiles]` sections. The config file IS the profile.

#### Scenario: Flat config loaded
- **WHEN** a config file contains `bot_token = "abc"`, `channel_ids = ["*"]`, `cli_only = true`
- **THEN** the system SHALL load these as top-level Config fields

#### Scenario: Old [profiles] format rejected
- **WHEN** a config file contains `[profiles.default]` section
- **THEN** the system SHALL NOT parse it as profiles (sections are ignored or cause error)

### Requirement: Config struct is flat
The `Config` struct SHALL contain profile fields directly (bot_token, channel_ids, cli_only, etc.) without a `profiles` HashMap.

#### Scenario: Config accessed directly
- **WHEN** config is loaded
- **THEN** `config.bot_token`, `config.channel_ids`, `config.cli_only` SHALL be accessible directly

### Requirement: --profile on all commands
All CLI commands (serve, send, profiles, future commands) SHALL support the `--profile <id>` flag.

#### Scenario: serve with --profile
- **WHEN** user runs `opencb serve --profile work`
- **THEN** the system SHALL use `~/.config/opencb/work/config.toml`

#### Scenario: send with --profile
- **WHEN** user runs `opencb send --profile work "hello"`
- **THEN** the system SHALL use `~/.config/opencb/work/config.toml`
