//! Abstract state — maps `ValueId` to `Interval`.
//!
//! Each SSA value maps to an abstract element representing its possible
//! range of concrete values at a given program point.
//!
//! # Memory Tracking
//!
//! The state also tracks values stored at known memory locations (allocas).
//! This enables tracking values through store/load sequences in simple cases.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use saf_core::ids::{LocId, ValueId};

use super::domain::AbstractDomain;
use super::interval::Interval;

/// Abstract state at a program point.
///
/// Maps each `ValueId` to an `Interval` representing the value's range.
/// Values not in the map are implicitly top (unknown).
///
/// Also tracks memory contents in three ways:
/// 1. `memory` - Simple `ValueId`-based tracking for backward compatibility
/// 2. `loc_memory` - `LocId`-based tracking for PTA-integrated analysis
/// 3. `field_memory` - Field-sensitive `(LocId, byte_offset)` tracking for struct fields
///
/// Additionally tracks GEP target locations to bridge PTA location tracking
/// with interval propagation (Plan 062).
#[derive(Clone, PartialEq, Eq)]
pub struct AbstractState {
    /// Map from value to its abstract interval.
    values: BTreeMap<ValueId, Interval>,
    /// Legacy: Map from memory location (pointer `ValueId`) to stored interval.
    ///
    /// This pre-PTA memory model uses raw `ValueId` keys with no alias analysis.
    /// After mem2reg, stores to promoted locals are gone, making this field
    /// empty for most functions. The PTA-aware `loc_memory` field handles
    /// address-taken locals. Retained because non-PTA checker entry points
    /// (`solve_abstract_interp`) still use `state.store()`/`state.load()`.
    memory: BTreeMap<ValueId, Interval>,
    /// Map from abstract location (`LocId`) to stored interval.
    /// Used when PTA is available for alias-aware memory tracking.
    loc_memory: BTreeMap<LocId, Interval>,
    /// Map from GEP result `ValueId` to target locations.
    /// Used to resolve field-sensitive memory operations through GEP pointers.
    gep_targets: BTreeMap<ValueId, BTreeSet<LocId>>,
    /// Field-sensitive memory: maps (`LocId`, byte_offset) to stored interval.
    /// Tracks individual struct field values through GEP + store/load sequences.
    field_memory: BTreeMap<(LocId, u64), Interval>,
    /// Field-sensitive GEP targets: maps GEP result `ValueId` to
    /// `(LocId, byte_offset)` pairs identifying specific struct fields.
    field_gep_targets: BTreeMap<ValueId, BTreeSet<(LocId, u64)>>,
    /// Whether this state represents unreachable code (bottom state).
    unreachable: bool,
}

impl fmt::Debug for AbstractState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.unreachable {
            write!(f, "⊥-state")
        } else {
            write!(
                f,
                "State({} values, {} mem)",
                self.values.len(),
                self.memory.len()
            )
        }
    }
}

