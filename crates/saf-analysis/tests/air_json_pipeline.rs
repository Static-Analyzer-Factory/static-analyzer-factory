//! Integration tests for AIR-JSON fixtures through the analysis pipeline.
//!
//! These tests verify that AIR-JSON fixtures can be loaded and processed
//! through the various analysis stages (CFG, def-use, pipeline).

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::pipeline::{PipelineConfig, run_pipeline};
use saf_test_utils::load_air_json_fixture;

#[test]
fn air_json_minimal_loads() {
    let module = load_air_json_fixture("minimal");
    assert!(
        !module.functions.is_empty(),
        "minimal fixture should have at least one function"
    );
}

#[test]
fn air_json_cfg_build() {
    let module = load_air_json_fixture("control_flow");
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let cfg = Cfg::build(func);
        // CFG should have at least one block for non-empty functions
        assert!(!cfg.blocks().is_empty() || func.blocks.is_empty());
    }
}

#[test]
fn air_json_defuse_build() {
    let module = load_air_json_fixture("memory_ops");
    let defuse = DefUseGraph::build(&module);
    // Def-use graph should have definitions for a non-trivial module
    assert!(
        !defuse.defs.is_empty(),
        "memory_ops fixture should produce definitions"
    );
}

#[test]
fn air_json_callgraph_build() {
    let module = load_air_json_fixture("calls");
    let cg = CallGraph::build(&module);
    assert!(
        !cg.nodes.is_empty(),
        "calls fixture should produce call graph nodes"
    );
}

#[test]
fn air_json_pipeline_runs() {
    let module = load_air_json_fixture("calls");
    let config = PipelineConfig::default();
    let result = run_pipeline(&module, &config);
    // Pipeline should produce a call graph with nodes
    assert!(
        !result.call_graph.nodes.is_empty(),
        "pipeline should produce a non-empty call graph"
    );
    // Pipeline should produce a def-use graph
    assert!(
        !result.defuse.defs.is_empty(),
        "pipeline should produce def-use information"
    );
}
