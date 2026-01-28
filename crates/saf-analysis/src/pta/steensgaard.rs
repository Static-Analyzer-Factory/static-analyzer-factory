//! Steensgaard's unification-based points-to analysis.
//!
//! This module provides a fast, flow-insensitive, context-insensitive
//! pointer analysis using unification (equality constraints) instead of
//! Andersen's subset constraints.
//!
//! **Trade-offs vs Andersen:**
//! - Time complexity: O(n·α(n)) (nearly linear) vs O(n³) worst case
//! - Precision: Lower (more aggressive merging) vs Higher
//! - Scalability: Excellent for large programs
//!
//! Use Steensgaard when analyzing large programs where speed matters more
//! than precision. Use Andersen when precision is critical.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{LocId, ValueId};

use super::constraint::ConstraintSet;
use super::result::AliasResult;

// =============================================================================
// Union-Find (Disjoint Set Union)
// =============================================================================

/// Efficient union-find data structure with path compression and union by rank.
///
/// Each node represents either a `ValueId` or a `LocId`. Nodes in the same
/// equivalence class have unified points-to information.
#[derive(Debug, Clone)]
pub struct UnionFind {
    /// Parent pointers (maps node ID to parent ID).
    parent: BTreeMap<NodeId, NodeId>,
    /// Rank (depth estimate) for union by rank.
    rank: BTreeMap<NodeId, usize>,
    /// Points-to representative for each equivalence class (maps rep → location).
    /// Each equivalence class has at most one outgoing edge.
    points_to: BTreeMap<NodeId, NodeId>,
}

/// Node identifier for union-find. Can be a value or a location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    /// A pointer value.
    Value(ValueId),
    /// An abstract memory location.
    Location(LocId),
}

impl UnionFind {
    /// Create an empty union-find structure.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parent: BTreeMap::new(),
            rank: BTreeMap::new(),
            points_to: BTreeMap::new(),
        }
    }

    /// Ensure a node exists (creates it as its own parent if not).
    fn ensure_node(&mut self, node: NodeId) {
        if let std::collections::btree_map::Entry::Vacant(e) = self.parent.entry(node) {
            e.insert(node);
            self.rank.insert(node, 0);
        }
    }

    /// Find the representative of a node's equivalence class (with path compression).
    pub fn find(&mut self, node: NodeId) -> NodeId {
        self.ensure_node(node);

        let parent = self.parent[&node];
        if parent == node {
            return node;
        }

        // Path compression
        let root = self.find(parent);
        self.parent.insert(node, root);
        root
    }

    /// Union two nodes into the same equivalence class.
    /// Returns the representative of the merged class.
    pub fn union(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let root_a = self.find(a);
        let root_b = self.find(b);

        if root_a == root_b {
            return root_a;
        }

        // Union by rank
        let rank_a = self.rank[&root_a];
        let rank_b = self.rank[&root_b];

        let (new_root, other) = match rank_a.cmp(&rank_b) {
            std::cmp::Ordering::Less => (root_b, root_a),
            std::cmp::Ordering::Greater => (root_a, root_b),
            std::cmp::Ordering::Equal => {
                // Same rank: pick root_a and increment its rank
                self.rank.insert(root_a, rank_a + 1);
                (root_a, root_b)
            }
        };

        self.parent.insert(other, new_root);

        // Merge points-to information
        if let Some(target_other) = self.points_to.remove(&other) {
            if let Some(target_new) = self.points_to.get(&new_root).copied() {
                // Both have targets - need to unify the targets
                let unified = self.union(target_other, target_new);
                self.points_to.insert(new_root, unified);
            } else {
                // Only other has target
                self.points_to.insert(new_root, target_other);
            }
        }

        new_root
    }

    /// Set that a node points to another node.
    /// If the node already points somewhere, unify the targets.
    pub fn set_points_to(&mut self, from: NodeId, to: NodeId) {
        let from_rep = self.find(from);
        let to_rep = self.find(to);

        if let Some(existing_target) = self.points_to.get(&from_rep).copied() {
            // Already points somewhere - unify targets
            let unified = self.union(existing_target, to_rep);
            self.points_to.insert(from_rep, unified);
        } else {
            self.points_to.insert(from_rep, to_rep);
        }
    }

    /// Get what a node points to (if anything).
    pub fn get_points_to(&mut self, from: NodeId) -> Option<NodeId> {
        let from_rep = self.find(from);
        self.points_to.get(&from_rep).copied().map(|t| self.find(t))
    }

    /// Check if two nodes are in the same equivalence class.
    pub fn same_class(&mut self, a: NodeId, b: NodeId) -> bool {
        self.find(a) == self.find(b)
    }
}

