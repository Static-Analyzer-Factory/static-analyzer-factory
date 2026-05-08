use std::env;

fn main() {
    let value = env::args().nth(1).unwrap_or_default();
    println!("{}", value);
}
