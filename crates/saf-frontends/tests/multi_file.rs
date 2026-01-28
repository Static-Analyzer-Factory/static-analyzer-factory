//! Tests for multi-file ingestion with caching.

use std::path::PathBuf;

use saf_core::cache::BundleCache;
use saf_core::config::Config;
use saf_frontends::air_json::AirJsonFrontend;
use saf_frontends::api::Frontend;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/incremental/two_module")
        .join(name)
}

#[test]
fn ingest_multi_air_json_returns_two_bundles() {
    let frontend = AirJsonFrontend;
    let config = Config::default();

    let main_path = fixture_path("main.air.json");
    let lib_path = fixture_path("lib.air.json");
    let inputs: Vec<&std::path::Path> = vec![main_path.as_path(), lib_path.as_path()];

    let bundles = frontend.ingest_multi(&inputs, &config, None).unwrap();

    assert_eq!(bundles.len(), 2);
    assert_eq!(bundles[0].module.name.as_deref(), Some("main"));
    assert_eq!(bundles[1].module.name.as_deref(), Some("lib"));
}

#[test]
fn ingest_multi_with_cache_hits_on_second_call() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = BundleCache::new(tmp.path());
    let frontend = AirJsonFrontend;
    let config = Config::default();

    let main_path = fixture_path("main.air.json");
    let lib_path = fixture_path("lib.air.json");
    let inputs: Vec<&std::path::Path> = vec![main_path.as_path(), lib_path.as_path()];

    // First call: cache miss, populates cache
    let bundles1 = frontend
        .ingest_multi(&inputs, &config, Some(&cache))
        .unwrap();
    assert_eq!(bundles1.len(), 2);

    // Second call: cache hit
    let bundles2 = frontend
        .ingest_multi(&inputs, &config, Some(&cache))
        .unwrap();
    assert_eq!(bundles2.len(), 2);

    // Results should be equivalent
    assert_eq!(bundles1[0].module.name, bundles2[0].module.name);
    assert_eq!(bundles1[1].module.name, bundles2[1].module.name);
    assert_eq!(
        bundles1[0].module.functions.len(),
        bundles2[0].module.functions.len()
    );
}

#[test]
fn ingest_multi_detects_changed_file() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = BundleCache::new(tmp.path());
    let frontend = AirJsonFrontend;
    let config = Config::default();

    let main_path = fixture_path("main.air.json");
    let lib_v1_path = fixture_path("lib.air.json");
    let lib_v2_path = fixture_path("lib_v2.air.json");

    // First run: main + lib_v1
    let inputs1: Vec<&std::path::Path> = vec![main_path.as_path(), lib_v1_path.as_path()];
    let bundles1 = frontend
        .ingest_multi(&inputs1, &config, Some(&cache))
        .unwrap();
    assert_eq!(bundles1.len(), 2);

    // Second run: main + lib_v2 (different file content)
    let inputs2: Vec<&std::path::Path> = vec![main_path.as_path(), lib_v2_path.as_path()];
    let bundles2 = frontend
        .ingest_multi(&inputs2, &config, Some(&cache))
        .unwrap();
    assert_eq!(bundles2.len(), 2);

    // main should be the same (cache hit), lib should differ
    assert_eq!(bundles1[0].module.id, bundles2[0].module.id); // main unchanged
    assert_ne!(bundles1[1].module.id, bundles2[1].module.id); // lib changed
}
