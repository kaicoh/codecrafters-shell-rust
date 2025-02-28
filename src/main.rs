use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        if input.as_str().trim() == "exit 0" {
            std::process::exit(0);
        } else {
            println!("{}: command not found", input.trim_end());
        }
    }
}
