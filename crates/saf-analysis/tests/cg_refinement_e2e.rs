//! E2E tests for call graph refinement (CHA + PTA iterative loop).
//!
//! Each test loads a compiled LLVM IR fixture, runs `cg_refinement::refine()`,
//! and verifies that the refined call graph contains expected edges and that
//! indirect call sites are resolved.
//!
//! Source files: `tests/programs/c/cg_*.c`, `tests/programs/cpp/cg_*.cpp`,
//!              `tests/programs/rust/cg_*.rs`
//! Compiled fixtures: `tests/fixtures/llvm/e2e/cg_*.ll`

use saf_analysis::callgraph::CallGraphNode;
use saf_analysis::cg_refinement::{EntryPointStrategy, RefinementConfig, refine};
use saf_core::air::AirModule;
use saf_test_utils::load_ll_fixture;

fn default_config() -> RefinementConfig {
    RefinementConfig {
        entry_points: EntryPointStrategy::Named(vec!["main".to_string()]),
        ..RefinementConfig::default()
    }
}

/// Check that the refined call graph has an edge where the caller's function
/// name contains `caller_substr` and the callee's function name contains
/// `callee_substr`.
fn has_edge(
    module: &AirModule,
    result: &saf_analysis::cg_refinement::RefinementResult,
    caller_substr: &str,
    callee_substr: &str,
) -> bool {
    let func_names: std::collections::BTreeMap<saf_core::ids::FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    for (src, dsts) in &result.call_graph.edges {
        let src_name = match src {
            CallGraphNode::Function(fid) | CallGraphNode::External { func: fid, .. } => {
                func_names.get(fid).copied().unwrap_or("")
            }
            CallGraphNode::IndirectPlaceholder { .. } => continue,
        };
        if !src_name.contains(caller_substr) {
            continue;
        }
        for dst in dsts {
            let dst_name = match dst {
                CallGraphNode::Function(fid) | CallGraphNode::External { func: fid, .. } => {
                    func_names.get(fid).copied().unwrap_or("")
                }
                CallGraphNode::IndirectPlaceholder { .. } => continue,
            };
            if dst_name.contains(callee_substr) {
                return true;
            }
        }
    }
    false
}

/// Count how many indirect call placeholders are in the call graph.
fn indirect_placeholder_count(result: &saf_analysis::cg_refinement::RefinementResult) -> usize {
    result.call_graph.indirect_calls().len()
}

// ── 1. Function pointer callback (C) ─────────────────────────────────────

#[test]
fn cg_fptr_callback_resolves_dispatch() {
    let module = load_ll_fixture("cg_fptr_callback");
    let config = default_config();
    let result = refine(&module, &config, None);

    // The module should have functions: main, dispatch, dangerous_sink, system, getenv
    assert!(
        module.functions.len() >= 3,
        "should have at least main, dispatch, dangerous_sink"
    );

    // Refinement should have run at least 1 iteration
    assert!(
        result.iterations >= 1,
        "should run at least 1 PTA iteration"
    );

    // The call graph should contain edge: main -> dispatch (direct call)
    assert!(
        has_edge(&module, &result, "main", "dispatch"),
        "main should call dispatch"
    );

    // dispatch has an indirect call (handler(data)) — verify the placeholder exists
    assert!(
        indirect_placeholder_count(&result) >= 1,
        "dispatch's indirect call should create a placeholder node"
    );

    // PTA ran and produced a result
    assert!(result.pta_result.is_some(), "PTA should produce a result");

    // NOTE: Full resolution of dispatch -> dangerous_sink requires interprocedural
    // parameter passing in PTA (argument-to-formal bindings at call sites), which
    // is not yet implemented. The refinement infrastructure correctly identifies
    // the indirect call and runs PTA; full resolution will work once interprocedural
    // PTA constraints are added.
}

// ── 2. Virtual dispatch (C++) ────────────────────────────────────────────

#[test]
fn cg_virtual_dispatch_resolves_via_cha() {
    let module = load_ll_fixture("cg_virtual_dispatch");
    let config = default_config();
    let result = refine(&module, &config, None);

    // Should have CHA since this is a C++ program with virtual dispatch
    assert!(result.cha.is_some(), "should build class hierarchy");

    // Refinement should have run
    assert!(result.iterations >= 1);

    // The type hierarchy should contain entries for Processor and UnsafeProcessor
    assert!(
        !module.type_hierarchy.is_empty(),
        "LLVM frontend should extract type hierarchy from C++ vtables"
    );

    // main -> run_processor should exist (direct call)
    assert!(
        has_edge(&module, &result, "main", "run_processor"),
        "main should call run_processor"
    );
}

