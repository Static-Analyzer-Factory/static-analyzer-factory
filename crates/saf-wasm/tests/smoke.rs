use serde_json::Value;

/// Minimal valid AIR JSON matching the `AirBundle` schema.
///
/// Contains a single function "main" with one basic block that has
/// a single `Ret` instruction. All IDs are `u128` hex strings.
fn minimal_air_json() -> String {
    serde_json::json!({
        "frontend_id": "test",
        "schema_version": "0.1.0",
        "module": {
            "id": "0x00000000000000000000000000000001",
            "name": "test_module",
            "functions": [
                {
                    "id": "0x00000000000000000000000000000002",
                    "name": "main",
                    "blocks": [
                        {
                            "id": "0x00000000000000000000000000000003",
                            "label": "entry",
                            "instructions": [
                                {
                                    "id": "0x00000000000000000000000000000004",
                                    "op": "ret"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    })
    .to_string()
}

#[test]
fn analyze_returns_valid_json() {
    let result = saf_wasm::analyze(&minimal_air_json(), "{}");
    let parsed: Value = serde_json::from_str(&result).expect("result should be valid JSON");
    assert!(
        parsed.get("error").is_none(),
        "expected no error, got: {result}"
    );
}

#[test]
fn analyze_contains_expected_keys() {
    let result = saf_wasm::analyze(&minimal_air_json(), "{}");
    let parsed: Value = serde_json::from_str(&result).expect("result should be valid JSON");

    assert!(parsed.get("cfg").is_some(), "missing cfg key");
    assert!(parsed.get("callgraph").is_some(), "missing callgraph key");
    assert!(parsed.get("defuse").is_some(), "missing defuse key");
    assert!(parsed.get("valueflow").is_some(), "missing valueflow key");
}

#[test]
fn analyze_cfg_is_property_graph() {
    let result = saf_wasm::analyze(&minimal_air_json(), "{}");
    let parsed: Value = serde_json::from_str(&result).expect("result should be valid JSON");

    let cfg = parsed.get("cfg").expect("missing cfg key");
    assert!(cfg.is_object(), "cfg should be a merged PropertyGraph");
    assert_eq!(
        cfg.get("graph_type").and_then(|v| v.as_str()),
        Some("cfg"),
        "cfg.graph_type should be \"cfg\""
    );
    let nodes = cfg.get("nodes").and_then(|v| v.as_array());
    assert!(
        nodes.is_some() && !nodes.unwrap().is_empty(),
        "cfg should have at least one node"
    );
}

#[test]
fn analyze_invalid_json_returns_error() {
    let result = saf_wasm::analyze("not valid json", "{}");
    let parsed: Value = serde_json::from_str(&result).expect("result should be valid JSON");
    assert!(
        parsed.get("error").is_some(),
        "expected error for invalid input"
    );
}
