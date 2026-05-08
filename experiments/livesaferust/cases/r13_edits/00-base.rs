// class=base
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from("worktree")
}

fn record_destination(path: &Path) {
    log_checkout("destination", path);
}

fn normalize_display_name(entry_name: &str) -> String {
    entry_name.to_string()
}

fn entry_kind(entry_name: &str) -> &'static str {
    if entry_name.is_empty() {
        "empty"
    } else {
        "named"
    }
}

fn note_entry_shape(entry_name: &str) {
    if should_log() {
        eprintln!("entry-kind={}", entry_kind(entry_name));
    }
}

fn note_destination_parent(path: &Path) {
    if should_log() {
        if let Some(parent) = path.parent() {
            eprintln!("parent={}", parent.display());
        }
    }
}

fn checkout_entry(worktree: PathBuf, entry_name: String) {
    note_entry_shape(&entry_name);
    let destination = worktree.join(entry_name);
    record_destination(&destination);
    note_destination_parent(&destination);
    let _ = File::create(destination);
}

fn sanitize_path(_: String) -> String {
    "safe-entry".to_string()
}

pub fn preview_entry(raw: String) -> PathBuf {
    workspace_root().join(sanitize_path(raw))
}

fn status_label() -> &'static str {
    "checkout"
}

fn should_log() -> bool {
    false
}

fn log_checkout(label: &str, path: &Path) {
    if should_log() {
        eprintln!("{}: {}", label, path.display());
    }
}

fn debug_point(worktree: &Path) {
    log_checkout(status_label(), worktree);
}

pub fn preview_from_args() -> PathBuf {
    let raw = env::args().nth(2).unwrap_or_default();
    preview_entry(normalize_display_name(&raw))
}

fn main() {
    let entry_name = env::args().nth(1).unwrap_or_default();
    let _preview = preview_from_args();
    let worktree = workspace_root();
    debug_point(&worktree);
    checkout_entry(worktree, entry_name);
}
