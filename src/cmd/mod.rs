use super::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum Command {
    Echo(String),
    Exit(i32),
    Empty,
}

#[derive(Debug, PartialEq)]
pub enum CommandResult {
    Continue,
    Exit(i32),
}

impl Command {
    pub fn new(inputs: &str) -> Result<Self> {
        match inputs.split_once(' ') {
            Some(("echo", rest)) => Ok(Self::Echo(rest.trim().into())),
            Some(("exit", rest)) => {
                let code = rest
                    .trim()
                    .parse::<i32>()
                    .map_err(|_| Error::ParseCommand("exit code should be a number".into()))?;
                Ok(Self::Exit(code))
            }
            Some((other, _)) => Err(Error::ParseCommand(format!("{}: command not found", other))),
            None if !inputs.is_empty() => Err(Error::ParseCommand(format!(
                "{}: command not found",
                inputs
            ))),
            None => Ok(Self::Empty),
        }
    }

    pub fn run(self) -> CommandResult {
        match self {
            Self::Echo(msg) => {
                println!("{msg}");
                CommandResult::Continue
            }
            Self::Exit(code) => CommandResult::Exit(code),
            Self::Empty => {
                println!();
                CommandResult::Continue
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_to_cmd() {
        let inputs = "echo foo bar";
        let cmd = Command::new(inputs).unwrap();
        assert_eq!(cmd, Command::Echo("foo bar".into()));

        let inputs = "exit 0";
        let cmd = Command::new(inputs).unwrap();
        assert_eq!(cmd, Command::Exit(0));

        let inputs = "";
        let cmd = Command::new(inputs).unwrap();
        assert_eq!(cmd, Command::Empty);

        match Command::new("invalid_command") {
            Err(Error::ParseCommand(msg)) => {
                assert_eq!(msg, "invalid_command: command not found");
            }
            _ => unreachable!(),
        }

        match Command::new("invalid_command foo bar") {
            Err(Error::ParseCommand(msg)) => {
                assert_eq!(msg, "invalid_command: command not found");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_ignores_additional_spaces() {
        let inputs = "echo  foo   bar";
        let cmd = Command::new(inputs).unwrap();
        assert_eq!(cmd, Command::Echo("foo   bar".into()));

        let inputs = "exit  1";
        let cmd = Command::new(inputs).unwrap();
        assert_eq!(cmd, Command::Exit(1));
    }
}
