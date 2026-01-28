//! Abstract memory locations for points-to analysis.
//!
//! A location represents an abstract memory region, consisting of an allocation
//! object and an optional field path for field-sensitive analysis.

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;
use saf_core::air::Constant;
use saf_core::ids::{LocId, ObjId, ValueId};
use serde::{Deserialize, Serialize};

use super::config::{FieldSensitivity, IndexSensitivity};

// =============================================================================
// Index expressions for array index sensitivity
// =============================================================================

/// Expression representing an array index in a field path.
///
/// Supports three levels of precision for array index tracking:
/// - `Unknown`: All indices collapse (legacy behavior, backward compatible)
/// - `Constant`: Distinguish constant indices (`a[0]` vs `a[1]`)
/// - `Symbolic`: Track symbolic indices (`a[i]` where `i` is an SSA value)
///
/// For `Symbolic` indices, Z3 can be used at query time to determine
/// if two symbolic indices may or must be equal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum IndexExpr {
    /// Unknown/collapsed index (all array elements treated as one).
    ///
    /// This is the default for backward compatibility with existing code
    /// that doesn't track array indices.
    Unknown,

    /// Known constant index value.
    ///
    /// Enables distinguishing `a[0]` from `a[1]` at analysis time.
    Constant(i64),

    /// Symbolic index (SSA value).
    ///
    /// The `ValueId` refers to the SSA value used as the index.
    /// At query time, Z3 can check if two symbolic indices may be equal.
    Symbolic(ValueId),
}

impl IndexExpr {
    /// Check if this is an unknown (collapsed) index.
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Check if this is a constant index.
    #[must_use]
    pub const fn is_constant(&self) -> bool {
        matches!(self, Self::Constant(_))
    }

    /// Check if this is a symbolic index.
    #[must_use]
    pub const fn is_symbolic(&self) -> bool {
        matches!(self, Self::Symbolic(_))
    }

    /// Get the constant value if this is a constant index.
    #[must_use]
    pub const fn as_constant(&self) -> Option<i64> {
        match self {
            Self::Constant(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the symbolic value ID if this is a symbolic index.
    #[must_use]
    pub const fn as_symbolic(&self) -> Option<ValueId> {
        match self {
            Self::Symbolic(v) => Some(*v),
            _ => None,
        }
    }
}

impl Default for IndexExpr {
    fn default() -> Self {
        Self::Unknown
    }
}

// =============================================================================
// Field paths
// =============================================================================

/// A single step in a field path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PathStep {
    /// Array/pointer index with optional expression tracking.
    ///
    /// The `IndexExpr` determines the precision:
    /// - `Unknown`: Collapsed (legacy behavior)
    /// - `Constant(n)`: Known constant index
    /// - `Symbolic(v)`: SSA value used as index
    Index(IndexExpr),
    /// Struct field index (compile-time constant).
    Field { index: u32 },
}

/// Path through nested structures for field-sensitive analysis.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct FieldPath {
    /// Sequence of field steps from base object.
    pub steps: Vec<PathStep>,
}

impl FieldPath {
    /// Create an empty field path (represents the base object itself).
    #[must_use]
    pub const fn empty() -> Self {
        Self { steps: Vec::new() }
    }

    /// Create a field path with a single struct field.
    #[must_use]
    pub fn field(index: u32) -> Self {
        Self {
            steps: vec![PathStep::Field { index }],
        }
    }

    /// Create a field path with a single array index step (unknown/collapsed).
    ///
    /// This is the legacy behavior where all array indices collapse to the same location.
    /// For index-sensitive analysis, use [`index_constant`](Self::index_constant) or
    /// [`index_symbolic`](Self::index_symbolic).
    #[must_use]
    pub fn index() -> Self {
        Self {
            steps: vec![PathStep::Index(IndexExpr::Unknown)],
        }
    }

    /// Create a field path with a constant array index.
    ///
    /// This allows distinguishing `a[0]` from `a[1]` in the analysis.
    #[must_use]
    pub fn index_constant(value: i64) -> Self {
        Self {
            steps: vec![PathStep::Index(IndexExpr::Constant(value))],
        }
    }

    /// Create a field path with a symbolic array index.
    ///
    /// The `value_id` is the SSA value used as the index (e.g., loop counter).
    /// Z3 can be used at query time to determine if two symbolic indices may be equal.
    #[must_use]
    pub fn index_symbolic(value_id: ValueId) -> Self {
        Self {
            steps: vec![PathStep::Index(IndexExpr::Symbolic(value_id))],
        }
    }

    /// Create a field path with a single index step using the given expression.
    #[must_use]
    pub fn index_with_expr(expr: IndexExpr) -> Self {
        Self {
            steps: vec![PathStep::Index(expr)],
        }
    }

    /// Get the depth (number of steps) in this path.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.steps.len()
    }

    /// Extend this path with another path's steps.
    #[must_use]
    pub fn extend(&self, other: &Self) -> Self {
        let mut steps = self.steps.clone();
        steps.extend(other.steps.iter().cloned());
        Self { steps }
    }

