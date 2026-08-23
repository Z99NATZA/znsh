#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnclosedDoubleQuote,
    UnclosedSingleQuote,
    BackslashAtEnd,
}

#[derive(Debug, PartialEq)]
enum QuoteMode {
    None,
    Single,
    Double,
}

pub fn tokenize(input: &str) -> Result<Vec<String>, ParseError> {
    let mut in_quote_mode = QuoteMode::None;
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut token_started = false;
    let mut is_bslash = false;
    let mut space_started = false;

    for c in input.chars() {
        match c {
            '\'' => {
                match in_quote_mode {
                    QuoteMode::None => in_quote_mode = QuoteMode::Single,
                    QuoteMode::Single => in_quote_mode = QuoteMode::None,
                    QuoteMode::Double => current.push(c),
                }

                token_started = true;
            }
            '"' => {
                if is_bslash {
                    current.push(c);
                    is_bslash = false;
                    token_started = true;

                    continue;
                }

                match in_quote_mode {
                    QuoteMode::None => in_quote_mode = QuoteMode::Double,
                    QuoteMode::Single => current.push(c),
                    QuoteMode::Double => in_quote_mode = QuoteMode::None,
                };

                token_started = true;
            }
            '\\' => match in_quote_mode {
                QuoteMode::None => is_bslash = true,
                QuoteMode::Single => current.push(c),
                QuoteMode::Double => is_bslash = true,
            },
            _ => {
                if c.is_whitespace() && in_quote_mode == QuoteMode::None && !is_bslash {
                    space_started = true;

                    if token_started {
                        tokens.push(std::mem::take(&mut current));
                        token_started = false;
                    }
                } else {
                    if c == '|' {
                        if space_started && in_quote_mode == QuoteMode::None && !token_started {
                            tokens.push('|'.to_string());
                            space_started = false;
                            continue;
                        }
                        if in_quote_mode != QuoteMode::None {
                            current.push(c);
                            continue;
                        } else {
                            tokens.push(std::mem::take(&mut current));
                            tokens.push('|'.to_string());
                            token_started = false;
                            continue;
                        }
                    }

                    current.push(c);
                    token_started = true;
                    is_bslash = false;
                }
            }
        };
    }

    match in_quote_mode {
        QuoteMode::None => {
            if is_bslash {
                return Err(ParseError::BackslashAtEnd);
            } else if token_started {
                tokens.push(current);
            }

            Ok(tokens)
        }
        QuoteMode::Single => Err(ParseError::UnclosedSingleQuote),
        QuoteMode::Double => Err(ParseError::UnclosedDoubleQuote),
    }
}

#[cfg(test)]
mod tests {
    use super::ParseError;
    use super::tokenize;

    #[test]
    fn splits_words_on_whitespace() {
        assert_eq!(
            tokenize("echo hello world"),
            Ok(vec![
                String::from("echo"),
                String::from("hello"),
                String::from("world"),
            ])
        );
    }

    #[test]
    fn keeps_double_quoted_words_together() {
        assert_eq!(
            tokenize(r#"echo "hello world""#),
            Ok(vec![String::from("echo"), String::from("hello world")])
        )
    }

    #[test]
    fn rejects_unclosed_double_quote() {
        assert_eq!(
            tokenize(r#"echo "hllo"#),
            Err(ParseError::UnclosedDoubleQuote)
        )
    }

    #[test]
    fn keeps_single_quote_words_together() {
        assert_eq!(
            tokenize("echo 'hello world' na"),
            Ok(vec![
                "echo".to_string(),
                "hello world".to_string(),
                "na".to_string(),
            ])
        );
    }

    #[test]
    fn rejects_unclosed_single_quote() {
        assert_eq!(
            tokenize("echo 'hello"),
            Err(ParseError::UnclosedSingleQuote),
        );
    }

    #[test]
    fn empty_arguments() {
        assert_eq!(
            tokenize(r#"echo "" '' "#),
            Ok(vec!["echo".to_string(), "".to_string(), "".to_string(),])
        );
    }

    #[test]
    fn double_quote_inside_single_quote() {
        assert_eq!(
            tokenize(r#"echo 'a "b" c'"#),
            Ok(vec!["echo".to_string(), r#"a "b" c"#.to_string(),])
        )
    }

    #[test]
    fn single_quote_inside_double_quote() {
        assert_eq!(
            tokenize(r#"echo "a 'b' c""#),
            Ok(vec!["echo".to_string(), r#"a 'b' c"#.to_string(),])
        )
    }

    #[test]
    fn word_concatenation() {
        assert_eq!(
            tokenize(r#"echo hello" worl"d"#),
            Ok(vec!["echo".to_string(), "hello world".to_string(),])
        )
    }

    #[test]
    fn back_slash_for_special_char() {
        assert_eq!(
            tokenize(r#"echo hello\ world"#),
            Ok(vec!["echo".to_string(), "hello world".to_string(),])
        );
    }

    #[test]
    fn escape_applies_to_only_one_character() {
        assert_eq!(
            tokenize(r#"echo hello\ world again"#),
            Ok(vec![
                "echo".to_string(),
                "hello world".to_string(),
                "again".to_string(),
            ])
        );
    }

    #[test]
    fn can_escape_quote() {
        assert_eq!(
            tokenize(r#"echo \"hello\""#),
            Ok(vec!["echo".to_string(), r#""hello""#.to_string()])
        );
    }

    #[test]
    fn bslash_is_literal_inside_single_quotes() {
        assert_eq!(
            tokenize(r#"echo 'a\b'"#),
            Ok(vec!["echo".to_string(), r#"a\b"#.to_string()])
        );
    }

    #[test]
    fn bslash_at_end() {
        assert_eq!(tokenize("echo hello\\"), Err(ParseError::BackslashAtEnd));
    }

    #[test]
    fn separates_pipe_from_words() {
        assert_eq!(
            tokenize("echo hello|wc"),
            Ok(vec![
                "echo".to_string(),
                "hello".to_string(),
                "|".to_string(),
                "wc".to_string(),
            ])
        );
    }

    #[test]
    fn pipe_inside_double_quotes_is_literal() {
        assert_eq!(
            tokenize(r#"echo "hello|world""#),
            Ok(vec!["echo".to_string(), "hello|world".to_string(),])
        );
    }

    #[test]
    fn separates_pipe_surrounded_by_spaces() {
        assert_eq!(
            tokenize("echo hello | wc"),
            Ok(vec![
                "echo".to_string(),
                "hello".to_string(),
                "|".to_string(),
                "wc".to_string(),
            ])
        );
    }
}
