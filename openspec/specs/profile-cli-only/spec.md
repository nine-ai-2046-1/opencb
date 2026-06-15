## Requirements

### Requirement: Profile has cli_only field
Each profile in config.toml SHALL support an optional `cli_only` boolean field. When not specified, it SHALL default to `true`.

#### Scenario: cli_only defaults to true
- **WHEN** a profile does not specify `cli_only` in config.toml
- **THEN** the profile SHALL behave as if `cli_only = true`

#### Scenario: cli_only explicitly set to true
- **WHEN** a profile has `cli_only = true` in config.toml
- **THEN** the bot SHALL reject messages not starting with `/`

#### Scenario: cli_only explicitly set to false
- **WHEN** a profile has `cli_only = false` in config.toml
- **THEN** the bot SHALL NOT reject messages based on `/` prefix

### Requirement: Rejection behavior controlled by cli_only
When `cli_only = true`, the bot SHALL reject (ignore) messages that do not start with `/`. When `cli_only = false`, the bot SHALL process all messages regardless of prefix.

#### Scenario: cli_only true rejects non-slash message
- **WHEN** profile has `cli_only = true` and a message arrives with content "hello world" (no `/` prefix)
- **THEN** the message SHALL be ignored and not processed

#### Scenario: cli_only true accepts slash command
- **WHEN** profile has `cli_only = true` and a message arrives with content "/cli search something"
- **THEN** the message SHALL be processed as a command

#### Scenario: cli_only false accepts non-slash message
- **WHEN** profile has `cli_only = false` and a message arrives with content "hello world" (no `/` prefix)
- **THEN** the message SHALL be processed and passed to the target CLI

#### Scenario: cli_only false accepts slash command
- **WHEN** profile has `cli_only = false` and a message arrives with content "/cli search something"
- **THEN** the message SHALL be processed as a command

### Requirement: Backward compatible config loading
Existing config.toml files without `cli_only` SHALL continue to work without modification. The absence of `cli_only` SHALL be treated as `cli_only = true`.

#### Scenario: Old config without cli_only
- **WHEN** a config.toml has a profile without `cli_only` field
- **THEN** the profile SHALL load successfully with `cli_only` defaulting to `true`

#### Scenario: New config with cli_only
- **WHEN** a config.toml has a profile with `cli_only = false`
- **THEN** the profile SHALL load successfully with `cli_only` set to `false`
