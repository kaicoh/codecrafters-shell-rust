use crate::Result;
use std::io::{self, ErrorKind, Write};

mod fs;
mod parser;

use parser::Args;

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Echo(Args<'a>),
    Type(Args<'a>),
    Exit(Args<'a>),
    Pwd,
    Cd(Args<'a>),
    Empty,
    Unknown(String, Args<'a>),
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
    pub fn new(inputs: impl Into<Args<'a>>) -> Self {
        let mut args: Args<'a> = inputs.into();
        let cmd = args.next().unwrap_or_default();

        match cmd.as_str() {
            "echo" => Self::Echo(args),
            "type" => Self::Type(args),
            "exit" => Self::Exit(args),
            "pwd" => Self::Pwd,
            "cd" => Self::Cd(args),
            "" => Self::Empty,
            _ => Self::Unknown(cmd, args),
        }
    }

    pub fn run(self) -> CommandResult {
        match self {
            Self::Echo(args) => {
                let msg = args.into_iter().collect::<Vec<String>>().join(" ");
                stdout!("{msg}")
            }
            Self::Type(args) => {
                let cmd = Command::new(args);
                let cmd_str = cmd.as_str();

                match cmd {
                    Command::Empty => stdout!("{cmd_str}: not found"),
                    Command::Unknown(ref name, _) => match executable(name) {
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
            Self::Unknown(name, rest) => match executable(&name) {
                Ok(Some(_)) => {
                    if let Err(err) = run_cmd(&name, rest) {
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
        assert_eq!(cmd, Command::Unknown("invalid_command".into(), "".into()));

        let inputs = "invalid_command foo bar";
        let cmd = Command::new(inputs);
        assert_eq!(
            cmd,
            Command::Unknown("invalid_command".into(), "foo bar".into())
        );
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
