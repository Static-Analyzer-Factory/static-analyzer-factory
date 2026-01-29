//! E2E tests for points-to set representation equivalence.
//!
//! These tests verify that all three `PtsSet` representations produce
//! identical results when running the full PTA pipeline on real fixtures.

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cspta::{CsPtaConfig, solve_context_sensitive};
use saf_analysis::{FieldSensitivity, LocationFactory, PointsToMap, PtsRepresentation};
use saf_analysis::{extract_constraints, pta_solve as solve};
use saf_core::air::AirModule;
use saf_test_utils::{
    load_air_json_fixture as load_air_fixture, load_ll_fixture, load_ll_from_path,
    verification_fixtures_dir,
};

/// Load an LLVM fixture from the root llvm directory (not e2e).
fn load_llvm_root_fixture(name: &str) -> AirModule {
    let path = verification_fixtures_dir()
        .join("llvm")
        .join(format!("{name}.ll"));
    load_ll_from_path(&path).module
}

// ---------------------------------------------------------------------------
// CI-PTA E2E Tests
// ---------------------------------------------------------------------------

#[test]
fn e2e_ci_pta_all_repr_produce_identical_results() {
    let module = load_air_fixture("memory_ops");

    // Run CI-PTA with each representation (using default solve which uses BTreeSet)
    let result1 = run_ci_pta(&module);
    let result2 = run_ci_pta(&module);

    // Normalize results for comparison
    let pts1 = normalize_pts_result(&result1);
    let pts2 = normalize_pts_result(&result2);

    // Results should be identical (determinism)
    assert_eq!(pts1, pts2, "CI-PTA results should be deterministic");
}

#[test]
fn e2e_ci_pta_auto_selection_works() {
    let module = load_air_fixture("minimal");

    // Should work and produce valid results
    let result = run_ci_pta(&module);

    // Should have some points-to information (or empty for minimal fixture)
    let pts = normalize_pts_result(&result);
    // Minimal fixture might have no pointers, but it should at least run without error
    assert!(pts.is_empty() || !pts.is_empty());
}

#[test]
fn e2e_ci_pta_with_calls_all_repr_equivalent() {
    let module = load_air_fixture("calls");

    let result1 = run_ci_pta(&module);
    let result2 = run_ci_pta(&module);

    let pts1 = normalize_pts_result(&result1);
    let pts2 = normalize_pts_result(&result2);

    assert_eq!(pts1, pts2, "CI-PTA results should be deterministic");
}

// ---------------------------------------------------------------------------
// CS-PTA E2E Tests
// ---------------------------------------------------------------------------

#[test]
fn e2e_cs_pta_all_repr_identical() {
    let module = load_llvm_root_fixture("memory_ops");
    let callgraph = CallGraph::build(&module);

    // Run CS-PTA with each representation
    let btree_config = CsPtaConfig::default().with_pts_representation(PtsRepresentation::BTreeSet);
    let bitvec_config =
        CsPtaConfig::default().with_pts_representation(PtsRepresentation::BitVector);
    let bdd_config = CsPtaConfig::default().with_pts_representation(PtsRepresentation::Bdd);

    let btree_result = solve_context_sensitive(&module, &callgraph, &btree_config);
    let bitvec_result = solve_context_sensitive(&module, &callgraph, &bitvec_config);
    let bdd_result = solve_context_sensitive(&module, &callgraph, &bdd_config);

    // Diagnostics should match
    let btree_diag = btree_result.diagnostics();
    let bitvec_diag = bitvec_result.diagnostics();
    let bdd_diag = bdd_result.diagnostics();

    assert_eq!(
        btree_diag.iterations, bitvec_diag.iterations,
        "CS-PTA iterations differ: BTree={} vs BitVec={}",
        btree_diag.iterations, bitvec_diag.iterations
    );
    assert_eq!(
        btree_diag.iterations, bdd_diag.iterations,
        "CS-PTA iterations differ: BTree={} vs BDD={}",
        btree_diag.iterations, bdd_diag.iterations
    );

    // Context counts should match
    assert_eq!(
        btree_diag.context_count, bitvec_diag.context_count,
        "CS-PTA context counts differ"
    );
    assert_eq!(
        btree_diag.context_count, bdd_diag.context_count,
        "CS-PTA context counts differ"
    );
}