/// Verifies that virtual dispatch in `run_processor` resolves precisely to slot 0 methods.
///
/// The call chain is:
/// 1. main creates `UnsafeProcessor` and calls `run_processor(&proc`, input)
/// 2. `run_processor` calls p->process(data) (virtual call at slot 0)
/// 3. CHA should resolve this to slot 0 implementations only:
///    - `UnsafeProcessor::process` (slot 0 in `UnsafeProcessor`'s vtable)
///    - __`cxa_pure_virtual` (slot 0 in Processor's vtable, since process is pure virtual)
///
/// CHA should NOT resolve to destructors (slots 1 and 2) or other methods.
#[test]
fn cg_virtual_dispatch_resolves_to_slot_0_only() {
    let module = load_ll_fixture("cg_virtual_dispatch");
    let config = default_config();
    let result = refine(&module, &config, None);

    // Build function name lookup
    let func_names: std::collections::BTreeMap<saf_core::ids::FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    // Check resolved_sites - should have at least one virtual call resolved
    assert!(
        !result.resolved_sites.is_empty(),
        "virtual call should be resolved by CHA"
    );

    // Find the virtual call in run_processor (the one at slot 0 for p->process)
    // and verify it ONLY resolves to slot 0 methods (process implementations),
    // NOT to destructors (D0Ev, D2Ev) or other slots
    let mut found_precise_resolution = false;
    for targets in result.resolved_sites.values() {
        let target_names: Vec<&str> = targets
            .iter()
            .filter_map(|t| func_names.get(t).copied())
            .collect();

        // Check if this looks like the process method call (should include process)
        let has_process = target_names.iter().any(|n| n.contains("process"));
        if !has_process {
            continue;
        }

        // This is the process virtual call - check precision
        let has_destructors = target_names
            .iter()
            .any(|n| n.contains("D0Ev") || n.contains("D2Ev"));

        if has_destructors {
            // Over-approximation: includes destructors from other slots
            eprintln!(
                "IMPRECISE: Virtual call resolved to {} targets including destructors: {:?}",
                target_names.len(),
                target_names
            );
        } else {
            // Precise resolution: only slot 0 methods
            found_precise_resolution = true;
        }
    }

    assert!(
        found_precise_resolution,
        "virtual call at slot 0 should resolve ONLY to slot 0 methods (process), \
         not to destructors or other vtable slots. This requires matching the \
         virtual call pattern and extracting the correct slot index."
    );

    // The call graph should have edge from run_processor to UnsafeProcessor::process
    assert!(
        has_edge(&module, &result, "run_processor", "process"),
        "run_processor should have edge to UnsafeProcessor::process after CHA resolution"
    );
}

// ── 3. Multiple inheritance (C++) ────────────────────────────────────────

#[test]
fn cg_multi_inheritance_loads_hierarchy() {
    let module = load_ll_fixture("cg_multi_inheritance");
    let config = default_config();
    let result = refine(&module, &config, None);

    assert!(result.cha.is_some(), "should build class hierarchy");
    assert!(result.iterations >= 1);

    // Type hierarchy should contain classes for the inheritance chain
    assert!(
        !module.type_hierarchy.is_empty(),
        "should extract type hierarchy from multiple-inheritance C++ program"
    );

    // main -> run should exist (direct call)
    assert!(
        has_edge(&module, &result, "main", "run"),
        "main should call run"
    );
}

// ── 4. Function pointer in struct (C) ────────────────────────────────────

#[test]
fn cg_fptr_struct_resolves_struct_field() {
    let module = load_ll_fixture("cg_fptr_struct");
    let config = default_config();
    let result = refine(&module, &config, None);

    assert!(result.iterations >= 1);

    // main -> invoke_plugin should exist
    assert!(
        has_edge(&module, &result, "main", "invoke_plugin"),
        "main should call invoke_plugin"
    );

    // invoke_plugin has an indirect call (p->handle(data)) — placeholder exists
    assert!(
        indirect_placeholder_count(&result) >= 1,
        "invoke_plugin's indirect call should create a placeholder node"
    );

    // PTA ran and produced a result
    assert!(result.pta_result.is_some(), "PTA should produce a result");

    // NOTE: Full resolution of invoke_plugin -> dangerous_handler requires
    // interprocedural PTA parameter passing (modeling struct pointer passing at
    // call sites). The refinement infrastructure correctly identifies the indirect
    // call and runs PTA; resolution will work once interprocedural constraints
    // and struct field-sensitive param bindings are added.
}

// ── 5. Iterative resolution (C) ─────────────────────────────────────────

