## ADDED Requirements

### Requirement: /cli command registration
The system SHALL register a `/cli` slash command with Discord with a single required `String` option named `"args"`. The option description SHALL be `"Arguments to pass to nine-cli (e.g. skill-name arg1 \"quoted arg\")"`.

#### Scenario: Command appears in Discord UI
- **WHEN** the bot starts and registers slash commands
- **THEN** `/cli` SHALL appear in the Discord command picker with the `args` text input field

### Requirement: Argument forwarding to nine-cli
The system SHALL tokenize the `args` string from the Discord interaction using a quote-aware tokenizer, then spawn `nine-cli` with those tokens as argv.

#### Scenario: Simple space-separated args
- **WHEN** user invokes `/cli` with args `testskill foo bar`
- **THEN** the system SHALL spawn `nine-cli testskill foo bar` (3 argv tokens)

#### Scenario: Quoted argument containing spaces
- **WHEN** user invokes `/cli` with args `testskill "hello world" foo`
- **THEN** the system SHALL spawn `nine-cli` with argv `["testskill", "hello world", "foo"]` (3 tokens; spaces inside quotes preserved)

#### Scenario: Mixed quotes and bare args
- **WHEN** user invokes `/cli` with args `greet "aaa" bbb 333 "ddd"`
- **THEN** the system SHALL spawn `nine-cli` with argv `["greet", "aaa", "bbb", "333", "ddd"]` (5 tokens)

#### Scenario: Single-quoted argument
- **WHEN** user invokes `/cli` with args `skill 'hello world'`
- **THEN** the system SHALL spawn `nine-cli` with argv `["skill", "hello world"]` (2 tokens)

### Requirement: Deferred Discord response
The system SHALL defer the Discord interaction response within 3 seconds of receiving the interaction, showing a "thinking" indicator to the user while nine-cli executes.

#### Scenario: Immediate acknowledgement
- **WHEN** `/cli` interaction is received
- **THEN** the system SHALL call `defer_response()` before spawning nine-cli, so Discord shows a loading state

### Requirement: Streaming status updates
The system SHALL edit the deferred Discord message with accumulated stdout output at most once every 2 seconds while nine-cli is running.

#### Scenario: Periodic update during execution
- **WHEN** nine-cli is running and has produced output
- **THEN** the Discord message SHALL be edited with current accumulated output at most once per 2 seconds

#### Scenario: Update message format (in progress)
- **WHEN** nine-cli is still running
- **THEN** the Discord message SHALL display a header line `🔄  nine-cli <args>` followed by a divider and the accumulated output

### Requirement: Final response on completion
The system SHALL edit the Discord message with the complete final output when nine-cli exits successfully.

#### Scenario: Successful completion
- **WHEN** nine-cli exits with code 0
- **THEN** the Discord message SHALL be updated to show `✅  nine-cli <args>  (<elapsed>s)` followed by the full output (within 2000-char limit)

#### Scenario: Non-zero exit code
- **WHEN** nine-cli exits with a non-zero exit code
- **THEN** the Discord message SHALL display `❌  nine-cli <args> exited with code <N>` followed by any captured output

### Requirement: Output truncation (rolling window)
The system SHALL display only the most recent output when accumulated output exceeds 1800 characters, prefixing the display with `[...earlier output truncated...]`.

#### Scenario: Output within limit
- **WHEN** total output is ≤ 1800 characters
- **THEN** the full output SHALL be displayed without truncation

#### Scenario: Output exceeds limit
- **WHEN** total accumulated output exceeds 1800 characters
- **THEN** the Discord message SHALL show `[...earlier output truncated...]\n` followed by the last 1600 characters of output

### Requirement: Execution timeout
The system SHALL terminate nine-cli and report a timeout error if execution exceeds 10 minutes.

#### Scenario: Skill completes within timeout
- **WHEN** nine-cli completes within 10 minutes
- **THEN** the final output SHALL be displayed normally

#### Scenario: Skill exceeds timeout
- **WHEN** nine-cli runs for more than 10 minutes
- **THEN** the system SHALL kill the process and edit the Discord message with `⏱️  nine-cli <args> timed out after 10 minutes`

### Requirement: nine-cli not found error
The system SHALL surface a clear error to Discord if `nine-cli` is not found in PATH.

#### Scenario: nine-cli missing
- **WHEN** nine-cli is not available in PATH at invocation time
- **THEN** the Discord message SHALL display `❌  nine-cli not found in PATH`
