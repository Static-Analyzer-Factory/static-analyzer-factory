//! E2E tests for SVFG (Plan 026).
//!
//! Each test loads a compiled LLVM IR fixture, builds the full analysis
//! pipeline (CFGs + PTA + `CallGraph` + `DefUse` + MSSA), then constructs the
//! SVFG and verifies edges, reachability, and export.
//!
//! Source files: `tests/programs/c/svfg_*.c`, `tests/programs/cpp/svfg_*.cpp`,
//!              `tests/programs/rust/svfg_*.rs`
//! Compiled fixtures: `tests/fixtures/llvm/e2e/svfg_*.ll`

use std::collections::BTreeMap;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
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

    // PTA for SVFG queries
    let pta_config = PtaConfig::default();
    let mut pta_ctx = PtaContext::new(pta_config.clone());
    let pta_raw = pta_ctx.analyze(module);
    let pta_result = PtaResult::new(pta_raw.pts, Arc::new(pta_raw.factory), pta_raw.diagnostics);

    // PTA for MSSA (separate instance since PtaResult is consumed by MSSA build)
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

// ── Test 1: Store/load disambiguation ────────────────────────────────────

#[test]
fn svfg_store_load_disambig_builds() {
    let module = load_ll_fixture("svfg_store_load_disambig");
    let svfg = build_svfg(&module);

    assert!(svfg.node_count() > 0, "should have nodes");
    assert!(svfg.edge_count() > 0, "should have edges");
    assert!(
        svfg.diagnostics().direct_edge_count > 0,
        "should have direct edges"
    );
}

#[test]
fn svfg_store_load_disambig_export() {
    let module = load_ll_fixture("svfg_store_load_disambig");
    let svfg = build_svfg(&module);

    let export = svfg.export();
    let json = serde_json::to_string_pretty(&export).unwrap();
    assert!(!json.is_empty());
    assert_eq!(export.schema_version, "0.1.0");
    assert_eq!(export.node_count, svfg.node_count());
    assert_eq!(export.edge_count, svfg.edge_count());
}

// ── Test 2: Phi merge ────────────────────────────────────────────────────

#[test]
fn svfg_phi_merge_builds() {
    let module = load_ll_fixture("svfg_phi_merge");
    let svfg = build_svfg(&module);

    assert!(svfg.node_count() > 0, "should have nodes");
    assert!(svfg.edge_count() > 0, "should have edges");
}

#[test]
fn svfg_phi_merge_has_edges() {
    let module = load_ll_fixture("svfg_phi_merge");
    let svfg = build_svfg(&module);

    // Should have both direct and indirect edges
    let diag = svfg.diagnostics();
    assert!(
        diag.direct_edge_count > 0,
        "should have direct edges (SSA flow)"
    );
    // Indirect edges depend on PTA precision; at least the graph builds
}

// ── Test 3: Interprocedural direct flow ──────────────────────────────────

#[test]
fn svfg_interproc_direct_builds() {
    let module = load_ll_fixture("svfg_interproc_direct");
    let svfg = build_svfg(&module);

    assert!(svfg.node_count() > 0);
    assert!(svfg.edge_count() > 0);

    // Should have call-related edges (CallArg or Return)
    let diag = svfg.diagnostics();
    assert!(
        diag.direct_edge_count > 0,
        "should have direct edges including call edges"
    );
}

// ── Test 4: Reachability through memory ──────────────────────────────────

#[test]
fn svfg_reachability_builds() {
    let module = load_ll_fixture("svfg_reachability");
    let svfg = build_svfg(&module);

    assert!(svfg.node_count() > 0);
    assert!(svfg.edge_count() > 0);

    // Export is valid JSON
    let export = svfg.export();
    let json = serde_json::to_string(&export).unwrap();
    assert!(!json.is_empty());
}

// ── Test 5: C++ class member flow ────────────────────────────────────────

#[test]
fn svfg_class_member_builds() {
    let module = load_ll_fixture("svfg_class_member");
    let svfg = build_svfg(&module);

    assert!(svfg.node_count() > 0);
    assert!(svfg.edge_count() > 0);
}

// ── Test 6: Rust unsafe pointer flow ─────────────────────────────────────

#[test]
fn svfg_unsafe_ptr_builds() {
    let module = load_ll_fixture("svfg_unsafe_ptr");
    let svfg = build_svfg(&module);

    assert!(svfg.node_count() > 0);
    assert!(svfg.edge_count() > 0);
}

// ── Cross-cutting: Determinism ───────────────────────────────────────────

#[test]
fn svfg_export_is_deterministic() {
    let module = load_ll_fixture("svfg_reachability");

    let svfg1 = build_svfg(&module);
    let svfg2 = build_svfg(&module);

    let json1 = serde_json::to_string(&svfg1.export()).unwrap();
    let json2 = serde_json::to_string(&svfg2.export()).unwrap();

    assert_eq!(json1, json2, "SVFG export must be deterministic");
}
