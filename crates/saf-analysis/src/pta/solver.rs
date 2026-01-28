//! Points-to analysis worklist solver.
//!
//! Implements Andersen-style inclusion-based analysis using a deterministic
//! worklist algorithm. Supports multiple points-to set representations through
//! the `PtsSet` trait for scalability:
//!
//! - `BTreePtsSet`: Simple baseline, good for small programs
//! - `RoaringPtsSet`: Fast operations for medium programs
//! - `BddPtsSet`: Compact representation for large programs
//!
//! Uses a priority worklist (`BTreeSet<(u32, ValueId)>`) ordered by topological rank
//! of the copy-constraint graph, so information flows "downhill" and each node is
//! processed after its inputs stabilize. Location worklists use `IdBitSet` for
//! deterministic iteration (NFR-DET).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashSet};
use smallvec::SmallVec;

use saf_core::air::AirModule;
use saf_core::ids::{LocId, ValueId};

use super::config::IndexSensitivity;
use super::constraint::{AddrConstraint, ConstraintSet, CopyConstraint};
use super::constraint_index::{ConstraintIndex, IndexedConstraints};
use super::hvn::HvnResult;
use super::location::{
    ConstantsTable, LocationFactory, merge_gep_with_base_path, resolve_gep_path,
};
use super::ptsset::{
    BTreePtsSet, BddPtsSet, ClusteringMode, FxHashPtsSet, IdBitSet, LocIdIndexer, PtsConfig,
    PtsRepresentation, PtsSet, RoaringPtsSet,
};
use saf_core::saf_log;

use super::solver_stats::SolverStats;
use std::cell::RefCell;

/// Points-to set type: maps values to the locations they may point to.
pub type PointsToMap = BTreeMap<ValueId, BTreeSet<LocId>>;

/// `IndexMap` with `FxHash` for fast u128 key lookups.
/// `FxHash` is ~6x faster than `SipHash` for u128 keys (2ns vs 13ns per hash).
/// Used only for internal solver maps; public API types remain `BTreeMap` for determinism.
type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

/// Generic points-to map using any `PtsSet` implementation.
pub type GenericPointsToMap<P> = FxIndexMap<ValueId, P>;

/// Solve constraints to compute points-to sets using the default representation.
///
/// Implements a worklist-based fixed-point algorithm:
/// 1. Initialize points-to sets from Addr constraints
/// 2. Iterate until fixed point, processing Copy/Load/Store/Gep constraints
///
/// Returns a map from values to their points-to sets.
pub fn solve(
    constraints: &ConstraintSet,
    factory: &LocationFactory,
    max_iterations: usize,
) -> PointsToMap {
    // Use BTreePtsSet internally, then normalize to BTreeSet<LocId>
    let generic_result = solve_generic::<BTreePtsSet>(constraints, factory, max_iterations);
    normalize_result(generic_result)
}

/// Solve constraints using a specific points-to set representation.
///
/// This is the generic version that can use any `PtsSet` implementation.
/// For API stability, consider using `solve()` which normalizes to `BTreeSet<LocId>`.
pub fn solve_generic<P: PtsSet>(
    constraints: &ConstraintSet,
    factory: &LocationFactory,
    max_iterations: usize,
) -> GenericPointsToMap<P> {
    let mut solver = GenericSolver::<P>::new(constraints, factory);
    solver.solve(max_iterations);
    solver.pts
}

/// Normalize a generic points-to map to the standard representation.
fn normalize_result<P: PtsSet>(generic: GenericPointsToMap<P>) -> PointsToMap {
    generic
        .into_iter()
        .map(|(v, pts)| (v, pts.to_btreeset()))
        .collect()
}

/// Solve constraints with runtime representation selection.
///
/// Selects the points-to set representation based on `PtsConfig`:
/// - `Auto`: Select based on allocation site count (BTreeSet < 10K < BitVec < 100K < BDD)
/// - Explicit: Use the specified representation
///
/// Returns a normalized `PointsToMap` and a boolean indicating whether the
/// iteration limit was reached before the solver converged. When `true`,
/// the results may be imprecise (under-approximate).
#[allow(dead_code)] // Public API for external use
pub fn solve_with_config(
    constraints: &ConstraintSet,
    factory: &LocationFactory,
    max_iterations: usize,
    pts_config: &PtsConfig,
    module: Option<&AirModule>,
) -> (PointsToMap, bool) {
    solve_with_index_config(
        constraints,
        factory,
        max_iterations,
        pts_config,
        module,
        IndexSensitivity::default(),
    )
}

/// Solve constraints with full configuration including index sensitivity.
///
/// Applies HVN (Hash-based Value Numbering) preprocessing to reduce the
/// constraint graph before solving. HVN merges pointer values with identical
/// constraint signatures into equivalence classes, typically removing 20-40%
/// of constraints for large programs.
///
/// Like `solve_with_config` but also accepts `IndexSensitivity` configuration
/// for array index tracking.
#[allow(dead_code)] // Public API for external use
pub fn solve_with_index_config(
    constraints: &ConstraintSet,
    factory: &LocationFactory,
    max_iterations: usize,
    pts_config: &PtsConfig,
    module: Option<&AirModule>,
    index_sensitivity: IndexSensitivity,
) -> (PointsToMap, bool) {
    // Phase 0: HVN preprocessing — reduce constraints by merging equivalent values
    let mut reduced = constraints.clone();
    let hvn_result = super::hvn::hvn_preprocess(&mut reduced);
    if hvn_result.removed > 0 {
        saf_log!(pta::hvn, stats, "HVN preprocessing";
            classes = hvn_result.num_classes,
            removed = hvn_result.removed,
            before = constraints.total_count(),
            after = reduced.total_count(),
        );
    }

    // Determine which representation to use
    let repr = if let Some(module) = module {
        pts_config.select_for_module(module)
    } else {
        // Without a module, use explicit config or default to BTreeSet
        match pts_config.representation {
            PtsRepresentation::Auto => PtsRepresentation::FxHash,
            explicit => explicit,
        }
    };

    // Get constants table if available
    let constants = module.map(|m| &m.constants);

    // Dispatch to the appropriate generic solver on reduced constraints
    let (mut result, iteration_limit_hit) = match repr {
        PtsRepresentation::Auto | PtsRepresentation::BTreeSet => {
            let (generic_result, limit_hit) = solve_generic_with_options::<BTreePtsSet>(
                &reduced,
                factory,
                max_iterations,
                constants,
                index_sensitivity,
                pts_config.clustering,
            );
            (normalize_result(generic_result), limit_hit)
        }
        PtsRepresentation::FxHash => {
            let (generic_result, limit_hit) = solve_generic_with_options::<FxHashPtsSet>(
                &reduced,
                factory,
                max_iterations,
                constants,
                index_sensitivity,
                pts_config.clustering,
            );
            (normalize_result(generic_result), limit_hit)
        }
        PtsRepresentation::BitVector | PtsRepresentation::Roaring => {
            let (generic_result, limit_hit) = solve_generic_with_options::<RoaringPtsSet>(
                &reduced,
                factory,
                max_iterations,
                constants,
                index_sensitivity,
                pts_config.clustering,
            );
            (normalize_result(generic_result), limit_hit)
        }
        PtsRepresentation::Bdd => {
            let (generic_result, limit_hit) = solve_generic_with_options::<BddPtsSet>(
                &reduced,
                factory,
                max_iterations,
                constants,
                index_sensitivity,
                pts_config.clustering,
            );
            (normalize_result(generic_result), limit_hit)
        }
    };

    // Expand HVN mapping: copy representative's pts to all merged originals
    for (original, rep) in &hvn_result.mapping {
        if let Some(pts) = result.get(rep).cloned() {
            result.insert(*original, pts);
        }
    }

    (result, iteration_limit_hit)
}

/// Solve constraints using Roaring bitmap representation without normalization.
///
/// Returns the raw `RoaringPtsSet` map and the HVN mapping for later expansion.
/// Use this when the caller can work with the generic representation and only
/// needs `PointsToMap` at the boundary (e.g., CG refinement hot path).
#[allow(dead_code)] // Public API for external use
pub fn solve_bitvec(
    constraints: &ConstraintSet,
    factory: &LocationFactory,
    max_iterations: usize,
    module: Option<&AirModule>,
) -> (GenericPointsToMap<RoaringPtsSet>, HvnResult) {
    let mut reduced = constraints.clone();
    let hvn_result = super::hvn::hvn_preprocess(&mut reduced);
    if hvn_result.removed > 0 {
        saf_log!(pta::hvn, stats, "HVN preprocessing";
            classes = hvn_result.num_classes,
            removed = hvn_result.removed,
            before = constraints.total_count(),
            after = reduced.total_count(),
        );
    }
    let constants = module.map(|m| &m.constants);
    let (generic_result, _iteration_limit_hit) = solve_generic_with_options::<RoaringPtsSet>(
        &reduced,
        factory,
        max_iterations,
        constants,
        IndexSensitivity::default(),
        ClusteringMode::Auto,
    );
    (generic_result, hvn_result)
}

