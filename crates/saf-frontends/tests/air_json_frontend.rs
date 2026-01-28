//! Integration tests for the AIR JSON frontend.

use std::path::Path;

use saf_core::air::{AirBundle, Constant, Operation};
use saf_core::config::Config;
use saf_frontends::air_json::AirJsonFrontend;
use saf_frontends::api::Frontend;

fn fixtures_dir() -> &'static Path {
    static FIXTURES: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    FIXTURES.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/air_json")
    })
}

fn parse_fixture(name: &str) -> AirBundle {
    let path = fixtures_dir().join(name);
    let frontend = AirJsonFrontend;
    let config = Config::default();
    frontend.ingest(&[path.as_path()], &config).expect("parse")
}

#[test]
fn parse_minimal_fixture() {
    let bundle = parse_fixture("minimal.air.json");

    assert_eq!(bundle.frontend_id, "air-json");
    assert_eq!(bundle.schema_version, "0.1.0");
    assert_eq!(bundle.module.name, Some("minimal".to_string()));
    assert_eq!(bundle.module.functions.len(), 1);

    let func = &bundle.module.functions[0];
    assert_eq!(func.name, "main");
    assert!(func.params.is_empty());
    assert_eq!(func.blocks.len(), 1);

    let block = &func.blocks[0];
    assert_eq!(block.label, Some("entry".to_string()));
    assert_eq!(block.instructions.len(), 1);
    assert!(matches!(block.instructions[0].op, Operation::Ret));

    // Snapshot test
    insta::assert_json_snapshot!("minimal_bundle", bundle);
}

#[test]
fn parse_memory_ops_fixture() {
    let bundle = parse_fixture("memory_ops.air.json");

    assert_eq!(bundle.module.name, Some("memory_ops".to_string()));
    assert_eq!(bundle.module.functions.len(), 1);

    let func = &bundle.module.functions[0];
    assert_eq!(func.name, "test_memory");
    assert_eq!(func.blocks.len(), 1);

    let block = &func.blocks[0];
    // alloca, alloca, store, load, gep, store, ret
    assert_eq!(block.instructions.len(), 7);

    // Check operations
    assert!(matches!(block.instructions[0].op, Operation::Alloca { .. }));
    assert!(matches!(block.instructions[1].op, Operation::Alloca { .. }));
    assert!(matches!(block.instructions[2].op, Operation::Store));
    assert!(matches!(block.instructions[3].op, Operation::Load));
    assert!(matches!(block.instructions[4].op, Operation::Gep { .. }));
    assert!(matches!(block.instructions[5].op, Operation::Store));
    assert!(matches!(block.instructions[6].op, Operation::Ret));

    // Check GEP field path
    if let Operation::Gep { ref field_path } = block.instructions[4].op {
        assert_eq!(field_path.steps.len(), 2);
    } else {
        panic!("Expected GEP operation");
    }

    insta::assert_json_snapshot!("memory_ops_bundle", bundle);
}

#[test]
fn parse_control_flow_fixture() {
    let bundle = parse_fixture("control_flow.air.json");

    assert_eq!(bundle.module.name, Some("control_flow".to_string()));
    assert_eq!(bundle.module.functions.len(), 1);

    let func = &bundle.module.functions[0];
    assert_eq!(func.name, "test_branch");
    assert_eq!(func.params.len(), 1);
    assert_eq!(func.params[0].name, Some("cond".to_string()));

    // 4 blocks: entry, then, else, merge
    assert_eq!(func.blocks.len(), 4);

    // Check entry block has cond_br
    let entry = &func.blocks[0];
    assert_eq!(entry.label, Some("entry".to_string()));
    assert!(matches!(
        entry.instructions.last().unwrap().op,
        Operation::CondBr { .. }
    ));

    // Check merge block has phi
    let merge = &func.blocks[3];
    assert_eq!(merge.label, Some("merge".to_string()));
    assert!(matches!(merge.instructions[0].op, Operation::Phi { .. }));

    if let Operation::Phi { ref incoming } = merge.instructions[0].op {
        assert_eq!(incoming.len(), 2);
    } else {
        panic!("Expected Phi operation");
    }

    insta::assert_json_snapshot!("control_flow_bundle", bundle);
}

#[test]
fn parse_calls_fixture() {
    let bundle = parse_fixture("calls.air.json");

    assert_eq!(bundle.module.name, Some("calls".to_string()));
    assert_eq!(bundle.module.functions.len(), 2);

    // Find main function
    let main = bundle
        .module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("main function");

    let block = &main.blocks[0];

    // Find call_direct
    let call_direct = block
        .instructions
        .iter()
        .find(|i| matches!(i.op, Operation::CallDirect { .. }))
        .expect("call_direct");
    assert!(call_direct.dst.is_some());

    // Find call_indirect
    let call_indirect = block
        .instructions
        .iter()
        .find(|i| matches!(i.op, Operation::CallIndirect { .. }))
        .expect("call_indirect");
    assert!(call_indirect.dst.is_some());

    insta::assert_json_snapshot!("calls_bundle", bundle);
}

