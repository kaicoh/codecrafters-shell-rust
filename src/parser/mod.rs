use crate::Result;

mod args;

use super::writer::Writer;
use args::Args;

#[derive(Debug)]
pub struct Inputs {
    pub args: Vec<String>,
    stdout_new: Option<String>,
    stdout_append: Option<String>,
    stderr_new: Option<String>,
    stderr_append: Option<String>,
}

impl Inputs {
    pub fn parse(input: &str) -> Self {
        let mut args: Args<'_> = input.into();
        let mut tokens: Vec<String> = vec![];
        let mut stdout_new: Option<String> = None;
        let mut stdout_append: Option<String> = None;
        let mut stderr_new: Option<String> = None;
        let mut stderr_append: Option<String> = None;

        while let Some(token) = args.next() {
            match token.as_str() {
                ">" | "1>" => {
                    stdout_new = args.next();
                }
                ">>" | "1>>" => {
                    stdout_append = args.next();
                }
                "2>" => {
                    stderr_new = args.next();
                }
                "2>>" => {
                    stderr_append = args.next();
                }
                _ => {
                    tokens.push(token);
                }
            }
        }

        Self {
            args: tokens,
            stdout_new,
            stdout_append,
            stderr_new,
            stderr_append,
        }
    }

    pub fn writer(&self) -> Result<Writer> {
        let mut builder = Writer::builder();

        if let Some(ref path) = self.stdout_new {
            builder = builder.stdout_new(path);
        }

        if let Some(ref path) = self.stdout_append {
            builder = builder.stdout_append(path);
        }

        if let Some(ref path) = self.stderr_new {
            builder = builder.stderr_new(path);
        }

        if let Some(ref path) = self.stderr_append {
            builder = builder.stderr_append(path);
        }

        builder.build()
    }
}
