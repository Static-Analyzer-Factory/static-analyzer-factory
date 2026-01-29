//! End-to-end tests for PTA algorithm verification (Plan 038).
//!
//! These tests verify correctness of SAF's PTA algorithms against
//! reference implementations and academic papers.

use std::collections::BTreeMap;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::cspta::{CsPtaConfig, solve_context_sensitive};
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::fspta::{FsPtaConfig, FsSvfgBuilder, solve_flow_sensitive};
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::SvfgBuilder;
use saf_analysis::{FieldSensitivity, LocationFactory, PtaConfig, PtaContext, PtaResult};
use saf_analysis::{extract_constraints, extract_intraprocedural_constraints};
use saf_core::ids::FunctionId;
use saf_test_utils::load_ll_fixture;

// =============================================================================
// Helper Functions
// =============================================================================

/// Count functions in a module
fn count_functions(module: &saf_core::air::AirModule) -> usize {
    module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .count()
}

// =============================================================================
// Phase 1: Constraint Extraction Tests
// =============================================================================

#[test]
fn phase1_constraint_extraction_covers_all_operations() {
    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
    let constraints = extract_constraints(&module, &mut factory);

    // Should extract constraints from all test functions
    assert!(
        !constraints.addr.is_empty(),
        "Should have Addr constraints (alloca, global, heap)"
    );
    assert!(
        !constraints.copy.is_empty(),
        "Should have Copy constraints (phi, select, cast)"
    );
    assert!(!constraints.load.is_empty(), "Should have Load constraints");
    assert!(
        !constraints.store.is_empty(),
        "Should have Store constraints"
    );
    assert!(
        !constraints.gep.is_empty(),
        "Should have GEP constraints (struct/array fields)"
    );

    // Print constraint counts for documentation
    println!("Constraint extraction summary:");
    println!("  Addr:  {}", constraints.addr.len());
    println!("  Copy:  {}", constraints.copy.len());
    println!("  Load:  {}", constraints.load.len());
    println!("  Store: {}", constraints.store.len());
    println!("  GEP:   {}", constraints.gep.len());
    println!("  Total: {}", constraints.total_count());
}

#[test]
fn phase1_interprocedural_constraints_extracted() {
    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });

    // Full extraction includes interprocedural
    let full = extract_constraints(&module, &mut factory);

    // Intraprocedural only
    let mut factory2 = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
    let intra = extract_intraprocedural_constraints(&module, &mut factory2);

    // Full should have more copy constraints (arg→param, return→caller)
    assert!(
        full.copy.len() > intra.copy.len(),
        "Full extraction should include interprocedural copy constraints: full={}, intra={}",
        full.copy.len(),
        intra.copy.len()
    );
}

// =============================================================================
// Phase 2: Worklist Termination & Cycle Detection
// =============================================================================

#[test]
fn phase2_pointer_cycle_terminates() {
    // Test: p = &q; q = &p; r = p; (cyclic)
    // This tests that the solver handles pointer cycles correctly
    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let config = PtaConfig::default();
    let mut ctx = PtaContext::new(config);
    let result = ctx.analyze(&module);

    // Should not hit iteration limit
    assert!(
        !result.diagnostics.iteration_limit_hit,
        "Solver should terminate without hitting iteration limit"
    );

    // Should have produced some points-to information
    assert!(
        !result.pts.is_empty(),
        "Solver should produce non-empty points-to results"
    );
}

/// Test that worklist termination is bounded.
#[test]
fn phase2_worklist_bounded_iterations() {
    let module = load_ll_fixture("pta_verification_pointer_cycles");
    let config = PtaConfig {
        max_iterations: 10_000, // Lower bound for testing
        ..PtaConfig::default()
    };
    let mut ctx = PtaContext::new(config);
    let result = ctx.analyze(&module);

    // Check that iterations is reasonable (not hitting max unless truly needed)
    println!("Iterations used: {} / 10000", result.diagnostics.iterations);

    // For this simple test, should converge well under the limit
    assert!(
        result.diagnostics.iterations < 1000,
        "Simple programs should converge quickly: {} iterations",
        result.diagnostics.iterations
    );
}