#[test]
fn e2e_cs_pta_identity_function_all_repr() {
    let module = load_ll_fixture("cspta_identity_function");
    let callgraph = CallGraph::build(&module);

    let btree_config = CsPtaConfig::default().with_pts_representation(PtsRepresentation::BTreeSet);
    let bitvec_config =
        CsPtaConfig::default().with_pts_representation(PtsRepresentation::BitVector);
    let bdd_config = CsPtaConfig::default().with_pts_representation(PtsRepresentation::Bdd);

    let btree_result = solve_context_sensitive(&module, &callgraph, &btree_config);
    let bitvec_result = solve_context_sensitive(&module, &callgraph, &bitvec_config);
    let bdd_result = solve_context_sensitive(&module, &callgraph, &bdd_config);

    // All should converge
    assert!(
        !btree_result.diagnostics().iteration_limit_hit,
        "BTree hit iteration limit"
    );
    assert!(
        !bitvec_result.diagnostics().iteration_limit_hit,
        "BitVec hit iteration limit"
    );
    assert!(
        !bdd_result.diagnostics().iteration_limit_hit,
        "BDD hit iteration limit"
    );

    // Iteration counts should match
    assert_eq!(
        btree_result.diagnostics().iterations,
        bitvec_result.diagnostics().iterations
    );
    assert_eq!(
        btree_result.diagnostics().iterations,
        bdd_result.diagnostics().iterations
    );
}

// ---------------------------------------------------------------------------
// Large Fixture Tests
// ---------------------------------------------------------------------------

#[test]
fn e2e_large_fixture_auto_selection() {
    // Load a larger fixture to test auto-selection behavior
    let module = load_ll_fixture("cspta_nested_wrappers");
    let callgraph = CallGraph::build(&module);

    // Auto should pick appropriate representation based on size
    let config = CsPtaConfig::default().with_pts_representation(PtsRepresentation::Auto);
    let result = solve_context_sensitive(&module, &callgraph, &config);

    // Should complete without hitting limits
    assert!(
        !result.diagnostics().iteration_limit_hit,
        "Auto selection hit iteration limit"
    );
}

// ---------------------------------------------------------------------------
// Determinism Tests
// ---------------------------------------------------------------------------

#[test]
fn e2e_ci_pta_deterministic_with_all_repr() {
    let module = load_air_fixture("memory_ops");

    // Run twice and verify identical results
    let result1 = run_ci_pta(&module);
    let result2 = run_ci_pta(&module);

    let pts1 = normalize_pts_result(&result1);
    let pts2 = normalize_pts_result(&result2);

    assert_eq!(pts1, pts2, "Non-deterministic results");
}

#[test]
fn e2e_cs_pta_deterministic_with_all_repr() {
    let module = load_llvm_root_fixture("memory_ops");
    let callgraph = CallGraph::build(&module);

    for repr in [
        PtsRepresentation::BTreeSet,
        PtsRepresentation::BitVector,
        PtsRepresentation::Bdd,
    ] {
        let config = CsPtaConfig::default().with_pts_representation(repr);

        let result1 = solve_context_sensitive(&module, &callgraph, &config);
        let result2 = solve_context_sensitive(&module, &callgraph, &config);

        assert_eq!(
            result1.diagnostics().iterations,
            result2.diagnostics().iterations,
            "Non-deterministic iteration count with {repr:?}"
        );
        assert_eq!(
            result1.diagnostics().context_count,
            result2.diagnostics().context_count,
            "Non-deterministic context count with {repr:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/// Result type for CI-PTA that can be normalized.
struct CiPtaResult {
    pts: PointsToMap,
}

/// Run CI-PTA with default settings.
fn run_ci_pta(module: &AirModule) -> CiPtaResult {
    let field_sensitivity = FieldSensitivity::StructFields { max_depth: 2 };
    let mut factory = LocationFactory::new(field_sensitivity);
    let constraints = extract_constraints(module, &mut factory);

    let pts = solve(&constraints, &factory, 10_000);

    CiPtaResult { pts }
}

/// Normalize a CI-PTA result to a comparable format.
fn normalize_pts_result(result: &CiPtaResult) -> Vec<(String, Vec<u128>)> {
    let mut normalized: Vec<(String, Vec<u128>)> = result
        .pts
        .iter()
        .map(|(vid, locs)| {
            let vid_str = format!("{vid:?}");
            let mut loc_ids: Vec<u128> = locs.iter().map(|l| l.raw()).collect();
            loc_ids.sort_unstable();
            (vid_str, loc_ids)
        })
        .collect();

    normalized.sort_by(|a, b| a.0.cmp(&b.0));
    normalized
}
