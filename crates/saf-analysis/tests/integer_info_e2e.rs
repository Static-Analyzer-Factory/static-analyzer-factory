//! E2E tests for Plan 009: Integer overflow, info leak, and uninitialized patterns.
//!
//! Each test loads a compiled LLVM IR fixture, runs the full analysis pipeline,
//! and verifies graph construction and `taint_flow` queries produce correct results.
//!
//! Uses fast mode for value flow (see `taint_e2e.rs` module doc).
//!
//! NOTE: This is a static analysis test suite that validates SAF's
//! ability to DETECT vulnerability patterns, not exploit them.

use std::collections::BTreeSet;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::selector::Selector;
use saf_analysis::{QueryLimits, ValueFlowBuilder, ValueFlowConfig};
use saf_core::air::AirModule;
use saf_core::ids::ValueId;
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

// ── CWE-190: Integer overflow ───────────────────────────────────────────

#[test]
fn cwe190_integer_overflow_loads_and_builds() {
    let module = load_ll_fixture("integer_overflow");
    let _graph = build_fast_vf(&module);
    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
}

#[test]
fn cwe190_integer_overflow_param_to_malloc_flow() {
    let module = load_ll_fixture("integer_overflow");
    let graph = build_fast_vf(&module);

    // User-controlled size params flow through multiplication to malloc
    let sources = resolve(&Selector::function_param("alloc_array", None), &module);
    let sinks = resolve(&Selector::arg_to("malloc", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "size param should resolve");
    assert!(!sinks.is_empty(), "malloc arg should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find flow from size param through mul to malloc"
    );
}

// ── CWE-200: Information leak ───────────────────────────────────────────

#[test]
fn cwe200_info_leak_loads_and_builds() {
    let module = load_ll_fixture("info_leak");
    let graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");
    assert!(graph.edge_count() > 0, "valueflow graph should have edges");
}

// ── CWE-457: Uninitialized variable ─────────────────────────────────────

#[test]
fn cwe457_uninitialized_loads_and_builds() {
    let module = load_ll_fixture("uninitialized");
    let graph = build_fast_vf(&module);

    let main_fn = module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("main function should exist");
    assert!(!main_fn.blocks.is_empty(), "should have at least one block");
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");
}

// ── CWE-457: Uninitialized heap struct field ────────────────────────────

#[test]
fn cwe457_uninitialized_heap_loads_and_builds() {
    let module = load_ll_fixture("uninitialized_heap");
    let graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");
    assert!(graph.edge_count() > 0, "valueflow graph should have edges");
}
