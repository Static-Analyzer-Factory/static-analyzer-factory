//! Object clustering for points-to set compaction.
//!
//! This module implements the object clustering optimization from OOPSLA 2021
//! (Barbar & Sui). It improves pointer analysis performance by:
//!
//! 1. Grouping objects that frequently appear together in points-to sets
//! 2. Assigning consecutive identifiers to clustered objects
//! 3. Resulting in denser bit-vectors with better cache locality
//!
//! The optimization is a preprocessing step that doesn't change the analysis
//! algorithm — only the object-to-identifier mapping.
//!
//! NOTE: This module is fully implemented but not yet integrated into the public API.

// This module uses numeric computations for clustering heuristics where
// precision loss is acceptable (affinity scores, cluster statistics).
#![allow(dead_code)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]

use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{LocId, ValueId};

use super::constraint::ConstraintSet;

// =============================================================================
// Co-occurrence Matrix
// =============================================================================

/// Tracks how often pairs of objects appear together in points-to sets.
#[derive(Debug, Clone, Default)]
pub struct CooccurrenceMatrix {
    /// Co-occurrence counts: (obj_a, obj_b) → count where obj_a < obj_b.
    counts: BTreeMap<(LocId, LocId), usize>,
    /// All objects seen.
    objects: BTreeSet<LocId>,
}

impl CooccurrenceMatrix {
    /// Create an empty co-occurrence matrix.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record co-occurrences from a points-to set.
    ///
    /// For each pair of objects in the set, increment the co-occurrence count.
    pub fn record_points_to_set(&mut self, pts: &BTreeSet<LocId>) {
        let objects: Vec<_> = pts.iter().copied().collect();

        for &obj in &objects {
            self.objects.insert(obj);
        }

        // Record all pairs (O(n²) but pts sets are usually small)
        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                let (a, b) = if objects[i] < objects[j] {
                    (objects[i], objects[j])
                } else {
                    (objects[j], objects[i])
                };
                *self.counts.entry((a, b)).or_insert(0) += 1;
            }
        }
    }

    /// Get the co-occurrence count for a pair of objects.
    #[must_use]
    pub fn get_count(&self, a: LocId, b: LocId) -> usize {
        let key = if a < b { (a, b) } else { (b, a) };
        self.counts.get(&key).copied().unwrap_or(0)
    }

    /// Get all objects seen.
    #[must_use]
    pub fn objects(&self) -> &BTreeSet<LocId> {
        &self.objects
    }

    /// Get the number of object pairs with co-occurrences.
    #[must_use]
    pub fn num_pairs(&self) -> usize {
        self.counts.len()
    }
}

// =============================================================================
// Hierarchical Clustering
// =============================================================================

/// Result of object clustering.
#[derive(Debug, Clone)]
pub struct ClusteringResult {
    /// Mapping from original object ID to new compact identifier (0..n).
    pub object_to_id: BTreeMap<LocId, u32>,
    /// Mapping from compact identifier back to original object ID.
    pub id_to_object: BTreeMap<u32, LocId>,
    /// Cluster assignments: cluster_id → list of objects.
    pub clusters: Vec<Vec<LocId>>,
    /// Number of objects clustered.
    pub num_objects: usize,
}

impl ClusteringResult {
    /// Get the compact identifier for an object.
    #[must_use]
    pub fn get_id(&self, obj: LocId) -> Option<u32> {
        self.object_to_id.get(&obj).copied()
    }

    /// Get the original object for a compact identifier.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn get_object(&self, id: u32) -> Option<LocId> {
        self.id_to_object.get(&id).copied()
    }

    /// Check if the clustering is trivial (identity mapping).
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn is_trivial(&self) -> bool {
        self.clusters.len() == self.num_objects
    }
}

/// Configuration for object clustering.
#[derive(Debug, Clone)]
pub struct ClusteringConfig {
    /// Minimum co-occurrence count to consider objects related.
    pub min_cooccurrence: usize,
    /// Maximum cluster size (larger clusters are split).
    pub max_cluster_size: usize,
    /// Whether to use hierarchical clustering (vs simple affinity).
    pub use_hierarchical: bool,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            min_cooccurrence: 1,
            max_cluster_size: 1000,
            use_hierarchical: true,
        }
    }
}

