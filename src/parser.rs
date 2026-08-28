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

const SINGLE_QUOTE: char = '\'';

fn push_to_word(character: char, current_word: &mut String, word_started: &mut bool) {
    current_word.push(character);
    *word_started = true;
}

pub fn tokenize(input: &str) -> Result<Vec<String>, ParseError> {
    let mut quote_mode = QuoteMode::None;
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let mut word_started = false;
    let mut escaping = false;

    for character in input.chars() {
        match character {
            SINGLE_QUOTE => {
                if escaping {
                    push_to_word(character, &mut current_word, &mut word_started);
                    escaping = false;
                    continue;
                }

                match quote_mode {
                    QuoteMode::None => {
                        quote_mode = QuoteMode::Single;
                    }
                    QuoteMode::Single => {
                        quote_mode = QuoteMode::None;
                    }
                    QuoteMode::Double => {
                        push_to_word(character, &mut current_word, &mut word_started)
                    }
                }

                word_started = true;
            }
            '"' => {
                if escaping {
                    push_to_word(character, &mut current_word, &mut word_started);
                    escaping = false;

                    continue;
                }

                match quote_mode {
                    QuoteMode::None => quote_mode = QuoteMode::Double,
                    QuoteMode::Single => {
                        push_to_word(character, &mut current_word, &mut word_started)
                    }
                    QuoteMode::Double => quote_mode = QuoteMode::None,
                };

                word_started = true;
            }
            '\\' => {
                if escaping {
                    push_to_word(character, &mut current_word, &mut word_started);
                    escaping = false;
                    continue;
                } else {
                    match quote_mode {
                        QuoteMode::None => escaping = true,
                        QuoteMode::Single => {
                            push_to_word(character, &mut current_word, &mut word_started)
                        }
                        QuoteMode::Double => escaping = true,
                    }
                }
            }
            _ => {
                if character.is_whitespace() && quote_mode == QuoteMode::None && !escaping {
                    if word_started {
                        tokens.push(std::mem::take(&mut current_word));
                        word_started = false;
                    }
                } else {
                    if character == '|' {
                        if escaping {
                            push_to_word(character, &mut current_word, &mut word_started);
                            escaping = false;
                            continue;
                        }
                        if quote_mode == QuoteMode::None && !word_started {
                            tokens.push('|'.to_string());
                            continue;
                        }
                        if quote_mode != QuoteMode::None {
                            push_to_word(character, &mut current_word, &mut word_started);
                            continue;
                        } else {
                            tokens.push(std::mem::take(&mut current_word));
                            tokens.push('|'.to_string());
                            word_started = false;
                            continue;
                        }
                    }

                    push_to_word(character, &mut current_word, &mut word_started);
                    escaping = false;
                }
            }
        };
    }

    match quote_mode {
        QuoteMode::None => {
            if escaping {
                return Err(ParseError::BackslashAtEnd);
            } else if word_started {
                tokens.push(current_word);
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

    #[test]
    fn leading_pipe_does_not_create_empty_token() {
        assert_eq!(
            tokenize("| echo"),
            Ok(vec!["|".to_string(), "echo".to_string(),])
        );
    }

    #[test]
    fn escaped_pipe_is_literal() {
        assert_eq!(
            tokenize(r#"echo hello\|world"#),
            Ok(vec!["echo".to_string(), "hello|world".to_string(),])
        );
    }

    #[test]
    fn pipe_escape_applies_to_only_one_character() {
        assert_eq!(
            tokenize(r#"echo hello\| world"#),
            Ok(vec![
                "echo".to_string(),
                "hello|".to_string(),
                "world".to_string(),
            ])
        );
    }

    #[test]
    fn escaped_backslash_is_literal() {
        assert_eq!(
            tokenize(r"echo a\\b"),
            Ok(vec!["echo".to_string(), r"a\b".to_string()])
        );
    }

    #[test]
    fn can_escape_single_quote() {
        assert_eq!(
            tokenize(r"echo \'hello\'"),
            Ok(vec!["echo".to_string(), "'hello'".to_string()])
        );
    }

    #[test]
    fn escaped_single_quote_can_be_a_token() {
        assert_eq!(
            tokenize(r"echo \'"),
            Ok(vec!["echo".to_string(), "'".to_string()])
        );
    }

    #[test]
    fn escaped_backslash_can_be_a_token() {
        assert_eq!(
            tokenize(r"echo \\"),
            Ok(vec!["echo".to_string(), "\\".to_string()])
        );
    }

    #[test]
    fn escaped_pipe_can_be_a_token() {
        assert_eq!(
            tokenize(r"echo \|"),
            Ok(vec!["echo".to_string(), "|".to_string()])
        );
    }
}
