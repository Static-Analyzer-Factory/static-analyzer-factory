# Plan 071: Test Suite Consolidation

## Motivation

The project has **1,613 Rust tests** and **248 Python tests** (1,861 total). Many are redundant, overlapping, or test trivial behavior. This plan reduces the test count while preserving all meaningful regression coverage.

## Current Test Inventory

### Rust Integration Tests (by file, top offenders)
| File | Tests | Issue |
|------|-------|-------|
| absint_verification_e2e.rs | 55 | Many tests load same fixture, assert only `converged` |
| pta_verification_e2e.rs | 35 | Many tests overlap with each other and with pta_integration.rs |
| mssa_verification_e2e.rs | 35 | Many SVFG tests just assert `node_count > 0` |
| absint_e2e.rs | 31 | `_builds` tests subsumed by checker tests on same fixtures |
| ifds_verification_e2e.rs | 27 | Some tests have weak assertions (`facts.is_empty()`) |
| z3_enhanced_e2e.rs | 20 | 8 tests just load a fixture with no assertions |

### Rust Inline Unit Tests (top offenders)
| File | Tests | Issue |
|------|-------|-------|
| interval.rs | 52 | Some overlap with absint_verification lattice tests |
| bdd.rs | 38 | 24 tests duplicate bitvec.rs/btree.rs/cross_impl_tests.rs |
| bitvec.rs | 34 | 32 tests duplicate bdd.rs/btree.rs/cross_impl_tests.rs |
| config_tests.rs | 25 | ~6 tests duplicate config.rs |
| btree.rs | 20 | 19 tests duplicate bdd.rs/bitvec.rs/cross_impl_tests.rs |

### Python Tests
| File | Tests | Issue |
|------|-------|-------|
| test_acceptance.py | 32 | Many are micro-tests of API surface |
| test_checkers.py | 27 | Individual checker "runs" tests mirror Rust |
| test_pts_repr.py | 21 | 95% duplicate of Rust pts_repr_e2e.rs |
| test_path_sensitive.py | 18 | 70% mirror pathsens_e2e.rs |
| test_ide.py | 18 | 80% mirror typestate_e2e.rs |
| test_absint.py | 15 | 70% mirror absint_e2e.rs |
| test_cg_refinement.py | 14 | 75% mirror cg_refinement_e2e.rs |
| test_svfg.py | 13 | 85% mirror svfg_e2e.rs |
| test_dda.py | 13 | 90% mirror dda_e2e.rs |
| test_fspta.py | 12 | 90% mirror fspta_e2e.rs |
| test_mssa.py | 10 | 85% mirror mssa_e2e.rs |
| test_ifds.py | 10 | 90% mirror ifds_e2e.rs |
| e2e/test_memory_e2e.py | 9 | 80% mirror memory_e2e.rs |
| e2e/test_oop_e2e.py | 8 | 85% mirror oop_e2e.rs |
| e2e/test_multi_module_e2e.py | 7 | 70% mirror multi_module_e2e.rs |
| e2e/test_integer_info_e2e.py | 7 | 80% mirror integer_info_e2e.rs |

## Strategy

### Guiding Principles
1. **Keep tests with specific behavioral assertions** (e.g., "finds leak in opened state")
2. **Remove tests that only assert non-emptiness/convergence** when another test on the same fixture already asserts more
3. **Remove fixture-load-only tests** (`let _module = load_fixture(...)`)
4. **Consolidate per-fixture tests** into parametrized or combined tests
5. **Remove Python tests that mirror Rust** — keep only Python-specific API tests
6. **Keep all inline unit tests that test algorithm correctness** (interval arithmetic, lattice properties, edge function composition)
7. **Remove cross-implementation duplication** in PtsSet (keep cross_impl_tests.rs, remove per-impl duplicates)

### Out of Scope
- Modifying any analysis logic
- Changing test fixtures
- Adding new tests

---

## Phase A: Verification E2E Test Consolidation (~80 tests removed)

### A1: absint_verification_e2e.rs (55 → ~18 tests)

**Remove tests that are subsets of other tests on the same fixture:**

- Phase 1 (6 → 2): Keep `test_simple_loop_converges` and `test_max_iterations_bounds_analysis`. Remove `test_widening_is_applied`, `test_nested_loops_converge`, `test_multiple_exits_converge`, `test_unbounded_counter_converges` — all load the same fixture and just check `converged` or `widening_applications > 0`, which is already covered by the first test.

