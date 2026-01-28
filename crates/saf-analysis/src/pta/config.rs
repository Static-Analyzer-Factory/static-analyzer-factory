//! PTA configuration types.
//!
//! Configures field sensitivity, analysis bounds, and points-to set
//! representation for points-to analysis.

use serde::{Deserialize, Serialize};

use super::ptsset::{PtsConfig, PtsRepresentation};

/// Field sensitivity configuration for points-to analysis.
///
/// Controls whether and how the analysis tracks individual struct fields
/// as separate abstract memory locations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldSensitivity {
    /// No field tracking — each allocation is one abstract object.
    None,
    /// Track struct fields up to `max_depth` (0 = None, 1 = top-level only).
    StructFields {
        /// Maximum nesting depth for field tracking.
        max_depth: u32,
    },
}

impl Default for FieldSensitivity {
    fn default() -> Self {
        Self::StructFields { max_depth: 2 }
    }
}

/// Array index sensitivity configuration for points-to analysis.
///
/// Controls whether and how the analysis distinguishes array elements
/// at different indices. More precise modes can reduce false positives
/// but increase analysis cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndexSensitivity {
    /// No index tracking — all array elements collapse to one location.
    ///
    /// This is the default and matches traditional Andersen-style analysis.
    /// `a[0]` and `a[1]` are treated as aliasing.
    #[default]
    Collapsed,

    /// Distinguish constant indices (`a[0]` vs `a[1]`).
    ///
    /// Indices known at compile time are tracked separately.
    /// Unknown/symbolic indices still collapse.
    ConstantOnly,

    /// Track symbolic indices (requires Z3 at query time).
    ///
    /// Each index expression gets its own abstract location.
    /// At query time, Z3 determines if two symbolic indices may be equal.
    Symbolic,
}

/// Points-to analysis configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtaConfig {
    /// Whether points-to analysis is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Field sensitivity configuration.
    #[serde(default)]
    pub field_sensitivity: FieldSensitivity,

    /// Array index sensitivity configuration.
    ///
    /// Controls whether array indices are tracked separately:
    /// - `Collapsed`: All indices treated as one (default, classic behavior)
    /// - `ConstantOnly`: Distinguish constant indices (`a[0]` vs `a[1]`)
    /// - `Symbolic`: Track symbolic indices (uses Z3 at query time)
    #[serde(default)]
    pub index_sensitivity: IndexSensitivity,

    /// Maximum number of abstract objects before collapsing.
    ///
    /// **Note:** This field is currently reserved and not enforced by the
    /// solver. It is retained for configuration compatibility and future use.
    /// Setting it has no effect on analysis behavior.
    #[serde(default = "default_max_objects")]
    pub max_objects: usize,

    /// Maximum solver iterations before stopping.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    /// Points-to set representation configuration.
    ///
    /// Controls which internal data structure is used for points-to sets:
    /// - `Auto`: Select based on allocation site count (default)
    /// - `BTreeSet`: Simple baseline, good for small programs
    /// - `BitVector`: Fast operations for medium programs
    /// - `Bdd`: Compact representation for large programs
    #[serde(default)]
    pub pts_config: PtsConfig,

    /// Enable Z3-based index alias refinement for symbolic indices.
    ///
    /// When enabled and `index_sensitivity` is `Symbolic`, alias queries
    /// will use Z3 to check if symbolic indices may be equal.
    #[serde(default)]
    pub z3_index_enabled: bool,

    /// Timeout in milliseconds for Z3 index checks (default: 100ms).
    #[serde(default = "default_z3_index_timeout")]
    pub z3_index_timeout_ms: u64,

    /// Enable path-sensitive alias queries.
    ///
    /// When enabled, alias queries at a specific program point will
    /// consider branch conditions that dominate the query point.
    #[serde(default)]
    pub path_sensitive_alias: bool,

    /// Maximum paths to enumerate for path-sensitive queries (default: 16).
    #[serde(default = "default_max_paths")]
    pub path_sensitive_max_paths: usize,

    /// Timeout in milliseconds for path feasibility checks (default: 500ms).
    #[serde(default = "default_path_sensitive_timeout")]
    pub path_sensitive_timeout_ms: u64,
}

fn default_z3_index_timeout() -> u64 {
    100
}

fn default_max_paths() -> usize {
    16
}

fn default_path_sensitive_timeout() -> u64 {
    500
}

impl Default for PtaConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            field_sensitivity: FieldSensitivity::default(),
            index_sensitivity: IndexSensitivity::default(),
            max_objects: default_max_objects(),
            max_iterations: default_max_iterations(),
            pts_config: PtsConfig::default(),
            z3_index_enabled: false,
            z3_index_timeout_ms: default_z3_index_timeout(),
            path_sensitive_alias: false,
            path_sensitive_max_paths: default_max_paths(),
            path_sensitive_timeout_ms: default_path_sensitive_timeout(),
        }
    }
}

impl PtaConfig {
    /// Create a config that uses `BTreeSet` for points-to sets.
    #[must_use]
    pub fn with_btreeset(mut self) -> Self {
        self.pts_config = PtsConfig::btreeset();
        self
    }

    /// Create a config that uses `BitVector` for points-to sets.
    #[must_use]
    pub fn with_bitvector(mut self) -> Self {
        self.pts_config = PtsConfig::bitvector();
        self
    }

    /// Create a config that uses `BDD` for points-to sets.
    #[must_use]
    pub fn with_bdd(mut self) -> Self {
        self.pts_config = PtsConfig::bdd();
        self
    }

