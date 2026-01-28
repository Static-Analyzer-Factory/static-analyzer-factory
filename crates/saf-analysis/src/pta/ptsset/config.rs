//! Configuration for points-to set representation selection.
//!
//! Provides types for selecting and configuring points-to set implementations
//! based on program characteristics.

use saf_core::air::AirModule;
use serde::{Deserialize, Serialize};

/// Points-to set representation selection.
///
/// Controls which internal representation is used for points-to sets
/// during pointer analysis. Different representations have different
/// performance characteristics:
///
/// - `BTreeSet`: Simple, good for small programs (<10K allocations)
/// - `BitVector`: Fast set operations for medium programs (10K-100K)
/// - `Bdd`: Memory-efficient for large programs with shared structure (>100K)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtsRepresentation {
    /// Automatically select based on program size.
    ///
    /// Uses `bitvec_threshold` and `bdd_threshold` from [`PtsConfig`]
    /// to determine the best representation.
    #[default]
    Auto,

    /// Use `BTreeSet<LocId>` (baseline implementation).
    ///
    /// Best for:
    /// - Small programs with <10K allocation sites
    /// - Debugging (deterministic, simple to inspect)
    /// - When memory is not a concern
    BTreeSet,

    /// Use `FxHashSet<LocId>` for O(1) operations.
    ///
    /// Best for:
    /// - Small to medium programs where BTreeSet's O(log n) is a bottleneck
    /// - Internal solver computation (iteration order doesn't affect fixed point)
    /// - Output is normalized to `BTreeSet` at the API boundary
    FxHash,

    /// Use bit-vector representation (via `bitvec` crate).
    ///
    /// Best for:
    /// - Medium programs with 10K-100K allocation sites
    /// - When fast membership testing is important
    /// - Programs with dense allocation site numbering
    BitVector,

    /// Use Roaring bitmap representation (via `roaring` crate).
    ///
    /// Best for:
    /// - Medium-to-large programs with 50K-100K allocation sites
    /// - Programs with mixed sparse and dense allocation numbering
    /// - When both compression and fast set operations matter
    Roaring,

    /// Use BDD (Binary Decision Diagram) representation.
    ///
    /// Best for:
    /// - Large programs with >100K allocation sites
    /// - Programs where many points-to sets share structure
    /// - When memory efficiency is critical
    Bdd,
}

/// Controls whether object clustering is applied as a preprocessing step.
///
/// Clustering groups frequently co-occurring locations into consecutive
/// bit positions, improving cache locality for bit-vector/BDD operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusteringMode {
    /// Enable for `BitVector`/`Bdd`, skip for `BTreeSet`.
    #[default]
    Auto,
    /// Always run clustering.
    Enabled,
    /// Never run clustering.
    Disabled,
}

impl PtsRepresentation {
    /// Parse from string (for Python/CLI interface).
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "btreeset" | "btree" => Some(Self::BTreeSet),
            "fxhash" | "fxhashset" => Some(Self::FxHash),
            "bitvector" | "bitvec" | "bv" => Some(Self::BitVector),
            "roaring" => Some(Self::Roaring),
            "bdd" => Some(Self::Bdd),
            _ => None,
        }
    }

    /// Convert to string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::BTreeSet => "btreeset",
            Self::FxHash => "fxhash",
            Self::BitVector => "bitvector",
            Self::Roaring => "roaring",
            Self::Bdd => "bdd",
        }
    }
}

/// Configuration for points-to set representation selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtsConfig {
    /// Which representation to use.
    pub representation: PtsRepresentation,

    /// Threshold for switching from BTreeSet to BitVector (in allocation sites).
    ///
    /// When `representation` is `Auto` and the program has more than this many
    /// allocation sites, BitVector will be used instead of BTreeSet.
    ///
    /// Default: 10,000
    pub bitvec_threshold: usize,

    /// Threshold for switching from BitVector to Roaring (in allocation sites).
    ///
    /// When `representation` is `Auto` and the program has more than this many
    /// allocation sites, Roaring will be used instead of BitVector.
    ///
    /// Default: 10,000
    pub roaring_threshold: usize,

    /// Threshold for switching from Roaring to BDD (in allocation sites).
    ///
    /// When `representation` is `Auto` and the program has more than this many
    /// allocation sites, BDD will be used instead of Roaring.
    ///
    /// Default: 100,000
    pub bdd_threshold: usize,

    /// Controls whether object clustering is applied as a preprocessing step.
    ///
    /// Clustering groups frequently co-occurring locations into consecutive
    /// bit positions, improving cache locality for bit-vector/BDD operations.
    #[serde(default)]
    pub clustering: ClusteringMode,
}

impl Default for PtsConfig {
    fn default() -> Self {
        Self {
            representation: PtsRepresentation::Auto,
            bitvec_threshold: 10_000,
            roaring_threshold: 10_000,
            bdd_threshold: 100_000,
            clustering: ClusteringMode::Auto,
        }
    }
}

