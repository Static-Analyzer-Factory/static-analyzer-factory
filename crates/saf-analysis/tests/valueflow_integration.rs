//! Integration tests for `ValueFlow` using AIR-JSON fixtures.

use std::collections::BTreeSet;
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::selector::{SelectorResolver, SinkSelector, SourceSelector};
use saf_analysis::{
    Finding, PtaConfig, PtaContext, PtaResult, QueryLimits, SarifExport, ValueFlowBuilder,
    ValueFlowConfig, ValueFlowExport, ValueFlowMode,
};
use saf_core::ids::ValueId;
use saf_test_utils::load_air_json_fixture as load_fixture;

fn make_pta_config() -> PtaConfig {
    PtaConfig::default()
}

fn make_valueflow_config() -> ValueFlowConfig {
    ValueFlowConfig::default()
}

fn build_analysis_stack(module: &saf_core::air::AirModule) -> (DefUseGraph, CallGraph, PtaResult) {
    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);
    let pta_config = make_pta_config();
    let mut pta_ctx = PtaContext::new(pta_config.clone());
    let pta_analysis = pta_ctx.analyze(module);
    let pta_result = PtaResult::new(
        pta_analysis.pts,
        Arc::new(pta_analysis.factory),
        pta_analysis.diagnostics,
    );
    (defuse, callgraph, pta_result)
}

#[test]
fn valueflow_minimal_fixture() {
    let module = load_fixture("minimal");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Minimal fixture has one function with a void return (no values defined)
    // Graph builds without error - that's the test
    let _ = graph.node_count(); // Verify build completed
}

#[test]
fn valueflow_memory_ops_fixture() {
    let module = load_fixture("memory_ops");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // memory_ops has allocas, loads, stores - should have nodes and edges
    assert!(
        graph.nodes().len() > 2,
        "memory_ops should have multiple nodes"
    );
    assert!(graph.edge_count() > 0, "memory_ops should have edges");
}

#[test]
fn valueflow_calls_fixture() {
    let module = load_fixture("calls");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // calls fixture has function calls - should have call edges
    assert!(!graph.nodes().is_empty(), "calls should have nodes");
}

#[test]
fn valueflow_fast_mode_no_pta() {
    let module = load_fixture("memory_ops");
    let defuse = DefUseGraph::build(&module);
    let callgraph = CallGraph::build(&module);

    let config = ValueFlowConfig {
        mode: ValueFlowMode::Fast,
        ..Default::default()
    };
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, None);
    let graph = builder.build();

    // Fast mode should still produce a graph
    assert!(!graph.nodes().is_empty(), "fast mode should produce nodes");
}

#[test]
fn valueflow_deterministic_graph_construction() {
    let module = load_fixture("memory_ops");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();

    // Build twice
    let builder1 = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph1 = builder1.build();

    let builder2 = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph2 = builder2.build();

    // Should be identical
    assert_eq!(
        graph1.nodes().len(),
        graph2.nodes().len(),
        "node counts should match"
    );
    assert_eq!(
        graph1.edge_count(),
        graph2.edge_count(),
        "edge counts should match"
    );

    // Nodes should be exactly the same
    assert_eq!(graph1.nodes(), graph2.nodes(), "nodes should be identical");
}

#[test]
fn valueflow_flows_query_basic() {
    let module = load_fixture("memory_ops");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Get value nodes using the iterator method
    let all_values: Vec<ValueId> = graph.value_nodes().collect();

    if all_values.len() >= 2 {
        let sources: BTreeSet<_> = [all_values[0]].into_iter().collect();
        let sinks: BTreeSet<_> = all_values.iter().skip(1).copied().collect();

        let limits = QueryLimits {
            max_depth: 10,
            max_paths: 100,
        };

        let flows = graph.flows(&sources, &sinks, &limits);
        // May or may not find flows depending on graph structure
        let _ = flows.len(); // Just verify it doesn't panic
    }
}

#[test]
fn valueflow_flows_deterministic() {
    let module = load_fixture("memory_ops");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Get all value nodes
    let all_values: Vec<ValueId> = graph.value_nodes().collect();

    if all_values.len() >= 2 {
        let sources: BTreeSet<_> = all_values.iter().take(1).copied().collect();
        let sinks: BTreeSet<_> = all_values.iter().skip(1).copied().collect();

        let limits = QueryLimits {
            max_depth: 5,
            max_paths: 10,
        };

        // Query twice
        let flows1 = graph.flows(&sources, &sinks, &limits);
        let flows2 = graph.flows(&sources, &sinks, &limits);

        // Results should be identical
        assert_eq!(flows1.len(), flows2.len(), "flow counts should match");

        // Traces should be identical
        for (f1, f2) in flows1.iter().zip(flows2.iter()) {
            assert_eq!(f1.source, f2.source, "sources should match");
            assert_eq!(f1.sink, f2.sink, "sinks should match");
            assert_eq!(
                f1.trace.steps.len(),
                f2.trace.steps.len(),
                "trace lengths should match"
            );
        }
    }
}

