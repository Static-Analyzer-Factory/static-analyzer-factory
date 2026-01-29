//! Integration tests for graph builders using AIR JSON fixtures.

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::icfg::Icfg;
use saf_test_utils::load_air_json_bundle as load_fixture;

#[test]
fn cfg_control_flow_fixture() {
    let bundle = load_fixture("control_flow");
    let module = &bundle.module;

    // Build CFGs for all functions
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let cfg = Cfg::build(func);

        // Verify basic properties
        assert!(!cfg.successors.is_empty(), "CFG should have blocks");
        assert!(!cfg.exits.is_empty(), "CFG should have exit blocks");

        // Export and snapshot
        let export = cfg.export(func);
        insta::assert_json_snapshot!(format!("cfg_{}", func.name), export);
    }
}

#[test]
fn callgraph_calls_fixture() {
    let bundle = load_fixture("calls");
    let module = &bundle.module;

    let cg = CallGraph::build(module);

    // Verify we have nodes for both functions
    assert!(cg.nodes.len() >= 2, "Should have at least 2 function nodes");

    // Verify indirect call creates placeholder
    let indirect_calls = cg.indirect_calls();
    assert_eq!(
        indirect_calls.len(),
        1,
        "Should have 1 indirect call placeholder"
    );

    // Export and snapshot
    let export = cg.export(module);
    insta::assert_json_snapshot!("callgraph_calls", export);
}

#[test]
fn defuse_control_flow_fixture() {
    let bundle = load_fixture("control_flow");
    let module = &bundle.module;

    let du = DefUseGraph::build(module);

    // Verify we have definitions
    assert!(!du.defs.is_empty(), "Should have definitions");

    // Verify parameter is recorded
    let param_count = du.defs.values().filter(|d| d.is_none()).count();
    assert!(param_count > 0, "Should have parameters recorded");

    // Export and snapshot
    let export = du.export();
    insta::assert_json_snapshot!("defuse_control_flow", export);
}

#[test]
fn icfg_calls_fixture() {
    let bundle = load_fixture("calls");
    let module = &bundle.module;

    let cg = CallGraph::build(module);
    let icfg = Icfg::build(module, &cg);

    // Should have CFGs for non-declaration functions
    let defined_funcs = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .count();
    assert_eq!(
        icfg.cfgs.len(),
        defined_funcs,
        "Should have CFG for each defined function"
    );

    // Should have interprocedural edges for direct call
    assert!(
        !icfg.inter_edges.is_empty(),
        "Should have interprocedural edges"
    );

    // Export and snapshot
    let export = icfg.export(module);
    insta::assert_json_snapshot!("icfg_calls", export);
}

#[test]
fn graphs_are_deterministic_across_runs() {
    // Load the same fixture twice and verify identical results
    let bundle1 = load_fixture("control_flow");
    let bundle2 = load_fixture("control_flow");

    // CFG
    for (func1, func2) in bundle1
        .module
        .functions
        .iter()
        .zip(bundle2.module.functions.iter())
    {
        if func1.is_declaration {
            continue;
        }
        let cfg1 = Cfg::build(func1);
        let cfg2 = Cfg::build(func2);

        let export1 = serde_json::to_string(&cfg1.export(func1)).unwrap();
        let export2 = serde_json::to_string(&cfg2.export(func2)).unwrap();
        assert_eq!(export1, export2, "CFG exports should be identical");
    }

    // CallGraph
    let cg1 = CallGraph::build(&bundle1.module);
    let cg2 = CallGraph::build(&bundle2.module);
    let cg_export1 = serde_json::to_string(&cg1.export(&bundle1.module)).unwrap();
    let cg_export2 = serde_json::to_string(&cg2.export(&bundle2.module)).unwrap();
    assert_eq!(
        cg_export1, cg_export2,
        "CallGraph exports should be identical"
    );

    // DefUse
    let du1 = DefUseGraph::build(&bundle1.module);
    let du2 = DefUseGraph::build(&bundle2.module);
    let du_export1 = serde_json::to_string(&du1.export()).unwrap();
    let du_export2 = serde_json::to_string(&du2.export()).unwrap();
    assert_eq!(du_export1, du_export2, "DefUse exports should be identical");
}

#[test]
fn all_fixtures_parse_and_build_graphs() {
    // Test that all fixtures can be processed without errors
    let fixtures = [
        "minimal",
        "memory_ops",
        "control_flow",
        "calls",
        "constants",
    ];

    for name in fixtures {
        let bundle = load_fixture(name);
        let module = &bundle.module;

        // Build all graphs
        let _cg = CallGraph::build(module);
        let _du = DefUseGraph::build(module);

        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            let _cfg = Cfg::build(func);
        }

        let cg = CallGraph::build(module);
        let _icfg = Icfg::build(module, &cg);
    }
}