impl PtsConfig {
    /// Create a new config with default thresholds.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a config that always uses BTreeSet.
    #[must_use]
    pub fn btreeset() -> Self {
        Self {
            representation: PtsRepresentation::BTreeSet,
            ..Self::default()
        }
    }

    /// Create a config that always uses BitVector.
    #[must_use]
    pub fn bitvector() -> Self {
        Self {
            representation: PtsRepresentation::BitVector,
            ..Self::default()
        }
    }

    /// Create a config that always uses BDD.
    #[must_use]
    pub fn bdd() -> Self {
        Self {
            representation: PtsRepresentation::Bdd,
            ..Self::default()
        }
    }

    /// Set the representation.
    #[must_use]
    pub fn with_representation(mut self, repr: PtsRepresentation) -> Self {
        self.representation = repr;
        self
    }

    /// Set the bitvec threshold.
    #[must_use]
    pub fn with_bitvec_threshold(mut self, threshold: usize) -> Self {
        self.bitvec_threshold = threshold;
        self
    }

    /// Set the BDD threshold.
    #[must_use]
    pub fn with_bdd_threshold(mut self, threshold: usize) -> Self {
        self.bdd_threshold = threshold;
        self
    }

    /// Select the appropriate representation for a module.
    ///
    /// If `representation` is `Auto`, uses allocation site count to decide.
    /// Otherwise returns the explicitly configured representation.
    #[must_use]
    pub fn select_for_module(&self, module: &AirModule) -> PtsRepresentation {
        match self.representation {
            PtsRepresentation::Auto => {
                let alloc_count = count_allocation_sites(module);
                self.select_by_count(alloc_count)
            }
            explicit => explicit,
        }
    }

    /// Create a config that always uses Roaring.
    #[must_use]
    pub fn roaring() -> Self {
        Self {
            representation: PtsRepresentation::Roaring,
            ..Self::default()
        }
    }

    /// Set the roaring threshold.
    #[must_use]
    pub fn with_roaring_threshold(mut self, threshold: usize) -> Self {
        self.roaring_threshold = threshold;
        self
    }

    /// Select representation based on allocation site count.
    #[must_use]
    pub fn select_by_count(&self, alloc_count: usize) -> PtsRepresentation {
        if alloc_count >= self.bdd_threshold {
            PtsRepresentation::Bdd
        } else if alloc_count >= self.roaring_threshold {
            PtsRepresentation::Roaring
        } else {
            // FxHash is faster than both BTreeSet and BitVector for typical
            // Andersen analysis workloads. Points-to sets are sparse (k << N),
            // so FxHash's O(k) operations beat BitVec's O(N/64) fixed cost.
            PtsRepresentation::FxHash
        }
    }
}

/// Count the number of allocation sites in a module.
///
/// This provides a rough estimate of program size for representation selection.
/// An "allocation site" is any instruction that creates a new memory object:
/// - `Alloca` (stack allocation)
/// - `HeapAlloc` (heap allocation)
/// - Global variables (implicit allocation)
///
/// # Example
///
/// ```ignore
/// use saf_analysis::pta::ptsset::count_allocation_sites;
///
/// let count = count_allocation_sites(&module);
/// println!("Program has {} allocation sites", count);
/// ```
#[must_use]
pub fn count_allocation_sites(module: &AirModule) -> usize {
    use saf_core::air::Operation;

    let mut count = 0;

    // Count global variables (each is an implicit allocation)
    count += module.globals.len();

    // Count stack and heap allocations in functions
    for func in &module.functions {
        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    Operation::Alloca { .. } | Operation::HeapAlloc { .. } => {
                        count += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pts_representation_default_is_auto() {
        assert_eq!(PtsRepresentation::default(), PtsRepresentation::Auto);
    }

    #[test]
    fn pts_config_default() {
        let config = PtsConfig::default();
        assert_eq!(config.representation, PtsRepresentation::Auto);
        assert_eq!(config.bitvec_threshold, 10_000);
        assert_eq!(config.roaring_threshold, 10_000);
        assert_eq!(config.bdd_threshold, 100_000);
    }

    #[test]
    fn pts_config_builders() {
        let btree = PtsConfig::btreeset();
        assert_eq!(btree.representation, PtsRepresentation::BTreeSet);

        let bitvec = PtsConfig::bitvector();
        assert_eq!(bitvec.representation, PtsRepresentation::BitVector);

        let roaring = PtsConfig::roaring();
        assert_eq!(roaring.representation, PtsRepresentation::Roaring);

        let bdd = PtsConfig::bdd();
        assert_eq!(bdd.representation, PtsRepresentation::Bdd);
    }

    #[test]
    fn select_by_count_custom_thresholds() {
        let config = PtsConfig::default()
            .with_roaring_threshold(500)
            .with_bdd_threshold(1000);

        assert_eq!(config.select_by_count(50), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(100), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(400), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(500), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(800), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(1000), PtsRepresentation::Bdd);
    }
}
