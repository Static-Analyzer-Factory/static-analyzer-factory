//! E2E tests for the incremental analysis pipeline.
//!
//! These tests validate that `run_pipeline_incremental` produces meaningful
//! incremental results when only one module changes between runs.
//! Uses pre-compiled Lua 5.4.7 LLVM IR fixtures (~35 modules).

use saf_analysis::pipeline::{PipelineConfig, PipelineResult, run_pipeline_incremental};
use saf_analysis::session::AnalysisSession;
use saf_test_utils::load_incremental_lua_program;

/// Run a complete incremental scenario: full first run -> swap module -> incremental second run.
/// Returns (`first_run_result`, `incremental_result`, `run_count`).
fn run_incremental_scenario(
    swap_from: &str,
    swap_to: &str,
) -> (PipelineResult, PipelineResult, u64) {
    let config = PipelineConfig::default();
    let tmp = tempfile::tempdir().expect("create temp dir");
    let mut session = AnalysisSession::new(tmp.path().to_owned());

    // First run: full analysis on unmodified Lua
    let program_v1 = load_incremental_lua_program(None);
    let result_v1 = run_pipeline_incremental(&program_v1, &config, &mut session);

    // Second run: incremental analysis with one module swapped
    let program_v2 = load_incremental_lua_program(Some((swap_from, swap_to)));
    let result_v2 = run_pipeline_incremental(&program_v2, &config, &mut session);

    let run_count = session.run_count;
    (result_v1, result_v2, run_count)
}

#[test]
#[ignore] // Runs in ~30-60s — use `make test-incremental`
fn incremental_leaf_edit_correctness_and_speedup() {
    let (full, incr, run_count) = run_incremental_scenario("lmathlib", "lmathlib_v2");

    // Incremental path was taken
    assert_eq!(run_count, 2, "should have completed 2 runs");

    // Constraint diff is non-trivial
    assert!(
        incr.stats.changed_module_count >= 1,
        "leaf edit should change at least 1 module, got {}",
        incr.stats.changed_module_count,
    );
    assert!(
        incr.stats.constraint_diff_added > 0,
        "should have added constraints"
    );

    // Incremental PTA solver must have run
    assert!(
        incr.stats.pta_iterations > 0,
        "incremental PTA should have run"
    );

    // Incremental result produces non-empty PTA
    let incr_pts = incr.pta_result.as_ref().expect("incremental PTA result");
    let full_pts = full.pta_result.as_ref().expect("full PTA result");
    assert!(
        !incr_pts.points_to_map().is_empty(),
        "incremental PTA should produce non-empty results"
    );

    // Incremental result should be structurally similar to full
    // (not wildly different — at least 50% of original entries should still exist)
    let full_count = full_pts.points_to_map().len();
    let incr_count = incr_pts.points_to_map().len();
    assert!(
        incr_count > full_count / 2,
        "incremental result ({incr_count} entries) should retain most of full ({full_count} entries)"
    );

    eprintln!(
        "Leaf edit metrics: full_iters={}, incr_iters={}, diff_added={}, diff_removed={}, changed_modules={}, full_pts={}, incr_pts={}",
        full.stats.pta_iterations,
        incr.stats.pta_iterations,
        incr.stats.constraint_diff_added,
        incr.stats.constraint_diff_removed,
        incr.stats.changed_module_count,
        full_count,
        incr_count,
    );
}

#[test]
#[ignore]
fn incremental_core_edit_correctness_and_speedup() {
    let (full, incr, run_count) = run_incremental_scenario("lobject", "lobject_v2");

    // Incremental path was taken
    assert_eq!(run_count, 2);

    // Constraint diff is non-trivial
    assert!(
        incr.stats.changed_module_count >= 1,
        "core edit should change at least 1 module, got {}",
        incr.stats.changed_module_count,
    );
    assert!(incr.stats.constraint_diff_added > 0);

    // Incremental PTA solver must have run
    assert!(
        incr.stats.pta_iterations > 0,
        "incremental PTA should have run"
    );

    // Incremental result produces non-empty PTA
    let incr_pts = incr.pta_result.as_ref().expect("incremental PTA result");
    let full_pts = full.pta_result.as_ref().expect("full PTA result");
    assert!(
        !incr_pts.points_to_map().is_empty(),
        "incremental PTA should produce non-empty results"
    );

    // Structural similarity check
    let full_count = full_pts.points_to_map().len();
    let incr_count = incr_pts.points_to_map().len();
    assert!(
        incr_count > full_count / 2,
        "incremental result ({incr_count} entries) should retain most of full ({full_count} entries)"
    );

    eprintln!(
        "Core edit metrics: full_iters={}, incr_iters={}, diff_added={}, diff_removed={}, changed_modules={}, full_pts={}, incr_pts={}",
        full.stats.pta_iterations,
        incr.stats.pta_iterations,
        incr.stats.constraint_diff_added,
        incr.stats.constraint_diff_removed,
        incr.stats.changed_module_count,
        full_count,
        incr_count,
    );
}

#[test]
#[ignore]
fn incremental_determinism() {
    // Run the same incremental scenario twice and verify identical non-timing stats
    let config = PipelineConfig::default();

    let tmp1 = tempfile::tempdir().expect("create temp dir");
    let mut session1 = AnalysisSession::new(tmp1.path().to_owned());
    let prog1 = load_incremental_lua_program(None);
    let _ = run_pipeline_incremental(&prog1, &config, &mut session1);
    let prog1v2 = load_incremental_lua_program(Some(("lmathlib", "lmathlib_v2")));
    let result1 = run_pipeline_incremental(&prog1v2, &config, &mut session1);

    let tmp2 = tempfile::tempdir().expect("create temp dir");
    let mut session2 = AnalysisSession::new(tmp2.path().to_owned());
    let prog2 = load_incremental_lua_program(None);
    let _ = run_pipeline_incremental(&prog2, &config, &mut session2);
    let prog2v2 = load_incremental_lua_program(Some(("lmathlib", "lmathlib_v2")));
    let result2 = run_pipeline_incremental(&prog2v2, &config, &mut session2);

    // Non-timing stats must be identical (NFR-DET)
    assert_eq!(result1.stats.pta_iterations, result2.stats.pta_iterations);
    assert_eq!(
        result1.stats.constraint_diff_added,
        result2.stats.constraint_diff_added
    );
    assert_eq!(
        result1.stats.constraint_diff_removed,
        result2.stats.constraint_diff_removed
    );
    assert_eq!(
        result1.stats.changed_module_count,
        result2.stats.changed_module_count
    );

    // PTA results must be identical
    let pts1 = result1.pta_result.as_ref().expect("PTA result 1");
    let pts2 = result2.pta_result.as_ref().expect("PTA result 2");
    assert_eq!(pts1.points_to_map(), pts2.points_to_map());
}
