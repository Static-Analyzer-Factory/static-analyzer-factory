use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let entry = env::args().nth(1).unwrap_or_default();
    let root = PathBuf::from("/tmp/worktree");
    let dest = root.join(entry);
    let _ = File::open(dest);

}
