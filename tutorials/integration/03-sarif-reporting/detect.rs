// Generate analysis output using the SAF Rust API.
//
// Note: SARIF envelope construction is shown in detect.py (Python is
// more natural for JSON manipulation). The Rust version demonstrates
// the analysis pipeline and finding export.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_sarif_reporting

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
    let tutorial_dir = PathBuf::from("tutorials/integration/03-sarif-reporting");
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

    let source_ids = Selector::function_param("main", Some(1))
        .resolve(&module)
        .expect("source resolve failed");
    let sink_ids = Selector::arg_to("system", Some(0))
        .resolve(&module)
        .expect("sink resolve failed");
    let limits = QueryLimits::default();

    let flows = graph.taint_flow(&source_ids, &sink_ids, &BTreeSet::new(), &limits);
    println!("Findings for SARIF export: {}", flows.len());

    for (i, flow) in flows.iter().enumerate() {
        println!("  [{}] source={:?} sink={:?}", i, flow.source, flow.sink);
        println!("       trace: {} hops", flow.trace.len());
    }
}
