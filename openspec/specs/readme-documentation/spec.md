## Requirements

### Requirement: Slash commands table documents /cli
Both `README.md` and `README-ZH.md` SHALL include `/cli` in the slash commands reference table with a description that mentions nine-cli invocation, quote-aware argument handling, and live streaming output updates.

#### Scenario: /cli entry present in English README
- **WHEN** a reader opens `README.md` and navigates to the slash commands section
- **THEN** the table SHALL contain a `/cli <args>` row describing the command

#### Scenario: /cli entry present in Chinese README
- **WHEN** a reader opens `README-ZH.md` and navigates to the slash commands section
- **THEN** the table SHALL contain a `/cli <args>` row with Chinese description

### Requirement: Directory tree includes libs/argv-parser and cli.rs
Both READMEs SHALL show `libs/argv-parser/mod.rs` and `src/slash_commands/cli.rs` in the project directory tree.

#### Scenario: libs/ folder visible in tree
- **WHEN** a reader looks at the directory tree in either README
- **THEN** `libs/argv-parser/` SHALL appear as a sibling of `src/`

#### Scenario: cli.rs visible in slash_commands/
- **WHEN** a reader looks at the slash_commands/ subtree
- **THEN** `cli.rs` SHALL appear alongside `echo.rs` and `mod.rs`

### Requirement: Module table reflects current architecture
Both READMEs SHALL document `slash_commands/mod.rs` as containing `ResponseHandle`, async `SlashCommand` trait, and `CommandDispatch` enum. A row for `slash_commands/cli.rs` SHALL be present. A row for `libs/argv-parser/mod.rs` (or `argv-parser`) SHALL be present.

#### Scenario: slash_commands/mod.rs description is current
- **WHEN** a developer reads the module table
- **THEN** the `slash_commands/mod.rs` row SHALL mention `ResponseHandle`, async trait, and `CommandDispatch`

#### Scenario: cli.rs has a module table entry
- **WHEN** a developer reads the module table
- **THEN** a row for `slash_commands/cli.rs` SHALL describe the `/cli` command implementation

#### Scenario: argv-parser has a module table entry
- **WHEN** a developer reads the module table
- **THEN** a row for `libs/argv-parser` SHALL describe the quote-aware tokenizer

### Requirement: Test count is current
Both READMEs SHALL show the test result line as `72 passed` (not `49 passed`).

#### Scenario: Test count in English README
- **WHEN** a reader looks at the testing section of `README.md`
- **THEN** the displayed count SHALL be `72 passed`

#### Scenario: Test count in Chinese README
- **WHEN** a reader looks at the testing section of `README-ZH.md`
- **THEN** the displayed count SHALL be `72 passed`
