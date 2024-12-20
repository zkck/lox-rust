use std::io;
use std::process::exit;

mod environment;
mod expr;
mod interpreter;
mod lox;
mod object;
mod parser;
mod scanning;
mod stmt;
mod tokens;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    filepath: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    match args.filepath {
        Some(filepath) => lox::run_file(&filepath)?,
        None => lox::run_prompt()?,
    };
    if lox::had_error() {
        exit(65);
    }
    Ok(())
}