- Phase 2 (5 → 2): Keep `test_threshold_extraction` and `test_threshold_widening_precision`. Remove `test_threshold_neighbors_included` (subset of extraction test), `test_type_boundary_thresholds` (subset), `test_bounded_loop_with_thresholds` (just checks convergence + function exists).

- Phase 3 (9 → 4): Keep `test_i8_overflow_interval`, `test_interval_bit_width_respected`, `test_zext_preserves_range`, `test_wrapped_arithmetic_module_converges`. Remove `test_multiplication_overflow` (covered by i8 overflow pattern), `test_subtraction_underflow` (same pattern), `test_sext_preserves_signed` (mirror of zext), `test_trunc_fits` + `test_trunc_overflow_becomes_top` (basic operations already tested in interval.rs unit tests).

- Phase 4 (7 → 3): Keep `test_narrowing_iterations_recorded`, `test_interval_narrow_from_top`, `test_narrowing_soundness`. Remove `test_different_narrowing_counts` (loop of `test_narrowing_iterations_recorded`), `test_zero_narrowing_converges` (subset of different_narrowing_counts), `test_interval_narrow_partial` (weaker version of narrow_from_top).

- Phase 5 (18 → 5): Keep `test_join_commutativity`, `test_meet_commutativity`, `test_bottom_join_identity`, `test_top_meet_identity`, `test_widening_soundness`. Remove: `test_join_commutativity_disjoint` (same as commutativity with different data), `test_join_associativity`, `test_join_idempotency`, `test_meet_associativity`, `test_meet_idempotency` (all basic algebraic properties already implied by the comprehensive tests in interval.rs), `test_join_is_upper_bound`, `test_meet_is_lower_bound`, `test_leq_reflexivity`, `test_leq_transitivity`, `test_narrowing_refinement`, `test_disjoint_meet_is_bottom` (tested in interval.rs), `test_state_*` (3 tests testing AbstractState lattice — keep if not covered in state.rs unit tests, check first).

- Phase 6 (10 → 2): Keep `test_all_fixtures_analyzable` (parametrized over all fixtures), `test_determinism`. Remove `test_integration_module_analysis`, `test_main_function_analyzed`, `test_full_config_analysis`, `test_minimal_config_analysis`, `test_buffer_overflow_function`, `test_safe_loop_access_function`, `test_complex_arithmetic_function`, `test_cascading_bounds_function` — all load "integration" fixture and check `converged` + function exists, already subsumed by `test_all_fixtures_analyzable`.

### A2: pta_verification_e2e.rs (35 → ~15 tests)

**Remove overlapping tests:**

- Phase 1 (3 → 2): Keep `phase1_constraint_extraction_covers_all_operations`, `phase1_interprocedural_constraints_extracted`. Remove `phase1_constraint_determinism` (determinism tested in phase8).

- Phase 2 (4 → 2): Keep `phase2_pointer_cycle_terminates`, `phase2_worklist_bounded_iterations`. Remove `phase2_worklist_order_deterministic` (tested in phase8), `phase2_explicit_pointer_cycles` (same pattern as `pointer_cycle_terminates` with different fixture).

- Phase 3 (4 → 2): Keep `phase3_identity_wrapper_contexts`, `phase3_nested_wrapper_requires_higher_k`. Remove `phase3_context_separation_for_wrapper` (weaker than identity_wrapper), `phase3_context_k_affects_precision` (subsumed by nested_wrapper), `phase3_factory_pattern_contexts` (just checks context_count > 0).

- Phase 4 (5 → 2): Keep `phase4_self_recursive_terminates`, `phase4_bounded_context_growth`. Remove `phase4_mutual_recursion_terminates`, `phase4_three_way_cycle_terminates`, `phase4_recursive_allocation_terminates` — all three just check `!iteration_limit_hit`, which is a weaker assertion than `bounded_context_growth` which tests all k values.

- Phase 5 (4 → 2): Keep `phase5_dedicated_flow_sensitive_fixture`, `phase5_strong_update_conditions_unit_tested`. Remove `phase5_strong_update_on_singleton` (uses wrong fixture, weaker assertions), `phase5_strong_vs_weak_tracking` (subset of dedicated_flow_sensitive_fixture).

- Phase 6 (5 → 2): Keep `phase6_inout_propagation_through_blocks`, `phase6_loop_propagation`. Remove `phase6_branch_merge_propagation` (same fixture, same assertion pattern), `phase6_interprocedural_propagation` (same), `phase6_deterministic_propagation` (tested in phase8).

