//! Path-sensitive alias queries.
//!
//! When two pointers have different alias relationships on different
//! execution paths, this module determines the aggregate result by:
//! 1. Extracting dominating guards at the query point
//! 2. Enumerating feasible paths (up to a limit)
//! 3. Computing alias result on each path
//! 4. Combining: No if none alias, Must if all alias, May otherwise

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;
use saf_core::air::AirModule;
use saf_core::ids::{BlockId, FunctionId, LocId, ValueId};
use serde::{Deserialize, Serialize};

use crate::cfg::Cfg;
use crate::guard::{Guard, PathCondition, ValueLocationIndex};
use crate::z3_utils::dominator::{compute_dominators, extract_dominating_guards};
use crate::z3_utils::solver::{FeasibilityResult, PathFeasibilityChecker};

use super::location::Location;
use super::result::AliasResult;
use super::solver::PointsToMap;
use super::value_origin::{ValueOriginMap, filter_pts_for_path};

/// Configuration for path-sensitive alias queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathSensitiveConfig {
    /// Enable path-sensitive alias queries.
    #[serde(default)]
    pub enabled: bool,

    /// Maximum paths to enumerate per query (default: 16).
    #[serde(default = "default_max_paths")]
    pub max_paths: usize,

    /// Timeout in milliseconds for path feasibility checks (default: 500).
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_max_paths() -> usize {
    16
}

fn default_timeout_ms() -> u64 {
    500
}

impl Default for PathSensitiveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_paths: default_max_paths(),
            timeout_ms: default_timeout_ms(),
        }
    }
}

impl PathSensitiveConfig {
    /// Create a config with path-sensitive queries enabled.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Self::default()
        }
    }

    /// Set maximum paths to enumerate.
    #[must_use]
    pub fn with_max_paths(mut self, max_paths: usize) -> Self {
        self.max_paths = max_paths;
        self
    }

    /// Set timeout for path feasibility checks.
    #[must_use]
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// Diagnostics for path-sensitive alias queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathSensitiveDiagnostics {
    /// Total queries performed.
    pub queries: u32,
    /// Queries that used path-sensitive analysis.
    pub path_sensitive_queries: u32,
    /// Paths enumerated across all queries.
    pub paths_enumerated: u32,
    /// Paths found feasible.
    pub paths_feasible: u32,
    /// Paths found infeasible.
    pub paths_infeasible: u32,
    /// Queries that hit max_paths limit.
    pub queries_at_limit: u32,
    /// Z3 timeouts.
    pub timeouts: u32,
}

/// Path-sensitive alias checker.
///
/// Uses dominator-based guard extraction and Z3 feasibility checking
/// to compute path-sensitive alias results.
pub struct PathSensitiveAliasChecker<'a> {
    /// Points-to map.
    pts: &'a PointsToMap,
    /// Location storage.
    locations: &'a FxHashMap<LocId, Location>,
    /// AIR module for guard extraction.
    module: &'a AirModule,
    /// Configuration.
    config: PathSensitiveConfig,
    /// Value location index (built lazily).
    index: Option<ValueLocationIndex>,
    /// CFG cache per function.
    cfgs: BTreeMap<FunctionId, Cfg>,
    /// Dominator cache per function.
    dominators: BTreeMap<FunctionId, BTreeMap<BlockId, BlockId>>,
    /// Value origin map for path-filtered alias queries.
    origin_map: Option<ValueOriginMap>,
}

impl<'a> PathSensitiveAliasChecker<'a> {
    /// Create a new path-sensitive alias checker.
    #[must_use]
    pub fn new(
        pts: &'a PointsToMap,
        locations: &'a FxHashMap<LocId, Location>,
        module: &'a AirModule,
        config: PathSensitiveConfig,
    ) -> Self {
        Self {
            pts,
            locations,
            module,
            config,
            index: None,
            cfgs: BTreeMap::new(),
            dominators: BTreeMap::new(),
            origin_map: None,
        }
    }

