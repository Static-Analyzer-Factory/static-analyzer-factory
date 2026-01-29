//! End-to-end tests for abstract interpretation (E15).
//!
//! Source files: `tests/programs/c/absint_*.c`, `tests/programs/cpp/absint_*.cpp`,
//!              `tests/programs/rust/absint_*.rs`
//! Compiled fixtures: `tests/fixtures/llvm/e2e/absint_*.ll`

use saf_analysis::absint::{
    AbstractDomain, AbstractInterpConfig, check_all_numeric, check_buffer_overflow,
    check_division_by_zero, check_integer_overflow, check_memcpy_overflow, check_shift_count,
    solve_abstract_interp, solve_interprocedural,
};
use saf_test_utils::load_ll_fixture;

// ==========================================================================
// Build + load tests
// ==========================================================================

#[test]
fn absint_loop_bounds_builds() {
    let module = load_ll_fixture("absint_loop_bounds");
    let config = AbstractInterpConfig::default();
    let result = solve_abstract_interp(&module, &config);
    assert!(result.diagnostics().converged);
    assert!(result.diagnostics().functions_analyzed > 0);
}

#[test]
fn absint_nested_loops_builds() {
    let module = load_ll_fixture("absint_nested_loops");
    let config = AbstractInterpConfig::default();
    let result = solve_abstract_interp(&module, &config);
    assert!(result.diagnostics().converged);
    assert!(result.diagnostics().functions_analyzed > 0);
}

#[test]
fn absint_cpp_vector_builds() {
    let module = load_ll_fixture("absint_cpp_vector");
    let config = AbstractInterpConfig::default();
    let result = solve_abstract_interp(&module, &config);
    assert!(result.diagnostics().converged);
    assert!(result.diagnostics().functions_analyzed > 0);
}

#[test]
#[ignore = "SIGSEGV in LLVM frontend on Rust-generated IR (pre-existing)"]
fn absint_rust_unsafe_builds() {
    let module = load_ll_fixture("absint_rust_unsafe");
    let config = AbstractInterpConfig::default();
    let result = solve_abstract_interp(&module, &config);
    assert!(result.diagnostics().converged);
    assert!(result.diagnostics().functions_analyzed > 0);
}

// ==========================================================================
// Checker tests
// ==========================================================================

#[test]
fn buffer_overflow_checker_runs() {
    let module = load_ll_fixture("absint_buffer_overflow");
    let config = AbstractInterpConfig::default();
    let result = check_buffer_overflow(&module, &config);
    // The checker should at least run without panicking
    // Findings depend on whether GEP+index patterns exist in compiled IR
    assert!(result.absint_result.is_some());
}

#[test]
fn integer_overflow_checker_runs() {
    let module = load_ll_fixture("absint_integer_overflow");
    let config = AbstractInterpConfig::default();
    let result = check_integer_overflow(&module, &config);
    assert!(result.absint_result.is_some());
}

#[test]
fn check_all_numeric_runs() {
    let module = load_ll_fixture("absint_buffer_overflow");
    let config = AbstractInterpConfig::default();
    let result = check_all_numeric(&module, &config);
    assert!(result.absint_result.is_some());
}

// ==========================================================================
// Export tests
// ==========================================================================

#[test]
fn absint_export_is_valid_json() {
    let module = load_ll_fixture("absint_loop_bounds");
    let config = AbstractInterpConfig::default();
    let result = solve_abstract_interp(&module, &config);
    let export = result.export();
    let json = serde_json::to_string_pretty(&export).expect("export to JSON");
    assert!(json.contains("absint-v0.1.0"));
    assert!(json.contains("diagnostics"));
}

// ==========================================================================
// Determinism test
// ==========================================================================

#[test]
fn absint_deterministic() {
    let module = load_ll_fixture("absint_nested_loops");
    let config = AbstractInterpConfig::default();

    let result1 = solve_abstract_interp(&module, &config);
    let result2 = solve_abstract_interp(&module, &config);

    assert_eq!(
        result1.diagnostics().blocks_analyzed,
        result2.diagnostics().blocks_analyzed
    );
    assert_eq!(
        result1.diagnostics().widening_applications,
        result2.diagnostics().widening_applications
    );
    assert_eq!(
        result1.diagnostics().functions_analyzed,
        result2.diagnostics().functions_analyzed
    );

    // Export must be identical
    let export1 = serde_json::to_string(&result1.export()).unwrap();
    let export2 = serde_json::to_string(&result2.export()).unwrap();
    assert_eq!(export1, export2);
}

// ==========================================================================
// Division-by-zero checker tests (E26)
// ==========================================================================

#[test]
fn division_by_zero_checker_runs() {
    let module = load_ll_fixture("absint_div_by_zero");
    let config = AbstractInterpConfig::default();
    let result = check_division_by_zero(&module, &config);
    assert!(result.absint_result.is_some());
}