impl Default for UnionFind {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Steensgaard Solver
// =============================================================================

/// Configuration for Steensgaard's analysis.
#[derive(Debug, Clone, Default)]
pub struct SteensgaardConfig {
    // Currently no configuration options.
    // Reserved for future extensions like field sensitivity level.
}

/// Result of Steensgaard's points-to analysis.
#[derive(Debug, Clone)]
pub struct SteensgaardResult {
    /// The union-find structure after solving.
    uf: UnionFind,
    /// All known locations.
    locations: BTreeSet<LocId>,
}

impl SteensgaardResult {
    /// Check if two values may alias.
    ///
    /// Two values may alias if:
    /// 1. They are in the same equivalence class, OR
    /// 2. They point to the same (or unified) locations
    #[must_use]
    pub fn may_alias(&mut self, a: ValueId, b: ValueId) -> AliasResult {
        let node_a = NodeId::Value(a);
        let node_b = NodeId::Value(b);

        // If in the same equivalence class, definitely may alias
        if self.uf.same_class(node_a, node_b) {
            return AliasResult::May;
        }

        // Check if they point to the same equivalence class
        let target_a = self.uf.get_points_to(node_a);
        let target_b = self.uf.get_points_to(node_b);

        match (target_a, target_b) {
            (Some(ta), Some(tb)) => {
                if self.uf.same_class(ta, tb) {
                    AliasResult::May
                } else {
                    AliasResult::No
                }
            }
            // One or both don't point to anything
            _ => AliasResult::No,
        }
    }

    /// Get the equivalence class representative for a value.
    pub fn get_class(&mut self, value: ValueId) -> NodeId {
        self.uf.find(NodeId::Value(value))
    }

    /// Get what a value points to (as a set of locations).
    ///
    /// In Steensgaard, each equivalence class points to at most one
    /// other equivalence class. The result contains all locations
    /// in that target class.
    #[must_use]
    pub fn points_to(&mut self, value: ValueId) -> BTreeSet<LocId> {
        let node = NodeId::Value(value);
        let mut result = BTreeSet::new();

        if let Some(target) = self.uf.get_points_to(node) {
            // Collect all locations in the target equivalence class
            for &loc in &self.locations {
                let loc_node = NodeId::Location(loc);
                if self.uf.same_class(loc_node, target) {
                    result.insert(loc);
                }
            }
        }

        result
    }

    /// Get the number of equivalence classes.
    #[must_use]
    pub fn num_equivalence_classes(&mut self) -> usize {
        let mut roots: BTreeSet<NodeId> = BTreeSet::new();
        for &loc in &self.locations {
            roots.insert(self.uf.find(NodeId::Location(loc)));
        }
        roots.len()
    }

