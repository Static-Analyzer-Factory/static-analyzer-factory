//! E2E tests for IFDS taint analysis.
//!
//! Each test loads a compiled LLVM IR fixture, runs the full IFDS pipeline
//! (LLVM frontend → AIR → ICFG → IFDS tabulation), and verifies results.
//!
//! Source files are in `tests/fixtures/sources/ifds_*` (C, C++, Rust).
//! Compiled fixtures are in `tests/fixtures/llvm/e2e/ifds_*.ll`.

use saf_analysis::callgraph::CallGraph;
use saf_analysis::icfg::Icfg;
use saf_analysis::ifds::{IfdsConfig, TaintFact, TaintIfdsProblem, solve_ifds};
use saf_analysis::selector::Selector;
use saf_core::air::AirModule;
use saf_test_utils::load_ll_fixture;

fn run_ifds_taint(
    module: &AirModule,
    sources: &[Selector],
    sanitizers: &[Selector],
) -> saf_analysis::ifds::IfdsResult<TaintFact> {
    let callgraph = CallGraph::build(module);
    let icfg = Icfg::build(module, &callgraph);
    let problem = TaintIfdsProblem::new(module, sources, sanitizers);
    let config = IfdsConfig::default();
    solve_ifds(&problem, &icfg, &callgraph, &config)
}

// ── Simple intraprocedural: getenv → system ──────────────────────────────

#[test]
fn ifds_simple_taint_reaches_sink() {
    let module = load_ll_fixture("ifds_simple_taint");
    let sources = vec![Selector::call_to("getenv")];
    let sinks = [Selector::arg_to("system", Some(0))];

    let result = run_ifds_taint(&module, &sources, &[]);

    // Verify taint was found
    assert!(
        !result.facts.is_empty(),
        "IFDS should produce facts for simple taint"
    );

    // Check that a tainted value reaches the sink instruction.
    let sink_values = sinks[0].resolve(&module).expect("sink should resolve");
    assert!(!sink_values.is_empty(), "system arg should resolve");

    let has_taint = has_taint_at_sink(&result, &module, &sink_values);
    assert!(has_taint, "taint from getenv should reach system()");
}

// ── Interprocedural: getenv → process() → system ────────────────────────

#[test]
fn ifds_interprocedural_taint_through_helper() {
    let module = load_ll_fixture("ifds_interprocedural_taint");
    let sources = vec![Selector::call_to("getenv")];
    let sinks = [Selector::arg_to("system", Some(0))];

    let result = run_ifds_taint(&module, &sources, &[]);

    assert!(
        !result.facts.is_empty(),
        "IFDS should produce facts for interprocedural taint"
    );

    // Verify summary edges were created for the process() function.
    assert!(
        !result.summaries.is_empty(),
        "should have summary edges for process()"
    );

    let sink_values = sinks[0].resolve(&module).expect("sink should resolve");
    let has_taint = has_taint_at_sink(&result, &module, &sink_values);
    assert!(
        has_taint,
        "taint from getenv should flow through process() to system()"
    );
}

// ── Sanitized: getenv called but safe buffer passed to system ────────────

#[test]
fn ifds_sanitized_taint_does_not_reach_sink() {
    let module = load_ll_fixture("ifds_sanitized_taint");
    let sources = vec![Selector::call_to("getenv")];
    let sinks = [Selector::arg_to("system", Some(0))];

    let result = run_ifds_taint(&module, &sources, &[]);

    let sink_values = sinks[0].resolve(&module).expect("sink should resolve");
    let has_taint = has_taint_at_sink(&result, &module, &sink_values);
    assert!(
        !has_taint,
        "taint from getenv should NOT reach system() when sanitized"
    );
}

// ── Multi-hop: getenv → step_one → step_two → step_three → system ──────

