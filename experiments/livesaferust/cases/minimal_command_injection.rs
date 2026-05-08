use std::env;
use std::process::Command;

fn main() {
    let cmd = env::args().nth(1).unwrap_or_default();
    let _ = Command::new(cmd).status();
}
