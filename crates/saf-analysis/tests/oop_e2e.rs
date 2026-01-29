//! E2E tests for Plan 010: OOP patterns (vtable, callback, RAII, trait dispatch).
//!
//! Each test loads a compiled LLVM IR fixture modeling an OOP design pattern,
//! runs the full analysis pipeline, and verifies graph construction
//! and structural properties.
//!
//! Uses fast mode for value flow (see `taint_e2e.rs` module doc).
//!
//! NOTE: This is a static analysis test suite that validates SAF's
//! ability to model OOP patterns, not exploit them.

use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::{ValueFlowBuilder, ValueFlowConfig};
use saf_core::air::AirModule;
use saf_test_utils::load_ll_fixture;

fn build_fast_vf(module: &AirModule) -> saf_analysis::ValueFlowGraph {
    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);
    let config = ValueFlowConfig::fast();
    let builder = ValueFlowBuilder::new(&config, module, &defuse, &callgraph, None);
    builder.build()
}

// ── C++ vtable dispatch ─────────────────────────────────────────────────

#[test]
fn vtable_dispatch_loads_and_builds() {
    let module = load_ll_fixture("vtable_dispatch");
    let graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");
}

// ── C function pointer callback ─────────────────────────────────────────

#[test]
fn callback_fn_ptr_loads_and_builds() {
    let module = load_ll_fixture("callback_fn_ptr");
    let graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");
    assert!(graph.edge_count() > 0, "valueflow graph should have edges");
}

// ── C++ RAII resource management ────────────────────────────────────────

#[test]
fn raii_resource_loads_and_builds() {
    let module = load_ll_fixture("raii_resource");
    let graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");
}

// ── Rust trait dispatch ─────────────────────────────────────────────────

#[test]
#[ignore = "SIGSEGV in LLVM frontend on Rust-generated IR (pre-existing)"]
fn trait_dispatch_loads_and_builds() {
    let module = load_ll_fixture("trait_dispatch");
    let graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");
}
