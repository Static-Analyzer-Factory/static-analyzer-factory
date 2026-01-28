//! Per-module constraint extraction, caching, and diffing.
//!
//! This module provides types for organizing PTA constraints at module
//! granularity, enabling incremental re-analysis when only some modules
//! change. Key types:
//!
//! - [`ModuleConstraints`]: Constraints from a single module, serializable to disk.
//! - [`ProgramConstraints`]: All modules' constraints with a merged view.
//! - [`ConstraintDiff`]: The difference between two program constraint sets.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use saf_core::ids::{FunctionId, ModuleId};

use super::constraint::{
    AddrConstraint, ConstraintSet, CopyConstraint, GepConstraint, LoadConstraint, StoreConstraint,
};

// =============================================================================
// ModuleConstraints
// =============================================================================

/// Constraints extracted from a single module, suitable for caching.
///
/// Each module's constraints are extracted independently and can be
/// serialized to disk. When the module hasn't changed (same fingerprint),
/// cached constraints are loaded instead of re-extracting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleConstraints {
    /// The module this constraint set belongs to.
    pub module_id: ModuleId,
    /// Content fingerprint used to detect changes (e.g., BLAKE3 hash).
    pub fingerprint: String,
    /// Address-of constraints from this module.
    pub addr: BTreeSet<AddrConstraint>,
    /// Copy constraints from this module.
    pub copy: BTreeSet<CopyConstraint>,
    /// Load constraints from this module.
    pub load: BTreeSet<LoadConstraint>,
    /// Store constraints from this module.
    pub store: BTreeSet<StoreConstraint>,
    /// GEP constraints from this module.
    pub gep: BTreeSet<GepConstraint>,
    /// Functions defined (not just declared) in this module.
    pub function_ids: BTreeSet<FunctionId>,
}

impl ModuleConstraints {
    /// Convert to a [`ConstraintSet`] for solver consumption.
    #[must_use]
    pub fn to_constraint_set(&self) -> ConstraintSet {
        ConstraintSet {
            addr: self.addr.clone(),
            copy: self.copy.clone(),
            load: self.load.clone(),
            store: self.store.clone(),
            gep: self.gep.clone(),
        }
    }

    /// Create from a [`ConstraintSet`] with module metadata.
    #[must_use]
    pub fn from_constraint_set(
        module_id: ModuleId,
        fingerprint: &str,
        constraints: &ConstraintSet,
        function_ids: BTreeSet<FunctionId>,
    ) -> Self {
        Self {
            module_id,
            fingerprint: fingerprint.to_owned(),
            addr: constraints.addr.clone(),
            copy: constraints.copy.clone(),
            load: constraints.load.clone(),
            store: constraints.store.clone(),
            gep: constraints.gep.clone(),
            function_ids,
        }
    }

    /// Save to `{cache_dir}/constraints/{module_id_hex}.json`.
    ///
    /// Creates the `constraints/` subdirectory if it does not exist.
    pub fn save(&self, cache_dir: &Path) -> Result<(), std::io::Error> {
        let dir = cache_dir.join("constraints");
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", self.module_id.to_hex()));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    /// Load from `{cache_dir}/constraints/{module_id_hex}.json`.
    ///
    /// Returns `Ok(None)` if the cache file does not exist.
    pub fn load(cache_dir: &Path, module_id: &ModuleId) -> Result<Option<Self>, std::io::Error> {
        let path = cache_dir
            .join("constraints")
            .join(format!("{}.json", module_id.to_hex()));
        if !path.exists() {
            return Ok(None);
        }
        let json = fs::read_to_string(path)?;
        let mc: Self = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(mc))
    }
}

// =============================================================================
// ProgramConstraints
// =============================================================================

/// Constraints for an entire program, organized per-module with a merged view.
///
/// The merged [`ConstraintSet`] is the union of all per-module constraints
/// and can be passed directly to the PTA solver.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProgramConstraints {
    /// Per-module constraint sets, keyed by `ModuleId`.
    pub modules: BTreeMap<ModuleId, ModuleConstraints>,
    /// Union of all module constraints for solver consumption.
    pub merged: ConstraintSet,
}

