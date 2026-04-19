//! Test utilities for SAF crates.
//!
//! This crate provides shared test helpers for loading fixtures and
//! common test setup across the SAF workspace.

use saf_core::air::{AirBundle, AirModule};
use saf_core::config::Config;
use saf_core::program::AirProgram;
use saf_frontends::air_json::AirJsonFrontend;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;
use std::path::{Path, PathBuf};

/// Returns the path to the E2E test fixtures directory.
///
/// Uses `CARGO_MANIFEST_DIR` to reliably locate the fixtures regardless
/// of the current working directory.
///
/// # Panics
///
/// Panics if the path structure doesn't match expected workspace layout
/// (saf-test-utils should be in `crates/` which should be in workspace root).
pub fn fixtures_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("saf-test-utils should be in crates/")
        .parent()
        .expect("crates/ should be in workspace root")
        .join("tests")
        .join("fixtures")
        .join("llvm")
        .join("e2e")
}

/// Load an LLVM IR fixture by name and parse it into an `AirBundle`.
///
/// # Arguments
///
/// * `name` - The fixture name without extension (e.g., `command_injection`)
///
/// # Panics
///
/// Panics if the fixture file doesn't exist or fails to parse.
pub fn load_ll_bundle(name: &str) -> AirBundle {
    let path = fixtures_dir().join(format!("{name}.ll"));
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    frontend
        .ingest(&[path.as_path()], &config)
        .unwrap_or_else(|e| panic!("Failed to load fixture '{name}' at {}: {e}", path.display()))
}

/// Load an LLVM IR fixture by name and return the module.
///
/// This is the most common use case for E2E tests that work with a single module.
///
/// # Arguments
///
/// * `name` - The fixture name without extension (e.g., `command_injection`)
///
/// # Panics
///
/// Panics if the fixture file doesn't exist or fails to parse.
pub fn load_ll_fixture(name: &str) -> AirModule {
    load_ll_bundle(name).module
}

/// Load an LLVM IR fixture from an explicit path.
///
/// Useful for tests that need to load fixtures from non-standard locations.
///
/// # Panics
///
/// Panics if the file doesn't exist or fails to parse.
pub fn load_ll_from_path(path: &Path) -> AirBundle {
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    frontend
        .ingest(&[path], &config)
        .unwrap_or_else(|e| panic!("Failed to load fixture at {}: {e}", path.display()))
}

/// Returns the path to the verification test fixtures directory.
///
/// Uses `CARGO_MANIFEST_DIR` to reliably locate fixtures.
///
/// # Panics
///
/// Panics if the path structure doesn't match expected workspace layout.
pub fn verification_fixtures_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("saf-test-utils should be in crates/")
        .parent()
        .expect("crates/ should be in workspace root")
        .join("tests")
        .join("fixtures")
}

/// Returns the path to the AIR-JSON test fixtures directory.
///
/// # Panics
///
/// Panics if the path structure doesn't match expected workspace layout.
pub fn air_json_fixtures_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("saf-test-utils should be in crates/")
        .parent()
        .expect("crates/ should be in workspace root")
        .join("tests")
        .join("fixtures")
        .join("air_json")
}

/// Load an AIR-JSON fixture by name and parse it into an `AirBundle`.
///
/// # Arguments
///
/// * `name` - The fixture name without the `.air.json` extension (e.g., `calls`)
///
/// # Panics
///
/// Panics if the fixture file doesn't exist or fails to parse.
pub fn load_air_json_bundle(name: &str) -> AirBundle {
    let path = air_json_fixtures_dir().join(format!("{name}.air.json"));
    let frontend = AirJsonFrontend;
    let config = Config::default();
    frontend
        .ingest(&[path.as_path()], &config)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to load AIR-JSON fixture '{name}' at {}: {e}",
                path.display()
            )
        })
}

/// Load an AIR-JSON fixture by name and return the module.
///
/// # Arguments
///
/// * `name` - The fixture name without the `.air.json` extension (e.g., `calls`)
///
/// # Panics
///
/// Panics if the fixture file doesn't exist or fails to parse.
pub fn load_air_json_fixture(name: &str) -> AirModule {
    load_air_json_bundle(name).module
}

/// Load an LLVM IR fixture from a verification subdirectory.
///
/// # Arguments
///
/// * `category` - The verification category (e.g., `ifds_verification`, `pta_verification`)
/// * `name` - The fixture name without extension (e.g., `zero_fact_branches`)
///
/// # Panics
///
/// Panics if the fixture file doesn't exist or fails to parse.
pub fn load_verification_fixture(category: &str, name: &str) -> AirModule {
    let path = verification_fixtures_dir()
        .join(category)
        .join(format!("{name}.ll"));
    let bundle = load_ll_from_path(&path);
    bundle.module
}

/// Return the path to `tests/fixtures/incremental/lua/`.
///
/// # Panics
///
/// Panics if the path structure doesn't match expected workspace layout.
#[must_use]
pub fn incremental_lua_fixtures_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("saf-test-utils should be in crates/")
        .parent()
        .expect("crates/ should be in workspace root")
        .join("tests")
        .join("fixtures")
        .join("incremental")
        .join("lua")
}

/// Load all Lua `.ll` fixtures as a multi-module `AirProgram`.
///
/// If `swap` is `Some(("lmathlib", "lmathlib_v2"))`, the named module's `.ll`
/// file is replaced with the v2 variant before linking.
///
/// # Panics
///
/// Panics if the fixture directory is unreadable or any `.ll` file fails to parse.
pub fn load_incremental_lua_program(swap: Option<(&str, &str)>) -> AirProgram {
    let dir = incremental_lua_fixtures_dir();
    let mut bundles = Vec::new();

    for entry in std::fs::read_dir(&dir).expect("read incremental/lua dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "ll") {
            let stem = path.file_stem().unwrap().to_str().unwrap();

            // Skip *_v2 files — they're only used when swapped in
            if stem.ends_with("_v2") {
                continue;
            }

            // If swap requested and this is the target, load v2 instead
            if let Some((orig, replacement)) = swap {
                if stem == orig {
                    let v2_path = dir.join(format!("{replacement}.ll"));
                    bundles.push(load_ll_from_path(&v2_path));
                    continue;
                }
            }

            bundles.push(load_ll_from_path(&path));
        }
    }

    AirProgram::link(bundles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_dir_exists() {
        let dir = fixtures_dir();
        assert!(dir.exists(), "Fixtures directory should exist: {dir:?}");
    }

    #[test]
    fn test_fixtures_dir_contains_ll_files() {
        let dir = fixtures_dir();
        let ll_files: Vec<_> = std::fs::read_dir(&dir)
            .expect("Should be able to read fixtures dir")
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "ll"))
            .collect();
        assert!(
            !ll_files.is_empty(),
            "Fixtures directory should contain .ll files"
        );
    }

    #[test]
    fn test_load_ll_fixture_works() {
        // Use a known fixture that should exist
        let module = load_ll_fixture("command_injection");
        assert!(!module.functions.is_empty(), "Module should have functions");
    }
}
