## Requirements

### Requirement: Default profile auto-setup
When `opencb serve` (or any command using default profile) runs and the config is missing or incomplete, the system SHALL start an interactive setup wizard.

#### Scenario: Config file missing
- **WHEN** `~/.config/opencb/default/config.toml` does not exist
- **THEN** the system SHALL create the directory and config, then start interactive setup

#### Scenario: bot_token missing or placeholder
- **WHEN** config exists but `bot_token` is empty or "YOUR_BOT_TOKEN_HERE"
- **THEN** the system SHALL start interactive setup to fill bot_token

#### Scenario: channel_ids empty
- **WHEN** config exists but `channel_ids` is empty
- **THEN** the system SHALL start interactive setup to fill channel_ids

#### Scenario: Setup completed
- **WHEN** user completes interactive setup and confirms
- **THEN** the system SHALL write the config and continue with the original command

#### Scenario: Setup cancelled
- **WHEN** user cancels interactive setup
- **THEN** the system SHALL exit with an error message

### Requirement: Flattened profiles commands
The profiles command SHALL use one level of subcommands: `new`, `rm`, `show`, `set`.

#### Scenario: profiles new
- **WHEN** user runs `opencb profiles new "id"`
- **THEN** the system SHALL create the profile (same as old `profiles add`)

#### Scenario: profiles rm
- **WHEN** user runs `opencb profiles rm "id"`
- **THEN** the system SHALL delete the profile (same as old `profiles remove`)

#### Scenario: profiles show
- **WHEN** user runs `opencb profiles show "id"`
- **THEN** the system SHALL display config key-value pairs (same as old `profiles config show`)

#### Scenario: profiles set
- **WHEN** user runs `opencb profiles set "id" "key" "values..."`
- **THEN** the system SHALL update the config value (same as old `profiles config set`)

### Requirement: profiles add renamed to profiles new
The `profiles add` command SHALL be renamed to `profiles new`.

#### Scenario: profiles new creates profile
- **WHEN** user runs `opencb profiles new "work" --bot-token "abc123"`
- **THEN** the system SHALL create the profile

### Requirement: profiles remove renamed to profiles rm
The `profiles remove` command SHALL be renamed to `profiles rm`.

#### Scenario: profiles rm deletes profile
- **WHEN** user runs `opencb profiles rm "work"`
- **THEN** the system SHALL delete the profile directory
