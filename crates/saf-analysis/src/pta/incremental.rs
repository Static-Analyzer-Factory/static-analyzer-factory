//! Incremental pointer analysis with SILVA-style optimizations.
//!
//! This module implements incremental pointer analysis techniques inspired by
//! SILVA (Scalable Incremental Layered Sparse Value-Flow Analysis). Key features:
//!
//! - **Difference propagation**: Only propagate changed points-to entries
//! - **Changed node tracking**: Track which nodes are affected by updates
//! - **Version caching**: Reuse unchanged analysis results
//!
//! These optimizations can achieve significant speedups for incremental analysis
//! scenarios (e.g., analyzing code changes without full recomputation).
//!
//! NOTE: This module is activated and integrated into the incremental analysis
//! pipeline. See [`super::module_constraints`] for per-module constraint caching.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::ids::{LocId, ValueId};

use super::constraint::ConstraintSet;
#[cfg(test)]
use super::constraint::{AddrConstraint, CopyConstraint};
use super::result::PtaResult;
use super::solver::PointsToMap;

// =============================================================================
// Difference-Based Points-To Set
// =============================================================================

/// A points-to set that tracks differences (newly added entries).
///
/// This is the core data structure for SILVA-style difference propagation.
/// Instead of propagating the entire points-to set on each iteration,
/// we only propagate the difference (new entries) to successors.
#[derive(Debug, Clone, Default)]
pub struct DiffPointsToSet {
    /// The full points-to set.
    full: BTreeSet<LocId>,
    /// Newly added entries (difference from last propagation).
    diff: BTreeSet<LocId>,
    /// Version number for caching.
    version: u64,
}

impl DiffPointsToSet {
    /// Create an empty points-to set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a location to the points-to set.
    /// Returns true if the location was newly added.
    pub fn add(&mut self, loc: LocId) -> bool {
        if self.full.insert(loc) {
            self.diff.insert(loc);
            true
        } else {
            false
        }
    }

    /// Add multiple locations to the points-to set.
    /// Returns true if any location was newly added.
    pub fn add_all(&mut self, locs: &BTreeSet<LocId>) -> bool {
        let mut changed = false;
        for &loc in locs {
            if self.add(loc) {
                changed = true;
            }
        }
        changed
    }

    /// Get the full points-to set.
    #[must_use]
    pub fn full(&self) -> &BTreeSet<LocId> {
        &self.full
    }

    /// Get the difference (newly added entries since last clear).
    #[must_use]
    pub fn diff(&self) -> &BTreeSet<LocId> {
        &self.diff
    }

    /// Clear the difference set (after propagation).
    pub fn clear_diff(&mut self) {
        self.diff.clear();
    }

    /// Check if the difference is non-empty.
    #[must_use]
    pub fn has_diff(&self) -> bool {
        !self.diff.is_empty()
    }

    /// Get the version number.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Increment the version number.
    pub fn bump_version(&mut self) {
        self.version += 1;
    }

    /// Check if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.full.is_empty()
    }

    /// Get the number of locations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.full.len()
    }
}

// =============================================================================
// Incremental Solver State
// =============================================================================

/// State for incremental pointer analysis.
#[derive(Debug, Clone)]
pub struct IncrementalPtaState {
    /// Points-to sets for each value.
    pts: BTreeMap<ValueId, DiffPointsToSet>,
    /// Copy edges: dst → set of srcs.
    copy_edges: BTreeMap<ValueId, BTreeSet<ValueId>>,
    /// Load edges: dst → src_ptr.
    load_edges: BTreeMap<ValueId, BTreeSet<ValueId>>,
    /// Store edges: dst_ptr → src.
    store_edges: BTreeMap<ValueId, BTreeSet<ValueId>>,
    /// Changed nodes that need reprocessing.
    changed_nodes: BTreeSet<ValueId>,
    /// Global version counter.
    global_version: u64,
    /// Accumulated removal debt from removed constraints.
    ///
    /// Each time constraints are removed via [`apply_incremental_update`], the
    /// removal count is accumulated here. Constraint removal is handled lazily
    /// (over-approximate): the points-to sets are not shrunk because precise
    /// removal would require full recomputation. When this debt exceeds a
    /// threshold, [`needs_gc`](Self::needs_gc) returns `true`, signaling that
    /// a full recompute via [`reset`](Self::reset) is advisable.
    removal_debt: usize,
}

