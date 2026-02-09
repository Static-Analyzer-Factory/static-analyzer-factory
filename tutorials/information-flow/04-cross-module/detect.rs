// Detect CWE-78 cross-module command injection using the SAF Rust API.
//
// Demonstrates taint flow across translation unit boundaries: the source
// (getenv) is in module_a.c and the sink (system) is in module_b.c.
// Uses llvm-link to combine the two modules before loading.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_cross_module

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
    let tutorial_dir = PathBuf::from("tutorials/taint/06-cross-module-taint");
    let module_a_src = tutorial_dir.join("module_a.c");
    let module_b_src = tutorial_dir.join("module_b.c");
    let module_a_ll = tutorial_dir.join("module_a.ll");
    let module_b_ll = tutorial_dir.join("module_b.ll");
    let combined_ll = tutorial_dir.join("combined.ll");

    // Step 1: Compile both modules to LLVM IR
    for (src, ll) in [(&module_a_src, &module_a_ll), (&module_b_src, &module_b_ll)] {
        let status = Command::new("clang-18")
            .args(["-S", "-emit-llvm", "-O0", "-g",
                   "-o", ll.to_str().unwrap(), src.to_str().unwrap()])
            .status()
            .expect("failed to run clang-18");
        assert!(status.success(), "clang-18 compilation failed");
    }

    // Step 2: Link modules
    let status = Command::new("llvm-link-18")
        .args(["-S", "-o", combined_ll.to_str().unwrap(),
               module_a_ll.to_str().unwrap(), module_b_ll.to_str().unwrap()])
        .status()
        .expect("failed to run llvm-link-18");
    assert!(status.success(), "llvm-link-18 failed");

    // Step 3: Load via LLVM frontend
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let bundle = frontend.ingest(&[combined_ll.as_path()], &config)
        .expect("failed to load LLVM IR");
    let module = bundle.module;

    // Step 4: Build analysis graphs
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

    let vf_config = ValueFlowConfig::default();
    let builder = ValueFlowBuilder::new(&vf_config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Step 5: Query for cross-module taint flow
    let source_ids = Selector::call_to("getenv")
        .resolve(&module)
        .expect("source resolve failed");
    let sink_ids = Selector::arg_to("system", Some(0))
        .resolve(&module)
        .expect("sink resolve failed");
    let limits = QueryLimits::default();

    println!("Sources resolved: {}", source_ids.len());
    println!("Sinks resolved: {}", sink_ids.len());

    let flows = graph.taint_flow(&source_ids, &sink_ids, &BTreeSet::new(), &limits);
    println!("Cross-module taint flows: {}", flows.len());
    for (i, flow) in flows.iter().enumerate() {
        println!("  [{}] source={:?} sink={:?} trace_hops={}", i, flow.source, flow.sink, flow.trace.len());
    }
}
