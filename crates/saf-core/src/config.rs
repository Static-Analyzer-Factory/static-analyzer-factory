//! SAF configuration contract (SRS Section 6).
//!
//! Config is JSON-serializable and hashed into cache keys. All fields
//! have documented defaults matching the SRS specification.

use serde::{Deserialize, Serialize};

/// Which frontend to use for ingestion.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Frontend {
    /// LLVM bitcode / IR frontend.
    #[default]
    Llvm,
    /// AIR-JSON frontend.
    #[serde(rename = "air-json")]
    AirJson,
    /// Custom frontend — the string is the frontend identifier.
    /// New frontends can use any string without modifying `saf-core`.
    #[serde(untagged)]
    Other(String),
}

/// Analysis mode controlling the speed/precision trade-off.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    /// Fast mode: fewer iterations, less precision.
    Fast,
    /// Precise mode: full fixed-point iteration.
    #[default]
    Precise,
}

/// Field sensitivity level for the top-level config.
///
/// This is a simpler enum than `saf_analysis::pta::config::FieldSensitivity`,
/// which additionally carries `max_depth`. This enum represents the
/// serializable config value only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFieldSensitivity {
    /// No field sensitivity -- treat structs as monolithic objects.
    None,
    /// Track individual struct fields.
    #[default]
    StructFields,
}

/// External call side-effect model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSideEffects {
    /// No side effects assumed for external calls.
    None,
    /// External calls may write to unknown locations.
    UnknownWrite,
    /// External calls may read from and write to unknown locations.
    #[default]
    UnknownReadwrite,
}

/// Top-level configuration for a SAF analysis run.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Which frontend to use: `Llvm` or `AirJson`.
    #[serde(default)]
    pub frontend: Frontend,

    /// Analysis sub-config.
    #[serde(default)]
    pub analysis: AnalysisConfig,

    /// External call side-effect model.
    #[serde(default)]
    pub external_side_effects: ExternalSideEffects,

    /// Path normalization sub-config.
    #[serde(default)]
    pub paths: PathsConfig,

    /// Rust-specific sub-config.
    #[serde(default)]
    pub rust: RustConfig,

    /// Incremental analysis sub-config.
    #[serde(default)]
    pub incremental: IncrementalConfig,
}

/// Analysis sub-configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AnalysisConfig {
    /// Analysis mode: `Fast` or `Precise`.
    #[serde(default)]
    pub mode: AnalysisMode,
}

/// Path normalization sub-configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PathsConfig {
    /// Prefixes to strip from source paths in outputs.
    #[serde(default)]
    pub strip_prefixes: Vec<String>,

    /// Whether to normalize path separators to `/`.
    #[serde(default)]
    pub normalize_separators: bool,
}

/// Rust-specific sub-configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RustConfig {
    /// Whether to demangle Rust symbols in outputs.
    #[serde(default)]
    pub demangle: bool,
}

/// Which PTA solver backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PtaSolver {
    /// Worklist-based imperative solver (default — matches SVF performance).
    #[default]
    Worklist,
    /// Datalog fixpoint solver (Ascent).
    Datalog,
}

/// Configuration for incremental analysis (Plan 165).
///
/// When `enabled` is false (default), the system behaves identically to
/// non-incremental analysis. All fields are ignored unless `enabled` is true.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IncrementalConfig {
    /// Master switch for incremental analysis.
    pub enabled: bool,

    /// Cache directory for per-module AIR bundles and analysis products.
    pub cache_dir: std::path::PathBuf,

    /// How to split a single pre-linked input into logical modules.
    pub split_strategy: crate::program::SplitStrategy,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_dir: std::path::PathBuf::from(".saf-cache"),
            split_strategy: crate::program::SplitStrategy::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }

    #[test]
    fn default_values_match_srs() {
        let config = Config::default();
        assert_eq!(config.frontend, Frontend::Llvm);
        assert_eq!(config.analysis.mode, AnalysisMode::Precise);
        assert_eq!(
            config.external_side_effects,
            ExternalSideEffects::UnknownReadwrite
        );
    }

    #[test]
    fn serde_json_backward_compatibility() {
        // Verify that the JSON serialized form matches the old string values.
        let config = Config::default();
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["frontend"], "llvm");
        assert_eq!(json["analysis"]["mode"], "precise");
        assert_eq!(json["external_side_effects"], "unknown_readwrite");

        // Verify air-json variant preserves hyphen.
        let air_json = serde_json::to_value(Frontend::AirJson).unwrap();
        assert_eq!(air_json, "air-json");

        // Verify deserialization from old string values still works.
        let old_json = r#"{
            "frontend": "air-json",
            "analysis": { "mode": "fast" },
            "external_side_effects": "unknown_write"
        }"#;
        let parsed: Config = serde_json::from_str(old_json).unwrap();
        assert_eq!(parsed.frontend, Frontend::AirJson);
        assert_eq!(parsed.analysis.mode, AnalysisMode::Fast);
        assert_eq!(
            parsed.external_side_effects,
            ExternalSideEffects::UnknownWrite
        );
    }

    #[test]
    fn custom_frontend_roundtrips() {
        let frontend = Frontend::Other("my-custom-frontend".to_string());
        let json = serde_json::to_string(&frontend).unwrap();
        // Untagged variant serializes as just the string
        assert_eq!(json, "\"my-custom-frontend\"");

        let parsed: Frontend = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Frontend::Other("my-custom-frontend".to_string()));
    }

    #[test]
    fn known_frontends_still_deserialize_correctly() {
        // "llvm" should still deserialize to Frontend::Llvm, not Frontend::Other("llvm")
        let parsed: Frontend = serde_json::from_str("\"llvm\"").unwrap();
        assert_eq!(parsed, Frontend::Llvm);

        let parsed: Frontend = serde_json::from_str("\"air-json\"").unwrap();
        assert_eq!(parsed, Frontend::AirJson);
    }
}
