//! Cascading invalidation with early-termination gates.
//!
//! The [`InvalidationController`] tracks product versions and computes
//! the minimum set of [`RecomputeStep`]s needed after a code change.
//! It supports early termination: if constraints are identical after
//! re-extraction, or if points-to sets are unchanged after incremental
//! PTA, downstream products are not invalidated.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{FunctionId, ModuleId};

use crate::pta::ConstraintSet;

// =============================================================================
// Product identifiers
// =============================================================================

/// Products that can be invalidated.
///
/// Each variant represents an analysis artifact that has a version
/// and depends on upstream products. The invalidation controller
/// tracks which products are stale and computes the cascade.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProductId {
    /// A parsed/ingested module.
    Module(ModuleId),
    /// Per-module PTA constraints.
    ModuleConstraints(ModuleId),
    /// Whole-program points-to analysis result.
    Pta,
    /// Call graph (depends on PTA for indirect call resolution).
    CallGraph,
    /// Per-function def-use chains.
    DefUse(FunctionId),
    /// Whole-program value-flow graph.
    ValueFlow,
    /// Sparse value-flow graph (depends on value-flow).
    Svfg,
    /// Checker results (depends on SVFG).
    CheckerResults,
    /// Per-function summary (depends on PTA).
    Summary(FunctionId),
}

// =============================================================================
// Recompute steps
// =============================================================================

/// A recompute action produced by the invalidation controller.
///
/// Steps are returned in dependency order: earlier steps must complete
/// before later steps can execute.
#[derive(Debug, Clone)]
pub enum RecomputeStep {
    /// Re-ingest a module from source/IR.
    ReingestModule(ModuleId),
    /// Re-extract PTA constraints for a module.
    ReextractConstraints(ModuleId),
    /// Run incremental PTA with the given constraint diff.
    IncrementalPta {
        /// Constraints added since the previous run.
        added: ConstraintSet,
        /// Constraints removed since the previous run.
        removed: ConstraintSet,
    },
    /// Rebuild the call graph from updated PTA results.
    RebuildCallGraph,
    /// Recompute the summary for a specific function.
    RecomputeSummary(FunctionId),
    /// Rebuild value-flow for the given set of affected functions.
    RebuildValueFlow(BTreeSet<FunctionId>),
    /// Invalidate the SVFG (must be rebuilt from value-flow).
    InvalidateSvfg,
    /// Re-run checkers for the given set of affected functions.
    RerunCheckers(BTreeSet<FunctionId>),
}

// =============================================================================
// Invalidation triggers
// =============================================================================

/// Trigger for invalidation.
///
/// Passed to [`InvalidationController::plan_recompute`] to compute
/// the minimum cascade of recompute steps.
#[derive(Debug, Clone)]
pub enum InvalidationTrigger {
    /// One or more modules have changed on disk.
    ModulesChanged(Vec<ModuleId>),
    /// Constraints have changed (already diffed).
    ConstraintsChanged {
        /// Constraints added since the previous run.
        added: ConstraintSet,
        /// Constraints removed since the previous run.
        removed: ConstraintSet,
    },
}

// =============================================================================
// Controller
// =============================================================================

/// Tracks product versions and computes minimal recompute plans.
///
/// The controller maintains a version counter for each product. When
/// a trigger arrives, it bumps the versions of affected products and
/// returns the ordered list of [`RecomputeStep`]s.
///
/// # Early termination
///
/// The controller supports two early-termination gates:
///
/// 1. **Constraint gate**: If re-extraction produces identical constraints
///    (the diff is empty), no PTA or downstream work is needed.
/// 2. **PTA gate**: If incremental PTA produces no points-to changes,
///    call graph and value-flow are not invalidated.
///
/// Callers signal these gates by calling [`notify_constraints_unchanged`]
/// or [`notify_pta_unchanged`] between executing steps.
#[derive(Debug, Clone)]
pub struct InvalidationController {
    /// Per-product version counters.
    versions: BTreeMap<ProductId, u64>,
}

impl InvalidationController {
    /// Create a new controller with no tracked products.
    #[must_use]
    pub fn new() -> Self {
        Self {
            versions: BTreeMap::new(),
        }
    }

    /// Get the current version of a product, or 0 if untracked.
    #[must_use]
    pub fn version(&self, product: &ProductId) -> u64 {
        self.versions.get(product).copied().unwrap_or(0)
    }

