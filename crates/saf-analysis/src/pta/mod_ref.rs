//! Mod/Ref analysis for function side effects.
//!
//! Computes which memory locations each function may modify (mod)
//! or reference (ref). Used for precise memory invalidation in
//! interprocedural abstract interpretation.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, LocId, ValueId};
use saf_core::spec::{Role, SpecRegistry};

use super::result::PtaResult;

/// Summary of a function's memory side effects.
#[derive(Debug, Clone, Default)]
pub struct ModRefSummary {
    /// Locations this function may modify.
    pub modified_locs: BTreeSet<LocId>,
    /// Locations this function may read.
    pub referenced_locs: BTreeSet<LocId>,
    /// Pointers this function stores to (`ValueId`-based, for no-PTA fallback).
    pub modified_ptrs: BTreeSet<ValueId>,
    /// Pointers this function loads from (`ValueId`-based).
    pub referenced_ptrs: BTreeSet<ValueId>,
    /// Whether function may modify unknown locations (escaping pointers).
    pub modifies_unknown: bool,
    /// Whether function may read unknown locations.
    pub references_unknown: bool,
}

#[allow(dead_code)] // Public API for mod/ref analysis queries
impl ModRefSummary {
    /// Create a new empty summary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a conservative summary (modifies/references everything).
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            modifies_unknown: true,
            references_unknown: true,
            ..Default::default()
        }
    }

    /// Check if this function has no side effects.
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.modified_locs.is_empty() && self.modified_ptrs.is_empty() && !self.modifies_unknown
    }

    /// Check if two summaries might interfere (writes of one could affect reads of another).
    #[must_use]
    pub fn may_interfere(&self, other: &Self) -> bool {
        // If either modifies unknown, we can't rule out interference
        if self.modifies_unknown || other.references_unknown {
            return true;
        }
        if other.modifies_unknown || self.references_unknown {
            return true;
        }

        // Check for overlap between modified and referenced locations
        !self.modified_locs.is_disjoint(&other.referenced_locs)
            || !other.modified_locs.is_disjoint(&self.referenced_locs)
    }
}

/// Compute mod/ref summaries for all functions in a module.
///
/// Convenience wrapper around [`compute_all_mod_ref_with_specs`] with no specs.
#[must_use]
#[allow(dead_code)] // Public API re-exported from pta::mod; used in tests
pub fn compute_all_mod_ref(
    module: &AirModule,
    pta: &PtaResult,
) -> BTreeMap<FunctionId, ModRefSummary> {
    compute_all_mod_ref_with_specs(module, pta, None)
}

/// Compute mod/ref summaries for all functions in a module, using specs for external functions.
///
/// When specs are provided, external functions use spec-derived summaries instead of
/// conservative fallback. This enables:
/// - Pure functions (no side effects) from `pure: true` or inference
/// - Read-only functions from `params[*].reads: true` without `modifies: true`
/// - Modifying functions from `params[*].modifies: true`
#[must_use]
pub fn compute_all_mod_ref_with_specs(
    module: &AirModule,
    pta: &PtaResult,
    specs: Option<&SpecRegistry>,
) -> BTreeMap<FunctionId, ModRefSummary> {
    let mut summaries = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            // External functions: try spec first, then conservative
            let summary = specs
                .and_then(|s| summary_from_spec(&func.name, s))
                .unwrap_or_else(ModRefSummary::conservative);
            summaries.insert(func.id, summary);
            continue;
        }

        let summary = compute_function_mod_ref(func, pta);
        summaries.insert(func.id, summary);
    }

    summaries
}

