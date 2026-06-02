## ADDED Requirements

### Requirement: Profile-based configuration structure
The system SHALL support a `[profiles]` section in `config.toml` where each sub-table represents a bot profile. Each profile SHALL contain: `profile_id` (string), `channel_type` (string, default `"discord"`), `channel_ids` (string array), `bot_token` (string), and optional `[targets]` sub-table.

#### Scenario: Config with multiple profiles
- **WHEN** `config.toml` contains `[profiles.default]` and `[profiles.work]` sections
- **THEN** `load_config()` SHALL parse both profiles into `Config.profiles` HashMap

#### Scenario: Profile with targets
- **WHEN** a profile section contains `[profiles.<name>.targets.<target>]` with `cmd` and `argv`
- **THEN** the target SHALL be stored in `Profile.targets` for that profile only

### Requirement: Profile name validation
The system SHALL validate that all profile names match the pattern `^[a-z0-9_-]+$`. Profile names containing spaces, uppercase letters, or special characters SHALL cause `load_config()` to return an error.

#### Scenario: Valid profile names
- **WHEN** `config.toml` contains profiles named `"default"`, `"work-bot"`, `"ai_v2"`, `"cmd1"`
- **THEN** `load_config()` SHALL accept all profiles without error

#### Scenario: Invalid profile name with space
- **WHEN** `config.toml` contains a profile named `"my bot"`
- **THEN** `load_config()` SHALL return an error indicating invalid profile name

#### Scenario: Invalid profile name with special character
- **WHEN** `config.toml` contains a profile named `"cmd!"`
- **THEN** `load_config()` SHALL return an error indicating invalid profile name

### Requirement: Channel IDs accept wildcard
The system SHALL allow `channel_ids` to contain the single value `"*"` to indicate all channels are accepted. When `"*"` is present, it SHALL be the only element in the array.

#### Scenario: Wildcard channel
- **WHEN** a profile has `channel_ids = ["*"]`
- **THEN** messages from any channel SHALL be accepted for that profile

#### Scenario: Specific channel IDs
- **WHEN** a profile has `channel_ids = ["123456", "789012"]`
- **THEN** only messages from those channel IDs SHALL be accepted

### Requirement: Fallback to top-level config
The system SHALL support the old flat config format as a fallback. When no `[profiles]` section exists, `load_config()` SHALL build a single `"default"` profile from top-level `bot_token`, `channel_id`, and `[targets]` fields.

#### Scenario: Old config format without profiles
- **WHEN** `config.toml` has top-level `bot_token` and `channel_id` but no `[profiles]` section
- **THEN** `load_config()` SHALL create a synthetic `"default"` profile using those values

#### Scenario: New config format with profiles
- **WHEN** `config.toml` has a `[profiles]` section
- **THEN** top-level `bot_token` and `channel_id` SHALL be ignored (profiles take precedence)

### Requirement: Profile field validation
The system SHALL validate that each profile has a non-empty `bot_token` that is not `"YOUR_BOT_TOKEN_HERE"`, and `channel_ids` is a non-empty array.

#### Scenario: Missing bot_token in profile
- **WHEN** a profile has no `bot_token` field
- **THEN** `load_config()` SHALL return an error indicating missing bot_token

#### Scenario: Placeholder bot_token
- **WHEN** a profile has `bot_token = "YOUR_BOT_TOKEN_HERE"`
- **THEN** `load_config()` SHALL return an error asking the user to set a real token

#### Scenario: Empty channel_ids
- **WHEN** a profile has `channel_ids = []`
- **THEN** `load_config()` SHALL return an error indicating channel_ids must not be empty