    /// Bump a product's version and return the new version.
    pub fn bump(&mut self, product: ProductId) -> u64 {
        let v = self.versions.entry(product).or_insert(0);
        *v += 1;
        *v
    }

    /// Plan the full cascade of recompute steps for a trigger.
    ///
    /// Returns steps in dependency order. The caller should execute
    /// each step and may call [`notify_constraints_unchanged`] or
    /// [`notify_pta_unchanged`] to prune the remaining steps.
    #[must_use]
    pub fn plan_recompute(&mut self, trigger: &InvalidationTrigger) -> Vec<RecomputeStep> {
        match trigger {
            InvalidationTrigger::ModulesChanged(modules) => self.plan_modules_changed(modules),
            InvalidationTrigger::ConstraintsChanged { added, removed } => {
                self.plan_constraints_changed(added, removed)
            }
        }
    }

    /// Plan recompute steps when modules have changed.
    ///
    /// Full cascade: reingest -> re-extract -> incremental PTA ->
    /// rebuild CG -> rebuild VF -> invalidate SVFG -> rerun checkers.
    ///
    /// The caller should check for early termination after constraint
    /// extraction and after PTA.
    fn plan_modules_changed(&mut self, modules: &[ModuleId]) -> Vec<RecomputeStep> {
        if modules.is_empty() {
            return Vec::new();
        }

        let mut steps = Vec::new();

        // Phase 1: Reingest and re-extract for each changed module
        for &module_id in modules {
            self.bump(ProductId::Module(module_id));
            steps.push(RecomputeStep::ReingestModule(module_id));

            self.bump(ProductId::ModuleConstraints(module_id));
            steps.push(RecomputeStep::ReextractConstraints(module_id));
        }

        // Phase 2: Incremental PTA with empty diff placeholder.
        // The actual diff is computed by the caller after re-extraction.
        // We include the step so the caller knows PTA must be considered.
        self.bump(ProductId::Pta);
        steps.push(RecomputeStep::IncrementalPta {
            added: ConstraintSet::default(),
            removed: ConstraintSet::default(),
        });

        // Phase 3: Downstream cascade
        self.bump(ProductId::CallGraph);
        steps.push(RecomputeStep::RebuildCallGraph);

        self.bump(ProductId::ValueFlow);
        steps.push(RecomputeStep::RebuildValueFlow(BTreeSet::new()));

        self.bump(ProductId::Svfg);
        steps.push(RecomputeStep::InvalidateSvfg);

        self.bump(ProductId::CheckerResults);
        steps.push(RecomputeStep::RerunCheckers(BTreeSet::new()));

        steps
    }

    /// Plan recompute steps when constraints have already been diffed.
    ///
    /// Skips reingest/re-extract phases and goes directly to PTA.
    fn plan_constraints_changed(
        &mut self,
        added: &ConstraintSet,
        removed: &ConstraintSet,
    ) -> Vec<RecomputeStep> {
        // Early termination: no constraint changes at all
        if added.is_empty() && removed.is_empty() {
            return Vec::new();
        }

        let mut steps = Vec::new();

        // Phase 2: Incremental PTA
        self.bump(ProductId::Pta);
        steps.push(RecomputeStep::IncrementalPta {
            added: added.clone(),
            removed: removed.clone(),
        });

        // Phase 3: Downstream cascade
        self.bump(ProductId::CallGraph);
        steps.push(RecomputeStep::RebuildCallGraph);

        self.bump(ProductId::ValueFlow);
        steps.push(RecomputeStep::RebuildValueFlow(BTreeSet::new()));

        self.bump(ProductId::Svfg);
        steps.push(RecomputeStep::InvalidateSvfg);

        self.bump(ProductId::CheckerResults);
        steps.push(RecomputeStep::RerunCheckers(BTreeSet::new()));

        steps
    }

    /// Prune a recompute plan after discovering that constraints are
    /// identical post-extraction. Returns only the steps that precede
    /// the PTA phase (reingest + re-extract).
    #[must_use]
    pub fn prune_after_constraints_unchanged(steps: &[RecomputeStep]) -> Vec<RecomputeStep> {
        steps
            .iter()
            .take_while(|s| !matches!(s, RecomputeStep::IncrementalPta { .. }))
            .cloned()
            .collect()
    }

