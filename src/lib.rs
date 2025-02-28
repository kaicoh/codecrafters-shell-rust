use std::io::{self, Write};

#[macro_use]
mod macros;

mod error;

pub type Result<T> = std::result::Result<T, Error>;
pub use error::Error;

#[derive(Debug)]
pub enum CommandResult {
    Continue,
    Exit(i32),
}

pub fn repl(f: impl Fn(&str) -> CommandResult) -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input)?;

        if let CommandResult::Exit(code) = f(input.as_str().trim()) {
            std::process::exit(code);
        }
    }
}

pub fn exec_cmd(inputs: &str) -> CommandResult {
    if inputs == "exit 0" {
        CommandResult::Exit(0)
    } else {
        println!("{}: command not found", inputs);
        CommandResult::Continue
    }
}
