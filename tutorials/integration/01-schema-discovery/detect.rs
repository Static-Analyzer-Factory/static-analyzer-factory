// Discover SAF's capabilities using the Rust API.
//
// The Rust API doesn't have a schema() equivalent, but you can
// inspect the module structure and available analysis types.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_schema_discovery

use std::path::PathBuf;
use std::process::Command;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::defuse::DefUseGraph;
use saf_core::config::Config;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    let tutorial_dir = PathBuf::from("tutorials/integration/01-schema-discovery");
    let source = tutorial_dir.join("program.c");
    let llvm_ir = tutorial_dir.join("program.ll");

    let status = Command::new("clang-18")
        .args(["-S", "-emit-llvm", "-O0", "-g",
               "-o", llvm_ir.to_str().unwrap(), source.to_str().unwrap()])
        .status()
        .expect("failed to run clang-18");
    assert!(status.success());

    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let bundle = frontend.ingest(&[llvm_ir.as_path()], &config)
        .expect("failed to load LLVM IR");
    let module = bundle.module;

    println!("SAF Module Discovery:");
    println!("  Functions: {}", module.functions.len());
    println!("  Globals: {}", module.globals.len());

    // Available analysis types
    println!("\nAvailable analyses:");
    println!("  - CFG (control flow graph)");
    println!("  - CallGraph (inter-procedural call graph)");
    println!("  - DefUse (definition-use chains)");
    println!("  - PTA (points-to analysis)");
    println!("  - ValueFlow (value flow graph)");

    // Function details
    println!("\nFunction details:");
    for func in &module.functions {
        println!("  {} ({} blocks, {} params, is_decl={})",
                 func.name, func.blocks.len(), func.params.len(), func.is_declaration);
    }
}
