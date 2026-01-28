//! Unified lookup for function summaries from all sources.
//!
//! The [`SummaryRegistry`] provides a three-tier priority lookup for function
//! summaries:
//!
//! 1. **User specs** (highest priority) -- hand-authored YAML overrides
//! 2. **Computed** -- analysis-derived summaries
//! 3. **Defaults** (lowest priority) -- shipped built-in specs
//!
//! This unifies YAML-authored specs and analysis-computed summaries into a
//! single lookup interface, used by incremental and compositional analyses.

use std::collections::BTreeMap;

use crate::ids::FunctionId;
use crate::spec::SpecRegistry;
use crate::summary::FunctionSummary;

/// Unified registry for function summaries with three-tier priority lookup.
///
/// When looking up a function, the registry checks each tier in order:
/// user specs > computed > defaults. The first match wins.
#[derive(Debug, Clone, Default)]
pub struct SummaryRegistry {
    /// User-edited YAML specs converted to summaries (highest priority).
    user_specs: BTreeMap<FunctionId, FunctionSummary>,
    /// Analysis-computed summaries.
    computed: BTreeMap<FunctionId, FunctionSummary>,
    /// Default shipped specs converted to summaries (lowest priority).
    defaults: BTreeMap<FunctionId, FunctionSummary>,
}

