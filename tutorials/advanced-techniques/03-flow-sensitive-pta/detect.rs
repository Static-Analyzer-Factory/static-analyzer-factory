// Tutorial 7: Flow-Sensitive PTA — comparing Andersen vs flow-sensitive.
//
// Demonstrates how flow-sensitive pointer analysis achieves higher precision
// than Andersen's flow-insensitive analysis through strong updates, using
// the Rust API.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_flow_sensitive_pta

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::fspta::{self, FsPtaConfig, FsSvfgBuilder};
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::SvfgBuilder;
use saf_analysis::{PtaConfig, PtaContext, PtaResult};
use saf_core::air::AirModule;
use saf_core::config::Config;
use saf_core::ids::FunctionId;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    let tutorial_dir = PathBuf::from("tutorials/pta/07-flow-sensitive-pta");
    let source = tutorial_dir.join("vulnerable.c");
    let llvm_ir = tutorial_dir.join("vulnerable.ll");

    // Step 1: Compile C source to LLVM IR
    let status = Command::new("clang-18")
        .args([
            "-S",
            "-emit-llvm",
            "-O0",
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

    // Step 3: Andersen (flow-insensitive) PTA
    let pta_config = PtaConfig::default();
    let mut ctx = PtaContext::new(pta_config.clone());
    let raw = ctx.analyze(&module);
    let andersen = PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics);

    println!("=== Andersen (flow-insensitive) PTA ===");
    let andersen_export = andersen.export();
    println!(
        "  Total values with points-to info: {}",
        andersen_export.points_to.len()
    );

    // Step 4: Build the full flow-sensitive pipeline
    let defuse = DefUseGraph::build(&module);
    let callgraph = CallGraph::build(&module);

    let cfgs: BTreeMap<FunctionId, Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect();

    // PTA for SVFG builder (shared ref)
    let mut ctx1 = PtaContext::new(pta_config.clone());
    let raw1 = ctx1.analyze(&module);
    let pta1 = PtaResult::new(raw1.pts, Arc::new(raw1.factory), raw1.diagnostics);

    // PTA consumed by MSSA for SVFG
    let mut ctx2 = PtaContext::new(pta_config.clone());
    let raw2 = ctx2.analyze(&module);
    let mssa_pta = PtaResult::new(raw2.pts, Arc::new(raw2.factory), raw2.diagnostics);

    let mut mssa = MemorySsa::build(&module, &cfgs, mssa_pta, &callgraph);
    let svfg = SvfgBuilder::new(&module, &defuse, &callgraph, &pta1, &mut mssa).build();

    // PTA for FsSvfg builder
    let mut ctx3 = PtaContext::new(pta_config.clone());
    let raw3 = ctx3.analyze(&module);
    let pta3 = PtaResult::new(raw3.pts, Arc::new(raw3.factory), raw3.diagnostics);

    // PTA consumed by MSSA for FsSvfg
    let mut ctx4 = PtaContext::new(pta_config);
    let raw4 = ctx4.analyze(&module);
    let mssa_pta2 = PtaResult::new(raw4.pts, Arc::new(raw4.factory), raw4.diagnostics);
    let mut mssa2 = MemorySsa::build(&module, &cfgs, mssa_pta2, &callgraph);

    let fs_svfg = FsSvfgBuilder::new(&module, &svfg, &pta3, &mut mssa2, &callgraph).build();

    // Step 5: Solve flow-sensitive
    let fs_config = FsPtaConfig::default();
    let result =
        fspta::solve_flow_sensitive(&module, &fs_svfg, &pta3, &callgraph, &fs_config);

    let diag = result.diagnostics();
    println!("\n=== Flow-Sensitive PTA ===");
    println!("  Solver iterations: {}", diag.iterations);
    println!("  Converged: {}", !diag.iteration_limit_hit);
    println!("  Strong updates: {}", diag.strong_updates);
    println!("  Weak updates: {}", diag.weak_updates);
    println!("  FsSvfg nodes: {}", diag.fs_svfg_nodes);
    println!("  FsSvfg edges: {}", diag.fs_svfg_edges);
    println!("  Store nodes: {}", diag.store_nodes);
    println!("  Load nodes: {}", diag.load_nodes);

    // Step 6: Export
    let export = result.export();
    println!("\n=== Export ===");
    println!("  Schema version: {}", export.schema_version);
    println!("  Points-to entries: {}", export.points_to.len());

    // Step 7: Summary
    println!("\n=== Summary ===");
    println!("  Andersen (flow-insensitive):");
    println!("    Merges all program points -- `conn` may point to");
    println!("    {{secret_conn, pub_conn}} everywhere.");
    println!("  Flow-sensitive:");
    println!("    Tracks per-program-point. After `conn = &pub_conn`,");
    println!("    the strong update kills secret_conn from the set.");
    println!("    At the load site, conn -> {{pub_conn}} only.");
    if diag.strong_updates > 0 {
        println!("    (Performed {} strong update(s))", diag.strong_updates);
    }
}
