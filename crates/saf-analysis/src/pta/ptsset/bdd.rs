//! BDD-backed points-to set implementation.
//!
//! Uses Binary Decision Diagrams (BDDs) for compact representation of large
//! points-to sets. Best for programs with >100K allocation sites where sets
//! share significant structure.
//!
//! Each location index is encoded in binary using `ceil(log2(max_index))` BDD
//! variables. Set `{a, b}` = `encode(idx_a) OR encode(idx_b)`.

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet, BddVariableSetBuilder};
use saf_core::ids::LocId;

use super::indexer::LocIdIndexer;
use super::trait_def::PtsSet;

/// Shared BDD context containing variable set and current encoding width.
///
/// The encoding uses binary representation: each location index is encoded
/// using `num_vars` BDD variables. As more locations are added, the encoding
/// may need to grow.
#[derive(Debug)]
pub struct BddContext {
    /// The BDD variable set for building BDDs.
    vars: BddVariableSet,

    /// Number of variables used for encoding (bit width).
    /// Each location index uses this many BDD variables.
    num_vars: usize,

    /// Pre-computed variable references for efficient encoding.
    var_refs: Vec<BddVariable>,
}

impl BddContext {
    /// Create a new BDD context with the given encoding width.
    ///
    /// # Arguments
    /// * `num_vars` - Number of BDD variables (bit width). Should be at least
    ///   `ceil(log2(max_expected_indices + 1))`.
    #[must_use]
    pub fn new(num_vars: usize) -> Self {
        let num_vars = num_vars.max(1); // At least 1 variable

        let mut builder = BddVariableSetBuilder::new();
        let mut var_refs = Vec::with_capacity(num_vars);

        for i in 0..num_vars {
            let var = builder.make_variable(&format!("v{i}"));
            var_refs.push(var);
        }

        Self {
            vars: builder.build(),
            num_vars,
            var_refs,
        }
    }

    /// Create a context with enough variables to encode up to `max_value` indices.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn for_max_value(max_value: usize) -> Self {
        let num_vars = Self::bits_needed(max_value);
        Self::new(num_vars)
    }

    /// Calculate number of bits needed to encode values up to `max_value`.
    pub(crate) fn bits_needed(max_value: usize) -> usize {
        if max_value == 0 {
            1
        } else {
            (usize::BITS - max_value.leading_zeros()) as usize
        }
    }

    /// Get the number of encoding variables.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn num_vars(&self) -> usize {
        self.num_vars
    }

    /// Get the maximum value that can be encoded with current variables.
    #[must_use]
    pub fn max_encodable(&self) -> usize {
        (1 << self.num_vars) - 1
    }

    /// Check if a value can be encoded with current variables.
    #[must_use]
    pub fn can_encode(&self, value: usize) -> bool {
        value <= self.max_encodable()
    }

    /// Create an empty BDD (false).
    #[must_use]
    pub fn empty(&self) -> Bdd {
        self.vars.mk_false()
    }

    /// Create a full BDD (true) — represents all possible values.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn full(&self) -> Bdd {
        self.vars.mk_true()
    }

    /// Encode a single index as a BDD (minterm).
    ///
    /// The index is encoded in binary: bit `i` of the index corresponds to
    /// variable `i`. If the bit is 1, use the variable; if 0, use its negation.
    /// The result is the conjunction of all literals.
    #[must_use]
    pub fn encode_index(&self, idx: usize) -> Bdd {
        // Start with true
        let mut result = self.vars.mk_true();

        for (i, &var) in self.var_refs.iter().enumerate() {
            let bit_set = (idx >> i) & 1 == 1;
            let literal = if bit_set {
                self.vars.mk_var(var)
            } else {
                self.vars.mk_not_var(var)
            };
            result = result.and(&literal);
        }

        result
    }

    /// Decode a BDD to extract all encoded indices.
    ///
    /// Returns indices in ascending order for deterministic iteration.
    pub fn decode_indices(&self, bdd: &Bdd) -> Vec<usize> {
        if bdd.is_false() {
            return Vec::new();
        }

        let mut indices = Vec::new();

        // Enumerate all satisfying assignments
        for valuation in bdd.sat_valuations() {
            let mut idx = 0usize;
            for (i, &var) in self.var_refs.iter().enumerate() {
                if valuation[var] {
                    idx |= 1 << i;
                }
            }
            indices.push(idx);
        }

        // Sort for deterministic order
        indices.sort_unstable();
        indices
    }

    /// Count the number of encoded indices (satisfying assignments).
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn count_indices(&self, bdd: &Bdd) -> usize {
        bdd.cardinality().to_string().parse().unwrap_or(0)
    }

    /// Check if an index is in the BDD.
    #[must_use]
    pub fn contains_index(&self, bdd: &Bdd, idx: usize) -> bool {
        if !self.can_encode(idx) {
            return false;
        }

        // Check if the minterm for this index is satisfiable in the BDD
        // (i.e., the AND of the BDD and the encoded index is not false)
        let minterm = self.encode_index(idx);
        !bdd.and(&minterm).is_false()
    }
}