impl SummaryRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a summary with priority: user > computed > default.
    ///
    /// Returns the highest-priority summary for the given function, or `None`
    /// if no summary exists in any tier.
    #[must_use]
    pub fn get(&self, id: &FunctionId) -> Option<&FunctionSummary> {
        self.user_specs
            .get(id)
            .or_else(|| self.computed.get(id))
            .or_else(|| self.defaults.get(id))
    }

    /// Insert an analysis-computed summary into the computed tier.
    ///
    /// If a computed summary already exists for this function, it is replaced.
    pub fn insert_computed(&mut self, summary: FunctionSummary) {
        self.computed.insert(summary.function_id, summary);
    }

    /// Insert a user-spec summary into the user tier (highest priority).
    ///
    /// If a user summary already exists for this function, it is replaced.
    pub fn insert_user(&mut self, summary: FunctionSummary) {
        self.user_specs.insert(summary.function_id, summary);
    }

    /// Insert a default summary into the defaults tier (lowest priority).
    ///
    /// If a default summary already exists for this function, it is replaced.
    pub fn insert_default(&mut self, summary: FunctionSummary) {
        self.defaults.insert(summary.function_id, summary);
    }

    /// Load all exact-match specs from a [`SpecRegistry`] and convert them to
    /// summaries in the defaults tier.
    ///
    /// Each spec is converted via [`FunctionSummary::from_spec`] with a
    /// `FunctionId` derived from the spec's name. Pattern-based specs are
    /// skipped (they require runtime name resolution at call sites).
    pub fn load_specs(&mut self, registry: &SpecRegistry) {
        for spec in registry.iter() {
            let function_id = FunctionId::derive(spec.name.as_bytes());
            let summary = FunctionSummary::from_spec(spec, function_id);
            self.defaults.insert(function_id, summary);
        }
    }

    /// Load all exact-match specs from a [`SpecRegistry`] into the user tier.
    ///
    /// Same as [`load_specs`](Self::load_specs) but inserts into the
    /// highest-priority user tier.
    pub fn load_user_specs(&mut self, registry: &SpecRegistry) {
        for spec in registry.iter() {
            let function_id = FunctionId::derive(spec.name.as_bytes());
            let summary = FunctionSummary::from_spec(spec, function_id);
            self.user_specs.insert(function_id, summary);
        }
    }

    /// Get the total number of summaries across all tiers.
    ///
    /// Note: if the same function appears in multiple tiers, it is counted
    /// multiple times. Use [`unique_count`](Self::unique_count) for deduplicated count.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.user_specs.len() + self.computed.len() + self.defaults.len()
    }

    /// Get the number of unique functions with at least one summary.
    #[must_use]
    pub fn unique_count(&self) -> usize {
        let mut ids: std::collections::BTreeSet<FunctionId> = std::collections::BTreeSet::new();
        ids.extend(self.user_specs.keys());
        ids.extend(self.computed.keys());
        ids.extend(self.defaults.keys());
        ids.len()
    }

    /// Check if the registry is empty (no summaries in any tier).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.user_specs.is_empty() && self.computed.is_empty() && self.defaults.is_empty()
    }

    /// Iterate over all unique function IDs, yielding the highest-priority
    /// summary for each.
    pub fn iter(&self) -> impl Iterator<Item = (&FunctionId, &FunctionSummary)> {
        // Collect all unique IDs then look up with priority
        let mut ids: std::collections::BTreeSet<&FunctionId> = std::collections::BTreeSet::new();
        ids.extend(self.user_specs.keys());
        ids.extend(self.computed.keys());
        ids.extend(self.defaults.keys());

        ids.into_iter()
            .filter_map(move |id| self.get(id).map(|s| (id, s)))
    }

    /// Get the number of summaries in the computed tier.
    #[must_use]
    pub fn computed_count(&self) -> usize {
        self.computed.len()
    }

    /// Get the number of summaries in the user tier.
    #[must_use]
    pub fn user_count(&self) -> usize {
        self.user_specs.len()
    }

    /// Get the number of summaries in the defaults tier.
    #[must_use]
    pub fn defaults_count(&self) -> usize {
        self.defaults.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{FunctionSpec, Nullness, Pointer, ReturnSpec, Role};
    use crate::summary::{SummaryNullness, SummaryPrecision, SummaryRole, SummarySource};

    fn make_fid(name: &str) -> FunctionId {
        FunctionId::derive(name.as_bytes())
    }

    fn make_summary(name: &str, source: SummarySource) -> FunctionSummary {
        let mut s = FunctionSummary::default_for(make_fid(name));
        s.source = source;
        s
    }

    #[test]
    fn empty_registry() {
        let reg = SummaryRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.total_count(), 0);
        assert_eq!(reg.unique_count(), 0);
        assert!(reg.get(&make_fid("malloc")).is_none());
    }

    #[test]
    fn insert_and_get_computed() {
        let mut reg = SummaryRegistry::new();
        let summary = make_summary("test_fn", SummarySource::Analysis);
        let fid = summary.function_id;

        reg.insert_computed(summary);

        let found = reg.get(&fid).expect("should find computed summary");
        assert_eq!(found.function_id, fid);
        assert_eq!(found.source, SummarySource::Analysis);
        assert_eq!(reg.computed_count(), 1);
    }

    #[test]
    fn priority_user_over_computed() {
        let mut reg = SummaryRegistry::new();
        let fid = make_fid("func");

        let mut computed = FunctionSummary::default_for(fid);
        computed.source = SummarySource::Analysis;
        computed.pure = false;
        reg.insert_computed(computed);

        let mut user = FunctionSummary::default_for(fid);
        user.source = SummarySource::Spec;
        user.pure = true;
        reg.insert_user(user);

        let found = reg.get(&fid).expect("should find summary");
        assert_eq!(found.source, SummarySource::Spec);
        assert!(found.pure);
    }

    #[test]
    fn priority_computed_over_default() {
        let mut reg = SummaryRegistry::new();
        let fid = make_fid("func");

        let mut default_s = FunctionSummary::default_for(fid);
        default_s.source = SummarySource::Default;
        default_s.version = 1;
        reg.insert_default(default_s);

        let mut computed = FunctionSummary::default_for(fid);
        computed.source = SummarySource::Analysis;
        computed.version = 5;
        reg.insert_computed(computed);

        let found = reg.get(&fid).expect("should find summary");
        assert_eq!(found.source, SummarySource::Analysis);
        assert_eq!(found.version, 5);
    }

    #[test]
    fn priority_user_over_all() {
        let mut reg = SummaryRegistry::new();
        let fid = make_fid("func");

        reg.insert_default(make_summary("func", SummarySource::Default));
        reg.insert_computed(make_summary("func", SummarySource::Analysis));

        let mut user = FunctionSummary::default_for(fid);
        user.source = SummarySource::Spec;
        user.noreturn = true;
        reg.insert_user(user);

        let found = reg.get(&fid).expect("should find summary");
        assert_eq!(found.source, SummarySource::Spec);
        assert!(found.noreturn);
    }

    #[test]
    fn unique_count_deduplicates() {
        let mut reg = SummaryRegistry::new();

        reg.insert_default(make_summary("func", SummarySource::Default));
        reg.insert_computed(make_summary("func", SummarySource::Analysis));
        reg.insert_user(make_summary("func", SummarySource::Spec));

        // Same function in all three tiers
        assert_eq!(reg.total_count(), 3);
        assert_eq!(reg.unique_count(), 1);
    }

    #[test]
    fn load_specs_populates_defaults() {
        let mut spec_reg = SpecRegistry::new();
        let mut malloc_spec = FunctionSpec::new("malloc");
        malloc_spec.role = Some(Role::Allocator);
        malloc_spec.returns = Some(ReturnSpec {
            pointer: Some(Pointer::FreshHeap),
            nullness: Some(Nullness::MaybeNull),
            ..ReturnSpec::default()
        });
        spec_reg.add(malloc_spec).expect("add spec");

        let mut free_spec = FunctionSpec::new("free");
        free_spec.role = Some(Role::Deallocator);
        spec_reg.add(free_spec).expect("add spec");

        let mut reg = SummaryRegistry::new();
        reg.load_specs(&spec_reg);

        assert_eq!(reg.defaults_count(), 2);

        let malloc_id = make_fid("malloc");
        let found = reg.get(&malloc_id).expect("should find malloc");
        assert_eq!(found.role, Some(SummaryRole::Allocator));
        assert_eq!(found.source, SummarySource::Spec);
        assert_eq!(found.return_nullness, Some(SummaryNullness::MaybeNull));
        assert_eq!(found.precision, SummaryPrecision::Sound);

        let free_id = make_fid("free");
        let found = reg.get(&free_id).expect("should find free");
        assert_eq!(found.role, Some(SummaryRole::Deallocator));
    }

    #[test]
    fn computed_overrides_loaded_specs() {
        let mut spec_reg = SpecRegistry::new();
        let mut spec = FunctionSpec::new("my_func");
        spec.pure = Some(true);
        spec_reg.add(spec).expect("add spec");

        let mut reg = SummaryRegistry::new();
        reg.load_specs(&spec_reg);

        // Now insert a computed summary that overrides the default
        let fid = make_fid("my_func");
        let mut computed = FunctionSummary::default_for(fid);
        computed.source = SummarySource::Analysis;
        computed.pure = false;
        computed.version = 3;
        reg.insert_computed(computed);

        let found = reg.get(&fid).expect("should find summary");
        assert_eq!(found.source, SummarySource::Analysis);
        assert!(!found.pure);
        assert_eq!(found.version, 3);
    }

    #[test]
    fn iter_yields_highest_priority() {
        let mut reg = SummaryRegistry::new();

        // Two different functions
        reg.insert_default(make_summary("a", SummarySource::Default));
        reg.insert_computed(make_summary("b", SummarySource::Analysis));

        // Function "a" also has a computed version
        let mut computed_a = FunctionSummary::default_for(make_fid("a"));
        computed_a.source = SummarySource::Analysis;
        computed_a.version = 10;
        reg.insert_computed(computed_a);

        let results: Vec<_> = reg.iter().collect();
        assert_eq!(results.len(), 2);

        // Find "a" - should be computed (higher priority than default)
        let a_id = make_fid("a");
        let a_summary = results.iter().find(|(id, _)| **id == a_id).expect("find a");
        assert_eq!(a_summary.1.source, SummarySource::Analysis);
        assert_eq!(a_summary.1.version, 10);
    }

    #[test]
    fn load_user_specs_goes_to_user_tier() {
        let mut spec_reg = SpecRegistry::new();
        let mut spec = FunctionSpec::new("custom_alloc");
        spec.role = Some(Role::Allocator);
        spec_reg.add(spec).expect("add spec");

        let mut reg = SummaryRegistry::new();
        reg.load_user_specs(&spec_reg);

        assert_eq!(reg.user_count(), 1);
        assert_eq!(reg.defaults_count(), 0);

        let fid = make_fid("custom_alloc");
        let found = reg.get(&fid).expect("should find");
        assert_eq!(found.role, Some(SummaryRole::Allocator));
    }

    #[test]
    fn replace_computed_summary() {
        let mut reg = SummaryRegistry::new();
        let fid = make_fid("evolving_func");

        let mut v1 = FunctionSummary::default_for(fid);
        v1.source = SummarySource::Analysis;
        v1.version = 1;
        v1.pure = false;
        reg.insert_computed(v1);

        let mut v2 = FunctionSummary::default_for(fid);
        v2.source = SummarySource::Analysis;
        v2.version = 2;
        v2.pure = true;
        reg.insert_computed(v2);

        let found = reg.get(&fid).expect("should find");
        assert_eq!(found.version, 2);
        assert!(found.pure);
        assert_eq!(reg.computed_count(), 1);
    }
}
