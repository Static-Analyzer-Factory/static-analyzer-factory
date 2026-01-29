//! May-Happen-in-Parallel (MHP) analysis.
//!
//! This module computes which threads may execute concurrently at each program point.

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, InstId};
use std::collections::{BTreeSet, VecDeque};

use crate::icfg::Icfg;

use super::MtaConfig;
use super::types::{MhpResult, ThreadConcurrencyGraph, ThreadId};

/// MHP analysis engine.
pub struct MhpAnalysis<'a> {
    module: &'a AirModule,
    #[allow(dead_code)]
    icfg: &'a Icfg,
    thread_graph: &'a ThreadConcurrencyGraph,
    #[allow(dead_code)]
    config: &'a MtaConfig,
}

impl<'a> MhpAnalysis<'a> {
    /// Create a new MHP analysis.
    pub fn new(
        module: &'a AirModule,
        icfg: &'a Icfg,
        thread_graph: &'a ThreadConcurrencyGraph,
        config: &'a MtaConfig,
    ) -> Self {
        Self {
            module,
            icfg,
            thread_graph,
            config,
        }
    }

    /// Compute MHP information for all program points.
    pub fn compute(&self) -> MhpResult {
        let mut result = MhpResult::new();

        // For each thread, compute which other threads may be concurrent at each instruction
        for &thread_id in self.thread_graph.threads.keys() {
            self.compute_for_thread(thread_id, &mut result);
        }

        result
    }

    /// Compute MHP for a single thread.
    fn compute_for_thread(&self, thread_id: ThreadId, result: &mut MhpResult) {
        let Some(thread_ctx) = self.thread_graph.get_thread(thread_id) else {
            return;
        };

        // Get the entry function for this thread
        let entry_fn = thread_ctx.entry_function;

        // Get all functions reachable from the entry
        let reachable_funcs = self.get_reachable_functions(entry_fn);

        // For each instruction in reachable functions, compute concurrent threads
        for func in &self.module.functions {
            if !reachable_funcs.contains(&func.id) {
                continue;
            }

            for block in &func.blocks {
                for inst in &block.instructions {
                    let concurrent = self.compute_concurrent_at(thread_id, inst.id);
                    result.set_concurrent_at(thread_id, inst.id, concurrent);
                }
            }
        }
    }

    /// Get all functions reachable from an entry point.
    fn get_reachable_functions(&self, entry: FunctionId) -> BTreeSet<FunctionId> {
        let mut reachable = BTreeSet::new();
        let mut worklist = VecDeque::new();
        worklist.push_back(entry);

        while let Some(func_id) = worklist.pop_front() {
            if reachable.contains(&func_id) {
                continue;
            }
            reachable.insert(func_id);

            // Find callees from this function using ICFG
            // For now, do a simple scan of the function's calls
            if let Some(func) = self.module.functions.iter().find(|f| f.id == func_id) {
                for block in &func.blocks {
                    for inst in &block.instructions {
                        if let Operation::CallDirect { callee } = &inst.op {
                            if !reachable.contains(callee) {
                                worklist.push_back(*callee);
                            }
                        }
                    }
                }
            }
        }

        reachable
    }

    /// Compute which threads may be concurrent at a specific instruction.
    fn compute_concurrent_at(&self, thread_id: ThreadId, inst_id: InstId) -> BTreeSet<ThreadId> {
        let mut concurrent = BTreeSet::new();

        // Always include self
        concurrent.insert(thread_id);

        // Add threads that may run concurrently with this thread
        // This is based on:
        // 1. The thread graph's concurrency relation
        // 2. Whether we're before/after any join points
        for &other_id in self.thread_graph.threads.keys() {
            if other_id != thread_id
                && self
                    .thread_graph
                    .may_run_concurrently(&thread_id, &other_id)
            {
                // Check if there's a join constraint that would prevent concurrency
                if !self.is_joined_before(thread_id, other_id, inst_id) {
                    concurrent.insert(other_id);
                }
            }
        }

        concurrent
    }

    /// Check if a thread has been joined before reaching an instruction.
    ///
    /// This is a simplified check - a full implementation would need to track
    /// the join points along CFG paths.
    #[allow(clippy::similar_names)]
    fn is_joined_before(
        &self,
        waiting_thread: ThreadId,
        joined_thread: ThreadId,
        _inst_id: InstId,
    ) -> bool {
        // Check if there's a join constraint for this pair
        // Would need path analysis to determine if inst_id is after join_site
        // For now, conservative: assume not joined
        let _ = self
            .thread_graph
            .join_constraints
            .contains_key(&(waiting_thread, joined_thread));
        false
    }
}

#[cfg(test)]
mod tests {
    // Integration tests in crates/saf-analysis/tests/mta_e2e.rs
}