impl ProgramConstraints {
    /// Build from per-module constraints by merging all into a single [`ConstraintSet`].
    #[must_use]
    pub fn from_modules(modules: Vec<ModuleConstraints>) -> Self {
        let mut merged = ConstraintSet::default();
        let mut module_map = BTreeMap::new();
        for mc in modules {
            merged.addr.extend(mc.addr.iter().cloned());
            merged.copy.extend(mc.copy.iter().cloned());
            merged.load.extend(mc.load.iter().cloned());
            merged.store.extend(mc.store.iter().cloned());
            merged.gep.extend(mc.gep.iter().cloned());
            module_map.insert(mc.module_id, mc);
        }
        Self {
            modules: module_map,
            merged,
        }
    }

    /// Compute the constraint diff between `self` (previous) and `current`.
    ///
    /// Uses module-level granularity: for changed modules, the old module's
    /// constraints are "removed" and the new module's constraints are "added".
    /// Unchanged modules (same fingerprint) contribute nothing to the diff.
    #[must_use]
    pub fn diff(&self, current: &ProgramConstraints) -> ConstraintDiff {
        let mut added = ConstraintSet::default();
        let mut removed = ConstraintSet::default();
        let mut changed_module_count: usize = 0;
        let mut unchanged_module_count: usize = 0;
        let mut changed_modules = BTreeSet::new();

        // Check modules in previous (self)
        for (id, old_mc) in &self.modules {
            if let Some(new_mc) = current.modules.get(id) {
                if old_mc.fingerprint == new_mc.fingerprint {
                    // Unchanged module
                    unchanged_module_count += 1;
                } else {
                    // Changed module: old constraints removed, new constraints added
                    changed_module_count += 1;
                    changed_modules.insert(*id);
                    removed.addr.extend(old_mc.addr.iter().cloned());
                    removed.copy.extend(old_mc.copy.iter().cloned());
                    removed.load.extend(old_mc.load.iter().cloned());
                    removed.store.extend(old_mc.store.iter().cloned());
                    removed.gep.extend(old_mc.gep.iter().cloned());

                    added.addr.extend(new_mc.addr.iter().cloned());
                    added.copy.extend(new_mc.copy.iter().cloned());
                    added.load.extend(new_mc.load.iter().cloned());
                    added.store.extend(new_mc.store.iter().cloned());
                    added.gep.extend(new_mc.gep.iter().cloned());
                }
            } else {
                // Module removed
                changed_module_count += 1;
                changed_modules.insert(*id);
                removed.addr.extend(old_mc.addr.iter().cloned());
                removed.copy.extend(old_mc.copy.iter().cloned());
                removed.load.extend(old_mc.load.iter().cloned());
                removed.store.extend(old_mc.store.iter().cloned());
                removed.gep.extend(old_mc.gep.iter().cloned());
            }
        }

        // Check modules added in current but not in previous
        for (id, new_mc) in &current.modules {
            if !self.modules.contains_key(id) {
                changed_module_count += 1;
                changed_modules.insert(*id);
                added.addr.extend(new_mc.addr.iter().cloned());
                added.copy.extend(new_mc.copy.iter().cloned());
                added.load.extend(new_mc.load.iter().cloned());
                added.store.extend(new_mc.store.iter().cloned());
                added.gep.extend(new_mc.gep.iter().cloned());
            }
        }

        ConstraintDiff {
            added,
            removed,
            changed_module_count,
            unchanged_module_count,
            changed_modules,
        }
    }
}

// =============================================================================
// ConstraintDiff
// =============================================================================

/// The difference between two program-level constraint sets.
///
/// Produced by [`ProgramConstraints::diff`]. The `added` set contains
/// constraints that are new in the current version, and `removed` contains
/// constraints that were in the previous version but not the current one.
/// Module-level counts track how many modules changed vs stayed the same.
#[derive(Debug, Clone, Default)]
pub struct ConstraintDiff {
    /// Constraints present in the new version but not the old.
    pub added: ConstraintSet,
    /// Constraints present in the old version but not the new.
    pub removed: ConstraintSet,
    /// Number of modules whose fingerprint changed (or were added/removed).
    pub changed_module_count: usize,
    /// Number of modules whose fingerprint was identical.
    pub unchanged_module_count: usize,
    /// IDs of modules that changed, were added, or were removed.
    pub changed_modules: BTreeSet<ModuleId>,
}

impl ConstraintDiff {
    /// Check if the diff is empty (no constraint changes at all).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::{FunctionId, LocId, ModuleId, ValueId};

    use crate::pta::location::FieldPath;