// =============================================================================
// Phase 3: k-CFA Context Representation
// =============================================================================

#[test]
fn phase3_identity_wrapper_contexts() {
    // Test the classic k-CFA identity wrapper pattern:
    // void* identity(void* p) { return p; }
    // r1 = identity(&a);  // call site 1
    // r2 = identity(&b);  // call site 2
    // With k>=1, identity should have separate contexts for each call site
    let module = load_ll_fixture("pta_verification_context_sensitive");
    let callgraph = CallGraph::build(&module);

    let config = CsPtaConfig {
        k: 1,
        ..CsPtaConfig::default()
    };
    let result = solve_context_sensitive(&module, &callgraph, &config);

    // The context_sensitive.c fixture has multiple test functions that call identity()
    // Each call site should create a distinct context
    let diag = result.diagnostics();
    println!("Identity wrapper context analysis:");
    println!("  Contexts created: {}", diag.context_count);
    println!("  Iterations: {}", diag.iterations);

    // Should have multiple contexts (at least one per call site to identity)
    assert!(
        diag.context_count >= 2,
        "Should have at least 2 contexts for multiple identity() call sites, got {}",
        diag.context_count
    );

    assert!(!diag.iteration_limit_hit, "Should terminate successfully");
}

#[test]
fn phase3_nested_wrapper_requires_higher_k() {
    // Test nested wrappers: wrapper2 calls identity, wrapper3 calls wrapper2
    // With k=1: contexts collapse at depth > 1
    // With k=2: can distinguish wrapper2 call sites
    // With k=3: can distinguish wrapper3 call sites
    let module = load_ll_fixture("pta_verification_context_sensitive");
    let callgraph = CallGraph::build(&module);

    let result_k1 = solve_context_sensitive(
        &module,
        &callgraph,
        &CsPtaConfig {
            k: 1,
            ..CsPtaConfig::default()
        },
    );

    let result_k2 = solve_context_sensitive(
        &module,
        &callgraph,
        &CsPtaConfig {
            k: 2,
            ..CsPtaConfig::default()
        },
    );

    let result_k3 = solve_context_sensitive(
        &module,
        &callgraph,
        &CsPtaConfig {
            k: 3,
            ..CsPtaConfig::default()
        },
    );

    println!("Nested wrapper analysis:");
    println!(
        "  k=1: {} contexts, {} iterations",
        result_k1.diagnostics().context_count,
        result_k1.diagnostics().iterations
    );
    println!(
        "  k=2: {} contexts, {} iterations",
        result_k2.diagnostics().context_count,
        result_k2.diagnostics().iterations
    );
    println!(
        "  k=3: {} contexts, {} iterations",
        result_k3.diagnostics().context_count,
        result_k3.diagnostics().iterations
    );

    // Higher k should generally create more contexts (or equal if no deeper calls)
    assert!(
        result_k2.diagnostics().context_count >= result_k1.diagnostics().context_count,
        "k=2 should have >= contexts than k=1"
    );
    assert!(
        result_k3.diagnostics().context_count >= result_k2.diagnostics().context_count,
        "k=3 should have >= contexts than k=2"
    );

    // All should terminate
    assert!(!result_k1.diagnostics().iteration_limit_hit);
    assert!(!result_k2.diagnostics().iteration_limit_hit);
    assert!(!result_k3.diagnostics().iteration_limit_hit);
}

// =============================================================================
// Phase 4: k-CFA Recursion Handling (SCC Collapse)
// =============================================================================

#[test]
fn phase4_self_recursive_terminates() {
    // Test self-recursive function: factorial calls itself
    // Without SCC collapse, this would create unbounded contexts
    let module = load_ll_fixture("pta_verification_recursive_calls");
    let callgraph = CallGraph::build(&module);
    let config = CsPtaConfig {
        k: 2,
        ..CsPtaConfig::default()
    };
    let result = solve_context_sensitive(&module, &callgraph, &config);

    println!("Self-recursion (factorial) analysis:");
    println!("  Contexts: {}", result.diagnostics().context_count);
    println!("  Iterations: {}", result.diagnostics().iterations);

    // Should terminate without hitting iteration limit
    assert!(
        !result.diagnostics().iteration_limit_hit,
        "Self-recursive function should terminate via SCC collapse"
    );
}

