use super::{writer::Writer, Result};

use std::fmt;
use std::io::ErrorKind;

mod fs;

#[derive(Debug, PartialEq)]
pub struct Command {
    r#type: CommandType,
    args: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CommandType {
    Echo,
    Type,
    Exit,
    Pwd,
    Cd,
    Empty,
    Unknown(String),
}

impl Command {
    pub fn new(tokens: Vec<String>) -> Self {
        let mut tokens = tokens.into_iter();
        let cmd = tokens.next().unwrap_or_default();

        let r#type = match cmd.as_str() {
            "echo" => CommandType::Echo,
            "type" => CommandType::Type,
            "exit" => CommandType::Exit,
            "pwd" => CommandType::Pwd,
            "cd" => CommandType::Cd,
            "" => CommandType::Empty,
            _ => CommandType::Unknown(cmd),
        };

        Self {
            r#type,
            args: tokens.collect(),
        }
    }

    pub fn autocomplete(s: &str) -> Option<String> {
        for cmd in CommandType::builtins() {
            if cmd.to_string().starts_with(s) {
                return Some(cmd.to_string());
            }
        }

        all_executable_names()
            .into_iter()
            .find(|name| name.starts_with(s))
    }

    pub fn run(self, w: &mut Writer) -> Result<()> {
        match &self.r#type {
            CommandType::Echo => {
                let msg = self.args.join(" ");
                w.writeln(msg)
            }
            CommandType::Type => {
                let cmd = Command::new(self.args);

                match cmd.r#type {
                    CommandType::Empty => w.writeln(format!("{}: not found", cmd.r#type)),
                    CommandType::Unknown(ref name) => match executable(name) {
                        Ok(Some(path)) => w.writeln(format!("{} is {path}", cmd.r#type)),
                        Ok(None) => w.writeln(format!("{}: not found", cmd.r#type)),
                        Err(err) => w.ewriteln(format!("{err}")),
                    },
                    _ => w.writeln(format!("{} is a shell builtin", cmd.r#type)),
                }
            }
            CommandType::Exit => {
                let code = self.args.into_iter().next().unwrap_or_default();
                match code.parse::<i32>() {
                    Ok(code) => {
                        std::process::exit(code);
                    }
                    Err(_) => w.ewriteln("exit code should be a number"),
                }
            }
            CommandType::Pwd => {
                let current_dir = fs::current_dir().and_then(|p| {
                    fs::path_stringify(p).ok_or(err!("Cannot stringify current directory path"))
                });
                match current_dir {
                    Ok(dir) => w.writeln(dir),
                    Err(err) => w.ewriteln(format!("{err}")),
                }
            }
            CommandType::Cd => {
                let dir = self.args.into_iter().next().unwrap_or_default();
                let target = if dir == "~" {
                    std::env::var("HOME").unwrap_or_default()
                } else {
                    dir.to_string()
                };

                if let Err(err) = std::env::set_current_dir(&target) {
                    if err.kind() == ErrorKind::NotFound {
                        w.ewriteln(format!("cd: {dir}: No such file or directory"))?;
                    } else {
                        w.ewriteln(format!("{err}"))?;
                    }
                }
                Ok(())
            }
            CommandType::Empty => Ok(()),
            CommandType::Unknown(name) => match executable(name) {
                Ok(Some(_)) => match run_cmd(name, &self.args) {
                    Ok(output) => {
                        w.write(&output.stdout)?;
                        w.ewrite(&output.stderr)
                    }
                    Err(err) => w.ewriteln(format!("{err}")),
                },
                Ok(None) => w.ewriteln(format!("{name}: command not found")),
                Err(err) => w.ewriteln(format!("{err}")),
            },
        }
    }
}

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Echo => "echo",
            Self::Type => "type",
            Self::Exit => "exit",
            Self::Pwd => "pwd",
            Self::Cd => "cd",
            Self::Empty => "",
            Self::Unknown(cmd) => cmd.as_str(),
        };
        write!(f, "{str}")
    }
}

impl CommandType {
    fn builtins() -> impl Iterator<Item = Self> {
        [Self::Echo, Self::Type, Self::Exit, Self::Pwd, Self::Cd].into_iter()
    }
}

fn run_cmd(name: &str, args: &[String]) -> Result<std::process::Output> {
    let output = std::process::Command::new(name).args(args).output()?;
    Ok(output)
}

fn all_executable_names() -> Vec<String> {
    let path = std::env::var("PATH").unwrap_or_default();
    fs::list_dirs(&path)
        .into_iter()
        .filter_map(|dir| fs::list_files(dir).ok())
        .flatten()
        .filter_map(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect()
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
        let inputs = vec!["echo".into(), "foo".into(), "bar".into()];
        let cmd = Command::new(inputs);
        let expected = Command {
            r#type: CommandType::Echo,
            args: vec!["foo".into(), "bar".into()],
        };
        assert_eq!(cmd, expected);

        let inputs = vec!["exit".into(), "0".into()];
        let cmd = Command::new(inputs);
        let expected = Command {
            r#type: CommandType::Exit,
            args: vec!["0".into()],
        };
        assert_eq!(cmd, expected);

        let inputs = vec![];
        let cmd = Command::new(inputs);
        let expected = Command {
            r#type: CommandType::Empty,
            args: vec![],
        };
        assert_eq!(cmd, expected);

        let inputs = vec!["invalid_command".into()];
        let cmd = Command::new(inputs);
        let expected = Command {
            r#type: CommandType::Unknown("invalid_command".into()),
            args: vec![],
        };
        assert_eq!(cmd, expected);

        let inputs = vec!["invalid_command".into(), "foo".into(), "bar".into()];
        let cmd = Command::new(inputs);
        let expected = Command {
            r#type: CommandType::Unknown("invalid_command".into()),
            args: vec!["foo".into(), "bar".into()],
        };
        assert_eq!(cmd, expected);
    }

    #[test]
    fn it_completes_the_command() {
        let subject = Command::autocomplete("ech");
        assert_eq!(subject, Some("echo".into()));

        let subject = Command::autocomplete("exi");
        assert_eq!(subject, Some("exit".into()));
    }
}
