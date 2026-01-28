//! `ValueFlow` configuration types.
//!
//! Configures analysis mode, location inclusion, and transform propagation
//! for value flow graph construction.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use saf_core::air::BinaryOp;

/// `ValueFlow` analysis mode.
///
/// Controls whether the analysis uses PTA results for memory precision
/// or falls back to a fast approximation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValueFlowMode {
    /// Use PTA for memory precision.
    #[default]
    Precise,
    /// All memory through `unknown_mem` (no PTA required).
    Fast,
}

/// Which memory locations to include as graph nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IncludeLocations {
    /// All PTA locations become nodes.
    All,
    /// Only locations with STORE/LOAD operations.
    #[default]
    Accessed,
    /// No location nodes (values only, memory implicit).
    None,
}

/// Which operations create TRANSFORM edges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TransformPropagation {
    /// All binary/unary ops create edges (default, sound).
    #[default]
    All,
    /// Only whitelisted operations create edges.
    Whitelist {
        /// Set of operations that create TRANSFORM edges.
        ops: BTreeSet<OpKind>,
    },
}

/// Kind of operation for transform whitelist.
///
/// Simplified operation kind for configuration purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpKind {
    // Integer arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    // Bitwise
    And,
    Or,
    Xor,
    Shl,
    Shr,
    // Comparisons
    Cmp,
    // Casts
    Cast,
    // Memory operations that produce derived pointers
    Gep,
}

impl From<BinaryOp> for OpKind {
    fn from(op: BinaryOp) -> Self {
        match op {
            BinaryOp::Add | BinaryOp::FAdd => OpKind::Add,
            BinaryOp::Sub | BinaryOp::FSub => OpKind::Sub,
            BinaryOp::Mul | BinaryOp::FMul => OpKind::Mul,
            BinaryOp::UDiv | BinaryOp::SDiv | BinaryOp::FDiv => OpKind::Div,
            BinaryOp::URem | BinaryOp::SRem | BinaryOp::FRem => OpKind::Rem,
            BinaryOp::And => OpKind::And,
            BinaryOp::Or => OpKind::Or,
            BinaryOp::Xor => OpKind::Xor,
            BinaryOp::Shl => OpKind::Shl,
            BinaryOp::LShr | BinaryOp::AShr => OpKind::Shr,
            BinaryOp::ICmpEq
            | BinaryOp::ICmpNe
            | BinaryOp::ICmpUgt
            | BinaryOp::ICmpUge
            | BinaryOp::ICmpUlt
            | BinaryOp::ICmpUle
            | BinaryOp::ICmpSgt
            | BinaryOp::ICmpSge
            | BinaryOp::ICmpSlt
            | BinaryOp::ICmpSle
            | BinaryOp::FCmpOeq
            | BinaryOp::FCmpOne
            | BinaryOp::FCmpOgt
            | BinaryOp::FCmpOge
            | BinaryOp::FCmpOlt
            | BinaryOp::FCmpOle => OpKind::Cmp,
        }
    }
}

/// `ValueFlow` graph configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueFlowConfig {
    /// Analysis mode (precise or fast).
    #[serde(default)]
    pub mode: ValueFlowMode,

    /// Which memory locations to include as nodes.
    #[serde(default)]
    pub include_locations: IncludeLocations,

    /// Max locations per STORE/LOAD before collapsing to `unknown_mem`.
    #[serde(default = "default_max_locations_per_access")]
    pub max_locations_per_access: usize,

    /// Collapse field paths to base object.
    #[serde(default)]
    pub collapse_field_paths: bool,

    /// Which operations create TRANSFORM edges.
    #[serde(default)]
    pub transform_propagation: TransformPropagation,
}

impl Default for ValueFlowConfig {
    fn default() -> Self {
        Self {
            mode: ValueFlowMode::default(),
            include_locations: IncludeLocations::default(),
            max_locations_per_access: default_max_locations_per_access(),
            collapse_field_paths: false,
            transform_propagation: TransformPropagation::default(),
        }
    }
}

fn default_max_locations_per_access() -> usize {
    100
}

