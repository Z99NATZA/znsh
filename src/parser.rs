pub fn tokenize(input: &str) -> Vec<String> {
    input.split_whitespace()
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::tokenize;

    #[test]
    fn splits_words_on_whitespace() {
        assert_eq!(
            tokenize("echo hello world"),
            vec![
                String::from("echo"),
                String::from("hello"),
                String::from("world"),
            ]
        );
    }
}