    /// Truncate this path to the given depth.
    #[must_use]
    pub fn truncate(&self, depth: usize) -> Self {
        Self {
            steps: self.steps.iter().take(depth).cloned().collect(),
        }
    }
}

// =============================================================================
// GEP path resolution
// =============================================================================

/// Type alias for the constants table from `AirModule`.
pub type ConstantsTable = BTreeMap<ValueId, Constant>;

/// Resolve a GEP path by replacing `IndexExpr::Unknown` with resolved expressions.
///
/// For each `Index` step, uses the corresponding index operand to determine:
/// - `IndexExpr::Constant(n)` if the operand is a known constant integer
/// - `IndexExpr::Symbolic(v)` if tracking symbolic indices
/// - `IndexExpr::Unknown` otherwise (collapsed, default behavior)
///
/// Used by both the CI-PTA solver and the `precompute_indexed_locations` pass.
pub fn resolve_gep_path(
    path: &FieldPath,
    index_operands: &[ValueId],
    constants: Option<&ConstantsTable>,
    index_sensitivity: IndexSensitivity,
) -> FieldPath {
    if index_sensitivity == IndexSensitivity::Collapsed || index_operands.is_empty() {
        // No resolution needed
        return path.clone();
    }

    let mut operand_iter = index_operands.iter();
    let steps: Vec<PathStep> = path
        .steps
        .iter()
        .map(|step| match step {
            PathStep::Index(IndexExpr::Unknown) => {
                // Try to resolve this index
                if let Some(&operand) = operand_iter.next() {
                    resolve_index_operand(operand, constants, index_sensitivity)
                } else {
                    PathStep::Index(IndexExpr::Unknown)
                }
            }
            other => other.clone(),
        })
        .collect();

    FieldPath { steps }
}

/// Resolve a single index operand to a `PathStep::Index`.
fn resolve_index_operand(
    operand: ValueId,
    constants: Option<&ConstantsTable>,
    index_sensitivity: IndexSensitivity,
) -> PathStep {
    // Try to look up the operand in constants table
    if let Some(constants) = constants {
        if let Some(Constant::Int { value, .. }) = constants.get(&operand) {
            // Constant index found
            return PathStep::Index(IndexExpr::Constant(*value));
        }
    }

    // Not a constant - use symbolic or collapse based on config
    match index_sensitivity {
        IndexSensitivity::Collapsed | IndexSensitivity::ConstantOnly => {
            PathStep::Index(IndexExpr::Unknown)
        }
        IndexSensitivity::Symbolic => PathStep::Index(IndexExpr::Symbolic(operand)),
    }
}

/// Attempt pointer-arithmetic merge of a single-step GEP with the base location's path.
///
/// When a single-step GEP chains off a location that already has a field path,
/// merge the GEP index with the last step of the base path instead of extending.
/// E.g., `base_loc=[F0,F0] + GEP [F1]` produces `[F0, F(0+1)]=[F0,F1]`.
/// This handles LLVM's `GEP ptr, %elem0, 1` pattern that advances within an
/// array by pointer arithmetic.
///
/// Returns `Some(merged_path)` if the merge applies, `None` otherwise.
pub fn merge_gep_with_base_path(base_loc: &Location, gep_path: &FieldPath) -> Option<FieldPath> {
    if gep_path.steps.len() != 1 || base_loc.path.steps.is_empty() {
        return None;
    }
    let gep_step = gep_path.steps.first()?;
    let parent_step = base_loc.path.steps.last()?;

    // Pointer-arithmetic GEP merge: the GEP offset is ADDED to the parent's
    // last path step, replacing it. Handles both Field+Field (struct member
    // advancement) and Field+Index (array element selection via parameter).
    let merged_step = match (gep_step, parent_step) {
        // Field + Field: add indices (original behavior)
        (PathStep::Field { index: child_idx }, PathStep::Field { index: parent_idx }) => {
            PathStep::Field {
                index: parent_idx.saturating_add(*child_idx),
            }
        }
        // Index(Constant) + Field: add constant to parent field index
        (PathStep::Index(IndexExpr::Constant(k)), PathStep::Field { index: parent_idx }) => {
            // INVARIANT: array indices are small non-negative values; truncation is safe
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let offset = if *k >= 0 { *k as u32 } else { 0 };
            PathStep::Field {
                index: parent_idx.saturating_add(offset),
            }
        }
        // Index(Unknown/Symbolic) + Field: collapse to Index (unknown offset)
        (PathStep::Index(idx_expr), PathStep::Field { .. }) => PathStep::Index(idx_expr.clone()),
        _ => return None,
    };

    let mut merged_steps = base_loc.path.steps.clone();
    if let Some(last) = merged_steps.last_mut() {
        *last = merged_step;
    }
    Some(FieldPath {
        steps: merged_steps,
    })
}

// =============================================================================
// Location
// =============================================================================

/// Abstract memory location (object + optional field path).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    /// The allocation object this location belongs to.
    pub obj: ObjId,
    /// The field path within the object.
    pub path: FieldPath,
}

impl Location {
    /// Create a new location.
    #[must_use]
    pub fn new(obj: ObjId, path: FieldPath) -> Self {
        Self { obj, path }
    }
}

// =============================================================================
// Collapse warnings
// =============================================================================