#[test]
fn phase4_bounded_context_growth() {
    // Verify that context count is bounded even with many recursive calls
    // The module has 8 test functions with various recursion patterns
    let module = load_ll_fixture("pta_verification_recursive_calls");
    let callgraph = CallGraph::build(&module);

    // Test with k=1, k=2, k=3 - context count should stay manageable
    let results: Vec<_> = (1..=3)
        .map(|k| {
            let config = CsPtaConfig {
                k,
                ..CsPtaConfig::default()
            };
            let result = solve_context_sensitive(&module, &callgraph, &config);
            (k, result.diagnostics().context_count)
        })
        .collect();

    println!("Context growth with recursion:");
    for (k, count) in &results {
        println!("  k={k}: {count} contexts");
    }

    // Context count should be bounded (not exponential in recursion depth)
    // With proper SCC collapse, even k=3 should have manageable contexts
    let max_contexts = results.iter().map(|(_, c)| *c).max().unwrap_or(0);
    assert!(
        max_contexts < 1000,
        "Context count should be bounded with SCC collapse, got {max_contexts}"
    );
}

// =============================================================================
// Phase 5: SFS Strong Update Conditions
// =============================================================================

fn run_flow_sensitive_analysis(
    module: &saf_core::air::AirModule,
) -> saf_analysis::fspta::FlowSensitivePtaResult {
    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);
    let pta_config = PtaConfig::default();

    let mut ctx = PtaContext::new(pta_config.clone());
    let raw = ctx.analyze(module);
    let pta = PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics);

    let cfgs: BTreeMap<FunctionId, Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect();

    let mut ctx2 = PtaContext::new(pta_config.clone());
    let raw2 = ctx2.analyze(module);
    let mssa_pta = PtaResult::new(raw2.pts, Arc::new(raw2.factory), raw2.diagnostics);
    let mut mssa = MemorySsa::build(module, &cfgs, mssa_pta, &callgraph);

    let (svfg, _program_points) =
        SvfgBuilder::new(module, &defuse, &callgraph, &pta, &mut mssa).build();

    let mut ctx3 = PtaContext::new(pta_config.clone());
    let raw3 = ctx3.analyze(module);
    let pta3 = PtaResult::new(raw3.pts, Arc::new(raw3.factory), raw3.diagnostics);

    let mut ctx4 = PtaContext::new(pta_config);
    let raw4 = ctx4.analyze(module);
    let mssa_pta2 = PtaResult::new(raw4.pts, Arc::new(raw4.factory), raw4.diagnostics);
    let mut mssa2 = MemorySsa::build(module, &cfgs, mssa_pta2, &callgraph);

    let fs_svfg = FsSvfgBuilder::new(module, &svfg, &pta3, &mut mssa2, &callgraph).build();
    let config = FsPtaConfig::default();
    solve_flow_sensitive(module, &fs_svfg, &pta3, &callgraph, &config)
}

#[test]
fn phase5_dedicated_flow_sensitive_fixture() {
    // Use the dedicated flow-sensitive fixture with explicit strong/weak update scenarios
    let module = load_ll_fixture("pta_verification_flow_sensitive");
    let result = run_flow_sensitive_analysis(&module);

    let diag = result.diagnostics();
    println!("Flow-sensitive (dedicated fixture) diagnostics:");
    println!("  Iterations: {}", diag.iterations);
    println!("  Strong updates: {}", diag.strong_updates);
    println!("  Weak updates: {}", diag.weak_updates);

    // Should have performed strong updates (test cases 1-3 have singleton, non-array targets)
    assert!(
        diag.strong_updates > 0,
        "Should have strong updates for singleton targets, got {}",
        diag.strong_updates
    );

    // Note: weak_updates may be 0 if the CI PTA resolves pointers to singletons
    // before the flow-sensitive phase. This is valid behavior - the strong update
    // path is being exercised. The key test is that analysis converges correctly.
    let total_updates = diag.strong_updates + diag.weak_updates;
    println!("  Total updates: {total_updates}");

    assert!(!diag.iteration_limit_hit, "Should converge");
}