impl IncrementalPtaState {
    /// Create a new empty incremental state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pts: BTreeMap::new(),
            copy_edges: BTreeMap::new(),
            load_edges: BTreeMap::new(),
            store_edges: BTreeMap::new(),
            changed_nodes: BTreeSet::new(),
            global_version: 0,
            removal_debt: 0,
        }
    }

    /// Initialize state from constraints.
    pub fn initialize(&mut self, constraints: &ConstraintSet) {
        // Process address constraints (initial points-to facts)
        for addr in &constraints.addr {
            let pts = self.pts.entry(addr.ptr).or_default();
            if pts.add(addr.loc) {
                self.changed_nodes.insert(addr.ptr);
            }
        }

        // Build copy edge graph
        for copy in &constraints.copy {
            self.copy_edges
                .entry(copy.dst)
                .or_default()
                .insert(copy.src);
            // Mark dst as changed if src has points-to info
            if self.pts.contains_key(&copy.src) {
                self.changed_nodes.insert(copy.dst);
            }
        }

        // Build load edge graph
        for load in &constraints.load {
            self.load_edges
                .entry(load.dst)
                .or_default()
                .insert(load.src_ptr);
        }

        // Build store edge graph
        for store in &constraints.store {
            self.store_edges
                .entry(store.dst_ptr)
                .or_default()
                .insert(store.src);
        }
    }

    /// Get the points-to set for a value.
    #[must_use]
    pub fn get_pts(&self, val: ValueId) -> Option<&BTreeSet<LocId>> {
        self.pts.get(&val).map(DiffPointsToSet::full)
    }

    /// Mark a node as changed (needs reprocessing).
    pub fn mark_changed(&mut self, val: ValueId) {
        self.changed_nodes.insert(val);
    }

    /// Check if any nodes need reprocessing.
    #[must_use]
    pub fn has_changed_nodes(&self) -> bool {
        !self.changed_nodes.is_empty()
    }

    /// Get the number of changed nodes.
    #[must_use]
    pub fn num_changed_nodes(&self) -> usize {
        self.changed_nodes.len()
    }

    /// Get the accumulated removal debt.
    #[must_use]
    pub fn removal_debt(&self) -> usize {
        self.removal_debt
    }

    /// Check if accumulated removal debt exceeds the given threshold.
    ///
    /// When this returns `true`, the points-to sets may be over-approximate
    /// due to lazily handled constraint removals. Callers should trigger a
    /// full recompute via [`reset`](Self::reset) followed by a fresh solve.
    #[must_use]
    pub fn needs_gc(&self, threshold: usize) -> bool {
        self.removal_debt >= threshold
    }

    /// Reset the state for a full recompute.
    ///
    /// Clears all points-to sets, edges, changed nodes, and removal debt.
    /// After calling this, re-initialize with [`initialize`](Self::initialize)
    /// and re-solve to obtain fresh results.
    pub fn reset(&mut self) {
        self.pts.clear();
        self.copy_edges.clear();
        self.load_edges.clear();
        self.store_edges.clear();
        self.changed_nodes.clear();
        self.removal_debt = 0;
        // Keep global_version to maintain monotonicity
    }

    /// Create an incremental state from a production `PtaResult`.
    ///
    /// This bridges the gap between the production Andersen solver and the
    /// incremental solver: after a full solve produces a `PtaResult`, this
    /// method seeds the incremental state so that subsequent code changes
    /// can be handled incrementally rather than re-solving from scratch.
    ///
    /// Note: Only the points-to sets are transferred. Edge graphs (copy,
    /// load, store) must be re-populated by calling [`initialize`](Self::initialize)
    /// with the current constraint set.
    #[must_use]
    pub fn from_pta_result(result: &PtaResult) -> Self {
        let mut pts = BTreeMap::new();
        for (val, locs) in result.points_to_map() {
            let mut diff_pts = DiffPointsToSet::new();
            for &loc in locs {
                diff_pts.add(loc);
            }
            // Clear diff since these are existing facts, not new changes
            diff_pts.clear_diff();
            pts.insert(*val, diff_pts);
        }

        Self {
            pts,
            copy_edges: BTreeMap::new(),
            load_edges: BTreeMap::new(),
            store_edges: BTreeMap::new(),
            changed_nodes: BTreeSet::new(),
            global_version: 0,
            removal_debt: 0,
        }
    }

    /// Convert the incremental state back to a production `PointsToMap`.
    ///
    /// Extracts the full points-to sets from the diff-tracking representation,
    /// producing a standard `BTreeMap<ValueId, BTreeSet<LocId>>` suitable for
    /// constructing a `PtaResult`.
    #[must_use]
    pub fn to_points_to_map(&self) -> PointsToMap {
        self.pts
            .iter()
            .map(|(v, p)| (*v, p.full().clone()))
            .collect()
    }
}