/// Warning generated when a field path is collapsed due to depth limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapseWarning {
    /// The object whose field was collapsed.
    pub obj: ObjId,
    /// The original (too-deep) field path.
    pub original_path: FieldPath,
    /// The path it was collapsed to.
    pub collapsed_path: FieldPath,
}

// =============================================================================
// Memory regions
// =============================================================================

/// Classification of an abstract memory location by allocation region.
///
/// Locations from different regions cannot alias (e.g., a stack variable
/// can never point to the same memory as a heap allocation). `Unknown`
/// is a conservative fallback that may alias with any region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MemoryRegion {
    /// Stack-allocated (from `Alloca`).
    Stack,
    /// Heap-allocated (from `HeapAlloc` / malloc family).
    Heap,
    /// Global variable.
    Global,
    /// Unknown or external — may alias with any region (sound over-approximation).
    #[default]
    Unknown,
}

impl MemoryRegion {
    /// Check if two regions may alias.
    ///
    /// Returns `true` if locations from these regions could potentially refer
    /// to the same memory. `Unknown` aliases with everything for soundness.
    #[must_use]
    pub fn may_alias(self, other: Self) -> bool {
        self == MemoryRegion::Unknown || other == MemoryRegion::Unknown || self == other
    }
}

// =============================================================================
// Allocation multiplicity
// =============================================================================

/// Whether an abstract allocation site represents a unique concrete object
/// or a summary of multiple possible objects.
///
/// Used by alias analysis: `Must` alias is only sound when both pointers'
/// singleton location is `Unique`. `Summary` locations always yield `May`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AllocationMultiplicity {
    /// Provably one concrete object (e.g., a global, or a non-loop
    /// stack alloca in a non-recursive function called once).
    Unique,
    /// May represent multiple concrete objects (default — sound fallback).
    #[default]
    Summary,
}

// =============================================================================
// Location factory
// =============================================================================

/// Manages location creation with field sensitivity bounds.
///
/// Ensures deterministic location ID assignment and handles field path
/// collapsing when depth limits are exceeded.
#[derive(Debug, Clone)]
pub struct LocationFactory {
    /// Field sensitivity configuration.
    config: FieldSensitivity,
    /// Map from location ID to location. O(1) lookup via `FxHashMap`.
    locations: FxHashMap<LocId, Location>,
    /// Reverse map from location to ID. O(1) lookup via `FxHashMap`.
    id_map: FxHashMap<Location, LocId>,
    /// Counter for generating unique location IDs.
    next_id: u128,
    /// Accumulated collapse warnings.
    collapse_warnings: Vec<CollapseWarning>,
    /// Memory region classification per base object.
    regions: BTreeMap<ObjId, MemoryRegion>,
    /// Allocation multiplicity classification per base object.
    multiplicities: BTreeMap<ObjId, AllocationMultiplicity>,
}

impl LocationFactory {
    /// Create a new location factory with the given field sensitivity config.
    #[must_use]
    pub fn new(config: FieldSensitivity) -> Self {
        Self {
            config,
            locations: FxHashMap::default(),
            id_map: FxHashMap::default(),
            next_id: 0,
            collapse_warnings: Vec::new(),
            regions: BTreeMap::new(),
            multiplicities: BTreeMap::new(),
        }
    }

    /// Get or create a location ID for the given object and field path.
    ///
    /// If the path exceeds the configured depth limit, it will be collapsed
    /// to the parent location and a warning will be recorded.
    pub fn get_or_create(&mut self, obj: ObjId, path: FieldPath) -> LocId {
        // Apply field sensitivity bounds
        let (effective_path, collapsed) = self.apply_sensitivity(obj, &path);

        // Record warning if collapsed
        if collapsed {
            self.collapse_warnings.push(CollapseWarning {
                obj,
                original_path: path,
                collapsed_path: effective_path.clone(),
            });
        }

        let location = Location::new(obj, effective_path);

        // Check if we already have this location
        if let Some(&id) = self.id_map.get(&location) {
            return id;
        }

        // Create new location ID
        let id = LocId::new(self.next_id);
        self.next_id += 1;

        self.locations.insert(id, location.clone());
        self.id_map.insert(location, id);

        id
    }

    /// Get a location by ID.
    #[must_use]
    pub fn get(&self, id: LocId) -> Option<&Location> {
        self.locations.get(&id)
    }

    /// Drain accumulated collapse warnings.
    pub fn drain_warnings(&mut self) -> Vec<CollapseWarning> {
        std::mem::take(&mut self.collapse_warnings)
    }

    /// Get the number of unique locations created.
    #[must_use]
    pub fn len(&self) -> usize {
        self.locations.len()
    }

    /// Check if no locations have been created.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    /// Get all locations (for export).
    #[must_use]
    pub fn all_locations(&self) -> &FxHashMap<LocId, Location> {
        &self.locations
    }

    /// Look up a location by object + exact field path. O(1).
    #[must_use]
    pub fn lookup(&self, obj: ObjId, path: &FieldPath) -> Option<LocId> {
        let key = Location {
            obj,
            path: path.clone(),
        };
        self.id_map.get(&key).copied()
    }