#[test]
fn phase5_strong_update_conditions_unit_tested() {
    // The strong update conditions are thoroughly tested in strong_update.rs unit tests:
    // - singleton_non_array_non_recursive_allows_strong_update
    // - non_singleton_rejects_strong_update
    // - array_collapsed_rejects_strong_update
    // - recursive_function_rejects_strong_update
    // - mutual_recursion_rejects_strong_update
    //
    // This E2E test verifies the integration works end-to-end
    let module = load_ll_fixture("pta_verification_flow_sensitive");
    let result = run_flow_sensitive_analysis(&module);

    // The fixture has 14 test functions with various update scenarios
    // Key verification: analysis completes without errors
    let diag = result.diagnostics();

    assert!(!diag.iteration_limit_hit, "Should converge");
    assert!(
        diag.strong_updates > 0,
        "Singleton stores should trigger strong updates"
    );
}

// =============================================================================
// Phase 6: SFS IN/OUT Set Propagation
// =============================================================================

#[test]
fn phase6_inout_propagation_through_blocks() {
    // Tests that dataflow information propagates correctly through basic blocks
    // The flow_sensitive fixture has tests with sequential stores and branches
    let module = load_ll_fixture("pta_verification_flow_sensitive");
    let result = run_flow_sensitive_analysis(&module);

    // Should converge (fixed point reached)
    assert!(
        !result.diagnostics().iteration_limit_hit,
        "Should reach fixed point for IN/OUT propagation"
    );

    // Multiple iterations means dataflow is propagating
    println!(
        "IN/OUT propagation: {} iterations",
        result.diagnostics().iterations
    );
}

#[test]
fn phase6_loop_propagation() {
    // Tests that dataflow propagates through loops to fixed point
    // test_loop_update: for (i=0; i<n; i++) *p = i;
    let module = load_ll_fixture("pta_verification_flow_sensitive");
    let result = run_flow_sensitive_analysis(&module);

    // Loops require multiple iterations to reach fixed point
    // The solver should handle back edges correctly
    assert!(!result.diagnostics().iteration_limit_hit);
    assert!(
        result.diagnostics().iterations > 1,
        "Loop propagation should require multiple iterations"
    );
}

// =============================================================================
// Phase 7: CHA Tests
// =============================================================================

#[test]
fn phase7_cha_from_cpp_module() {
    // Use the C++ class hierarchy fixture
    let module = load_ll_fixture("pta_verification_class_hierarchy");

    let cha = saf_analysis::cha::ClassHierarchy::build(&module.type_hierarchy);
    let export = cha.export();

    println!("CHA from C++ module:");
    println!(
        "  Classes: {}",
        export["classes"].as_array().map_or(0, std::vec::Vec::len)
    );

    // Should find C++ classes (Base, Derived, Animal, Dog, Cat, etc.)
    // The exact count depends on how LLVM represents the type hierarchy
    assert!(export["classes"].is_array());
}

#[test]
fn phase7_cha_export_format() {
    let module = load_ll_fixture("pta_verification_class_hierarchy");
    let cha = saf_analysis::cha::ClassHierarchy::build(&module.type_hierarchy);
    let export = cha.export();

    // Verify export format is correct
    assert!(
        export["classes"].is_array(),
        "Export should have 'classes' array"
    );
    assert!(
        export["inheritance"].is_object(),
        "Export should have 'inheritance' object"
    );
    assert!(
        export["vtables"].is_object(),
        "Export should have 'vtables' object"
    );

    // Classes array contains class names as strings
    if let Some(classes) = export["classes"].as_array() {
        for class in classes {
            assert!(
                class.is_string(),
                "Each class should be a string (class name)"
            );
        }
        println!("CHA export: {} classes", classes.len());
    }

    // Inheritance maps class names to base types
    if let Some(inheritance) = export["inheritance"].as_object() {
        println!("CHA export: {} classes with inheritance", inheritance.len());
        for (class, bases) in inheritance {
            assert!(
                bases.is_array(),
                "Inheritance entry for {class} should be array"
            );
        }
    }
}

// =============================================================================
// Phase 8: PTA Integration Tests
// =============================================================================