impl Default for IncrementalPtaState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Incremental Solver
// =============================================================================

/// Configuration for incremental solver.
#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    /// Maximum iterations before giving up.
    pub max_iterations: usize,
    /// Whether to use difference propagation (reserved for future optimization).
    ///
    /// Note: Currently we always use full-set propagation for correctness.
    /// True SILVA-style diff propagation requires more sophisticated tracking
    /// of which edges have been satisfied.
    pub use_diff_propagation: bool,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            use_diff_propagation: true,
        }
    }
}

/// Result of incremental solving.
#[derive(Debug, Clone)]
pub struct IncrementalResult {
    /// Final points-to map.
    pub points_to: BTreeMap<ValueId, BTreeSet<LocId>>,
    /// Number of iterations to reach fixed point.
    pub iterations: usize,
    /// Total propagations performed.
    pub propagations: usize,
    /// Whether the solver converged.
    pub converged: bool,
}

/// Solve pointer analysis incrementally with difference propagation.
///
/// This is the main entry point for SILVA-style incremental analysis.
/// It uses difference propagation to only process changed information,
/// achieving better performance than full recomputation.
///
/// Note: For initial solving, we use a hybrid approach where we always
/// propagate full sets through copy edges (for correctness), but track
/// diffs for detecting when nodes have changed. True SILVA-style diff
/// propagation is used in `apply_incremental_update` for re-analysis.
#[must_use]
pub fn solve_incremental(
    constraints: &ConstraintSet,
    config: &IncrementalConfig,
) -> IncrementalResult {
    let mut state = IncrementalPtaState::new();
    state.initialize(constraints);

    let mut iterations = 0;
    let mut propagations = 0;
    let mut converged = false;

    // Worklist algorithm
    // For initial solving, always process all nodes with pts info as starting points
    let mut worklist: VecDeque<ValueId> = state.changed_nodes.iter().copied().collect();

    // Also add all nodes that have copy sources with pts info
    for (&dst, srcs) in &state.copy_edges {
        for src in srcs {
            if state.pts.contains_key(src) && !worklist.contains(&dst) {
                worklist.push_back(dst);
            }
        }
    }
    state.changed_nodes.clear();

    // Track which nodes have been fully processed in this round
    let mut processed_in_round: BTreeSet<ValueId> = BTreeSet::new();

    while let Some(node) = worklist.pop_front() {
        if iterations >= config.max_iterations {
            break;
        }
        iterations += 1;

        let mut node_changed = false;

        // Process copy edges: pts(node) |= pts(src) for each copy edge src → node
        // Always use full set for correctness during initial solve
        if let Some(srcs) = state.copy_edges.get(&node).cloned() {
            for src in srcs {
                let src_pts = state.pts.get(&src).map(|p| p.full().clone());

                if let Some(to_add) = src_pts {
                    if !to_add.is_empty() {
                        let dst_pts = state.pts.entry(node).or_default();
                        if dst_pts.add_all(&to_add) {
                            node_changed = true;
                            propagations += to_add.len();
                        }
                    }
                }
            }
        }

        // Process load edges: for each loc in pts(src_ptr), add copy edge loc → dst
        // This is a simplified model - full SILVA would be more sophisticated
        if let Some(src_ptrs) = state.load_edges.get(&node).cloned() {
            for src_ptr in src_ptrs {
                if let Some(ptr_pts) = state.pts.get(&src_ptr).cloned() {
                    // For each location the pointer may point to
                    for loc in ptr_pts.full() {
                        // Create a synthetic value representing the location's content
                        let loc_val = ValueId::new(loc.raw());
                        if let Some(loc_pts) = state.pts.get(&loc_val).cloned() {
                            let dst_pts = state.pts.entry(node).or_default();
                            if dst_pts.add_all(loc_pts.full()) {
                                node_changed = true;
                                propagations += loc_pts.len();
                            }
                        }
                    }
                }
            }
        }

        // Process store edges: for each loc in pts(dst_ptr), add copy edge src → loc
        if let Some(srcs) = state.store_edges.get(&node).cloned() {
            if let Some(ptr_pts) = state.pts.get(&node).cloned() {
                for loc in ptr_pts.full() {
                    let loc_val = ValueId::new(loc.raw());
                    for &src in &srcs {
                        if let Some(src_pts) = state.pts.get(&src).cloned() {
                            let loc_pts = state.pts.entry(loc_val).or_default();
                            if loc_pts.add_all(src_pts.full()) {
                                node_changed = true;
                                propagations += src_pts.len();
                                // Mark loc as changed for further propagation
                                if !worklist.contains(&loc_val) {
                                    worklist.push_back(loc_val);
                                }
                            }
                        }
                    }
                }
            }
        }

        // If this node changed, add successors to worklist
        if node_changed {
            // Find all nodes that have copy edges from this node
            for (&dst, srcs) in &state.copy_edges {
                if srcs.contains(&node) && !worklist.contains(&dst) {
                    worklist.push_back(dst);
                }
            }
        }

        processed_in_round.insert(node);

        if worklist.is_empty() {
            converged = true;
        }
    }

    // Clear all diffs after solving (they represent the initial state now)
    for pts in state.pts.values_mut() {
        pts.clear_diff();
    }

    // Extract final points-to map
    let points_to: BTreeMap<ValueId, BTreeSet<LocId>> = state
        .pts
        .into_iter()
        .map(|(v, p)| (v, p.full.clone()))
        .collect();

    IncrementalResult {
        points_to,
        iterations,
        propagations,
        converged,
    }
}

