//! Integration tests for multi-module `AirProgram` linking.

use std::path::PathBuf;

use saf_core::program::AirProgram;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/incremental/two_module")
        .join(name)
}

#[test]
fn ingest_and_link_two_modules() {
    let main_json = std::fs::read_to_string(fixture_path("main.air.json")).unwrap();
    let lib_json = std::fs::read_to_string(fixture_path("lib.air.json")).unwrap();

    let main_bundle: saf_core::air::AirBundle = serde_json::from_str(&main_json).unwrap();
    let lib_bundle: saf_core::air::AirBundle = serde_json::from_str(&lib_json).unwrap();

    let program = AirProgram::link(vec![main_bundle, lib_bundle]);

    assert_eq!(program.modules.len(), 2);

    // helper declaration in main should resolve to helper definition in lib
    assert_eq!(program.link_table.function_resolutions.len(), 1);
    assert!(program.link_table.conflicts.is_empty());
}

#[test]
fn merged_view_has_definition_not_declaration() {
    let main_json = std::fs::read_to_string(fixture_path("main.air.json")).unwrap();
    let lib_json = std::fs::read_to_string(fixture_path("lib.air.json")).unwrap();

    let main_bundle: saf_core::air::AirBundle = serde_json::from_str(&main_json).unwrap();
    let lib_bundle: saf_core::air::AirBundle = serde_json::from_str(&lib_json).unwrap();

    let program = AirProgram::link(vec![main_bundle, lib_bundle]);
    let merged = program.merged_view();

    // merged should have 2 functions: main (def) + helper (def)
    assert_eq!(merged.functions.len(), 2);

    let helper = merged.function_by_name("helper").unwrap();
    assert!(
        !helper.is_declaration,
        "helper should be a definition in merged view"
    );
    assert!(
        !helper.blocks.is_empty(),
        "helper definition should have blocks"
    );
}

#[test]
fn merged_view_produces_valid_module_for_analysis() {
    let main_json = std::fs::read_to_string(fixture_path("main.air.json")).unwrap();
    let lib_json = std::fs::read_to_string(fixture_path("lib.air.json")).unwrap();

    let main_bundle: saf_core::air::AirBundle = serde_json::from_str(&main_json).unwrap();
    let lib_bundle: saf_core::air::AirBundle = serde_json::from_str(&lib_json).unwrap();

    let program = AirProgram::link(vec![main_bundle, lib_bundle]);
    let merged = program.merged_view();

    // The merged module should be round-trippable through JSON
    let json = serde_json::to_string(&merged).unwrap();
    let deserialized: saf_core::air::AirModule = serde_json::from_str(&json).unwrap();
    assert_eq!(merged.functions.len(), deserialized.functions.len());

    // Function index should work
    assert!(merged.function_by_name("main").is_some());
    assert!(merged.function_by_name("helper").is_some());
}

#[test]
fn program_diff_detects_changed_module() {
    let main_json = std::fs::read_to_string(fixture_path("main.air.json")).unwrap();
    let lib_v1_json = std::fs::read_to_string(fixture_path("lib.air.json")).unwrap();
    let lib_v2_json = std::fs::read_to_string(fixture_path("lib_v2.air.json")).unwrap();

    let main_bundle: saf_core::air::AirBundle = serde_json::from_str(&main_json).unwrap();
    let lib_v1: saf_core::air::AirBundle = serde_json::from_str(&lib_v1_json).unwrap();
    let lib_v2: saf_core::air::AirBundle = serde_json::from_str(&lib_v2_json).unwrap();

    let prev_ids = vec![main_bundle.module.id, lib_v1.module.id];
    let curr_ids = vec![main_bundle.module.id, lib_v2.module.id];

    let diff = AirProgram::diff(&prev_ids, &curr_ids);

    // main unchanged, lib_v1 removed, lib_v2 added
    assert_eq!(diff.unchanged_modules.len(), 1);
    assert_eq!(diff.removed_modules.len(), 1);
    assert_eq!(diff.added_modules.len(), 1);
    assert_eq!(diff.removed_modules[0], lib_v1.module.id);
    assert_eq!(diff.added_modules[0], lib_v2.module.id);
}
