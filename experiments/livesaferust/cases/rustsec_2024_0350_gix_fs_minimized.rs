use std::env;
use std::fs::File;
use std::path::PathBuf;

fn checkout_entry(worktree: PathBuf, entry_name: String) {
    let destination = worktree.join(entry_name);
    let _ = File::create(destination);
}

fn main() {
    let entry_name = env::args().nth(1).unwrap_or_default();
    checkout_entry(PathBuf::from("worktree"), entry_name);
}
