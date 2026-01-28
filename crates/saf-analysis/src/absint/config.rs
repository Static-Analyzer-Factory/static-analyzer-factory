//! Configuration for abstract interpretation analysis.

use serde::{Deserialize, Serialize};

use super::partition::PartitionConfig;

/// Configuration for the abstract interpretation engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbstractInterpConfig {
    /// Maximum widening iterations before forcing convergence.
    #[serde(default = "default_max_widening_iterations")]
    pub max_widening_iterations: u32,

    /// Number of narrowing iterations after ascending phase.
    #[serde(default = "default_narrowing_iterations")]
    pub narrowing_iterations: u32,

    /// Whether to extract widening thresholds from program constants.
    #[serde(default = "default_use_threshold_widening")]
    pub use_threshold_widening: bool,

    /// Maximum number of blocks to process (scalability bound).
    #[serde(default = "default_max_blocks")]
    pub max_blocks: u64,

    /// Trace partitioning configuration (Plan 148 Phase C).
    #[serde(skip)]
    pub partition: PartitionConfig,
}

fn default_max_widening_iterations() -> u32 {
    100
}
fn default_narrowing_iterations() -> u32 {
    3
}
fn default_use_threshold_widening() -> bool {
    true
}
fn default_max_blocks() -> u64 {
    100_000
}

impl Default for AbstractInterpConfig {
    fn default() -> Self {
        Self {
            max_widening_iterations: default_max_widening_iterations(),
            narrowing_iterations: default_narrowing_iterations(),
            use_threshold_widening: default_use_threshold_widening(),
            max_blocks: default_max_blocks(),
            partition: PartitionConfig::default(),
        }
    }
}