    /// Prune a recompute plan after discovering that PTA results are
    /// unchanged. Returns steps up to and including the PTA phase,
    /// but drops CG/VF/SVFG/checker steps.
    #[must_use]
    pub fn prune_after_pta_unchanged(steps: &[RecomputeStep]) -> Vec<RecomputeStep> {
        let mut result = Vec::new();
        for step in steps {
            let is_pta = matches!(step, RecomputeStep::IncrementalPta { .. });
            result.push(step.clone());
            if is_pta {
                break;
            }
        }
        result
    }
}

impl Default for InvalidationController {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::{LocId, ModuleId, ValueId};

    use crate::pta::AddrConstraint;

    /// Helper: create a `ConstraintSet` with one addr constraint.
    fn make_constraints(ptr: u128, loc: u128) -> ConstraintSet {
        let mut cs = ConstraintSet::default();
        cs.addr.insert(AddrConstraint {
            ptr: ValueId::new(ptr),
            loc: LocId::new(loc),
        });
        cs
    }

    // =========================================================================
    // Empty / no-op cases
    // =========================================================================

    #[test]
    fn no_changes_produces_empty_plan() {
        let mut ctrl = InvalidationController::new();
        let trigger = InvalidationTrigger::ModulesChanged(vec![]);
        let steps = ctrl.plan_recompute(&trigger);
        assert!(steps.is_empty());
    }

    #[test]
    fn constraints_unchanged_produces_empty_plan() {
        let mut ctrl = InvalidationController::new();
        let trigger = InvalidationTrigger::ConstraintsChanged {
            added: ConstraintSet::default(),
            removed: ConstraintSet::default(),
        };
        let steps = ctrl.plan_recompute(&trigger);
        assert!(steps.is_empty());
    }

    // =========================================================================
    // Full cascade
    // =========================================================================

    #[test]
    fn module_changed_produces_full_cascade() {
        let mut ctrl = InvalidationController::new();
        let m1 = ModuleId::new(1);
        let trigger = InvalidationTrigger::ModulesChanged(vec![m1]);
        let steps = ctrl.plan_recompute(&trigger);

        // Expect: Reingest, ReextractConstraints, IncrementalPta,
        //         RebuildCallGraph, RebuildValueFlow, InvalidateSvfg,
        //         RerunCheckers
        assert_eq!(steps.len(), 7);
        assert!(matches!(&steps[0], RecomputeStep::ReingestModule(id) if *id == m1));
        assert!(matches!(&steps[1], RecomputeStep::ReextractConstraints(id) if *id == m1));
        assert!(matches!(&steps[2], RecomputeStep::IncrementalPta { .. }));
        assert!(matches!(&steps[3], RecomputeStep::RebuildCallGraph));
        assert!(matches!(&steps[4], RecomputeStep::RebuildValueFlow(_)));
        assert!(matches!(&steps[5], RecomputeStep::InvalidateSvfg));
        assert!(matches!(&steps[6], RecomputeStep::RerunCheckers(_)));
    }

    #[test]
    fn multiple_modules_changed() {
        let mut ctrl = InvalidationController::new();
        let m1 = ModuleId::new(1);
        let m2 = ModuleId::new(2);
        let trigger = InvalidationTrigger::ModulesChanged(vec![m1, m2]);
        let steps = ctrl.plan_recompute(&trigger);

        // 2 reingest + 2 re-extract + 1 PTA + 1 CG + 1 VF + 1 SVFG + 1 checkers = 9
        assert_eq!(steps.len(), 9);
        assert!(matches!(&steps[0], RecomputeStep::ReingestModule(id) if *id == m1));
        assert!(matches!(&steps[1], RecomputeStep::ReextractConstraints(id) if *id == m1));
        assert!(matches!(&steps[2], RecomputeStep::ReingestModule(id) if *id == m2));
        assert!(matches!(&steps[3], RecomputeStep::ReextractConstraints(id) if *id == m2));
        assert!(matches!(&steps[4], RecomputeStep::IncrementalPta { .. }));
    }

    #[test]
    fn constraints_changed_skips_reingest() {
        let mut ctrl = InvalidationController::new();
        let added = make_constraints(1, 100);
        let trigger = InvalidationTrigger::ConstraintsChanged {
            added: added.clone(),
            removed: ConstraintSet::default(),
        };
        let steps = ctrl.plan_recompute(&trigger);

        // No reingest/re-extract, starts at PTA
        assert_eq!(steps.len(), 5);
        assert!(
            matches!(&steps[0], RecomputeStep::IncrementalPta { added: a, .. } if !a.is_empty())
        );
        assert!(matches!(&steps[1], RecomputeStep::RebuildCallGraph));
    }

