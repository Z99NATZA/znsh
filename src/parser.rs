#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnclosedDoubleQuote,
    UnclosedSingleQuote,
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
                match in_quote_mode {
                    QuoteMode::None => in_quote_mode = QuoteMode::Double,
                    QuoteMode::Single => current.push(c),
                    QuoteMode::Double => in_quote_mode = QuoteMode::None,
                };

                token_started = true;
            }
            '\\' => {
                is_bslash = true;
            }
            _ => {
                if c.is_whitespace() && in_quote_mode == QuoteMode::None && !is_bslash {
                    if token_started {
                        tokens.push(std::mem::take(&mut current));
                        token_started = false;
                    }
                } else {
                    current.push(c);
                    token_started = true;
                }
            }
        };
    }

    match in_quote_mode {
        QuoteMode::None => {
            if token_started {
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
            tokenize(r#"echo "hello"#),
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
    fn slash_for_special_char() {
        assert_eq!(
            tokenize(r#"echo hello\ world"#),
            Ok(
                vec![
                    "echo".to_string(),
                    "hello world".to_string(),
                ]
            )
        );
    }
}