#[test]
fn parse_constants_fixture() {
    let bundle = parse_fixture("constants.air.json");

    assert_eq!(bundle.module.name, Some("constants".to_string()));
    assert_eq!(bundle.module.globals.len(), 7);

    // Check integer constant
    let const_int = bundle
        .module
        .globals
        .iter()
        .find(|g| g.name == "CONST_INT")
        .expect("CONST_INT");
    assert!(const_int.is_constant);
    if let Some(Constant::Int { value, bits }) = const_int.init {
        assert_eq!(value, 42);
        assert_eq!(bits, 32);
    } else {
        panic!("Expected Int constant");
    }

    // Check float constant
    let const_float = bundle
        .module
        .globals
        .iter()
        .find(|g| g.name == "CONST_FLOAT")
        .expect("CONST_FLOAT");
    assert!(const_float.is_constant);
    if let Some(Constant::Float { value, bits }) = const_float.init {
        assert!((value - std::f64::consts::PI).abs() < 0.001);
        assert_eq!(bits, 64);
    } else {
        panic!("Expected Float constant");
    }

    // Check string constant
    let const_string = bundle
        .module
        .globals
        .iter()
        .find(|g| g.name == "CONST_STRING")
        .expect("CONST_STRING");
    assert!(matches!(
        const_string.init,
        Some(Constant::String { ref value }) if value == "Hello, World!"
    ));

    // Check null constant
    let const_null = bundle
        .module
        .globals
        .iter()
        .find(|g| g.name == "CONST_NULL")
        .expect("CONST_NULL");
    assert!(matches!(const_null.init, Some(Constant::Null)));

    // Check aggregate constant
    let const_agg = bundle
        .module
        .globals
        .iter()
        .find(|g| g.name == "CONST_AGGREGATE")
        .expect("CONST_AGGREGATE");
    if let Some(Constant::Aggregate { ref elements }) = const_agg.init {
        assert_eq!(elements.len(), 3);
    } else {
        panic!("Expected Aggregate constant");
    }

    // Check zero_init
    let zero_init = bundle
        .module
        .globals
        .iter()
        .find(|g| g.name == "ZERO_INIT")
        .expect("ZERO_INIT");
    assert!(!zero_init.is_constant);
    assert!(matches!(zero_init.init, Some(Constant::ZeroInit)));

    // Check undef
    let undef = bundle
        .module
        .globals
        .iter()
        .find(|g| g.name == "UNDEF_VAL")
        .expect("UNDEF_VAL");
    assert!(matches!(undef.init, Some(Constant::Undef)));

    insta::assert_json_snapshot!("constants_bundle", bundle);
}

#[test]
fn determinism_same_input_same_output() {
    let bundle1 = parse_fixture("minimal.air.json");
    let bundle2 = parse_fixture("minimal.air.json");

    // Serialize both to JSON and compare
    let json1 = serde_json::to_string(&bundle1).expect("serialize 1");
    let json2 = serde_json::to_string(&bundle2).expect("serialize 2");

    assert_eq!(
        json1, json2,
        "Parsing same file twice should produce identical output"
    );
}

#[test]
fn input_fingerprint_determinism() {
    let path = fixtures_dir().join("minimal.air.json");
    let frontend = AirJsonFrontend;
    let config = Config::default();

    let fp1 = frontend
        .input_fingerprint_bytes(&[path.as_path()], &config)
        .expect("fingerprint 1");
    let fp2 = frontend
        .input_fingerprint_bytes(&[path.as_path()], &config)
        .expect("fingerprint 2");

    assert_eq!(fp1, fp2, "Fingerprints should be deterministic");
    assert_eq!(fp1.len(), 32, "BLAKE3 produces 32 bytes");
}

#[test]
fn error_on_invalid_schema_version() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let json = r#"{
        "frontend_id": "air-json",
        "schema_version": "99.0.0",
        "module": { "functions": [] }
    }"#;

    let mut file = NamedTempFile::new().expect("temp file");
    file.write_all(json.as_bytes()).expect("write");

    let frontend = AirJsonFrontend;
    let config = Config::default();
    let result = frontend.ingest(&[file.path()], &config);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unsupported schema version"));
}

#[test]
fn error_on_no_inputs() {
    let frontend = AirJsonFrontend;
    let config = Config::default();
    let result = frontend.ingest(&[], &config);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("no input files"));
}

#[test]
fn error_on_invalid_json() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().expect("temp file");
    file.write_all(b"{ invalid json }").expect("write");

    let frontend = AirJsonFrontend;
    let config = Config::default();
    let result = frontend.ingest(&[file.path()], &config);

    assert!(result.is_err());
}
