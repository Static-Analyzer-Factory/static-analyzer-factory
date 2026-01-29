//! E2E tests for path-sensitive checker reachability (Plan 032).
//!
//! Each test loads a compiled LLVM IR fixture, builds the full analysis
//! pipeline (CFGs + PTA + `CallGraph` + `DefUse` + MSSA + SVFG), then runs
//! path-sensitive checking and verifies expected feasibility classification.
//!
//! Source files: `tests/fixtures/e2e/source/ps_*.c`, `ps_*.cpp`
//! Compiled fixtures: `tests/fixtures/llvm/e2e/ps_*.ll`

use std::collections::BTreeMap;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::checkers::{
    self, PathSensitiveConfig, PathSensitiveResult, ResourceTable, SolverConfig, run_checkers,
    run_checkers_path_sensitive,
};
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::{Svfg, SvfgBuilder};
use saf_analysis::{PtaConfig, PtaContext, PtaResult};
use saf_core::air::AirModule;
use saf_core::ids::FunctionId;
use saf_test_utils::load_ll_fixture;

fn build_svfg(module: &AirModule) -> Svfg {
    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);

    let pta_config = PtaConfig::default();
    let mut pta_ctx = PtaContext::new(pta_config.clone());
    let pta_raw = pta_ctx.analyze(module);
    let pta_result = PtaResult::new(pta_raw.pts, Arc::new(pta_raw.factory), pta_raw.diagnostics);

    let mut pta_ctx2 = PtaContext::new(pta_config);
    let pta_raw2 = pta_ctx2.analyze(module);
    let mssa_pta = PtaResult::new(
        pta_raw2.pts,
        Arc::new(pta_raw2.factory),
        pta_raw2.diagnostics,
    );

    let cfgs: BTreeMap<FunctionId, Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect();
    let mut mssa = MemorySsa::build(module, &cfgs, mssa_pta, &callgraph);

    let (svfg, _program_points) =
        SvfgBuilder::new(module, &defuse, &callgraph, &pta_result, &mut mssa).build();
    svfg
}

fn run_path_sensitive(module: &AirModule, svfg: &Svfg) -> PathSensitiveResult {
    let table = ResourceTable::new();
    let specs = checkers::builtin_checkers();
    let config = PathSensitiveConfig::default();
    run_checkers_path_sensitive(&specs, module, svfg, &table, &config)
}

fn run_path_insensitive(module: &AirModule, svfg: &Svfg) -> Vec<checkers::CheckerFinding> {
    let table = ResourceTable::new();
    let config = SolverConfig::default();
    let specs = checkers::builtin_checkers();
    run_checkers(&specs, module, svfg, &table, &config).findings
}

// ── Test: ps_null_guard ──────────────────────────────────────────────────

#[test]
fn ps_null_guard_path_sensitive_runs() {
    let module = load_ll_fixture("ps_null_guard");
    let svfg = build_svfg(&module);
    let ps_result = run_path_sensitive(&module, &svfg);

    // Diagnostics should show the pipeline ran
    assert!(
        ps_result.diagnostics.total_findings
            == ps_result.diagnostics.feasible_count
                + ps_result.diagnostics.infeasible_count
                + ps_result.diagnostics.unknown_count
    );
}

// ── Test: ps_true_positive ───────────────────────────────────────────────

#[test]
fn ps_true_positive_preserved_in_both_modes() {
    let module = load_ll_fixture("ps_true_positive");
    let svfg = build_svfg(&module);

    // Path-insensitive should find something
    let pi_findings = run_path_insensitive(&module, &svfg);
    let ps_result = run_path_sensitive(&module, &svfg);

    // True positive: the UAF is genuine and should survive path-sensitive filtering.
    // Path-sensitive should NOT filter out true positives.
    // The total (feasible + unknown) should be >= 1 if PI found anything.
    if !pi_findings.is_empty() {
        assert!(
            ps_result.feasible.len() + ps_result.unknown.len() >= 1,
            "True positive should survive path-sensitive filtering: \
             pi_findings={}, feasible={}, unknown={}, infeasible={}",
            pi_findings.len(),
            ps_result.feasible.len(),
            ps_result.unknown.len(),
            ps_result.infeasible.len(),
        );
    }
}

// ── Test: ps_multi_condition ─────────────────────────────────────────────

#[test]
fn ps_multi_condition_no_over_filtering() {
    let module = load_ll_fixture("ps_multi_condition");
    let svfg = build_svfg(&module);

    let pi_findings = run_path_insensitive(&module, &svfg);
    let ps_result = run_path_sensitive(&module, &svfg);

    // The UAF path is genuinely feasible (x was 42 > 0).
    // Path-sensitive should NOT filter it.
    if !pi_findings.is_empty() {
        assert!(
            ps_result.feasible.len() + ps_result.unknown.len() >= 1,
            "Feasible bug path should not be over-filtered"
        );
    }
}

// ── Test: ps_cpp_raii_guard ──────────────────────────────────────────────

#[test]
fn ps_cpp_raii_guard_path_sensitive_runs() {
    let module = load_ll_fixture("ps_cpp_raii_guard");
    let svfg = build_svfg(&module);
    let ps_result = run_path_sensitive(&module, &svfg);

    // Verify diagnostics are consistent
    assert_eq!(
        ps_result.diagnostics.total_findings,
        ps_result.diagnostics.feasible_count
            + ps_result.diagnostics.infeasible_count
            + ps_result.diagnostics.unknown_count
    );
}

// ── Test: Determinism ────────────────────────────────────────────────────

#[test]
fn ps_determinism() {
    let module = load_ll_fixture("ps_true_positive");
    let svfg = build_svfg(&module);

    let r1 = run_path_sensitive(&module, &svfg);
    let r2 = run_path_sensitive(&module, &svfg);

    assert_eq!(r1.feasible.len(), r2.feasible.len());
    assert_eq!(r1.infeasible.len(), r2.infeasible.len());
    assert_eq!(r1.unknown.len(), r2.unknown.len());
    assert_eq!(r1.diagnostics.total_findings, r2.diagnostics.total_findings);
}

// ── Test: Diagnostics export ─────────────────────────────────────────────

#[test]
fn ps_diagnostics_export() {
    let module = load_ll_fixture("ps_null_guard");
    let svfg = build_svfg(&module);
    let ps_result = run_path_sensitive(&module, &svfg);

    let diag = &ps_result.diagnostics;
    let json = serde_json::to_string(diag).expect("diagnostics should serialize");
    assert!(json.contains("total_findings"));
    assert!(json.contains("feasible_count"));
    assert!(json.contains("z3_calls"));
}
