use codecrafters_shell::{exec_cmd, repl};

fn main() {
    if let Err(err) = repl(exec_cmd) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
