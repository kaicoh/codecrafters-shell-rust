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
                            let common = common_parts(&candidates);

                            if input.len() < common.len() {
                                term.clear_line()?;

                                write!(term, "$ {common}")?;
                                buf = common.into_bytes();
                            } else {
                                write!(term, "\x07")?;
                            }
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

fn common_parts(names: &[String]) -> String {
    if let Some(name) = names.first() {
        for i in 1..(name.len() + 1) {
            let pattern = &name[..i];

            if !names.iter().all(|n| n.starts_with(pattern)) {
                return name[..i - 1].to_string();
            }
        }

        name.to_string()
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_extracts_common_parts() {
        let names = vec![];
        let subject = common_parts(&names);
        assert_eq!(subject, "");

        let names = vec!["foo".to_string(), "bar".to_string()];
        let subject = common_parts(&names);
        assert_eq!(subject, "");

        let names = vec!["foo".to_string(), "far".to_string()];
        let subject = common_parts(&names);
        assert_eq!(subject, "f");

        let names = vec!["foo_bar".to_string(), "foo_baz".to_string()];
        let subject = common_parts(&names);
        assert_eq!(subject, "foo_ba");

        let names = vec![
            "foo_bar".to_string(),
            "foo_baz".to_string(),
            "foo_c".to_string(),
        ];
        let subject = common_parts(&names);
        assert_eq!(subject, "foo_");
    }
}
