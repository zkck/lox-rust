use std::fs;
use std::io;
use std::process::exit;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    filepath: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut lox = Lox::new();
    match args.filepath {
        Some(filepath) => lox.run_file(&filepath),
        None => lox.run_prompt(),
    }?;
    if lox.had_error {
        exit(65);
    }
    Ok(())
}

struct Lox {
    had_error: bool,
}

impl Lox {
    fn new() -> Self {
        Lox { had_error: false }
    }

    fn run_file(&mut self, filepath: &str) -> io::Result<()> {
        Ok(self.run(&fs::read_to_string(filepath)?))
    }

    fn run_prompt(&mut self) -> io::Result<()> {
        let mut stdin = io::stdin().lines();
        let mut stdout = io::stdout();
        loop {
            print!("> ");
            io::Write::flush(&mut stdout)?;
            if let Some(line) = stdin.next() {
                self.run(&line?);
                self.had_error = false;
            } else {
                dbg!("EOF");
                break;
            }
        }
        Ok(())
    }

    fn run(&mut self, string: &str) {
        dbg!(string);
    }

    fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message)
    }

    fn report(&mut self, line: usize, r#where: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, r#where, message);
        self.had_error = true;
    }
}