impl Clone for BddContext {
    fn clone(&self) -> Self {
        // Rebuild the context since BddVariableSet is not Clone
        Self::new(self.num_vars)
    }
}

/// Points-to set backed by a BDD (Binary Decision Diagram).
///
/// Uses binary encoding where each location index is represented using
/// `ceil(log2(max_index))` BDD variables. Provides compact representation
/// when many sets share structure.
///
/// Best for:
/// - Large programs with >100K allocation sites
/// - Sets with significant overlap (BDD sharing)
/// - Memory-constrained environments
#[derive(Clone, Debug)]
pub struct BddPtsSet {
    /// The BDD representing the set of indices.
    bdd: Bdd,

    /// Shared BDD context (variable set + encoding).
    context: Arc<RwLock<BddContext>>,

    /// Shared indexer for LocId ↔ index mapping.
    indexer: Arc<RwLock<LocIdIndexer>>,

    /// Cached cardinality for O(1) len() calls.
    cached_len: usize,
}

impl BddPtsSet {
    /// Create a new empty BDD-backed set with default context.
    ///
    /// The default context uses 16 variables (supports up to 65535 indices).
    #[must_use]
    pub fn new() -> Self {
        let context = Arc::new(RwLock::new(BddContext::new(16)));
        let empty_bdd = context.read().expect("context lock poisoned").empty();

        Self {
            bdd: empty_bdd,
            context,
            indexer: Arc::new(RwLock::new(LocIdIndexer::new())),
            cached_len: 0,
        }
    }

    /// Create a new empty set with a shared context and indexer.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn with_context_and_indexer(
        context: Arc<RwLock<BddContext>>,
        indexer: Arc<RwLock<LocIdIndexer>>,
    ) -> Self {
        let empty_bdd = context.read().expect("context lock poisoned").empty();

        Self {
            bdd: empty_bdd,
            context,
            indexer,
            cached_len: 0,
        }
    }

    /// Get the shared context.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn context(&self) -> &Arc<RwLock<BddContext>> {
        &self.context
    }

    /// Get the shared indexer.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn indexer(&self) -> &Arc<RwLock<LocIdIndexer>> {
        &self.indexer
    }

    /// Ensure the context has enough variables to encode the given index.
    /// If not, grows the context (which rebuilds all BDDs).
    fn ensure_context_capacity(&mut self, idx: usize) {
        let needs_growth = {
            let ctx = self.context.read().expect("context lock poisoned");
            !ctx.can_encode(idx)
        };

        if needs_growth {
            // Need to grow the context
            let mut ctx = self.context.write().expect("context lock poisoned");
            let new_num_vars = BddContext::bits_needed(idx);

            if new_num_vars > ctx.num_vars {
                // Decode current indices, rebuild context, re-encode
                let current_indices = ctx.decode_indices(&self.bdd);

                // Create new context with more variables
                *ctx = BddContext::new(new_num_vars);

                // Re-encode all indices
                let mut new_bdd = ctx.empty();
                for i in current_indices {
                    let encoded = ctx.encode_index(i);
                    new_bdd = new_bdd.or(&encoded);
                }
                self.bdd = new_bdd;
            }
        }
    }

    /// Recompute cached length from BDD.
    fn update_cached_len(&mut self) {
        let ctx = self.context.read().expect("context lock poisoned");
        self.cached_len = ctx.count_indices(&self.bdd);
    }
}

impl Default for BddPtsSet {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for BddPtsSet {
    fn eq(&self, other: &Self) -> bool {
        // Compare by contents
        self.to_btreeset() == other.to_btreeset()
    }
}

impl Eq for BddPtsSet {}

impl Hash for BddPtsSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the LocIds in sorted order for deterministic hashing
        for loc in self.iter() {
            loc.hash(state);
        }
    }
}

// SAFETY: All interior mutability is through RwLock which is Send+Sync
unsafe impl Send for BddPtsSet {}
unsafe impl Sync for BddPtsSet {}

impl PtsSet for BddPtsSet {
    const BENEFITS_FROM_CLUSTERING: bool = true;

    fn empty() -> Self {
        Self::new()
    }

    fn singleton(loc: LocId) -> Self {
        let mut set = Self::new();
        set.insert(loc);
        set
    }

