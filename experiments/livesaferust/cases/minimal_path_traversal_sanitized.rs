use std::env;
use std::fs::File;

fn sanitize_path(_: String) -> String {
    "/tmp/livesaferust-safe.txt".to_string()
}

fn main() {
    let raw = env::args().nth(1).unwrap_or_default();
    let path = sanitize_path(raw);
    let _ = File::open(path);
}