/// Normalize a generic points-to map to `PointsToMap` and expand HVN mappings.
#[allow(dead_code)] // Public API for external use
pub fn normalize_and_expand_hvn<P: PtsSet>(
    generic: GenericPointsToMap<P>,
    hvn_result: &HvnResult,
) -> PointsToMap {
    let mut result = normalize_result(generic);
    for (original, rep) in &hvn_result.mapping {
        if let Some(pts) = result.get(rep).cloned() {
            result.insert(*original, pts);
        }
    }
    result
}

/// Solve constraints with constants table and index sensitivity.
fn solve_generic_with_options<P: PtsSet>(
    constraints: &ConstraintSet,
    factory: &LocationFactory,
    max_iterations: usize,
    constants: Option<&ConstantsTable>,
    index_sensitivity: IndexSensitivity,
    clustering: ClusteringMode,
) -> (GenericPointsToMap<P>, bool) {
    let template = create_template::<P>(constraints, clustering);
    let mut solver = GenericSolver::<P>::new_with_template(constraints, factory, template)
        .with_index_sensitivity(index_sensitivity);
    if let Some(c) = constants {
        solver = solver.with_constants(c);
    }
    solver.solve(max_iterations);
    let limit_hit = solver.iteration_limit_hit;
    (solver.pts, limit_hit)
}

/// Create a solver template, optionally pre-seeded with cluster ordering.
pub(crate) fn create_template<P: PtsSet>(constraints: &ConstraintSet, mode: ClusteringMode) -> P {
    // Collect ALL LocIds from addr constraints for frozen indexer
    let all_locs: Vec<LocId> = constraints.addr.iter().map(|a| a.loc).collect();

    let should_cluster = match mode {
        ClusteringMode::Disabled => false,
        ClusteringMode::Enabled => true,
        ClusteringMode::Auto => P::BENEFITS_FROM_CLUSTERING,
    };
    if should_cluster && !constraints.addr.is_empty() {
        // Try Steensgaard-based clustering first (richer co-occurrence from
        // load/store handling), falling through to copy-only approximation
        // if Steensgaard produces no pairs.
        #[cfg(feature = "experimental")]
        {
            let steen_config = super::steensgaard::SteensgaardConfig::default();
            let mut steen_result =
                super::steensgaard::solve_steensgaard(constraints, &steen_config);
            let matrix =
                super::clustering::cooccurrence_from_steensgaard(&mut steen_result, constraints);
            if matrix.num_pairs() > 0 {
                let config = super::clustering::ClusteringConfig::default();
                let result = super::clustering::cluster_objects(&matrix, &config);
                let ordered: Vec<saf_core::ids::LocId> =
                    result.clusters.iter().flatten().copied().collect();

                // Build frozen indexer: clustered locs first, then remaining
                let mut indexer = LocIdIndexer::new();
                indexer.register_batch(ordered.iter().copied());
                indexer.register_batch(all_locs.iter().copied());
                let frozen = std::sync::Arc::new(indexer.freeze());
                return P::with_frozen_ordering(frozen);
            }
        }

        let matrix = super::clustering::approximate_cooccurrence(constraints);
        if matrix.num_pairs() > 0 {
            let config = super::clustering::ClusteringConfig::default();
            let result = super::clustering::cluster_objects(&matrix, &config);
            let ordered: Vec<saf_core::ids::LocId> =
                result.clusters.iter().flatten().copied().collect();

            // Build frozen indexer: clustered locs first, then remaining
            let mut indexer = LocIdIndexer::new();
            indexer.register_batch(ordered.iter().copied());
            indexer.register_batch(all_locs.iter().copied());
            let frozen = std::sync::Arc::new(indexer.freeze());
            P::with_frozen_ordering(frozen)
        } else {
            // No clusters — register all locs in natural order
            let mut indexer = LocIdIndexer::new();
            indexer.register_batch(all_locs.iter().copied());
            let frozen = std::sync::Arc::new(indexer.freeze());
            P::with_frozen_ordering(frozen)
        }
    } else if !all_locs.is_empty() {
        // No clustering — register all locs in natural order
        let mut indexer = LocIdIndexer::new();
        indexer.register_batch(all_locs.iter().copied());
        let frozen = std::sync::Arc::new(indexer.freeze());
        P::with_frozen_ordering(frozen)
    } else {
        P::empty()
    }
}

/// Run iterative Tarjan's SCC algorithm starting from `start`.
///
/// Finds all strongly connected components reachable from `start` in the
/// directed graph `adj`. Uses an explicit call stack to avoid recursion
/// depth issues on large constraint graphs.
// INVARIANT: Iterative Tarjan requires mutable access to shared SCC state
// (counter, stack, on_stack, indices, lowlinks, sccs) plus the graph.
#[allow(clippy::too_many_arguments)]
fn tarjan_iterative(
    start: ValueId,
    adj: &BTreeMap<ValueId, Vec<ValueId>>,
    index_counter: &mut u32,
    stack: &mut Vec<ValueId>,
    on_stack: &mut BTreeSet<ValueId>,
    indices: &mut BTreeMap<ValueId, u32>,
    lowlinks: &mut BTreeMap<ValueId, u32>,
    sccs: &mut Vec<Vec<ValueId>>,
) {
    // Frame: (node, neighbor_index)
    let mut call_stack: Vec<(ValueId, usize)> = Vec::new();

    // Initialize start node
    indices.insert(start, *index_counter);
    lowlinks.insert(start, *index_counter);
    *index_counter += 1;
    stack.push(start);
    on_stack.insert(start);
    call_stack.push((start, 0));

    while let Some((v, ni)) = call_stack.last_mut() {
        let v = *v;
        let neighbors = adj.get(&v).map_or(&[][..], Vec::as_slice);

        if *ni < neighbors.len() {
            let w = neighbors[*ni];
            *ni += 1;

            if let std::collections::btree_map::Entry::Vacant(e) = indices.entry(w) {
                // Not visited: push onto call stack
                e.insert(*index_counter);
                lowlinks.insert(w, *index_counter);
                *index_counter += 1;
                stack.push(w);
                on_stack.insert(w);
                call_stack.push((w, 0));
            } else if on_stack.contains(&w) {
                // On stack: update lowlink
                let w_idx = indices[&w];
                if let Some(ll) = lowlinks.get_mut(&v) {
                    *ll = (*ll).min(w_idx);
                }
            }
        } else {
            // Done with all neighbors of v
            call_stack.pop();

            // Propagate lowlink to parent
            if let Some((parent, _)) = call_stack.last() {
                let v_ll = lowlinks[&v];
                if let Some(p_ll) = lowlinks.get_mut(parent) {
                    *p_ll = (*p_ll).min(v_ll);
                }
            }

            // If v is a root, pop SCC from stack
            if lowlinks[&v] == indices[&v] {
                let mut scc = Vec::new();
                loop {
                    let w = stack
                        .pop()
                        .expect("Tarjan stack is non-empty during SCC pop");
                    on_stack.remove(&w);
                    scc.push(w);
                    if w == v {
                        break;
                    }
                }
                sccs.push(scc);
            }
        }
    }
}

