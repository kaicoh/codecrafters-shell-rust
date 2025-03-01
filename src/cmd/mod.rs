#[derive(Debug, PartialEq)]
pub enum Command {
    Echo(String),
    Exit(String),
    Empty,
    Unknown(String),
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

impl Command {
    pub fn new(inputs: &str) -> Self {
        match split_token(inputs.trim()) {
            ("echo", rest) => Self::Echo(rest.into()),
            ("exit", rest) => Self::Exit(rest.into()),
            ("", _) => Self::Empty,
            (other, _) => Self::Unknown(other.into()),
        }
    }

    pub fn run(self) -> CommandResult {
        match self {
            Self::Echo(msg) => stdout!("{msg}"),
            Self::Exit(code) => match code.parse::<i32>() {
                Ok(code) => CommandResult::Exit(code),
                Err(_) => stdout!("exit code should be a number"),
            },
            Self::Empty => stdout!(),
            Self::Unknown(cmd) => stdout!("{cmd}: command not found"),
        }
    }
}

fn split_token(inputs: &str) -> (&str, &str) {
    match inputs.split_once(' ') {
        Some((first, rest)) => (first, rest.trim()),
        None => (inputs, ""),
    }
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
        assert_eq!(cmd, Command::Unknown("invalid_command".into()));

        let inputs = "invalid_command foo bar";
        let cmd = Command::new(inputs);
        assert_eq!(cmd, Command::Unknown("invalid_command".into()));
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
