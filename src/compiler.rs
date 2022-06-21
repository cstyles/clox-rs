use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::chunk::{Chunk, OpCode};
use crate::object::Object;
use crate::scanner::Scanner;
use crate::string::LoxString;
use crate::token::{Token, TokenType};
use crate::value::Value;
use crate::vm::Vm;

#[derive(Debug)]
pub struct Compiler<'src, 'vm> {
    vm: &'vm mut Vm,
    scanner: Scanner<'src>,
    current: Option<Token<'src>>,
    previous: Option<Token<'src>>,
    had_error: bool,
    panic_mode: bool,
    compiling_chunk: Chunk,
}

impl<'src, 'vm> Compiler<'src, 'vm> {
    fn new(vm: &'vm mut Vm, source: &'src str) -> Self {
        let scanner = Scanner::new(source);
        let chunk = Chunk::new();

        Self {
            vm,
            scanner,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            compiling_chunk: chunk,
        }
    }

    pub fn compile(vm: &'vm mut Vm, source: &'src str) -> Result<Chunk, ()> {
        let mut compiler = Self::new(vm, source);

        compiler.advance();
        while !compiler.match_(TokenType::Eof) {
            compiler.declaration();
        }

        compiler.end_compiler();

        if compiler.had_error {
            Err(())
        } else {
            Ok(compiler.compiling_chunk)
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

    fn check(&mut self, token_type: TokenType) -> bool {
        self.current
            .map_or(false, |token| token.token_type == token_type)
    }

    fn check_previous(&mut self, token_type: TokenType) -> bool {
        self.previous
            .map_or(false, |token| token.token_type == token_type)
    }

    fn match_(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
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

    fn emit_opcodes(&mut self, opcode1: OpCode, opcode2: OpCode) {
        self.emit_opcode(opcode1);
        self.emit_opcode(opcode2);
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

    fn declaration(&mut self) {
        self.statement();

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_opcode(OpCode::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_opcode(OpCode::Pop);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while !self.check(TokenType::Eof) {
            if self.check_previous(TokenType::Semicolon) {
                return;
            }

            use TokenType::*;
            if let Some(Class | Fun | Var | For | If | While | Print | Return) =
                self.current.map(|token| token.token_type)
            {
                return;
            }
        }

        self.advance();
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
        RULE_TABLE[token_type as usize]
    }
}

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
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
        let as_u8: u8 = (*self).into();
        Precedence::try_from(as_u8 + 1).unwrap()
    }
}

#[derive(Copy, Clone)]
struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

type ParseFn = fn(compiler: &mut Compiler<'_, '_>) -> ();

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
        TokenType::BangEqual => compiler.emit_opcodes(OpCode::Equal, OpCode::Not),
        TokenType::EqualEqual => compiler.emit_opcode(OpCode::Equal),
        TokenType::Greater => compiler.emit_opcode(OpCode::Greater),
        TokenType::GreaterEqual => compiler.emit_opcodes(OpCode::Less, OpCode::Not),
        TokenType::Less => compiler.emit_opcode(OpCode::Less),
        TokenType::LessEqual => compiler.emit_opcodes(OpCode::Greater, OpCode::Not),
        _ => unreachable!(),
    }
}

fn unary(compiler: &mut Compiler) {
    let operator_type = compiler.previous.unwrap().token_type;

    // Compile the expression
    compiler.parse_precedence(Precedence::Unary);

    match operator_type {
        TokenType::Minus => compiler.emit_opcode(OpCode::Negate),
        TokenType::Bang => compiler.emit_opcode(OpCode::Not),
        _ => {}
    }
}

fn number(compiler: &mut Compiler) {
    let value: f64 = compiler.previous.unwrap().lexeme.parse().unwrap();
    compiler.emit_constant(Value::Number(value));
}

fn literal(compiler: &mut Compiler) {
    match compiler.previous.unwrap().token_type {
        TokenType::False => compiler.emit_opcode(OpCode::False),
        TokenType::Nil => compiler.emit_opcode(OpCode::Nil),
        TokenType::True => compiler.emit_opcode(OpCode::True),
        _ => unreachable!(),
    }
}

fn string(compiler: &mut Compiler) {
    let lexeme = compiler.previous.unwrap().lexeme;
    let lexeme = &lexeme[1..lexeme.len() - 1];
    let object = Object::Str(LoxString::copy_string(compiler.vm, lexeme));
    let value = Value::Obj(Box::new(object));

    compiler.emit_constant(value);
}

static RULE_TABLE: [ParseRule; 40] = [
    // LeftParen
    ParseRule {
        prefix: Some(grouping),
        infix: None,
        precedence: Precedence::None,
    },
    // RightParen
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // LeftBrace
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // RightBrace
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Comma
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Dot
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Minus
    ParseRule {
        prefix: Some(unary),
        infix: Some(binary),
        precedence: Precedence::Term,
    },
    // Plus
    ParseRule {
        prefix: None,
        infix: Some(binary),
        precedence: Precedence::Term,
    },
    // Semicolon
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Slash
    ParseRule {
        prefix: None,
        infix: Some(binary),
        precedence: Precedence::Factor,
    },
    // Star
    ParseRule {
        prefix: None,
        infix: Some(binary),
        precedence: Precedence::Factor,
    },
    // Bang
    ParseRule {
        prefix: Some(unary),
        infix: None,
        precedence: Precedence::None,
    },
    // BangEqual
    ParseRule {
        prefix: Some(binary),
        infix: Some(binary),
        precedence: Precedence::Equality,
    },
    // Equal
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // EqualEqual
    ParseRule {
        prefix: Some(binary),
        infix: Some(binary),
        precedence: Precedence::Equality,
    },
    // Greater
    ParseRule {
        prefix: Some(binary),
        infix: Some(binary),
        precedence: Precedence::Comparison,
    },
    // GreaterEqual
    ParseRule {
        prefix: Some(binary),
        infix: Some(binary),
        precedence: Precedence::Comparison,
    },
    // Less
    ParseRule {
        prefix: Some(binary),
        infix: Some(binary),
        precedence: Precedence::Comparison,
    },
    // LessEqual
    ParseRule {
        prefix: Some(binary),
        infix: Some(binary),
        precedence: Precedence::Comparison,
    },
    // Identifier
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // String
    ParseRule {
        prefix: Some(string),
        infix: None,
        precedence: Precedence::None,
    },
    // Number
    ParseRule {
        prefix: Some(number),
        infix: None,
        precedence: Precedence::None,
    },
    // And
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Class
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Else
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // False
    ParseRule {
        prefix: Some(literal),
        infix: None,
        precedence: Precedence::None,
    },
    // Fun
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // For
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // If
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Nil
    ParseRule {
        prefix: Some(literal),
        infix: None,
        precedence: Precedence::None,
    },
    // Or
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Print
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Return
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Super
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // This
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // True
    ParseRule {
        prefix: Some(literal),
        infix: None,
        precedence: Precedence::None,
    },
    // Var
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // While
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Error
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
    // Eof
    ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    },
];