/// Internal solver state, generic over points-to set representation.
pub(crate) struct GenericSolver<'a, P: PtsSet> {
    /// Points-to sets for each value.
    pub(crate) pts: GenericPointsToMap<P>,
    /// Points-to sets for each location (for Load/Store).
    loc_pts: FxIndexMap<LocId, P>,
    /// Priority worklist of values to process, ordered by topological rank.
    ///
    /// Each entry is `(topo_rank, value_id)`. `BTreeSet` pops the lowest rank
    /// first, so upstream nodes (closer to addr sources) are processed before
    /// their downstream dependents.
    worklist: BTreeSet<(u32, ValueId)>,
    /// Worklist of locations to process.
    loc_worklist: IdBitSet<LocId>,
    /// Addr constraints extracted from the original `ConstraintSet`.
    /// Only these are needed after construction (all other constraints are
    /// pre-indexed in `index` + `indexed`).
    addr_constraints: Vec<AddrConstraint>,
    /// Reference to location factory.
    pub(crate) factory: &'a LocationFactory,
    /// Constants table for resolving index operands.
    constants: Option<&'a ConstantsTable>,
    /// Index sensitivity configuration.
    index_sensitivity: IndexSensitivity,
    /// Template for creating empty sets that share indexer state.
    pub(crate) template: P,
    /// Pre-built constraint index for O(1) lookup by `ValueId`.
    index: ConstraintIndex,
    /// Constraints stored as `Vec`s for indexed access.
    indexed: IndexedConstraints,
    /// Reverse index: for each location, the load constraint indices whose
    /// `src_ptr` points to that location. Built incrementally in
    /// `handle_load_constraints` so `process_location` can skip the full
    /// linear scan of all load constraints.
    ///
    /// Uses `FxHashSet<usize>` for O(1) deduplication on insert (was
    /// `Vec<usize>` with O(n) `contains` check per insertion).
    load_loc_index: FxIndexMap<LocId, FxHashSet<usize>>,
    /// Pending (src, dst) copy-edge pairs where dst's pts changed.
    /// Checked between waves for mutual subset inclusion (LCD cycle detection).
    pending_cycle_pairs: Vec<(ValueId, ValueId)>,
    /// Previous points-to sets for diff-based propagation (P2).
    ///
    /// Stores a snapshot of each value's pts from the last time it was processed.
    /// Only the diff (new elements) is propagated to successors.
    prev_pts: FxIndexMap<ValueId, P>,
    /// Previous location points-to sets for diff-based `process_location`.
    ///
    /// Mirrors `prev_pts` for the location worklist: stores a snapshot of each
    /// location's `loc_pts` from the last time it was processed. Only the diff
    /// (new elements since last processing) is propagated to load destinations.
    prev_loc_pts: FxIndexMap<LocId, P>,
    /// Maps each merged node to its SCC representative (union-find).
    rep: FxIndexMap<ValueId, ValueId>,
    /// Topological rank for each `ValueId` in the copy-constraint graph.
    ///
    /// Computed once at the start of solving via Kahn's algorithm.
    /// Nodes not in the copy graph (or in cycles) get `u32::MAX`.
    topo_order: FxIndexMap<ValueId, u32>,
    /// Whether the solver hit the iteration limit before reaching a fixed point.
    pub(crate) iteration_limit_hit: bool,
    /// Profiling statistics.
    /// Wrapped in `RefCell` for interior mutability in `find_rep(&self)`.
    stats: RefCell<SolverStats>,
}

impl<'a, P: PtsSet> GenericSolver<'a, P> {
    pub(crate) fn new(constraints: &ConstraintSet, factory: &'a LocationFactory) -> Self {
        Self::new_with_template(constraints, factory, P::empty())
    }

    pub(crate) fn new_with_template(
        constraints: &ConstraintSet,
        factory: &'a LocationFactory,
        template: P,
    ) -> Self {
        let (index, indexed) = ConstraintIndex::build(constraints);
        Self {
            pts: FxIndexMap::default(),
            loc_pts: FxIndexMap::default(),
            worklist: BTreeSet::new(),
            loc_worklist: IdBitSet::empty(),
            addr_constraints: constraints.addr.iter().cloned().collect(),
            factory,
            constants: None,
            index_sensitivity: IndexSensitivity::default(),
            template,
            index,
            indexed,
            load_loc_index: FxIndexMap::default(),
            pending_cycle_pairs: Vec::new(),
            prev_pts: FxIndexMap::default(),
            prev_loc_pts: FxIndexMap::default(),
            rep: FxIndexMap::default(),
            topo_order: FxIndexMap::default(),
            iteration_limit_hit: false,
            stats: RefCell::new(SolverStats::default()),
        }
    }

    pub(crate) fn with_constants(mut self, constants: &'a ConstantsTable) -> Self {
        self.constants = Some(constants);
        self
    }

    fn with_index_sensitivity(mut self, sensitivity: IndexSensitivity) -> Self {
        self.index_sensitivity = sensitivity;
        self
    }

    /// Get mutable access to profiling stats. Safe in `&mut self` methods.
    #[inline]
    fn stats_mut(&mut self) -> &mut SolverStats {
        self.stats.get_mut()
    }