    fn insert(&mut self, loc: LocId) -> bool {
        // Get or create index for this location
        let idx = {
            let mut indexer = self.indexer.write().expect("indexer lock poisoned");
            indexer.get_or_insert(loc)
        };

        // Ensure context can encode this index
        self.ensure_context_capacity(idx);

        // Check if already present
        let already_present = {
            let ctx = self.context.read().expect("context lock poisoned");
            ctx.contains_index(&self.bdd, idx)
        };

        if already_present {
            return false;
        }

        // Add the index to the BDD
        let encoded = {
            let ctx = self.context.read().expect("context lock poisoned");
            ctx.encode_index(idx)
        };
        self.bdd = self.bdd.or(&encoded);
        self.cached_len += 1;

        true
    }

    fn remove(&mut self, loc: LocId) -> bool {
        // Get index for this location
        let idx = {
            let indexer = self.indexer.read().expect("indexer lock poisoned");
            match indexer.get(loc) {
                Some(idx) => idx,
                None => return false,
            }
        };

        // Check if present
        let is_present = {
            let ctx = self.context.read().expect("context lock poisoned");
            ctx.contains_index(&self.bdd, idx)
        };

        if !is_present {
            return false;
        }

        // Remove by AND-ing with NOT of the encoded index
        let encoded = {
            let ctx = self.context.read().expect("context lock poisoned");
            ctx.encode_index(idx)
        };
        self.bdd = self.bdd.and(&encoded.not());
        self.cached_len -= 1;

        true
    }

    fn contains(&self, loc: LocId) -> bool {
        // Get index for this location
        let idx = {
            let indexer = self.indexer.read().expect("indexer lock poisoned");
            match indexer.get(loc) {
                Some(idx) => idx,
                None => return false,
            }
        };

        let ctx = self.context.read().expect("context lock poisoned");
        ctx.contains_index(&self.bdd, idx)
    }

    fn len(&self) -> usize {
        self.cached_len
    }

    fn is_empty(&self) -> bool {
        self.cached_len == 0
    }

    fn iter(&self) -> impl Iterator<Item = LocId> {
        // Decode indices from BDD
        let indices = {
            let ctx = self.context.read().expect("context lock poisoned");
            ctx.decode_indices(&self.bdd)
        };

        // Resolve to LocIds
        let indexer = self.indexer.read().expect("indexer lock poisoned");
        let mut locs: Vec<LocId> = indices
            .into_iter()
            .filter_map(|idx| indexer.resolve(idx))
            .collect();

        // Sort by LocId for deterministic iteration
        locs.sort();
        locs.into_iter()
    }

    fn union(&mut self, other: &Self) -> bool {
        let old_len = self.cached_len;

        if Arc::ptr_eq(&self.context, &other.context) {
            // Same context - direct BDD OR
            self.bdd = self.bdd.or(&other.bdd);
        } else {
            // Different contexts - iterate and insert
            for loc in other.iter() {
                self.insert(loc);
            }
        }

        self.update_cached_len();
        self.cached_len > old_len
    }

    fn intersect(&mut self, other: &Self) -> bool {
        let old_len = self.cached_len;

        if Arc::ptr_eq(&self.context, &other.context) {
            // Same context - direct BDD AND
            self.bdd = self.bdd.and(&other.bdd);
        } else {
            // Different contexts - rebuild via set intersection
            let to_keep: BTreeSet<LocId> = self.iter().filter(|loc| other.contains(*loc)).collect();
            *self = Self::from_btreeset(&to_keep);
        }

        self.update_cached_len();
        self.cached_len < old_len
    }

    fn difference(&mut self, other: &Self) -> bool {
        let old_len = self.cached_len;

        if Arc::ptr_eq(&self.context, &other.context) {
            // Same context - BDD AND NOT
            self.bdd = self.bdd.and(&other.bdd.not());
        } else {
            // Different contexts - rebuild via set difference
            let to_keep: BTreeSet<LocId> =
                self.iter().filter(|loc| !other.contains(*loc)).collect();
            *self = Self::from_btreeset(&to_keep);
        }

        self.update_cached_len();
        self.cached_len < old_len
    }

    fn intersects(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.context, &other.context) {
            // Same context - check if AND is not false
            !self.bdd.and(&other.bdd).is_false()
        } else {
            // Different contexts - iterate
            if self.len() <= other.len() {
                self.iter().any(|loc| other.contains(loc))
            } else {
                other.iter().any(|loc| self.contains(loc))
            }
        }
    }

    fn is_subset(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.context, &other.context) {
            // Same context - check if self AND NOT other is false
            self.bdd.and(&other.bdd.not()).is_false()
        } else {
            // Different contexts - iterate
            self.iter().all(|loc| other.contains(loc))
        }
    }

    fn to_btreeset(&self) -> BTreeSet<LocId> {
        self.iter().collect()
    }

    fn from_btreeset(set: &BTreeSet<LocId>) -> Self {
        let mut result = Self::new();
        for &loc in set {
            result.insert(loc);
        }
        result
    }

    fn clone_empty(&self) -> Self {
        Self::with_context_and_indexer(Arc::clone(&self.context), Arc::clone(&self.indexer))
    }

    fn with_seeded_ordering(ordered_locs: &[LocId]) -> Self {
        let num_vars = if ordered_locs.is_empty() {
            16
        } else {
            BddContext::bits_needed(ordered_locs.len()).max(1)
        };
        let context = Arc::new(RwLock::new(BddContext::new(num_vars)));
        let mut indexer = LocIdIndexer::new();
        indexer.register_batch(ordered_locs.iter().copied());
        Self::with_context_and_indexer(context, Arc::new(RwLock::new(indexer)))
    }
}

