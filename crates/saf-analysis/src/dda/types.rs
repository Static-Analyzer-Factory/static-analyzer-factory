//! Core data structures for demand-driven pointer analysis (DDA).
//!
//! DDA computes points-to information only for explicitly queried pointers,
//! using CFL-reachability for context-sensitive backward traversal on SVFG.
//!
//! See Plan 043 for full design documentation.

use crate::timer::Timer;
use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::pta::ptsset::{IdBitSet, Indexer, PtsConfig, PtsRepresentation};
use crate::svfg::SvfgNodeId;

pub use crate::svfg::context::CallString;

// ---------------------------------------------------------------------------
// Dpm (Demand-Driven Points-To Message)
// ---------------------------------------------------------------------------

/// A demand-driven points-to message (DPM).
///
/// Represents a node being traced during backward traversal, along with
/// its calling context. Used as worklist elements and cache keys.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dpm {
    /// Current SVFG node being traced.
    pub node: SvfgNodeId,
    /// Calling context for CFL matching.
    pub context: CallString,
}

impl Dpm {
    /// Create a new DPM with empty context.
    #[must_use]
    pub fn new(node: SvfgNodeId) -> Self {
        Self {
            node,
            context: CallString::empty(),
        }
    }

    /// Create a new DPM with explicit context.
    #[must_use]
    pub fn with_context(node: SvfgNodeId, context: CallString) -> Self {
        Self { node, context }
    }
}

// ---------------------------------------------------------------------------
// DdaConfig
// ---------------------------------------------------------------------------

/// Configuration for demand-driven pointer analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdaConfig {
    /// Maximum number of traversal steps per query. 0 = unlimited.
    pub max_steps: usize,
    /// Maximum call-string depth. 0 = unlimited.
    pub max_context_depth: usize,
    /// Timeout in milliseconds per query. 0 = unlimited.
    pub timeout_ms: u64,
    /// Enable strong update optimization.
    pub enable_strong_updates: bool,
    /// Points-to set representation configuration.
    ///
    /// Note: DDA currently uses `IdBitSet<LocId>` internally for its cache
    /// regardless of this setting, as it operates demand-driven and uses
    /// CI-PTA fallback. This field is provided for API consistency and
    /// future extension.
    #[serde(default)]
    pub pts_config: PtsConfig,
}

impl Default for DdaConfig {
    fn default() -> Self {
        Self {
            max_steps: 100_000,
            max_context_depth: 10,
            timeout_ms: 5_000,
            enable_strong_updates: true,
            pts_config: PtsConfig::default(),
        }
    }
}

impl DdaConfig {
    /// Create a new config with custom settings.
    #[must_use]
    pub fn new(
        max_steps: usize,
        max_context_depth: usize,
        timeout_ms: u64,
        enable_strong_updates: bool,
    ) -> Self {
        Self {
            max_steps,
            max_context_depth,
            timeout_ms,
            enable_strong_updates,
            pts_config: PtsConfig::default(),
        }
    }

    /// Create an unlimited config (no budget constraints).
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_steps: 0,
            max_context_depth: 0,
            timeout_ms: 0,
            enable_strong_updates: true,
            pts_config: PtsConfig::default(),
        }
    }

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
}

// ---------------------------------------------------------------------------
// ExhaustionReason
// ---------------------------------------------------------------------------

/// Reason for budget exhaustion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExhaustionReason {
    /// Maximum step count exceeded.
    StepsExceeded,
    /// Timeout exceeded.
    TimeoutExceeded,
    /// Maximum context depth exceeded.
    ContextDepthExceeded,
}

// ---------------------------------------------------------------------------
// Budget
// ---------------------------------------------------------------------------

