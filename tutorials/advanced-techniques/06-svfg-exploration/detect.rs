// Build and inspect a Sparse Value-Flow Graph via the SAF Rust API.
//
// Demonstrates SVFG construction, diagnostics, export, and reachability
// queries using the Rust API directly.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_svfg

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::SvfgBuilder;
use saf_analysis::{PtaConfig, PtaContext, PtaResult};
use saf_core::config::Config;
use saf_core::ids::FunctionId;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    let tutorial_dir = PathBuf::from("tutorials/graphs/06-svfg");
    let source = tutorial_dir.join("vulnerable.c");
    let llvm_ir = tutorial_dir.join("program.ll");

    // Step 1: Compile C to LLVM IR
    let status = Command::new("clang-18")
        .args([
            "-S", "-emit-llvm", "-O0", "-g0", "-fno-discard-value-names",
            "-o", llvm_ir.to_str().unwrap(), source.to_str().unwrap(),
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

    // Step 3: Build analysis prerequisites
    let defuse = DefUseGraph::build(&module);
    let callgraph = CallGraph::build(&module);

    // PTA for SVFG queries
    let pta_config = PtaConfig::default();
    let mut pta_ctx = PtaContext::new(pta_config.clone());
    let pta_raw = pta_ctx.analyze(&module);
    let pta_result = PtaResult::new(pta_raw.pts, Arc::new(pta_raw.factory), pta_raw.diagnostics);

    // PTA for MSSA (separate instance)
    let mut pta_ctx2 = PtaContext::new(pta_config);
    let pta_raw2 = pta_ctx2.analyze(&module);
    let mssa_pta = PtaResult::new(pta_raw2.pts, Arc::new(pta_raw2.factory), pta_raw2.diagnostics);

    let cfgs: BTreeMap<FunctionId, Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect();
    let mut mssa = MemorySsa::build(&module, &cfgs, mssa_pta, &callgraph);

    // Step 4: Build the SVFG
    let svfg = SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

    println!("SVFG Construction Results:");
    println!("  Nodes: {}", svfg.node_count());
    println!("  Edges: {}", svfg.edge_count());

    // Step 5: Inspect diagnostics
    let diag = svfg.diagnostics();
    println!("\n  Direct edges:   {}", diag.direct_edge_count);
    println!("  Indirect edges: {}", diag.indirect_edge_count);
    println!("  MemPhi nodes:   {}", diag.mem_phi_count);

    // Step 6: Export
    let export = svfg.export();
    println!("\n  Export schema: {}", export.schema_version);
    println!("  Exported nodes: {}", export.nodes.len());
    println!("  Exported edges: {}", export.edges.len());

    // Edge kind breakdown
    let mut kind_counts: BTreeMap<String, usize> = BTreeMap::new();
    for edge in &export.edges {
        *kind_counts.entry(edge.kind.name().to_string()).or_default() += 1;
    }
    println!("\n  Edge breakdown:");
    for (kind, count) in &kind_counts {
        println!("    {kind}: {count}");
    }
}