#[test]
fn valueflow_taint_flow_with_sanitizer() {
    let module = load_fixture("memory_ops");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Get all value nodes
    let all_values: Vec<ValueId> = graph.value_nodes().collect();

    if all_values.len() >= 3 {
        let sources: BTreeSet<_> = [all_values[0]].into_iter().collect();
        let sinks: BTreeSet<_> = [all_values[all_values.len() - 1]].into_iter().collect();
        let sanitizers: BTreeSet<_> = [all_values[1]].into_iter().collect();

        let limits = QueryLimits {
            max_depth: 10,
            max_paths: 100,
        };

        // Query with and without sanitizer
        let flows_with_sanitizer = graph.taint_flow(&sources, &sinks, &sanitizers, &limits);
        let flows_no_sanitizer = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);

        // Sanitizer should block some paths (or all)
        assert!(
            flows_with_sanitizer.len() <= flows_no_sanitizer.len(),
            "sanitizer should not increase findings"
        );
    }
}

#[test]
fn selector_resolve_with_module() {
    let module = load_fixture("calls");

    let resolver = SelectorResolver::new(&module);

    // Resolve all function parameters
    let source_selector = SourceSelector::AllParams;
    for selector in source_selector.to_selectors() {
        let result = resolver.resolve(&selector);
        assert!(result.is_ok(), "selector resolution should succeed");
    }

    // Resolve function returns
    let sink_selector = SinkSelector::FunctionReturns(vec!["main".to_string()]);
    for selector in sink_selector.to_selectors() {
        let result = resolver.resolve(&selector);
        assert!(result.is_ok(), "selector resolution should succeed");
    }
}

#[test]
fn selector_resolve_wildcard_pattern() {
    let module = load_fixture("calls");
    let resolver = SelectorResolver::new(&module);

    use saf_analysis::selector::Selector;

    // Match all functions with wildcard
    let selector = Selector::function_param("*", None);
    let result = resolver.resolve(&selector).unwrap();

    // Should match params from both functions in calls.air.json
    assert!(
        result.len() >= 2,
        "wildcard should match multiple params, got {}",
        result.len()
    );
}

#[test]
fn valueflow_export_json_deterministic() {
    let module = load_fixture("memory_ops");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Get some flows to export
    let all_values: Vec<ValueId> = graph.value_nodes().collect();

    if all_values.len() >= 2 {
        let sources: BTreeSet<_> = all_values.iter().take(1).copied().collect();
        let sinks: BTreeSet<_> = all_values.iter().skip(1).take(1).copied().collect();

        let limits = QueryLimits {
            max_depth: 5,
            max_paths: 10,
        };

        let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);

        // Convert flows to findings
        let findings: Vec<Finding> = flows
            .into_iter()
            .map(|f| Finding::from_flow(f, None))
            .collect();

        // Export to JSON twice
        let export = ValueFlowExport::new(&config, &limits, &findings, &module);
        let json1 = serde_json::to_string(&export).unwrap();

        let export2 = ValueFlowExport::new(&config, &limits, &findings, &module);
        let json2 = serde_json::to_string(&export2).unwrap();

        assert_eq!(json1, json2, "JSON exports should be identical");
    }
}

#[test]
fn valueflow_finding_id_deterministic() {
    let module = load_fixture("memory_ops");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Get some flows
    let all_values: Vec<ValueId> = graph.value_nodes().collect();

    if all_values.len() >= 2 {
        let sources: BTreeSet<_> = all_values.iter().take(1).copied().collect();
        let sinks: BTreeSet<_> = all_values.iter().skip(1).take(1).copied().collect();

        let limits = QueryLimits {
            max_depth: 5,
            max_paths: 10,
        };

        // Query twice and convert to findings
        let flows1 = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
        let flows2 = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);

        let findings1: Vec<Finding> = flows1
            .into_iter()
            .map(|f| Finding::from_flow(f, None))
            .collect();
        let findings2: Vec<Finding> = flows2
            .into_iter()
            .map(|f| Finding::from_flow(f, None))
            .collect();

        // Finding IDs should be identical
        let ids1: Vec<_> = findings1.iter().map(|f| f.id).collect();
        let ids2: Vec<_> = findings2.iter().map(|f| f.id).collect();

        assert_eq!(ids1, ids2, "finding IDs should be deterministic");
    }
}

#[test]
fn valueflow_sarif_export() {
    let module = load_fixture("memory_ops");
    let (defuse, callgraph, pta_result) = build_analysis_stack(&module);

    let config = make_valueflow_config();
    let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta_result));
    let graph = builder.build();

    // Get some flows
    let all_values: Vec<ValueId> = graph.value_nodes().collect();

    if !all_values.is_empty() {
        let sources: BTreeSet<_> = all_values.iter().take(1).copied().collect();
        let sinks: BTreeSet<_> = all_values.iter().skip(1).take(1).copied().collect();

        let limits = QueryLimits {
            max_depth: 5,
            max_paths: 10,
        };

        let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);

        // Convert flows to findings
        let findings: Vec<Finding> = flows
            .into_iter()
            .map(|f| Finding::from_flow(f, None))
            .collect();

        // Export to SARIF
        let sarif = SarifExport::from_findings(&findings, &module, "saf", Some("0.1.0"));
        let json = serde_json::to_string_pretty(&sarif).unwrap();

        // Validate basic SARIF structure
        assert!(json.contains("\"$schema\""), "SARIF should have schema");
        assert!(
            json.contains("\"version\": \"2.1.0\""),
            "SARIF should have version"
        );
        assert!(json.contains("\"runs\""), "SARIF should have runs");
    }
}