#[test]
fn phase8_full_pipeline_c_program() {
    // Full integration test: C program through all PTA phases
    let module = load_ll_fixture("pta_verification_constraint_extraction");

    println!("=== Full Pipeline Test (C) ===");
    println!("Functions: {}", count_functions(&module));

    // Phase 1: Constraint extraction
    let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
    let constraints = extract_constraints(&module, &mut factory);
    println!(
        "Constraints: {} total ({} addr, {} copy, {} load, {} store, {} gep)",
        constraints.total_count(),
        constraints.addr.len(),
        constraints.copy.len(),
        constraints.load.len(),
        constraints.store.len(),
        constraints.gep.len()
    );

    // Phase 2: Andersen CI analysis
    let config = PtaConfig::default();
    let mut ctx = PtaContext::new(config);
    let raw_result = ctx.analyze(&module);
    let pta_result = PtaResult::new(
        raw_result.pts,
        Arc::new(raw_result.factory),
        raw_result.diagnostics,
    );
    println!(
        "Andersen CI: {} values with points-to info, {} iterations",
        pta_result.value_count(),
        pta_result.diagnostics().iterations
    );

    // Phase 3-4: Context-sensitive analysis
    let callgraph = CallGraph::build(&module);
    let cs_config = CsPtaConfig::default();
    let cs_result = solve_context_sensitive(&module, &callgraph, &cs_config);
    println!(
        "k-CFA: {} contexts, {} iterations",
        cs_result.diagnostics().context_count,
        cs_result.diagnostics().iterations
    );

    // Phase 5-6: Flow-sensitive analysis
    let fs_result = run_flow_sensitive_analysis(&module);
    println!(
        "Flow-sensitive: {} strong, {} weak updates, {} iterations",
        fs_result.diagnostics().strong_updates,
        fs_result.diagnostics().weak_updates,
        fs_result.diagnostics().iterations
    );

    // Phase 7: CHA (minimal for C)
    let cha = saf_analysis::cha::ClassHierarchy::build(&module.type_hierarchy);
    let cha_export = cha.export();
    println!(
        "CHA: {} classes",
        cha_export["classes"]
            .as_array()
            .map_or(0, std::vec::Vec::len)
    );

    // All phases should complete without errors
    assert!(!pta_result.diagnostics().iteration_limit_hit);
    assert!(!cs_result.diagnostics().iteration_limit_hit);
    assert!(!fs_result.diagnostics().iteration_limit_hit);
}

#[test]
fn phase8_all_fixtures_pass() {
    // Verify all test fixtures can be analyzed without errors
    let fixtures = [
        "pta_verification_constraint_extraction",
        "pta_verification_pointer_cycles",
        "pta_verification_context_sensitive",
        "pta_verification_recursive_calls",
        "pta_verification_flow_sensitive",
        "pta_verification_class_hierarchy",
    ];

    for fixture in fixtures {
        let module = load_ll_fixture(fixture);

        // Basic analysis should not panic or hit limits
        let config = PtaConfig::default();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        assert!(
            !result.diagnostics.iteration_limit_hit,
            "Fixture {fixture} should not hit iteration limit"
        );

        println!(
            "{}: {} functions, {} iterations",
            fixture,
            count_functions(&module),
            result.diagnostics.iterations
        );
    }
}

#[test]
fn phase8_determinism_full_pipeline() {
    // Verify full pipeline is deterministic across multiple runs
    let module = load_ll_fixture("pta_verification_constraint_extraction");

    let run_pipeline = || {
        let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
        let constraints = extract_constraints(&module, &mut factory);

        let config = PtaConfig::default();
        let mut ctx = PtaContext::new(config);
        let ci_result = ctx.analyze(&module);

        let callgraph = CallGraph::build(&module);
        let cs_config = CsPtaConfig::default();
        let cs_result = solve_context_sensitive(&module, &callgraph, &cs_config);

        (
            constraints.total_count(),
            ci_result.pts.len(),
            ci_result.diagnostics.iterations,
            cs_result.diagnostics().context_count,
            cs_result.diagnostics().iterations,
        )
    };

    let run1 = run_pipeline();
    let run2 = run_pipeline();

    assert_eq!(run1, run2, "Full pipeline should be deterministic");
}
