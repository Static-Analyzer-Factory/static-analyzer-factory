#![cfg(feature = "llvm-22")]

use std::path::PathBuf;

use saf_core::config::Config;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("llvm")
        .join("e2e")
        .join(name)
}

#[test]
fn rust_std_command_ir_loads_frontend() {
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let path = fixture("rust_std_command.ll");
    let bundle = frontend
        .ingest(&[path.as_path()], &config)
        .expect("rust std::process::Command IR should load through LLVM frontend");
    assert!(!bundle.module.functions.is_empty());
}
