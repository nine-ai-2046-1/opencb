### Requirement: Quote-aware argument tokenization
The system SHALL provide a `tokenize_argv(input: &str) -> Vec<String>` function in `libs/argv-parser/mod.rs` that splits input on ASCII whitespace while treating double-quoted and single-quoted groups as single tokens. Quote characters SHALL be stripped from the output tokens.

#### Scenario: Empty input
- **WHEN** input is `""`
- **THEN** the function SHALL return an empty `Vec`

#### Scenario: Single bare token
- **WHEN** input is `"hello"`
- **THEN** the function SHALL return `["hello"]`

#### Scenario: Multiple bare tokens
- **WHEN** input is `"foo bar baz"`
- **THEN** the function SHALL return `["foo", "bar", "baz"]`

#### Scenario: Double-quoted token with spaces
- **WHEN** input is `"\"hello world\""`
- **THEN** the function SHALL return `["hello world"]` (one token, quotes stripped)

#### Scenario: Single-quoted token with spaces
- **WHEN** input is `"'hello world'"`
- **THEN** the function SHALL return `["hello world"]` (one token, quotes stripped)

#### Scenario: Mixed quoted and bare tokens
- **WHEN** input is `"greet \"aaa\" bbb 333 \"ddd\""`
- **THEN** the function SHALL return `["greet", "aaa", "bbb", "333", "ddd"]`

#### Scenario: Leading and trailing whitespace
- **WHEN** input is `"  foo  bar  "`
- **THEN** the function SHALL return `["foo", "bar"]` (no empty tokens)

#### Scenario: Adjacent quoted tokens
- **WHEN** input is `"\"foo\"\"bar\""`
- **THEN** the function SHALL return `["foo", "bar"]` (two separate tokens)

### Requirement: Unclosed quote handling
The system SHALL treat an unclosed quote as extending to the end of the input, including all remaining characters in the current token.

#### Scenario: Unclosed double quote
- **WHEN** input is `"foo \"bar baz"`
- **THEN** the function SHALL return `["foo", "bar baz"]` (unclosed quote captured to end)