    /// Set the points-to set representation explicitly.
    #[must_use]
    pub fn with_pts_representation(mut self, repr: PtsRepresentation) -> Self {
        self.pts_config = self.pts_config.with_representation(repr);
        self
    }

    /// Enable constant-only index sensitivity.
    ///
    /// Distinguishes `a[0]` from `a[1]` but collapses symbolic indices.
    #[must_use]
    pub fn with_constant_indices(mut self) -> Self {
        self.index_sensitivity = IndexSensitivity::ConstantOnly;
        self
    }

    /// Enable symbolic index sensitivity.
    ///
    /// Tracks all index expressions; uses Z3 at query time to determine aliasing.
    #[must_use]
    pub fn with_symbolic_indices(mut self) -> Self {
        self.index_sensitivity = IndexSensitivity::Symbolic;
        self
    }

    /// Set index sensitivity explicitly.
    #[must_use]
    pub fn with_index_sensitivity(mut self, sensitivity: IndexSensitivity) -> Self {
        self.index_sensitivity = sensitivity;
        self
    }

    /// Enable Z3-based index alias refinement.
    ///
    /// When enabled with `Symbolic` index sensitivity, alias queries
    /// use Z3 to determine if symbolic indices may be equal.
    #[must_use]
    pub fn with_z3_index_refinement(mut self, enabled: bool) -> Self {
        self.z3_index_enabled = enabled;
        self
    }

    /// Set Z3 index check timeout in milliseconds.
    #[must_use]
    pub fn with_z3_index_timeout(mut self, timeout_ms: u64) -> Self {
        self.z3_index_timeout_ms = timeout_ms;
        self
    }

    /// Enable path-sensitive alias queries.
    ///
    /// When enabled, alias queries at a specific program point will
    /// consider branch conditions that dominate the query point.
    #[must_use]
    pub fn with_path_sensitive_alias(mut self, enabled: bool) -> Self {
        self.path_sensitive_alias = enabled;
        self
    }

    /// Set maximum paths to enumerate for path-sensitive queries.
    #[must_use]
    pub fn with_path_sensitive_max_paths(mut self, max_paths: usize) -> Self {
        self.path_sensitive_max_paths = max_paths;
        self
    }

    /// Set timeout for path feasibility checks in milliseconds.
    #[must_use]
    pub fn with_path_sensitive_timeout(mut self, timeout_ms: u64) -> Self {
        self.path_sensitive_timeout_ms = timeout_ms;
        self
    }
}

fn default_enabled() -> bool {
    true
}

fn default_max_objects() -> usize {
    100_000
}

fn default_max_iterations() -> usize {
    1_000_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_sensitivity_none_serializes_correctly() {
        let fs = FieldSensitivity::None;
        let json = serde_json::to_string(&fs).unwrap();
        assert_eq!(json, r#"{"kind":"none"}"#);
    }

    #[test]
    fn field_sensitivity_struct_fields_serializes_correctly() {
        let fs = FieldSensitivity::StructFields { max_depth: 3 };
        let json = serde_json::to_string(&fs).unwrap();
        assert_eq!(json, r#"{"kind":"struct_fields","max_depth":3}"#);
    }

    #[test]
    fn field_sensitivity_roundtrip() {
        let variants = vec![
            FieldSensitivity::None,
            FieldSensitivity::StructFields { max_depth: 0 },
            FieldSensitivity::StructFields { max_depth: 5 },
        ];
        for fs in variants {
            let json = serde_json::to_string(&fs).unwrap();
            let parsed: FieldSensitivity = serde_json::from_str(&json).unwrap();
            assert_eq!(fs, parsed);
        }
    }

    #[test]
    fn pta_config_default_values() {
        let config = PtaConfig::default();
        assert!(config.enabled);
        assert_eq!(
            config.field_sensitivity,
            FieldSensitivity::StructFields { max_depth: 2 }
        );
        assert_eq!(config.max_objects, 100_000);
        assert_eq!(config.max_iterations, 1_000_000);
    }

    #[test]
    fn pta_config_serialization_roundtrip() {
        let config = PtaConfig {
            enabled: false,
            field_sensitivity: FieldSensitivity::None,
            index_sensitivity: IndexSensitivity::Collapsed,
            max_objects: 50_000,
            max_iterations: 500_000,
            pts_config: PtsConfig::default(),
            z3_index_enabled: true,
            z3_index_timeout_ms: 200,
            path_sensitive_alias: true,
            path_sensitive_max_paths: 32,
            path_sensitive_timeout_ms: 1000,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: PtaConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn pta_config_disabled() {
        let config = PtaConfig {
            enabled: false,
            ..PtaConfig::default()
        };
        assert!(!config.enabled);
    }

    #[test]
    fn index_sensitivity_default_is_collapsed() {
        assert_eq!(IndexSensitivity::default(), IndexSensitivity::Collapsed);
    }

    #[test]
    fn index_sensitivity_serialization_roundtrip() {
        let variants = vec![
            IndexSensitivity::Collapsed,
            IndexSensitivity::ConstantOnly,
            IndexSensitivity::Symbolic,
        ];
        for is in variants {
            let json = serde_json::to_string(&is).unwrap();
            let parsed: IndexSensitivity = serde_json::from_str(&json).unwrap();
            assert_eq!(is, parsed);
        }
    }

    #[test]
    fn pta_config_with_constant_indices() {
        let config = PtaConfig::default().with_constant_indices();
        assert_eq!(config.index_sensitivity, IndexSensitivity::ConstantOnly);
    }

    #[test]
    fn pta_config_with_symbolic_indices() {
        let config = PtaConfig::default().with_symbolic_indices();
        assert_eq!(config.index_sensitivity, IndexSensitivity::Symbolic);
    }
}