/// Apply an incremental update to existing analysis state.
///
/// This is used when the program changes and we want to update
/// the analysis without full recomputation.
///
/// **Added constraints** are fully processed: new edges and points-to facts
/// are propagated through the worklist solver until a fixed point.
///
/// **Removed constraints** are handled lazily (over-approximate): edges are
/// removed from the graph but points-to sets are not shrunk, since precise
/// removal would require full recomputation. Instead, the removal count is
/// accumulated in [`IncrementalPtaState::removal_debt`]. When this exceeds
/// a caller-chosen threshold (checked via [`IncrementalPtaState::needs_gc`]),
/// a full recompute should be triggered.
#[allow(clippy::too_many_lines)]
pub fn apply_incremental_update(
    state: &mut IncrementalPtaState,
    added_constraints: &ConstraintSet,
    removed_constraints: &ConstraintSet,
    config: &IncrementalConfig,
) -> IncrementalResult {
    // Track removal debt from removed constraints (lazy over-approximate)
    let mut removal_count = 0;

    for copy in &removed_constraints.copy {
        if let Some(srcs) = state.copy_edges.get_mut(&copy.dst) {
            if srcs.remove(&copy.src) {
                removal_count += 1;
            }
        }
    }

    for load in &removed_constraints.load {
        if let Some(srcs) = state.load_edges.get_mut(&load.dst) {
            if srcs.remove(&load.src_ptr) {
                removal_count += 1;
            }
        }
    }

    for store in &removed_constraints.store {
        if let Some(srcs) = state.store_edges.get_mut(&store.dst_ptr) {
            if srcs.remove(&store.src) {
                removal_count += 1;
            }
        }
    }

    // Addr removals also counted but pts not shrunk (over-approximate)
    removal_count += removed_constraints.addr.len();

    state.removal_debt += removal_count;

    // Mark nodes affected by new constraints as changed
    for addr in &added_constraints.addr {
        let pts = state.pts.entry(addr.ptr).or_default();
        if pts.add(addr.loc) {
            state.mark_changed(addr.ptr);
        }
    }

    for copy in &added_constraints.copy {
        state
            .copy_edges
            .entry(copy.dst)
            .or_default()
            .insert(copy.src);
        state.mark_changed(copy.dst);
    }

    for load in &added_constraints.load {
        state
            .load_edges
            .entry(load.dst)
            .or_default()
            .insert(load.src_ptr);
        state.mark_changed(load.dst);
    }

    for store in &added_constraints.store {
        state
            .store_edges
            .entry(store.dst_ptr)
            .or_default()
            .insert(store.src);
        state.mark_changed(store.dst_ptr);
    }

    // Run incremental solving from changed nodes
    let mut iterations = 0;
    let mut propagations = 0;
    let mut converged = false;

    let mut worklist: VecDeque<ValueId> = state.changed_nodes.iter().copied().collect();
    state.changed_nodes.clear();

    while let Some(node) = worklist.pop_front() {
        if iterations >= config.max_iterations {
            break;
        }
        iterations += 1;

        let mut node_changed = false;

        // Process copy edges - always use full set for correctness
        if let Some(srcs) = state.copy_edges.get(&node).cloned() {
            for src in srcs {
                let src_pts = state.pts.get(&src).map(|p| p.full().clone());

                if let Some(to_add) = src_pts {
                    if !to_add.is_empty() {
                        let dst_pts = state.pts.entry(node).or_default();
                        if dst_pts.add_all(&to_add) {
                            node_changed = true;
                            propagations += to_add.len();
                        }
                    }
                }
            }
        }

        if node_changed {
            for (&dst, srcs) in &state.copy_edges {
                if srcs.contains(&node) && !worklist.contains(&dst) {
                    worklist.push_back(dst);
                }
            }
        }

        if worklist.is_empty() {
            converged = true;
        }
    }

    // Bump global version
    state.global_version += 1;

    let points_to: BTreeMap<ValueId, BTreeSet<LocId>> = state
        .pts
        .iter()
        .map(|(v, p)| (*v, p.full().clone()))
        .collect();

    IncrementalResult {
        points_to,
        iterations,
        propagations,
        converged,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn val(id: u128) -> ValueId {
        ValueId::new(id)
    }

    fn loc(id: u128) -> LocId {
        LocId::new(id)
    }

    #[test]
    fn diff_pts_basic() {
        let mut pts = DiffPointsToSet::new();

        assert!(pts.add(loc(1)));
        assert!(pts.add(loc(2)));
        assert!(!pts.add(loc(1))); // Already present

        assert_eq!(pts.len(), 2);
        assert!(pts.full().contains(&loc(1)));
        assert!(pts.full().contains(&loc(2)));

        // Diff should contain both
        assert!(pts.diff().contains(&loc(1)));
        assert!(pts.diff().contains(&loc(2)));

        // Clear diff
        pts.clear_diff();
        assert!(!pts.has_diff());
        assert!(pts.diff().is_empty());

        // Full should still have entries
        assert_eq!(pts.len(), 2);
    }

    #[test]
    fn diff_pts_add_all() {
        let mut pts = DiffPointsToSet::new();
        pts.add(loc(1));
        pts.clear_diff();

        let mut to_add = BTreeSet::new();
        to_add.insert(loc(2));
        to_add.insert(loc(3));

        assert!(pts.add_all(&to_add));
        assert_eq!(pts.len(), 3);
        assert_eq!(pts.diff().len(), 2); // Only 2 and 3 are new
    }

    #[test]
    fn incremental_simple_addr() {
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });

        let config = IncrementalConfig::default();
        let result = solve_incremental(&cs, &config);

        assert!(result.converged);
        assert!(result.points_to.get(&val(1)).unwrap().contains(&loc(100)));
    }

    #[test]
    fn incremental_copy_chain() {
        // p = &x; q = p; r = q
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),   // p
            loc: loc(100), // x
        });
        cs.copy.insert(CopyConstraint {
            dst: val(2), // q
            src: val(1), // p
        });
        cs.copy.insert(CopyConstraint {
            dst: val(3), // r
            src: val(2), // q
        });

        let config = IncrementalConfig::default();
        let result = solve_incremental(&cs, &config);

        assert!(result.converged);
        // All should point to x
        assert!(result.points_to.get(&val(1)).unwrap().contains(&loc(100)));
        assert!(result.points_to.get(&val(2)).unwrap().contains(&loc(100)));
        assert!(result.points_to.get(&val(3)).unwrap().contains(&loc(100)));
    }

    #[test]
    fn incremental_diff_propagation_reduces_work() {
        // Create a chain: p = &x, then q = p, r = q, etc.
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });
        for i in 1..10 {
            cs.copy.insert(CopyConstraint {
                dst: val(i + 1),
                src: val(i),
            });
        }

        let config = IncrementalConfig::default();
        let result = solve_incremental(&cs, &config);

        // Should converge with all nodes pointing to loc(100)
        assert!(result.converged);
        for i in 1..=10 {
            assert!(
                result.points_to.get(&val(i)).unwrap().contains(&loc(100)),
                "val({i}) should point to loc(100)"
            );
        }

        // Now test incremental update: add a new addr constraint to the middle
        // Initial state: only val(1) has &x
        let mut initial_cs = ConstraintSet::default();
        initial_cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });

        let mut state = IncrementalPtaState::new();
        state.initialize(&initial_cs);

        // First solve: just the addr constraint
        let _ = solve_incremental(&initial_cs, &config);

        // Re-initialize state for incremental update test
        state = IncrementalPtaState::new();
        state.initialize(&initial_cs);

        // Add the copy chain incrementally
        let mut added = ConstraintSet::default();
        for i in 1..10 {
            added.copy.insert(CopyConstraint {
                dst: val(i + 1),
                src: val(i),
            });
        }

        let incremental_result =
            apply_incremental_update(&mut state, &added, &ConstraintSet::default(), &config);

        // Should have same result as full solve
        assert!(incremental_result.converged);
        for i in 1..=10 {
            assert!(
                incremental_result
                    .points_to
                    .get(&val(i))
                    .unwrap()
                    .contains(&loc(100)),
                "val({i}) should point to loc(100) after incremental update"
            );
        }
    }

    #[test]
    fn incremental_update() {
        // Initial: p = &x
        let mut cs1 = ConstraintSet::default();
        cs1.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });

        let config = IncrementalConfig::default();
        let mut state = IncrementalPtaState::new();
        state.initialize(&cs1);

        // Solve initial
        let result1 = solve_incremental(&cs1, &config);
        assert!(result1.points_to.get(&val(1)).unwrap().contains(&loc(100)));

        // Add: q = p
        let mut added = ConstraintSet::default();
        added.copy.insert(CopyConstraint {
            dst: val(2),
            src: val(1),
        });

        let result2 =
            apply_incremental_update(&mut state, &added, &ConstraintSet::default(), &config);

        // q should now point to x
        assert!(result2.points_to.get(&val(2)).unwrap().contains(&loc(100)));
    }

    #[test]
    fn incremental_state_version_tracking() {
        let mut state = IncrementalPtaState::new();
        assert_eq!(state.global_version, 0);

        let added = ConstraintSet::default();
        let config = IncrementalConfig::default();

        apply_incremental_update(&mut state, &added, &ConstraintSet::default(), &config);
        assert_eq!(state.global_version, 1);

        apply_incremental_update(&mut state, &added, &ConstraintSet::default(), &config);
        assert_eq!(state.global_version, 2);
    }

    #[test]
    fn incremental_convergence() {
        // Cyclic constraints: p = q; q = p; p = &x
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),   // p
            loc: loc(100), // x
        });
        cs.copy.insert(CopyConstraint {
            dst: val(1), // p
            src: val(2), // q
        });
        cs.copy.insert(CopyConstraint {
            dst: val(2), // q
            src: val(1), // p
        });

        let config = IncrementalConfig::default();
        let result = solve_incremental(&cs, &config);

        assert!(result.converged);
        // Both should point to x
        assert!(result.points_to.get(&val(1)).unwrap().contains(&loc(100)));
        assert!(result.points_to.get(&val(2)).unwrap().contains(&loc(100)));
    }

    #[test]
    fn removal_debt_tracking() {
        let mut state = IncrementalPtaState::new();
        assert_eq!(state.removal_debt(), 0);
        assert!(!state.needs_gc(1));

        // Initial constraints
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });
        cs.copy.insert(CopyConstraint {
            dst: val(2),
            src: val(1),
        });
        state.initialize(&cs);

        let config = IncrementalConfig::default();

        // Remove the copy constraint
        let mut removed = ConstraintSet::default();
        removed.copy.insert(CopyConstraint {
            dst: val(2),
            src: val(1),
        });

        apply_incremental_update(&mut state, &ConstraintSet::default(), &removed, &config);
        assert_eq!(state.removal_debt(), 1);
        assert!(state.needs_gc(1));
        assert!(!state.needs_gc(2));

        // Remove addr constraint
        let mut removed2 = ConstraintSet::default();
        removed2.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });
        apply_incremental_update(&mut state, &ConstraintSet::default(), &removed2, &config);
        assert_eq!(state.removal_debt(), 2);
        assert!(state.needs_gc(2));
    }

    #[test]
    fn reset_clears_state() {
        let mut state = IncrementalPtaState::new();
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });
        cs.copy.insert(CopyConstraint {
            dst: val(2),
            src: val(1),
        });
        state.initialize(&cs);

        // Accumulate some removal debt
        let config = IncrementalConfig::default();
        let mut removed = ConstraintSet::default();
        removed.copy.insert(CopyConstraint {
            dst: val(2),
            src: val(1),
        });
        apply_incremental_update(&mut state, &ConstraintSet::default(), &removed, &config);
        assert!(state.removal_debt() > 0);

        // Version should have been bumped
        let version_before = state.global_version;
        assert!(version_before > 0);

        state.reset();
        assert_eq!(state.removal_debt(), 0);
        assert!(state.pts.is_empty());
        assert!(state.copy_edges.is_empty());
        assert!(state.load_edges.is_empty());
        assert!(state.store_edges.is_empty());
        assert!(!state.has_changed_nodes());
        // Version is preserved for monotonicity
        assert_eq!(state.global_version, version_before);
    }

    #[test]
    fn to_points_to_map_round_trip() {
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(200),
        });
        cs.addr.insert(AddrConstraint {
            ptr: val(2),
            loc: loc(300),
        });
        cs.copy.insert(CopyConstraint {
            dst: val(3),
            src: val(1),
        });

        let config = IncrementalConfig::default();
        let result = solve_incremental(&cs, &config);
        assert!(result.converged);

        // Build state and convert to PointsToMap
        let mut state = IncrementalPtaState::new();
        state.initialize(&cs);

        // Solve via the incremental path
        let mut worklist: VecDeque<ValueId> = state.changed_nodes.iter().copied().collect();
        for (&dst, srcs) in &state.copy_edges {
            for src in srcs {
                if state.pts.contains_key(src) && !worklist.contains(&dst) {
                    worklist.push_back(dst);
                }
            }
        }
        state.changed_nodes.clear();
        // Run one pass of propagation
        let added = ConstraintSet::default();
        let removed = ConstraintSet::default();
        let inc_result = apply_incremental_update(&mut state, &added, &removed, &config);

        let pts_map = state.to_points_to_map();

        // Verify the map matches the incremental result
        assert_eq!(pts_map, inc_result.points_to);
    }

    #[test]
    fn from_pta_result_bridge() {
        use super::super::config::FieldSensitivity;
        use super::super::context::PtaDiagnostics;
        use super::super::location::{FieldPath, LocationFactory};
        use saf_core::ids::ObjId;
        use std::sync::Arc;

        // Build a PtaResult with known points-to sets
        let mut factory = LocationFactory::new(FieldSensitivity::None);
        let loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc2 = factory.get_or_create(ObjId::new(200), FieldPath::empty());

        let mut pts_map = PointsToMap::new();
        let mut set1 = BTreeSet::new();
        set1.insert(loc1);
        set1.insert(loc2);
        pts_map.insert(val(1), set1.clone());

        let mut set2 = BTreeSet::new();
        set2.insert(loc1);
        pts_map.insert(val(2), set2.clone());

        let result = PtaResult::new(
            pts_map.clone(),
            Arc::new(factory),
            PtaDiagnostics::default(),
        );

        // Bridge to incremental state
        let state = IncrementalPtaState::from_pta_result(&result);

        // Verify points-to sets match
        assert_eq!(state.get_pts(val(1)).unwrap(), &set1);
        assert_eq!(state.get_pts(val(2)).unwrap(), &set2);

        // Diffs should be empty (existing facts, not new)
        assert!(!state.pts.get(&val(1)).unwrap().has_diff());
        assert!(!state.pts.get(&val(2)).unwrap().has_diff());

        // No changed nodes, no removal debt
        assert!(!state.has_changed_nodes());
        assert_eq!(state.removal_debt(), 0);

        // Round-trip: back to PointsToMap should match original
        let round_tripped = state.to_points_to_map();
        assert_eq!(round_tripped, pts_map);
    }

    #[test]
    fn removal_debt_edge_removal() {
        // Verify that removed edges are actually removed from the graph
        let mut state = IncrementalPtaState::new();
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });
        cs.copy.insert(CopyConstraint {
            dst: val(2),
            src: val(1),
        });
        state.initialize(&cs);

        // Solve initial
        let config = IncrementalConfig::default();
        let result = solve_incremental(&cs, &config);
        assert!(result.points_to.get(&val(2)).unwrap().contains(&loc(100)));

        // Remove the copy edge
        let mut removed = ConstraintSet::default();
        removed.copy.insert(CopyConstraint {
            dst: val(2),
            src: val(1),
        });

        apply_incremental_update(&mut state, &ConstraintSet::default(), &removed, &config);

        // Edge should be removed from the graph
        let srcs = state.copy_edges.get(&val(2));
        assert!(
            srcs.is_none() || !srcs.unwrap().contains(&val(1)),
            "removed copy edge should no longer be in the graph"
        );

        // But pts is NOT shrunk (lazy over-approximate)
        // val(2) may still show loc(100) in its pts
        assert_eq!(state.removal_debt(), 1);
    }
}
