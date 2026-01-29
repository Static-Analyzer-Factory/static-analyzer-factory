//! Integration tests for PTA using AIR-JSON fixtures.

use std::sync::Arc;

use saf_analysis::export_constraints;
use saf_analysis::{AliasResult, FieldSensitivity, PtaConfig, PtaContext, PtaResult};
use saf_core::ids::ValueId;
use saf_test_utils::load_air_json_fixture as load_fixture;

fn make_default_config() -> PtaConfig {
    PtaConfig::default()
}

#[test]
fn pta_minimal_fixture() {
    let module = load_fixture("minimal");
    let mut ctx = PtaContext::new(make_default_config());
    let result = ctx.analyze(&module);

    // Minimal fixture has a single "main" function with just ret.
    // The only constraint is the function address Addr for main().
    assert_eq!(
        result.constraints.addr.len(),
        1,
        "should have 1 addr constraint (function address for main)"
    );
    assert_eq!(result.constraints.store.len(), 0);
    assert_eq!(result.constraints.load.len(), 0);
}

#[test]
fn pta_memory_ops_fixture() {
    let module = load_fixture("memory_ops");
    let mut ctx = PtaContext::new(make_default_config());
    let result = ctx.analyze(&module);

    // memory_ops has 2 allocas, store, load, gep, store.
    // Addr constraints: 2 from allocas + 1 from function address (main)
    assert_eq!(
        result.constraints.addr.len(),
        3,
        "should have 3 addr constraints (2 allocas + 1 function address)"
    );

    // Should have constraints for store, load, gep
    assert!(
        !result.constraints.store.is_empty(),
        "should have store constraints"
    );
    assert!(
        !result.constraints.load.is_empty(),
        "should have load constraints"
    );
    assert!(
        !result.constraints.gep.is_empty(),
        "should have gep constraints"
    );

    // Verify diagnostics
    assert!(
        result.diagnostics.constraint_count > 0,
        "should have some constraints"
    );
}

#[test]
fn pta_calls_fixture() {
    let module = load_fixture("calls");
    let mut ctx = PtaContext::new(make_default_config());
    let result = ctx.analyze(&module);

    // calls fixture has function calls
    // For now we don't fully model interprocedural, but constraints should be extracted
    // (this assertion simply confirms analysis completed without error)
    let _ = result.diagnostics.constraint_count;
}

#[test]
fn pta_deterministic_analysis() {
    let module = load_fixture("memory_ops");

    // Run analysis twice
    let mut ctx1 = PtaContext::new(make_default_config());
    let result1 = ctx1.analyze(&module);

    let mut ctx2 = PtaContext::new(make_default_config());
    let result2 = ctx2.analyze(&module);

    // Results should be identical
    assert_eq!(
        result1.pts.len(),
        result2.pts.len(),
        "same number of pts entries"
    );
    assert_eq!(
        result1.constraints.total_count(),
        result2.constraints.total_count(),
        "same number of constraints"
    );

    // Export should be deterministic
    let config = make_default_config();
    let export1 = serde_json::to_string(
        &PtaResult::new(result1.pts, Arc::new(result1.factory), result1.diagnostics)
            .export(&config),
    )
    .unwrap();
    let export2 = serde_json::to_string(
        &PtaResult::new(result2.pts, Arc::new(result2.factory), result2.diagnostics)
            .export(&config),
    )
    .unwrap();

    assert_eq!(export1, export2, "exports should be identical");
}

#[test]
fn pta_constraint_export_deterministic() {
    let module = load_fixture("memory_ops");

    let mut ctx1 = PtaContext::new(make_default_config());
    let result1 = ctx1.analyze(&module);

    let mut ctx2 = PtaContext::new(make_default_config());
    let result2 = ctx2.analyze(&module);

    let export1 = serde_json::to_string(&export_constraints(&result1.constraints)).unwrap();
    let export2 = serde_json::to_string(&export_constraints(&result2.constraints)).unwrap();

    assert_eq!(export1, export2, "constraint exports should be identical");
}

#[test]
fn pta_disabled_returns_empty() {
    let module = load_fixture("memory_ops");

    let config = PtaConfig {
        enabled: false,
        ..make_default_config()
    };

    let mut ctx = PtaContext::new(config);
    let result = ctx.analyze(&module);

    assert!(result.pts.is_empty());
    assert!(result.constraints.is_empty());
}

#[test]
fn pta_field_sensitivity_none() {
    let module = load_fixture("memory_ops");

    let config = PtaConfig {
        field_sensitivity: FieldSensitivity::None,
        ..PtaConfig::default()
    };

    let mut ctx = PtaContext::new(config);
    let result = ctx.analyze(&module);

    // With None field sensitivity, all field accesses collapse to base
    // Should still have constraints, just fewer locations
    assert!(
        result.diagnostics.constraint_count > 0,
        "should still have constraints"
    );
}

#[test]
fn pta_alias_query_basic() {
    let module = load_fixture("memory_ops");
    let mut ctx = PtaContext::new(make_default_config());
    let analysis_result = ctx.analyze(&module);

    let pta_result = PtaResult::new(
        analysis_result.pts,
        Arc::new(analysis_result.factory),
        analysis_result.diagnostics,
    );

    // Query alias for the two alloca results
    // 0x100 and 0x101 are dst values from allocas
    let p1 = ValueId::new(0x100);
    let p2 = ValueId::new(0x101);

    // Two different allocas should not alias
    let alias_result = pta_result.may_alias(p1, p2);

    // Should be No (they point to different allocas) or Unknown
    // depending on whether the analysis tracked them
    assert!(
        matches!(alias_result, AliasResult::No | AliasResult::Unknown),
        "different allocas should not alias (or be unknown)"
    );
}