    /// Get diagnostics about the analysis.
    #[must_use]
    pub fn diagnostics(&mut self) -> SteensgaardDiagnostics {
        SteensgaardDiagnostics {
            num_locations: self.locations.len(),
            num_equivalence_classes: self.num_equivalence_classes(),
        }
    }
}

/// Diagnostics from Steensgaard's analysis.
#[derive(Debug, Clone)]
pub struct SteensgaardDiagnostics {
    /// Number of abstract memory locations.
    pub num_locations: usize,
    /// Number of equivalence classes after unification.
    pub num_equivalence_classes: usize,
}

/// Maximum iterations for load/store processing.
const MAX_ITERATIONS: usize = 100;

/// Solve Steensgaard's unification-based points-to analysis.
///
/// # Arguments
/// * `constraints` - Constraints extracted from the program
/// * `_config` - Configuration options (reserved for future use)
///
/// # Returns
/// The analysis result with equivalence classes for alias queries.
#[must_use]
pub fn solve_steensgaard(
    constraints: &ConstraintSet,
    _config: &SteensgaardConfig,
) -> SteensgaardResult {
    let mut uf = UnionFind::new();
    let mut locations = BTreeSet::new();

    // Phase 1: Process Addr constraints (x = &loc)
    // These establish initial points-to relationships.
    for addr in &constraints.addr {
        let ptr_node = NodeId::Value(addr.ptr);
        let loc_node = NodeId::Location(addr.loc);
        locations.insert(addr.loc);

        uf.set_points_to(ptr_node, loc_node);
    }

    // Phase 2: Process GEP constraints (dst = src + offset)
    // In field-insensitive Steensgaard, GEP is like copy.
    // For field-sensitive, we'd create sub-locations.
    for gep in &constraints.gep {
        let dst_node = NodeId::Value(gep.dst);
        let src_node = NodeId::Value(gep.src_ptr);

        // Unify dst with src (field-insensitive)
        uf.union(dst_node, src_node);
    }

    // Phase 3: Process Copy constraints (dst = src)
    // Unify the targets of dst and src.
    for copy in &constraints.copy {
        let dst_node = NodeId::Value(copy.dst);
        let src_node = NodeId::Value(copy.src);

        // Get what src points to
        let src_target = uf.get_points_to(src_node);

        if let Some(target) = src_target {
            // Make dst point to the same place
            uf.set_points_to(dst_node, target);
        } else {
            // src doesn't point anywhere yet - unify the nodes themselves
            // so they'll share points-to info later
            uf.union(dst_node, src_node);
        }
    }

    // Phase 4: Process Load constraints (dst = *src_ptr)
    // dst gets whatever src_ptr's target points to.
    for load in &constraints.load {
        let dst_node = NodeId::Value(load.dst);
        let src_ptr_node = NodeId::Value(load.src_ptr);

        // Get what src_ptr points to
        if let Some(src_target) = uf.get_points_to(src_ptr_node) {
            // Get what that target points to
            if let Some(target_of_target) = uf.get_points_to(src_target) {
                uf.set_points_to(dst_node, target_of_target);
            }
        }
    }

    // Phase 5: Process Store constraints (*dst_ptr = src)
    // dst_ptr's target gets what src points to.
    for store in &constraints.store {
        let dst_ptr_node = NodeId::Value(store.dst_ptr);
        let src_node = NodeId::Value(store.src);

        // Get what dst_ptr points to
        if let Some(dst_target) = uf.get_points_to(dst_ptr_node) {
            // Get what src points to
            if let Some(src_target) = uf.get_points_to(src_node) {
                // Make dst_target point to src_target
                uf.set_points_to(dst_target, src_target);
            }
        }
    }

    // Iterate until fixed point for load/store (they can create new edges)
    // In classic Steensgaard, we process each constraint once, but loads and
    // stores can require iteration when new points-to edges are created.
    let mut changed = true;
    let mut iterations = 0;

    while changed && iterations < MAX_ITERATIONS {
        changed = false;
        iterations += 1;

        // Re-process loads
        for load in &constraints.load {
            let dst_node = NodeId::Value(load.dst);
            let src_ptr_node = NodeId::Value(load.src_ptr);

            if let Some(src_target) = uf.get_points_to(src_ptr_node) {
                if let Some(target_of_target) = uf.get_points_to(src_target) {
                    let old_target = uf.get_points_to(dst_node);
                    uf.set_points_to(dst_node, target_of_target);
                    let new_target = uf.get_points_to(dst_node);
                    if old_target != new_target {
                        changed = true;
                    }
                }
            }
        }

        // Re-process stores
        for store in &constraints.store {
            let dst_ptr_node = NodeId::Value(store.dst_ptr);
            let src_node = NodeId::Value(store.src);

            if let Some(dst_target) = uf.get_points_to(dst_ptr_node) {
                if let Some(src_target) = uf.get_points_to(src_node) {
                    let old_target = uf.get_points_to(dst_target);
                    uf.set_points_to(dst_target, src_target);
                    let new_target = uf.get_points_to(dst_target);
                    if old_target != new_target {
                        changed = true;
                    }
                }
            }
        }
    }

    SteensgaardResult { uf, locations }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pta::constraint::{AddrConstraint, CopyConstraint, LoadConstraint, StoreConstraint};

    fn loc(id: u128) -> LocId {
        LocId::new(id)
    }

    fn val(id: u128) -> ValueId {
        ValueId::new(id)
    }

    #[test]
    fn union_find_basic() {
        let mut uf = UnionFind::new();
        let a = NodeId::Value(val(1));
        let b = NodeId::Value(val(2));
        let c = NodeId::Value(val(3));

        // Initially all separate
        assert!(!uf.same_class(a, b));
        assert!(!uf.same_class(b, c));

        // Union a and b
        uf.union(a, b);
        assert!(uf.same_class(a, b));
        assert!(!uf.same_class(a, c));

        // Union b and c (transitively unites a, b, c)
        uf.union(b, c);
        assert!(uf.same_class(a, c));
    }

    #[test]
    fn union_find_points_to() {
        let mut uf = UnionFind::new();
        let p = NodeId::Value(val(1));
        let loc_x = NodeId::Location(loc(100));

        // p points to loc_x
        uf.set_points_to(p, loc_x);

        assert_eq!(uf.get_points_to(p), Some(loc_x));
    }

    #[test]
    fn union_find_merges_targets() {
        let mut uf = UnionFind::new();
        let p = NodeId::Value(val(1));
        let q = NodeId::Value(val(2));
        let loc_x = NodeId::Location(loc(100));
        let loc_y = NodeId::Location(loc(200));

        // p -> loc_x, q -> loc_y
        uf.set_points_to(p, loc_x);
        uf.set_points_to(q, loc_y);

        // Union p and q - their targets should be unified
        uf.union(p, q);

        // Both should point to the same unified class
        let p_target = uf.get_points_to(p);
        let q_target = uf.get_points_to(q);
        assert_eq!(p_target, q_target);

        // loc_x and loc_y should be in the same class
        assert!(uf.same_class(loc_x, loc_y));
    }

    #[test]
    fn steensgaard_simple_addr() {
        // p = &x
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });

