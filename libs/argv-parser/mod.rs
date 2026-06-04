//! argv-parser: quote-aware command-line argument tokenizer.
//!
//! Splits an input string into tokens using shell-like rules:
//! - Splits on ASCII whitespace
//! - Double-quoted groups are treated as a single token (quotes stripped)
//! - Single-quoted groups are treated as a single token (quotes stripped)
//! - Unclosed quotes consume the rest of the input
//! - No shell expansion, no escape sequences

/// Tokenize an input string into argv-style tokens.
///
/// Quotes are stripped from the output; their contents are preserved as single tokens.
/// Leading/trailing whitespace is ignored; consecutive spaces produce no empty tokens.
///
/// # Examples
/// ```
/// let tokens = tokenize_argv("testskill \"hello world\" foo");
/// assert_eq!(tokens, vec!["testskill", "hello world", "foo"]);
/// ```
pub fn tokenize_argv(input: &str) -> Vec<String> {
    // State machine states for the tokenizer
    enum State {
        /// Between tokens — whitespace is skipped
        Normal,
        /// Inside a double-quoted section
        InDoubleQuote,
        /// Inside a single-quoted section
        InSingleQuote,
    }

    let mut tokens: Vec<String> = Vec::new();
    // Current token being built
    let mut current = String::new();
    let mut state = State::Normal;

    for ch in input.chars() {
        match state {
            State::Normal => {
                if ch == '"' {
                    // Start of a double-quoted group; switch state without adding quote char
                    state = State::InDoubleQuote;
                } else if ch == '\'' {
                    // Start of a single-quoted group
                    state = State::InSingleQuote;
                } else if ch.is_ascii_whitespace() {
                    // Whitespace ends the current token (if any)
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                } else {
                    // Regular character — append to current token
                    current.push(ch);
                }
            }
            State::InDoubleQuote => {
                if ch == '"' {
                    // End of double-quoted group; flush token and return to Normal
                    // (adjacent quoted groups merge into one token if no space between them)
                    tokens.push(current.clone());
                    current.clear();
                    state = State::Normal;
                } else {
                    // All characters inside quotes (including spaces) are literal
                    current.push(ch);
                }
            }
            State::InSingleQuote => {
                if ch == '\'' {
                    // End of single-quoted group
                    tokens.push(current.clone());
                    current.clear();
                    state = State::Normal;
                } else {
                    current.push(ch);
                }
            }
        }
    }

    // Handle any remaining content: unclosed quote or trailing bare token
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        // Empty string produces no tokens
        assert_eq!(tokenize_argv(""), Vec::<String>::new());
    }

    #[test]
    fn test_single_bare_token() {
        assert_eq!(tokenize_argv("hello"), vec!["hello"]);
    }

    #[test]
    fn test_multiple_bare_tokens() {
        assert_eq!(tokenize_argv("foo bar baz"), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_double_quoted_token_with_spaces() {
        // Quotes stripped; space inside preserved as single token
        assert_eq!(tokenize_argv("\"hello world\""), vec!["hello world"]);
    }

    #[test]
    fn test_single_quoted_token_with_spaces() {
        assert_eq!(tokenize_argv("'hello world'"), vec!["hello world"]);
    }

    #[test]
    fn test_mixed_quoted_and_bare() {
        // Reproduces the spec example: greet "aaa" bbb 333 "ddd"
        assert_eq!(
            tokenize_argv("greet \"aaa\" bbb 333 \"ddd\""),
            vec!["greet", "aaa", "bbb", "333", "ddd"]
        );
    }

    #[test]
    fn test_leading_trailing_whitespace() {
        assert_eq!(tokenize_argv("  foo  bar  "), vec!["foo", "bar"]);
    }

    #[test]
    fn test_adjacent_double_quoted_tokens() {
        // Two quoted groups with no space between them become two separate tokens
        assert_eq!(tokenize_argv("\"foo\"\"bar\""), vec!["foo", "bar"]);
    }

    #[test]
    fn test_unclosed_double_quote() {
        // Unclosed quote captures everything to end of input
        assert_eq!(tokenize_argv("foo \"bar baz"), vec!["foo", "bar baz"]);
    }

    #[test]
    fn test_unclosed_single_quote() {
        assert_eq!(tokenize_argv("foo 'bar baz"), vec!["foo", "bar baz"]);
    }

    #[test]
    fn test_whitespace_only() {
        // Only whitespace produces no tokens
        assert_eq!(tokenize_argv("   "), Vec::<String>::new());
    }

    #[test]
    fn test_quoted_with_surrounding_bare() {
        // Matches the /cli use case: testskill "hello world" foo
        assert_eq!(
            tokenize_argv("testskill \"hello world\" foo"),
            vec!["testskill", "hello world", "foo"]
        );
    }

    #[test]
    fn test_single_quoted_with_surrounding_bare() {
        assert_eq!(
            tokenize_argv("skill 'hello world'"),
            vec!["skill", "hello world"]
        );
    }
}