impl AbstractState {
    /// Create a new empty state (all values implicitly top).
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            memory: BTreeMap::new(),
            loc_memory: BTreeMap::new(),
            gep_targets: BTreeMap::new(),
            field_memory: BTreeMap::new(),
            field_gep_targets: BTreeMap::new(),
            unreachable: false,
        }
    }

    /// Create an unreachable (bottom) state.
    #[must_use]
    pub fn bottom() -> Self {
        Self {
            values: BTreeMap::new(),
            memory: BTreeMap::new(),
            loc_memory: BTreeMap::new(),
            gep_targets: BTreeMap::new(),
            field_memory: BTreeMap::new(),
            field_gep_targets: BTreeMap::new(),
            unreachable: true,
        }
    }

    /// Check if this state is unreachable.
    #[must_use]
    pub fn is_unreachable(&self) -> bool {
        self.unreachable
    }

    /// Get the interval for a value, or top if not present.
    #[must_use]
    pub fn get(&self, value: ValueId, bits: u8) -> Interval {
        if self.unreachable {
            return Interval::make_bottom(bits);
        }
        self.values
            .get(&value)
            .cloned()
            .unwrap_or_else(|| Interval::make_top(bits))
    }

    /// Get the interval for a value, returning `None` if not tracked.
    #[must_use]
    pub fn get_opt(&self, value: ValueId) -> Option<&Interval> {
        self.values.get(&value)
    }

    /// Set the interval for a value.
    pub fn set(&mut self, value: ValueId, interval: Interval) {
        if !self.unreachable {
            self.values.insert(value, interval);
        }
    }

    /// Store an interval to a memory location.
    ///
    /// Records that the memory pointed to by `ptr` contains `interval`.
    pub fn store(&mut self, ptr: ValueId, interval: Interval) {
        if !self.unreachable {
            self.memory.insert(ptr, interval);
        }
    }

    /// Load an interval from a memory location.
    ///
    /// Returns the stored interval if known, `None` otherwise.
    #[must_use]
    pub fn load(&self, ptr: ValueId) -> Option<&Interval> {
        if self.unreachable {
            return None;
        }
        self.memory.get(&ptr)
    }

    /// Clear memory for a location (e.g., when it may be modified indirectly).
    pub fn invalidate_memory(&mut self, ptr: ValueId) {
        self.memory.remove(&ptr);
    }

    /// Clear all memory (conservative: after function call or unknown store).
    pub fn invalidate_all_memory(&mut self) {
        self.memory.clear();
    }

    // =========================================================================
    // LocId-based memory operations (for PTA-integrated analysis)
    // =========================================================================

    /// Store an interval to an abstract memory location (strong update).
    ///
    /// Replaces any existing value at the location.
    pub fn store_loc(&mut self, loc: LocId, interval: Interval) {
        if !self.unreachable {
            self.loc_memory.insert(loc, interval);
        }
    }

    /// Store an interval to an abstract memory location (weak update).
    ///
    /// Joins with any existing value at the location.
    #[allow(clippy::needless_pass_by_value)] // Interval is small and consumed by join
    pub fn store_loc_weak(&mut self, loc: LocId, interval: Interval) {
        if !self.unreachable {
            let existing = self
                .loc_memory
                .get(&loc)
                .cloned()
                .unwrap_or_else(|| Interval::make_bottom(interval.bits()));
            self.loc_memory.insert(loc, existing.join(&interval));
        }
    }

    /// Load an interval from an abstract memory location.
    ///
    /// Returns the stored interval if known, `None` otherwise.
    #[must_use]
    pub fn load_loc(&self, loc: LocId) -> Option<&Interval> {
        if self.unreachable {
            return None;
        }
        self.loc_memory.get(&loc)
    }

    /// Invalidate a specific abstract memory location.
    pub fn invalidate_loc(&mut self, loc: LocId) {
        self.loc_memory.remove(&loc);
    }

    /// Invalidate a set of abstract memory locations.
    pub fn invalidate_locs(&mut self, locs: &BTreeSet<LocId>) {
        for loc in locs {
            self.loc_memory.remove(loc);
        }
    }

    /// Invalidate all abstract memory locations.
    pub fn invalidate_all_loc_memory(&mut self) {
        self.loc_memory.clear();
    }

    /// Get all tracked `LocId` memory entries.
    #[must_use]
    pub fn loc_memory_entries(&self) -> &BTreeMap<LocId, Interval> {
        &self.loc_memory
    }

    /// Get all tracked `ValueId`-based memory entries.
    #[must_use]
    pub fn memory_entries(&self) -> &BTreeMap<ValueId, Interval> {
        &self.memory
    }

    // =========================================================================
    // GEP target tracking (for location-aware interval domain)
    // =========================================================================

    /// Register a GEP result pointing to specific locations.
    ///
    /// This records the mapping from a GEP result pointer to the abstract
    /// locations it points to, enabling field-sensitive load/store operations.
    pub fn register_gep(&mut self, ptr: ValueId, targets: BTreeSet<LocId>) {
        if !self.unreachable && !targets.is_empty() {
            self.gep_targets.insert(ptr, targets);
        }
    }

    /// Resolve a pointer to its GEP targets (if any).
    ///
    /// Returns the set of locations this pointer was computed to point to
    /// via a GEP operation, or `None` if this pointer wasn't a GEP result.
    #[must_use]
    pub fn resolve_gep(&self, ptr: ValueId) -> Option<&BTreeSet<LocId>> {
        if self.unreachable {
            return None;
        }
        self.gep_targets.get(&ptr)
    }

    /// Get all tracked GEP target mappings.
    #[must_use]
    pub fn gep_targets(&self) -> &BTreeMap<ValueId, BTreeSet<LocId>> {
        &self.gep_targets
    }

    /// Invalidate GEP targets for a specific pointer.
    pub fn invalidate_gep(&mut self, ptr: ValueId) {
        self.gep_targets.remove(&ptr);
    }

    /// Invalidate all GEP target mappings.
    pub fn invalidate_all_gep_targets(&mut self) {
        self.gep_targets.clear();
    }

    // =========================================================================
    // Field-sensitive memory operations (struct field tracking)
    // =========================================================================

    /// Register a field-sensitive GEP result pointing to specific (`LocId`, byte_offset) pairs.
    ///
    /// Records the mapping from a GEP result pointer to the struct fields it
    /// addresses, enabling field-sensitive store/load operations.
    pub fn register_field_gep(&mut self, ptr: ValueId, targets: BTreeSet<(LocId, u64)>) {
        if !self.unreachable && !targets.is_empty() {
            self.field_gep_targets.insert(ptr, targets);
        }
    }

    /// Resolve a pointer to its field-sensitive GEP targets (if any).
    ///
    /// Returns the set of (`LocId`, byte_offset) pairs this pointer was computed
    /// to address via a struct-field GEP, or `None` if not a field GEP result.
    #[must_use]
    pub fn resolve_field_gep(&self, ptr: ValueId) -> Option<&BTreeSet<(LocId, u64)>> {
        if self.unreachable {
            return None;
        }
        self.field_gep_targets.get(&ptr)
    }

    /// Store an interval to a struct field (strong update).
    ///
    /// Replaces any existing value at the (`LocId`, byte_offset) location.
    pub fn store_field(&mut self, loc: LocId, offset: u64, interval: Interval) {
        if !self.unreachable {
            self.field_memory.insert((loc, offset), interval);
        }
    }

    /// Store an interval to a struct field (weak update).
    ///
    /// Joins with any existing value at the (`LocId`, byte_offset) location.
    #[allow(clippy::needless_pass_by_value)] // Interval is small and consumed by join
    pub fn store_field_weak(&mut self, loc: LocId, offset: u64, interval: Interval) {
        if !self.unreachable {
            let existing = self
                .field_memory
                .get(&(loc, offset))
                .cloned()
                .unwrap_or_else(|| Interval::make_bottom(interval.bits()));
            self.field_memory
                .insert((loc, offset), existing.join(&interval));
        }
    }

    /// Load an interval from a struct field.
    ///
    /// Returns the stored interval if known, `None` otherwise.
    #[must_use]
    pub fn load_field(&self, loc: LocId, offset: u64) -> Option<&Interval> {
        if self.unreachable {
            return None;
        }
        self.field_memory.get(&(loc, offset))
    }

    /// Get all tracked field memory entries.
    #[must_use]
    pub fn field_memory_entries(&self) -> &BTreeMap<(LocId, u64), Interval> {
        &self.field_memory
    }

    /// Get all tracked field GEP target mappings.
    #[must_use]
    pub fn field_gep_targets(&self) -> &BTreeMap<ValueId, BTreeSet<(LocId, u64)>> {
        &self.field_gep_targets
    }

    /// Invalidate a specific struct field.
    pub fn invalidate_field(&mut self, loc: LocId, offset: u64) {
        self.field_memory.remove(&(loc, offset));
    }

    /// Invalidate all field memory.
    pub fn invalidate_all_field_memory(&mut self) {
        self.field_memory.clear();
    }

    /// Invalidate all field GEP target mappings.
    pub fn invalidate_all_field_gep_targets(&mut self) {
        self.field_gep_targets.clear();
    }

    /// Join two states (point-wise join of intervals).
    // NOTE: Field-sensitive memory tracking adds necessary join logic for field_memory
    // and field_gep_targets that pushes this over the line limit.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        if self.unreachable {
            return other.clone();
        }
        if other.unreachable {
            return self.clone();
        }

        let mut result_values = BTreeMap::new();
        let mut result_memory = BTreeMap::new();

        // Join values present in both or either state
        let mut all_value_keys: BTreeSet<ValueId> = BTreeSet::new();
        for key in self.values.keys() {
            all_value_keys.insert(*key);
        }
        for key in other.values.keys() {
            all_value_keys.insert(*key);
        }

        for key in &all_value_keys {
            let a = self.values.get(key);
            let b = other.values.get(key);

            // Only join when both sides have concrete values; otherwise result is top (not stored)
            // This ensures convergence in loops while still being sound for SSA form.
            if let (Some(a_val), Some(b_val)) = (a, b) {
                let joined = a_val.join(b_val);
                // Only store if not top (to save memory)
                if !joined.is_top() {
                    result_values.insert(*key, joined);
                }
            }
        }

        // Join memory state: only keep entries present in both with joined intervals
        let mut all_mem_keys: BTreeSet<ValueId> = BTreeSet::new();
        for key in self.memory.keys() {
            all_mem_keys.insert(*key);
        }
        for key in other.memory.keys() {
            all_mem_keys.insert(*key);
        }

        for key in &all_mem_keys {
            let a = self.memory.get(key);
            let b = other.memory.get(key);

            // Only keep memory entries present in BOTH states with joined intervals
            // One-sided entries are treated as TOP (unknown memory content)
            if let (Some(a_val), Some(b_val)) = (a, b) {
                let joined = a_val.join(b_val);
                if !joined.is_top() {
                    result_memory.insert(*key, joined);
                }
            }
        }

        // Join loc_memory state: only keep entries present in both with joined intervals
        let mut all_loc_keys: BTreeSet<LocId> = BTreeSet::new();
        for key in self.loc_memory.keys() {
            all_loc_keys.insert(*key);
        }
        for key in other.loc_memory.keys() {
            all_loc_keys.insert(*key);
        }

        let mut result_loc_memory = BTreeMap::new();
        for key in &all_loc_keys {
            let a = self.loc_memory.get(key);
            let b = other.loc_memory.get(key);

            // Only keep loc_memory entries present in BOTH states with joined intervals
            // One-sided entries are treated as TOP (unknown memory content)
            if let (Some(a_val), Some(b_val)) = (a, b) {
                let joined = a_val.join(b_val);
                if !joined.is_top() {
                    result_loc_memory.insert(*key, joined);
                }
            }
        }

        // Join GEP targets: keep mappings present in both with union of targets
        let mut all_gep_keys: BTreeSet<ValueId> = BTreeSet::new();
        for key in self.gep_targets.keys() {
            all_gep_keys.insert(*key);
        }
        for key in other.gep_targets.keys() {
            all_gep_keys.insert(*key);
        }

        let mut result_gep_targets = BTreeMap::new();
        for key in &all_gep_keys {
            let a = self.gep_targets.get(key);
            let b = other.gep_targets.get(key);

            match (a, b) {
                (Some(a_set), Some(b_set)) => {
                    // Both have targets - union them
                    let mut joined = a_set.clone();
                    joined.extend(b_set.iter().copied());
                    result_gep_targets.insert(*key, joined);
                }
                (Some(a_set), None) | (None, Some(a_set)) => {
                    // Only one side has the GEP target - keep it conservatively
                    // This helps propagate GEP info through control flow
                    result_gep_targets.insert(*key, a_set.clone());
                }
                (None, None) => {}
            }
        }

        // Join field_memory: only keep entries present in both with joined intervals
        let mut result_field_memory = BTreeMap::new();
        for (key, a_val) in &self.field_memory {
            if let Some(b_val) = other.field_memory.get(key) {
                let joined = a_val.join(b_val);
                if !joined.is_top() {
                    result_field_memory.insert(*key, joined);
                }
            }
        }

        // Join field_gep_targets: keep mappings present in either with union of targets
        let mut result_field_gep_targets: BTreeMap<ValueId, BTreeSet<(LocId, u64)>> =
            BTreeMap::new();
        for (key, a_set) in &self.field_gep_targets {
            let entry = result_field_gep_targets.entry(*key).or_default();
            entry.extend(a_set.iter().copied());
        }
        for (key, b_set) in &other.field_gep_targets {
            let entry = result_field_gep_targets.entry(*key).or_default();
            entry.extend(b_set.iter().copied());
        }

        Self {
            values: result_values,
            memory: result_memory,
            loc_memory: result_loc_memory,
            gep_targets: result_gep_targets,
            field_memory: result_field_memory,
            field_gep_targets: result_field_gep_targets,
            unreachable: false,
        }
    }

    /// Join two states, preserving one-sided entries.
    ///
    /// Unlike `join()` which drops entries present in only one state (treating
    /// them as TOP), this method preserves one-sided entries. This is useful
    /// at non-loop-header merge points where we want to preserve precision
    /// from branches that define values the other branch doesn't touch.
    ///
    /// For example, in:
    /// ```text
    /// if (cond) {
    ///     store 5 to *p  // memory[p] = [5,5]
    /// } else {
    ///     // no store to *p
    /// }
    /// // merge point - preserving join keeps memory[p] = [5,5]
    /// ```
    ///
    /// **IMPORTANT**: Only use this at non-loop-header merge points. Using it
    /// at loop headers would break convergence as new entries could accumulate.
    // NOTE: One-sided join merges intervals, memory, nullness, and escape
    // state in a single pass. Splitting would break the coordinated state update.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn join_preserving_one_sided(&self, other: &Self) -> Self {
        if self.unreachable {
            return other.clone();
        }
        if other.unreachable {
            return self.clone();
        }

        let mut result_values = BTreeMap::new();
        let mut result_memory = BTreeMap::new();

        // Join values, preserving one-sided entries
        let mut all_value_keys: BTreeSet<ValueId> = BTreeSet::new();
        for key in self.values.keys() {
            all_value_keys.insert(*key);
        }
        for key in other.values.keys() {
            all_value_keys.insert(*key);
        }

        for key in &all_value_keys {
            let a = self.values.get(key);
            let b = other.values.get(key);

            match (a, b) {
                (Some(a_val), Some(b_val)) => {
                    let joined = a_val.join(b_val);
                    if !joined.is_top() {
                        result_values.insert(*key, joined);
                    }
                }
                (Some(val), None) | (None, Some(val)) => {
                    // Preserve one-sided value entries
                    if !val.is_top() {
                        result_values.insert(*key, val.clone());
                    }
                }
                (None, None) => {}
            }
        }

        // Join memory, preserving one-sided entries
        let mut all_mem_keys: BTreeSet<ValueId> = BTreeSet::new();
        for key in self.memory.keys() {
            all_mem_keys.insert(*key);
        }
        for key in other.memory.keys() {
            all_mem_keys.insert(*key);
        }

        for key in &all_mem_keys {
            let a = self.memory.get(key);
            let b = other.memory.get(key);

            match (a, b) {
                (Some(a_val), Some(b_val)) => {
                    let joined = a_val.join(b_val);
                    if !joined.is_top() {
                        result_memory.insert(*key, joined);
                    }
                }
                (Some(val), None) | (None, Some(val)) => {
                    // Preserve one-sided memory entries
                    if !val.is_top() {
                        result_memory.insert(*key, val.clone());
                    }
                }
                (None, None) => {}
            }
        }

        // Join loc_memory, preserving one-sided entries
        let mut all_loc_keys: BTreeSet<LocId> = BTreeSet::new();
        for key in self.loc_memory.keys() {
            all_loc_keys.insert(*key);
        }
        for key in other.loc_memory.keys() {
            all_loc_keys.insert(*key);
        }

        let mut result_loc_memory = BTreeMap::new();
        for key in &all_loc_keys {
            let a = self.loc_memory.get(key);
            let b = other.loc_memory.get(key);

            match (a, b) {
                (Some(a_val), Some(b_val)) => {
                    let joined = a_val.join(b_val);
                    if !joined.is_top() {
                        result_loc_memory.insert(*key, joined);
                    }
                }
                (Some(val), None) | (None, Some(val)) => {
                    // Preserve one-sided loc_memory entries
                    if !val.is_top() {
                        result_loc_memory.insert(*key, val.clone());
                    }
                }
                (None, None) => {}
            }
        }

        // Join GEP targets: keep mappings present in either with union of targets
        // (same as regular join since GEP targets already preserve one-sided)
        let mut all_gep_keys: BTreeSet<ValueId> = BTreeSet::new();
        for key in self.gep_targets.keys() {
            all_gep_keys.insert(*key);
        }
        for key in other.gep_targets.keys() {
            all_gep_keys.insert(*key);
        }

        let mut result_gep_targets = BTreeMap::new();
        for key in &all_gep_keys {
            let a = self.gep_targets.get(key);
            let b = other.gep_targets.get(key);

            match (a, b) {
                (Some(a_set), Some(b_set)) => {
                    let mut joined = a_set.clone();
                    joined.extend(b_set.iter().copied());
                    result_gep_targets.insert(*key, joined);
                }
                (Some(a_set), None) | (None, Some(a_set)) => {
                    result_gep_targets.insert(*key, a_set.clone());
                }
                (None, None) => {}
            }
        }

        // Join field_memory, preserving one-sided entries
        let mut all_field_keys: BTreeSet<(LocId, u64)> = BTreeSet::new();
        for key in self.field_memory.keys() {
            all_field_keys.insert(*key);
        }
        for key in other.field_memory.keys() {
            all_field_keys.insert(*key);
        }

        let mut result_field_memory = BTreeMap::new();
        for key in &all_field_keys {
            let a = self.field_memory.get(key);
            let b = other.field_memory.get(key);

            match (a, b) {
                (Some(a_val), Some(b_val)) => {
                    let joined = a_val.join(b_val);
                    if !joined.is_top() {
                        result_field_memory.insert(*key, joined);
                    }
                }
                (Some(val), None) | (None, Some(val)) => {
                    // Preserve one-sided field memory entries
                    if !val.is_top() {
                        result_field_memory.insert(*key, val.clone());
                    }
                }
                (None, None) => {}
            }
        }

        // Join field_gep_targets: keep mappings present in either with union of targets
        let mut result_field_gep_targets: BTreeMap<ValueId, BTreeSet<(LocId, u64)>> =
            BTreeMap::new();
        for (key, a_set) in &self.field_gep_targets {
            let entry = result_field_gep_targets.entry(*key).or_default();
            entry.extend(a_set.iter().copied());
        }
        for (key, b_set) in &other.field_gep_targets {
            let entry = result_field_gep_targets.entry(*key).or_default();
            entry.extend(b_set.iter().copied());
        }

        Self {
            values: result_values,
            memory: result_memory,
            loc_memory: result_loc_memory,
            gep_targets: result_gep_targets,
            field_memory: result_field_memory,
            field_gep_targets: result_field_gep_targets,
            unreachable: false,
        }
    }

    /// Check if this state is less-or-equal to another (point-wise).
    #[must_use]
    pub fn leq(&self, other: &Self) -> bool {
        if self.unreachable {
            return true;
        }
        if other.unreachable {
            return false;
        }

        // Check value intervals
        for (key, val) in &self.values {
            if let Some(other_val) = other.values.get(key) {
                if !val.leq(other_val) {
                    return false;
                }
            }
            // If key not in other, other has top for it → val ⊑ top always
        }

        // Check memory intervals
        for (key, val) in &self.memory {
            if let Some(other_val) = other.memory.get(key) {
                if !val.leq(other_val) {
                    return false;
                }
            }
            // If key not in other, other has top for it → val ⊑ top always
        }

        // Check loc_memory intervals
        for (key, val) in &self.loc_memory {
            if let Some(other_val) = other.loc_memory.get(key) {
                if !val.leq(other_val) {
                    return false;
                }
            }
            // If key not in other, other has top for it → val ⊑ top always
        }

        // Check field_memory intervals
        for (key, val) in &self.field_memory {
            if let Some(other_val) = other.field_memory.get(key) {
                if !val.leq(other_val) {
                    return false;
                }
            }
            // If key not in other, other has top for it → val ⊑ top always
        }

        true
    }

    /// Get all tracked value-interval pairs.
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<ValueId, Interval> {
        &self.values
    }

    /// Number of tracked values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if no values are tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl Default for AbstractState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }

    #[test]
    fn empty_state_returns_top() {
        let state = AbstractState::new();
        let val = state.get(vid(1), 32);
        assert!(val.is_top());
    }

    #[test]
    fn set_and_get() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 10, 32));
        let val = state.get(vid(1), 32);
        assert_eq!(val.lo(), 0);
        assert_eq!(val.hi(), 10);
    }

    #[test]
    fn bottom_state() {
        let state = AbstractState::bottom();
        assert!(state.is_unreachable());
        let val = state.get(vid(1), 32);
        assert!(val.is_bottom());
    }

    #[test]
    fn join_with_bottom() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 10, 32));
        let bottom = AbstractState::bottom();
        assert_eq!(state.join(&bottom), state);
        assert_eq!(bottom.join(&state), state);
    }

    #[test]
    fn join_pointwise() {
        let mut a = AbstractState::new();
        a.set(vid(1), Interval::new(0, 5, 32));
        a.set(vid(2), Interval::new(10, 20, 32));

        let mut b = AbstractState::new();
        b.set(vid(1), Interval::new(3, 8, 32));
        b.set(vid(3), Interval::new(100, 200, 32));

        let j = a.join(&b);

        // vid(1): join of [0,5] and [3,8] = [0,8]
        let v1 = j.get(vid(1), 32);
        assert_eq!(v1.lo(), 0);
        assert_eq!(v1.hi(), 8);

        // vid(2): only in a, not in b → b has top → join = top → not stored
        assert!(j.get_opt(vid(2)).is_none());

        // vid(3): only in b, not in a → a has top → join = top → not stored
        assert!(j.get_opt(vid(3)).is_none());
    }

    #[test]
    fn leq_states() {
        let mut a = AbstractState::new();
        a.set(vid(1), Interval::new(3, 7, 32));

        let mut b = AbstractState::new();
        b.set(vid(1), Interval::new(0, 10, 32));

        assert!(a.leq(&b));
        assert!(!b.leq(&a));
    }

    #[test]
    fn leq_bottom_leq_anything() {
        let bottom = AbstractState::bottom();
        let state = AbstractState::new();
        assert!(bottom.leq(&state));
    }

    // =========================================================================
    // LocId-based memory tests
    // =========================================================================

    fn lid(n: u128) -> LocId {
        LocId::new(n)
    }

    #[test]
    fn store_load_with_locid() {
        let mut state = AbstractState::new();
        let loc = lid(100);

        state.store_loc(loc, Interval::singleton(42, 32));
        let loaded = state.load_loc(loc);

        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().lo(), 42);
        assert_eq!(loaded.unwrap().hi(), 42);
    }

    #[test]
    fn store_weak_update_joins() {
        let mut state = AbstractState::new();
        let loc = lid(100);

        state.store_loc(loc, Interval::new(0, 10, 32));
        state.store_loc_weak(loc, Interval::new(20, 30, 32));

        let loaded = state.load_loc(loc).unwrap();
        assert_eq!(loaded.lo(), 0);
        assert_eq!(loaded.hi(), 30);
    }

    #[test]
    fn join_locid_memory() {
        let mut a = AbstractState::new();
        a.store_loc(lid(100), Interval::new(0, 10, 32));

        let mut b = AbstractState::new();
        b.store_loc(lid(100), Interval::new(20, 30, 32));

        let joined = a.join(&b);
        let loaded = joined.load_loc(lid(100)).unwrap();

        assert_eq!(loaded.lo(), 0);
        assert_eq!(loaded.hi(), 30);
    }

    #[test]
    fn invalidate_loc() {
        let mut state = AbstractState::new();
        state.store_loc(lid(100), Interval::singleton(42, 32));
        assert!(state.load_loc(lid(100)).is_some());

        state.invalidate_loc(lid(100));
        assert!(state.load_loc(lid(100)).is_none());
    }

    #[test]
    fn invalidate_all_loc_memory() {
        let mut state = AbstractState::new();
        state.store_loc(lid(100), Interval::singleton(1, 32));
        state.store_loc(lid(101), Interval::singleton(2, 32));
        assert_eq!(state.loc_memory_entries().len(), 2);

        state.invalidate_all_loc_memory();
        assert!(state.loc_memory_entries().is_empty());
    }

    #[test]
    fn leq_loc_memory() {
        let mut a = AbstractState::new();
        a.store_loc(lid(100), Interval::new(3, 7, 32));

        let mut b = AbstractState::new();
        b.store_loc(lid(100), Interval::new(0, 10, 32));

        assert!(a.leq(&b));
        assert!(!b.leq(&a));
    }

    #[test]
    fn unreachable_state_ignores_loc_operations() {
        let mut state = AbstractState::bottom();
        state.store_loc(lid(100), Interval::singleton(42, 32));
        assert!(state.load_loc(lid(100)).is_none());
    }

    // =========================================================================
    // GEP target tracking tests
    // =========================================================================

    #[test]
    fn register_and_resolve_gep_targets() {
        let mut state = AbstractState::new();
        let ptr = vid(200);
        let mut targets = BTreeSet::new();
        targets.insert(lid(100));
        targets.insert(lid(101));

        state.register_gep(ptr, targets.clone());

        let resolved = state.resolve_gep(ptr);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap(), &targets);
    }

    #[test]
    fn resolve_gep_returns_none_for_unknown() {
        let state = AbstractState::new();
        assert!(state.resolve_gep(vid(999)).is_none());
    }

    #[test]
    fn unreachable_state_ignores_gep_registration() {
        let mut state = AbstractState::bottom();
        let mut targets = BTreeSet::new();
        targets.insert(lid(100));

        state.register_gep(vid(1), targets);
        assert!(state.resolve_gep(vid(1)).is_none());
    }

    #[test]
    fn join_gep_targets_union() {
        let mut a = AbstractState::new();
        let mut b = AbstractState::new();

        let mut targets_a = BTreeSet::new();
        targets_a.insert(lid(100));
        a.register_gep(vid(1), targets_a);

        let mut targets_b = BTreeSet::new();
        targets_b.insert(lid(101));
        b.register_gep(vid(1), targets_b);

        let joined = a.join(&b);
        let resolved = joined.resolve_gep(vid(1)).unwrap();

        // Should have both targets
        assert!(resolved.contains(&lid(100)));
        assert!(resolved.contains(&lid(101)));
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn join_gep_targets_keeps_one_sided() {
        let mut a = AbstractState::new();
        let b = AbstractState::new();

        let mut targets = BTreeSet::new();
        targets.insert(lid(100));
        a.register_gep(vid(1), targets.clone());

        let joined = a.join(&b);
        let resolved = joined.resolve_gep(vid(1));

        // Should keep the GEP target from a
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap(), &targets);
    }

    #[test]
    fn invalidate_gep_targets() {
        let mut state = AbstractState::new();
        let mut targets = BTreeSet::new();
        targets.insert(lid(100));
        state.register_gep(vid(1), targets);

        assert!(state.resolve_gep(vid(1)).is_some());
        state.invalidate_gep(vid(1));
        assert!(state.resolve_gep(vid(1)).is_none());
    }

    #[test]
    fn invalidate_all_gep_targets() {
        let mut state = AbstractState::new();
        let mut targets1 = BTreeSet::new();
        targets1.insert(lid(100));
        let mut targets2 = BTreeSet::new();
        targets2.insert(lid(200));

        state.register_gep(vid(1), targets1);
        state.register_gep(vid(2), targets2);

        assert_eq!(state.gep_targets().len(), 2);
        state.invalidate_all_gep_targets();
        assert!(state.gep_targets().is_empty());
    }
}
