use std::io::{self, Write};

#[macro_use]
mod macros;

mod cmd;
mod error;

pub type Result<T> = std::result::Result<T, Error>;
pub use error::Error;

use cmd::{Command, CommandResult};

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
    match Command::new(inputs) {
        Ok(cmd) => cmd.run(),
        Err(err) => {
            println!("{err}");
            CommandResult::Continue
        }
    }
}
