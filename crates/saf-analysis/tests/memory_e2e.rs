//! E2E tests for Plan 008: Memory safety patterns.
//!
//! Each test loads a compiled LLVM IR fixture modeling a CWE memory-safety
//! pattern, runs the full analysis pipeline, and verifies graph construction
//! and `taint_flow` queries produce correct results.
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

// ── CWE-416: Use after free ─────────────────────────────────────────────

#[test]
fn cwe416_use_after_free_malloc_to_free_flow() {
    let module = load_ll_fixture("use_after_free");
    let graph = build_fast_vf(&module);

    let sources = resolve(&Selector::call_to("malloc"), &module);
    let sinks = resolve(&Selector::arg_to("free", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "malloc source should resolve");
    assert!(!sinks.is_empty(), "free sink should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find flow from malloc result to free arg"
    );
}

// ── CWE-415: Double free ────────────────────────────────────────────────

#[test]
fn cwe415_double_free_malloc_to_free_flow() {
    let module = load_ll_fixture("double_free");
    let graph = build_fast_vf(&module);

    let sources = resolve(&Selector::call_to("malloc"), &module);
    let sinks = resolve(&Selector::arg_to("free", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "malloc source should resolve");
    assert!(!sinks.is_empty(), "free sink should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find flow from malloc to free (double free pattern)"
    );
}

// ── CWE-476: Null pointer dereference ───────────────────────────────────

#[test]
fn cwe476_null_deref_loads_and_builds() {
    let module = load_ll_fixture("null_deref");
    let graph = build_fast_vf(&module);

    assert!(!module.functions.is_empty(), "should have functions");
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");
}

// ── CWE-120: Buffer overflow ────────────────────────────────────────────

#[test]
fn cwe120_buffer_overflow_malloc_to_free_flow() {
    let module = load_ll_fixture("buffer_overflow");
    let graph = build_fast_vf(&module);

    // Real compiled code uses loop writes (buf[i] = i), not memcpy.
    // Verify malloc → free flow exists (data flows through alloc lifecycle).
    let sources = resolve(&Selector::call_to("malloc"), &module);
    let sinks = resolve(&Selector::arg_to("free", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "malloc source should resolve");
    assert!(!sinks.is_empty(), "free sink should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find flow from malloc to free (buffer lifecycle)"
    );
}

// ── Dangling pointer (C++) ──────────────────────────────────────────────

#[test]
fn dangling_ptr_cpp_loads_and_builds() {
    let module = load_ll_fixture("dangling_ptr_cpp");
    let _graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
}

// ── Dangling pointer (Rust) ─────────────────────────────────────────────

#[test]
#[ignore = "SIGSEGV in LLVM frontend on Rust-generated IR (pre-existing)"]
fn dangling_ptr_rs_loads_and_builds() {
    let module = load_ll_fixture("dangling_ptr_rs");
    let _graph = build_fast_vf(&module);

    assert!(
        !module.functions.is_empty(),
        "should have at least one function"
    );
}