    // =========================================================================
    // Early termination
    // =========================================================================

    #[test]
    fn prune_after_constraints_unchanged() {
        let mut ctrl = InvalidationController::new();
        let m1 = ModuleId::new(1);
        let trigger = InvalidationTrigger::ModulesChanged(vec![m1]);
        let steps = ctrl.plan_recompute(&trigger);

        // Simulate: after re-extraction, constraints are identical
        let pruned = InvalidationController::prune_after_constraints_unchanged(&steps);

        // Only reingest + re-extract remain
        assert_eq!(pruned.len(), 2);
        assert!(matches!(&pruned[0], RecomputeStep::ReingestModule(_)));
        assert!(matches!(&pruned[1], RecomputeStep::ReextractConstraints(_)));
    }

    #[test]
    fn prune_after_pta_unchanged() {
        let mut ctrl = InvalidationController::new();
        let m1 = ModuleId::new(1);
        let trigger = InvalidationTrigger::ModulesChanged(vec![m1]);
        let steps = ctrl.plan_recompute(&trigger);

        // Simulate: after PTA, points-to sets are identical
        let pruned = InvalidationController::prune_after_pta_unchanged(&steps);

        // Reingest + re-extract + incremental PTA, but no CG/VF/SVFG/checkers
        assert_eq!(pruned.len(), 3);
        assert!(matches!(&pruned[0], RecomputeStep::ReingestModule(_)));
        assert!(matches!(&pruned[1], RecomputeStep::ReextractConstraints(_)));
        assert!(matches!(&pruned[2], RecomputeStep::IncrementalPta { .. }));
    }

    #[test]
    fn prune_constraints_unchanged_on_empty_plan() {
        let pruned = InvalidationController::prune_after_constraints_unchanged(&[]);
        assert!(pruned.is_empty());
    }

    #[test]
    fn prune_pta_unchanged_on_empty_plan() {
        let pruned = InvalidationController::prune_after_pta_unchanged(&[]);
        assert!(pruned.is_empty());
    }

    // =========================================================================
    // Version tracking
    // =========================================================================

    #[test]
    fn version_starts_at_zero() {
        let ctrl = InvalidationController::new();
        assert_eq!(ctrl.version(&ProductId::Pta), 0);
        assert_eq!(ctrl.version(&ProductId::CallGraph), 0);
    }

    #[test]
    fn bump_increments_version() {
        let mut ctrl = InvalidationController::new();
        assert_eq!(ctrl.bump(ProductId::Pta), 1);
        assert_eq!(ctrl.bump(ProductId::Pta), 2);
        assert_eq!(ctrl.version(&ProductId::Pta), 2);
    }

    #[test]
    fn versions_bumped_during_plan() {
        let mut ctrl = InvalidationController::new();
        let m1 = ModuleId::new(1);
        let trigger = InvalidationTrigger::ModulesChanged(vec![m1]);
        let _ = ctrl.plan_recompute(&trigger);

        assert_eq!(ctrl.version(&ProductId::Module(m1)), 1);
        assert_eq!(ctrl.version(&ProductId::ModuleConstraints(m1)), 1);
        assert_eq!(ctrl.version(&ProductId::Pta), 1);
        assert_eq!(ctrl.version(&ProductId::CallGraph), 1);
        assert_eq!(ctrl.version(&ProductId::ValueFlow), 1);
        assert_eq!(ctrl.version(&ProductId::Svfg), 1);
        assert_eq!(ctrl.version(&ProductId::CheckerResults), 1);
    }

    #[test]
    fn successive_plans_increment_versions() {
        let mut ctrl = InvalidationController::new();
        let m1 = ModuleId::new(1);

        let trigger = InvalidationTrigger::ModulesChanged(vec![m1]);
        let _ = ctrl.plan_recompute(&trigger);
        let _ = ctrl.plan_recompute(&trigger);

        assert_eq!(ctrl.version(&ProductId::Module(m1)), 2);
        assert_eq!(ctrl.version(&ProductId::Pta), 2);
    }

    // =========================================================================
    // Default trait
    // =========================================================================

    #[test]
    fn default_creates_empty_controller() {
        let ctrl = InvalidationController::default();
        assert_eq!(ctrl.version(&ProductId::Pta), 0);
    }
}