- Phase 7 (4 → 2): Keep `phase7_cha_from_cpp_module`, `phase7_cha_export_format`. Remove `phase7_cha_from_llvm_module` (C module, minimal CHA), `phase7_cha_deterministic` (trivial).

- Phase 8 (6 → 3): Keep `phase8_full_pipeline_c_program`, `phase8_all_fixtures_pass`, `phase8_determinism_full_pipeline`. Remove `phase8_full_pipeline_cpp_program` (redundant with all_fixtures_pass), `phase8_cross_analysis_consistency` (subset of full_pipeline).

### A3: mssa_verification_e2e.rs (35 → ~15 tests)

**Remove tests with weak assertions (`node_count > 0`, `access_count > 0`):**

- Phase 1 (4 → 2): Keep `test_diamond_phi_placement`, `test_phi_operand_count_matches_predecessors`. Remove `test_loop_phi_placement` (same pattern), `test_nested_loop_phi_placement` (same pattern).

- Phase 2 (4 → 1): Keep `test_diamond_cfg_dominance`. Remove `test_entry_dominates_all` (just checks blocks exist in CFG), `test_loop_cfg_dominance` (just asserts access_count > 0), `test_nested_conditional_dominance` (same weak assertion).

- Phase 3 (4 → 2): Keep `test_self_recursive_modref`, `test_three_way_scc_modref`. Remove `test_mutual_recursion_modref` (similar to three_way), `test_transitive_modref` (just asserts access_count > 0).

- Phase 4 (5 → 3): Keep `test_no_alias_clobber`, `test_must_alias_clobber`, `test_clobber_with_phi`. Remove `test_clobber_chain_most_recent` (just checks def_count >= 3), `test_live_on_entry_clobber` (just checks sentinel exists).

- Phase 5 (5 → 2): Keep `test_simple_indirect_def_edge`, `test_no_edge_different_locations`. Remove `test_multiple_stores_single_load`, `test_single_store_multiple_loads`, `test_interleaved_store_load` — all three just assert `node_count > 0`.

- Phase 6 (5 → 1): Keep `test_simple_mem_phi_edges`. Remove `test_nested_phi_flow_edges`, `test_loop_mem_phi_edges`, `test_sequential_mem_phi`, `test_three_way_mem_phi` — all four just assert `node_count > 0`.

- Phase 7 (8 → 4): Keep `test_cpp_class_field_flow`, `test_mssa_svfg_determinism`, `test_svfg_reachability_query`, `test_all_fixtures_pass`. Remove `test_rust_unsafe_pointer_flow` (ignored), `test_mssa_export_json`, `test_svfg_export_json` (export tested elsewhere), `test_cross_analysis_consistency` (just asserts non-zero counts).

### A4: ifds_verification_e2e.rs (27 → ~14 tests)

- Phase 1 (3 → 2): Keep `ifds_zero_fact_reaches_both_branches`, `ifds_zero_fact_reaches_loop_body`. Remove `ifds_zero_fact_reaches_nested_branches` (similar pattern).

- Phase 2 (4 → 3): Keep `ifds_summary_edge_reuse`, `ifds_sanitizer_summary`, `ifds_multi_level_summary`. Remove `ifds_summary_multiple_callers` (weaker version of reuse test).

- Phase 3 (3 → 2): Keep `ide_edge_function_composition_order`, `ide_edge_function_join`. Remove `ide_all_top_all_bottom_composition` (basic top/bottom, likely tested in unit tests).

- Phase 4-5 (5 → 2): Keep `ide_jump_function_computation`, `ide_typestate_double_close`. Remove `ide_produces_facts` (subset of jump function), `ide_value_propagation_basic` (subset of jump function), `ide_typestate_file_leak` (weak assertion — `if leak_finding.is_some() { /* ok */ }`).

- Phase 6 (12 → 5): Keep `ifds_recursive_terminates`, `ifds_struct_field_taint`, `ifds_diamond_flow`, `ifds_results_deterministic`, `ifds_full_pipeline`. Remove `ifds_mutual_recursion_terminates` (just checks facts non-empty), `ifds_early_return_taint` (similar to diamond_flow), `ifds_pointer_alias_taint` (just checks facts non-empty), `ide_results_deterministic` (merge with ifds_results_deterministic), `ide_cpp_class_typestate` (just checks jump_fn_updates > 0), `ifds_diagnostics_populated` (subset of full_pipeline), `ide_diagnostics_populated` (subset).