impl From<BTreeSet<LocId>> for BddPtsSet {
    fn from(set: BTreeSet<LocId>) -> Self {
        Self::from_btreeset(&set)
    }
}

impl FromIterator<LocId> for BddPtsSet {
    fn from_iter<T: IntoIterator<Item = LocId>>(iter: T) -> Self {
        let mut result = Self::new();
        for loc in iter {
            result.insert(loc);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_bits_needed() {
        assert_eq!(BddContext::bits_needed(0), 1);
        assert_eq!(BddContext::bits_needed(1), 1);
        assert_eq!(BddContext::bits_needed(2), 2);
        assert_eq!(BddContext::bits_needed(3), 2);
        assert_eq!(BddContext::bits_needed(4), 3);
        assert_eq!(BddContext::bits_needed(7), 3);
        assert_eq!(BddContext::bits_needed(8), 4);
        assert_eq!(BddContext::bits_needed(255), 8);
        assert_eq!(BddContext::bits_needed(256), 9);
    }

    #[test]
    fn context_encode_decode_single() {
        let ctx = BddContext::new(8);

        for idx in 0..=255 {
            let bdd = ctx.encode_index(idx);
            let decoded = ctx.decode_indices(&bdd);
            assert_eq!(decoded, vec![idx], "Encode/decode mismatch for {}", idx);
        }
    }

    #[test]
    fn context_encode_decode_multiple() {
        let ctx = BddContext::new(8);

        let indices = vec![1, 3, 7, 15, 100];
        let mut bdd = ctx.empty();
        for &idx in &indices {
            bdd = bdd.or(&ctx.encode_index(idx));
        }

        let decoded = ctx.decode_indices(&bdd);
        assert_eq!(decoded, indices);
    }

    #[test]
    fn context_contains_index() {
        let ctx = BddContext::new(8);

        let mut bdd = ctx.empty();
        bdd = bdd.or(&ctx.encode_index(5));
        bdd = bdd.or(&ctx.encode_index(10));

        assert!(ctx.contains_index(&bdd, 5));
        assert!(ctx.contains_index(&bdd, 10));
        assert!(!ctx.contains_index(&bdd, 3));
        assert!(!ctx.contains_index(&bdd, 7));
    }

    #[test]
    fn to_and_from_btreeset() {
        let mut pts = BddPtsSet::empty();
        pts.insert(LocId::new(1));
        pts.insert(LocId::new(2));

        let btree = pts.to_btreeset();
        assert_eq!(btree.len(), 2);

        let pts2 = BddPtsSet::from_btreeset(&btree);
        assert_eq!(pts.len(), pts2.len());
        assert!(pts2.contains(LocId::new(1)));
        assert!(pts2.contains(LocId::new(2)));
    }

    #[test]
    fn dynamic_growth() {
        // Start with small context and grow beyond it
        let context = Arc::new(RwLock::new(BddContext::new(4))); // Only supports 0-15
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut pts = BddPtsSet::with_context_and_indexer(context, indexer);

        // Insert values that require growth
        pts.insert(LocId::new(1));
        pts.insert(LocId::new(100)); // Requires > 4 bits

        assert!(pts.contains(LocId::new(1)));
        assert!(pts.contains(LocId::new(100)));
        assert_eq!(pts.len(), 2);
    }

    #[test]
    fn large_set() {
        let mut pts = BddPtsSet::empty();
        for i in 0..100 {
            pts.insert(LocId::new(i * 3)); // Non-contiguous IDs
        }

        assert_eq!(pts.len(), 100);

        // Verify all are present
        for i in 0..100 {
            assert!(pts.contains(LocId::new(i * 3)));
            assert!(!pts.contains(LocId::new(i * 3 + 1)));
        }
    }

    #[test]
    fn iter_empty_set() {
        let pts = BddPtsSet::empty();
        let collected: Vec<_> = pts.iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn clone_empty_shares_context_and_indexer() {
        let mut pts = BddPtsSet::empty();
        pts.insert(LocId::new(1));
        let empty = pts.clone_empty();
        assert!(empty.is_empty());
        assert!(Arc::ptr_eq(pts.context(), empty.context()));
        assert!(Arc::ptr_eq(pts.indexer(), empty.indexer()));
    }
}
