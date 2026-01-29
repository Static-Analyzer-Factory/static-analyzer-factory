//! Multi-Threaded Analysis (MTA) module.
//!
//! This module implements May-Happen-in-Parallel (MHP) analysis for concurrent programs.
//! It models pthread-based threading constructs to determine which code sections can
//! execute concurrently.
//!
//! # Architecture
//!
//! The MTA analysis follows these phases:
//! 1. **Thread Discovery**: Find all `pthread_create` call sites
//! 2. **Thread Context Assignment**: Assign unique `ThreadId` based on calling context
//! 3. **Synchronization Analysis**: Track `pthread_join` constraints
//! 4. **MHP Computation**: Determine which threads can run in parallel at each program point
//!
//! # Example
//!
//! ```ignore
//! use saf_analysis::mta::{MtaAnalysis, MtaConfig};
//!
//! let mta = MtaAnalysis::new(&module, &callgraph, &icfg, config);
//! let result = mta.analyze();
//!
//! // Check if two program points may execute concurrently
//! let concurrent = result.may_run_concurrently(thread_a, thread_b);
//! ```

mod discovery;
mod export;
pub mod lockset;
mod mhp;
mod types;

pub use discovery::ThreadDiscovery;
pub use export::{ConcurrencyPair, EXPORT_SCHEMA_VERSION, ExportedThread, MhpPoint, MtaExport};
pub use lockset::{
    LockSet, LockSetResult, RaceCheckResult, check_race_with_locks, compute_locksets,
    compute_module_locksets,
};
pub use mhp::MhpAnalysis;
pub use types::{
    AccessKind, CallsiteLabel, MemoryAccess, MhpResult, ThreadConcurrencyGraph, ThreadContext,
    ThreadId,
};

use saf_core::air::AirModule;
use saf_core::ids::InstId;
use std::collections::{BTreeMap, BTreeSet};

use crate::callgraph::CallGraph;
use crate::icfg::Icfg;
use crate::pta::{LocationFactory, PointsToMap};

/// Configuration for MTA analysis.
#[derive(Debug, Clone)]
pub struct MtaConfig {
    /// Maximum context depth for thread identification (similar to k-CFA).
    /// Higher values provide more precision but increase cost.
    pub max_context_depth: usize,

    /// Whether to track lock sets for race detection.
    pub track_locks: bool,

    /// Function names recognized as thread creation (default: `pthread_create`).
    pub thread_create_funcs: Vec<String>,

    /// Function names recognized as thread join (default: `pthread_join`).
    pub thread_join_funcs: Vec<String>,

    /// Function names recognized as lock acquisition.
    pub lock_funcs: Vec<String>,

    /// Function names recognized as lock release.
    pub unlock_funcs: Vec<String>,
}

impl Default for MtaConfig {
    fn default() -> Self {
        Self {
            max_context_depth: 10,
            track_locks: false,
            thread_create_funcs: vec!["pthread_create".to_string()],
            thread_join_funcs: vec!["pthread_join".to_string()],
            lock_funcs: vec!["pthread_mutex_lock".to_string()],
            unlock_funcs: vec!["pthread_mutex_unlock".to_string()],
        }
    }
}

/// Main MTA analysis entry point.
pub struct MtaAnalysis<'a> {
    module: &'a AirModule,
    callgraph: &'a CallGraph,
    icfg: &'a Icfg,
    config: MtaConfig,
    /// Optional PTA points-to map for resolving function pointers in pthread_create.
    pts: Option<&'a PointsToMap>,
    /// Optional location factory for PTA-based resolution.
    factory: Option<&'a LocationFactory>,
}

impl<'a> MtaAnalysis<'a> {
    /// Create a new MTA analysis context without PTA.
    pub fn new(
        module: &'a AirModule,
        callgraph: &'a CallGraph,
        icfg: &'a Icfg,
        config: MtaConfig,
    ) -> Self {
        Self {
            module,
            callgraph,
            icfg,
            config,
            pts: None,
            factory: None,
        }
    }

    /// Create a new MTA analysis context with PTA results.
    ///
    /// When PTA results are provided, thread functions passed to `pthread_create`
    /// via function pointers can be resolved.
    pub fn with_pta(
        module: &'a AirModule,
        callgraph: &'a CallGraph,
        icfg: &'a Icfg,
        config: MtaConfig,
        pts: &'a PointsToMap,
        factory: &'a LocationFactory,
    ) -> Self {
        Self {
            module,
            callgraph,
            icfg,
            config,
            pts: Some(pts),
            factory: Some(factory),
        }
    }

    /// Run the MTA analysis and return the result.
    pub fn analyze(&self) -> MtaResult {
        // Phase 1: Discover thread creation sites (with optional PTA)
        let discovery = ThreadDiscovery::with_pta(
            self.module,
            self.callgraph,
            &self.config,
            self.pts,
            self.factory,
        );
        let thread_graph = discovery.discover();

        // Phase 2: Compute MHP relationships
        let mhp_analysis = MhpAnalysis::new(self.module, self.icfg, &thread_graph, &self.config);
        let mhp_result = mhp_analysis.compute();

        MtaResult {
            thread_graph,
            mhp_result,
        }
    }
}

/// Result of MTA analysis.
#[derive(Debug, Clone)]
pub struct MtaResult {
    /// Thread discovery and concurrency graph.
    pub thread_graph: ThreadConcurrencyGraph,

    /// May-happen-in-parallel results per program point.
    pub mhp_result: MhpResult,
}

impl MtaResult {
    /// Get all discovered threads.
    pub fn threads(&self) -> &BTreeMap<ThreadId, ThreadContext> {
        &self.thread_graph.threads
    }

    /// Check if two threads may run concurrently.
    pub fn may_run_concurrently(&self, t1: ThreadId, t2: ThreadId) -> bool {
        self.thread_graph.may_run_concurrently(&t1, &t2)
    }

    /// Get threads that may execute concurrently at a given program point.
    pub fn concurrent_threads_at(
        &self,
        thread_id: ThreadId,
        inst_id: InstId,
    ) -> BTreeSet<ThreadId> {
        self.mhp_result.concurrent_at(thread_id, inst_id)
    }

    /// Export results to JSON format.
    pub fn export(&self) -> MtaExport {
        MtaExport::from_result(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MtaConfig::default();
        assert_eq!(config.max_context_depth, 10);
        assert!(
            config
                .thread_create_funcs
                .contains(&"pthread_create".to_string())
        );
        assert!(
            config
                .thread_join_funcs
                .contains(&"pthread_join".to_string())
        );
    }
}

// Silence unused warnings for items that are publicly exported but not yet used internally
#[allow(unused_imports)]
use saf_core::ids::FunctionId as _;
