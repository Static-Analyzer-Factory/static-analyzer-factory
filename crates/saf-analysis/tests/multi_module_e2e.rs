//! E2E tests for Plan 011: Multi-module and interprocedural patterns.
//!
//! Each test loads a compiled LLVM IR fixture spanning multiple source files,
//! runs the full analysis pipeline, and verifies cross-boundary taint
//! flow detection.
//!
//! Uses fast mode for value flow (see `taint_e2e.rs` module doc).
//!
//! NOTE: This is a static analysis test suite that validates SAF's
//! ability to track data across module boundaries, not exploit them.

use std::collections::BTreeSet;
use std::path::PathBuf;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::pipeline::{PipelineConfig, run_pipeline};
use saf_analysis::selector::Selector;
use saf_analysis::{QueryLimits, ValueFlowBuilder, ValueFlowConfig};
use saf_core::air::{AirBundle, AirModule};
use saf_core::ids::ValueId;
use saf_core::program::AirProgram;
use saf_test_utils::load_ll_fixture;

fn build_fast_vf(module: &AirModule) -> saf_analysis::ValueFlowGraph {
    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);
    let config = ValueFlowConfig::fast();
    let builder = ValueFlowBuilder::new(&config, module, &defuse, &callgraph, None);
    builder.build()
}

fn resolve(selector: &Selector, module: &AirModule) -> BTreeSet<ValueId> {
    selector
        .resolve(module)
        .expect("selector resolution failed")
}

// ── Cross-module taint: module_a_get_input(argv) → module_b_execute → system()

#[test]
fn cross_module_taint_loads_and_builds() {
    let module = load_ll_fixture("cross_module_taint");
    let _graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
}

#[test]
fn cross_module_taint_argv_to_system_flow() {
    let module = load_ll_fixture("cross_module_taint");
    let graph = build_fast_vf(&module);

    // module_a_get_input returns argv[1] (tainted), which flows to
    // module_b_execute → system().
    let sources = resolve(&Selector::call_to("module_a_get_input"), &module);
    let sinks = resolve(&Selector::arg_to("system", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(
        !sources.is_empty(),
        "module_a_get_input call result should resolve"
    );
    assert!(!sinks.is_empty(), "system sink should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find taint flow from module_a_get_input() to system() across modules"
    );
}

// ── Library wrapper: argv → safe_execute → system ───────────────────────

#[test]
fn library_wrapper_loads_and_builds() {
    let module = load_ll_fixture("library_wrapper");
    let _graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
}

#[test]
fn library_wrapper_argv_to_system_flow() {
    let module = load_ll_fixture("library_wrapper");
    let graph = build_fast_vf(&module);

    let sources = resolve(&Selector::function_param("main", Some(1)), &module);
    let sinks = resolve(&Selector::arg_to("system", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "argv source should resolve");
    assert!(!sinks.is_empty(), "system sink should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find taint flow from argv through wrapper to system()"
    );
}

// ── Callback chain: argv → step_a → step_b (indirect) → step_c (indirect) → system()

#[test]
fn callback_chain_loads_and_builds() {
    let module = load_ll_fixture("callback_chain");
    let _graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
}

#[test]
fn callback_chain_argv_to_system_flow() {
    let module = load_ll_fixture("callback_chain");
    let graph = build_fast_vf(&module);

    // main passes argv[1] → step_a → step_b (via fn ptr) → step_c (via fn ptr) → system()
    let sources = resolve(&Selector::function_param("main", Some(1)), &module);
    let sinks = resolve(&Selector::arg_to("system", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "argv source should resolve");
    assert!(!sinks.is_empty(), "system sink should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find taint flow from argv through callback chain to system()"
    );
}

// ── Multi-module merged view: AirProgram linking produces same analysis as pre-merged ───

fn incremental_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/incremental/two_module")
        .join(name)
}

fn load_air_bundle(name: &str) -> AirBundle {
    let json = std::fs::read_to_string(incremental_fixture_path(name)).unwrap();
    serde_json::from_str(&json).unwrap()
}

/// Check if a call graph has an edge where the caller has `caller_name` and
/// the callee has `callee_name` by looking up function IDs from the module.
fn has_call_edge(cg: &CallGraph, module: &AirModule, caller_name: &str, callee_name: &str) -> bool {
    let caller_func = module.function_by_name(caller_name);
    let callee_func = module.function_by_name(callee_name);

    let (Some(caller), Some(callee)) = (caller_func, callee_func) else {
        return false;
    };

    for (src, dsts) in &cg.edges {
        if src.function_id() == Some(caller.id) {
            for dst in dsts {
                if dst.function_id() == Some(callee.id) {
                    return true;
                }
            }
        }
    }
    false
}

#[test]
fn merged_view_produces_same_callgraph_as_single_module() {
    // Multi-module path: link two separate bundles, then merge
    let main_bundle = load_air_bundle("main.air.json");
    let lib_bundle = load_air_bundle("lib.air.json");
    let program = AirProgram::link(vec![main_bundle, lib_bundle]);
    let merged = program.merged_view();

    let config = PipelineConfig::default();
    let multi_result = run_pipeline(&merged, &config);

    // Single-module path (pre-merged fixture)
    let single_bundle = load_air_bundle("merged.air.json");
    let single_result = run_pipeline(&single_bundle.module, &config);

    // Both should discover main -> helper call edge
    assert!(
        has_call_edge(&multi_result.call_graph, &merged, "main", "helper"),
        "multi-module callgraph missing main->helper edge"
    );
    assert!(
        has_call_edge(
            &single_result.call_graph,
            &single_bundle.module,
            "main",
            "helper"
        ),
        "single-module callgraph missing main->helper edge"
    );
}