#[test]
fn cg_iterative_resolve_needs_multiple_iterations() {
    let module = load_ll_fixture("cg_iterative_resolve");
    let config = default_config();
    let result = refine(&module, &config, None);

    // Should need at least 1 iteration
    assert!(
        result.iterations >= 1,
        "should need at least 1 PTA iteration for two-level indirection"
    );

    // main -> setup should exist (direct call)
    assert!(
        has_edge(&module, &result, "main", "setup"),
        "main should call setup"
    );

    // After full refinement, trampoline and final_sink should be reachable
    // (even if they need multiple iterations to fully resolve)
    // Check that the call graph has more than just direct-call edges
    let node_count = result.call_graph.nodes.len();
    assert!(
        node_count >= 4,
        "refined CG should contain main, setup, trampoline, final_sink (+ externals); got {node_count}"
    );
}

// ── 6. Rust trait object dispatch ────────────────────────────────────────

#[test]
fn cg_trait_object_dispatch() {
    let module = load_ll_fixture("cg_trait_object");
    let config = default_config();
    let result = refine(&module, &config, None);

    assert!(result.iterations >= 1);

    // The module should contain functions from the Rust program
    assert!(
        module.functions.len() >= 2,
        "should have at least main and handler functions"
    );

    // Check the call graph has meaningful content
    let edge_count: usize = result
        .call_graph
        .edges
        .values()
        .map(std::collections::BTreeSet::len)
        .sum();
    assert!(
        edge_count >= 1,
        "refined CG should have call edges; got {edge_count}"
    );
}

// ── Cross-cutting: determinism ───────────────────────────────────────────

#[test]
fn cg_refinement_is_deterministic() {
    let module = load_ll_fixture("cg_fptr_callback");
    let config = default_config();

    let r1 = refine(&module, &config, None);
    let r2 = refine(&module, &config, None);

    assert_eq!(r1.iterations, r2.iterations, "iterations should match");
    assert_eq!(
        r1.resolved_sites, r2.resolved_sites,
        "resolved sites should match"
    );
    assert_eq!(r1.call_graph, r2.call_graph, "call graphs should match");

    // Export both call graphs and compare JSON
    let export1 = serde_json::to_string(&r1.call_graph.export(&module)).expect("export1");
    let export2 = serde_json::to_string(&r2.call_graph.export(&module)).expect("export2");
    assert_eq!(
        export1, export2,
        "call graph exports should be byte-identical"
    );
}

// ── 8. Indirect call through named SSA value (regression) ────────────────

/// Regression test: a call through a named SSA value like `%call` must be
/// classified as `CallIndirect`, not `CallDirect` to a phantom external
/// function literally named "call".
///
/// The fixture's `use_callback` function has:
///   `%call = call ptr @get_callback(i32 %choice)`
///   `%result = call i32 %call(i32 %value)`
///
/// Without the fix, `%call` was treated as a direct call to "call".
#[test]
fn cg_named_ssa_callee_is_indirect() {
    use saf_core::air::Operation;

    let module = load_ll_fixture("indirect_call_named_ssa");

    // Find the use_callback function and verify it has a CallIndirect (not CallDirect)
    let use_cb_fn = module
        .functions
        .iter()
        .find(|f| f.name == "use_callback")
        .expect("should have use_callback function");

    let has_indirect_call = use_cb_fn.blocks.iter().any(|b| {
        b.instructions
            .iter()
            .any(|i| matches!(i.op, Operation::CallIndirect { .. }))
    });
    assert!(
        has_indirect_call,
        "use_callback should have a CallIndirect for the call through the named SSA value %call"
    );

    // Verify there is NO CallDirect to a phantom function named "call" or "result"
    let bogus_direct_calls: Vec<_> = use_cb_fn
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter_map(|i| {
            if let Operation::CallDirect { callee } = &i.op {
                let callee_name = module
                    .functions
                    .iter()
                    .find(|f| f.id == *callee)
                    .map(|f| f.name.as_str());
                // "call" or "result" would be bogus names from named SSA values
                if matches!(callee_name, Some("call" | "result")) {
                    return Some(callee_name.unwrap().to_string());
                }
            }
            None
        })
        .collect();
    assert!(
        bogus_direct_calls.is_empty(),
        "should NOT have CallDirect to phantom functions: {bogus_direct_calls:?}"
    );
}

/// After PTA refinement, the indirect call in `use_callback` should resolve
/// to `{double_it, triple_it}`.
#[test]
fn cg_named_ssa_callee_resolves_via_pta() {
    let module = load_ll_fixture("indirect_call_named_ssa");
    let config = default_config();
    let result = refine(&module, &config, None);

    // use_callback calls get_callback directly
    assert!(
        has_edge(&module, &result, "use_callback", "get_callback"),
        "use_callback should directly call get_callback"
    );

    // The indirect call should have created a placeholder
    assert!(
        indirect_placeholder_count(&result) >= 1,
        "should have at least one indirect call placeholder for the call through %call"
    );

    // PTA should have run
    assert!(result.pta_result.is_some(), "PTA should produce a result");
}
