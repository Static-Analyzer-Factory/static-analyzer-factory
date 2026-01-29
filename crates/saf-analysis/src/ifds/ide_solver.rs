//! IDE solver — two-phase algorithm (Sagiv/Reps/Horwitz, TCS'96).
//!
//! **Phase 1 (Jump Functions):** Extends the IFDS tabulation algorithm to
//! additionally compute a *jump function* `JumpFn(d1, n, d2)` — the composed
//! edge function from procedure entry fact `d1` to fact `d2` at program point
//! `n`. Re-propagates when the join of new and existing functions improves.
//!
//! **Phase 2 (Value Propagation):** Top-down BFS propagating actual lattice
//! values through computed jump functions. Seeds entry points with `top_value`,
//! then for each reachable `(d1, n, d2)` with jump function `f`:
//! `values[(n, d2)] = join(values[(n, d2)], f(values[(entry, d1)]))`.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::Operation;
use saf_core::ids::{FunctionId, InstId};

use crate::callgraph::CallGraph;
use crate::icfg::Icfg;

use super::config::IfdsConfig;
use super::edge_fn::BuiltinEdgeFn;
use super::icfg_index::IcfgIndex;
use super::ide_problem::IdeProblem;
use super::ide_result::{IdeDiagnostics, IdeResult};
use super::lattice::Lattice;
use super::result::{IfdsDiagnostics, IfdsResult};

/// A path edge in the exploded supergraph with an associated jump function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PathEdge<F: Ord + Clone> {
    func: FunctionId,
    d1: F,
    inst: InstId,
    d2: F,
}