    /// Create a new path-sensitive alias checker with a pre-built origin map.
    #[must_use]
    pub fn with_origin_map(
        pts: &'a PointsToMap,
        locations: &'a FxHashMap<LocId, Location>,
        module: &'a AirModule,
        config: PathSensitiveConfig,
        origin_map: ValueOriginMap,
    ) -> Self {
        Self {
            pts,
            locations,
            module,
            config,
            index: None,
            cfgs: BTreeMap::new(),
            dominators: BTreeMap::new(),
            origin_map: Some(origin_map),
        }
    }

    /// Check if two pointers may alias at a specific program point.
    ///
    /// This performs path-sensitive analysis using two strategies:
    ///
    /// **Strategy 1 (origin-based):** When a `ValueOriginMap` is available,
    /// directly checks if any overlapping PTS entries can coexist on the
    /// same path. This works even when the query point is in a merge block
    /// with no dominating guards.
    ///
    /// **Strategy 2 (dominator-based):** Falls back to extracting dominating
    /// guards at the query point, enumerating feasible paths, and computing
    /// alias per path.
    pub fn may_alias_at(
        &mut self,
        p: ValueId,
        q: ValueId,
        query_block: BlockId,
        query_function: FunctionId,
        diag: &mut PathSensitiveDiagnostics,
    ) -> AliasResult {
        diag.queries += 1;

        if !self.config.enabled {
            return self.flow_insensitive_alias(p, q);
        }

        // Strategy 1: Origin-based pair-wise feasibility check
        // This doesn't require dominating guards at the query point
        if let Some(result) = self.origin_based_alias(p, q, diag) {
            return result;
        }

        // Strategy 2: Dominator-based path enumeration (original approach)
        self.dominator_based_alias(p, q, query_block, query_function, diag)
    }

    /// Origin-based alias check: for each pair of overlapping PTS entries,
    /// check if the entries can coexist on the same execution path.
    ///
    /// If ALL overlapping entries require contradictory branch conditions,
    /// the alias is impossible → `NoAlias`.
    ///
    /// If on every feasible path combination the pointers have the same
    /// singleton target → `MustAlias`.
    fn origin_based_alias(
        &self,
        p: ValueId,
        q: ValueId,
        diag: &mut PathSensitiveDiagnostics,
    ) -> Option<AliasResult> {
        let origin_map = self.origin_map.as_ref()?;

        let p_pts = self.pts.get(&p)?;
        let q_pts = self.pts.get(&q)?;

        if p_pts.is_empty() || q_pts.is_empty() {
            return None; // Defer to flow-insensitive
        }

        // Check if BOTH values have origin tracking — if either doesn't,
        // we can't do origin-based filtering
        if !origin_map.has_origins(p) && !origin_map.has_origins(q) {
            return None;
        }

        diag.path_sensitive_queries += 1;

        // Collect all unique branch conditions from both values' origins
        let mut all_conditions: std::collections::BTreeSet<(ValueId, BlockId)> =
            std::collections::BTreeSet::new();

        for &loc in p_pts {
            if let Some(origin_sets) = origin_map.get_origins(p, loc) {
                for origins in origin_sets {
                    for o in origins {
                        all_conditions.insert((o.condition, o.block));
                    }
                }
            }
        }
        for &loc in q_pts {
            if let Some(origin_sets) = origin_map.get_origins(q, loc) {
                for origins in origin_sets {
                    for o in origins {
                        all_conditions.insert((o.condition, o.block));
                    }
                }
            }
        }

        if all_conditions.is_empty() {
            return None; // No conditions to reason about
        }

        let conditions: Vec<(ValueId, BlockId)> = all_conditions.into_iter().collect();
        let n = conditions.len().min(16); // Cap at 16 conditions = 65536 paths max

        // Enumerate all 2^n path combinations
        let total = 1usize << n;
        let limit = total.min(self.config.max_paths);

        let mut results = Vec::new();

        for i in 0..limit {
            // Build a path condition for this combination
            let guards: Vec<Guard> = (0..n)
                .map(|j| {
                    let (cond, block) = conditions[j];
                    Guard {
                        block,
                        function: FunctionId::new(0), // Placeholder
                        condition: cond,
                        branch_taken: ((i >> j) & 1) == 1,
                    }
                })
                .collect();
            let path = PathCondition { guards };

            diag.paths_enumerated += 1;

            // Filter both PTS for this path
            let p_filtered = filter_pts_for_path(p, p_pts, origin_map, &path);
            let q_filtered = filter_pts_for_path(q, q_pts, origin_map, &path);

            // Skip empty paths (no pointer assigned on this path)
            if p_filtered.is_empty() || q_filtered.is_empty() {
                diag.paths_infeasible += 1;
                continue;
            }

            diag.paths_feasible += 1;

            // Compute alias on filtered sets
            let alias = self.compute_alias_from_sets(&p_filtered, &q_filtered);
            results.push(alias);
        }

        if results.is_empty() {
            return None; // All paths had empty sets — defer
        }

        Some(self.combine_path_results(&results))
    }

