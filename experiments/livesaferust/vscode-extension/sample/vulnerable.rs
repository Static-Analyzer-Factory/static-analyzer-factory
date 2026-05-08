use std::env;
use std::process::Command;

fn main() {

    let cmdsssss = env::args().nth(1).unwrap_or_default();
    let _ = Command::new(cmdsssss).status();
}
