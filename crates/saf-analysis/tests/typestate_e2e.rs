//! E2E tests for typestate analysis via the IDE solver.
//!
//! Each test loads a compiled LLVM IR fixture, runs the full IDE pipeline
//! (LLVM frontend → AIR → ICFG → IDE solver → collect findings), and
//! verifies typestate violation detection.
//!
//! Source files are in `tests/fixtures/sources/typestate_*` (C, C++).
//! Compiled fixtures are in `tests/fixtures/llvm/e2e/typestate_*.ll`.

use saf_analysis::callgraph::CallGraph;
use saf_analysis::icfg::Icfg;
use saf_analysis::ifds::{
    IfdsConfig, TypestateFindingKind, TypestateIdeProblem, builtin_typestate_spec, solve_ide,
};
use saf_core::air::AirModule;
use saf_test_utils::load_ll_fixture;

fn run_typestate_file_io(module: &AirModule) -> Vec<saf_analysis::ifds::TypestateFinding> {
    let spec = builtin_typestate_spec("file_io").expect("file_io spec should exist");
    let problem = TypestateIdeProblem::new(module, spec);
    let callgraph = CallGraph::build(module);
    let icfg = Icfg::build(module, &callgraph);
    let config = IfdsConfig::default();
    let result = solve_ide(&problem, &icfg, &callgraph, &config);
    problem.collect_findings(&result)
}

fn run_typestate_mutex(module: &AirModule) -> Vec<saf_analysis::ifds::TypestateFinding> {
    let spec = builtin_typestate_spec("mutex_lock").expect("mutex_lock spec should exist");
    let problem = TypestateIdeProblem::new(module, spec);
    let callgraph = CallGraph::build(module);
    let icfg = Icfg::build(module, &callgraph);
    let config = IfdsConfig::default();
    let result = solve_ide(&problem, &icfg, &callgraph, &config);
    problem.collect_findings(&result)
}

fn run_typestate_memory(module: &AirModule) -> Vec<saf_analysis::ifds::TypestateFinding> {
    let spec = builtin_typestate_spec("memory_alloc").expect("memory_alloc spec should exist");
    let problem = TypestateIdeProblem::new(module, spec);
    let callgraph = CallGraph::build(module);
    let icfg = Icfg::build(module, &callgraph);
    let config = IfdsConfig::default();
    let result = solve_ide(&problem, &icfg, &callgraph, &config);
    problem.collect_findings(&result)
}

// ── File leak: fopen without fclose ──────────────────────────────────────

#[test]
fn typestate_file_leak_detects_non_accepting() {
    let module = load_ll_fixture("typestate_file_leak");
    let findings = run_typestate_file_io(&module);

    let leak_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == TypestateFindingKind::NonAcceptingAtExit)
        .collect();

    assert!(
        !leak_findings.is_empty(),
        "should detect file leak (fopen without fclose): findings = {findings:?}"
    );

    // All leak findings should be in the "opened" state.
    for f in &leak_findings {
        assert_eq!(
            f.state, "opened",
            "leaked file should be in 'opened' state, got '{}'",
            f.state
        );
    }
}

// ── Double fclose: error state ───────────────────────────────────────────

#[test]
fn typestate_double_close_detects_error() {
    let module = load_ll_fixture("typestate_double_close");
    let findings = run_typestate_file_io(&module);

    let error_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == TypestateFindingKind::ErrorState)
        .collect();

    assert!(
        !error_findings.is_empty(),
        "should detect double-close error: findings = {findings:?}"
    );

    for f in &error_findings {
        assert_eq!(
            f.state, "error",
            "double-close should produce 'error' state, got '{}'",
            f.state
        );
    }
}

// ── Use-after-close: fread after fclose → error state ────────────────────

#[test]
fn typestate_use_after_close_detects_error() {
    let module = load_ll_fixture("typestate_use_after_close");
    let findings = run_typestate_file_io(&module);

    let error_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == TypestateFindingKind::ErrorState)
        .collect();

    assert!(
        !error_findings.is_empty(),
        "should detect use-after-close error: findings = {findings:?}"
    );

    for f in &error_findings {
        assert_eq!(
            f.state, "error",
            "use-after-close should produce 'error' state, got '{}'",
            f.state
        );
    }
}

// ── Correct program: no findings ─────────────────────────────────────────

#[test]
fn typestate_correct_program_no_findings() {
    let module = load_ll_fixture("typestate_correct");
    let findings = run_typestate_file_io(&module);

    // A correct program that opens and closes the file on all paths
    // should produce no error-state findings.
    let error_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == TypestateFindingKind::ErrorState)
        .collect();

    assert!(
        error_findings.is_empty(),
        "correct program should have no error-state findings, got: {error_findings:?}"
    );
}

// ── Mutex lock without unlock: non-accepting at exit ─────────────────────

#[test]
fn typestate_lock_without_unlock_detects_non_accepting() {
    let module = load_ll_fixture("typestate_lock");
    let findings = run_typestate_mutex(&module);

    let leak_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == TypestateFindingKind::NonAcceptingAtExit)
        .collect();

    assert!(
        !leak_findings.is_empty(),
        "should detect lock without unlock: findings = {findings:?}"
    );

    for f in &leak_findings {
        assert_eq!(
            f.state, "locked",
            "unreleased mutex should be in 'locked' state, got '{}'",
            f.state
        );
    }
}

// ── Double free: error state ─────────────────────────────────────────────

#[test]
fn typestate_double_free_detects_error() {
    let module = load_ll_fixture("typestate_double_free");
    let findings = run_typestate_memory(&module);

    let error_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == TypestateFindingKind::ErrorState)
        .collect();

    assert!(
        !error_findings.is_empty(),
        "should detect double-free error: findings = {findings:?}"
    );

    for f in &error_findings {
        assert_eq!(
            f.state, "error",
            "double-free should produce 'error' state, got '{}'",
            f.state
        );
    }
}

// ── Determinism: results should be identical across runs ─────────────────

#[test]
fn typestate_results_are_deterministic() {
    let module = load_ll_fixture("typestate_double_close");

    let findings1 = run_typestate_file_io(&module);
    let findings2 = run_typestate_file_io(&module);

    assert_eq!(
        findings1, findings2,
        "typestate findings should be deterministic"
    );
}