/// Budget tracker for demand-driven analysis.
///
/// Tracks steps, time, and context depth to prevent runaway analysis.
/// When budget is exhausted, the solver falls back to CI-PTA.
#[derive(Debug, Clone)]
pub struct Budget {
    /// Remaining steps (decremented each tick).
    steps_remaining: usize,
    /// Whether steps are unlimited.
    unlimited_steps: bool,
    /// Query start time.
    start_time: Timer,
    /// Timeout in milliseconds.
    timeout_ms: u64,
    /// Whether timeout is unlimited.
    unlimited_timeout: bool,
    /// Maximum context depth.
    max_context_depth: usize,
    /// Whether context depth is unlimited.
    unlimited_depth: bool,
    /// Reason for exhaustion (if any).
    exhausted_reason: Option<ExhaustionReason>,
}

impl Budget {
    /// Create a new budget from configuration.
    #[must_use]
    pub fn new(config: &DdaConfig) -> Self {
        Self {
            steps_remaining: config.max_steps,
            unlimited_steps: config.max_steps == 0,
            start_time: Timer::now(),
            timeout_ms: config.timeout_ms,
            unlimited_timeout: config.timeout_ms == 0,
            max_context_depth: config.max_context_depth,
            unlimited_depth: config.max_context_depth == 0,
            exhausted_reason: None,
        }
    }

    /// Reset the budget for a new query.
    pub fn reset(&mut self, config: &DdaConfig) {
        self.steps_remaining = config.max_steps;
        self.unlimited_steps = config.max_steps == 0;
        self.start_time = Timer::now();
        self.timeout_ms = config.timeout_ms;
        self.unlimited_timeout = config.timeout_ms == 0;
        self.max_context_depth = config.max_context_depth;
        self.unlimited_depth = config.max_context_depth == 0;
        self.exhausted_reason = None;
    }

    /// Tick one step. Returns true if budget is still available.
    pub fn tick(&mut self) -> bool {
        if self.exhausted_reason.is_some() {
            return false;
        }

        // Check steps
        if !self.unlimited_steps {
            if self.steps_remaining == 0 {
                self.exhausted_reason = Some(ExhaustionReason::StepsExceeded);
                return false;
            }
            self.steps_remaining -= 1;
        }

        // Check timeout
        if !self.unlimited_timeout {
            let elapsed = self.start_time.elapsed_ms();
            if elapsed > self.timeout_ms {
                self.exhausted_reason = Some(ExhaustionReason::TimeoutExceeded);
                return false;
            }
        }

        true
    }

    /// Check if a context depth would exceed the budget.
    #[must_use]
    pub fn check_context_depth(&mut self, depth: usize) -> bool {
        if self.exhausted_reason.is_some() {
            return false;
        }

        if !self.unlimited_depth && depth > self.max_context_depth {
            self.exhausted_reason = Some(ExhaustionReason::ContextDepthExceeded);
            return false;
        }

        true
    }

    /// Check if the budget is exhausted.
    #[must_use]
    pub fn exhausted(&self) -> bool {
        self.exhausted_reason.is_some()
    }

    /// Get the exhaustion reason (if any).
    #[must_use]
    pub fn exhausted_reason(&self) -> Option<ExhaustionReason> {
        self.exhausted_reason
    }

    /// Get remaining steps.
    #[must_use]
    pub fn steps_remaining(&self) -> Option<usize> {
        if self.unlimited_steps {
            None
        } else {
            Some(self.steps_remaining)
        }
    }

    /// Get elapsed time in milliseconds.
    #[must_use]
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed_ms()
    }
}

// ---------------------------------------------------------------------------
// DdaCache
// ---------------------------------------------------------------------------

/// Two-level cache for demand-driven analysis.
///
/// - **TL (top-level)**: Cache for top-level pointer queries.
/// - **AT (address-taken)**: Cache for address-taken object queries.
///
/// The cache is persistent across queries to amortize analysis cost.
#[derive(Debug, Clone)]
pub struct DdaCache {
    /// Top-level pointer cache: DPM → points-to set.
    tl_cache: std::collections::BTreeMap<Dpm, IdBitSet<saf_core::ids::LocId>>,
    /// Shared indexer for all cached sets.
    loc_indexer: Arc<RwLock<Indexer<saf_core::ids::LocId>>>,
    /// Per-query visited set (reset each query).
    visited: BTreeSet<Dpm>,
}

