use std::fs;
use std::io;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use crate::interpreter;
use crate::parser::Parser;
use crate::scanning::Scanner;
use crate::tokens;

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

pub fn run_file(filepath: &str) -> io::Result<()> {
    Ok(run(&fs::read_to_string(filepath)?))
}

pub fn run_prompt() -> io::Result<()> {
    let mut stdin = io::stdin().lines();
    while let Some(line) = {
        print!("> ");
        io::stdout().flush()?;
        stdin.next()
    } {
        run(&line?);
        HAD_ERROR.store(false, Ordering::Relaxed)
    }
    Ok(())
}

fn run(string: &str) {
    let tokens = Scanner::new(string).scan_tokens();
    let parser = Parser::new(tokens);
    let statements = parser.parse();
    if !had_error() {
        interpreter::interpret(&statements)
            .unwrap_or_else(|interpreter::EvaluateError(message)| error(0, &message))
    }
}

pub fn error(line: usize, message: &str) {
    report(line, "", message)
}

pub fn error_from_token(token: &tokens::Token, message: &str) {
    if token.token_type == tokens::TokenType::EOF {
        report(token.line, "end", message);
    } else {
        report(token.line, &format!("'{}'", token.lexeme), message);
    }
}

fn report(line: usize, at: &str, message: &str) {
    eprintln!("[line {}] Error at {}: {}", line, at, message);
    HAD_ERROR.store(true, Ordering::Relaxed)
}

pub fn had_error() -> bool {
    HAD_ERROR.load(Ordering::Relaxed)
}
