// Detect CWE-78 command injection using the SAF Rust API.
//
// This tutorial shows how to use the SAF analysis pipeline
// to find taint flows from user input to dangerous sinks.
//
// The program compiles the vulnerable C source to LLVM IR,
// loads it through the LLVM frontend, and runs taint analysis.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_injection

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::selector::Selector;
use saf_analysis::{
    FieldSensitivity, PtaConfig, PtaContext, PtaResult, QueryLimits, ValueFlowBuilder,
    ValueFlowConfig,
};
use saf_core::config::Config;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    let tutorial_dir = PathBuf::from("tutorials/taint/01-command-injection");
    let source = tutorial_dir.join("vulnerable.c");
    let llvm_ir = tutorial_dir.join("vulnerable.ll");

    // Step 1: Compile C source to LLVM IR
    let status = Command::new("clang-18")
        .args(["-S", "-emit-llvm", "-O0", "-g",
               "-o", llvm_ir.to_str().unwrap(), source.to_str().unwrap()])
        .status()
        .expect("failed to run clang-18");
    assert!(status.success(), "clang-18 compilation failed");

    // Step 2: Load via LLVM frontend
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let bundle = frontend.ingest(&[llvm_ir.as_path()], &config).expect("failed to load LLVM IR");
    let module = bundle.module;

    // Step 3: Build analysis graphs (DefUse, CallGraph, PTA)
    let defuse = DefUseGraph::build(&module);
    let callgraph = CallGraph::build(&module);
    let pta_config = PtaConfig {
        enabled: true,
        field_sensitivity: FieldSensitivity::StructFields { max_depth: 2 },
        max_objects: 100_000,
        max_iterations: 1_000_000,
    };
    let mut pta_ctx = PtaContext::new(pta_config);
    let pta_analysis = pta_ctx.analyze(&module);
    let pta_result = PtaResult::new(
        pta_analysis.pts,
        Arc::new(pta_analysis.factory),
        pta_analysis.diagnostics,
    );

    // Step 4: Build the ValueFlow graph
    let vf_config = ValueFlowConfig::default();
    let builder = ValueFlowBuilder::new(&vf_config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Step 5: Define sources and sinks using selectors
    let source_selector = Selector::function_param("main", Some(1)); // argv
    let sink_selector = Selector::arg_to("system", Some(0)); // system(cmd)

    let source_ids = source_selector.resolve(&module).expect("source resolve failed");
    let sink_ids = sink_selector.resolve(&module).expect("sink resolve failed");
    let limits = QueryLimits::default();

    println!("Sources resolved: {}", source_ids.len());
    println!("Sinks resolved: {}", sink_ids.len());

    // Step 6: Run the taint flow query
    let flows = graph.taint_flow(&source_ids, &sink_ids, &BTreeSet::new(), &limits);
    println!("Taint flows found: {}", flows.len());

    for (i, flow) in flows.iter().enumerate() {
        println!("  [{}] source={:?} sink={:?} trace_hops={}", i, flow.source, flow.sink, flow.trace.len());
    }
}