impl Default for DdaCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DdaCache {
    /// Create a new empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tl_cache: std::collections::BTreeMap::new(),
            loc_indexer: Arc::new(RwLock::new(Indexer::new())),
            visited: BTreeSet::new(),
        }
    }

    /// Clear the per-query visited set (call before each query).
    pub fn clear_visited(&mut self) {
        self.visited.clear();
    }

    /// Mark a DPM as visited. Returns `true` if already visited.
    pub fn mark_visited(&mut self, dpm: &Dpm) -> bool {
        !self.visited.insert(dpm.clone())
    }

    /// Check if a DPM has been visited.
    #[must_use]
    pub fn is_visited(&self, dpm: &Dpm) -> bool {
        self.visited.contains(dpm)
    }

    /// Get a cached result for a DPM.
    #[must_use]
    pub fn get(&self, dpm: &Dpm) -> Option<&IdBitSet<saf_core::ids::LocId>> {
        self.tl_cache.get(dpm)
    }

    /// Insert a result into the TL cache.
    pub fn insert(&mut self, dpm: Dpm, pts: &BTreeSet<saf_core::ids::LocId>) {
        let bitset = IdBitSet::from_btreeset_with_indexer(pts, Arc::clone(&self.loc_indexer));
        self.tl_cache.insert(dpm, bitset);
    }

    /// Get cache statistics.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            tl_entries: self.tl_cache.len(),
        }
    }

    /// Clear all cached results (keep for testing or memory pressure).
    pub fn clear(&mut self) {
        self.tl_cache.clear();
        self.visited.clear();
        // Keep the indexer — it's reused across queries
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of entries in the TL cache.
    pub tl_entries: usize,
}

// ---------------------------------------------------------------------------
// DdaDiagnostics
// ---------------------------------------------------------------------------

/// Diagnostics from demand-driven analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DdaDiagnostics {
    /// Total number of queries processed.
    pub queries: usize,
    /// Number of cache hits (query answered from cache).
    pub cache_hits: usize,
    /// Number of CI-PTA fallbacks (budget exhausted).
    pub fallbacks: usize,
    /// Number of strong updates applied.
    pub strong_updates: usize,
    /// Total traversal steps across all queries.
    pub total_steps: usize,
    /// Number of CFL context terminations (unmatched returns with empty context).
    ///
    /// Incremented when backward traversal encounters a return edge but the
    /// context stack is empty, meaning there is no matching call site. Per
    /// CFL-reachability semantics the traversal stops along this path rather
    /// than propagating to all callers (which would introduce spurious paths).
    pub context_terminations: usize,
}

impl DdaDiagnostics {
    /// Record a query.
    pub fn record_query(&mut self) {
        self.queries += 1;
    }

    /// Record a cache hit.
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record a CI-PTA fallback.
    pub fn record_fallback(&mut self) {
        self.fallbacks += 1;
    }

    /// Record a strong update.
    pub fn record_strong_update(&mut self) {
        self.strong_updates += 1;
    }

    /// Record a CFL context termination (unmatched return with empty context).
    pub fn record_context_termination(&mut self) {
        self.context_terminations += 1;
    }

    /// Add steps to the total.
    pub fn add_steps(&mut self, steps: usize) {
        self.total_steps += steps;
    }
}

// ---------------------------------------------------------------------------
// ReachabilityResult
// ---------------------------------------------------------------------------

/// Result of a refined reachability query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachabilityResult {
    /// Whether the source can reach the sink.
    pub reachable: bool,
    /// Whether reachability was determined via alias analysis.
    pub via_alias: bool,
    /// Whether reachability was determined via SVFG path.
    pub via_svfg: bool,
    /// Number of locations in source's points-to set.
    pub src_pts_count: usize,
    /// Number of locations in sink's points-to set.
    pub sink_pts_count: usize,
}