/// Build a mod/ref summary from a function spec.
///
/// Returns `None` if no spec exists or if the spec provides insufficient
/// information to improve on conservative.
///
/// # Logic
///
/// - `pure: true` → pure summary (no mod, no ref of unknown)
/// - Any param with `modifies: true` → modifies_unknown = true
/// - Any param with `reads: true` → references_unknown = true
/// - No params have `modifies` or `reads` and not marked pure → None (use conservative)
/// - Allocators always modify (they allocate memory)
#[must_use]
pub fn summary_from_spec(name: &str, specs: &SpecRegistry) -> Option<ModRefSummary> {
    let spec = specs.lookup(name)?;

    // Explicitly marked pure: no side effects
    if spec.is_pure() {
        return Some(ModRefSummary::new());
    }

    // Allocators always have side effects (they allocate memory)
    if matches!(spec.role, Some(Role::Allocator | Role::Reallocator)) {
        return Some(ModRefSummary::conservative());
    }

    // Deallocators modify memory
    if matches!(spec.role, Some(Role::Deallocator)) {
        return Some(ModRefSummary {
            modifies_unknown: true,
            references_unknown: false,
            ..Default::default()
        });
    }

    // Check params for modifies/reads
    if spec.params.is_empty() {
        // No param info, can't improve on conservative
        return None;
    }

    let mut has_modifies = false;
    let mut has_reads = false;
    let mut has_any_info = false;

    for param in &spec.params {
        if param.modifies == Some(true) {
            has_modifies = true;
            has_any_info = true;
        }
        if param.reads == Some(true) {
            has_reads = true;
            has_any_info = true;
        }
    }

    // If we have no useful info from params, can't improve on conservative
    if !has_any_info {
        return None;
    }

    // Build summary from param info
    // Note: We set modifies_unknown/references_unknown because we don't know
    // which specific locations, but we do know whether the function modifies at all
    Some(ModRefSummary {
        modifies_unknown: has_modifies,
        references_unknown: has_reads,
        ..Default::default()
    })
}