    /// Dominator-based path-sensitive alias check (original Strategy 2).
    fn dominator_based_alias(
        &mut self,
        p: ValueId,
        q: ValueId,
        query_block: BlockId,
        query_function: FunctionId,
        diag: &mut PathSensitiveDiagnostics,
    ) -> AliasResult {
        // Build index lazily
        if self.index.is_none() {
            self.index = Some(ValueLocationIndex::build(self.module));
        }
        let index = self.index.as_ref().expect("index should be set");

        // Build CFG if not cached
        #[allow(clippy::map_entry)]
        if !self.cfgs.contains_key(&query_function) {
            let cfg = self.build_cfg_for_function(query_function);
            self.cfgs.insert(query_function, cfg);
        }

        let cfg = self.cfgs.get(&query_function).expect("cfg should be set");
        #[allow(clippy::map_entry)]
        if !self.dominators.contains_key(&query_function) {
            let doms = compute_dominators(cfg);
            self.dominators.insert(query_function, doms);
        }

        let cfg = self.cfgs.get(&query_function).expect("cfg should be set");
        let doms = self
            .dominators
            .get(&query_function)
            .expect("dominators should be set");

        let path_condition =
            extract_dominating_guards(query_block, query_function, cfg, doms, index);

        if path_condition.is_empty() {
            return self.flow_insensitive_alias(p, q);
        }

        diag.path_sensitive_queries += 1;

        let paths = self.enumerate_paths(&path_condition, diag);

        if paths.is_empty() {
            return self.flow_insensitive_alias(p, q);
        }

        let checker = PathFeasibilityChecker::new(self.config.timeout_ms);
        let mut results = Vec::new();

        for path in paths {
            let feasibility = checker.check_feasibility(&path, index);
            match feasibility {
                FeasibilityResult::Feasible => {
                    diag.paths_feasible += 1;
                    let alias = self.alias_under_path(p, q, &path);
                    results.push(alias);
                }
                FeasibilityResult::Infeasible => {
                    diag.paths_infeasible += 1;
                }
                FeasibilityResult::Unknown => {
                    diag.timeouts += 1;
                    let alias = self.alias_under_path(p, q, &path);
                    results.push(alias);
                }
            }
        }

        self.combine_path_results(&results)
    }

    /// Enumerate paths from guards (up to max_paths).
    ///
    /// Each guard has two branches (taken/not taken), so we enumerate
    /// all 2^n combinations up to the limit.
    fn enumerate_paths(
        &self,
        path_condition: &PathCondition,
        diag: &mut PathSensitiveDiagnostics,
    ) -> Vec<PathCondition> {
        let n = path_condition.guards.len();
        let total_paths = 1usize << n.min(20); // Avoid overflow for large n

        let limit = self.config.max_paths.min(total_paths);
        if total_paths > self.config.max_paths {
            diag.queries_at_limit += 1;
        }

        let mut paths = Vec::with_capacity(limit);

        for i in 0..limit {
            let mut guards = Vec::with_capacity(n);
            for (j, guard) in path_condition.guards.iter().enumerate() {
                let branch_taken = ((i >> j) & 1) == 1;
                guards.push(Guard {
                    block: guard.block,
                    function: guard.function,
                    condition: guard.condition,
                    branch_taken,
                });
            }
            paths.push(PathCondition { guards });
            diag.paths_enumerated += 1;
        }

        paths
    }

