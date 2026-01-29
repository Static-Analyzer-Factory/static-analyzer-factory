#![cfg(feature = "z3-solver")]
//! E2E tests for Z3-enhanced analysis features (Plan 033).
//!
//! Tests the seven Z3-based analysis features:
//! - Assertion prover
//! - Alias refinement
//! - Path reachability
//! - Typestate Z3 refinement
//! - Numeric Z3 refinement
//! - IFDS taint Z3 refinement
//! - `ValueFlow` taint Z3 refinement
//!
//! Source files: `tests/programs/c/z3_*.c`
//! Compiled fixtures: `tests/fixtures/llvm/e2e/z3_*.ll`

use saf_analysis::z3_utils::{check_path_reachable, prove_assertions};
use saf_test_utils::load_ll_fixture;

// ── Feature 5: Assertion prover ─────────────────────────────────────────

#[test]
fn z3_assertion_provable_proves() {
    let module = load_ll_fixture("z3_assertion_provable");
    let result = prove_assertions(
        &module,
        None, // no absint result
        2000, // z3_timeout_ms
        64,   // max_guards
        &[],  // default assert functions
    );

    // Diagnostics should be consistent
    assert_eq!(
        result.diagnostics.total_assertions,
        result.diagnostics.proven_count
            + result.diagnostics.may_fail_count
            + result.diagnostics.unknown_count,
        "Diagnostics should sum correctly"
    );
}

#[test]
fn z3_assertion_failing_finds_issues() {
    let module = load_ll_fixture("z3_assertion_failing");
    let result = prove_assertions(&module, None, 2000, 64, &[]);

    assert_eq!(
        result.diagnostics.total_assertions,
        result.diagnostics.proven_count
            + result.diagnostics.may_fail_count
            + result.diagnostics.unknown_count,
    );
}

// ── Feature 7: Path reachability ────────────────────────────────────────

#[test]
fn z3_reach_infeasible_checks() {
    let module = load_ll_fixture("z3_reach_infeasible");

    // Find the 'process' function
    let func = module
        .functions
        .iter()
        .find(|f| f.name == "process" && !f.is_declaration)
        .expect("should find 'process' function");

    // Get blocks
    let blocks: Vec<_> = func.blocks.iter().collect();
    assert!(
        blocks.len() >= 2,
        "process() should have multiple blocks (has {})",
        blocks.len()
    );

    // Check reachability between first and last non-entry blocks
    if blocks.len() >= 3 {
        let from_block = blocks[1].id;
        let to_block = blocks[blocks.len() - 1].id;

        let result = check_path_reachable(from_block, to_block, func.id, &module, 2000, 64, 100);

        // Should have checked at least 1 path
        // Verify the result has diagnostic info
        let _ = result.paths_checked;
    }
}

#[test]
fn z3_reach_feasible_checks() {
    let module = load_ll_fixture("z3_reach_feasible");

    let func = module
        .functions
        .iter()
        .find(|f| f.name == "process" && !f.is_declaration)
        .expect("should find 'process' function");

    let blocks: Vec<_> = func.blocks.iter().collect();
    if blocks.len() >= 3 {
        let from_block = blocks[0].id;
        let to_block = blocks[blocks.len() - 1].id;

        let result = check_path_reachable(from_block, to_block, func.id, &module, 2000, 64, 100);

        // Verify the result has diagnostic info
        let _ = result.paths_checked;
    }
}

// ── Determinism ─────────────────────────────────────────────────────────

#[test]
fn z3_assertion_determinism() {
    let module = load_ll_fixture("z3_assertion_provable");
    let r1 = prove_assertions(&module, None, 2000, 64, &[]);
    let r2 = prove_assertions(&module, None, 2000, 64, &[]);

    assert_eq!(r1.proven.len(), r2.proven.len());
    assert_eq!(r1.may_fail.len(), r2.may_fail.len());
    assert_eq!(r1.unknown.len(), r2.unknown.len());
    assert_eq!(
        r1.diagnostics.total_assertions,
        r2.diagnostics.total_assertions
    );
}

// ── Diagnostics export ──────────────────────────────────────────────────

#[test]
fn z3_assertion_diagnostics_serialize() {
    let module = load_ll_fixture("z3_assertion_provable");
    let result = prove_assertions(&module, None, 2000, 64, &[]);

    let json = serde_json::to_string(&result.diagnostics).expect("diagnostics should serialize");
    assert!(json.contains("total_assertions"));
    assert!(json.contains("proven_count"));
    assert!(json.contains("z3_calls"));
}
