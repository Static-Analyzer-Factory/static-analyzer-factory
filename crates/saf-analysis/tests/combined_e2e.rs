//! End-to-end tests for combined PTA + Abstract Interpretation analysis (E32).
//!
//! Source files: `tests/programs/c/pta_absint_*.c`, `tests/programs/c/indirect_call.c`
//! Compiled fixtures: `tests/fixtures/llvm/e2e/pta_absint_*.ll`, `indirect_call.ll`
//!
//! These tests verify the PTA-absint integration layer, including:
//! - Alias-aware memory tracking
//! - Indirect call resolution
//! - Combined analysis orchestration

use saf_analysis::combined::{CombinedAnalysisConfig, analyze_combined};
use saf_core::air::AirModule;
use saf_core::ids::ModuleId;

// ==========================================================================
// Unit tests with synthetic modules
// ==========================================================================

fn empty_module() -> AirModule {
    AirModule::new(ModuleId::derive(b"combined_test"))
}

#[test]
fn combined_analysis_empty_module() {
    let module = empty_module();
    let config = CombinedAnalysisConfig::default();
    let result = analyze_combined(&module, &config);

    assert_eq!(result.refinement_iterations, 0);
}

#[test]
fn combined_analysis_config_defaults() {
    let config = CombinedAnalysisConfig::default();
    assert!(config.enable_refinement);
    assert_eq!(config.max_refinement_iterations, 3);
    assert!(!config.context_sensitive_indirect);
}

#[test]
fn combined_analysis_uses_pta() {
    // Use analyze_combined instead of manual PTA + absint setup
    let module = empty_module();
    let config = CombinedAnalysisConfig::default();
    let result = analyze_combined(&module, &config);

    // Should complete without error even on empty module
    assert!(result.absint.diagnostics().converged);
}

// ==========================================================================
// E2E tests with LLVM fixtures (require Docker)
// These tests are commented out by default since they require compiled fixtures.
// Uncomment after compiling the test programs:
//   make shell
//   clang -S -emit-llvm -g -O0 tests/programs/c/pta_absint_alias.c -o tests/fixtures/llvm/e2e/pta_absint_alias.ll
//   clang -S -emit-llvm -g -O0 tests/programs/c/indirect_call.c -o tests/fixtures/llvm/e2e/indirect_call.ll
// ==========================================================================

// #[test]
// fn combined_analysis_alias_aware_store() {
//     use saf_test_utils::load_ll_fixture;
//
//     let module = load_ll_fixture("pta_absint_alias");
//     let config = CombinedAnalysisConfig::default();
//     let result = analyze_combined(&module, &config);
//
//     // Analysis should complete
//     let diag = result.absint.diagnostics();
//     assert!(diag.converged);
//     assert!(diag.functions_analyzed > 0);
// }

// #[test]
// fn combined_analysis_indirect_call_resolution() {
//     use saf_test_utils::load_ll_fixture;
//
//     let module = load_ll_fixture("indirect_call");
//     let config = CombinedAnalysisConfig::default();
//     let result = analyze_combined(&module, &config);
//
//     let diag = result.absint.diagnostics();
//     assert!(diag.converged);
//     assert!(diag.functions_analyzed >= 4); // main + 3 return_* functions
// }
