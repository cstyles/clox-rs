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
    locals: Locals<'src>,
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
            locals: Locals::new(),
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
        if self.match_(TokenType::Var) {
            self.variable_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_(TokenType::Print) {
            self.print_statement();
        } else if self.match_(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_opcode(OpCode::Print);
    }

    fn begin_scope(&mut self) {
        self.locals.scope_depth += 1;
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn end_scope(&mut self) {
        self.locals.scope_depth -= 1;

        // Clean up any local variables in the current scope
        while let Some(local) = self.locals.locals.last() {
            // Stop if we encounter a parent scope
            if local
                .depth
                .map_or(true, |depth| depth < self.locals.scope_depth)
            {
                break;
            }

            self.emit_opcode(OpCode::Pop);
            self.locals.locals.pop();
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_opcode(OpCode::Pop);
    }

    fn variable_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_opcode(OpCode::Nil);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
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

        let can_assign = match prefix_rule {
            Some(prefix_func) => {
                let can_assign = precedence <= Precedence::Assignment;
                prefix_func(self, can_assign);
                can_assign
            }
            None => {
                self.error("Expect expression.");
                return;
            }
        };

        while (precedence)
            <= self
                .get_parse_rule(self.current.unwrap().token_type)
                .precedence
        {
            self.advance();
            let infix_rule = self.get_parse_rule(self.previous.unwrap().token_type).infix;
            if let Some(infix_func) = infix_rule {
                infix_func(self, can_assign);
            }
        }

        if can_assign && self.match_(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);
        self.declare_variable();

        // We don't look up local variables by name at runtime so we
        // don't need to add the variable's name to the constant table
        if self.locals.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(&self.previous.unwrap())
    }

    fn identifier_constant(&mut self, name: &Token) -> u8 {
        let object = Object::Str(LoxString::copy_string(self.vm, name.lexeme));
        let value = Value::Obj(Box::new(object));
        self.make_constant(value)
    }

    fn declare_variable(&mut self) {
        // We only declare local variables so exit early if we're in the global scope
        if self.locals.scope_depth == 0 {
            return;
        }

        // We just consumed the identifier so this is safe
        let name = self.previous.unwrap();

        if self.locals.contains_in_current_scope(name) {
            self.error("Already a variable with this name in this scope.");
        }

        if self.locals.add(name).is_err() {
            self.error("Too many local variables in function.");
        }
    }

    fn define_variable(&mut self, global: u8) {
        // We don't need to create a local variable at runtime because
        // its value is already on top of the stack.
        if self.locals.scope_depth > 0 {
            self.locals.mark_initialized();
            return;
        }

        self.emit_opcode(OpCode::DefineGlobal);
        self.emit_byte(global);
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.locals.resolve_local(name);
        let (arg, get_op, set_op) = match arg {
            Err(ResolveLocalError::Uninitialized) => {
                self.error("Can't read local variable in its own initializer.");
                return;
            }
            Ok(arg) => (arg, OpCode::GetLocal, OpCode::SetLocal),
            Err(ResolveLocalError::NotFound) => (
                self.identifier_constant(&name),
                OpCode::GetGlobal,
                OpCode::SetGlobal,
            ),
        };

        if can_assign && self.match_(TokenType::Equal) {
            self.expression();
            self.emit_opcode(set_op);
            self.emit_byte(arg);
        } else {
            self.emit_opcode(get_op);
            self.emit_byte(arg);
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

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive, PartialOrd, PartialEq, Ord, Eq)]
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

type ParseFn = fn(compiler: &mut Compiler<'_, '_>, can_assign: bool) -> ();

const UINT8_COUNT: usize = u8::MAX as usize + 1;

#[derive(Debug)]
struct Locals<'src> {
    locals: Vec<Local<'src>>,
    scope_depth: usize,
}

impl<'src> Locals<'src> {
    fn new() -> Self {
        Self {
            locals: Vec::with_capacity(UINT8_COUNT),
            scope_depth: 0,
        }
    }

    fn add(&mut self, name: Token<'src>) -> Result<(), ()> {
        // Check if we already have the maximum number of local variables
        if self.locals.len() == UINT8_COUNT {
            return Err(());
        }

        let local = Local::new(name);
        self.locals.push(local);
        Ok(())
    }

    fn contains_in_current_scope(&self, name: Token<'src>) -> bool {
        for local in self.locals.iter().rev() {
            if local.depth.map_or(false, |depth| depth < self.scope_depth) {
                break;
            }

            if local.name.identifiers_equal(&name) {
                return true;
            }
        }

        false
    }

    fn resolve_local(&self, name: Token<'src>) -> Result<u8, ResolveLocalError> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name.identifiers_equal(&name) {
                if local.depth.is_none() {
                    // We're trying to resolve a variable before it's initialized
                    // (e.g., inside its own initializer).
                    return Err(ResolveLocalError::Uninitialized);
                } else {
                    return Ok(i as u8);
                }
            }
        }

        Err(ResolveLocalError::NotFound)
    }

    fn mark_initialized(&mut self) {
        self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
    }
}

enum ResolveLocalError {
    Uninitialized,
    NotFound,
}

#[derive(Debug)]
struct Local<'src> {
    name: Token<'src>,
    depth: Option<usize>,
}

impl<'src> Local<'src> {
    fn new(name: Token<'src>) -> Self {
        Self { name, depth: None }
    }
}

fn grouping(compiler: &mut Compiler, _can_assign: bool) {
    compiler.expression();
    compiler.consume(TokenType::RightParen, "Expect ')' after expression.");
}

fn binary(compiler: &mut Compiler, _can_assign: bool) {
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

fn unary(compiler: &mut Compiler, _can_assign: bool) {
    let operator_type = compiler.previous.unwrap().token_type;

    // Compile the expression
    compiler.parse_precedence(Precedence::Unary);

    match operator_type {
        TokenType::Minus => compiler.emit_opcode(OpCode::Negate),
        TokenType::Bang => compiler.emit_opcode(OpCode::Not),
        _ => {}
    }
}

fn number(compiler: &mut Compiler, _can_assign: bool) {
    let value: f64 = compiler.previous.unwrap().lexeme.parse().unwrap();
    compiler.emit_constant(Value::Number(value));
}

fn literal(compiler: &mut Compiler, _can_assign: bool) {
    match compiler.previous.unwrap().token_type {
        TokenType::False => compiler.emit_opcode(OpCode::False),
        TokenType::Nil => compiler.emit_opcode(OpCode::Nil),
        TokenType::True => compiler.emit_opcode(OpCode::True),
        _ => unreachable!(),
    }
}

fn string(compiler: &mut Compiler, _can_assign: bool) {
    let lexeme = compiler.previous.unwrap().lexeme;
    let lexeme = &lexeme[1..lexeme.len() - 1];
    let object = Object::Str(LoxString::copy_string(compiler.vm, lexeme));
    let value = Value::Obj(Box::new(object));

    compiler.emit_constant(value);
}

fn variable(compiler: &mut Compiler, can_assign: bool) {
    compiler.named_variable(compiler.previous.unwrap(), can_assign);
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
        prefix: Some(variable),
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
