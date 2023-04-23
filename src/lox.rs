use std::fs;
use std::io;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

pub fn run_file(filepath: &str) -> io::Result<()> {
    Ok(run(&fs::read_to_string(filepath)?))
}

pub fn run_prompt() -> io::Result<()> {
    let mut stdin = io::stdin().lines();
    loop {
        print!("> ");
        io::stdout().flush()?;
        if let Some(line) = stdin.next() {
            run(&line?);
            HAD_ERROR.store(false, Ordering::Relaxed)
        } else {
            dbg!("EOF");
            break;
        }
    }
    Ok(())
}

fn run(string: &str) {
    dbg!(string);
}

pub fn error(line: usize, message: &str) {
    report(line, "", message)
}

fn report(line: usize, r#where: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, r#where, message);
    HAD_ERROR.store(true, Ordering::Relaxed)
}

pub fn had_error() -> bool {
    HAD_ERROR.load(Ordering::Relaxed)
}
