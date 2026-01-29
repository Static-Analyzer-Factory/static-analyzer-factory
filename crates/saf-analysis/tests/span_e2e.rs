//! E2E tests for source line propagation (Plan 121).
//!
//! Verifies that LLVM debug metadata flows through the pipeline:
//! LLVM IR → `AirModule` spans → `PropertyGraph` node properties.
//!
//! Source: `tests/programs/c/debug_info.c` (compiled with `-g`)
//! Fixture: `tests/fixtures/llvm/e2e/debug_info.ll`

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::export::PropertyGraph;
use saf_analysis::{ValueFlowConfig, ValueFlowMode, build_valueflow, to_property_graph};
use saf_test_utils::{load_ll_bundle, load_ll_fixture};

// =========================================================================
// Task 6: LLVM Frontend span extraction
// =========================================================================

#[test]
fn debug_info_function_spans_populated() {
    let module = load_ll_fixture("debug_info");

    // At least one defined function should have a span
    let func_with_span = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .any(|f| f.span.is_some());
    assert!(
        func_with_span,
        "No function spans found — debug info not extracted"
    );

    // Verify span has reasonable line numbers (> 0)
    for func in &module.functions {
        if let Some(span) = &func.span {
            assert!(span.line_start > 0, "Function span has line 0");
        }
    }
}

#[test]
fn debug_info_instruction_spans_populated() {
    let module = load_ll_fixture("debug_info");

    // At least one instruction should have a span
    let inst_with_span = module.functions.iter().any(|f| {
        f.blocks
            .iter()
            .any(|b| b.instructions.iter().any(|i| i.span.is_some()))
    });
    assert!(
        inst_with_span,
        "No instruction spans found — debug info not extracted"
    );

    // Verify instruction spans have reasonable line numbers
    for func in &module.functions {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(span) = &inst.span {
                    assert!(span.line_start > 0, "Instruction span has line 0");
                }
            }
        }
    }
}

#[test]
fn debug_info_source_files_tracked() {
    let bundle = load_ll_bundle("debug_info");
    let module = &bundle.module;

    // The fixture was compiled from a C file, so source_files should be non-empty
    assert!(
        !module.source_files.is_empty(),
        "No source files tracked from debug info"
    );
}

#[test]
fn no_debug_info_spans_none() {
    // Load a fixture compiled without -g (checker_no_leak has no debug info)
    let module = load_ll_fixture("checker_no_leak");

    // All function spans should be None (no debug info)
    for func in &module.functions {
        if !func.is_declaration {
            assert!(
                func.span.is_none(),
                "Function '{}' should have no span without debug info",
                func.name
            );
        }
    }
}

// =========================================================================
// Task 7: PropertyGraph span properties
// =========================================================================

#[test]
fn cfg_property_graph_nodes_have_spans() {
    let bundle = load_ll_bundle("debug_info");
    let module = &bundle.module;

    let func = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration)
        .expect("main function not found");
    let cfg = Cfg::build(func);
    let pg = cfg.to_pg(func, &module.source_files, None);

    // At least one node should have a span property
    let has_span = pg.nodes.iter().any(|n| n.properties.contains_key("span"));
    assert!(has_span, "CFG PropertyGraph nodes missing span properties");

    verify_span_structure(&pg);
}

#[test]
fn callgraph_property_graph_nodes_have_spans() {
    let bundle = load_ll_bundle("debug_info");
    let module = &bundle.module;

    let callgraph = CallGraph::build(module);
    let pg = callgraph.to_pg(module, None);

    // At least one Function node should have a span
    let has_span = pg
        .nodes
        .iter()
        .any(|n| n.labels.contains(&"Function".to_string()) && n.properties.contains_key("span"));
    assert!(
        has_span,
        "CallGraph PropertyGraph function nodes missing span properties"
    );

    verify_span_structure(&pg);
}

#[test]
fn defuse_property_graph_nodes_have_spans() {
    let bundle = load_ll_bundle("debug_info");
    let module = &bundle.module;

    let defuse = DefUseGraph::build(module);
    let pg = defuse.to_pg(module, None);

    // At least one node should have a span
    let has_span = pg.nodes.iter().any(|n| n.properties.contains_key("span"));
    assert!(
        has_span,
        "DefUse PropertyGraph nodes missing span properties"
    );

    verify_span_structure(&pg);
}

#[test]
fn valueflow_property_graph_nodes_have_spans() {
    let bundle = load_ll_bundle("debug_info");
    let module = &bundle.module;

    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);
    let vf_config = ValueFlowConfig {
        mode: ValueFlowMode::Fast,
        ..ValueFlowConfig::default()
    };
    let vfg = build_valueflow(&vf_config, module, &defuse, &callgraph, None);
    let pg = to_property_graph(&vfg, module, None);

    // At least one node should have a span
    let has_span = pg.nodes.iter().any(|n| n.properties.contains_key("span"));
    assert!(
        has_span,
        "ValueFlow PropertyGraph nodes missing span properties"
    );

    verify_span_structure(&pg);
}

// =========================================================================
// Task 8: Differential test — verify actual line numbers match C source
// =========================================================================

/// Verify extracted line numbers match the known C source structure.
///
/// The debug_info.ll fixture was compiled from:
/// ```c
/// int add(int a, int b) { return a + b; }    // line 1
/// int main() { return add(1, 2); }            // line 2
/// ```
#[test]
fn span_line_numbers_match_c_source() {
    let bundle = load_ll_bundle("debug_info");
    let module = &bundle.module;

    // Verify function declaration lines
    let add_func = module
        .functions
        .iter()
        .find(|f| f.name == "add")
        .expect("add function not found");
    let main_func = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration)
        .expect("main function not found");

    assert_eq!(
        add_func.span.as_ref().map(|s| s.line_start),
        Some(1),
        "add() should be declared on line 1"
    );
    assert_eq!(
        main_func.span.as_ref().map(|s| s.line_start),
        Some(2),
        "main() should be declared on line 2"
    );

    // Verify instruction lines fall within the expected source lines
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let expected_line = if func.name == "add" { 1 } else { 2 };
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(span) = &inst.span {
                    assert_eq!(
                        span.line_start, expected_line,
                        "Instruction in {}() should be on line {}, got {}",
                        func.name, expected_line, span.line_start
                    );
                }
            }
        }
    }

    // Verify PropertyGraph callgraph nodes carry correct function lines
    let callgraph = CallGraph::build(module);
    let pg = callgraph.to_pg(module, None);
    for node in &pg.nodes {
        if let Some(name) = node.properties.get("name") {
            let name = name.as_str().unwrap_or("");
            if let Some(span_val) = node.properties.get("span") {
                let line = span_val.as_object().unwrap()["line_start"]
                    .as_u64()
                    .unwrap();
                match name {
                    "add" => assert_eq!(line, 1, "add() callgraph node should show line 1"),
                    "main" => assert_eq!(line, 2, "main() callgraph node should show line 2"),
                    _ => {}
                }
            }
        }
    }
}

/// Verify that all span properties have the expected structure.
fn verify_span_structure(pg: &PropertyGraph) {
    for node in &pg.nodes {
        if let Some(span_val) = node.properties.get("span") {
            let span_obj = span_val.as_object().expect("span should be an object");
            assert!(
                span_obj.contains_key("line_start"),
                "span missing line_start"
            );
            let line = span_obj["line_start"].as_u64().unwrap();
            assert!(line > 0, "span line_start should be > 0");
        }
    }
}
