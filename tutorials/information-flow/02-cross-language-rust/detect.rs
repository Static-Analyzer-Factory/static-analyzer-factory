// Detect CWE-78 in Rust unsafe code using the SAF Rust API.
//
// Demonstrates cross-language taint detection: getenv() return value
// flows through Rust unsafe FFI to libc system().
//
// The program compiles the vulnerable Rust source to LLVM IR,
// loads it through the LLVM frontend, and runs taint analysis.

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
    let tutorial_dir = PathBuf::from("tutorials/taint/05-unsafe-rust");
    let source = tutorial_dir.join("vulnerable.rs");
    let llvm_ir = tutorial_dir.join("vulnerable.ll");

    // Step 1: Compile Rust source to LLVM IR
    let status = Command::new("rustc")
        .args(["--emit=llvm-ir", "-o", llvm_ir.to_str().unwrap(), source.to_str().unwrap()])
        .status()
        .expect("failed to run rustc");
    assert!(status.success(), "rustc compilation failed");

    // Step 2: Load via LLVM frontend
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let bundle = frontend.ingest(&[llvm_ir.as_path()], &config).expect("failed to load LLVM IR");
    let module = bundle.module;

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

    // SOURCE: getenv() return — user-controlled environment variable
    let sources = Selector::call_to("getenv")
        .resolve(&module)
        .expect("source resolve failed");
    // SINK: system() arg 0 — libc command execution
    let sinks = Selector::arg_to("system", Some(0))
        .resolve(&module)
        .expect("sink resolve failed");
    let limits = QueryLimits::default();

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    println!("Cross-language taint flows: {}", flows.len());
    for (i, flow) in flows.iter().enumerate() {
        println!("  [{}] source={:?} sink={:?}", i, flow.source, flow.sink);
    }
}