---

## Phase B: Feature E2E Test Consolidation (~50 tests removed)

### B1: z3_enhanced_e2e.rs (20 → 8 tests)

Remove 12 "load-only" tests that have no assertions:
- `z3_assertion_provable_loads`, `z3_assertion_failing_loads`, `z3_reach_infeasible_loads`, `z3_reach_feasible_loads`, `z3_alias_disjoint_loads`, `z3_alias_confirmed_loads`, `z3_typestate_guarded_loads`, `z3_typestate_genuine_leak_loads`, `z3_numeric_guarded_loads`, `z3_numeric_genuine_overflow_loads`, `z3_ifds_correlated_branch_loads`, `z3_ifds_genuine_taint_loads`, `z3_vf_sanitized_path_loads`, `z3_vf_confirmed_flow_loads`

These only do `let _module = load_ll_fixture(...)` with zero assertions. The fixture loading is already exercised by tests that actually run analysis.

### B2: absint_e2e.rs (31 → ~18 tests)

Remove `_builds` tests that are subsumed by checker tests on the same fixture:
- `absint_buffer_overflow_builds` (subsumed by `buffer_overflow_checker_runs`)
- `absint_integer_overflow_builds` (subsumed by `integer_overflow_checker_runs`)
- `absint_div_by_zero_builds` (subsumed by `division_by_zero_checker_runs`)
- `absint_shift_count_builds` (subsumed by `shift_count_checker_runs`)
- `absint_memcpy_overflow_builds` (subsumed by `memcpy_overflow_checker_runs`)
- `absint_without_threshold_widening` (only checks `converged`, already tested in verification)
- `absint_with_limited_narrowing` (same)
- `absint_analyzes_all_functions` (weak assertion)

Also merge: `interprocedural_call_result_refined` and `interprocedural_parameter_binding` — they test the same thing (add_one(10) returns 11) with identical code.

Remove ~13 tests.

### B3: pathsens_e2e.rs (12 → 7 tests)

Remove load-only tests:
- `ps_null_guard_loads_and_builds`
- `ps_true_positive_loads_and_builds`
- `ps_multi_condition_loads_and_builds`
- `ps_correlated_branch_loads_and_builds`
- `ps_error_path_leak_loads_and_builds`

Keep all tests with real assertions.

### B4: Other E2E files (minor reductions)

- **memory_e2e.rs** (9 → 6): Remove `_loads_and_builds` variants that are subsumed by the graph building tests.
- **oop_e2e.rs** (4 → 3): Remove pure load test.
- **checker_e2e.rs** (17): Keep as-is — these have specific behavioral assertions.
- **dda_e2e.rs** (11): Keep as-is — each tests a different DDA feature.

---

## Phase C: PtsSet Inline Test Consolidation (~60 tests removed)

### C1: bdd.rs (38 → ~8 tests)

Remove 24 tests that duplicate cross_impl_tests.rs:
- All 12 property tests (insert_equivalence, union_equivalence, etc.) — superseded by cross_impl_tests.rs which tests all three impls simultaneously
- 12 standard PtsSet trait unit tests (empty_set, singleton_set, insert_new/duplicate, remove, contains, iter_sorted, union, intersect, difference, intersects, is_subset, from_iterator, hash, equality tests) — identical to btree.rs/bitvec.rs

Keep: BDD-specific tests (bits_needed, encode_decode, contains_index, dynamic_growth, large_set, to_and_from_btreeset, iter_empty_set) + 1 smoke test for basic operations.

### C2: bitvec.rs (34 → ~6 tests)

Remove 28 tests:
- All 12 property tests (same as bdd.rs)
- 16 standard PtsSet trait unit tests identical to btree.rs/bdd.rs

Keep: BitVec-specific tests (with_shared_indexer, large_set, capacity growth, iter_empty_set, to_and_from_btreeset) + 1 smoke test.

### C3: btree.rs (20 → ~3 tests)

Remove 17 standard PtsSet trait tests — all tested by cross_impl_tests.rs.

Keep: `into_inner`, `to_and_from_btreeset`, 1 smoke test for basic insert/contains.

### C4: config.rs (10 → 4 tests)

Remove 6 tests that duplicate config_tests.rs:
- `pts_representation_from_str` (duplicated in config_tests.rs)
- `pts_representation_as_str` (duplicated)
- `select_by_count_small/medium/large` (duplicated)
- `pts_config_with_methods` (duplicated)

