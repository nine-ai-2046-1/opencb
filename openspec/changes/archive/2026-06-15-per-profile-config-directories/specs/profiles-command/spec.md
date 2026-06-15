## ADDED Requirements

### Requirement: profiles command lists all profiles
The `opencb profiles` command SHALL list all available profiles by scanning subdirectories under `~/.config/opencb/` that contain a `config.toml` file.

#### Scenario: Multiple profiles exist
- **WHEN** `~/.config/opencb/` contains `work/config.toml` and `chatbot/config.toml`
- **THEN** the command SHALL output both profile names and their config paths

#### Scenario: No profiles exist
- **WHEN** `~/.config/opencb/` has no subdirectories with `config.toml`
- **THEN** the command SHALL display a message indicating no profiles are configured

#### Scenario: Global config exists
- **WHEN** `~/.config/opencb/config.toml` exists (the global default)
- **THEN** the command SHALL list it as the default profile with path `~/.config/opencb/config.toml`

### Requirement: profiles command output format
The output SHALL show each profile's name and full config file path.

#### Scenario: Profile listing output
- **WHEN** user runs `opencb profiles`
- **THEN** output SHALL include profile name and absolute path for each profile

#### Scenario: Profile with --config flag
- **WHEN** user runs `opencb profiles --config /custom/path/config.toml`
- **THEN** the command SHALL scan the parent directory of the specified config file for profile subdirectories
