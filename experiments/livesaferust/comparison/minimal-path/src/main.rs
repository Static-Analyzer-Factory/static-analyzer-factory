use std::env;
use std::fs::File;

fn main() {
    let path = env::args().nth(1).unwrap_or_default();
    let _ = File::open(path);
}