Keep: `pts_representation_default_is_auto`, `pts_config_default`, `pts_config_builders`, `select_by_count_custom_thresholds`.

### C5: edge_case_tests.rs (17 → ~12 tests)

Remove 5 tests covered by cross_impl_tests.rs:
- `deterministic_iteration_order_btree/bitvec/bdd` (3 tests — iteration order equivalence tested in cross_impl)
- `hash_consistency_same_contents` (tested in cross_impl)
- `iteration_order_independent_of_insertion_order` (tested in cross_impl)

Keep all impl-specific edge case tests.

---

## Phase D: Python Test Consolidation (~160 tests removed)

### D1: Delete entire files (replace with 1 smoke test each)

These files are 85-95% mirrors of Rust tests. Replace each with 1 smoke test verifying the PyO3 binding works:

- **test_pts_repr.py** (21 → 1): Keep 1 test that runs PTA with `pts_repr="btreeset"` and verifies result has `value_count`.
- **test_fspta.py** (12 → 1): Keep 1 test that runs FS-PTA and verifies diagnostics dict has expected keys.
- **test_dda.py** (13 → 1): Keep 1 test that runs DDA and verifies diagnostics.
- **test_svfg.py** (13 → 1): Keep 1 test that builds SVFG and verifies node_count.
- **test_mssa.py** (10 → 1): Keep 1 test that builds MSSA and verifies access_count.
- **e2e/test_integer_info_e2e.py** (7 → 0): Delete entirely — all covered by Rust.

### D2: Heavy reductions

- **test_ifds.py** (10 → 3): Keep `test_simple_taint_reaches_sink`, `test_diagnostics_returns_dict`, `test_export_returns_dict`. Remove 7 tests that mirror Rust ifds_e2e.rs.

- **test_checkers.py** (27 → 10): Keep custom checker tests, resource_table tests, CheckerFinding property tests, schema tests. Remove individual `test_check_X()` tests (5 tests), `test_check_list/all` (2 tests), generic diagnostic tests.

- **test_path_sensitive.py** (18 → 5): Keep z3_timeout config test, filter_infeasible test, feasibility categorization test, diagnostics test, determinism test. Remove 13 fixture-loading/running tests.

- **test_ide.py** (18 → 5): Keep custom TypestateSpec creation, TypestateFinding properties, error handling. Remove 13 "runs on fixture" tests.

- **test_cg_refinement.py** (14 → 5): Keep ClassHierarchy API tests, resolved_sites export. Remove "builds" tests.

- **test_absint.py** (15 → 4): Keep custom checker filtering, interprocedural test. Remove diagnostic-only tests.

- **test_acceptance.py** (32 → 20): Keep schema validation, Finding properties, Selector combinators, error handling. Remove trivial "returns object" tests.

### D3: E2E reductions

- **e2e/test_taint_e2e.py** (12 → 4): Keep CWE-78 flow test, Finding.to_dict format, determinism, multi-hop trace. Remove redundant CWE tests.
- **e2e/test_memory_e2e.py** (9 → 2): Keep use-after-free with flow proof, null deref. Remove load-only tests.
- **e2e/test_oop_e2e.py** (8 → 2): Keep vtable dispatch, trait object. Remove load-only tests.
- **e2e/test_multi_module_e2e.py** (7 → 2): Keep cross-module flow with trace, callback chain. Remove load-only tests.

---

## Expected Results

| Category | Before | After | Removed |
|----------|--------|-------|---------|
| Verification E2E (Rust) | 152 | ~62 | ~90 |
| Feature E2E (Rust) | ~180 | ~140 | ~40 |
| PtsSet inline (Rust) | 166 | ~100 | ~66 |
| Other inline (Rust) | ~1,115 | ~1,115 | 0 |
| Python tests | 248 | ~60 | ~188 |
| **Total** | **~1,861** | **~1,477** | **~384** |

**~20% overall reduction** while maintaining all meaningful regression coverage.

## Verification

After each phase:
1. `make test` — all remaining tests pass
2. `make lint` — no new warnings
3. Spot-check that no tests with specific behavioral assertions were removed (only convergence/non-empty/load-only tests)

## Implementation Order

1. Phase A (verification E2E) — highest impact, most redundancy
2. Phase C (PtsSet inline) — straightforward mechanical removal
3. Phase B (feature E2E) — moderate impact
4. Phase D (Python) — highest count reduction but requires Docker

Each phase is independently committable.
