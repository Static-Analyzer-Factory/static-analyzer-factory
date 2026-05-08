use std::env;
use std::process::Command;

fn sanitize_cmd(value: String) -> String {
    value.replace(";", "")
}

fn main() {
    let raw = env::args().nth(1).unwrap_or_default();
    let cmd = sanitize_cmd(raw);
    let _ = Command::new(cmd).status();
}
