//! Loop invariant synthesis for verification.
//!
//! This module provides template-based loop invariant synthesis to enable
//! verification of loop properties. The synthesis algorithm:
//!
//! 1. Identifies loops in the CFG
//! 2. Generates candidate invariant templates from loop structure
//! 3. Checks each candidate using abstract interpretation
//! 4. Returns valid invariants for use in verification
//!
//! # Algorithm
//!
//! ```text
//! for each loop L with header H:
//!     candidates := generate_templates(H, body(L))
//!     for each template T in candidates:
//!         result := check_invariant(T, H, body(L))
//!         if result == Valid:
//!             add T to invariants[H]
//! return invariants
//! ```
//!
//! # Example
//!
//! ```ignore
//! use saf_analysis::invariants::{synthesize_invariants, InvariantConfig};
//!
//! let config = InvariantConfig::default();
//! let invariants = synthesize_invariants(&func, &cfg, &module, &config);
//!
//! // Use invariants for verification
//! for (header, invs) in &invariants {
//!     println!("Loop at {:?} has {} invariants", header, invs.len());
//! }
//! ```

pub mod checker;
pub mod templates;

pub use checker::{CheckerConfig, InvariantCheckResult, StrengtheningHints, check_invariant};
pub use templates::{InvariantTemplate, generate_templates};

use std::collections::BTreeMap;

use saf_core::air::{AirFunction, AirModule};
use saf_core::ids::BlockId;

use crate::cfg::Cfg;

/// Configuration for invariant synthesis.
#[derive(Clone, Debug)]
pub struct InvariantConfig {
    /// Maximum templates to generate per loop.
    pub max_templates: usize,
    /// Maximum valid invariants to keep per loop.
    pub max_invariants: usize,
    /// Configuration for invariant checking.
    pub checker: CheckerConfig,
    /// Whether to attempt strengthening of weak invariants.
    pub try_strengthening: bool,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            max_templates: 50,
            max_invariants: 20,
            checker: CheckerConfig::default(),
            try_strengthening: false,
        }
    }
}

/// Result of invariant synthesis.
#[derive(Clone, Debug)]
pub struct SynthesisResult {
    /// Valid invariants per loop header.
    pub invariants: BTreeMap<BlockId, Vec<InvariantTemplate>>,
    /// Statistics about the synthesis process.
    pub stats: SynthesisStats,
}

/// Statistics from invariant synthesis.
#[derive(Clone, Debug, Default)]
pub struct SynthesisStats {
    /// Number of loops analyzed.
    pub loops_analyzed: usize,
    /// Total templates generated.
    pub templates_generated: usize,
    /// Templates that were valid.
    pub templates_valid: usize,
    /// Templates that were invalid.
    pub templates_invalid: usize,
    /// Templates with unknown status.
    pub templates_unknown: usize,
}

/// Synthesize loop invariants for a function.
///
/// Analyzes all loops in the function and attempts to synthesize
/// invariants that hold at each loop header.
///
/// # Arguments
///
/// * `func` - The function to analyze
/// * `cfg` - Control flow graph for the function
/// * `module` - The AIR module
/// * `config` - Synthesis configuration
///
/// # Returns
///
/// A mapping from loop header block IDs to their valid invariants.
#[must_use]
pub fn synthesize_invariants(
    func: &AirFunction,
    cfg: &Cfg,
    module: &AirModule,
    config: &InvariantConfig,
) -> SynthesisResult {
    let mut invariants: BTreeMap<BlockId, Vec<InvariantTemplate>> = BTreeMap::new();
    let mut stats = SynthesisStats::default();

    // Find all loops (back edges in CFG)
    let loops = find_loops(cfg);
    stats.loops_analyzed = loops.len();

    for (header, body) in loops {
        // Generate candidate templates
        let candidates = generate_templates(header, &body, func, module);
        let num_candidates = candidates.len().min(config.max_templates);
        stats.templates_generated += num_candidates;

        let mut valid_invariants = Vec::new();

        // Check each candidate
        for template in candidates.into_iter().take(config.max_templates) {
            let result =
                check_invariant(&template, header, &body, cfg, func, module, &config.checker);

            match result {
                InvariantCheckResult::Valid => {
                    stats.templates_valid += 1;
                    if valid_invariants.len() < config.max_invariants {
                        valid_invariants.push(template);
                    }
                }
                InvariantCheckResult::Invalid { .. } => {
                    stats.templates_invalid += 1;
                }
                InvariantCheckResult::Unknown { .. } => {
                    stats.templates_unknown += 1;
                }
            }
        }

        if !valid_invariants.is_empty() {
            invariants.insert(header, valid_invariants);
        }
    }

    SynthesisResult { invariants, stats }
}