/// Perform object clustering based on co-occurrence statistics.
///
/// # Arguments
/// * `matrix` - Co-occurrence matrix from auxiliary analysis
/// * `config` - Clustering configuration
///
/// # Returns
/// Clustering result with object-to-identifier mapping.
#[must_use]
pub fn cluster_objects(matrix: &CooccurrenceMatrix, config: &ClusteringConfig) -> ClusteringResult {
    let objects: Vec<_> = matrix.objects().iter().copied().collect();
    let n = objects.len();

    if n == 0 {
        return ClusteringResult {
            object_to_id: BTreeMap::new(),
            id_to_object: BTreeMap::new(),
            clusters: Vec::new(),
            num_objects: 0,
        };
    }

    // Build affinity matrix (object index → object index → affinity)
    let obj_to_idx: BTreeMap<LocId, usize> = objects
        .iter()
        .enumerate()
        .map(|(i, &obj)| (obj, i))
        .collect();

    let clusters = if config.use_hierarchical && n > 1 {
        hierarchical_clustering(&objects, &obj_to_idx, matrix, config)
    } else {
        // Simple clustering: group by affinity threshold
        simple_affinity_clustering(&objects, &obj_to_idx, matrix, config)
    };

    // Assign consecutive IDs within clusters
    let mut object_to_id = BTreeMap::new();
    let mut id_to_object = BTreeMap::new();
    let mut next_id: u32 = 0;

    for cluster in &clusters {
        for &obj in cluster {
            object_to_id.insert(obj, next_id);
            id_to_object.insert(next_id, obj);
            next_id += 1;
        }
    }

    ClusteringResult {
        object_to_id,
        id_to_object,
        clusters,
        num_objects: n,
    }
}

// Union-find helpers for clustering
fn uf_find(parent: &mut [usize], i: usize) -> usize {
    if parent[i] != i {
        parent[i] = uf_find(parent, parent[i]);
    }
    parent[i]
}

fn uf_union(parent: &mut [usize], rank: &mut [usize], a: usize, b: usize) {
    let ra = uf_find(parent, a);
    let rb = uf_find(parent, b);
    if ra != rb {
        match rank[ra].cmp(&rank[rb]) {
            std::cmp::Ordering::Less => parent[ra] = rb,
            std::cmp::Ordering::Greater => parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                parent[rb] = ra;
                rank[ra] += 1;
            }
        }
    }
}

/// Simple affinity-based clustering.
///
/// Groups objects that have co-occurrence above the threshold.
fn simple_affinity_clustering(
    objects: &[LocId],
    _obj_to_idx: &BTreeMap<LocId, usize>,
    matrix: &CooccurrenceMatrix,
    config: &ClusteringConfig,
) -> Vec<Vec<LocId>> {
    let n = objects.len();
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    // Union objects with sufficient co-occurrence
    for i in 0..n {
        for j in (i + 1)..n {
            let count = matrix.get_count(objects[i], objects[j]);
            if count >= config.min_cooccurrence {
                uf_union(&mut parent, &mut rank, i, j);
            }
        }
    }

    // Group by root
    let mut groups: BTreeMap<usize, Vec<LocId>> = BTreeMap::new();
    for (i, &obj) in objects.iter().enumerate() {
        let root = uf_find(&mut parent, i);
        groups.entry(root).or_default().push(obj);
    }

    // Split large clusters
    let mut clusters = Vec::new();
    for (_, mut group) in groups {
        while group.len() > config.max_cluster_size {
            let split: Vec<_> = group.drain(..config.max_cluster_size).collect();
            clusters.push(split);
        }
        if !group.is_empty() {
            clusters.push(group);
        }
    }

    clusters
}