    /// Look up with fallback: exact path, then parent path, then base object.
    /// Each step is O(1) via `FxHashMap` lookup.
    ///
    /// Replaces the solver's 3x linear scan in `find_or_approximate_location`.
    #[must_use]
    pub fn lookup_approx(&self, obj: ObjId, path: &FieldPath) -> Option<LocId> {
        // 1. Exact match
        if let Some(id) = self.lookup(obj, path) {
            return Some(id);
        }
        // 2. Parent path (truncate last step)
        if !path.steps.is_empty() {
            let parent = path.truncate(path.depth() - 1);
            if let Some(id) = self.lookup(obj, &parent) {
                return Some(id);
            }
        }
        // 3. Base object (empty path)
        self.lookup(obj, &FieldPath::empty())
    }

    /// Set the memory region for a base object.
    ///
    /// Child locations (fields) inherit their parent object's region
    /// automatically via [`region`](Self::region).
    pub fn set_region(&mut self, obj: ObjId, region: MemoryRegion) {
        self.regions.insert(obj, region);
    }

    /// Get the memory region for a location.
    ///
    /// Looks up the location's base object and returns its region.
    /// Defaults to `Unknown` if the object has no assigned region.
    #[must_use]
    pub fn region(&self, loc: LocId) -> MemoryRegion {
        self.locations
            .get(&loc)
            .and_then(|location| self.regions.get(&location.obj))
            .copied()
            .unwrap_or_default()
    }

    /// Check if two locations may alias based on their memory regions.
    ///
    /// Returns `false` (definitely no alias) if the locations belong to
    /// incompatible regions (e.g., Stack vs Heap). Returns `true` if
    /// either region is `Unknown` or both are the same region.
    #[must_use]
    pub fn may_alias_region(&self, a: LocId, b: LocId) -> bool {
        self.region(a).may_alias(self.region(b))
    }

    /// Get the region map (for transferring to `PtaResult`).
    #[must_use]
    pub fn regions(&self) -> &BTreeMap<ObjId, MemoryRegion> {
        &self.regions
    }

    /// Set the allocation multiplicity for a base object.
    pub fn set_multiplicity(&mut self, obj: ObjId, mult: AllocationMultiplicity) {
        self.multiplicities.insert(obj, mult);
    }

    /// Get the multiplicity for a location by `LocId`.
    ///
    /// Looks up the location's base object and returns its multiplicity.
    /// Defaults to `Summary` if the object has no assigned multiplicity.
    #[must_use]
    pub fn multiplicity(&self, loc: LocId) -> AllocationMultiplicity {
        self.locations
            .get(&loc)
            .and_then(|location| self.multiplicities.get(&location.obj))
            .copied()
            .unwrap_or_default()
    }

    /// Get the multiplicity for an object directly by `ObjId`.
    ///
    /// Defaults to `Summary` if the object has no assigned multiplicity.
    #[must_use]
    pub fn multiplicity_by_obj(&self, obj: ObjId) -> AllocationMultiplicity {
        self.multiplicities.get(&obj).copied().unwrap_or_default()
    }

    /// Get the multiplicity map (for transferring to `PtaResult`).
    #[must_use]
    pub fn multiplicities(&self) -> &BTreeMap<ObjId, AllocationMultiplicity> {
        &self.multiplicities
    }

    /// Get the memory region for an object directly by `ObjId`.
    ///
    /// Defaults to `Unknown` if the object has no assigned region.
    #[must_use]
    pub fn region_by_obj(&self, obj: ObjId) -> MemoryRegion {
        self.regions.get(&obj).copied().unwrap_or_default()
    }