#[test]
fn ifds_multi_hop_taint_through_three_level_call_chain() {
    let module = load_ll_fixture("ifds_multi_hop_taint");
    let sources = vec![Selector::call_to("getenv")];
    let sinks = [Selector::arg_to("system", Some(0))];

    let result = run_ifds_taint(&module, &sources, &[]);

    assert!(
        !result.facts.is_empty(),
        "IFDS should produce facts for multi-hop taint"
    );

    // Should have summary edges for step_one, step_two, step_three.
    assert!(
        !result.summaries.is_empty(),
        "should have summary edges for the 3-level call chain"
    );

    let sink_values = sinks[0].resolve(&module).expect("sink should resolve");
    let has_taint = has_taint_at_sink(&result, &module, &sink_values);
    assert!(
        has_taint,
        "taint from getenv should flow through step_one → step_two → step_three → system()"
    );
}

// ── C++ class: getenv → CommandWrapper(ctor) → get() → system ──────────

#[test]
fn ifds_cpp_class_taint_through_constructor_and_getter() {
    let module = load_ll_fixture("ifds_cpp_class_taint");
    let sources = vec![Selector::call_to("getenv")];
    let sinks = [Selector::arg_to("system", Some(0))];

    let result = run_ifds_taint(&module, &sources, &[]);

    assert!(
        !result.facts.is_empty(),
        "IFDS should produce facts for C++ class taint"
    );

    let sink_values = sinks[0].resolve(&module).expect("sink should resolve");
    let has_taint = has_taint_at_sink(&result, &module, &sink_values);
    assert!(
        has_taint,
        "taint from getenv should flow through CommandWrapper ctor/get to system()"
    );
}

// ── Rust FFI: getenv → transform() → system ────────────────────────────

#[test]
fn ifds_rust_ffi_taint_through_mangled_function() {
    let module = load_ll_fixture("ifds_rust_ffi_taint");
    let sources = vec![Selector::call_to("getenv")];
    let sinks = [Selector::arg_to("system", Some(0))];

    let result = run_ifds_taint(&module, &sources, &[]);

    assert!(
        !result.facts.is_empty(),
        "IFDS should produce facts for Rust FFI taint"
    );

    let sink_values = sinks[0].resolve(&module).expect("sink should resolve");
    let has_taint = has_taint_at_sink(&result, &module, &sink_values);
    assert!(
        has_taint,
        "taint from getenv should flow through Rust transform() to system()"
    );
}

// ── Determinism ──────────────────────────────────────────────────────────

#[test]
fn ifds_results_are_deterministic() {
    let module = load_ll_fixture("ifds_simple_taint");
    let sources = vec![Selector::call_to("getenv")];

    let result1 = run_ifds_taint(&module, &sources, &[]);
    let result2 = run_ifds_taint(&module, &sources, &[]);

    // Compare exported JSON for byte-identical determinism.
    let export1 = serde_json::to_string(&result1.export()).expect("serialize");
    let export2 = serde_json::to_string(&result2.export()).expect("serialize");
    assert_eq!(export1, export2, "IFDS results should be deterministic");
}

// ── Helper: check if taint reaches sink operands ─────────────────────────

fn has_taint_at_sink(
    result: &saf_analysis::ifds::IfdsResult<TaintFact>,
    module: &AirModule,
    sink_values: &std::collections::BTreeSet<saf_core::ids::ValueId>,
) -> bool {
    // Check if any Tainted(v) fact exists where v is a sink value.
    for facts in result.facts.values() {
        for fact in facts {
            if let TaintFact::Tainted(v) = fact {
                if sink_values.contains(v) {
                    return true;
                }
            }
        }
    }

    // Also check if taint reaches any instruction that uses a sink value.
    for func in &module.functions {
        for block in &func.blocks {
            for inst in &block.instructions {
                let uses_sink = inst.operands.iter().any(|op| sink_values.contains(op));
                if uses_sink {
                    if let Some(facts) = result.facts_at(inst.id) {
                        for fact in facts {
                            if let TaintFact::Tainted(v) = fact {
                                if inst.operands.contains(v) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}
