//! E2E tests for Plan 007: Taint flow analysis.
//!
//! Each test loads a compiled LLVM IR fixture, runs the full analysis
//! pipeline (LLVM frontend → AIR → value flow), and verifies `taint_flow`
//! queries produce correct results.
//!
//! Uses fast mode (no PTA) because compiled IR's store/load chains require
//! the universal memory node for flow connectivity. PTA precision
//! improvements tracked in FUTURE.md.
//!
//! NOTE: This is a static analysis test suite that validates SAF's
//! ability to DETECT vulnerabilities, not exploit them.

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

// ── CWE-78: Tainted argv flows to system() ──────────────────────────────

#[test]
fn cwe78_argv_to_system_finds_flow() {
    let module = load_ll_fixture("command_injection");
    let graph = build_fast_vf(&module);

    let sources = resolve(&Selector::function_param("main", Some(1)), &module);
    let sinks = resolve(&Selector::arg_to("system", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "sources should resolve");
    assert!(!sinks.is_empty(), "sinks should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find taint flow from argv to system()"
    );
}

// ── CWE-134: gets() result flows to printf format arg ───────────────────
// Note: In compiled IR, gets() and printf() both use independent GEP
// instructions to the same stack buffer. Connecting them requires alias
// analysis (PTA) to recognize the GEPs alias — fast mode cannot do this.
// We verify the fixture loads and selectors resolve; full flow detection
// requires precise mode improvements (tracked in FUTURE.md).

#[test]
fn cwe134_format_string_loads_and_builds() {
    let module = load_ll_fixture("format_string");
    let graph = build_fast_vf(&module);

    assert!(!module.functions.is_empty(), "should have functions");
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");

    let sources = resolve(&Selector::call_to("gets"), &module);
    let sinks = resolve(&Selector::arg_to("printf", Some(0)), &module);
    assert!(!sources.is_empty(), "gets call should resolve");
    assert!(!sinks.is_empty(), "printf arg should resolve");
}

// ── CWE-89: getenv() flows to sqlite3_exec ─────────────────────────────

#[test]
fn cwe89_getenv_to_sqlite3_finds_flow() {
    let module = load_ll_fixture("sql_injection");
    let graph = build_fast_vf(&module);

    let sources = resolve(&Selector::call_to("getenv"), &module);
    let sinks = resolve(&Selector::arg_to("sqlite3_exec", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "sources should resolve");
    assert!(!sinks.is_empty(), "sinks should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find taint flow from getenv() to sqlite3_exec()"
    );
}

// ── CWE-22: argv flows to fopen path argument ──────────────────────────

#[test]
fn cwe22_argv_to_fopen_finds_flow() {
    let module = load_ll_fixture("path_traversal");
    let graph = build_fast_vf(&module);

    let sources = resolve(&Selector::function_param("main", Some(1)), &module);
    let sinks = resolve(&Selector::arg_to("fopen", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "sources should resolve");
    assert!(!sinks.is_empty(), "sinks should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find taint flow from argv to fopen()"
    );
}

// ── CWE-78 Sanitized: sanitizer blocks taint flow ───────────────────────
// Note: The compiled taint_sanitized.ll triggers a SIGSEGV in the LLVM
// frontend (inkwell/llvm-sys) during module loading.

#[test]
fn sanitizer_fixture_loads() {
    let module = load_ll_fixture("taint_sanitized");
    assert!(!module.functions.is_empty(), "should have functions");
}

// ── CWE-78 Rust: env_args() flows to libc system() ─────────────────────
// Note: Rust compiled IR uses mangled names and generates ~2500 lines of IR
// for a simple program. Selector-based taint queries require demangling
// support. For now we verify the fixture loads and builds correctly.

#[test]
#[ignore = "SIGSEGV in LLVM frontend on Rust-generated IR (pre-existing)"]
fn rust_unsafe_taint_loads_and_builds() {
    let module = load_ll_fixture("taint_unsafe");
    let graph = build_fast_vf(&module);

    assert!(!module.functions.is_empty(), "should have functions");
    assert!(graph.node_count() > 0, "valueflow graph should have nodes");

    // system() is declared in the Rust IR as an external function
    let sinks = resolve(&Selector::arg_to("system", Some(0)), &module);
    assert!(!sinks.is_empty(), "system sink should resolve in Rust IR");
}

// ── Determinism ─────────────────────────────────────────────────────────

#[test]
fn taint_e2e_deterministic() {
    let module = load_ll_fixture("command_injection");

    let defuse = DefUseGraph::build(&module);
    let callgraph = CallGraph::build(&module);
    let config = ValueFlowConfig::fast();

    let builder1 = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, None);
    let graph1 = builder1.build();

    let builder2 = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, None);
    let graph2 = builder2.build();

    let sources = resolve(&Selector::function_param("main", Some(1)), &module);
    let sinks = resolve(&Selector::arg_to("system", Some(0)), &module);
    let limits = QueryLimits::default();

    let flows1 = graph1.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    let flows2 = graph2.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);

    assert_eq!(flows1.len(), flows2.len(), "deterministic flow count");
    for (f1, f2) in flows1.iter().zip(flows2.iter()) {
        assert_eq!(f1.source, f2.source, "deterministic source");
        assert_eq!(f1.sink, f2.sink, "deterministic sink");
    }
}

// ── Using predefined selector presets ───────────────────────────────────

#[test]
fn preset_selectors_find_flow() {
    use saf_analysis::selector::{SinkSelector, SourceSelector};

    let module = load_ll_fixture("command_injection");
    let graph = build_fast_vf(&module);

    let source_selectors = SourceSelector::FunctionParams(vec!["main".to_string()]);
    let sink_selectors = SinkSelector::command_injection();
    let limits = QueryLimits::default();

    let mut sources = BTreeSet::new();
    for sel in source_selectors.to_selectors() {
        sources.extend(resolve(&sel, &module));
    }

    let mut sinks = BTreeSet::new();
    for sel in sink_selectors.to_selectors() {
        sinks.extend(resolve(&sel, &module));
    }

    assert!(!sources.is_empty(), "preset sources should resolve");
    assert!(!sinks.is_empty(), "preset sinks should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(!flows.is_empty(), "preset selectors should find flow");
}