    /// Apply field sensitivity configuration to a path.
    ///
    /// Returns the effective path and whether it was collapsed.
    fn apply_sensitivity(&self, _obj: ObjId, path: &FieldPath) -> (FieldPath, bool) {
        match &self.config {
            FieldSensitivity::None => {
                // All paths collapse to base object
                let collapsed = !path.steps.is_empty();
                (FieldPath::empty(), collapsed)
            }
            FieldSensitivity::StructFields { max_depth } => {
                let max = *max_depth as usize;
                if path.depth() > max {
                    (path.truncate(max), true)
                } else {
                    (path.clone(), false)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ObjId;

    #[test]
    fn field_path_empty() {
        let path = FieldPath::empty();
        assert!(path.steps.is_empty());
        assert_eq!(path.depth(), 0);
    }

    #[test]
    fn field_path_single_field() {
        let path = FieldPath::field(0);
        assert_eq!(path.steps.len(), 1);
        assert_eq!(path.depth(), 1);
        assert!(matches!(path.steps[0], PathStep::Field { index: 0 }));
    }

    #[test]
    fn field_path_extend() {
        let base = FieldPath::field(0);
        let extended = base.extend(&FieldPath::field(1));
        assert_eq!(extended.steps.len(), 2);
        assert!(matches!(extended.steps[0], PathStep::Field { index: 0 }));
        assert!(matches!(extended.steps[1], PathStep::Field { index: 1 }));
    }

    #[test]
    fn field_path_ordering() {
        let p1 = FieldPath::empty();
        let p2 = FieldPath::field(0);
        let p3 = FieldPath::field(1);
        assert!(p1 < p2);
        assert!(p2 < p3);
    }

    #[test]
    fn location_base_object() {
        let obj = ObjId::derive(b"test_alloc");
        let loc = Location::new(obj, FieldPath::empty());
        assert_eq!(loc.obj, obj);
        assert!(loc.path.steps.is_empty());
    }

    #[test]
    fn location_with_field() {
        let obj = ObjId::derive(b"test_alloc");
        let loc = Location::new(obj, FieldPath::field(2));
        assert_eq!(loc.path.depth(), 1);
    }

    #[test]
    fn location_ordering_by_obj_then_path() {
        let obj1 = ObjId::new(1);
        let obj2 = ObjId::new(2);
        let loc1 = Location::new(obj1, FieldPath::empty());
        let loc2 = Location::new(obj1, FieldPath::field(0));
        let loc3 = Location::new(obj2, FieldPath::empty());

        assert!(loc1 < loc2); // Same obj, different path
        assert!(loc2 < loc3); // Different obj
    }

    #[test]
    fn location_factory_creates_new_locations() {
        let config = super::super::config::FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);

        let obj = ObjId::derive(b"test_alloc");
        let loc1 = factory.get_or_create(obj, FieldPath::empty());
        let loc2 = factory.get_or_create(obj, FieldPath::field(0));

        assert_ne!(loc1, loc2);
    }

    #[test]
    fn location_factory_returns_same_id_for_same_location() {
        let config = super::super::config::FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);

        let obj = ObjId::derive(b"test_alloc");
        let loc1 = factory.get_or_create(obj, FieldPath::empty());
        let loc2 = factory.get_or_create(obj, FieldPath::empty());

        assert_eq!(loc1, loc2);
    }

    #[test]
    fn location_factory_collapses_deep_paths() {
        let config = super::super::config::FieldSensitivity::StructFields { max_depth: 1 };
        let mut factory = LocationFactory::new(config);

        let obj = ObjId::derive(b"test_alloc");

        // At depth 1, this should be fine
        let shallow = factory.get_or_create(obj, FieldPath::field(0));

        // At depth 2, this should collapse to depth 1
        let deep_path = FieldPath::field(0).extend(&FieldPath::field(1));
        let deep = factory.get_or_create(obj, deep_path);

        // Both should resolve to the same parent location
        assert_eq!(shallow, deep);

        // Should have generated a warning
        let warnings = factory.drain_warnings();
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn location_factory_field_sensitivity_none_collapses_all() {
        let config = super::super::config::FieldSensitivity::None;
        let mut factory = LocationFactory::new(config);

        let obj = ObjId::derive(b"test_alloc");

        let base = factory.get_or_create(obj, FieldPath::empty());
        let with_field = factory.get_or_create(obj, FieldPath::field(0));

        // With None sensitivity, all paths collapse to base object
        assert_eq!(base, with_field);
    }

    #[test]
    fn location_factory_get_returns_location() {
        let config = super::super::config::FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);

        let obj = ObjId::derive(b"test_alloc");
        let path = FieldPath::field(1);
        let loc_id = factory.get_or_create(obj, path.clone());

        let loc = factory.get(loc_id).expect("should find location");
        assert_eq!(loc.obj, obj);
        assert_eq!(loc.path, path);
    }

    #[test]
    fn collapse_warning_contains_info() {
        let obj = ObjId::derive(b"test");
        let original = FieldPath::field(0).extend(&FieldPath::field(1));
        let collapsed = FieldPath::field(0);

        let warning = CollapseWarning {
            obj,
            original_path: original.clone(),
            collapsed_path: collapsed.clone(),
        };

        assert_eq!(warning.obj, obj);
        assert_eq!(warning.original_path, original);
        assert_eq!(warning.collapsed_path, collapsed);
    }

    // =========================================================================
    // IndexExpr tests
    // =========================================================================

    #[test]
    fn index_expr_unknown_is_default() {
        let expr = IndexExpr::default();
        assert!(expr.is_unknown());
        assert!(!expr.is_constant());
        assert!(!expr.is_symbolic());
    }

    #[test]
    fn index_expr_constant_value() {
        let expr = IndexExpr::Constant(42);
        assert!(!expr.is_unknown());
        assert!(expr.is_constant());
        assert!(!expr.is_symbolic());
        assert_eq!(expr.as_constant(), Some(42));
        assert_eq!(expr.as_symbolic(), None);
    }

    #[test]
    fn index_expr_constant_negative() {
        let expr = IndexExpr::Constant(-1);
        assert_eq!(expr.as_constant(), Some(-1));
    }

    #[test]
    fn index_expr_symbolic_value() {
        let value_id = ValueId::new(123);
        let expr = IndexExpr::Symbolic(value_id);
        assert!(!expr.is_unknown());
        assert!(!expr.is_constant());
        assert!(expr.is_symbolic());
        assert_eq!(expr.as_symbolic(), Some(value_id));
        assert_eq!(expr.as_constant(), None);
    }

    #[test]
    fn index_expr_equality() {
        assert_eq!(IndexExpr::Unknown, IndexExpr::Unknown);
        assert_eq!(IndexExpr::Constant(0), IndexExpr::Constant(0));
        assert_ne!(IndexExpr::Constant(0), IndexExpr::Constant(1));
        assert_ne!(IndexExpr::Unknown, IndexExpr::Constant(0));

        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        assert_eq!(IndexExpr::Symbolic(v1), IndexExpr::Symbolic(v1));
        assert_ne!(IndexExpr::Symbolic(v1), IndexExpr::Symbolic(v2));
    }

    #[test]
    fn index_expr_ordering() {
        // Unknown < Constant < Symbolic (by enum variant order)
        assert!(IndexExpr::Unknown < IndexExpr::Constant(0));
        assert!(IndexExpr::Constant(0) < IndexExpr::Constant(1));
        assert!(IndexExpr::Constant(100) < IndexExpr::Symbolic(ValueId::new(0)));
    }

    #[test]
    fn index_expr_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn hash_of<T: Hash>(t: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            t.hash(&mut hasher);
            hasher.finish()
        }

        // Same values should have same hash
        assert_eq!(hash_of(&IndexExpr::Unknown), hash_of(&IndexExpr::Unknown));
        assert_eq!(
            hash_of(&IndexExpr::Constant(42)),
            hash_of(&IndexExpr::Constant(42))
        );
        assert_eq!(
            hash_of(&IndexExpr::Symbolic(ValueId::new(1))),
            hash_of(&IndexExpr::Symbolic(ValueId::new(1)))
        );

        // Different values should (likely) have different hashes
        assert_ne!(
            hash_of(&IndexExpr::Constant(0)),
            hash_of(&IndexExpr::Constant(1))
        );
    }

    // =========================================================================
    // PathStep with IndexExpr tests
    // =========================================================================

    #[test]
    fn path_step_index_unknown() {
        let step = PathStep::Index(IndexExpr::Unknown);
        assert!(matches!(step, PathStep::Index(IndexExpr::Unknown)));
    }

    #[test]
    fn path_step_index_constant() {
        let step = PathStep::Index(IndexExpr::Constant(5));
        if let PathStep::Index(IndexExpr::Constant(v)) = step {
            assert_eq!(v, 5);
        } else {
            panic!("expected PathStep::Index(Constant)");
        }
    }

    #[test]
    fn path_step_index_symbolic() {
        let vid = ValueId::new(99);
        let step = PathStep::Index(IndexExpr::Symbolic(vid));
        if let PathStep::Index(IndexExpr::Symbolic(v)) = step {
            assert_eq!(v, vid);
        } else {
            panic!("expected PathStep::Index(Symbolic)");
        }
    }

    #[test]
    fn path_step_ordering_index_variants() {
        // Index(Unknown) < Index(Constant(0)) < Index(Constant(1)) < Index(Symbolic)
        let unknown = PathStep::Index(IndexExpr::Unknown);
        let const0 = PathStep::Index(IndexExpr::Constant(0));
        let const1 = PathStep::Index(IndexExpr::Constant(1));
        let symbolic = PathStep::Index(IndexExpr::Symbolic(ValueId::new(0)));

        assert!(unknown < const0);
        assert!(const0 < const1);
        assert!(const1 < symbolic);
    }

    #[test]
    fn path_step_index_vs_field_ordering() {
        // Field < Index (by enum variant order in declaration)
        // But we declared Index first, so Index < Field
        let index = PathStep::Index(IndexExpr::Unknown);
        let field = PathStep::Field { index: 0 };
        assert!(index < field);
    }

    // =========================================================================
    // FieldPath with index variants tests
    // =========================================================================

    #[test]
    fn field_path_index_unknown() {
        let path = FieldPath::index();
        assert_eq!(path.steps.len(), 1);
        assert!(matches!(path.steps[0], PathStep::Index(IndexExpr::Unknown)));
    }

    #[test]
    fn field_path_index_constant() {
        let path = FieldPath::index_constant(3);
        assert_eq!(path.steps.len(), 1);
        if let PathStep::Index(IndexExpr::Constant(v)) = &path.steps[0] {
            assert_eq!(*v, 3);
        } else {
            panic!("expected Index(Constant)");
        }
    }

    #[test]
    fn field_path_index_symbolic() {
        let vid = ValueId::new(42);
        let path = FieldPath::index_symbolic(vid);
        assert_eq!(path.steps.len(), 1);
        if let PathStep::Index(IndexExpr::Symbolic(v)) = &path.steps[0] {
            assert_eq!(*v, vid);
        } else {
            panic!("expected Index(Symbolic)");
        }
    }

    #[test]
    fn field_path_index_with_expr() {
        let expr = IndexExpr::Constant(7);
        let path = FieldPath::index_with_expr(expr.clone());
        assert_eq!(path.steps.len(), 1);
        assert!(matches!(
            path.steps[0],
            PathStep::Index(IndexExpr::Constant(7))
        ));
    }

    #[test]
    fn field_path_extend_with_index_variants() {
        // field.index[0].index[1]
        let base = FieldPath::field(0);
        let with_idx0 = base.extend(&FieldPath::index_constant(0));
        let with_idx1 = with_idx0.extend(&FieldPath::index_constant(1));

        assert_eq!(with_idx1.steps.len(), 3);
        assert!(matches!(with_idx1.steps[0], PathStep::Field { index: 0 }));
        assert!(matches!(
            with_idx1.steps[1],
            PathStep::Index(IndexExpr::Constant(0))
        ));
        assert!(matches!(
            with_idx1.steps[2],
            PathStep::Index(IndexExpr::Constant(1))
        ));
    }

    // =========================================================================
    // Location with indexed paths tests
    // =========================================================================

    #[test]
    fn location_with_constant_index_differs() {
        let obj = ObjId::derive(b"array");
        let loc0 = Location::new(obj, FieldPath::index_constant(0));
        let loc1 = Location::new(obj, FieldPath::index_constant(1));
        let loc_unknown = Location::new(obj, FieldPath::index());

        // Different constant indices produce different locations
        assert_ne!(loc0, loc1);
        // Unknown index is different from constant 0
        assert_ne!(loc0, loc_unknown);
    }

    #[test]
    fn location_with_symbolic_index_differs() {
        let obj = ObjId::derive(b"array");
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);

        let loc1 = Location::new(obj, FieldPath::index_symbolic(v1));
        let loc2 = Location::new(obj, FieldPath::index_symbolic(v2));
        let loc_same = Location::new(obj, FieldPath::index_symbolic(v1));

        // Different symbolic indices produce different locations
        assert_ne!(loc1, loc2);
        // Same symbolic index produces same location
        assert_eq!(loc1, loc_same);
    }

