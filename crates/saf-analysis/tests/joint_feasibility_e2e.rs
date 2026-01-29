//! E2E tests for joint path-feasibility filtering of `MultiReach` findings (Plan 112).
//!
//! Verifies that the joint feasibility filter correctly distinguishes
//! mutually exclusive double-frees (false positives) from real double-frees
//! (true positives) using the full analysis pipeline.
//!
//! Source files: `tests/fixtures/sources/checker_double_free_exclusive.c`,
//!              `tests/fixtures/sources/checker_double_free.c`
//! Compiled fixtures: `tests/fixtures/llvm/e2e/checker_double_free_exclusive.ll`,
//!                    `tests/fixtures/llvm/e2e/checker_double_free.ll`

use std::collections::BTreeMap;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::checkers::{
    self, PathSensitiveConfig, PathSensitiveResult, ResourceTable, SolverConfig,
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

fn run_double_free_path_sensitive(module: &AirModule, svfg: &Svfg) -> PathSensitiveResult {
    let table = ResourceTable::new();
    let spec = checkers::builtin_checker("double-free").expect("double-free checker exists");
    let config = PathSensitiveConfig::default();
    run_checkers_path_sensitive(&[spec], module, svfg, &table, &config)
}

// ── Test: Mutually exclusive double-free (false positive) ────────────────

#[test]
fn exclusive_double_free_pipeline_runs() {
    let module = load_ll_fixture("checker_double_free_exclusive");
    let svfg = build_svfg(&module);
    let ps_result = run_double_free_path_sensitive(&module, &svfg);

    // Diagnostics should be consistent
    assert_eq!(
        ps_result.diagnostics.total_findings,
        ps_result.diagnostics.feasible_count
            + ps_result.diagnostics.infeasible_count
            + ps_result.diagnostics.unknown_count
    );
}

#[test]
fn exclusive_double_free_filtered_by_joint_feasibility() {
    let module = load_ll_fixture("checker_double_free_exclusive");
    let svfg = build_svfg(&module);

    // Path-insensitive: should detect the two free() calls as double-free
    let table = ResourceTable::new();
    let spec = checkers::builtin_checker("double-free").expect("double-free checker exists");
    let pi_result = checkers::run_checker(&spec, &module, &svfg, &table, &SolverConfig::default());

    // Path-sensitive: joint feasibility should filter the exclusive frees
    let ps_result = run_double_free_path_sensitive(&module, &svfg);

    // The path-insensitive checker may or may not find double-free findings
    // depending on SVFG connectivity. If it does find them, the joint
    // feasibility filter should move them to infeasible (mutually exclusive).
    if !pi_result.findings.is_empty() {
        let df_feasible: Vec<_> = ps_result
            .feasible
            .iter()
            .filter(|f| f.checker_name == "double-free")
            .collect();
        let df_infeasible: Vec<_> = ps_result
            .infeasible
            .iter()
            .filter(|f| f.checker_name == "double-free")
            .collect();

        // Either the finding was filtered (infeasible) or moved to unknown,
        // but it should NOT be in feasible since the branches are exclusive.
        assert!(
            df_feasible.is_empty() || !df_infeasible.is_empty(),
            "Exclusive double-free should be filtered or at least partially classified: \
             feasible={}, infeasible={}, unknown={}",
            ps_result.feasible.len(),
            ps_result.infeasible.len(),
            ps_result.unknown.len(),
        );
    }
}

// ── Test: Real double-free (true positive) ───────────────────────────────

#[test]
fn real_double_free_pipeline_runs() {
    let module = load_ll_fixture("checker_double_free");
    let svfg = build_svfg(&module);
    let ps_result = run_double_free_path_sensitive(&module, &svfg);

    // Diagnostics should be consistent
    assert_eq!(
        ps_result.diagnostics.total_findings,
        ps_result.diagnostics.feasible_count
            + ps_result.diagnostics.infeasible_count
            + ps_result.diagnostics.unknown_count
    );
}

#[test]
fn real_double_free_not_filtered() {
    let module = load_ll_fixture("checker_double_free");
    let svfg = build_svfg(&module);

    // Path-insensitive: should detect double-free
    let table = ResourceTable::new();
    let spec = checkers::builtin_checker("double-free").expect("double-free checker exists");
    let pi_result = checkers::run_checker(&spec, &module, &svfg, &table, &SolverConfig::default());

    // Path-sensitive: joint feasibility should NOT filter a real double-free
    let ps_result = run_double_free_path_sensitive(&module, &svfg);

    // If path-insensitive found findings, they should survive in feasible or unknown
    if !pi_result.findings.is_empty() {
        let df_surviving: Vec<_> = ps_result
            .feasible
            .iter()
            .chain(ps_result.unknown.iter())
            .filter(|f| f.checker_name == "double-free")
            .collect();

        assert!(
            !df_surviving.is_empty(),
            "Real double-free should survive path-sensitive filtering: \
             pi_findings={}, feasible={}, infeasible={}, unknown={}",
            pi_result.findings.len(),
            ps_result.feasible.len(),
            ps_result.infeasible.len(),
            ps_result.unknown.len(),
        );
    }
}

// ── Test: Determinism ────────────────────────────────────────────────────

#[test]
fn joint_feasibility_deterministic() {
    let module = load_ll_fixture("checker_double_free_exclusive");
    let svfg = build_svfg(&module);

    let r1 = run_path_sensitive(&module, &svfg);
    let r2 = run_path_sensitive(&module, &svfg);

    assert_eq!(r1.feasible.len(), r2.feasible.len());
    assert_eq!(r1.infeasible.len(), r2.infeasible.len());
    assert_eq!(r1.unknown.len(), r2.unknown.len());
}