    /// Borrow profiling stats for interior mutability in `&self` methods.
    #[inline]
    fn stats_borrow_mut(&self) -> std::cell::RefMut<'_, SolverStats> {
        self.stats.borrow_mut()
    }

    pub(crate) fn solve(&mut self, max_iterations: usize) {
        // Phase 1: Initialize from Addr constraints
        // Clone the vec to avoid borrowing self.addr_constraints while mutating self.
        let addr_constraints = self.addr_constraints.clone();
        for addr in &addr_constraints {
            if let indexmap::map::Entry::Vacant(e) = self.pts.entry(addr.ptr) {
                e.insert(self.template.clone_empty());
            }
            if self
                .pts
                .get_mut(&addr.ptr)
                .expect("just inserted")
                .insert(addr.loc)
            {
                self.worklist_insert(addr.ptr);
            }
        }

        // Compute topological ordering from copy constraints so upstream
        // nodes (closer to addr sources) are processed first.
        self.topo_order = self.compute_topo_order();

        // Re-insert all initial worklist entries with their topo priorities.
        // (They were inserted above before topo_order was computed.)
        let initial: Vec<ValueId> = self.worklist.iter().map(|&(_, v)| v).collect();
        self.worklist.clear();
        for v in initial {
            self.worklist_insert(v);
        }

        // Phase 2: Wave-based fixed-point iteration
        self.drain_worklist(max_iterations);
    }

    /// Drain the worklist to a local fixed point.
    ///
    /// Processes value and location worklists in wave-front order until
    /// both are empty or `max_iterations` is reached. Called by `solve()`
    /// after initialization, and can be called again after adding new
    /// constraints for online CG construction.
    // Timing variables (wl_start, scc_start, lcd_start, pl_start) are intentionally
    // named for readability across the solver's profiling instrumentation.
    #[allow(clippy::similar_names)]
    pub(crate) fn drain_worklist(&mut self, max_iterations: usize) {
        let mut iterations = 0;
        let mut value_pops = 0u64;

        while iterations < max_iterations {
            // Process all values at the current minimum rank (wave front)
            if let Some(&(current_rank, _)) = self.worklist.first() {
                let mut wave: Vec<ValueId> = Vec::new();

                let wl_start = SolverStats::start_section();

                while let Some(&(rank, _)) = self.worklist.first() {
                    if rank != current_rank {
                        break;
                    }
                    let (_, v) = self.worklist.pop_first().expect("just peeked");
                    wave.push(v);
                }

                SolverStats::end_section(wl_start, &mut self.stats_mut().time_worklist);

                for v in wave {
                    iterations += 1;
                    if iterations >= max_iterations {
                        self.iteration_limit_hit = true;
                        return;
                    }
                    value_pops += 1;

                    self.stats_mut().value_pops += 1;

                    if value_pops % 50_000 == 0 {
                        let scc_start = SolverStats::start_section();

                        self.detect_and_collapse_cycles();

                        SolverStats::end_section(scc_start, &mut self.stats_mut().time_scc);
                    }
                    self.process_value(v);
                }

                let lcd_start = SolverStats::start_section();

                self.check_pending_cycles();

                SolverStats::end_section(lcd_start, &mut self.stats_mut().time_lcd);

                continue;
            }

            // Process location worklist (between waves)
            if let Some(loc) = self.loc_worklist.pop_first() {
                iterations += 1;

                self.stats_mut().loc_pops += 1;

                let pl_start = SolverStats::start_section();

                self.process_location(loc);

                SolverStats::end_section(pl_start, &mut self.stats_mut().time_process_location);

                continue;
            }

            // Both worklists empty — fixed point reached
            break;
        }

        // If the while loop exited because iterations >= max_iterations
        // (e.g., after processing a location worklist item), record the limit hit.
        if iterations >= max_iterations {
            self.iteration_limit_hit = true;
        }
    }

    /// Add a copy constraint incrementally and eagerly propagate.
    ///
    /// Used by online CG construction to add interprocedural edges
    /// (actual→formal, return→caller) when new call targets are resolved.
    ///
    /// Eagerly propagates `src.pts → dst.pts` so that the diff-based
    /// worklist can pick up the new information. Without this, adding
    /// a copy edge after the initial solve would never activate: `src`
    /// has an empty diff (prev == current) and `dst` being on the
    /// worklist only propagates dst's outgoing edges, not incoming.
    pub(crate) fn add_copy_constraint(&mut self, c: CopyConstraint) {
        let src = c.src;
        let dst = c.dst;
        self.index.add_copy(&mut self.indexed, c);

        // Eagerly propagate src's current pts to dst.
        // This mirrors what handle_copy_constraints does during normal
        // worklist processing, but triggered immediately when the
        // constraint is added online.
        let src_pts = self.pts.get(&src).cloned();
        if let Some(ref src_pts) = src_pts {
            if !src_pts.is_empty() {
                let changed = self
                    .pts
                    .entry(dst)
                    .or_insert_with(|| self.template.clone_empty())
                    .union(src_pts);
                if changed {
                    self.worklist_insert(dst);
                }
                return;
            }
        }
        // If src has no pts yet, schedule dst so it gets re-evaluated
        // when src eventually gets points-to information.
        self.worklist_insert(dst);
    }

    /// Recompute topological ordering after new copy edges are added.
    ///
    /// Called after `add_copy_constraint()` batches to update the priority
    /// ordering in the worklist. Re-prioritizes existing worklist entries.
    pub(crate) fn recompute_topo_order(&mut self) {
        self.topo_order = self.compute_topo_order();
        // Re-prioritize existing worklist entries with updated ranks
        let entries: Vec<ValueId> = self.worklist.iter().map(|&(_, v)| v).collect();
        self.worklist.clear();
        for v in entries {
            self.worklist_insert(v);
        }
    }

    /// Find the representative for a value, following the union-find chain.
    ///
    /// This is the immutable version without path compression, used in
    /// `&self` contexts (e.g., `compute_topo_order`). For hot paths, prefer
    /// [`find_rep_mut`] which applies path compression.
    #[allow(clippy::similar_names)]
    fn find_rep(&self, mut v: ValueId) -> ValueId {
        let mut hops: u32 = 0;

        while let Some(&r) = self.rep.get(&v) {
            if r == v {
                break;
            }
            v = r;
            hops += 1;
        }

        {
            let mut stats = self.stats_borrow_mut();
            stats.find_rep_calls += 1;
            stats.find_rep_total_hops += u64::from(hops);
            if hops > stats.find_rep_max_chain {
                stats.find_rep_max_chain = hops;
            }
        }

        v
    }

    /// Find the representative for a value with path compression.
    ///
    /// After finding the root representative, updates all intermediate nodes
    /// to point directly to the root, making subsequent lookups nearly O(1).
    fn find_rep_mut(&mut self, v: ValueId) -> ValueId {
        // First pass: find the root representative (same as find_rep).
        let root = self.find_rep(v);

        // Second pass: path compression — point all intermediate nodes to root.
        let mut cur = v;
        while let Some(&r) = self.rep.get(&cur) {
            if r == root {
                break;
            }
            self.rep.insert(cur, root);
            cur = r;
        }

        root
    }

    /// Insert a value into the priority worklist using its topological rank.
    ///
    /// Values with lower rank (closer to addr sources in the copy graph) are
    /// processed first, so information flows "downhill". Values not in the
    /// copy graph receive `u32::MAX` and are processed last.
    fn worklist_insert(&mut self, v: ValueId) {
        let priority = self.topo_order.get(&v).copied().unwrap_or(u32::MAX);
        self.worklist.insert((priority, v));
    }

    /// Compute a topological ordering over the copy-constraint graph using Kahn's algorithm.
    ///
    /// Returns a map from `ValueId` to its topological rank. Nodes in cycles
    /// (that survive SCC collapsing) are omitted and will default to `u32::MAX`
    /// when looked up, ensuring they are processed after all acyclic nodes.
    fn compute_topo_order(&self) -> FxIndexMap<ValueId, u32> {
        // Build adjacency list and in-degree from copy constraints
        let mut adj: FxIndexMap<ValueId, Vec<ValueId>> = FxIndexMap::default();
        let mut in_degree: FxIndexMap<ValueId, u32> = FxIndexMap::default();

        for copy in &self.indexed.copy {
            let src = self.find_rep(copy.src);
            let dst = self.find_rep(copy.dst);
            if src != dst {
                adj.entry(src).or_default().push(dst);
                *in_degree.entry(dst).or_default() += 1;
                in_degree.entry(src).or_default();
            }
        }

        // Kahn's algorithm (BFS topological sort)
        let mut queue: VecDeque<ValueId> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&v, _)| v)
            .collect();

        let mut order = FxIndexMap::default();
        let mut rank: u32 = 0;

        while let Some(v) = queue.pop_front() {
            order.insert(v, rank);
            rank = rank.saturating_add(1);
            if let Some(neighbors) = adj.get(&v) {
                for &w in neighbors {
                    if let Some(deg) = in_degree.get_mut(&w) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(w);
                        }
                    }
                }
            }
        }

        order
    }

    /// Merge `other` into `rep` during SCC collapsing.
    ///
    /// Unions the points-to set of `other` into `rep`, cleans up diff tracking,
    /// and ensures `rep` is re-processed on the worklist.
    fn merge_nodes(&mut self, rep: ValueId, other: ValueId) {
        // Record representative mapping
        self.rep.insert(other, rep);

        // Merge pts sets
        if let Some(other_pts) = self.pts.swap_remove(&other) {
            self.pts
                .entry(rep)
                .or_insert_with(|| self.template.clone_empty())
                .union(&other_pts);
        }

        // Clean up diff tracking
        self.prev_pts.swap_remove(&other);

        // Ensure rep is on worklist since its pts set grew
        self.worklist_insert(rep);
    }

    /// Lazy Cycle Detection: check pending (src, dst) copy-edge pairs for
    /// actual cycles. A cycle requires both a forward copy path (src→dst)
    /// and a reverse copy edge (dst→src). Mutual pts-set inclusion alone
    /// is not sufficient — one-way propagation can produce identical sets
    /// without forming a cycle.
    fn check_pending_cycles(&mut self) {
        if self.pending_cycle_pairs.is_empty() {
            return;
        }
        self.stats_mut().lcd_invocations += 1;
        let pairs: Vec<(ValueId, ValueId)> = std::mem::take(&mut self.pending_cycle_pairs);
        for (src, dst) in pairs {
            let src = self.find_rep_mut(src);
            let dst = self.find_rep_mut(dst);
            if src == dst {
                continue;
            }
            // Verify a reverse copy edge (dst→src) exists — without this,
            // pts equality after one-way propagation would incorrectly merge
            // non-cyclic nodes.
            let has_reverse = self
                .index
                .copies_by_src(dst)
                .iter()
                .any(|&j| self.find_rep(self.indexed.copy[j].dst) == src);
            if !has_reverse {
                continue;
            }
            let (Some(src_pts), Some(dst_pts)) = (self.pts.get(&src), self.pts.get(&dst)) else {
                continue;
            };
            if src_pts.is_subset(dst_pts) && dst_pts.is_subset(src_pts) {
                let (rep, other) = if src < dst { (src, dst) } else { (dst, src) };
                self.merge_nodes(rep, other);
                self.stats_mut().lcd_merges += 1;
            }
        }
    }

    /// Detect strongly connected components in the copy-constraint graph and
    /// collapse each cycle into a single representative node.
    ///
    /// Uses iterative Tarjan's SCC algorithm to avoid stack overflow on deep
    /// chains. Called as a rare fallback from the main solve loop.
    fn detect_and_collapse_cycles(&mut self) {
        // Build adjacency list from copy constraints where both src and dst
        // have non-empty pts sets (only active edges matter for cycles).
        let mut adj: BTreeMap<ValueId, Vec<ValueId>> = BTreeMap::new();
        for copy in &self.indexed.copy {
            let src = self.find_rep(copy.src);
            let dst = self.find_rep(copy.dst);
            if src != dst && self.pts.contains_key(&src) && self.pts.contains_key(&dst) {
                adj.entry(src).or_default().push(dst);
            }
        }

        if adj.is_empty() {
            return;
        }

        // Tarjan's SCC
        let mut index_counter: u32 = 0;
        let mut stack: Vec<ValueId> = Vec::new();
        let mut on_stack: BTreeSet<ValueId> = BTreeSet::new();
        let mut indices: BTreeMap<ValueId, u32> = BTreeMap::new();
        let mut lowlinks: BTreeMap<ValueId, u32> = BTreeMap::new();
        let mut sccs: Vec<Vec<ValueId>> = Vec::new();

        // Collect all nodes that appear in the adjacency list
        let nodes: Vec<ValueId> = adj.keys().copied().collect();

        for node in &nodes {
            if !indices.contains_key(node) {
                tarjan_iterative(
                    *node,
                    &adj,
                    &mut index_counter,
                    &mut stack,
                    &mut on_stack,
                    &mut indices,
                    &mut lowlinks,
                    &mut sccs,
                );
            }
        }

        // Collapse SCCs with >1 node
        for scc in &sccs {
            if scc.len() <= 1 {
                continue;
            }
            self.stats_mut().scc_invocations += 1;
            // Pick representative (smallest `ValueId` for determinism)
            let rep = *scc.iter().min().expect("SCC is non-empty");
            for &node in scc {
                if node != rep {
                    self.merge_nodes(rep, node);
                    self.stats_mut().scc_merges += 1;
                }
            }
        }
    }

    /// Process constraints where `v` is a source.
    ///
    /// Uses diff-based propagation: only new elements (not in `prev_pts`)
    /// are propagated to successors, avoiding redundant work.
    // Timing variables (pv_start, diff_start, clone_start, t) are intentionally
    // named for readability across the solver's profiling instrumentation.
    #[allow(clippy::similar_names)]
    fn process_value(&mut self, v: ValueId) {
        let pv_start = SolverStats::start_section();

        // Canonicalize to SCC representative (with path compression)
        let v = self.find_rep_mut(v);

        self.stats_mut().process_value_calls += 1;

        // Borrow current without cloning — compute diff element-by-element
        let Some(current) = self.pts.get(&v) else {
            SolverStats::end_section(pv_start, &mut self.stats_mut().time_process_value_total);
            return;
        };

        // Compute diff WITHOUT cloning current pts set
        let diff_start = SolverStats::start_section();

        let first_visit = !self.prev_pts.contains_key(&v);
        let diff = if first_visit {
            // First time seeing this value — diff IS current (one clone)
            current.clone()
        } else {
            // Build diff element-by-element: elements in current but not in prev
            let prev = self.prev_pts.get(&v).expect("checked above");
            let mut d = self.template.clone_empty();
            for loc in current.iter() {
                if !prev.contains(loc) {
                    d.insert(loc);
                }
            }
            d
        };

        SolverStats::end_section(diff_start, &mut self.stats_mut().time_diff);

        if diff.is_empty() {
            self.stats_mut().empty_diff_skips += 1;
            SolverStats::end_section(pv_start, &mut self.stats_mut().time_process_value_total);
            return; // Nothing new to propagate
        }

        let t = SolverStats::start_section();
        self.handle_copy_constraints(v, &diff);
        SolverStats::end_section(t, &mut self.stats_mut().time_copy);

        let t = SolverStats::start_section();
        self.handle_load_constraints(v, &diff);
        SolverStats::end_section(t, &mut self.stats_mut().time_load);

        let t = SolverStats::start_section();
        self.handle_store_constraints(v, &diff);
        SolverStats::end_section(t, &mut self.stats_mut().time_store);

        let t = SolverStats::start_section();
        self.handle_gep_constraints(v, &diff);
        SolverStats::end_section(t, &mut self.stats_mut().time_gep);

        // Incremental prev_pts update (same pattern as process_location, Plan 129).
        // Move diff on first visit (zero extra clone), union diff on subsequent
        // visits (O(|diff|) vs O(|current|) full clone). Correct because pts is
        // monotone: prev ∪ diff = prev ∪ (current \ prev) = current.
        let clone_start = SolverStats::start_section();

        if first_visit {
            self.prev_pts.insert(v, diff);
        } else if let Some(prev) = self.prev_pts.get_mut(&v) {
            prev.union(&diff);
        }

        SolverStats::end_section(clone_start, &mut self.stats_mut().time_prev_pts_clone);

        SolverStats::end_section(pv_start, &mut self.stats_mut().time_process_value_total);
    }

    /// Propagate points-to sets through Copy constraints where `v` is the source.
    ///
    /// Uses field-level borrow splitting to avoid `.to_vec()` on index slices.
    #[allow(clippy::similar_names)]
    fn handle_copy_constraints(&mut self, v: ValueId, v_pts: &P) {
        let indices = self.index.copies_by_src(v);
        if indices.is_empty() {
            return;
        }

        self.stats.get_mut().copy_indices_processed += indices.len() as u64;

        let template = &self.template;
        let topo_order = &self.topo_order;

        for &i in indices {
            let dst = self.indexed.copy[i].dst;

            self.stats.get_mut().union_calls += 1;

            let changed = self
                .pts
                .entry(dst)
                .or_insert_with(|| template.clone_empty())
                .union(v_pts);

            if changed {
                self.stats.get_mut().union_changed += 1;

                let priority = topo_order.get(&dst).copied().unwrap_or(u32::MAX);
                self.worklist.insert((priority, dst));
                // Record cycle candidate for LCD: if dst's pts changed after
                // copying from v, they might form a mutual-inclusion cycle.
                self.pending_cycle_pairs.push((v, dst));
            }
        }
    }

    /// Propagate points-to sets through Load constraints where `v` is the dereferenced pointer.
    ///
    /// Uses field-level borrow splitting to avoid `.to_vec()` on index slices.
    ///
    /// Optimization (Plan 126): accumulate all `loc_pts` entries into a single
    /// temporary set, then union once into dst (instead of one clone per location).
    ///
    /// Optimization (Plan 130): hoist the accumulation out of the per-constraint
    /// loop. The accumulated set is identical for all K load constraints sharing
    /// the same `src_ptr = v`, so computing it once saves (K-1)*M lookups.
    #[allow(clippy::similar_names)]
    fn handle_load_constraints(&mut self, v: ValueId, v_pts: &P) {
        let indices = self.index.loads_by_src_ptr(v);
        if indices.is_empty() {
            return;
        }

        let template = &self.template;
        let topo_order = &self.topo_order;

        // Single pass over v_pts: accumulate loc_pts AND register load_loc_index.
        // (Plan 132: merged from two separate iterations.)
        let mut accumulated = template.clone_empty();
        let mut any_loc_found = false;
        for loc in v_pts.iter() {
            self.stats.get_mut().load_locs_iterated += 1;
            if let Some(loc_set) = self.loc_pts.get(&loc) {
                accumulated.union(loc_set);
                any_loc_found = true;
            }
            // Register load constraint indices in the reverse index so
            // `process_location` can find relevant constraints.
            let entry = self.load_loc_index.entry(loc).or_default();
            for &idx in indices {
                entry.insert(idx);
            }
        }

        // Union the shared accumulated set into each load destination.
        if any_loc_found && !accumulated.is_empty() {
            for &i in indices {
                let dst = self.indexed.load[i].dst;
                let changed = self
                    .pts
                    .entry(dst)
                    .or_insert_with(|| template.clone_empty())
                    .union(&accumulated);
                if changed {
                    let priority = topo_order.get(&dst).copied().unwrap_or(u32::MAX);
                    self.worklist.insert((priority, dst));
                }
            }
        }

        // Ensure all load destinations are tracked (even with empty set).
        for &i in indices {
            let dst = self.indexed.load[i].dst;
            if let indexmap::map::Entry::Vacant(e) = self.pts.entry(dst) {
                e.insert(template.clone_empty());
            }
        }
    }

    /// Propagate points-to sets through Store constraints where `v` is involved.
    ///
    /// Uses field-level borrow splitting to avoid `.to_vec()` on index slices
    /// and `.cloned()` on pts lookups (pts and loc_pts are disjoint fields).
    ///
    /// Part B (Plan 130): Only add to `loc_worklist` if `load_loc_index` has an
    /// entry for the location. Locations without load constraints don't need
    /// `process_location` calls. The handle_load side retroactively adds to
    /// `loc_worklist` when a new `load_loc_index` entry is created.
    #[allow(clippy::similar_names)]
    fn handle_store_constraints(&mut self, v: ValueId, v_pts: &P) {
        let template = &self.template;

        // v is the pointer being written through (dst_ptr == v):
        // For each store(dst_ptr=v, src), propagate src's pts to each loc in v_pts.
        let dst_indices = self.index.stores_by_dst_ptr(v);
        for &i in dst_indices {
            let src = self.indexed.store[i].src;
            let Some(src_pts) = self.pts.get(&src) else {
                continue;
            };
            if src_pts.is_empty() {
                continue;
            }
            for loc in v_pts.iter() {
                self.stats.get_mut().store_locs_iterated += 1;
                let changed = self
                    .loc_pts
                    .entry(loc)
                    .or_insert_with(|| template.clone_empty())
                    .union(src_pts);
                if changed && self.load_loc_index.contains_key(&loc) {
                    self.loc_worklist.insert(loc);
                }
            }
        }
        // v is the source being stored (src == v):
        // For each store(dst_ptr, src=v), propagate v_pts to each loc in dst_ptr's pts.
        let src_indices = self.index.stores_by_src(v);
        for &i in src_indices {
            let dst_ptr = self.indexed.store[i].dst_ptr;
            let Some(ptr_targets) = self.pts.get(&dst_ptr) else {
                continue;
            };
            if ptr_targets.is_empty() {
                continue;
            }
            for loc in ptr_targets.iter() {
                self.stats.get_mut().store_locs_iterated += 1;
                let changed = self
                    .loc_pts
                    .entry(loc)
                    .or_insert_with(|| template.clone_empty())
                    .union(v_pts);
                if changed && self.load_loc_index.contains_key(&loc) {
                    self.loc_worklist.insert(loc);
                }
            }
        }
    }

    /// Propagate points-to sets through GEP constraints where `v` is the source pointer.
    ///
    /// Uses field-level borrow splitting to avoid `.to_vec()` on index slices
    /// and `.clone()` on GEP path/operands. `resolve_gep_path` is a free function
    /// to avoid `&self` borrow conflicts.
    ///
    /// Optimization: accumulate all field locations per GEP constraint into one
    /// set, then union once into dst (instead of one `union_into_value` per location).
    #[allow(clippy::similar_names)]
    fn handle_gep_constraints(&mut self, v: ValueId, v_pts: &P) {
        let indices = self.index.geps_by_src_ptr(v);
        if indices.is_empty() {
            return;
        }

        let template = &self.template;
        let topo_order = &self.topo_order;
        let constants = self.constants;
        let index_sensitivity = self.index_sensitivity;

        for &i in indices {
            let gep_dst = self.indexed.gep[i].dst;

            // Plan 132: resolve GEP path ONCE per constraint (was inside inner loop).
            // Free function avoids &self borrow conflict with &mut self.pts below.
            let resolved_path = resolve_gep_path(
                &self.indexed.gep[i].path,
                &self.indexed.gep[i].index_operands,
                constants,
                index_sensitivity,
            );

            // Accumulate ALL field locations for this GEP constraint into one set
            let mut accumulated = template.clone_empty();
            for loc in v_pts.iter() {
                self.stats.get_mut().gep_locs_iterated += 1;
                if let Some(base_loc) = self.factory.get(loc) {
                    let merged = merge_gep_with_base_path(base_loc, &resolved_path);
                    let field_loc = merged
                        .as_ref()
                        .and_then(|p| self.factory.lookup_approx(base_loc.obj, p))
                        .or_else(|| {
                            let new_path = base_loc.path.extend(&resolved_path);
                            self.factory.lookup_approx(base_loc.obj, &new_path)
                        });

                    if let Some(field_loc) = field_loc {
                        accumulated.insert(field_loc);
                    }
                }
            }

            // Single union into dst instead of one per location
            if !accumulated.is_empty() {
                let changed = self
                    .pts
                    .entry(gep_dst)
                    .or_insert_with(|| template.clone_empty())
                    .union(&accumulated);
                if changed {
                    let priority = topo_order.get(&gep_dst).copied().unwrap_or(u32::MAX);
                    self.worklist.insert((priority, gep_dst));
                }
            }
        }
    }

    /// Process constraints affected by location `loc` changing.
    ///
    /// Uses diff-based propagation (mirroring `process_value`): only new
    /// elements in `loc_pts[loc]` since the last processing are propagated
    /// to load destinations. This avoids re-unioning the entire set on every
    /// call, dramatically reducing work for popular locations.
    ///
    /// Uses the `load_loc_index` reverse index built incrementally in
    /// `handle_load_constraints` to find only the load constraints whose
    /// `src_ptr` is known to point to `loc`.
    ///
    /// Optimizations (Plan 129):
    /// - Early-exit if no load constraints reference this location
    /// - `SmallVec` for load indices avoids heap allocation
    /// - Incremental `prev_loc_pts` update (union diff or move) instead of
    ///   full clone of current
    #[allow(clippy::similar_names)]
    fn process_location(&mut self, loc: LocId) {
        // Opt 1: Early-exit if no load constraints reference this location.
        // Many locations are written to but never loaded from.
        if !self.load_loc_index.contains_key(&loc) {
            self.stats_mut().proc_loc_early_exits += 1;
            return;
        }

        let Some(current) = self.loc_pts.get(&loc) else {
            return;
        };

        let first_visit = !self.prev_loc_pts.contains_key(&loc);

        // Compute diff: elements in current loc_pts but not in prev_loc_pts
        let diff = if first_visit {
            // First visit: diff IS current (one clone)
            current.clone()
        } else {
            let prev = self.prev_loc_pts.get(&loc).expect("checked above");
            let mut d = self.template.clone_empty();
            for l in current.iter() {
                if !prev.contains(l) {
                    d.insert(l);
                }
            }
            d
        };

        // Stats counters deferred past diff computation to avoid borrow
        // conflict with `current` (which borrows self.loc_pts).
        if first_visit {
            self.stats_mut().proc_loc_first_visits += 1;
        }

        if diff.is_empty() {
            self.stats_mut().proc_loc_diff_empty += 1;
            return;
        }

        // Opt 2: Copy load indices to stack-local SmallVec (avoids heap clone).
        // Most locations have 1-4 load constraints, fitting in inline storage.
        let load_indices: SmallVec<[usize; 8]> = self
            .load_loc_index
            .get(&loc)
            .map_or_else(SmallVec::new, |s| s.iter().copied().collect());

        // Propagate diff to load destinations
        for i in load_indices {
            let dst = self.indexed.load[i].dst;
            if self.union_into_value(dst, &diff) {
                self.worklist_insert(dst);
            }
        }

        // Opt 3: Update prev_loc_pts incrementally instead of cloning current.
        if first_visit {
            // Move diff into prev_loc_pts — zero additional clones since diff
            // IS current on first visit and propagation is already done.
            self.prev_loc_pts.insert(loc, diff);
        } else {
            // Union diff into existing prev — O(|diff|) vs O(|current|) for
            // full clone. Correctness: loc_pts is monotone, so
            // prev ∪ diff = prev ∪ (current \ prev) = current.
            if let Some(prev) = self.prev_loc_pts.get_mut(&loc) {
                prev.union(&diff);
            }
        }
    }

    /// Add locations to a value's points-to set. Returns true if changed.
    fn union_into_value(&mut self, v: ValueId, locs: &P) -> bool {
        self.pts
            .entry(v)
            .or_insert_with(|| self.template.clone_empty())
            .union(locs)
    }
}