    #[test]
    fn location_factory_distinguishes_constant_indices() {
        let config = FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);
        let obj = ObjId::derive(b"array");

        let loc0 = factory.get_or_create(obj, FieldPath::index_constant(0));
        let loc1 = factory.get_or_create(obj, FieldPath::index_constant(1));
        let loc0_again = factory.get_or_create(obj, FieldPath::index_constant(0));

        // Different indices → different LocIds
        assert_ne!(loc0, loc1);
        // Same index → same LocId
        assert_eq!(loc0, loc0_again);
        // Factory should have 2 unique locations
        assert_eq!(factory.len(), 2);
    }

    #[test]
    fn location_factory_distinguishes_symbolic_indices() {
        let config = FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);
        let obj = ObjId::derive(b"array");

        let vi = ValueId::new(100);
        let vj = ValueId::new(200);

        let loc_i = factory.get_or_create(obj, FieldPath::index_symbolic(vi));
        let loc_j = factory.get_or_create(obj, FieldPath::index_symbolic(vj));
        let loc_i_again = factory.get_or_create(obj, FieldPath::index_symbolic(vi));

        // Different symbolic values → different LocIds
        assert_ne!(loc_i, loc_j);
        // Same symbolic value → same LocId
        assert_eq!(loc_i, loc_i_again);
    }

    #[test]
    fn location_factory_unknown_index_collapses() {
        let config = FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);
        let obj = ObjId::derive(b"array");

        let loc1 = factory.get_or_create(obj, FieldPath::index());
        let loc2 = factory.get_or_create(obj, FieldPath::index());

        // Unknown indices always collapse to same location
        assert_eq!(loc1, loc2);
        assert_eq!(factory.len(), 1);
    }

    #[test]
    fn lookup_approx_exact_parent_base_fallback() {
        let config = FieldSensitivity::StructFields { max_depth: 3 };
        let mut factory = LocationFactory::new(config);
        let obj = ObjId::new(1);

        // Create base, field 0, and field 0.1
        let base_loc = factory.get_or_create(obj, FieldPath::empty());
        let f0_loc = factory.get_or_create(obj, FieldPath::field(0));
        let f01_loc = factory.get_or_create(obj, FieldPath::field(0).extend(&FieldPath::field(1)));

        // Exact match
        assert_eq!(
            factory.lookup_approx(obj, &FieldPath::field(0).extend(&FieldPath::field(1))),
            Some(f01_loc)
        );
        // Parent fallback: field(0).field(2) not created -> falls to field(0)
        assert_eq!(
            factory.lookup_approx(obj, &FieldPath::field(0).extend(&FieldPath::field(2))),
            Some(f0_loc)
        );
        // Base fallback: field(9) not created -> falls to base
        assert_eq!(
            factory.lookup_approx(obj, &FieldPath::field(9)),
            Some(base_loc)
        );
        // Exact lookup returns None for non-existent
        assert_eq!(factory.lookup(obj, &FieldPath::field(9)), None);
    }

    // =========================================================================
    // MemoryRegion tests
    // =========================================================================

    #[test]
    fn memory_region_default_is_unknown() {
        assert_eq!(MemoryRegion::default(), MemoryRegion::Unknown);
    }

    #[test]
    fn memory_region_may_alias_same_region() {
        assert!(MemoryRegion::Stack.may_alias(MemoryRegion::Stack));
        assert!(MemoryRegion::Heap.may_alias(MemoryRegion::Heap));
        assert!(MemoryRegion::Global.may_alias(MemoryRegion::Global));
        assert!(MemoryRegion::Unknown.may_alias(MemoryRegion::Unknown));
    }

    #[test]
    fn memory_region_may_alias_different_region() {
        assert!(!MemoryRegion::Stack.may_alias(MemoryRegion::Heap));
        assert!(!MemoryRegion::Stack.may_alias(MemoryRegion::Global));
        assert!(!MemoryRegion::Heap.may_alias(MemoryRegion::Global));
        assert!(!MemoryRegion::Heap.may_alias(MemoryRegion::Stack));
        assert!(!MemoryRegion::Global.may_alias(MemoryRegion::Stack));
        assert!(!MemoryRegion::Global.may_alias(MemoryRegion::Heap));
    }

    #[test]
    fn memory_region_unknown_aliases_with_all() {
        assert!(MemoryRegion::Unknown.may_alias(MemoryRegion::Stack));
        assert!(MemoryRegion::Unknown.may_alias(MemoryRegion::Heap));
        assert!(MemoryRegion::Unknown.may_alias(MemoryRegion::Global));
        assert!(MemoryRegion::Stack.may_alias(MemoryRegion::Unknown));
        assert!(MemoryRegion::Heap.may_alias(MemoryRegion::Unknown));
        assert!(MemoryRegion::Global.may_alias(MemoryRegion::Unknown));
    }

    #[test]
    fn region_tracking_set_and_get() {
        let config = FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);

        let obj = ObjId::new(1);
        let loc = factory.get_or_create(obj, FieldPath::empty());

        factory.set_region(obj, MemoryRegion::Stack);
        assert_eq!(factory.region(loc), MemoryRegion::Stack);
    }

    #[test]
    fn region_default_unknown() {
        let config = FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);

        let obj = ObjId::new(1);
        let loc = factory.get_or_create(obj, FieldPath::empty());

        // No region set — defaults to Unknown
        assert_eq!(factory.region(loc), MemoryRegion::Unknown);
    }

    #[test]
    fn field_inherits_region() {
        let config = FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);

        let obj = ObjId::new(1);
        let _base = factory.get_or_create(obj, FieldPath::empty());
        let field = factory.get_or_create(obj, FieldPath::field(0));

        factory.set_region(obj, MemoryRegion::Heap);

        // Child location inherits parent's region via shared ObjId
        assert_eq!(factory.region(field), MemoryRegion::Heap);
    }

    #[test]
    fn may_alias_region_same() {
        let config = FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);

        let obj1 = ObjId::new(1);
        let obj2 = ObjId::new(2);
        let loc1 = factory.get_or_create(obj1, FieldPath::empty());
        let loc2 = factory.get_or_create(obj2, FieldPath::empty());

        factory.set_region(obj1, MemoryRegion::Stack);
        factory.set_region(obj2, MemoryRegion::Stack);

        assert!(factory.may_alias_region(loc1, loc2));
    }

    #[test]
    fn may_alias_region_different() {
        let config = FieldSensitivity::StructFields { max_depth: 2 };
        let mut factory = LocationFactory::new(config);

        let obj1 = ObjId::new(1);
        let obj2 = ObjId::new(2);
        let loc1 = factory.get_or_create(obj1, FieldPath::empty());
        let loc2 = factory.get_or_create(obj2, FieldPath::empty());

        factory.set_region(obj1, MemoryRegion::Stack);
        factory.set_region(obj2, MemoryRegion::Heap);

        assert!(!factory.may_alias_region(loc1, loc2));
    }

    // =========================================================================
    // AllocationMultiplicity tests
    // =========================================================================

    #[test]
    fn default_multiplicity_is_summary() {
        let factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
        let loc = LocId::new(999);
        assert_eq!(factory.multiplicity(loc), AllocationMultiplicity::Summary);
    }

    #[test]
    fn set_and_get_multiplicity() {
        let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
        let obj = ObjId::new(42);
        let loc = factory.get_or_create(obj, FieldPath::empty());

        // Default is Summary
        assert_eq!(factory.multiplicity(loc), AllocationMultiplicity::Summary);

        // Set to Unique
        factory.set_multiplicity(obj, AllocationMultiplicity::Unique);
        assert_eq!(factory.multiplicity(loc), AllocationMultiplicity::Unique);
    }

    #[test]
    fn field_location_inherits_multiplicity() {
        let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
        let obj = ObjId::new(42);
        let base_loc = factory.get_or_create(obj, FieldPath::empty());
        let field_loc = factory.get_or_create(obj, FieldPath::field(0));

        factory.set_multiplicity(obj, AllocationMultiplicity::Unique);

        // Both base and field locations should report Unique
        assert_eq!(
            factory.multiplicity(base_loc),
            AllocationMultiplicity::Unique
        );
        assert_eq!(
            factory.multiplicity(field_loc),
            AllocationMultiplicity::Unique
        );
    }

    #[test]
    fn region_by_obj_default_unknown() {
        let factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
        assert_eq!(
            factory.region_by_obj(ObjId::new(999)),
            MemoryRegion::Unknown
        );
    }

    #[test]
    fn region_by_obj_returns_set_region() {
        let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
        let obj = ObjId::new(42);
        factory.set_region(obj, MemoryRegion::Heap);
        assert_eq!(factory.region_by_obj(obj), MemoryRegion::Heap);
    }
}