    /// Compute alias result under a specific path.
    ///
    /// When an origin map is available, filters the points-to sets of
    /// both pointers to only include entries consistent with the given
    /// path conditions. Then computes alias on the filtered sets.
    fn alias_under_path(&self, p: ValueId, q: ValueId, path: &PathCondition) -> AliasResult {
        let Some(origin_map) = &self.origin_map else {
            // No origin map → fall back to flow-insensitive
            return self.flow_insensitive_alias(p, q);
        };

        let p_pts = self.pts.get(&p);
        let q_pts = self.pts.get(&q);

        let (Some(p_set), Some(q_set)) = (p_pts, q_pts) else {
            return AliasResult::Unknown;
        };

        if p_set.is_empty() || q_set.is_empty() {
            return AliasResult::Unknown;
        }

        // Filter both sets using the origin map
        let p_filtered = filter_pts_for_path(p, p_set, origin_map, path);
        let q_filtered = filter_pts_for_path(q, q_set, origin_map, path);

        if p_filtered.is_empty() || q_filtered.is_empty() {
            return AliasResult::Unknown;
        }

        // Compute alias on filtered sets
        self.compute_alias_from_sets(&p_filtered, &q_filtered)
    }

    /// Compute alias result from two points-to sets.
    fn compute_alias_from_sets(
        &self,
        p_set: &std::collections::BTreeSet<LocId>,
        q_set: &std::collections::BTreeSet<LocId>,
    ) -> AliasResult {
        if p_set == q_set && p_set.len() == 1 {
            AliasResult::Must
        } else if p_set == q_set {
            AliasResult::May
        } else if p_set.is_disjoint(q_set) && !self.has_field_overlap(p_set, q_set) {
            AliasResult::No
        } else if p_set.is_subset(q_set) || q_set.is_subset(p_set) {
            AliasResult::Partial
        } else {
            AliasResult::May
        }
    }

    /// Combine alias results from multiple paths.
    #[allow(clippy::unused_self)]
    fn combine_path_results(&self, results: &[AliasResult]) -> AliasResult {
        if results.is_empty() {
            return AliasResult::Unknown;
        }

        // Check if all paths agree
        let first = results[0];
        let all_same = results.iter().all(|&r| r == first);

        if all_same {
            return first;
        }

        // Check special cases
        let any_must = results.iter().any(|&r| r == AliasResult::Must);
        let any_no = results.iter().any(|&r| r == AliasResult::No);
        let any_unknown = results.iter().any(|&r| r == AliasResult::Unknown);

        if any_unknown {
            // If any path is unknown, be conservative
            return AliasResult::May;
        }

        if any_must && any_no {
            // Some paths alias, some don't
            return AliasResult::May;
        }

        if any_must {
            // All aliasing paths are Must, some may be Partial/May
            return AliasResult::May;
        }

        if any_no {
            // Some paths don't alias, but we still have other results
            return AliasResult::May;
        }

        // Mixed Partial/May results
        AliasResult::May
    }

    /// Flow-insensitive alias query (fallback).
    fn flow_insensitive_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        let p_pts = self.pts.get(&p);
        let q_pts = self.pts.get(&q);