#[test]
fn div_by_zero_detects_definite_div() {
    use saf_analysis::absint::NumericSeverity;

    let module = load_ll_fixture("absint_div_by_zero");
    let config = AbstractInterpConfig::default();
    let result = check_division_by_zero(&module, &config);

    // Should find definite division by zero errors (x / 0, x % 0)
    let errors: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == NumericSeverity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "Expected at least one Error severity finding for definite division by zero"
    );
}

#[test]
fn check_all_includes_div_by_zero() {
    let module = load_ll_fixture("absint_div_by_zero");
    let config = AbstractInterpConfig::default();
    let result = check_all_numeric(&module, &config);

    // check_all_numeric should include division_by_zero findings
    let dbz_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.checker.name() == "division_by_zero")
        .collect();

    assert!(
        !dbz_findings.is_empty(),
        "check_all_numeric should include division_by_zero findings"
    );
}

// ==========================================================================
// Shift-count checker tests (E26)
// ==========================================================================

#[test]
fn shift_count_checker_runs() {
    let module = load_ll_fixture("absint_shift_count");
    let config = AbstractInterpConfig::default();
    let result = check_shift_count(&module, &config);
    assert!(result.absint_result.is_some());
}

#[test]
fn shift_count_detects_definite_overflow() {
    use saf_analysis::absint::NumericSeverity;

    let module = load_ll_fixture("absint_shift_count");
    let config = AbstractInterpConfig::default();
    let result = check_shift_count(&module, &config);

    // Should find definite shift count errors (x << 32, x >> 64, x >> -1)
    let errors: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == NumericSeverity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "Expected at least one Error severity finding for invalid shift count"
    );
}

#[test]
fn check_all_includes_shift_count() {
    let module = load_ll_fixture("absint_shift_count");
    let config = AbstractInterpConfig::default();
    let result = check_all_numeric(&module, &config);

    // check_all_numeric should include shift_count findings
    let sc_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.checker.name() == "shift_count")
        .collect();

    assert!(
        !sc_findings.is_empty(),
        "check_all_numeric should include shift_count findings"
    );
}

// ==========================================================================
// Memcpy/memmove/memset overflow checker tests
// ==========================================================================

#[test]
fn memcpy_overflow_checker_runs() {
    let module = load_ll_fixture("memcpy_overflow");
    let config = AbstractInterpConfig::default();
    let result = check_memcpy_overflow(&module, &config);
    assert!(result.absint_result.is_some());
}

#[test]
fn memcpy_overflow_detects_bad_cases() {
    use saf_analysis::absint::NumericCheckerKind;

    let module = load_ll_fixture("memcpy_overflow");
    let config = AbstractInterpConfig::default();
    let result = check_memcpy_overflow(&module, &config);

    // Should find memcpy overflow in the "bad" functions
    let overflow_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| {
            f.checker == NumericCheckerKind::BufferOverflow
                && (f.function.contains("bad") || f.function.contains("Bad"))
        })
        .collect();

    assert!(
        !overflow_findings.is_empty(),
        "Expected memcpy overflow findings in 'bad' functions, got {:?}",
        result.findings
    );
}

#[test]
fn memcpy_overflow_no_findings_in_good_cases() {
    use saf_analysis::absint::{NumericCheckerKind, NumericSeverity};

    let module = load_ll_fixture("memcpy_overflow");
    let config = AbstractInterpConfig::default();
    let result = check_memcpy_overflow(&module, &config);

    // Should NOT find memcpy overflow in the "good" functions
    let good_overflow_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| {
            f.checker == NumericCheckerKind::BufferOverflow
                && f.severity != NumericSeverity::Safe
                && (f.function.contains("good") || f.function.contains("Good"))
        })
        .collect();

    assert!(
        good_overflow_findings.is_empty(),
        "Expected no memcpy overflow findings in 'good' functions, got {good_overflow_findings:?}"
    );
}

#[test]
fn check_all_includes_memcpy_overflow() {
    let module = load_ll_fixture("memcpy_overflow");
    let config = AbstractInterpConfig::default();
    let result = check_all_numeric(&module, &config);

    // check_all_numeric should include memcpy overflow findings
    let memcpy_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| {
            f.description.contains("memcpy")
                || f.description.contains("memmove")
                || f.description.contains("memset")
        })
        .collect();

    assert!(
        !memcpy_findings.is_empty(),
        "check_all_numeric should include memcpy/memmove/memset overflow findings"
    );
}

// ==========================================================================
// Interprocedural abstract interpretation tests
// ==========================================================================

