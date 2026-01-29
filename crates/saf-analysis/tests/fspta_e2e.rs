//! E2E tests for flow-sensitive PTA (Plan 027).
//!
//! Each test loads a compiled LLVM IR fixture, builds the full pipeline
//! (CFGs + PTA + `CallGraph` + `DefUse` + MSSA + SVFG + `FsSvfg` + SFS solver),
//! and verifies that the flow-sensitive result is at least as precise as
//! Andersen CI.
//!
//! Source files: `tests/programs/c/fspta_*.c`, `tests/programs/cpp/fspta_*.cpp`,
//!              `tests/programs/rust/fspta_*.rs`
//! Compiled fixtures: `tests/fixtures/llvm/e2e/fspta_*.ll`

use std::collections::BTreeMap;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::fspta::{self, FlowSensitivePtaResult, FsPtaConfig, FsSvfgBuilder};
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::SvfgBuilder;
use saf_analysis::{PtaConfig, PtaContext, PtaResult};
use saf_core::air::AirModule;
use saf_core::ids::FunctionId;
use saf_test_utils::load_ll_fixture;

fn run_andersen(module: &AirModule) -> PtaResult {
    let mut ctx = PtaContext::new(PtaConfig::default());
    let raw = ctx.analyze(module);
    PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics)
}

fn run_flow_sensitive(module: &AirModule) -> (FlowSensitivePtaResult, PtaResult) {
    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);
    let pta_config = PtaConfig::default();

    // PTA 1: for SVFG builder
    let mut ctx1 = PtaContext::new(pta_config.clone());
    let raw1 = ctx1.analyze(module);
    let pta1 = PtaResult::new(raw1.pts, Arc::new(raw1.factory), raw1.diagnostics);

    // PTA 2: consumed by MSSA for SVFG
    let mut ctx2 = PtaContext::new(pta_config.clone());
    let raw2 = ctx2.analyze(module);
    let mssa_pta = PtaResult::new(raw2.pts, Arc::new(raw2.factory), raw2.diagnostics);

    let cfgs: BTreeMap<FunctionId, Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect();

    let mut mssa = MemorySsa::build(module, &cfgs, mssa_pta, &callgraph);
    let (svfg, _program_points) =
        SvfgBuilder::new(module, &defuse, &callgraph, &pta1, &mut mssa).build();

    // PTA 3: for FsSvfg builder queries
    let mut ctx3 = PtaContext::new(pta_config.clone());
    let raw3 = ctx3.analyze(module);
    let pta3 = PtaResult::new(raw3.pts, Arc::new(raw3.factory), raw3.diagnostics);

    // PTA 4: consumed by MSSA for FsSvfg builder
    let mut ctx4 = PtaContext::new(pta_config);
    let raw4 = ctx4.analyze(module);
    let mssa_pta2 = PtaResult::new(raw4.pts, Arc::new(raw4.factory), raw4.diagnostics);
    let mut mssa2 = MemorySsa::build(module, &cfgs, mssa_pta2, &callgraph);

    let fs_svfg = FsSvfgBuilder::new(module, &svfg, &pta3, &mut mssa2, &callgraph).build();

    let config = FsPtaConfig::default();
    let result = fspta::solve_flow_sensitive(module, &fs_svfg, &pta3, &callgraph, &config);

    (result, pta3)
}

// ── Test 1: Strong update (C) ────────────────────────────────────────────

#[test]
fn fspta_strong_update_builds_and_converges() {
    let module = load_ll_fixture("fspta_strong_update");
    let (result, _andersen) = run_flow_sensitive(&module);

    // Should converge without hitting the limit
    assert!(
        !result.diagnostics().iteration_limit_hit,
        "solver should converge"
    );
}

#[test]
fn fspta_strong_update_export() {
    let module = load_ll_fixture("fspta_strong_update");
    let (result, _) = run_flow_sensitive(&module);

    let export = result.export();
    assert_eq!(export.schema_version, "0.1.0");
    let json = serde_json::to_string_pretty(&export).expect("serialize");
    assert!(!json.is_empty());
}

// ── Test 2: Branch merge (C) ─────────────────────────────────────────────

#[test]
fn fspta_branch_merge_builds() {
    let module = load_ll_fixture("fspta_branch_merge");
    let (result, _) = run_flow_sensitive(&module);

    assert!(!result.diagnostics().iteration_limit_hit);
}

// ── Test 3: Loop weak update (C) ─────────────────────────────────────────

#[test]
fn fspta_loop_weak_update_builds() {
    let module = load_ll_fixture("fspta_loop_weak_update");
    let (result, _) = run_flow_sensitive(&module);

    assert!(!result.diagnostics().iteration_limit_hit);
}

// ── Test 4: Interprocedural (C) ──────────────────────────────────────────

#[test]
fn fspta_interproc_builds() {
    let module = load_ll_fixture("fspta_interproc");
    let (result, _) = run_flow_sensitive(&module);

    assert!(!result.diagnostics().iteration_limit_hit);
}

// ── Test 5: C++ class field ──────────────────────────────────────────────

#[test]
fn fspta_cpp_field_builds() {
    let module = load_ll_fixture("fspta_cpp_field");
    let (result, _) = run_flow_sensitive(&module);

    assert!(!result.diagnostics().iteration_limit_hit);
}

// ── Test 6: Rust unsafe ──────────────────────────────────────────────────

#[test]
fn fspta_rust_unsafe_builds() {
    let module = load_ll_fixture("fspta_rust_unsafe");
    let (result, _) = run_flow_sensitive(&module);

    assert!(!result.diagnostics().iteration_limit_hit);
}

// ── Cross-cutting: Determinism ───────────────────────────────────────────

#[test]
fn fspta_export_is_deterministic() {
    let module = load_ll_fixture("fspta_strong_update");

    let (r1, _) = run_flow_sensitive(&module);
    let (r2, _) = run_flow_sensitive(&module);

    let json1 = serde_json::to_string(&r1.export()).unwrap();
    let json2 = serde_json::to_string(&r2.export()).unwrap();

    assert_eq!(
        json1, json2,
        "flow-sensitive PTA export must be deterministic"
    );
}

// ── Precision: FS should be no worse than Andersen ───────────────────────

#[test]
fn fspta_strong_update_no_worse_than_andersen() {
    let module = load_ll_fixture("fspta_strong_update");
    let andersen = run_andersen(&module);
    let (fs_result, _) = run_flow_sensitive(&module);

    // For every value, FS pts should be a subset of (or equal to) Andersen pts
    for (vid, fs_pts) in fs_result.points_to_map() {
        let andersen_pts: std::collections::BTreeSet<_> =
            andersen.points_to(*vid).into_iter().collect();
        if !andersen_pts.is_empty() {
            assert!(
                fs_pts.is_subset(&andersen_pts),
                "FS pts for {vid:?} should be subset of Andersen: FS={fs_pts:?}, Andersen={andersen_pts:?}"
            );
        }
    }
}