        match (p_pts, q_pts) {
            (None, _) | (_, None) => AliasResult::Unknown,
            (Some(p_set), Some(q_set)) => {
                if p_set.is_empty() || q_set.is_empty() {
                    AliasResult::Unknown
                } else if p_set == q_set && p_set.len() == 1 {
                    // Singleton sets pointing to the same location
                    AliasResult::Must
                } else if p_set == q_set {
                    // Non-singleton equal sets are MayAlias
                    AliasResult::May
                } else if p_set.is_disjoint(q_set) && !self.has_field_overlap(p_set, q_set) {
                    AliasResult::No
                } else if p_set.is_subset(q_set) || q_set.is_subset(p_set) {
                    AliasResult::Partial
                } else {
                    AliasResult::May
                }
            }
        }
    }

    /// Check if two location sets have field path overlap.
    fn has_field_overlap(
        &self,
        p_set: &std::collections::BTreeSet<LocId>,
        q_set: &std::collections::BTreeSet<LocId>,
    ) -> bool {
        for &p_loc in p_set {
            for &q_loc in q_set {
                if self.locations_overlap(p_loc, q_loc) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if two locations have a field path overlap.
    fn locations_overlap(&self, p: LocId, q: LocId) -> bool {
        let (Some(p_loc), Some(q_loc)) = (self.locations.get(&p), self.locations.get(&q)) else {
            return false;
        };

        if p_loc.obj != q_loc.obj {
            return false;
        }

        let p_path = &p_loc.path.steps;
        let q_path = &q_loc.path.steps;

        // One path is prefix of the other
        let (shorter, longer) = if p_path.len() <= q_path.len() {
            (p_path, q_path)
        } else {
            (q_path, p_path)
        };

        longer.starts_with(shorter)
    }

    /// Build CFG for a function.
    fn build_cfg_for_function(&self, function_id: FunctionId) -> Cfg {
        for func in &self.module.functions {
            if func.id == function_id && !func.is_declaration {
                return Cfg::build(func);
            }
        }
        // Return empty CFG if function not found
        Cfg {
            function: function_id,
            entry: BlockId::new(0),
            exits: std::collections::BTreeSet::new(),
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ObjId;
    use std::collections::BTreeSet;

    use crate::pta::config::FieldSensitivity;
    use crate::pta::location::{FieldPath, LocationFactory};

    fn make_module() -> AirModule {
        use saf_core::ids::ModuleId;
        AirModule::new(ModuleId::new(1))
    }

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 })
    }

    #[test]
    fn path_sensitive_config_default() {
        let config = PathSensitiveConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_paths, 16);
        assert_eq!(config.timeout_ms, 500);
    }

    #[test]
    fn path_sensitive_config_enabled() {
        let config = PathSensitiveConfig::enabled();
        assert!(config.enabled);
    }

    #[test]
    fn path_sensitive_config_builder() {
        let config = PathSensitiveConfig::enabled()
            .with_max_paths(32)
            .with_timeout(1000);
        assert!(config.enabled);
        assert_eq!(config.max_paths, 32);
        assert_eq!(config.timeout_ms, 1000);
    }

    #[test]
    fn flow_insensitive_alias_unknown() {
        let pts = PointsToMap::new();
        let module = make_module();
        let config = PathSensitiveConfig::default();
        let locations = FxHashMap::default();

        let checker = PathSensitiveAliasChecker::new(&pts, &locations, &module, config);

        let result = checker.flow_insensitive_alias(ValueId::new(1), ValueId::new(2));
        assert_eq!(result, AliasResult::Unknown);
    }

    #[test]
    fn flow_insensitive_alias_must() {
        let mut factory = make_factory();
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc);
        pts.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc);
        pts.insert(q, q_set);

        let module = make_module();
        let config = PathSensitiveConfig::default();
        let locations = factory.all_locations().clone();

        let checker = PathSensitiveAliasChecker::new(&pts, &locations, &module, config);

        let result = checker.flow_insensitive_alias(p, q);
        assert_eq!(result, AliasResult::Must);
    }

    #[test]
    fn flow_insensitive_alias_no() {
        let mut factory = make_factory();
        let loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc2 = factory.get_or_create(ObjId::new(200), FieldPath::empty());

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc1);
        pts.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc2);
        pts.insert(q, q_set);

        let module = make_module();
        let config = PathSensitiveConfig::default();
        let locations = factory.all_locations().clone();

        let checker = PathSensitiveAliasChecker::new(&pts, &locations, &module, config);

        let result = checker.flow_insensitive_alias(p, q);
        assert_eq!(result, AliasResult::No);
    }

    #[test]
    fn combine_results_all_same() {
        let pts = PointsToMap::new();
        let module = make_module();
        let config = PathSensitiveConfig::default();
        let locations = FxHashMap::default();

        let checker = PathSensitiveAliasChecker::new(&pts, &locations, &module, config);

        let results = vec![AliasResult::Must, AliasResult::Must, AliasResult::Must];
        assert_eq!(checker.combine_path_results(&results), AliasResult::Must);

        let results = vec![AliasResult::No, AliasResult::No];
        assert_eq!(checker.combine_path_results(&results), AliasResult::No);
    }

    #[test]
    fn combine_results_mixed() {
        let pts = PointsToMap::new();
        let module = make_module();
        let config = PathSensitiveConfig::default();
        let locations = FxHashMap::default();

        let checker = PathSensitiveAliasChecker::new(&pts, &locations, &module, config);

        let results = vec![AliasResult::Must, AliasResult::No];
        assert_eq!(checker.combine_path_results(&results), AliasResult::May);

        let results = vec![AliasResult::Must, AliasResult::May];
        assert_eq!(checker.combine_path_results(&results), AliasResult::May);
    }

    #[test]
    fn combine_results_empty() {
        let pts = PointsToMap::new();
        let module = make_module();
        let config = PathSensitiveConfig::default();
        let locations = FxHashMap::default();

        let checker = PathSensitiveAliasChecker::new(&pts, &locations, &module, config);

        let results: Vec<AliasResult> = vec![];
        assert_eq!(checker.combine_path_results(&results), AliasResult::Unknown);
    }

    #[test]
    fn enumerate_paths_basic() {
        let pts = PointsToMap::new();
        let module = make_module();
        let config = PathSensitiveConfig::enabled();
        let locations = FxHashMap::default();

        let checker = PathSensitiveAliasChecker::new(&pts, &locations, &module, config);

        let pc = PathCondition {
            guards: vec![
                Guard {
                    block: BlockId::new(1),
                    function: FunctionId::new(1),
                    condition: ValueId::new(100),
                    branch_taken: true,
                },
                Guard {
                    block: BlockId::new(2),
                    function: FunctionId::new(1),
                    condition: ValueId::new(101),
                    branch_taken: false,
                },
            ],
        };

        let mut diag = PathSensitiveDiagnostics::default();
        let paths = checker.enumerate_paths(&pc, &mut diag);

        // 2 guards = 4 paths
        assert_eq!(paths.len(), 4);
        assert_eq!(diag.paths_enumerated, 4);
    }

    #[test]
    fn enumerate_paths_respects_limit() {
        let pts = PointsToMap::new();
        let module = make_module();
        let config = PathSensitiveConfig::enabled().with_max_paths(2);
        let locations = FxHashMap::default();

        let checker = PathSensitiveAliasChecker::new(&pts, &locations, &module, config);

        let pc = PathCondition {
            guards: vec![
                Guard {
                    block: BlockId::new(1),
                    function: FunctionId::new(1),
                    condition: ValueId::new(100),
                    branch_taken: true,
                },
                Guard {
                    block: BlockId::new(2),
                    function: FunctionId::new(1),
                    condition: ValueId::new(101),
                    branch_taken: false,
                },
            ],
        };

        let mut diag = PathSensitiveDiagnostics::default();
        let paths = checker.enumerate_paths(&pc, &mut diag);

        assert_eq!(paths.len(), 2);
        assert_eq!(diag.queries_at_limit, 1);
    }

    #[test]
    fn diagnostics_default() {
        let diag = PathSensitiveDiagnostics::default();
        assert_eq!(diag.queries, 0);
        assert_eq!(diag.path_sensitive_queries, 0);
        assert_eq!(diag.paths_enumerated, 0);
    }
}
