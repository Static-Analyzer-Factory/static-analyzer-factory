// Export all SAF graphs to JSON using the Rust API.
//
// Demonstrates building and exporting each graph type: CFG, CallGraph,
// DefUse, and ValueFlow.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_json_export

use std::path::PathBuf;
use std::process::Command;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::{ValueFlowBuilder, ValueFlowConfig};
use saf_core::config::Config;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    let tutorial_dir = PathBuf::from("tutorials/integration/02-json-export-pipeline");
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

    // Build all graphs
    let cfg = Cfg::build(&module);
    let callgraph = CallGraph::build(&module);
    let defuse = DefUseGraph::build(&module);

    let vf_config = ValueFlowConfig::default();
    let builder = ValueFlowBuilder::new(&vf_config, &module, &defuse, &callgraph, None);
    let vf = builder.build();

    // Export and report
    let cfg_export = cfg.export();
    println!("CFG: {} nodes, {} edges", cfg_export.nodes.len(), cfg_export.edges.len());

    let cg_export = callgraph.export();
    println!("CallGraph: {} nodes, {} edges", cg_export.nodes.len(), cg_export.edges.len());

    let du_export = defuse.export();
    println!("DefUse: {} nodes, {} edges", du_export.nodes.len(), du_export.edges.len());

    let vf_export = vf.export();
    println!("ValueFlow: {} nodes, {} edges", vf_export.nodes.len(), vf_export.edges.len());
}