/// Find all natural loops in the CFG.
///
/// Returns a map from loop header to the set of blocks in the loop body.
fn find_loops(cfg: &Cfg) -> Vec<(BlockId, Vec<BlockId>)> {
    let mut loops = Vec::new();

    // Find back edges: edges where target dominates source
    // Simplified: look for blocks that have successors appearing as predecessors
    for block in cfg.blocks() {
        if let Some(succs) = cfg.successors_of(block) {
            for succ in succs {
                // Check if this is a back edge (succ appears before block in some path)
                // Simplified heuristic: if succ has block as a successor path, it's a loop
                if is_back_edge(block, *succ, cfg) {
                    // succ is a loop header
                    let body = collect_loop_body(*succ, block, cfg);
                    loops.push((*succ, body));
                }
            }
        }
    }

    loops
}

/// Check if an edge from source to target is a back edge.
fn is_back_edge(source: BlockId, target: BlockId, cfg: &Cfg) -> bool {
    // A back edge goes from a block to an earlier block in some path
    // Simplified check: target can reach source through CFG
    let mut visited = std::collections::BTreeSet::new();
    let mut worklist = vec![target];

    while let Some(block) = worklist.pop() {
        if block == source {
            return true;
        }
        if visited.contains(&block) {
            continue;
        }
        visited.insert(block);

        if let Some(succs) = cfg.successors_of(block) {
            for succ in succs {
                if !visited.contains(succ) {
                    worklist.push(*succ);
                }
            }
        }
    }

    false
}

/// Collect all blocks in a loop body.
fn collect_loop_body(header: BlockId, latch: BlockId, cfg: &Cfg) -> Vec<BlockId> {
    let mut body = vec![header];
    let mut worklist = vec![latch];
    let mut visited = std::collections::BTreeSet::new();
    visited.insert(header);

    while let Some(block) = worklist.pop() {
        if visited.contains(&block) {
            continue;
        }
        visited.insert(block);
        body.push(block);

        // Add predecessors that aren't already visited
        if let Some(preds) = cfg.predecessors_of(block) {
            for pred in preds {
                if !visited.contains(pred) {
                    worklist.push(*pred);
                }
            }
        }
    }

    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::AirFunction;
    use saf_core::ids::{FunctionId, ModuleId};

    fn make_empty_function() -> AirFunction {
        AirFunction {
            id: FunctionId::new(1),
            name: "test".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_empty_module() -> AirModule {
        AirModule::new(ModuleId::new(0))
    }

    #[test]
    fn test_config_defaults() {
        let config = InvariantConfig::default();
        assert_eq!(config.max_templates, 50);
        assert_eq!(config.max_invariants, 20);
        assert!(!config.try_strengthening);
    }

    #[test]
    fn test_synthesize_empty_function() {
        let func = make_empty_function();
        let cfg = Cfg::build(&func);
        let module = make_empty_module();
        let config = InvariantConfig::default();

        let result = synthesize_invariants(&func, &cfg, &module, &config);

        assert!(result.invariants.is_empty());
        assert_eq!(result.stats.loops_analyzed, 0);
    }

    #[test]
    fn test_synthesis_stats_default() {
        let stats = SynthesisStats::default();
        assert_eq!(stats.loops_analyzed, 0);
        assert_eq!(stats.templates_generated, 0);
        assert_eq!(stats.templates_valid, 0);
    }

    #[test]
    fn test_find_loops_no_loops() {
        let func = make_empty_function();
        let cfg = Cfg::build(&func);

        let loops = find_loops(&cfg);
        assert!(loops.is_empty());
    }
}