    /// Helper: create a simple `ModuleConstraints` with the given id and fingerprint.
    fn make_module(id: u128, fingerprint: &str) -> ModuleConstraints {
        let mut mc = ModuleConstraints {
            module_id: ModuleId::new(id),
            fingerprint: fingerprint.to_owned(),
            addr: BTreeSet::new(),
            copy: BTreeSet::new(),
            load: BTreeSet::new(),
            store: BTreeSet::new(),
            gep: BTreeSet::new(),
            function_ids: BTreeSet::new(),
        };
        // Add one unique addr constraint per module
        mc.addr.insert(AddrConstraint {
            ptr: ValueId::new(id),
            loc: LocId::new(id + 1000),
        });
        mc.copy.insert(CopyConstraint {
            dst: ValueId::new(id + 100),
            src: ValueId::new(id),
        });
        mc.function_ids.insert(FunctionId::new(id + 2000));
        mc
    }

    // =========================================================================
    // ModuleConstraints
    // =========================================================================

    #[test]
    fn to_constraint_set_preserves_all_constraints() {
        let mc = make_module(1, "fp1");
        let cs = mc.to_constraint_set();
        assert_eq!(cs.addr.len(), mc.addr.len());
        assert_eq!(cs.copy.len(), mc.copy.len());
        assert_eq!(cs.load.len(), mc.load.len());
        assert_eq!(cs.store.len(), mc.store.len());
        assert_eq!(cs.gep.len(), mc.gep.len());
    }

    #[test]
    fn from_constraint_set_round_trip() {
        let mc = make_module(1, "fp1");
        let cs = mc.to_constraint_set();
        let mc2 = ModuleConstraints::from_constraint_set(
            mc.module_id,
            &mc.fingerprint,
            &cs,
            mc.function_ids.clone(),
        );
        assert_eq!(mc.module_id, mc2.module_id);
        assert_eq!(mc.fingerprint, mc2.fingerprint);
        assert_eq!(mc.addr, mc2.addr);
        assert_eq!(mc.copy, mc2.copy);
        assert_eq!(mc.load, mc2.load);
        assert_eq!(mc.store, mc2.store);
        assert_eq!(mc.gep, mc2.gep);
        assert_eq!(mc.function_ids, mc2.function_ids);
    }

    #[test]
    fn from_constraint_set_with_gep() {
        let mut cs = ConstraintSet::default();
        cs.gep.insert(GepConstraint {
            dst: ValueId::new(10),
            src_ptr: ValueId::new(20),
            path: FieldPath::field(0),
            index_operands: vec![],
        });
        let mc =
            ModuleConstraints::from_constraint_set(ModuleId::new(1), "fp", &cs, BTreeSet::new());
        assert_eq!(mc.gep.len(), 1);
        let cs2 = mc.to_constraint_set();
        assert_eq!(cs2.gep, cs.gep);
    }

    #[test]
    fn save_and_load_round_trip() {
        let mc = make_module(42, "fingerprint_42");
        let tmp = tempfile::tempdir().expect("create temp dir");
        mc.save(tmp.path()).expect("save");

        let loaded = ModuleConstraints::load(tmp.path(), &mc.module_id)
            .expect("load")
            .expect("should exist");
        assert_eq!(mc.module_id, loaded.module_id);
        assert_eq!(mc.fingerprint, loaded.fingerprint);
        assert_eq!(mc.addr, loaded.addr);
        assert_eq!(mc.copy, loaded.copy);
        assert_eq!(mc.function_ids, loaded.function_ids);
    }

