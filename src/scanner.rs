#[derive(Debug)]
pub struct Scanner<'src> {
    start: &'src str,
    current: &'src str,
    line: usize,
}

impl<'src> Scanner<'src> {
    // TODO: use &'src [char]
    pub fn new(source: &'src str) -> Self {
        Self {
            start: source,
            current: source,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'_> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        match self.advance() {
            '(' => return self.make_token(TokenType::LeftParen),
            ')' => return self.make_token(TokenType::RightParen),
            '{' => return self.make_token(TokenType::LeftBrace),
            '}' => return self.make_token(TokenType::RightBrace),
            ';' => return self.make_token(TokenType::Semicolon),
            ',' => return self.make_token(TokenType::Comma),
            '.' => return self.make_token(TokenType::Dot),
            '-' => return self.make_token(TokenType::Minus),
            '+' => return self.make_token(TokenType::Plus),
            '/' => return self.make_token(TokenType::Slash),
            '*' => return self.make_token(TokenType::Star),
            '!' => {
                return if self.match_('=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                };
            }
            '=' => {
                return if self.match_('=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                };
            }
            '<' => {
                return if self.match_('=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                };
            }
            '>' => {
                return if self.match_('=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                };
            }
            '"' => return self.string(),
            '0'..='9' => return self.number(),
            c if is_alpha(c) => return self.identifier(),
            _ => {}
        };

        self.error_token("Unexpected character.")
    }

    #[must_use]
    fn is_at_end(&self) -> bool {
        self.current.is_empty()
    }

    #[must_use]
    fn make_token(&self, token_type: TokenType) -> Token<'_> {
        Token {
            token_type,
            start: slice_to(self.start, self.current),
            line: self.line,
        }
    }

    #[must_use]
    fn error_token<'msg>(&self, message: &'msg str) -> Token<'msg> {
        Token {
            token_type: TokenType::Error,
            start: message,
            line: self.line,
        }
    }

    #[must_use]
    fn advance(&mut self) -> char {
        let c = self.current.chars().next().unwrap();
        self.bump_current(c);
        c
    }

    fn bump_current(&mut self, c: char) {
        self.bump_current_by(c.len_utf8());
    }

    fn bump_current_by(&mut self, offset: usize) {
        self.current = &self.current[offset..];
    }

    #[must_use]
    fn match_(&mut self, expected: char) -> bool {
        let c = match self.current.chars().next() {
            Some(c) => c,
            None => return false, // at end
        };

        if c == expected {
            self.bump_current(c);
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.current.chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        self.current.chars().nth(1)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\r' | '\t' => {
                    self.bump_current_by(1);
                }
                '\n' => {
                    self.line += 1;
                    self.bump_current_by(1);
                }
                '/' => {
                    // Comments
                    if let Some('/') = self.peek_next() {
                        self.bump_current_by(2); // skip both slashes

                        loop {
                            match self.peek() {
                                None => break,
                                Some('\n') => break,
                                Some(c) => self.bump_current(c),
                            }
                        }
                    } else {
                        return; // Not a comment; just a slash
                    }
                }
                _ => return, // Not whitespace
            };
        }
    }

    fn string(&mut self) -> Token {
        loop {
            match self.peek() {
                None => return self.error_token("Unterminated string."),
                Some('"') => break,
                Some('\n') => {
                    self.line += 1;
                    self.bump_current_by(1);
                }
                Some(c) => {
                    self.bump_current(c);
                }
            }
        }

        // The closing quote.
        self.bump_current_by(1);

        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Token {
        self.consume_digits();

        // Look for a fractional part.
        if let Some('.') = self.peek() {
            if let Some('0'..='9') = self.peek_next() {
                self.bump_current_by(1);
                self.consume_digits();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn consume_digits(&mut self) {
        loop {
            match self.peek() {
                None => break,
                Some(c) if !c.is_ascii_digit() => break,
                Some(c) => self.bump_current(c),
            }
        }
    }

    fn identifier(&mut self) -> Token {
        loop {
            match self.peek() {
                None => break,
                Some(c) if is_alpha(c) || c.is_ascii_digit() => self.bump_current_by(1),
                _ => break,
            }
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match slice_to(self.start, self.current) {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        }
    }
}

const fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

#[derive(Debug)]
pub struct Token<'src> {
    pub token_type: TokenType,
    pub start: &'src str,
    pub line: usize,
}

#[derive(Debug, PartialEq, Eq)]
#[allow(unused)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}

// fn ref_diff(start: &[char], current: &[char]) -> usize {
fn ref_diff(start: &str, current: &str) -> usize {
    let start = start.as_ptr();
    let current = current.as_ptr();

    // SAFETY: `current` and `start` are derived from the same source.
    // Pretty sure that as long as the original references are valid, this is too.
    let diff = unsafe { current.offset_from(start) };

    // `current` is always >= start so diff will never be negative
    diff as usize
}

fn slice_to<'a>(start: &'a str, end: &str) -> &'a str {
    &start[..ref_diff(start, end)]
}