/// Solve an IDE problem using the two-phase algorithm.
///
/// Phase 1: IFDS tabulation + jump function computation.
/// Phase 2: Top-down value propagation through jump functions.
// NOTE: This function implements the IDE algorithm (Sagiv/Reps/Horwitz TCS'96)
// with its two tightly-coupled phases. Splitting would obscure the algorithm's
// structure where Phase 2 directly consumes the jump function table from Phase 1.
#[allow(clippy::too_many_lines)]
pub fn solve_ide<P: IdeProblem>(
    problem: &P,
    icfg: &Icfg,
    _callgraph: &CallGraph,
    config: &IfdsConfig,
) -> IdeResult<P::Fact, P::Value> {
    let module = problem.module();
    let zero = problem.zero_value();

    // Build shared ICFG navigation index.
    let idx = IcfgIndex::build(module, icfg);

    // ── Phase 1: IFDS tabulation + jump function tracking ────────────────

    let mut path_edges_full: BTreeSet<PathEdge<P::Fact>> = BTreeSet::new();
    // NOTE: Summary edges use tuple keys (entry_fact, exit_fact) per function —
    // a type alias would not significantly improve readability here.
    #[allow(clippy::type_complexity)]
    let mut summary_edges: BTreeMap<FunctionId, BTreeSet<(P::Fact, P::Fact)>> = BTreeMap::new();
    let mut facts_at: BTreeMap<InstId, BTreeSet<P::Fact>> = BTreeMap::new();
    let mut worklist: BTreeSet<PathEdge<P::Fact>> = BTreeSet::new();

    // Jump function table: (d1, inst, d2) -> edge function
    // This tracks the composed edge function from (entry, d1) to (inst, d2).
    // NOTE: The tuple key (Fact, InstId, Fact) matches the IDE algorithm's definition.
    #[allow(clippy::type_complexity)]
    let mut jump_fn: BTreeMap<(P::Fact, InstId, P::Fact), BuiltinEdgeFn<P::Value>> =
        BTreeMap::new();

    // Summary edge functions: (func, d1, d3) -> edge function
    // The composed edge function for summary (entry d1 -> exit d3).
    // NOTE: The tuple key matches the IDE algorithm's summary edge definition.
    #[allow(clippy::type_complexity)]
    let mut summary_edge_fn: BTreeMap<(FunctionId, P::Fact, P::Fact), BuiltinEdgeFn<P::Value>> =
        BTreeMap::new();

    let seeds = problem.initial_seeds();

    let mut seeded_funcs: BTreeSet<FunctionId> = seeds.keys().copied().collect();
    for func in &module.functions {
        if !func.is_declaration {
            seeded_funcs.insert(func.id);
        }
    }

    // Seed: jump_fn[(zero, entry, zero)] = Identity
    for func_id in &seeded_funcs {
        if let Some(entry_inst) = idx.func_entry_inst.get(func_id) {
            let edge = PathEdge {
                func: *func_id,
                d1: zero.clone(),
                inst: *entry_inst,
                d2: zero.clone(),
            };
            if path_edges_full.insert(edge.clone()) {
                worklist.insert(edge);
            }
            facts_at
                .entry(*entry_inst)
                .or_default()
                .insert(zero.clone());
            jump_fn.insert(
                (zero.clone(), *entry_inst, zero.clone()),
                BuiltinEdgeFn::Identity,
            );

            if let Some(extra_facts) = seeds.get(func_id) {
                for fact in extra_facts {
                    let edge = PathEdge {
                        func: *func_id,
                        d1: fact.clone(),
                        inst: *entry_inst,
                        d2: fact.clone(),
                    };
                    if path_edges_full.insert(edge.clone()) {
                        worklist.insert(edge);
                    }
                    facts_at
                        .entry(*entry_inst)
                        .or_default()
                        .insert(fact.clone());
                    jump_fn.insert(
                        (fact.clone(), *entry_inst, fact.clone()),
                        BuiltinEdgeFn::Identity,
                    );
                }
            }
        }
    }

    let mut ifds_diagnostics = IfdsDiagnostics::default();
    let mut ide_diagnostics = IdeDiagnostics::default();

    // Main tabulation loop with jump function tracking.
    while let Some(edge) = worklist.pop_first() {
        ifds_diagnostics.iterations += 1;
        ifds_diagnostics.path_edges_explored += 1;

        if ifds_diagnostics.iterations > config.max_iterations {
            ifds_diagnostics.reached_limit = true;
            break;
        }

        let PathEdge {
            func,
            ref d1,
            inst: n,
            ref d2,
        } = edge;

        let instruction = match idx.inst_lookup.get(&n) {
            Some(i) => *i,
            None => continue,
        };

        let is_call = matches!(
            instruction.op,
            Operation::CallDirect { .. } | Operation::CallIndirect { .. }
        );
        let is_exit = idx
            .func_exit_insts
            .get(&func)
            .is_some_and(|exits| exits.contains(&n));
        let has_defined_callees = is_call && idx.call_to_callees.contains_key(&n);

        // Get the current jump function for this path edge.
        let current_jf = jump_fn
            .get(&(d1.clone(), n, d2.clone()))
            .cloned()
            .unwrap_or(BuiltinEdgeFn::AllTop);

        if is_call && has_defined_callees {
            // ── Call node with defined callees ───────────────────────

            // 1. Call-to-return
            let c2r_facts = problem.call_to_return_flow(instruction, d2);
            if let Some(return_insts) = idx.call_site_return_inst.get(&n) {
                for ret_inst in return_insts {
                    for d3 in &c2r_facts {
                        let ef = problem.call_to_return_edge_fn(instruction, d2, d3);
                        let new_jf = ef.compose_with(&current_jf);
                        ide_propagate(
                            func,
                            d1,
                            *ret_inst,
                            d3,
                            &new_jf,
                            &mut path_edges_full,
                            &mut worklist,
                            &mut facts_at,
                            &mut jump_fn,
                            config,
                            &mut ifds_diagnostics,
                            &mut ide_diagnostics,
                        );
                    }
                }
            }

            // 2. Call flow: enter callee
            if let Some(callees) = idx.call_to_callees.get(&n) {
                for callee_id in callees {
                    if let Some(callee_func) = module.function(*callee_id) {
                        let call_facts = problem.call_flow(instruction, callee_func, d2);
                        if let Some(callee_entry) = idx.func_entry_inst.get(callee_id) {
                            for d3 in &call_facts {
                                // At callee entry, jump function is Identity —
                                // the caller's call edge function is stored separately
                                // and composed when applying summaries.
                                ide_propagate(
                                    *callee_id,
                                    d3,
                                    *callee_entry,
                                    d3,
                                    &BuiltinEdgeFn::Identity,
                                    &mut path_edges_full,
                                    &mut worklist,
                                    &mut facts_at,
                                    &mut jump_fn,
                                    config,
                                    &mut ifds_diagnostics,
                                    &mut ide_diagnostics,
                                );
                            }
                        }
                    }

                    // 3. Apply existing summaries.
                    if let Some(func_summaries) = summary_edges.get(callee_id) {
                        if let Some(callee_func) = module.function(*callee_id) {
                            let call_facts = problem.call_flow(instruction, callee_func, d2);
                            for d3 in &call_facts {
                                for (sum_d1, sum_d2) in func_summaries {
                                    if sum_d1 == d3 {
                                        if let Some(return_insts) =
                                            idx.call_site_return_inst.get(&n)
                                        {
                                            // Compose: summary_ef . call_ef . caller_jf
                                            let call_ef = problem.call_edge_fn(
                                                instruction,
                                                callee_func,
                                                d2,
                                                d3,
                                            );
                                            let sum_ef = summary_edge_fn
                                                .get(&(*callee_id, sum_d1.clone(), sum_d2.clone()))
                                                .cloned()
                                                .unwrap_or(BuiltinEdgeFn::Identity);
                                            let new_jf = sum_ef
                                                .compose_with(&call_ef.compose_with(&current_jf));

                                            for ret_inst in return_insts {
                                                ide_propagate(
                                                    func,
                                                    d1,
                                                    *ret_inst,
                                                    sum_d2,
                                                    &new_jf,
                                                    &mut path_edges_full,
                                                    &mut worklist,
                                                    &mut facts_at,
                                                    &mut jump_fn,
                                                    config,
                                                    &mut ifds_diagnostics,
                                                    &mut ide_diagnostics,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if is_call && !has_defined_callees {
            // ── External call ────────────────────────────────────────
            // For external calls (no callee body), we use `normal_flow` and
            // `normal_edge_fn` for both fact registration and propagation.
            // We intentionally do NOT use `call_to_return_flow` here because
            // its kill semantics assume a through-callee path exists to
            // recover killed facts. With no callee body, killed facts would
            // be permanently lost.
            let new_facts = problem.normal_flow(instruction, d2);

            // Register genuinely new facts at the call site. Skip d3 == d2
            // because that fact already exists at `n` (it's the current path
            // edge). Self-propagating with the post-call edge function would
            // join pre-call and post-call values, destroying information.
            for d3 in &new_facts {
                if d3 != d2 {
                    let ef = problem.normal_edge_fn(instruction, d2, d3);
                    let new_jf = ef.compose_with(&current_jf);
                    ide_propagate(
                        func,
                        d1,
                        n,
                        d3,
                        &new_jf,
                        &mut path_edges_full,
                        &mut worklist,
                        &mut facts_at,
                        &mut jump_fn,
                        config,
                        &mut ifds_diagnostics,
                        &mut ide_diagnostics,
                    );
                }
            }

            let successors = idx.successor_instructions(n, func, instruction, icfg);

            for succ in &successors {
                for d3 in &new_facts {
                    let ef = problem.normal_edge_fn(instruction, d2, d3);
                    let new_jf = ef.compose_with(&current_jf);
                    ide_propagate(
                        func,
                        d1,
                        *succ,
                        d3,
                        &new_jf,
                        &mut path_edges_full,
                        &mut worklist,
                        &mut facts_at,
                        &mut jump_fn,
                        config,
                        &mut ifds_diagnostics,
                        &mut ide_diagnostics,
                    );
                }
            }
        } else if is_exit {
            // ── Exit node ────────────────────────────────────────────
            let entry_fact = d1;
            let exit_fact = d2;

            if let Some(call_sites) = idx.callee_to_call_sites.get(&func) {
                for call_site_id in call_sites {
                    let call_inst = match idx.inst_lookup.get(call_site_id) {
                        Some(i) => *i,
                        None => continue,
                    };
                    let Some(exit_fn) = module.function(func) else {
                        continue;
                    };

                    let return_facts =
                        problem.return_flow(call_inst, exit_fn, instruction, exit_fact);

                    for d3 in &return_facts {
                        let ret_ef =
                            problem.return_edge_fn(call_inst, exit_fn, instruction, exit_fact, d3);

                        // Summary edge function = ret_ef . jump_fn(entry->exit)
                        let sum_ef = ret_ef.compose_with(&current_jf);

                        let is_new = summary_edges
                            .entry(func)
                            .or_default()
                            .insert((entry_fact.clone(), d3.clone()));

                        // Update summary edge function (join with existing).
                        let sum_key = (func, entry_fact.clone(), d3.clone());
                        let existing_sum = summary_edge_fn
                            .get(&sum_key)
                            .cloned()
                            .unwrap_or(BuiltinEdgeFn::AllTop);
                        let joined_sum = existing_sum.join_with(&sum_ef);
                        if !joined_sum.equal_to(&existing_sum) || is_new {
                            summary_edge_fn.insert(sum_key, joined_sum);
                        }

                        if is_new {
                            ifds_diagnostics.summary_edges_created += 1;
                        }

                        // Propagate to callers.
                        let call_site_fn = match idx.inst_to_func.get(call_site_id) {
                            Some(f) => *f,
                            None => continue,
                        };

                        let matching_edges: Vec<_> = path_edges_full
                            .iter()
                            .filter(|pe| pe.inst == *call_site_id && pe.func == call_site_fn)
                            .cloned()
                            .collect();

                        for caller_edge in &matching_edges {
                            let cf = problem.call_flow(call_inst, exit_fn, &caller_edge.d2);
                            if cf.contains(entry_fact) {
                                if let Some(return_insts) =
                                    idx.call_site_return_inst.get(call_site_id)
                                {
                                    let caller_jf = jump_fn
                                        .get(&(
                                            caller_edge.d1.clone(),
                                            *call_site_id,
                                            caller_edge.d2.clone(),
                                        ))
                                        .cloned()
                                        .unwrap_or(BuiltinEdgeFn::AllTop);

                                    let call_ef = problem.call_edge_fn(
                                        call_inst,
                                        exit_fn,
                                        &caller_edge.d2,
                                        entry_fact,
                                    );
                                    let s_ef = summary_edge_fn
                                        .get(&(func, entry_fact.clone(), d3.clone()))
                                        .cloned()
                                        .unwrap_or(BuiltinEdgeFn::Identity);
                                    let new_jf =
                                        s_ef.compose_with(&call_ef.compose_with(&caller_jf));

                                    for ret_inst in return_insts {
                                        ide_propagate(
                                            call_site_fn,
                                            &caller_edge.d1,
                                            *ret_inst,
                                            d3,
                                            &new_jf,
                                            &mut path_edges_full,
                                            &mut worklist,
                                            &mut facts_at,
                                            &mut jump_fn,
                                            config,
                                            &mut ifds_diagnostics,
                                            &mut ide_diagnostics,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // ── Normal node ──────────────────────────────────────────
            let new_facts = problem.normal_flow(instruction, d2);

            let successors = idx.successor_instructions(n, func, instruction, icfg);

            for succ in &successors {
                for d3 in &new_facts {
                    let ef = problem.normal_edge_fn(instruction, d2, d3);
                    let new_jf = ef.compose_with(&current_jf);
                    ide_propagate(
                        func,
                        d1,
                        *succ,
                        d3,
                        &new_jf,
                        &mut path_edges_full,
                        &mut worklist,
                        &mut facts_at,
                        &mut jump_fn,
                        config,
                        &mut ifds_diagnostics,
                        &mut ide_diagnostics,
                    );
                }
            }
        }
    }

    // Compute peak facts.
    for set in facts_at.values() {
        if set.len() > ifds_diagnostics.facts_at_peak {
            ifds_diagnostics.facts_at_peak = set.len();
        }
    }

    ide_diagnostics.jump_fn_entries = jump_fn.len();

    let ifds_result = IfdsResult {
        facts: facts_at,
        summaries: summary_edges,
        diagnostics: ifds_diagnostics,
    };

    // ── Phase 2: Value Propagation ───────────────────────────────────────
    //
    // TCS'96 Phase 2: compute MFP values top-down.
    //   - For entry fact d1: val[(entry, d1)] = top (all inputs possible)
    //   - For all other (n, d2): val[(n, d2)] = bottom (no info yet)
    //   - For each (d1, n, d2) with JumpFn f:
    //       MFP(n, d2) = join(MFP(n, d2), f(val[(entry, d1)]))

    let top_val = problem.top_value();
    let bottom_val = problem.bottom_value();
    let mut values: BTreeMap<InstId, BTreeMap<P::Fact, P::Value>> = BTreeMap::new();

    // Seed entry points: val[(entry, d1)] = top for each entry fact d1.
    let mut entry_values: BTreeMap<(FunctionId, P::Fact), P::Value> = BTreeMap::new();
    for func_id in &seeded_funcs {
        if let Some(entry_inst) = idx.func_entry_inst.get(func_id) {
            if let Some(facts) = ifds_result.facts.get(entry_inst) {
                for fact in facts {
                    entry_values.insert((*func_id, fact.clone()), top_val.clone());
                }
            }
        }
    }

    // Compute MFP values from jump functions.
    for ((d1, inst, d2), jf) in &jump_fn {
        // Find which function this instruction belongs to.
        let Some(&func_id) = idx.inst_to_func.get(inst) else {
            continue;
        };

        // Get entry value for d1 in this function.
        let entry_val = entry_values
            .get(&(func_id, d1.clone()))
            .cloned()
            .unwrap_or_else(|| bottom_val.clone());

        // Skip if entry value is bottom (no information to propagate).
        if entry_val == bottom_val {
            continue;
        }

        // Apply jump function: f(val[(entry, d1)]).
        let new_val = jf.compute_target(entry_val);
        ide_diagnostics.value_propagations += 1;

        // Join into MFP: values[(inst, d2)] = join(current, new_val).
        let current_val = values
            .entry(*inst)
            .or_default()
            .entry(d2.clone())
            .or_insert_with(|| bottom_val.clone());
        let joined = current_val.join(&new_val);
        *current_val = joined;
    }

    // Also set entry values in the values map.
    for ((func_id, fact), val) in &entry_values {
        if let Some(&entry_inst) = idx.func_entry_inst.get(func_id) {
            let current = values
                .entry(entry_inst)
                .or_default()
                .entry(fact.clone())
                .or_insert_with(|| bottom_val.clone());
            let joined = current.join(val);
            *current = joined;
        }
    }

    IdeResult {
        values,
        ifds_result,
        diagnostics: ide_diagnostics,
    }
}

/// Propagate a path edge with jump function tracking.
///
/// Adds the edge to the worklist if unseen, or re-adds if the join of the
/// new jump function with the existing one produces an improvement.
// NOTE: This function requires many parameters because it operates on shared
// solver state (path edges, worklist, facts, jump functions, diagnostics) that
// cannot be bundled into a struct without introducing borrow checker conflicts.
#[allow(clippy::too_many_arguments)]
fn ide_propagate<F: Ord + Clone + std::fmt::Debug, V: Lattice>(
    func: FunctionId,
    d1: &F,
    inst: InstId,
    d2: &F,
    new_jf: &BuiltinEdgeFn<V>,
    path_edges: &mut BTreeSet<PathEdge<F>>,
    worklist: &mut BTreeSet<PathEdge<F>>,
    facts_at: &mut BTreeMap<InstId, BTreeSet<F>>,
    jump_fn: &mut BTreeMap<(F, InstId, F), BuiltinEdgeFn<V>>,
    config: &IfdsConfig,
    ifds_diag: &mut IfdsDiagnostics,
    ide_diag: &mut IdeDiagnostics,
) {
    let current_count = facts_at.get(&inst).map_or(0, BTreeSet::len);
    if current_count >= config.max_facts_per_point {
        ifds_diag.reached_limit = true;
        return;
    }

    let jf_key = (d1.clone(), inst, d2.clone());
    let existing = jump_fn.get(&jf_key).cloned();

    let should_propagate = match &existing {
        None => {
            // New edge — insert jump function and propagate.
            jump_fn.insert(jf_key, new_jf.clone());
            ide_diag.jump_fn_updates += 1;
            true
        }
        Some(old_jf) => {
            // Existing edge — join functions and check for improvement.
            let joined = old_jf.join_with(new_jf);
            if joined == *old_jf {
                false
            } else {
                jump_fn.insert(jf_key, joined);
                ide_diag.jump_fn_updates += 1;
                true
            }
        }
    };

    if should_propagate {
        let edge = PathEdge {
            func,
            d1: d1.clone(),
            inst,
            d2: d2.clone(),
        };
        path_edges.insert(edge.clone());
        worklist.insert(edge);
        facts_at.entry(inst).or_default().insert(d2.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callgraph::CallGraph;
    use crate::icfg::Icfg;
    use crate::ifds::config::IfdsConfig;
    use crate::ifds::edge_fn::BuiltinEdgeFn;
    use crate::ifds::ide_problem::IdeProblem;
    use crate::ifds::lattice::Lattice;
    use crate::ifds::problem::IfdsProblem;
    use saf_core::air::{AirBlock, AirFunction, AirModule, AirParam, Instruction, Operation};
    use saf_core::ids::{BlockId, ModuleId, ValueId};

    // ── Test lattice: flat {A, B, C} + Top + Bottom ─────────────────────

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    #[allow(dead_code)]
    enum FlatVal {
        Bottom,
        A,
        B,
        C,
        Top,
    }

    impl Lattice for FlatVal {
        fn top() -> Self {
            FlatVal::Top
        }
        fn bottom() -> Self {
            FlatVal::Bottom
        }
        fn join(&self, other: &Self) -> Self {
            if self == other {
                return self.clone();
            }
            match (self, other) {
                (FlatVal::Bottom, x) | (x, FlatVal::Bottom) => x.clone(),
                (FlatVal::Top, _) | (_, FlatVal::Top) => FlatVal::Top,
                _ => FlatVal::Top,
            }
        }
        fn meet(&self, other: &Self) -> Self {
            if self == other {
                return self.clone();
            }
            match (self, other) {
                (FlatVal::Top, x) | (x, FlatVal::Top) => x.clone(),
                (FlatVal::Bottom, _) | (_, FlatVal::Bottom) => FlatVal::Bottom,
                _ => FlatVal::Bottom,
            }
        }
        fn leq(&self, other: &Self) -> bool {
            self.join(other) == *other
        }
    }

    // ── Test fact type ──────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TestFact {
        Zero,
        Tracked(ValueId),
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    fn make_module(functions: Vec<AirFunction>) -> AirModule {
        AirModule {
            id: ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions,
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    fn make_function(id: u128, name: &str, blocks: Vec<AirBlock>) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: Vec::new(),
            blocks,
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_declaration(id: u128, name: &str) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_block(id: u128, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id: BlockId::new(id),
            label: None,
            instructions,
        }
    }

    // ── Constant-gen IDE problem ────────────────────────────────────────
    //
    // At a call to "source" (external), generates Tracked(dst) with value A.
    // Normal flow propagates tracked facts with Identity.

    struct ConstGenIdeProblem {
        module: AirModule,
        source_func: FunctionId,
    }

    impl IfdsProblem for ConstGenIdeProblem {
        type Fact = TestFact;

        fn zero_value(&self) -> Self::Fact {
            TestFact::Zero
        }

        fn module(&self) -> &AirModule {
            &self.module
        }

        fn initial_seeds(&self) -> BTreeMap<FunctionId, BTreeSet<Self::Fact>> {
            BTreeMap::new()
        }

        fn normal_flow(&self, inst: &Instruction, fact: &Self::Fact) -> BTreeSet<Self::Fact> {
            if let Operation::CallDirect { callee } = &inst.op {
                if *callee == self.source_func {
                    if let TestFact::Zero = fact {
                        let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                        if let Some(dst) = inst.dst {
                            result.insert(TestFact::Tracked(dst));
                        }
                        return result;
                    }
                }
            }
            // Copy propagation
            if let Operation::Copy = &inst.op {
                if let TestFact::Tracked(v) = fact {
                    if inst.operands.contains(v) {
                        let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                        if let Some(dst) = inst.dst {
                            result.insert(TestFact::Tracked(dst));
                        }
                        return result;
                    }
                }
            }
            [fact.clone()].into_iter().collect()
        }

        fn call_flow(
            &self,
            call_site: &Instruction,
            callee: &AirFunction,
            fact: &Self::Fact,
        ) -> BTreeSet<Self::Fact> {
            if let TestFact::Tracked(v) = fact {
                for (i, op) in call_site.operands.iter().enumerate() {
                    if op == v {
                        if let Some(param) = callee.params.get(i) {
                            return [TestFact::Tracked(param.id)].into_iter().collect();
                        }
                    }
                }
            }
            if let TestFact::Zero = fact {
                return [TestFact::Zero].into_iter().collect();
            }
            BTreeSet::new()
        }

        fn return_flow(
            &self,
            call_site: &Instruction,
            _callee: &AirFunction,
            exit_inst: &Instruction,
            fact: &Self::Fact,
        ) -> BTreeSet<Self::Fact> {
            if let TestFact::Tracked(v) = fact {
                if exit_inst.operands.contains(v) {
                    if let Some(dst) = call_site.dst {
                        return [TestFact::Tracked(dst)].into_iter().collect();
                    }
                }
            }
            BTreeSet::new()
        }

        fn call_to_return_flow(
            &self,
            call_site: &Instruction,
            fact: &Self::Fact,
        ) -> BTreeSet<Self::Fact> {
            if let TestFact::Tracked(v) = fact {
                if call_site.operands.contains(v) {
                    return BTreeSet::new();
                }
            }
            [fact.clone()].into_iter().collect()
        }
    }

    impl IdeProblem for ConstGenIdeProblem {
        type Value = FlatVal;

        fn normal_edge_fn(
            &self,
            inst: &Instruction,
            src_fact: &Self::Fact,
            succ_fact: &Self::Fact,
        ) -> BuiltinEdgeFn<Self::Value> {
            // When generating a new tracked fact at a source call:
            // zero -> Tracked(dst): Constant(A)
            if let Operation::CallDirect { callee } = &inst.op {
                if *callee == self.source_func {
                    if matches!(src_fact, TestFact::Zero)
                        && matches!(succ_fact, TestFact::Tracked(_))
                    {
                        return BuiltinEdgeFn::Constant(FlatVal::A);
                    }
                }
            }
            BuiltinEdgeFn::Identity
        }

        fn call_edge_fn(
            &self,
            _call_site: &Instruction,
            _callee: &AirFunction,
            _src_fact: &Self::Fact,
            _dest_fact: &Self::Fact,
        ) -> BuiltinEdgeFn<Self::Value> {
            BuiltinEdgeFn::Identity
        }

        fn return_edge_fn(
            &self,
            _call_site: &Instruction,
            _callee: &AirFunction,
            _exit_inst: &Instruction,
            _exit_fact: &Self::Fact,
            _ret_fact: &Self::Fact,
        ) -> BuiltinEdgeFn<Self::Value> {
            BuiltinEdgeFn::Identity
        }

        fn call_to_return_edge_fn(
            &self,
            _call_site: &Instruction,
            _src_fact: &Self::Fact,
            _ret_fact: &Self::Fact,
        ) -> BuiltinEdgeFn<Self::Value> {
            BuiltinEdgeFn::Identity
        }

        fn top_value(&self) -> Self::Value {
            FlatVal::Top
        }

        fn bottom_value(&self) -> Self::Value {
            FlatVal::Bottom
        }
    }

    // ── Tests ───────────────────────────────────────────────────────────

    #[test]
    fn ide_linear_const_gen() {
        // source() generates Tracked(dst) with value A.
        let source = make_declaration(2, "source");
        let module = make_module(vec![
            make_function(
                1,
                "main",
                vec![make_block(
                    1,
                    vec![
                        Instruction::new(
                            InstId::new(100),
                            Operation::CallDirect {
                                callee: FunctionId::new(2),
                            },
                        )
                        .with_dst(ValueId::new(10)),
                        Instruction::new(InstId::new(101), Operation::Ret),
                    ],
                )],
            ),
            source,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = ConstGenIdeProblem {
            module,
            source_func: FunctionId::new(2),
        };
        let config = IfdsConfig::default();

        let result = solve_ide(&problem, &icfg, &cg, &config);

        // Tracked(10) should exist and have value A at ret.
        assert!(result.holds_at(InstId::new(101), &TestFact::Tracked(ValueId::new(10))));
        let val = result.value_at(InstId::new(101), &TestFact::Tracked(ValueId::new(10)));
        assert!(val.is_some(), "should have a value for Tracked(10) at ret");
        // The value should be A (from the Constant(A) edge function).
        assert_eq!(*val.unwrap(), FlatVal::A);
    }

    #[test]
    fn ide_interprocedural_propagation() {
        // source() -> helper(x) -> ret: value should propagate through.
        let source = make_declaration(3, "source");
        let helper = AirFunction {
            id: FunctionId::new(2),
            name: "helper".to_string(),
            params: vec![AirParam::new(ValueId::new(30), 0)],
            blocks: vec![make_block(
                2,
                vec![
                    Instruction::new(InstId::new(200), Operation::Ret)
                        .with_operands(vec![ValueId::new(30)]),
                ],
            )],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(3),
                        },
                    )
                    .with_dst(ValueId::new(10)),
                    Instruction::new(
                        InstId::new(101),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    )
                    .with_operands(vec![ValueId::new(10)])
                    .with_dst(ValueId::new(11)),
                    Instruction::new(InstId::new(102), Operation::Ret),
                ],
            )],
        );
        let module = make_module(vec![main, helper, source]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = ConstGenIdeProblem {
            module,
            source_func: FunctionId::new(3),
        };
        let config = IfdsConfig::default();

        let result = solve_ide(&problem, &icfg, &cg, &config);

        // Tracked(11) should propagate through helper and have value A.
        assert!(result.holds_at(InstId::new(102), &TestFact::Tracked(ValueId::new(11))));
    }

    #[test]
    fn ide_export_is_deterministic() {
        let source = make_declaration(2, "source");
        let module = make_module(vec![
            make_function(
                1,
                "main",
                vec![make_block(
                    1,
                    vec![
                        Instruction::new(
                            InstId::new(100),
                            Operation::CallDirect {
                                callee: FunctionId::new(2),
                            },
                        )
                        .with_dst(ValueId::new(10)),
                        Instruction::new(InstId::new(101), Operation::Ret),
                    ],
                )],
            ),
            source,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = ConstGenIdeProblem {
            module: module.clone(),
            source_func: FunctionId::new(2),
        };
        let config = IfdsConfig::default();
        let result1 = solve_ide(&problem, &icfg, &cg, &config);

        let problem2 = ConstGenIdeProblem {
            module,
            source_func: FunctionId::new(2),
        };
        let result2 = solve_ide(&problem2, &icfg, &cg, &config);

        let json1 = serde_json::to_string(&result1.export()).unwrap();
        let json2 = serde_json::to_string(&result2.export()).unwrap();
        assert_eq!(json1, json2, "IDE export must be deterministic");
    }

    #[test]
    fn ide_diamond_cfg_merge() {
        // entry -> (then, else) -> join -> ret
        // Source in entry. Value should merge at join.
        let source = make_declaration(2, "source");
        let module = make_module(vec![
            make_function(
                1,
                "main",
                vec![
                    make_block(
                        1,
                        vec![
                            Instruction::new(
                                InstId::new(100),
                                Operation::CallDirect {
                                    callee: FunctionId::new(2),
                                },
                            )
                            .with_dst(ValueId::new(10)),
                            Instruction::new(
                                InstId::new(101),
                                Operation::CondBr {
                                    then_target: BlockId::new(2),
                                    else_target: BlockId::new(3),
                                },
                            ),
                        ],
                    ),
                    make_block(
                        2,
                        vec![Instruction::new(
                            InstId::new(200),
                            Operation::Br {
                                target: BlockId::new(4),
                            },
                        )],
                    ),
                    make_block(
                        3,
                        vec![Instruction::new(
                            InstId::new(300),
                            Operation::Br {
                                target: BlockId::new(4),
                            },
                        )],
                    ),
                    make_block(4, vec![Instruction::new(InstId::new(400), Operation::Ret)]),
                ],
            ),
            source,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = ConstGenIdeProblem {
            module,
            source_func: FunctionId::new(2),
        };
        let config = IfdsConfig::default();

        let result = solve_ide(&problem, &icfg, &cg, &config);

        // The tracked fact should reach the join point.
        assert!(result.holds_at(InstId::new(400), &TestFact::Tracked(ValueId::new(10))));
        // Value should be A (merged from both branches with Identity).
        let val = result.value_at(InstId::new(400), &TestFact::Tracked(ValueId::new(10)));
        assert!(val.is_some());
        assert_eq!(*val.unwrap(), FlatVal::A);
    }

    #[test]
    fn ide_diagnostics_tracked() {
        let source = make_declaration(2, "source");
        let module = make_module(vec![
            make_function(
                1,
                "main",
                vec![make_block(
                    1,
                    vec![
                        Instruction::new(
                            InstId::new(100),
                            Operation::CallDirect {
                                callee: FunctionId::new(2),
                            },
                        )
                        .with_dst(ValueId::new(10)),
                        Instruction::new(InstId::new(101), Operation::Ret),
                    ],
                )],
            ),
            source,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = ConstGenIdeProblem {
            module,
            source_func: FunctionId::new(2),
        };
        let config = IfdsConfig::default();

        let result = solve_ide(&problem, &icfg, &cg, &config);

        assert!(result.diagnostics.jump_fn_entries > 0);
        assert!(result.diagnostics.jump_fn_updates > 0);
        assert!(result.diagnostics.value_propagations > 0);
    }
}