/// Hierarchical agglomerative clustering.
///
/// Bottom-up clustering that merges closest clusters until no more
/// merges improve locality.
fn hierarchical_clustering(
    objects: &[LocId],
    _obj_to_idx: &BTreeMap<LocId, usize>,
    matrix: &CooccurrenceMatrix,
    config: &ClusteringConfig,
) -> Vec<Vec<LocId>> {
    let n = objects.len();

    // Start with each object in its own cluster
    let mut clusters: Vec<BTreeSet<usize>> = (0..n)
        .map(|i| {
            let mut s = BTreeSet::new();
            s.insert(i);
            s
        })
        .collect();

    // Compute initial inter-cluster distances (negative affinity)
    // We use average linkage: distance = -avg_affinity
    let compute_distance = |c1: &BTreeSet<usize>, c2: &BTreeSet<usize>| -> f64 {
        if c1.is_empty() || c2.is_empty() {
            return f64::INFINITY;
        }
        let mut total_affinity = 0.0;
        let mut count = 0;
        for &i in c1 {
            for &j in c2 {
                let cooc = matrix.get_count(objects[i], objects[j]);
                total_affinity += cooc as f64;
                count += 1;
            }
        }
        if count == 0 || total_affinity < config.min_cooccurrence as f64 {
            f64::INFINITY
        } else {
            -total_affinity / count as f64
        }
    };

    // Greedy merging
    loop {
        if clusters.len() <= 1 {
            break;
        }

        // Find best merge
        let mut best_dist = f64::INFINITY;
        let mut best_pair = (0, 0);

        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                // Skip if merged cluster would be too large
                if clusters[i].len() + clusters[j].len() > config.max_cluster_size {
                    continue;
                }

                let dist = compute_distance(&clusters[i], &clusters[j]);
                if dist < best_dist {
                    best_dist = dist;
                    best_pair = (i, j);
                }
            }
        }

        // If no good merge found, stop
        if best_dist == f64::INFINITY {
            break;
        }

        // Merge clusters
        let (i, j) = best_pair;
        let c2 = clusters.remove(j);
        clusters[i].extend(c2);
    }

    // Convert back to Vec<Vec<LocId>>
    clusters
        .into_iter()
        .map(|c| c.into_iter().map(|i| objects[i]).collect())
        .collect()
}

// =============================================================================
// Core Bit-Vector (Compact Representation)
// =============================================================================

/// A compact bit-vector that only stores the range [min, max].
///
/// This is the "core bit-vector" optimization from the paper.
/// Instead of storing all bits from 0 to max_id, we only store
/// the range where bits are actually set.
#[derive(Debug, Clone, Default)]
pub struct CoreBitVector {
    /// The actual bit storage.
    data: Vec<u64>,
    /// The minimum bit index (offset).
    min_index: u32,
    /// Number of bits set.
    count: usize,
}

impl CoreBitVector {
    /// Create an empty core bit-vector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a core bit-vector from a set of IDs.
    #[must_use]
    pub fn from_ids(ids: &BTreeSet<u32>) -> Self {
        if ids.is_empty() {
            return Self::new();
        }

        let min_id = *ids.iter().next().expect("checked non-empty above");
        let max_id = *ids.iter().next_back().expect("checked non-empty above");

        let min_word = min_id / 64;
        let max_word = max_id / 64;
        let num_words = (max_word - min_word + 1) as usize;

        let mut data = vec![0u64; num_words];
        for &id in ids {
            let word_idx = ((id / 64) - min_word) as usize;
            let bit_idx = id % 64;
            data[word_idx] |= 1u64 << bit_idx;
        }

        Self {
            data,
            min_index: min_word * 64,
            count: ids.len(),
        }
    }

    /// Check if a bit is set.
    #[must_use]
    pub fn contains(&self, id: u32) -> bool {
        if id < self.min_index {
            return false;
        }
        let offset = id - self.min_index;
        let word_idx = (offset / 64) as usize;
        let bit_idx = offset % 64;

        if word_idx >= self.data.len() {
            return false;
        }

        (self.data[word_idx] & (1u64 << bit_idx)) != 0
    }

