use std::io::{self, Read, Write};

#[macro_use]
mod macros;

mod cmd;
mod error;
mod parser;
mod writer;

pub type Result<T> = std::result::Result<T, Error>;
pub use error::Error;

use cmd::Command;
use parser::Inputs;

pub fn repl(f: impl Fn(&str) -> Result<()>) -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut buf: Vec<u8> = vec![];

        for byte in stdin.bytes() {
            let byte = byte?;
            buf.push(byte);

            if byte == b'\n' {
                break;
            }
        }

        let input = String::from_utf8(buf)?;

        f(input.as_str().trim())?;
    }
}

pub fn exec_cmd(inputs: &str) -> Result<()> {
    let input = Inputs::parse(inputs);
    let mut writer = input.writer()?;
    Command::new(input.args).run(&mut writer)
}
