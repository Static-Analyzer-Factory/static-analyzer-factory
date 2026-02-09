// Tutorial 6: Memory SSA — detecting stale-data reads.
//
// Demonstrates how Memory SSA disambiguates memory operations using
// the Rust API. Builds Memory SSA for a program where a function call
// overwrites a pointer target between a store and a load.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_memory_ssa

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::mssa::MemorySsa;
use saf_analysis::{PtaConfig, PtaContext, PtaResult};
use saf_core::air::AirModule;
use saf_core::config::Config;
use saf_core::ids::FunctionId;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    let tutorial_dir = PathBuf::from("tutorials/pta/06-memory-ssa");
    let source = tutorial_dir.join("vulnerable.c");
    let llvm_ir = tutorial_dir.join("vulnerable.ll");

    // Step 1: Compile C source to LLVM IR
    let status = Command::new("clang-18")
        .args([
            "-S",
            "-emit-llvm",
            "-O0",
            "-g",
            "-o",
            llvm_ir.to_str().unwrap(),
            source.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run clang-18");
    assert!(status.success(), "clang-18 compilation failed");

    // Step 2: Load via LLVM frontend
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let bundle = frontend
        .ingest(&[llvm_ir.as_path()], &config)
        .expect("failed to load LLVM IR");
    let module = bundle.module;

    // Step 3: Build CFGs and call graph
    let cfgs: BTreeMap<FunctionId, Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect();
    let callgraph = CallGraph::build(&module);

    // Step 4: Run PTA
    let pta_config = PtaConfig::default();
    let mut pta_ctx = PtaContext::new(pta_config);
    let pta_analysis = pta_ctx.analyze(&module);
    let pta_result = PtaResult::new(
        pta_analysis.pts,
        Arc::new(pta_analysis.factory),
        pta_analysis.diagnostics,
    );

    // Step 5: Build Memory SSA
    let mssa = MemorySsa::build(&module, &cfgs, pta_result, &callgraph);
    println!("Memory SSA built successfully");
    println!("  Total memory accesses: {}", mssa.access_count());

    // Step 6: Export and inspect
    let export = mssa.export();
    println!("  Schema version: {}", export.schema_version);
    println!("  Functions with MSSA: {}", export.functions.len());
    println!("  Functions with mod/ref: {}", export.mod_ref.len());

    // Step 7: Check mod/ref for modify()
    if let Some(modify_fn) = module.functions.iter().find(|f| f.name == "modify") {
        if let Some(summary) = mssa.mod_ref(modify_fn.id) {
            println!("\n  modify() mod/ref summary:");
            println!("    may_mod locations: {}", summary.may_mod.len());
            println!("    may_ref locations: {}", summary.may_ref.len());
            if !summary.may_mod.is_empty() {
                println!(
                    "    (modify() writes to {} memory location(s))",
                    summary.may_mod.len()
                );
            }
        } else {
            println!("\n  modify() has no mod/ref summary");
        }
    }

    // Step 8: Check test() function's mod/ref (transitive)
    if let Some(test_fn) = module.functions.iter().find(|f| f.name == "test") {
        if let Some(summary) = mssa.mod_ref(test_fn.id) {
            println!("\n  test() mod/ref summary:");
            println!("    may_mod locations: {}", summary.may_mod.len());
            println!("    may_ref locations: {}", summary.may_ref.len());
            println!("    (test() transitively includes modify()'s effects)");
        }
    }

    println!("\nMemory SSA analysis complete.");
    println!("  The call to modify(p) is recognized as a memory Def,");
    println!("  which clobbers the earlier store *p = source().");
    println!("  This means the load x = *p reads from modify(),");
    println!("  not from the tainted source() call.");
}