impl ValueFlowConfig {
    /// Create a new config with precise mode (default).
    #[must_use]
    pub fn precise() -> Self {
        Self::default()
    }

    /// Create a new config with fast mode (no PTA).
    #[must_use]
    pub fn fast() -> Self {
        Self {
            mode: ValueFlowMode::Fast,
            ..Self::default()
        }
    }

    /// Check if the config requires PTA.
    #[must_use]
    pub fn requires_pta(&self) -> bool {
        matches!(self.mode, ValueFlowMode::Precise)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_default_is_precise() {
        assert_eq!(ValueFlowMode::default(), ValueFlowMode::Precise);
    }

    #[test]
    fn include_locations_default_is_accessed() {
        assert_eq!(IncludeLocations::default(), IncludeLocations::Accessed);
    }

    #[test]
    fn transform_propagation_default_is_all() {
        assert_eq!(TransformPropagation::default(), TransformPropagation::All);
    }

    #[test]
    fn config_default_values() {
        let config = ValueFlowConfig::default();
        assert_eq!(config.mode, ValueFlowMode::Precise);
        assert_eq!(config.include_locations, IncludeLocations::Accessed);
        assert_eq!(config.max_locations_per_access, 100);
        assert!(!config.collapse_field_paths);
        assert_eq!(config.transform_propagation, TransformPropagation::All);
    }

    #[test]
    fn config_precise_requires_pta() {
        let config = ValueFlowConfig::precise();
        assert!(config.requires_pta());
    }

    #[test]
    fn config_fast_does_not_require_pta() {
        let config = ValueFlowConfig::fast();
        assert!(!config.requires_pta());
    }

    #[test]
    fn mode_serialization_roundtrip() {
        let modes = vec![ValueFlowMode::Precise, ValueFlowMode::Fast];
        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let parsed: ValueFlowMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, parsed);
        }
    }

    #[test]
    fn include_locations_serialization_roundtrip() {
        let variants = vec![
            IncludeLocations::All,
            IncludeLocations::Accessed,
            IncludeLocations::None,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let parsed: IncludeLocations = serde_json::from_str(&json).unwrap();
            assert_eq!(v, parsed);
        }
    }

    #[test]
    fn transform_propagation_all_serializes() {
        let tp = TransformPropagation::All;
        let json = serde_json::to_string(&tp).unwrap();
        assert_eq!(json, r#"{"kind":"all"}"#);
    }

    #[test]
    fn transform_propagation_whitelist_serializes() {
        let mut ops = BTreeSet::new();
        ops.insert(OpKind::Add);
        ops.insert(OpKind::Sub);
        let tp = TransformPropagation::Whitelist { ops };
        let json = serde_json::to_string(&tp).unwrap();
        let parsed: TransformPropagation = serde_json::from_str(&json).unwrap();
        assert_eq!(tp, parsed);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = ValueFlowConfig {
            mode: ValueFlowMode::Fast,
            include_locations: IncludeLocations::None,
            max_locations_per_access: 50,
            collapse_field_paths: true,
            transform_propagation: TransformPropagation::All,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ValueFlowConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn op_kind_from_binary_op() {
        assert_eq!(OpKind::from(BinaryOp::Add), OpKind::Add);
        assert_eq!(OpKind::from(BinaryOp::FAdd), OpKind::Add);
        assert_eq!(OpKind::from(BinaryOp::Sub), OpKind::Sub);
        assert_eq!(OpKind::from(BinaryOp::UDiv), OpKind::Div);
        assert_eq!(OpKind::from(BinaryOp::And), OpKind::And);
        assert_eq!(OpKind::from(BinaryOp::ICmpEq), OpKind::Cmp);
    }

    #[test]
    fn op_kind_ordering() {
        // OpKind should be orderable for BTreeSet
        let mut set = BTreeSet::new();
        set.insert(OpKind::Sub);
        set.insert(OpKind::Add);
        set.insert(OpKind::Mul);

        let v: Vec<_> = set.iter().collect();
        assert_eq!(v.len(), 3);
    }
}
