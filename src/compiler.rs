use num_enum::IntoPrimitive;

use crate::chunk::{Chunk, OpCode};
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::value::Value;

#[derive(Debug)]
pub struct Compiler<'src> {
    scanner: Scanner<'src>,
    current: Option<Token<'src>>,
    previous: Option<Token<'src>>,
    had_error: bool,
    panic_mode: bool,
    compiling_chunk: Chunk,
}

impl<'src> Compiler<'src> {
    pub fn new(scanner: Scanner<'src>, chunk: Chunk) -> Self {
        Self {
            scanner,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            compiling_chunk: chunk,
        }
    }

    #[must_use]
    pub fn compile(mut self) -> Result<Chunk, ()> {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");

        self.end_compiler();

        if self.had_error {
            Err(())
        } else {
            Ok(self.compiling_chunk)
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.take();

        loop {
            let token = self.scanner.scan_token();
            self.current = Some(token);
            match token.token_type {
                TokenType::Error => self.error_at_current(token.lexeme),
                _ => break,
            }
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.current.unwrap(), message);
        self.had_error = true;
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.previous.unwrap(), message);
        self.had_error = true;
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        match token.token_type {
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error => {}
            _ => eprint!(" at {}", token.lexeme),
        }

        eprintln!(": {message}");
        self.had_error = true;
    }

    fn consume(&mut self, expected: TokenType, message: &str) {
        if self
            .current
            .map_or(false, |token| token.token_type == expected)
        {
            self.advance();
        } else {
            self.error_at_current(message)
        }
    }

    fn current_chunk(&self) -> &Chunk {
        &self.compiling_chunk
    }

    fn current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous.unwrap().line;
        self.current_chunk_mut().write_byte(byte, line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        let line = self.previous.unwrap().line;
        self.current_chunk_mut().write_opcode(opcode, line);
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        #[cfg(debug_assertions)]
        if !self.had_error {
            self.current_chunk().disassemble("code");
            println!();
        }
    }

    fn emit_return(&mut self) {
        self.emit_opcode(OpCode::Return);
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_opcode(OpCode::Constant);

        let constant = self.make_constant(value);
        self.emit_byte(constant);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self
            .get_parse_rule(self.previous.unwrap().token_type)
            .prefix;

        match prefix_rule {
            Some(prefix_func) => prefix_func(self),
            None => {
                self.error("Expect expression.");
                return;
            }
        }

        while (precedence as u8)
            <= self
                .get_parse_rule(self.current.unwrap().token_type)
                .precedence as u8
        {
            self.advance();
            let infix_rule = self.get_parse_rule(self.previous.unwrap().token_type).infix;
            if let Some(f) = infix_rule {
                f(self);
            }
        }
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk_mut().add_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    fn get_parse_rule(&self, token_type: TokenType) -> ParseRule {
        match token_type {
            TokenType::LeftParen => ParseRule {
                prefix: Some(grouping),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::RightParen => ParseRule::default(),
            TokenType::LeftBrace => ParseRule::default(),
            TokenType::RightBrace => ParseRule::default(),
            TokenType::Comma => ParseRule::default(),
            TokenType::Dot => ParseRule::default(),
            TokenType::Minus => ParseRule {
                prefix: Some(unary),
                infix: Some(binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(binary),
                precedence: Precedence::Term,
            },
            TokenType::Semicolon => ParseRule::default(),
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(binary),
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule::default(),
            TokenType::BangEqual => ParseRule::default(),
            TokenType::Equal => ParseRule::default(),
            TokenType::EqualEqual => ParseRule::default(),
            TokenType::Greater => ParseRule::default(),
            TokenType::GreaterEqual => ParseRule::default(),
            TokenType::Less => ParseRule::default(),
            TokenType::LessEqual => ParseRule::default(),
            TokenType::Identifier => ParseRule::default(),
            TokenType::String => ParseRule::default(),
            TokenType::Number => ParseRule {
                prefix: Some(number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::And => ParseRule::default(),
            TokenType::Class => ParseRule::default(),
            TokenType::Else => ParseRule::default(),
            TokenType::False => ParseRule::default(),
            TokenType::Fun => ParseRule::default(),
            TokenType::For => ParseRule::default(),
            TokenType::If => ParseRule::default(),
            TokenType::Nil => ParseRule::default(),
            TokenType::Or => ParseRule::default(),
            TokenType::Print => ParseRule::default(),
            TokenType::Return => ParseRule::default(),
            TokenType::Super => ParseRule::default(),
            TokenType::This => ParseRule::default(),
            TokenType::True => ParseRule::default(),
            TokenType::Var => ParseRule::default(),
            TokenType::While => ParseRule::default(),
            TokenType::Error => ParseRule::default(),
            TokenType::Eof => ParseRule::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, IntoPrimitive)]
#[repr(u8)]
enum Precedence {
    None = 0,
    Assignment = 1, // =
    Or = 2,         // or
    And = 3,        // and
    Equality = 4,   // == !=
    Comparison = 5, // < > <= >=
    Term = 6,       // + -
    Factor = 7,     // * /
    Unary = 8,      // ! -
    Call = 9,       // . ()
    Primary = 10,
}

impl Precedence {
    fn higher(&self) -> Self {
        use Precedence::*;

        match self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => unreachable!(),
        }
    }
}

struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl Default for ParseRule {
    fn default() -> Self {
        Self {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }
    }
}

type ParseFn = fn(compiler: &mut Compiler<'_>) -> ();

fn grouping(compiler: &mut Compiler) {
    compiler.expression();
    compiler.consume(TokenType::RightParen, "Expect ')' after expression.");
}

fn binary(compiler: &mut Compiler) {
    let operator_type = compiler.previous.unwrap().token_type;
    let parse_rule = compiler.get_parse_rule(operator_type);
    let precedence = parse_rule.precedence.higher();
    compiler.parse_precedence(precedence);

    match operator_type {
        TokenType::Plus => compiler.emit_opcode(OpCode::Add),
        TokenType::Minus => compiler.emit_opcode(OpCode::Subtract),
        TokenType::Star => compiler.emit_opcode(OpCode::Multiply),
        TokenType::Slash => compiler.emit_opcode(OpCode::Divide),
        _ => unreachable!(),
    }
}

fn unary(compiler: &mut Compiler) {
    let operator_type = compiler.previous.unwrap().token_type;

    // Compile the expression
    compiler.parse_precedence(Precedence::Unary);

    match operator_type {
        TokenType::Minus => compiler.emit_opcode(OpCode::Negate),
        // TokenType::Bang => compiler.emit_opcode(OpCode::?),
        _ => {}
    }
}

fn number(compiler: &mut Compiler) {
    let value: f64 = compiler.previous.unwrap().lexeme.parse().unwrap();
    compiler.emit_constant(value);
}
