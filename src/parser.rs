#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnclosedDoubleQuote,
}

pub fn tokenize(input: &str) -> Result<Vec<String>, ParseError> {
    let mut in_double_quotes = false;
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut token_started = false;

    for c in input.chars() {
        if c == '"' {
            in_double_quotes = !in_double_quotes;
            token_started = true;
        }
        // else if: space and not in doubel qoutes
        else if c.is_whitespace() && !in_double_quotes {
            if token_started {
                tokens.push(std::mem::take(&mut current));
                token_started = false;
            }
        } else {
            current.push(c);
            token_started = true;
        }
    }

    if in_double_quotes {
        return Err(ParseError::UnclosedDoubleQuote);
    }

    if token_started {
        tokens.push(current);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
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
            Ok(vec![String::from("echo"), String::from("hello world"),])
        )
    }
}
