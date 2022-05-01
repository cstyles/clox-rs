use crate::scanner::{Scanner, TokenType};

pub fn compile(source: &str) {
    // let source: Vec<char> = source.chars().collect();
    let mut scanner = Scanner::new(source);
    let mut line = usize::MAX;

    loop {
        let token = scanner.scan_token();
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        // TODO
        println!("{:2?} {:?}", token.token_type, token.start);

        if token.token_type == TokenType::Eof {
            break;
        }
    }
}
