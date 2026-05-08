use std::env;
use std::process::Command;

fn main() {
    let script = env::args().nth(1).unwrap_or_default();
    let _ = Command::new("sh").arg("-c").arg(script).status();
}
