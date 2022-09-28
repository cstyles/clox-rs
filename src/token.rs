#[derive(Debug, Copy, Clone)]
pub struct Token<'src> {
    pub token_type: TokenType,
    pub lexeme: &'src str,
    pub line: usize,
}

impl<'src> Token<'src> {
    pub fn identifiers_equal(&self, other: &Self) -> bool {
        self.lexeme == other.lexeme
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen = 0,
    RightParen = 1,
    LeftBrace = 2,
    RightBrace = 3,
    Comma = 4,
    Dot = 5,
    Minus = 6,
    Plus = 7,
    Semicolon = 8,
    Slash = 9,
    Star = 10,

    // One or two character tokens.
    Bang = 11,
    BangEqual = 12,
    Equal = 13,
    EqualEqual = 14,
    Greater = 15,
    GreaterEqual = 16,
    Less = 17,
    LessEqual = 18,

    // Literals.
    Identifier = 19,
    String = 20,
    Number = 21,

    // Keywords.
    And = 22,
    Class = 23,
    Else = 24,
    False = 25,
    Fun = 26,
    For = 27,
    If = 28,
    Nil = 29,
    Or = 30,
    Print = 31,
    Return = 32,
    Super = 33,
    This = 34,
    True = 35,
    Var = 36,
    While = 37,

    Error = 38,
    Eof = 39,
}
