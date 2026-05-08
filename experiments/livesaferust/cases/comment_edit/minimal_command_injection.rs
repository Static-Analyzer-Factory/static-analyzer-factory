use std::env;
use std::process::Command;

fn main() {
    // Formatting-only edit for the LiveSafeRust cache-hit experiment.
    let cmd = env::args().nth(1).unwrap_or_default();
    let _ = Command::new(cmd).status();
}
