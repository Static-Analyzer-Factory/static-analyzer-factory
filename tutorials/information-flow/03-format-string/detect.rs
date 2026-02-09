// Detect CWE-134 format string vulnerability using the SAF Rust API.
//
// Shows how to use call-return selectors: the return value of gets()
// is tainted, and it flows to printf()'s format string argument.
//
// The program compiles the vulnerable C source to LLVM IR,
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
    let tutorial_dir = PathBuf::from("tutorials/taint/02-format-string");
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

    // SOURCE: return value of gets() calls
    let sources = Selector::call_to("gets")
        .resolve(&module)
        .expect("source resolve failed");
    // SINK: argument 0 of printf() — the format string
    let sinks = Selector::arg_to("printf", Some(0))
        .resolve(&module)
        .expect("sink resolve failed");
    let limits = QueryLimits::default();

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    println!("Format string taint flows found: {}", flows.len());
}