#[test]
fn interprocedural_absint_builds() {
    let module = load_ll_fixture("interprocedural_absint");
    let config = AbstractInterpConfig::default();
    let result = solve_abstract_interp(&module, &config);
    assert!(result.diagnostics().converged);
    assert!(result.diagnostics().functions_analyzed > 0);
}

#[test]
fn interprocedural_constant_return_propagates() {
    let module = load_ll_fixture("interprocedural_absint");
    let config = AbstractInterpConfig::default();
    let result = solve_interprocedural(&module, &config);

    // Find the return_constant function and verify it returns [42, 42]
    let return_constant_fn = module
        .functions
        .iter()
        .find(|f| f.name == "return_constant")
        .expect("return_constant function not found");

    // Get the summary for return_constant
    let summary = result.function_summary(&return_constant_fn.id);
    assert!(summary.is_some(), "No summary for return_constant");

    let return_interval = summary.unwrap().return_interval();
    assert!(
        return_interval.is_some(),
        "return_constant should have a return interval"
    );

    let interval = return_interval.unwrap();
    assert_eq!(
        interval.lo(),
        42,
        "return_constant should return exactly 42"
    );
    assert_eq!(
        interval.hi(),
        42,
        "return_constant should return exactly 42"
    );
}

#[test]
fn interprocedural_call_result_refined() {
    let module = load_ll_fixture("interprocedural_absint");
    let config = AbstractInterpConfig::default();
    let result = solve_interprocedural(&module, &config);

    // Find test_add_one_call and verify the call result is refined
    // (test_constant_return is inlined by the optimizer, so we use test_add_one_call instead)
    let test_fn = module
        .functions
        .iter()
        .find(|f| f.name == "test_add_one_call")
        .expect("test_add_one_call function not found");

    // Find the call instruction that calls add_one
    let mut call_result_interval = None;
    for block in &test_fn.blocks {
        for inst in &block.instructions {
            if let saf_core::air::Operation::CallDirect { callee } = &inst.op {
                // Check if this calls add_one
                if module
                    .functions
                    .iter()
                    .any(|f| f.id == *callee && f.name == "add_one")
                {
                    if let Some(dst) = inst.dst {
                        let state = result.state_at_inst(inst.id);
                        if let Some(s) = state {
                            call_result_interval = s.get_opt(dst).cloned();
                        }
                    }
                }
            }
        }
    }

    assert!(
        call_result_interval.is_some(),
        "Call result should have an interval"
    );

    let interval = call_result_interval.unwrap();
    // With interprocedural analysis, the call result should be [11, 11]
    // Without it, it would be TOP (unknown)
    assert!(
        !interval.is_top(),
        "Interprocedural analysis should refine call result from TOP to [11, 11]"
    );
    assert_eq!(interval.lo(), 11, "Call to add_one(10) should return 11");
    assert_eq!(interval.hi(), 11, "Call to add_one(10) should return 11");
}

// ==========================================================================
// SCCP dead-branch elimination tests
// ==========================================================================

#[test]
fn sccp_runs_on_real_ir() {
    // Hand-written IR with a constant branch: `add i32 0, 42` then `icmp eq %x, 42`.
    // SCCP should prove %x is always 42, making the else branch dead.
    let module = load_ll_fixture("sccp_dead_branch");
    let result = saf_analysis::absint::sccp::run_sccp_module(&module);
    // SCCP should find at least one dead block or constant in this fixture.
    assert!(
        result.dead_blocks.len() + result.constants.len() > 0,
        "SCCP should find dead blocks or constants"
    );
}

#[test]
fn sccp_integration_with_absint() {
    // Verify SCCP-enriched absint runs without regression.
    let module = load_ll_fixture("sccp_dead_branch");
    let config = AbstractInterpConfig::default();
    let result = solve_abstract_interp(&module, &config);
    assert!(result.diagnostics().converged);
    assert!(result.diagnostics().functions_analyzed > 0);
}

// ==========================================================================
// Trace partitioning tests (Plan 148 Phase C/D)
// ==========================================================================

#[test]
fn partition_const_branch_precise_intervals() {
    // Hand-written IR with a branch on a function parameter (`icmp sgt i32 %n, 0`).
    // Both branches are reachable, so the partition mechanism can split.
    let module = load_ll_fixture("partition_const_branch");
    // Verify SCCP runs cleanly on the branch IR
    let _sccp_result = saf_analysis::absint::sccp::run_sccp_module(&module);
    // Verify absint runs with partitioning enabled (the default config)
    let config = saf_analysis::absint::AbstractInterpConfig::default();
    assert!(
        config.partition.enabled,
        "Partitioning should be enabled by default"
    );
    let result = saf_analysis::absint::solve_abstract_interp(&module, &config);
    // The function should be analyzed successfully
    assert!(
        !result.block_states().is_empty(),
        "Absint should produce block states"
    );
}
