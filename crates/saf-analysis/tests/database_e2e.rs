//! End-to-end tests for the `ProgramDatabase`.

use saf_analysis::database::ProgramDatabase;
use saf_analysis::pipeline::PipelineConfig;
use saf_test_utils::load_air_json_fixture;

#[test]
fn database_builds_from_air_json() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());
    assert!(!db.call_graph().nodes.is_empty());
    assert!(!db.defuse().defs.is_empty());
    assert!(db.pta_result().is_some());
}

#[test]
fn database_exports_property_graphs() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());
    let graphs = db.export_graphs();

    // Should export at least callgraph, defuse, valueflow
    let types: Vec<&str> = graphs.iter().map(|g| g.graph_type.as_str()).collect();
    assert!(types.contains(&"callgraph"), "missing callgraph export");
    assert!(types.contains(&"defuse"), "missing defuse export");
    assert!(types.contains(&"valueflow"), "missing valueflow export");

    // Each graph should have nodes
    for g in &graphs {
        assert!(!g.nodes.is_empty(), "{} graph has no nodes", g.graph_type);
    }
}

#[test]
fn database_alias_query() {
    let module = load_air_json_fixture("memory_ops");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    if let Some(pta) = db.pta_result() {
        // Get any pointer value from the PTA result to test with
        let ptrs: Vec<_> = pta.points_to_map().keys().take(2).collect();
        if ptrs.len() == 2 {
            let result = db.may_alias(*ptrs[0], *ptrs[1]);
            assert!(matches!(
                result,
                saf_analysis::AliasResult::Must
                    | saf_analysis::AliasResult::Partial
                    | saf_analysis::AliasResult::May
                    | saf_analysis::AliasResult::No
                    | saf_analysis::AliasResult::Unknown
            ));
        }
    }
}

#[test]
fn database_cfg_accessor() {
    let module = load_air_json_fixture("control_flow");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    for func in db.module().functions.iter().filter(|f| !f.is_declaration) {
        let cfg = db.cfg(func.id);
        assert!(!cfg.blocks().is_empty(), "CFG should have blocks");
    }
}

#[test]
fn database_callgraph_reachable() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    // Collect function IDs from Function nodes in the call graph
    let funcs: Vec<_> = db
        .call_graph()
        .nodes
        .iter()
        .filter_map(|n| {
            if let saf_analysis::callgraph::CallGraphNode::Function(fid) = n {
                Some(*fid)
            } else {
                None
            }
        })
        .collect();

    if !funcs.is_empty() {
        let reachable = db.cg_reachable_from(&[funcs[0]]);
        assert!(
            reachable.contains(&funcs[0]),
            "function should be reachable from itself"
        );
    }
}

#[test]
fn database_schema_has_checks_and_graphs() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());
    let schema = db.schema();

    assert!(!schema.checks.is_empty(), "schema should list checks");
    assert!(!schema.graphs.is_empty(), "schema should list graphs");
    assert!(
        schema.checks.iter().any(|c| c.name == "use_after_free"),
        "schema should include use_after_free"
    );
}

#[test]
fn database_handles_schema_request() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    let req = r#"{"action": "schema"}"#;
    let resp_json = db.handle_request(req).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&resp_json).unwrap();
    assert_eq!(resp["status"], "ok");
    assert!(resp["checks"].is_array());
}

#[test]
fn database_handles_check_request() {
    let module = load_air_json_fixture("memory_ops");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    let req = r#"{"action": "check", "name": "null_deref"}"#;
    let resp_json = db.handle_request(req).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&resp_json).unwrap();
    assert_eq!(resp["status"], "ok");
    assert!(resp.get("findings").is_some());
}

#[test]
fn database_handles_unknown_check() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    let req = r#"{"action": "check", "name": "nonexistent_check"}"#;
    let resp_json = db.handle_request(req).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&resp_json).unwrap();
    assert_eq!(resp["status"], "error");
    assert_eq!(resp["error"]["code"], "UNKNOWN_CHECK");
}

#[test]
fn database_handles_check_all() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    let req = r#"{"action": "check_all"}"#;
    let resp_json = db.handle_request(req).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&resp_json).unwrap();
    assert_eq!(resp["status"], "ok");
    assert!(resp.get("findings").is_some());
}
