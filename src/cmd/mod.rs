use crate::Result;
use std::io::{self, ErrorKind, Write};

mod fs;
mod parser;

use parser::Args;

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Echo(Args<'a>),
    Type(String),
    Exit(Args<'a>),
    Pwd,
    Cd(Args<'a>),
    Empty,
    Unknown(&'a str, Args<'a>),
}

#[derive(Debug, PartialEq)]
pub enum CommandResult {
    Continue,
    Exit(i32),
}

macro_rules! stdout {
    () => {{
        println!();
        CommandResult::Continue
    }};
    ($fmt:expr) => {{
        println!($fmt);
        CommandResult::Continue
    }};
    ($fmt:expr, $($arg:tt)+) => {{
        println!($fmt, $($arg)+);
        CommandResult::Continue
    }};
}

impl<'a> Command<'a> {
    pub fn new(inputs: &'a str) -> Self {
        match split_token(inputs.trim()) {
            ("echo", rest) => Self::Echo(rest.into()),
            ("type", rest) => Self::Type(rest.into()),
            ("exit", rest) => Self::Exit(rest.into()),
            ("pwd", _) => Self::Pwd,
            ("cd", rest) => Self::Cd(rest.into()),
            ("", _) => Self::Empty,
            (cmd, rest) => Self::Unknown(cmd, rest.into()),
        }
    }

    pub fn run(self) -> CommandResult {
        match self {
            Self::Echo(args) => {
                let msg = args.into_iter().collect::<Vec<String>>().join(" ");
                stdout!("{msg}")
            }
            Self::Type(rest) => {
                let cmd = Command::new(&rest);
                let cmd_str = cmd.as_str();

                match cmd {
                    Command::Empty => stdout!("{cmd_str}: not found"),
                    Command::Unknown(name, _) => match executable(name) {
                        Ok(Some(path)) => stdout!("{cmd_str} is {path}"),
                        Ok(None) => stdout!("{cmd_str}: not found"),
                        Err(err) => stdout!("{err}"),
                    },
                    _ => stdout!("{cmd_str} is a shell builtin"),
                }
            }
            Self::Exit(mut args) => {
                let code = args.next().unwrap_or_default();
                match code.parse::<i32>() {
                    Ok(code) => CommandResult::Exit(code),
                    Err(_) => stdout!("exit code should be a number"),
                }
            }
            Self::Pwd => {
                let current_dir = fs::current_dir().and_then(|p| {
                    fs::path_stringify(p).ok_or(err!("Cannot stringify current directory path"))
                });
                match current_dir {
                    Ok(dir) => stdout!("{dir}"),
                    Err(err) => stdout!("{err}"),
                }
            }
            Self::Cd(mut args) => {
                let dir = args.next().unwrap_or_default();
                let target = if dir == "~" {
                    std::env::var("HOME").unwrap_or_default()
                } else {
                    dir.to_string()
                };

                match std::env::set_current_dir(&target) {
                    Ok(_) => CommandResult::Continue,
                    Err(err) if err.kind() == ErrorKind::NotFound => {
                        stdout!("cd: {dir}: No such file or directory")
                    }
                    Err(err) => stdout!("{err}"),
                }
            }
            Self::Empty => stdout!(),
            Self::Unknown(name, rest) => match executable(name) {
                Ok(Some(_)) => {
                    if let Err(err) = run_cmd(name, rest) {
                        eprintln!("{err}");
                    }
                    CommandResult::Continue
                }
                Ok(None) => stdout!("{name}: command not found"),
                Err(err) => stdout!("{err}"),
            },
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Echo(_) => "echo",
            Self::Type(_) => "type",
            Self::Exit(_) => "exit",
            Self::Pwd => "pwd",
            Self::Cd(_) => "cd",
            Self::Empty => "",
            Self::Unknown(cmd, _) => cmd,
        }
    }
}

fn split_token(inputs: &str) -> (&str, &str) {
    match inputs.split_once(' ') {
        Some((first, rest)) => (first, rest.trim()),
        None => (inputs, ""),
    }
}

fn run_cmd(name: &str, args: Args<'_>) -> Result<()> {
    let output = std::process::Command::new(name).args(args).output()?;
    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;
    Ok(())
}

fn executable(name: &str) -> Result<Option<String>> {
    let path = std::env::var("PATH").unwrap_or_default();
    find_executable(&path, name)
}

fn find_executable(path: &str, name: &str) -> Result<Option<String>> {
    for dir in fs::list_dirs(path) {
        if let Some(p) = fs::list_files(dir)?
            .into_iter()
            .find(fs::filename(name))
            .and_then(fs::path_stringify)
        {
            return Ok(Some(p));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_the_first_token() {
        let inputs = "echo foo bar";
        let (first, rest) = split_token(inputs);
        assert_eq!(first, "echo");
        assert_eq!(rest, "foo bar");

        let inputs = "echo  foo   bar";
        let (first, rest) = split_token(inputs);
        assert_eq!(first, "echo");
        assert_eq!(rest, "foo   bar");
    }

    #[test]
    fn it_parses_to_cmd() {
        let inputs = "echo foo bar";
        let cmd = Command::new(inputs);
        assert_eq!(cmd, Command::Echo("foo bar".into()));

        let inputs = "exit 0";
        let cmd = Command::new(inputs);
        assert_eq!(cmd, Command::Exit("0".into()));

        let inputs = "";
        let cmd = Command::new(inputs);
        assert_eq!(cmd, Command::Empty);

        let inputs = "invalid_command";
        let cmd = Command::new(inputs);
        assert_eq!(cmd, Command::Unknown("invalid_command", "".into()));

        let inputs = "invalid_command foo bar";
        let cmd = Command::new(inputs);
        assert_eq!(cmd, Command::Unknown("invalid_command", "foo bar".into()));
    }

    #[test]
    fn it_ignores_additional_spaces() {
        let inputs = "echo  foo   bar";
        let cmd = Command::new(inputs);
        assert_eq!(cmd, Command::Echo("foo   bar".into()));

        let inputs = "exit  1";
        let cmd = Command::new(inputs);
        assert_eq!(cmd, Command::Exit("1".into()));
    }
}