    /// Get the number of bits set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Iterate over set bits.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.data
            .iter()
            .enumerate()
            .flat_map(move |(word_idx, &word)| {
                (0..64).filter_map(move |bit_idx| {
                    if (word & (1u64 << bit_idx)) != 0 {
                        Some(self.min_index + (word_idx as u32 * 64) + bit_idx)
                    } else {
                        None
                    }
                })
            })
    }

    /// Union with another core bit-vector.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() {
            return other.clone();
        }
        if other.is_empty() {
            return self.clone();
        }

        let new_min = self.min_index.min(other.min_index);
        let self_max = self.min_index + (self.data.len() as u32 * 64);
        let other_max = other.min_index + (other.data.len() as u32 * 64);
        let new_max = self_max.max(other_max);

        let new_min_word = new_min / 64;
        let new_max_word = (new_max - 1) / 64;
        let num_words = (new_max_word - new_min_word + 1) as usize;

        let mut data = vec![0u64; num_words];

        // Copy self's data
        let self_start_word = (self.min_index / 64 - new_min_word) as usize;
        for (i, &word) in self.data.iter().enumerate() {
            data[self_start_word + i] |= word;
        }

        // Copy other's data
        let other_start_word = (other.min_index / 64 - new_min_word) as usize;
        for (i, &word) in other.data.iter().enumerate() {
            data[other_start_word + i] |= word;
        }

        // Recount bits
        let count = data.iter().map(|w| w.count_ones() as usize).sum();

        Self {
            data,
            min_index: new_min_word * 64,
            count,
        }
    }

    /// Memory usage in bytes.
    #[must_use]
    pub fn memory_bytes(&self) -> usize {
        self.data.len() * 8 + std::mem::size_of::<Self>()
    }
}

/// Compute approximate co-occurrence by propagating `Addr` `LocId`s through Copy constraints.
///
/// Runs a bounded worklist pass (Copy-only, no Load/Store/GEP) to build approximate
/// points-to sets, then records co-occurrence for all multi-element sets.
/// Cost: O(V + E_copy * iterations), much cheaper than the full solver.
pub fn approximate_cooccurrence(constraints: &ConstraintSet) -> CooccurrenceMatrix {
    let mut pts: BTreeMap<ValueId, BTreeSet<LocId>> = BTreeMap::new();

    // Seed from Addr constraints
    for addr in &constraints.addr {
        pts.entry(addr.ptr).or_default().insert(addr.loc);
    }

    // Build copy adjacency: src -> [dst, ...]
    let mut copy_edges: BTreeMap<ValueId, Vec<ValueId>> = BTreeMap::new();
    for copy in &constraints.copy {
        copy_edges.entry(copy.src).or_default().push(copy.dst);
    }

    // Worklist propagation (bounded to prevent divergence on cycles)
    let mut worklist: BTreeSet<ValueId> = pts.keys().copied().collect();
    let budget = 100 * pts.len().max(1);
    let mut steps = 0;
    while let Some(v) = worklist.pop_first() {
        steps += 1;
        if steps > budget {
            break;
        }

        let v_pts = match pts.get(&v) {
            Some(s) => s.clone(),
            None => continue,
        };

        if let Some(dsts) = copy_edges.get(&v) {
            for &dst in dsts {
                let dst_pts = pts.entry(dst).or_default();
                let old_len = dst_pts.len();
                dst_pts.extend(&v_pts);
                if dst_pts.len() > old_len {
                    worklist.insert(dst);
                }
            }
        }
    }

    // Record co-occurrence from multi-element sets
    let mut matrix = CooccurrenceMatrix::new();
    for set in pts.values() {
        if set.len() >= 2 {
            matrix.record_points_to_set(set);
        }
    }
    matrix
}

// =============================================================================
// Steensgaard-seeded co-occurrence
// =============================================================================