// ---------------------------------------------------------------------------
// DdaExport
// ---------------------------------------------------------------------------

/// JSON-serializable export of DDA analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdaExport {
    /// Schema version for compatibility.
    pub schema_version: String,
    /// Configuration used for analysis.
    pub config: DdaConfigExport,
    /// Analysis diagnostics.
    pub diagnostics: DdaDiagnostics,
    /// Cache statistics.
    pub cache_stats: CacheStats,
}

/// Serializable DDA configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdaConfigExport {
    /// Maximum traversal steps.
    pub max_steps: usize,
    /// Maximum context depth.
    pub max_context_depth: usize,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether strong updates are enabled.
    pub enable_strong_updates: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::{InstId, ValueId};

    // ---------------------------------------------------------------------------
    // Dpm tests
    // ---------------------------------------------------------------------------

    #[test]
    fn dpm_creation() {
        let node = SvfgNodeId::Value(ValueId::new(42));
        let dpm = Dpm::new(node);

        assert_eq!(dpm.node, node);
        assert!(dpm.context.is_empty());
    }

    #[test]
    fn dpm_with_context() {
        let node = SvfgNodeId::Value(ValueId::new(42));
        let ctx = CallString::empty().push(InstId::new(1));
        let dpm = Dpm::with_context(node, ctx.clone());

        assert_eq!(dpm.node, node);
        assert_eq!(dpm.context.depth(), 1);
    }

    #[test]
    fn dpm_ordering() {
        let dpm1 = Dpm::new(SvfgNodeId::Value(ValueId::new(1)));
        let dpm2 = Dpm::new(SvfgNodeId::Value(ValueId::new(2)));

        // Different nodes should have different ordering
        assert!(dpm1 != dpm2);
    }

    // ---------------------------------------------------------------------------
    // DdaConfig tests
    // ---------------------------------------------------------------------------

    #[test]
    fn config_default() {
        let cfg = DdaConfig::default();
        assert_eq!(cfg.max_steps, 100_000);
        assert_eq!(cfg.max_context_depth, 10);
        assert_eq!(cfg.timeout_ms, 5_000);
        assert!(cfg.enable_strong_updates);
    }

    #[test]
    fn config_unlimited() {
        let cfg = DdaConfig::unlimited();
        assert_eq!(cfg.max_steps, 0);
        assert_eq!(cfg.max_context_depth, 0);
        assert_eq!(cfg.timeout_ms, 0);
        assert!(cfg.enable_strong_updates);
    }

    #[test]
    fn config_custom() {
        let cfg = DdaConfig::new(1000, 5, 500, false);
        assert_eq!(cfg.max_steps, 1000);
        assert_eq!(cfg.max_context_depth, 5);
        assert_eq!(cfg.timeout_ms, 500);
        assert!(!cfg.enable_strong_updates);
    }

    // ---------------------------------------------------------------------------
    // Budget tests
    // ---------------------------------------------------------------------------

    #[test]
    fn budget_steps() {
        let cfg = DdaConfig::new(3, 0, 0, true);
        let mut budget = Budget::new(&cfg);

        assert!(!budget.exhausted());
        assert!(budget.tick()); // 3 -> 2
        assert!(budget.tick()); // 2 -> 1
        assert!(budget.tick()); // 1 -> 0
        assert!(!budget.tick()); // exhausted
        assert!(budget.exhausted());
        assert_eq!(
            budget.exhausted_reason(),
            Some(ExhaustionReason::StepsExceeded)
        );
    }

    #[test]
    fn budget_unlimited_steps() {
        let cfg = DdaConfig::unlimited();
        let mut budget = Budget::new(&cfg);

        for _ in 0..1000 {
            assert!(budget.tick());
        }
        assert!(!budget.exhausted());
    }

    #[test]
    fn budget_context_depth() {
        let cfg = DdaConfig::new(0, 2, 0, true);
        let mut budget = Budget::new(&cfg);

        assert!(budget.check_context_depth(0));
        assert!(budget.check_context_depth(1));
        assert!(budget.check_context_depth(2));
        assert!(!budget.check_context_depth(3));
        assert!(budget.exhausted());
        assert_eq!(
            budget.exhausted_reason(),
            Some(ExhaustionReason::ContextDepthExceeded)
        );
    }

    #[test]
    fn budget_reset() {
        let cfg = DdaConfig::new(1, 0, 0, true);
        let mut budget = Budget::new(&cfg);

        assert!(budget.tick()); // 1 -> 0
        assert!(!budget.tick()); // exhausted
        assert!(budget.exhausted());

        budget.reset(&cfg);
        assert!(!budget.exhausted());
        assert!(budget.tick()); // 1 -> 0 again
    }

    // ---------------------------------------------------------------------------
    // DdaCache tests
    // ---------------------------------------------------------------------------

    #[test]
    fn cache_new() {
        let cache = DdaCache::new();
        assert_eq!(cache.stats().tl_entries, 0);
    }

    #[test]
    fn cache_visited() {
        let mut cache = DdaCache::new();
        let dpm = Dpm::new(SvfgNodeId::Value(ValueId::new(1)));

        assert!(!cache.is_visited(&dpm));
        assert!(!cache.mark_visited(&dpm)); // First time: returns false
        assert!(cache.is_visited(&dpm));
        assert!(cache.mark_visited(&dpm)); // Second time: returns true

        cache.clear_visited();
        assert!(!cache.is_visited(&dpm));
    }

    #[test]
    fn cache_insert_get() {
        let mut cache = DdaCache::new();
        let dpm = Dpm::new(SvfgNodeId::Value(ValueId::new(1)));
        let mut pts = BTreeSet::new();
        pts.insert(saf_core::ids::LocId::new(100));

        assert!(cache.get(&dpm).is_none());
        cache.insert(dpm.clone(), &pts);
        assert_eq!(cache.get(&dpm).map(|s| s.to_btreeset()), Some(pts));
        assert_eq!(cache.stats().tl_entries, 1);
    }

    #[test]
    fn cache_clear() {
        let mut cache = DdaCache::new();
        let dpm = Dpm::new(SvfgNodeId::Value(ValueId::new(1)));
        cache.insert(dpm.clone(), &BTreeSet::new());
        cache.mark_visited(&dpm);

        cache.clear();
        assert_eq!(cache.stats().tl_entries, 0);
        assert!(!cache.is_visited(&dpm));
    }

    // ---------------------------------------------------------------------------
    // DdaDiagnostics tests
    // ---------------------------------------------------------------------------

    #[test]
    fn diagnostics_default() {
        let diag = DdaDiagnostics::default();
        assert_eq!(diag.queries, 0);
        assert_eq!(diag.cache_hits, 0);
        assert_eq!(diag.fallbacks, 0);
        assert_eq!(diag.strong_updates, 0);
        assert_eq!(diag.total_steps, 0);
        assert_eq!(diag.context_terminations, 0);
    }

    #[test]
    fn diagnostics_recording() {
        let mut diag = DdaDiagnostics::default();

        diag.record_query();
        diag.record_query();
        assert_eq!(diag.queries, 2);

        diag.record_cache_hit();
        assert_eq!(diag.cache_hits, 1);

        diag.record_fallback();
        assert_eq!(diag.fallbacks, 1);

        diag.record_strong_update();
        diag.record_strong_update();
        assert_eq!(diag.strong_updates, 2);

        diag.add_steps(100);
        diag.add_steps(50);
        assert_eq!(diag.total_steps, 150);

        diag.record_context_termination();
        diag.record_context_termination();
        diag.record_context_termination();
        assert_eq!(diag.context_terminations, 3);
    }
}