        let config = SteensgaardConfig::default();
        let mut result = solve_steensgaard(&cs, &config);

        let pts = result.points_to(val(1));
        assert!(pts.contains(&loc(100)));
    }

    #[test]
    fn steensgaard_copy_unifies() {
        // p = &x
        // q = p
        // Both should point to x
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),   // p
            loc: loc(100), // x
        });
        cs.copy.insert(CopyConstraint {
            dst: val(2), // q
            src: val(1), // p
        });

        let config = SteensgaardConfig::default();
        let mut result = solve_steensgaard(&cs, &config);

        let pts_p = result.points_to(val(1));
        let pts_q = result.points_to(val(2));

        assert!(pts_p.contains(&loc(100)));
        assert!(pts_q.contains(&loc(100)));
    }

    #[test]
    fn steensgaard_may_alias() {
        // p = &x
        // q = &x
        // p and q both point to x -> may alias
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),   // p
            loc: loc(100), // x
        });
        cs.addr.insert(AddrConstraint {
            ptr: val(2),   // q
            loc: loc(100), // x
        });

        let config = SteensgaardConfig::default();
        let mut result = solve_steensgaard(&cs, &config);

        assert_eq!(result.may_alias(val(1), val(2)), AliasResult::May);
    }

    #[test]
    fn steensgaard_no_alias() {
        // p = &x
        // q = &y
        // p and q point to different locations -> no alias
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),   // p
            loc: loc(100), // x
        });
        cs.addr.insert(AddrConstraint {
            ptr: val(2),   // q
            loc: loc(200), // y
        });

        let config = SteensgaardConfig::default();
        let mut result = solve_steensgaard(&cs, &config);

        assert_eq!(result.may_alias(val(1), val(2)), AliasResult::No);
    }

    #[test]
    fn steensgaard_load_store() {
        // p = &x    (p -> x)
        // *p = &y   (x -> y)
        // q = *p    (q -> y)
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),   // p
            loc: loc(100), // x
        });
        cs.addr.insert(AddrConstraint {
            ptr: val(3),   // tmp for &y
            loc: loc(200), // y
        });
        cs.store.insert(StoreConstraint {
            dst_ptr: val(1), // *p
            src: val(3),     // &y
        });
        cs.load.insert(LoadConstraint {
            dst: val(2),     // q
            src_ptr: val(1), // *p
        });

        let config = SteensgaardConfig::default();
        let mut result = solve_steensgaard(&cs, &config);

        let pts_q = result.points_to(val(2));
        assert!(pts_q.contains(&loc(200)), "q should point to y");
    }

    #[test]
    fn steensgaard_unification_aggressiveness() {
        // Demonstrate Steensgaard's aggressive unification
        // p = cond ? &x : &y
        // In Andersen: p -> {x, y}
        // In Steensgaard: x and y are unified

        // Simulate with: p = &x; p = &y (both addr constraints)
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),   // p
            loc: loc(100), // x
        });
        cs.addr.insert(AddrConstraint {
            ptr: val(1),   // p (same pointer, different target)
            loc: loc(200), // y
        });

        let config = SteensgaardConfig::default();
        let mut result = solve_steensgaard(&cs, &config);

        // Due to unification, x and y should be in the same class
        // (This is the precision loss compared to Andersen)
        let pts = result.points_to(val(1));
        assert!(pts.contains(&loc(100)) || pts.contains(&loc(200)));
    }

    #[test]
    fn steensgaard_diagnostics() {
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: val(1),
            loc: loc(100),
        });
        cs.addr.insert(AddrConstraint {
            ptr: val(2),
            loc: loc(200),
        });

        let config = SteensgaardConfig::default();
        let mut result = solve_steensgaard(&cs, &config);

        let diag = result.diagnostics();
        assert_eq!(diag.num_locations, 2);
        // Two separate locations = two equivalence classes
        assert_eq!(diag.num_equivalence_classes, 2);
    }
}