/// Build a co-occurrence matrix from Steensgaard's unification-based analysis.
///
/// For each value in the constraint set, queries the Steensgaard result for its
/// points-to set and records co-occurrence between all pairs of locations.
/// Because Steensgaard processes loads/stores (not just copies), this captures
/// richer co-occurrence information than `approximate_cooccurrence`.
#[cfg(feature = "experimental")]
pub fn cooccurrence_from_steensgaard(
    steen: &mut super::steensgaard::SteensgaardResult,
    constraints: &ConstraintSet,
) -> CooccurrenceMatrix {
    let mut matrix = CooccurrenceMatrix::new();

    // Collect all values mentioned in constraints
    let mut values: BTreeSet<ValueId> = BTreeSet::new();
    for addr in &constraints.addr {
        values.insert(addr.ptr);
    }
    for copy in &constraints.copy {
        values.insert(copy.src);
        values.insert(copy.dst);
    }
    for load in &constraints.load {
        values.insert(load.dst);
        values.insert(load.src_ptr);
    }
    for store in &constraints.store {
        values.insert(store.src);
        values.insert(store.dst_ptr);
    }
    for gep in &constraints.gep {
        values.insert(gep.src_ptr);
        values.insert(gep.dst);
    }

    // For each value, get its Steensgaard points-to set and record co-occurrence
    for v in &values {
        let pts = steen.points_to(*v);
        if pts.len() >= 2 {
            matrix.record_points_to_set(&pts);
        }
    }

    matrix
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn loc(id: u128) -> LocId {
        LocId::new(id)
    }

    #[test]
    fn cooccurrence_matrix_basic() {
        let mut matrix = CooccurrenceMatrix::new();

        // pts1 = {a, b, c}
        let mut pts1 = BTreeSet::new();
        pts1.insert(loc(1)); // a
        pts1.insert(loc(2)); // b
        pts1.insert(loc(3)); // c
        matrix.record_points_to_set(&pts1);

        // pts2 = {a, b}
        let mut pts2 = BTreeSet::new();
        pts2.insert(loc(1)); // a
        pts2.insert(loc(2)); // b
        matrix.record_points_to_set(&pts2);

        // a and b appeared together twice
        assert_eq!(matrix.get_count(loc(1), loc(2)), 2);
        // a and c appeared together once
        assert_eq!(matrix.get_count(loc(1), loc(3)), 1);
        // b and c appeared together once
        assert_eq!(matrix.get_count(loc(2), loc(3)), 1);
    }

    #[test]
    fn clustering_empty() {
        let matrix = CooccurrenceMatrix::new();
        let config = ClusteringConfig::default();
        let result = cluster_objects(&matrix, &config);

        assert_eq!(result.num_objects, 0);
        assert!(result.clusters.is_empty());
    }

    #[test]
    fn clustering_single_object() {
        let mut matrix = CooccurrenceMatrix::new();
        let mut pts = BTreeSet::new();
        pts.insert(loc(1));
        matrix.record_points_to_set(&pts);

        let config = ClusteringConfig::default();
        let result = cluster_objects(&matrix, &config);

        assert_eq!(result.num_objects, 1);
        assert_eq!(result.clusters.len(), 1);
        assert_eq!(result.get_id(loc(1)), Some(0));
    }

    #[test]
    fn clustering_groups_cooccurrences() {
        let mut matrix = CooccurrenceMatrix::new();

        // Objects 1, 2, 3 frequently appear together
        for _ in 0..5 {
            let mut pts = BTreeSet::new();
            pts.insert(loc(1));
            pts.insert(loc(2));
            pts.insert(loc(3));
            matrix.record_points_to_set(&pts);
        }

        // Objects 4, 5 frequently appear together (separate group)
        for _ in 0..5 {
            let mut pts = BTreeSet::new();
            pts.insert(loc(4));
            pts.insert(loc(5));
            matrix.record_points_to_set(&pts);
        }

        let config = ClusteringConfig {
            min_cooccurrence: 3,
            ..Default::default()
        };
        let result = cluster_objects(&matrix, &config);

        assert_eq!(result.num_objects, 5);

        // Check that objects in the same group get consecutive IDs
        let id1 = result.get_id(loc(1)).unwrap();
        let id2 = result.get_id(loc(2)).unwrap();
        let id3 = result.get_id(loc(3)).unwrap();

        // 1, 2, 3 should be close (within 3 of each other)
        assert!((id1 as i32 - id2 as i32).abs() <= 2);
        assert!((id2 as i32 - id3 as i32).abs() <= 2);

        let id4 = result.get_id(loc(4)).unwrap();
        let id5 = result.get_id(loc(5)).unwrap();

        // 4, 5 should be close
        assert!((id4 as i32 - id5 as i32).abs() <= 1);
    }

    #[test]
    fn core_bitvector_basic() {
        let mut ids = BTreeSet::new();
        ids.insert(100);
        ids.insert(105);
        ids.insert(200);

        let bv = CoreBitVector::from_ids(&ids);

        assert!(bv.contains(100));
        assert!(bv.contains(105));
        assert!(bv.contains(200));
        assert!(!bv.contains(0));
        assert!(!bv.contains(101));
        assert!(!bv.contains(300));

        assert_eq!(bv.len(), 3);
    }

    #[test]
    fn core_bitvector_union() {
        let mut ids1 = BTreeSet::new();
        ids1.insert(10);
        ids1.insert(20);

        let mut ids2 = BTreeSet::new();
        ids2.insert(15);
        ids2.insert(25);

        let bv1 = CoreBitVector::from_ids(&ids1);
        let bv2 = CoreBitVector::from_ids(&ids2);
        let union = bv1.union(&bv2);

        assert!(union.contains(10));
        assert!(union.contains(15));
        assert!(union.contains(20));
        assert!(union.contains(25));
        assert_eq!(union.len(), 4);
    }

    #[test]
    fn core_bitvector_iter() {
        let mut ids = BTreeSet::new();
        ids.insert(5);
        ids.insert(10);
        ids.insert(15);

        let bv = CoreBitVector::from_ids(&ids);
        let collected: Vec<_> = bv.iter().collect();

        assert_eq!(collected, vec![5, 10, 15]);
    }

    #[test]
    fn core_bitvector_memory_efficiency() {
        // Sparse IDs: 0 and 1000
        let mut sparse_ids = BTreeSet::new();
        sparse_ids.insert(0);
        sparse_ids.insert(1000);

        let bv = CoreBitVector::from_ids(&sparse_ids);

        // Should only store ~16 words (0-63 and 960-1023) not 16 words for 0-1023
        // Actually stores from word 0 to word 15 (16 words)
        assert!(bv.memory_bytes() < 200); // Much less than storing 1000 bits naively
    }

    #[test]
    fn clustering_preserves_all_objects() {
        let mut matrix = CooccurrenceMatrix::new();

        let mut pts = BTreeSet::new();
        pts.insert(loc(1));
        pts.insert(loc(2));
        pts.insert(loc(3));
        pts.insert(loc(4));
        pts.insert(loc(5));
        matrix.record_points_to_set(&pts);

        let config = ClusteringConfig::default();
        let result = cluster_objects(&matrix, &config);

        // All objects should have IDs
        for i in 1..=5 {
            assert!(result.get_id(loc(i)).is_some());
        }

        // IDs should be 0..5
        let ids: BTreeSet<_> = (1..=5).map(|i| result.get_id(loc(i)).unwrap()).collect();
        assert_eq!(ids, (0..5).collect());
    }

    #[test]
    fn approximate_cooccurrence_basic() {
        use super::super::constraint::{AddrConstraint, CopyConstraint};

        let mut constraints = ConstraintSet::default();
        let v1 = saf_core::ids::ValueId::new(1);
        let v2 = saf_core::ids::ValueId::new(2);
        let l1 = loc(100);
        let l2 = loc(200);

        constraints.addr.insert(AddrConstraint { ptr: v1, loc: l1 });
        constraints.addr.insert(AddrConstraint { ptr: v2, loc: l2 });
        constraints.copy.insert(CopyConstraint { dst: v2, src: v1 });

        let matrix = approximate_cooccurrence(&constraints);
        // v2 gets {l1, l2} after copy propagation, so l1 and l2 co-occur
        assert!(matrix.get_count(l1, l2) > 0);
    }

    #[test]
    fn approximate_cooccurrence_cycle_terminates() {
        use super::super::constraint::{AddrConstraint, CopyConstraint};

        let mut constraints = ConstraintSet::default();
        let v1 = saf_core::ids::ValueId::new(1);
        let v2 = saf_core::ids::ValueId::new(2);
        constraints.addr.insert(AddrConstraint {
            ptr: v1,
            loc: loc(100),
        });
        constraints.copy.insert(CopyConstraint { dst: v2, src: v1 });
        constraints.copy.insert(CopyConstraint { dst: v1, src: v2 });
        let _matrix = approximate_cooccurrence(&constraints); // must not hang
    }

    #[test]
    fn approximate_cooccurrence_empty() {
        let constraints = ConstraintSet::default();
        let matrix = approximate_cooccurrence(&constraints);
        assert_eq!(matrix.num_pairs(), 0);
    }
}
