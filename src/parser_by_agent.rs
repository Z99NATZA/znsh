//! A reference lexer for study.
//!
//! This file is intentionally not included from `lib.rs`, so it does not
//! replace the implementation in `parser.rs` or participate in its tests.

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnclosedDoubleQuote,
    UnclosedSingleQuote,
    BackslashAtEnd,
}

/// Operators and words are different token kinds.
///
/// Keeping this distinction here prevents a later parser from confusing the
/// operator in `echo | wc` with the word in `echo "|"`.
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Pipe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteMode {
    None,
    Single,
    Double,
}

struct Lexer {
    mode: QuoteMode,
    tokens: Vec<Token>,
    current: String,
    word_started: bool,
    escaping: bool,
}

impl Lexer {
    fn new() -> Self {
        Self {
            mode: QuoteMode::None,
            tokens: Vec::new(),
            current: String::new(),
            word_started: false,
            escaping: false,
        }
    }

    fn scan(mut self, input: &str) -> Result<Vec<Token>, ParseError> {
        for character in input.chars() {
            self.consume(character);
        }

        self.finish()
    }

    fn consume(&mut self, character: char) {
        match character {
            '\'' => self.consume_single_quote(),
            '"' => self.consume_double_quote(),
            '\\' => self.consume_backslash(),
            '|' => self.consume_pipe(),
            character if character.is_whitespace() => self.consume_whitespace(character),
            character => self.push_character(character),
        }
    }

    fn consume_single_quote(&mut self) {
        // Deliberately mirrors the unfinished behavior in parser.rs.
        // Handling a single quote while `escaping` is true remains the next
        // exercise; this reference does not solve that test.
        match self.mode {
            QuoteMode::None => self.mode = QuoteMode::Single,
            QuoteMode::Single => self.mode = QuoteMode::None,
            QuoteMode::Double => self.current.push('\''),
        }

        self.word_started = true;
    }

    fn consume_double_quote(&mut self) {
        if self.escaping {
            self.push_escaped('"');
            return;
        }

        match self.mode {
            QuoteMode::None => self.mode = QuoteMode::Double,
            QuoteMode::Single => self.current.push('"'),
            QuoteMode::Double => self.mode = QuoteMode::None,
        }

        self.word_started = true;
    }

    fn consume_backslash(&mut self) {
        if self.escaping {
            self.push_escaped('\\');
            return;
        }

        match self.mode {
            QuoteMode::Single => {
                self.current.push('\\');
                self.word_started = true;
            }
            QuoteMode::None | QuoteMode::Double => self.escaping = true,
        }
    }

    fn consume_pipe(&mut self) {
        if self.escaping {
            self.push_escaped('|');
        } else if self.mode == QuoteMode::None {
            self.finish_word();
            self.tokens.push(Token::Pipe);
        } else {
            self.push_character('|');
        }
    }

    fn consume_whitespace(&mut self, character: char) {
        if self.mode == QuoteMode::None && !self.escaping {
            self.finish_word();
        } else {
            self.push_character(character);
        }
    }

    fn push_character(&mut self, character: char) {
        self.current.push(character);
        self.word_started = true;
        self.escaping = false;
    }

    fn push_escaped(&mut self, character: char) {
        self.current.push(character);
        self.word_started = true;
        self.escaping = false;
    }

    fn finish_word(&mut self) {
        if !self.word_started {
            return;
        }

        self.tokens
            .push(Token::Word(std::mem::take(&mut self.current)));
        self.word_started = false;
    }

    fn finish(mut self) -> Result<Vec<Token>, ParseError> {
        match self.mode {
            QuoteMode::Single => return Err(ParseError::UnclosedSingleQuote),
            QuoteMode::Double => return Err(ParseError::UnclosedDoubleQuote),
            QuoteMode::None => {}
        }

        if self.escaping {
            return Err(ParseError::BackslashAtEnd);
        }

        self.finish_word();
        Ok(self.tokens)
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    Lexer::new().scan(input)
}
