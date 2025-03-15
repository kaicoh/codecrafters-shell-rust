use std::io::Write;

#[macro_use]
mod macros;

mod cmd;
mod error;
mod parser;
mod writer;

pub type Result<T> = std::result::Result<T, Error>;
pub use error::Error;

use cmd::Command;
use console::{Key, Term};
use parser::Inputs;

pub fn repl(f: impl Fn(&str) -> Result<()>) -> Result<()> {
    let mut term = Term::stdout();

    loop {
        write!(term, "$ ")?;

        let mut buf: Vec<u8> = vec![];
        let mut candidates: Vec<String> = vec![];

        while let Ok(key) = term.read_key() {
            match key {
                Key::Enter => {
                    writeln!(term)?;
                    break;
                }
                Key::Char(c) => {
                    write!(term, "{c}")?;
                    buf.push(c as u8);
                }
                Key::Tab => {
                    let input = std::str::from_utf8(&buf)?;

                    if candidates.is_empty() {
                        candidates = Command::autocomplete(input);
                        candidates.sort();

                        if candidates.len() == 1 {
                            if let Some(cmd) = candidates.pop() {
                                term.clear_line()?;

                                let completed = format!("{cmd} ");
                                write!(term, "$ {completed}")?;
                                buf = completed.into_bytes();
                            }
                        } else {
                            write!(term, "\x07")?;
                        }
                    } else {
                        writeln!(term, "\n{}", candidates.join("  "))?;
                        write!(term, "$ {input}")?;
                    }

                    continue;
                }
                Key::Backspace => {
                    term.clear_chars(1)?;
                    buf.pop();
                }
                _ => {}
            }

            candidates = vec![];
        }

        let input = std::str::from_utf8(&buf)?;

        f(input.trim())?;
    }
}

pub fn exec_cmd(inputs: &str) -> Result<()> {
    let input = Inputs::parse(inputs);
    let mut writer = input.writer()?;
    Command::new(input.args).run(&mut writer)
}