impl<P: PtsSet> GenericSolver<'_, P> {
    /// Print profiling statistics via `saf_log!`.
    pub(crate) fn print_stats(&mut self, label: &str) {
        let constraint_counts = (
            self.indexed.copy.len(),
            self.indexed.load.len(),
            self.indexed.store.len(),
            self.indexed.gep.len(),
        );
        self.stats_mut().print_summary(label, constraint_counts);
    }
}

// Legacy solver for backwards compatibility (delegates to generic)
#[allow(dead_code)]
struct Solver<'a> {
    inner: GenericSolver<'a, BTreePtsSet>,
}

#[allow(dead_code)]
impl<'a> Solver<'a> {
    fn new(constraints: &'a ConstraintSet, factory: &'a LocationFactory) -> Self {
        Self {
            inner: GenericSolver::new(constraints, factory),
        }
    }

    fn solve(&mut self, max_iterations: usize) {
        self.inner.solve(max_iterations);
    }

    /// Get the result as a standard `PointsToMap`.
    fn into_result(self) -> PointsToMap {
        normalize_result(self.inner.pts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::Constant;
    use saf_core::ids::{ObjId, ValueId};

    use crate::pta::config::FieldSensitivity;
    use crate::pta::constraint::{
        AddrConstraint, ConstraintSet, CopyConstraint, GepConstraint, LoadConstraint,
        StoreConstraint,
    };
    use crate::pta::location::{FieldPath, LocationFactory};
    use crate::pta::ptsset::{BddPtsSet, RoaringPtsSet};

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 })
    }

    #[test]
    fn solver_empty_constraints() {
        let factory = make_factory();
        let constraints = ConstraintSet::default();
        let result = solve(&constraints, &factory, 1_000_000);
        assert!(result.is_empty());
    }

    #[test]
    fn solver_single_addr_constraint() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let ptr = ValueId::new(1);
        let obj = ObjId::new(100);
        let loc = factory.get_or_create(obj, FieldPath::empty());

        constraints.addr.insert(AddrConstraint { ptr, loc });

        let result = solve(&constraints, &factory, 1_000_000);

        assert_eq!(result.get(&ptr).map(|s| s.len()), Some(1));
        assert!(result.get(&ptr).unwrap().contains(&loc));
    }

    #[test]
    fn solver_copy_propagates_points_to() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let p = ValueId::new(1);
        let q = ValueId::new(2);
        let obj = ObjId::new(100);
        let loc = factory.get_or_create(obj, FieldPath::empty());

        // p points to loc
        constraints.addr.insert(AddrConstraint { ptr: p, loc });
        // q = p (copy)
        constraints.copy.insert(CopyConstraint { dst: q, src: p });

        let result = solve(&constraints, &factory, 1_000_000);

        // Both p and q should point to loc
        assert!(result.get(&p).unwrap().contains(&loc));
        assert!(result.get(&q).unwrap().contains(&loc));
    }

    #[test]
    fn solver_copy_chain() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let a = ValueId::new(1);
        let b = ValueId::new(2);
        let c = ValueId::new(3);
        let obj = ObjId::new(100);
        let loc = factory.get_or_create(obj, FieldPath::empty());

        // a -> b -> c
        constraints.addr.insert(AddrConstraint { ptr: a, loc });
        constraints.copy.insert(CopyConstraint { dst: b, src: a });
        constraints.copy.insert(CopyConstraint { dst: c, src: b });

        let result = solve(&constraints, &factory, 1_000_000);

        // All should point to loc
        assert!(result.get(&a).unwrap().contains(&loc));
        assert!(result.get(&b).unwrap().contains(&loc));
        assert!(result.get(&c).unwrap().contains(&loc));
    }

    #[test]
    fn solver_load_dereference() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        // p -> loc_x (location of x)
        // *p = &y (store y to location pointed by p)
        // q = *p (load from p)
        // Result: q should point to y

        let p = ValueId::new(1);
        let q = ValueId::new(2);
        let loc_x_obj = ObjId::new(100);
        let loc_y_obj = ObjId::new(200);
        let loc_x = factory.get_or_create(loc_x_obj, FieldPath::empty());
        let loc_y = factory.get_or_create(loc_y_obj, FieldPath::empty());

        // p points to loc_x
        constraints
            .addr
            .insert(AddrConstraint { ptr: p, loc: loc_x });

        // We also need something that points to loc_y so we can store it
        let y_ptr = ValueId::new(3);
        constraints.addr.insert(AddrConstraint {
            ptr: y_ptr,
            loc: loc_y,
        });

        // Store y_ptr's points-to into loc_x (via p)
        constraints.store.insert(StoreConstraint {
            dst_ptr: p,
            src: y_ptr,
        });

        // Load from p into q
        constraints
            .load
            .insert(LoadConstraint { dst: q, src_ptr: p });

        let result = solve(&constraints, &factory, 1_000_000);

        // q should point to loc_y
        assert!(
            result.get(&q).map_or(false, |s| s.contains(&loc_y)),
            "q should point to loc_y after load through p"
        );
    }

    #[test]
    fn solver_store_write() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let p = ValueId::new(1);
        let val = ValueId::new(2);
        let loc_p = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc_val = factory.get_or_create(ObjId::new(200), FieldPath::empty());

        // p -> loc_p
        constraints
            .addr
            .insert(AddrConstraint { ptr: p, loc: loc_p });
        // val -> loc_val
        constraints.addr.insert(AddrConstraint {
            ptr: val,
            loc: loc_val,
        });
        // *p = val (store)
        constraints.store.insert(StoreConstraint {
            dst_ptr: p,
            src: val,
        });

        // We check by loading from p in a new solve
        let q = ValueId::new(3);
        let mut constraints2 = constraints.clone();
        constraints2
            .load
            .insert(LoadConstraint { dst: q, src_ptr: p });

        let result2 = solve(&constraints2, &factory, 1_000_000);
        assert!(result2.get(&q).map_or(false, |s| s.contains(&loc_val)));
    }

    #[test]
    fn solver_gep_field_access() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let base = ValueId::new(1);
        let field_ptr = ValueId::new(2);
        let obj = ObjId::new(100);
        let base_loc = factory.get_or_create(obj, FieldPath::empty());

        // Pre-create the field location so solver can find it
        let _field_loc = factory.get_or_create(obj, FieldPath::field(0));

        // base -> base_loc
        constraints.addr.insert(AddrConstraint {
            ptr: base,
            loc: base_loc,
        });

        // field_ptr = gep(base, .0)
        constraints.gep.insert(GepConstraint {
            dst: field_ptr,
            src_ptr: base,
            path: FieldPath::field(0),
            index_operands: vec![],
        });

        let result = solve(&constraints, &factory, 1_000_000);

        // field_ptr should point to obj.field[0]
        // Check that field_ptr points to a location with field path
        assert!(
            result.get(&field_ptr).is_some_and(|s| !s.is_empty()),
            "field_ptr should have a points-to set"
        );
    }

    #[test]
    fn solver_multiple_locations() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let p = ValueId::new(1);
        let q = ValueId::new(2);
        let r = ValueId::new(3);

        let loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc2 = factory.get_or_create(ObjId::new(200), FieldPath::empty());

        // p -> loc1, q -> loc2
        constraints
            .addr
            .insert(AddrConstraint { ptr: p, loc: loc1 });
        constraints
            .addr
            .insert(AddrConstraint { ptr: q, loc: loc2 });

        // r = p or r = q (simulated by two copy edges)
        constraints.copy.insert(CopyConstraint { dst: r, src: p });
        constraints.copy.insert(CopyConstraint { dst: r, src: q });

        let result = solve(&constraints, &factory, 1_000_000);

        // r should point to both loc1 and loc2
        let r_pts = result.get(&r).expect("r should have points-to set");
        assert!(r_pts.contains(&loc1));
        assert!(r_pts.contains(&loc2));
        assert_eq!(r_pts.len(), 2);
    }

    #[test]
    fn solver_iteration_limit() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let p = ValueId::new(1);
        let q = ValueId::new(2);
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        constraints.addr.insert(AddrConstraint { ptr: p, loc });
        constraints.copy.insert(CopyConstraint { dst: q, src: p });

        // Should still work with very limited iterations for simple case
        let result = solve(&constraints, &factory, 10);
        assert!(result.get(&q).map_or(false, |s| s.contains(&loc)));
    }

    #[test]
    fn solver_deterministic_results() {
        let mut factory1 = make_factory();
        let mut factory2 = make_factory();
        let mut constraints = ConstraintSet::default();

        // Create some constraints in a specific order
        for i in 0..10 {
            let ptr = ValueId::new(i);
            let loc = factory1.get_or_create(ObjId::new(i as u128 * 100), FieldPath::empty());
            factory2.get_or_create(ObjId::new(i as u128 * 100), FieldPath::empty());
            constraints.addr.insert(AddrConstraint { ptr, loc });
        }

        // Add copy chain
        for i in 1..10 {
            constraints.copy.insert(CopyConstraint {
                dst: ValueId::new(i),
                src: ValueId::new(i - 1),
            });
        }

        let result1 = solve(&constraints, &factory1, 1_000_000);
        let result2 = solve(&constraints, &factory2, 1_000_000);

        // Results should be identical
        assert_eq!(result1.len(), result2.len());
        for (k, v1) in &result1 {
            let v2 = result2.get(k).expect("key should exist in both");
            assert_eq!(v1, v2, "points-to sets should be identical for {:?}", k);
        }
    }

    // Tests for generic solver with different representations

    #[test]
    fn solver_generic_btreeptsset() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let p = ValueId::new(1);
        let q = ValueId::new(2);
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        constraints.addr.insert(AddrConstraint { ptr: p, loc });
        constraints.copy.insert(CopyConstraint { dst: q, src: p });

        let result = solve_generic::<BTreePtsSet>(&constraints, &factory, 1_000_000);

        assert!(result.get(&p).map_or(false, |s| s.contains(loc)));
        assert!(result.get(&q).map_or(false, |s| s.contains(loc)));
    }

    #[test]
    fn solver_generic_bddptsset() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let p = ValueId::new(1);
        let q = ValueId::new(2);
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        constraints.addr.insert(AddrConstraint { ptr: p, loc });
        constraints.copy.insert(CopyConstraint { dst: q, src: p });

        let result = solve_generic::<BddPtsSet>(&constraints, &factory, 1_000_000);

        assert!(result.get(&p).map_or(false, |s| s.contains(loc)));
        assert!(result.get(&q).map_or(false, |s| s.contains(loc)));
    }

    #[test]
    fn solver_generic_roaringptsset() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let p = ValueId::new(1);
        let q = ValueId::new(2);
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        constraints.addr.insert(AddrConstraint { ptr: p, loc });
        constraints.copy.insert(CopyConstraint { dst: q, src: p });

        let result = solve_generic::<RoaringPtsSet>(&constraints, &factory, 1_000_000);

        assert!(result.get(&p).map_or(false, |s| s.contains(loc)));
        assert!(result.get(&q).map_or(false, |s| s.contains(loc)));
    }

    #[test]
    fn solver_all_representations_equivalent() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        // Create a moderately complex constraint set
        for i in 0..5 {
            let ptr = ValueId::new(i);
            let loc = factory.get_or_create(ObjId::new(i as u128 * 100), FieldPath::empty());
            constraints.addr.insert(AddrConstraint { ptr, loc });
        }

        for i in 1..5 {
            constraints.copy.insert(CopyConstraint {
                dst: ValueId::new(i),
                src: ValueId::new(i - 1),
            });
        }

        // Solve with all three representations
        let result_btree = solve_generic::<BTreePtsSet>(&constraints, &factory, 1_000_000);
        let result_roaring = solve_generic::<RoaringPtsSet>(&constraints, &factory, 1_000_000);
        let result_bdd = solve_generic::<BddPtsSet>(&constraints, &factory, 1_000_000);

        // Normalize all results
        let norm_btree = normalize_result(result_btree);
        let norm_roaring = normalize_result(result_roaring);
        let norm_bdd = normalize_result(result_bdd);

        // All should be equal
        assert_eq!(
            norm_btree, norm_roaring,
            "BTree and Roaring should produce same result"
        );
        assert_eq!(
            norm_btree, norm_bdd,
            "BTree and BDD should produce same result"
        );
    }

    // =========================================================================
    // Tests for index sensitivity
    // =========================================================================

    #[test]
    fn solver_index_sensitivity_collapsed() {
        // With Collapsed sensitivity, different index operands should not matter
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let base = ValueId::new(1);
        let ptr0 = ValueId::new(2);
        let ptr1 = ValueId::new(3);
        let idx0 = ValueId::new(100);
        let idx1 = ValueId::new(101);
        let obj = ObjId::new(1000);

        // Create base location and indexed location
        let base_loc = factory.get_or_create(obj, FieldPath::empty());
        let _idx_loc = factory.get_or_create(obj, FieldPath::index());

        // base -> base_loc
        constraints.addr.insert(AddrConstraint {
            ptr: base,
            loc: base_loc,
        });

        // ptr0 = gep(base, [idx0])
        constraints.gep.insert(GepConstraint {
            dst: ptr0,
            src_ptr: base,
            path: FieldPath::index(),
            index_operands: vec![idx0],
        });

        // ptr1 = gep(base, [idx1])
        constraints.gep.insert(GepConstraint {
            dst: ptr1,
            src_ptr: base,
            path: FieldPath::index(),
            index_operands: vec![idx1],
        });

        // Without any constants, both should collapse to same location
        let (result, _) = solve_generic_with_options::<BTreePtsSet>(
            &constraints,
            &factory,
            1_000_000,
            None,
            IndexSensitivity::Collapsed,
            ClusteringMode::Disabled,
        );
        let result = normalize_result(result);

        // Both should point to the same location (collapsed)
        assert_eq!(
            result.get(&ptr0),
            result.get(&ptr1),
            "With Collapsed, different indices should resolve to same location"
        );
    }

    #[test]
    fn solver_index_sensitivity_constant_resolves() {
        // With ConstantOnly, constant indices should be distinguished
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let base = ValueId::new(1);
        let ptr0 = ValueId::new(2);
        let ptr1 = ValueId::new(3);
        let idx0 = ValueId::new(100);
        let idx1 = ValueId::new(101);
        let obj = ObjId::new(1000);

        // Create locations for indices 0 and 1
        let base_loc = factory.get_or_create(obj, FieldPath::empty());
        let loc0 = factory.get_or_create(obj, FieldPath::index_constant(0));
        let loc1 = factory.get_or_create(obj, FieldPath::index_constant(1));

        // base -> base_loc
        constraints.addr.insert(AddrConstraint {
            ptr: base,
            loc: base_loc,
        });

        // ptr0 = gep(base, [idx0]) where idx0 = 0
        constraints.gep.insert(GepConstraint {
            dst: ptr0,
            src_ptr: base,
            path: FieldPath::index(),
            index_operands: vec![idx0],
        });

        // ptr1 = gep(base, [idx1]) where idx1 = 1
        constraints.gep.insert(GepConstraint {
            dst: ptr1,
            src_ptr: base,
            path: FieldPath::index(),
            index_operands: vec![idx1],
        });

        // Create constants table
        let mut constants = ConstantsTable::new();
        constants.insert(idx0, Constant::Int { value: 0, bits: 64 });
        constants.insert(idx1, Constant::Int { value: 1, bits: 64 });

        let (result, _) = solve_generic_with_options::<BTreePtsSet>(
            &constraints,
            &factory,
            1_000_000,
            Some(&constants),
            IndexSensitivity::ConstantOnly,
            ClusteringMode::Disabled,
        );
        let result = normalize_result(result);

        // ptr0 should point to loc0, ptr1 should point to loc1
        assert!(
            result.get(&ptr0).map_or(false, |s| s.contains(&loc0)),
            "ptr0 should point to index 0 location"
        );
        assert!(
            result.get(&ptr1).map_or(false, |s| s.contains(&loc1)),
            "ptr1 should point to index 1 location"
        );
        assert_ne!(
            result.get(&ptr0),
            result.get(&ptr1),
            "Different constant indices should resolve to different locations"
        );
    }

    #[test]
    fn solver_index_sensitivity_symbolic() {
        // With Symbolic, non-constant indices should be tracked as symbolic
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let base = ValueId::new(1);
        let ptr_i = ValueId::new(2);
        let ptr_j = ValueId::new(3);
        let idx_i = ValueId::new(100);
        let idx_j = ValueId::new(101);
        let obj = ObjId::new(1000);

        // Create locations for symbolic indices
        let base_loc = factory.get_or_create(obj, FieldPath::empty());
        let loc_i = factory.get_or_create(obj, FieldPath::index_symbolic(idx_i));
        let loc_j = factory.get_or_create(obj, FieldPath::index_symbolic(idx_j));

        // base -> base_loc
        constraints.addr.insert(AddrConstraint {
            ptr: base,
            loc: base_loc,
        });

        // ptr_i = gep(base, [idx_i])
        constraints.gep.insert(GepConstraint {
            dst: ptr_i,
            src_ptr: base,
            path: FieldPath::index(),
            index_operands: vec![idx_i],
        });

        // ptr_j = gep(base, [idx_j])
        constraints.gep.insert(GepConstraint {
            dst: ptr_j,
            src_ptr: base,
            path: FieldPath::index(),
            index_operands: vec![idx_j],
        });

        // No constants - both are symbolic
        let (result, _) = solve_generic_with_options::<BTreePtsSet>(
            &constraints,
            &factory,
            1_000_000,
            None,
            IndexSensitivity::Symbolic,
            ClusteringMode::Disabled,
        );
        let result = normalize_result(result);

        // Each symbolic index should get its own location
        assert!(
            result.get(&ptr_i).map_or(false, |s| s.contains(&loc_i)),
            "ptr_i should point to symbolic location for idx_i"
        );
        assert!(
            result.get(&ptr_j).map_or(false, |s| s.contains(&loc_j)),
            "ptr_j should point to symbolic location for idx_j"
        );
        assert_ne!(
            result.get(&ptr_i),
            result.get(&ptr_j),
            "Different symbolic indices should resolve to different locations"
        );
    }
}