    #[test]
    fn load_missing_returns_none() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let result = ModuleConstraints::load(tmp.path(), &ModuleId::new(999))
            .expect("load should not error");
        assert!(result.is_none());
    }

    // =========================================================================
    // ProgramConstraints
    // =========================================================================

    #[test]
    fn from_modules_merges_non_overlapping() {
        let m1 = make_module(1, "fp1");
        let m2 = make_module(2, "fp2");
        let pc = ProgramConstraints::from_modules(vec![m1, m2]);
        assert_eq!(pc.modules.len(), 2);
        // Each module contributes 1 addr + 1 copy
        assert_eq!(pc.merged.addr.len(), 2);
        assert_eq!(pc.merged.copy.len(), 2);
    }

    #[test]
    fn from_modules_deduplicates_overlapping() {
        let mut m1 = make_module(1, "fp1");
        let mut m2 = make_module(2, "fp2");
        // Add the same constraint to both modules
        let shared = AddrConstraint {
            ptr: ValueId::new(999),
            loc: LocId::new(888),
        };
        m1.addr.insert(shared.clone());
        m2.addr.insert(shared);
        let pc = ProgramConstraints::from_modules(vec![m1, m2]);
        // 2 unique + 1 shared = 3, but shared is deduplicated to 1
        assert_eq!(pc.merged.addr.len(), 3);
    }

    #[test]
    fn from_modules_empty() {
        let pc = ProgramConstraints::from_modules(vec![]);
        assert!(pc.modules.is_empty());
        assert!(pc.merged.is_empty());
    }

    // =========================================================================
    // ConstraintDiff
    // =========================================================================

    #[test]
    fn diff_no_change() {
        let m1 = make_module(1, "fp1");
        let m2 = make_module(2, "fp2");
        let prev = ProgramConstraints::from_modules(vec![m1.clone(), m2.clone()]);
        let curr = ProgramConstraints::from_modules(vec![m1, m2]);
        let diff = prev.diff(&curr);
        assert!(diff.is_empty());
        assert_eq!(diff.unchanged_module_count, 2);
        assert_eq!(diff.changed_module_count, 0);
    }

    #[test]
    fn diff_module_added() {
        let m1 = make_module(1, "fp1");
        let m2 = make_module(2, "fp2");
        let prev = ProgramConstraints::from_modules(vec![m1.clone()]);
        let curr = ProgramConstraints::from_modules(vec![m1, m2.clone()]);
        let diff = prev.diff(&curr);
        assert_eq!(diff.unchanged_module_count, 1);
        assert_eq!(diff.changed_module_count, 1);
        // Added constraints should match m2's constraints
        assert_eq!(diff.added.addr.len(), m2.addr.len());
        assert_eq!(diff.added.copy.len(), m2.copy.len());
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn diff_module_removed() {
        let m1 = make_module(1, "fp1");
        let m2 = make_module(2, "fp2");
        let prev = ProgramConstraints::from_modules(vec![m1.clone(), m2.clone()]);
        let curr = ProgramConstraints::from_modules(vec![m1]);
        let diff = prev.diff(&curr);
        assert_eq!(diff.unchanged_module_count, 1);
        assert_eq!(diff.changed_module_count, 1);
        assert!(diff.added.is_empty());
        assert_eq!(diff.removed.addr.len(), m2.addr.len());
        assert_eq!(diff.removed.copy.len(), m2.copy.len());
    }

    #[test]
    fn diff_module_changed() {
        let m1 = make_module(1, "fp1");
        let m2_old = make_module(2, "fp2_old");
        let m2_new = make_module(2, "fp2_new"); // same id, different fingerprint
        let prev = ProgramConstraints::from_modules(vec![m1.clone(), m2_old.clone()]);
        let curr = ProgramConstraints::from_modules(vec![m1, m2_new.clone()]);
        let diff = prev.diff(&curr);
        assert_eq!(diff.unchanged_module_count, 1);
        assert_eq!(diff.changed_module_count, 1);
        // Old module's constraints removed, new module's constraints added
        assert_eq!(diff.removed.addr.len(), m2_old.addr.len());
        assert_eq!(diff.added.addr.len(), m2_new.addr.len());
    }

    #[test]
    fn diff_module_same_fingerprint_is_unchanged() {
        let m1 = make_module(1, "same_fp");
        // Create a second instance with the same id and fingerprint
        let m1_copy = make_module(1, "same_fp");
        let prev = ProgramConstraints::from_modules(vec![m1]);
        let curr = ProgramConstraints::from_modules(vec![m1_copy]);
        let diff = prev.diff(&curr);
        assert!(diff.is_empty());
        assert_eq!(diff.unchanged_module_count, 1);
        assert_eq!(diff.changed_module_count, 0);
    }

    #[test]
    fn diff_is_empty_when_no_changes() {
        let diff = ConstraintDiff::default();
        assert!(diff.is_empty());
    }

    #[test]
    fn diff_all_modules_replaced() {
        let m1 = make_module(1, "fp1");
        let m2 = make_module(2, "fp2");
        let m3 = make_module(3, "fp3");
        let m4 = make_module(4, "fp4");
        let prev = ProgramConstraints::from_modules(vec![m1, m2]);
        let curr = ProgramConstraints::from_modules(vec![m3, m4]);
        let diff = prev.diff(&curr);
        // All previous modules removed, all current modules added
        assert_eq!(diff.changed_module_count, 4);
        assert_eq!(diff.unchanged_module_count, 0);
        assert_eq!(diff.removed.addr.len(), 2);
        assert_eq!(diff.added.addr.len(), 2);
    }
}