/// Compute mod/ref summary for a single function.
// `ptr` (pointer operand) and `pta` (points-to analysis) are distinct concepts
#[allow(clippy::similar_names)]
fn compute_function_mod_ref(func: &saf_core::air::AirFunction, pta: &PtaResult) -> ModRefSummary {
    let mut summary = ModRefSummary::default();

    for block in &func.blocks {
        for inst in &block.instructions {
            match &inst.op {
                Operation::Store => {
                    // Store operands: [value, pointer]
                    if inst.operands.len() >= 2 {
                        let ptr = inst.operands[1];
                        summary.modified_ptrs.insert(ptr);

                        let pts = pta.points_to(ptr);
                        if pts.is_empty() {
                            summary.modifies_unknown = true;
                        } else {
                            for loc in pts {
                                summary.modified_locs.insert(loc);
                            }
                        }
                    }
                }
                Operation::Load => {
                    // Load operands: [pointer]
                    if let Some(&ptr) = inst.operands.first() {
                        summary.referenced_ptrs.insert(ptr);

                        let pts = pta.points_to(ptr);
                        if pts.is_empty() {
                            summary.references_unknown = true;
                        } else {
                            for loc in pts {
                                summary.referenced_locs.insert(loc);
                            }
                        }
                    }
                }
                Operation::Memcpy | Operation::Memset => {
                    // These modify memory at destination pointer
                    summary.modifies_unknown = true;
                    if matches!(&inst.op, Operation::Memcpy) {
                        summary.references_unknown = true;
                    }
                }
                Operation::CallDirect { .. } | Operation::CallIndirect { .. } => {
                    // Conservative: assume callees may modify/reference anything
                    // TODO: Use callee summaries for interprocedural precision
                    summary.modifies_unknown = true;
                    summary.references_unknown = true;
                }
                _ => {}
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction};
    use saf_core::ids::{BlockId, InstId, ModuleId};

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }
    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }
    fn bid(n: u128) -> BlockId {
        BlockId::new(n)
    }
    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }

    fn empty_pta() -> PtaResult {
        use std::sync::Arc;

        use crate::{FieldSensitivity, LocationFactory, PointsToMap, PtaDiagnostics};

        PtaResult::new(
            PointsToMap::new(),
            Arc::new(LocationFactory::new(FieldSensitivity::None)),
            PtaDiagnostics::default(),
        )
    }

    #[test]
    fn mod_ref_detects_store_as_mod() {
        let store_inst =
            Instruction::new(iid(1), Operation::Store).with_operands(vec![vid(1), vid(2)]);

        let mut block = AirBlock::new(bid(1));
        block.instructions = vec![store_inst];

        let mut func = AirFunction::new(fid(1), "test");
        func.blocks = vec![block];
        func.is_declaration = false;

        let mut module = AirModule::new(ModuleId::derive(b"test"));
        module.functions = vec![func];

        let summaries = compute_all_mod_ref(&module, &empty_pta());
        let summary = summaries.get(&fid(1)).unwrap();

        assert!(summary.modified_ptrs.contains(&vid(2)));
        assert!(summary.modifies_unknown);
    }

    #[test]
    fn mod_ref_detects_load_as_ref() {
        let load_inst = Instruction::new(iid(1), Operation::Load)
            .with_operands(vec![vid(1)])
            .with_dst(vid(2));

        let mut block = AirBlock::new(bid(1));
        block.instructions = vec![load_inst];

        let mut func = AirFunction::new(fid(1), "test");
        func.blocks = vec![block];
        func.is_declaration = false;

        let mut module = AirModule::new(ModuleId::derive(b"test"));
        module.functions = vec![func];

        let summaries = compute_all_mod_ref(&module, &empty_pta());
        let summary = summaries.get(&fid(1)).unwrap();

        assert!(summary.referenced_ptrs.contains(&vid(1)));
        assert!(summary.references_unknown);
    }

    #[test]
    fn mod_ref_external_is_conservative() {
        let mut func = AirFunction::new(fid(1), "external");
        func.is_declaration = true;

        let mut module = AirModule::new(ModuleId::derive(b"test"));
        module.functions = vec![func];

        let summaries = compute_all_mod_ref(&module, &empty_pta());
        let summary = summaries.get(&fid(1)).unwrap();

        assert!(summary.modifies_unknown);
        assert!(summary.references_unknown);
    }

    #[test]
    fn summary_is_pure() {
        let summary = ModRefSummary::new();
        assert!(summary.is_pure());

        let mut impure = ModRefSummary::new();
        impure.modifies_unknown = true;
        assert!(!impure.is_pure());
    }

    #[test]
    fn summaries_may_interfere() {
        let mut writer = ModRefSummary::new();
        writer.modified_locs.insert(LocId::new(100));

        let mut reader = ModRefSummary::new();
        reader.referenced_locs.insert(LocId::new(100));

        assert!(writer.may_interfere(&reader));

        // Non-interfering: different locations
        let mut other_reader = ModRefSummary::new();
        other_reader.referenced_locs.insert(LocId::new(200));

        assert!(!writer.may_interfere(&other_reader));
    }

    #[test]
    fn summary_from_spec_pure_function() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: strlen
                pure: true
            "#,
        )
        .unwrap();

        let summary = summary_from_spec("strlen", &registry).unwrap();
        assert!(summary.is_pure());
        assert!(!summary.modifies_unknown);
        assert!(!summary.references_unknown);
    }

    #[test]
    fn summary_from_spec_modifying_function() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: strcpy
                params:
                  - index: 0
                    modifies: true
                  - index: 1
                    reads: true
            "#,
        )
        .unwrap();

        let summary = summary_from_spec("strcpy", &registry).unwrap();
        assert!(!summary.is_pure());
        assert!(summary.modifies_unknown);
        assert!(summary.references_unknown);
    }

    #[test]
    fn summary_from_spec_read_only_function() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: strcmp
                params:
                  - index: 0
                    reads: true
                  - index: 1
                    reads: true
            "#,
        )
        .unwrap();

        let summary = summary_from_spec("strcmp", &registry).unwrap();
        assert!(summary.is_pure()); // No modifies = pure
        assert!(!summary.modifies_unknown);
        assert!(summary.references_unknown);
    }

    #[test]
    fn summary_from_spec_allocator_is_conservative() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: malloc
                role: allocator
            "#,
        )
        .unwrap();

        let summary = summary_from_spec("malloc", &registry).unwrap();
        assert!(summary.modifies_unknown);
        assert!(summary.references_unknown);
    }

    #[test]
    fn summary_from_spec_unknown_function_returns_none() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: strlen
                pure: true
            "#,
        )
        .unwrap();

        let summary = summary_from_spec("unknown_func", &registry);
        assert!(summary.is_none());
    }
}
