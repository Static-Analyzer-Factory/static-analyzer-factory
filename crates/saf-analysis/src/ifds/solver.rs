//! IFDS tabulation solver.
//!
//! Implements the tabulation algorithm from Reps, Horwitz, Sagiv (POPL'95).
//! Solves interprocedural data-flow problems expressed as IFDS problems by
//! reducing them to graph reachability on an "exploded supergraph."

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::Operation;
use saf_core::ids::{FunctionId, InstId};

use crate::callgraph::CallGraph;
use crate::icfg::Icfg;

use super::config::IfdsConfig;
use super::icfg_index::IcfgIndex;
use super::problem::IfdsProblem;
use super::result::{IfdsDiagnostics, IfdsResult};

/// A path edge in the exploded supergraph: fact `d1` at function entry of
/// `func` reaches fact `d2` at instruction `inst`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PathEdge<F: Ord + Clone> {
    /// Function containing the program point.
    func: FunctionId,
    /// Fact at the function entry.
    d1: F,
    /// Current program point (instruction ID).
    inst: InstId,
    /// Fact at the current program point.
    d2: F,
}

/// Solve an IFDS problem using the tabulation algorithm.
///
/// Takes a problem definition, pre-built ICFG, call graph, and configuration.
/// Returns the computed data-flow facts at each program point.
// NOTE: This function implements the IFDS tabulation algorithm (Reps/Horwitz/Sagiv
// POPL'95) as a single cohesive unit. The algorithm's phases (initialization,
// worklist processing, path/summary edge propagation) are tightly coupled and
// splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines)]
pub fn solve_ifds<P: IfdsProblem>(
    problem: &P,
    icfg: &Icfg,
    _callgraph: &CallGraph,
    config: &IfdsConfig,
) -> IfdsResult<P::Fact> {
    let module = problem.module();
    let zero = problem.zero_value();

    // Build shared ICFG navigation index.
    let idx = IcfgIndex::build(module, icfg);

    // ── Tabulation algorithm ─────────────────────────────────────────────

    let mut path_edges_full: BTreeSet<PathEdge<P::Fact>> = BTreeSet::new();

    // Summary edges: per function, (entry_fact, exit_fact) pairs.
    // NOTE: The tuple (P::Fact, P::Fact) matches IFDS algorithm's summary edge definition.
    #[allow(clippy::type_complexity)]
    let mut summary_edges: BTreeMap<FunctionId, BTreeSet<(P::Fact, P::Fact)>> = BTreeMap::new();

    // Facts at each instruction (the final result).
    let mut facts_at: BTreeMap<InstId, BTreeSet<P::Fact>> = BTreeMap::new();

    // Worklist of path edges to process.
    let mut worklist: BTreeSet<PathEdge<P::Fact>> = BTreeSet::new();

    // Initialize: seed zero fact at all entry points of functions with seeds.
    let seeds = problem.initial_seeds();

    // Collect seeded functions.
    let mut seeded_funcs: BTreeSet<FunctionId> = seeds.keys().copied().collect();
    // Always seed main (or any function with initial seeds) plus all functions
    // reachable from them via the call graph.
    // For simplicity, seed all defined (non-declaration) functions with the zero fact.
    for func in &module.functions {
        if !func.is_declaration {
            seeded_funcs.insert(func.id);
        }
    }

    for func_id in &seeded_funcs {
        if let Some(entry_inst) = idx.func_entry_inst.get(func_id) {
            // Seed zero fact.
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

            // Seed additional problem-specific facts.
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
                }
            }
        }
    }

    let mut diagnostics = IfdsDiagnostics::default();

    // Main loop.
    while let Some(edge) = worklist.pop_first() {
        diagnostics.iterations += 1;
        diagnostics.path_edges_explored += 1;

        if diagnostics.iterations > config.max_iterations {
            diagnostics.reached_limit = true;
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

        if is_call && has_defined_callees {
            // ── Call node with defined callees ───────────────────────

            // 1. Call-to-return: facts that bypass the callee.
            let c2r_facts = problem.call_to_return_flow(instruction, d2);
            if let Some(return_insts) = idx.call_site_return_inst.get(&n) {
                for ret_inst in return_insts {
                    for d3 in &c2r_facts {
                        propagate(
                            func,
                            d1,
                            *ret_inst,
                            d3,
                            &mut path_edges_full,
                            &mut worklist,
                            &mut facts_at,
                            config,
                            &mut diagnostics,
                        );
                    }
                }
            }

            // 2. Call flow: enter callee.
            if let Some(callees) = idx.call_to_callees.get(&n) {
                for callee_id in callees {
                    if let Some(callee_func) = module.function(*callee_id) {
                        let call_facts = problem.call_flow(instruction, callee_func, d2);
                        if let Some(callee_entry) = idx.func_entry_inst.get(callee_id) {
                            for d3 in &call_facts {
                                propagate(
                                    *callee_id,
                                    d3,
                                    *callee_entry,
                                    d3,
                                    &mut path_edges_full,
                                    &mut worklist,
                                    &mut facts_at,
                                    config,
                                    &mut diagnostics,
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
                                            for ret_inst in return_insts {
                                                propagate(
                                                    func,
                                                    d1,
                                                    *ret_inst,
                                                    sum_d2,
                                                    &mut path_edges_full,
                                                    &mut worklist,
                                                    &mut facts_at,
                                                    config,
                                                    &mut diagnostics,
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
            // ── External call (no defined callee body) ───────────────
            // For external calls, we use `normal_flow` for both fact
            // registration and propagation. We intentionally do NOT use
            // `call_to_return_flow` here because its kill semantics assume
            // a through-callee path exists to recover killed facts. With
            // no callee body, killed facts would be permanently lost.
            let new_facts = problem.normal_flow(instruction, d2);

            // Record generated facts at the call instruction itself.
            for d3 in &new_facts {
                propagate(
                    func,
                    d1,
                    n,
                    d3,
                    &mut path_edges_full,
                    &mut worklist,
                    &mut facts_at,
                    config,
                    &mut diagnostics,
                );
            }

            let successors = idx.successor_instructions(n, func, instruction, icfg);

            for succ in &successors {
                for d3 in &new_facts {
                    propagate(
                        func,
                        d1,
                        *succ,
                        d3,
                        &mut path_edges_full,
                        &mut worklist,
                        &mut facts_at,
                        config,
                        &mut diagnostics,
                    );
                }
            }
        } else if is_exit {
            // ── Exit node processing ─────────────────────────────────

            // Create summary edges and propagate to callers.
            let entry_fact = d1;
            let exit_fact = d2;

            // For each caller of this function:
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
                        // Record summary edge.
                        let is_new = summary_edges
                            .entry(func)
                            .or_default()
                            .insert((entry_fact.clone(), d3.clone()));

                        if is_new {
                            diagnostics.summary_edges_created += 1;
                        }

                        // Propagate to all callers that had a matching entry fact.
                        let call_site_fn = match idx.inst_to_func.get(call_site_id) {
                            Some(f) => *f,
                            None => continue,
                        };

                        // Find path edges at the call site with matching call flow.
                        let matching_edges: Vec<_> = path_edges_full
                            .iter()
                            .filter(|pe| pe.inst == *call_site_id && pe.func == call_site_fn)
                            .cloned()
                            .collect();

                        for caller_edge in &matching_edges {
                            // Check if call_flow from caller_d2 produces entry_fact.
                            let cf = problem.call_flow(call_inst, exit_fn, &caller_edge.d2);
                            if cf.contains(entry_fact) {
                                if let Some(return_insts) =
                                    idx.call_site_return_inst.get(call_site_id)
                                {
                                    for ret_inst in return_insts {
                                        propagate(
                                            call_site_fn,
                                            &caller_edge.d1,
                                            *ret_inst,
                                            d3,
                                            &mut path_edges_full,
                                            &mut worklist,
                                            &mut facts_at,
                                            config,
                                            &mut diagnostics,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // ── Normal node processing ───────────────────────────────

            let new_facts = problem.normal_flow(instruction, d2);

            // Determine successor instructions.
            let successors = idx.successor_instructions(n, func, instruction, icfg);

            for succ in &successors {
                for d3 in &new_facts {
                    propagate(
                        func,
                        d1,
                        *succ,
                        d3,
                        &mut path_edges_full,
                        &mut worklist,
                        &mut facts_at,
                        config,
                        &mut diagnostics,
                    );
                }
            }
        }
    }

    // Compute peak facts.
    for set in facts_at.values() {
        if set.len() > diagnostics.facts_at_peak {
            diagnostics.facts_at_peak = set.len();
        }
    }

    IfdsResult {
        facts: facts_at,
        summaries: summary_edges,
        diagnostics,
    }
}

/// Propagate a new path edge, adding it to the worklist if unseen.
// NOTE: This function requires many parameters because it operates on the shared
// solver state (path edges, worklist, facts, diagnostics) that cannot be bundled
// into a struct without introducing borrow checker conflicts in the main loop.
#[allow(clippy::too_many_arguments)]
fn propagate<F: Ord + Clone + std::fmt::Debug>(
    func: FunctionId,
    d1: &F,
    inst: InstId,
    d2: &F,
    path_edges: &mut BTreeSet<PathEdge<F>>,
    worklist: &mut BTreeSet<PathEdge<F>>,
    facts_at: &mut BTreeMap<InstId, BTreeSet<F>>,
    config: &IfdsConfig,
    diagnostics: &mut IfdsDiagnostics,
) {
    // Check facts-per-point limit.
    let current_count = facts_at.get(&inst).map_or(0, BTreeSet::len);
    if current_count >= config.max_facts_per_point {
        diagnostics.reached_limit = true;
        return;
    }

    let edge = PathEdge {
        func,
        d1: d1.clone(),
        inst,
        d2: d2.clone(),
    };

    if path_edges.insert(edge.clone()) {
        worklist.insert(edge);
        facts_at.entry(inst).or_default().insert(d2.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callgraph::CallGraph;
    use saf_core::air::{AirBlock, AirFunction, AirModule, AirParam, Instruction};
    use saf_core::ids::{BlockId, ModuleId, ValueId};

    // ── Test fact type ───────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TestFact {
        Zero,
        Tainted(ValueId),
    }

    // ── Helpers to build AIR modules ─────────────────────────────────────

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

    fn make_function_with_params(
        id: u128,
        name: &str,
        params: Vec<AirParam>,
        blocks: Vec<AirBlock>,
    ) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params,
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

    // ── Identity problem: propagates all facts unchanged ─────────────────

    struct IdentityProblem {
        module: AirModule,
    }

    impl IfdsProblem for IdentityProblem {
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

        fn normal_flow(&self, _inst: &Instruction, fact: &Self::Fact) -> BTreeSet<Self::Fact> {
            [fact.clone()].into_iter().collect()
        }

        fn call_flow(
            &self,
            _call_site: &Instruction,
            _callee: &AirFunction,
            fact: &Self::Fact,
        ) -> BTreeSet<Self::Fact> {
            [fact.clone()].into_iter().collect()
        }

        fn return_flow(
            &self,
            _call_site: &Instruction,
            _callee: &AirFunction,
            _exit_inst: &Instruction,
            fact: &Self::Fact,
        ) -> BTreeSet<Self::Fact> {
            [fact.clone()].into_iter().collect()
        }

        fn call_to_return_flow(
            &self,
            _call_site: &Instruction,
            fact: &Self::Fact,
        ) -> BTreeSet<Self::Fact> {
            [fact.clone()].into_iter().collect()
        }
    }

    // ── Gen/kill problem: generates taint at source calls, kills at sanitizer ──

    struct GenKillProblem {
        module: AirModule,
        source_func: FunctionId,
        sanitizer_func: FunctionId,
    }

    impl IfdsProblem for GenKillProblem {
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
            // At a call to source, zero generates Tainted(dst).
            if let Operation::CallDirect { callee } = &inst.op {
                if *callee == self.source_func {
                    if let TestFact::Zero = fact {
                        let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                        if let Some(dst) = inst.dst {
                            result.insert(TestFact::Tainted(dst));
                        }
                        return result;
                    }
                }
                // Kill taint at sanitizer.
                if *callee == self.sanitizer_func {
                    if let TestFact::Tainted(_) = fact {
                        return BTreeSet::new();
                    }
                }
            }

            // Copy propagation: if instruction copies a tainted value, propagate.
            if let Operation::Copy = &inst.op {
                if let TestFact::Tainted(v) = fact {
                    if inst.operands.contains(v) {
                        let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                        if let Some(dst) = inst.dst {
                            result.insert(TestFact::Tainted(dst));
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
            // Map tainted arguments to callee parameters.
            if let TestFact::Tainted(v) = fact {
                for (i, operand) in call_site.operands.iter().enumerate() {
                    if operand == v {
                        if let Some(param) = callee.params.get(i) {
                            return [TestFact::Tainted(param.id)].into_iter().collect();
                        }
                    }
                }
            }
            // Always propagate zero.
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
            // Map tainted return value back to call result.
            if let TestFact::Tainted(v) = fact {
                if exit_inst.operands.contains(v) {
                    if let Some(dst) = call_site.dst {
                        return [TestFact::Tainted(dst)].into_iter().collect();
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
            // Pass through facts not related to call arguments.
            if let TestFact::Tainted(v) = fact {
                if call_site.operands.contains(v) {
                    return BTreeSet::new(); // Killed at call site (handled by call_flow).
                }
            }
            [fact.clone()].into_iter().collect()
        }
    }

    // ── Tests ────────────────────────────────────────────────────────────

    #[test]
    fn linear_flow_propagates_zero() {
        // Single function, 3 instructions, zero fact propagates through.
        let module = make_module(vec![make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
                        .with_dst(ValueId::new(10)),
                    Instruction::new(InstId::new(101), Operation::Copy)
                        .with_operands(vec![ValueId::new(10)])
                        .with_dst(ValueId::new(11)),
                    Instruction::new(InstId::new(102), Operation::Ret),
                ],
            )],
        )]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = IdentityProblem { module };
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Zero fact should reach all three instructions.
        assert!(result.holds_at(InstId::new(100), &TestFact::Zero));
        assert!(result.holds_at(InstId::new(101), &TestFact::Zero));
        assert!(result.holds_at(InstId::new(102), &TestFact::Zero));
    }

    #[test]
    fn gen_flow_creates_tainted_fact() {
        // Call to source generates Tainted(dst).
        let source = make_declaration(2, "getenv");
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
        let problem = GenKillProblem {
            module,
            source_func: FunctionId::new(2),
            sanitizer_func: FunctionId::new(99),
        };
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // After the call to getenv, Tainted(10) should be generated.
        assert!(result.holds_at(InstId::new(100), &TestFact::Tainted(ValueId::new(10))));
        // And it should propagate to the ret.
        assert!(result.holds_at(InstId::new(101), &TestFact::Tainted(ValueId::new(10))));
    }

    #[test]
    fn kill_flow_removes_fact() {
        // Source → sanitizer → sink. Taint should be killed at sanitizer.
        let source = make_declaration(2, "getenv");
        let sanitizer = make_declaration(3, "sanitize");
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
                        Instruction::new(
                            InstId::new(101),
                            Operation::CallDirect {
                                callee: FunctionId::new(3),
                            },
                        )
                        .with_operands(vec![ValueId::new(10)])
                        .with_dst(ValueId::new(11)),
                        Instruction::new(InstId::new(102), Operation::Ret),
                    ],
                )],
            ),
            source,
            sanitizer,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = GenKillProblem {
            module,
            source_func: FunctionId::new(2),
            sanitizer_func: FunctionId::new(3),
        };
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Tainted(10) should be generated at call to getenv.
        assert!(result.holds_at(InstId::new(100), &TestFact::Tainted(ValueId::new(10))));
        // But killed at sanitizer — should NOT reach ret.
        assert!(!result.holds_at(InstId::new(102), &TestFact::Tainted(ValueId::new(10))));
    }

    #[test]
    fn diamond_cfg_propagates_through_both_branches() {
        // entry -> (then, else) -> join -> ret
        let module = make_module(vec![make_function(
            1,
            "main",
            vec![
                make_block(
                    1,
                    vec![
                        Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
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
        )]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = IdentityProblem { module };
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Zero should reach all blocks.
        assert!(result.holds_at(InstId::new(100), &TestFact::Zero));
        assert!(result.holds_at(InstId::new(200), &TestFact::Zero));
        assert!(result.holds_at(InstId::new(300), &TestFact::Zero));
        assert!(result.holds_at(InstId::new(400), &TestFact::Zero));
    }

    #[test]
    fn loop_propagates_through_body() {
        // entry -> loop_body -> (back to loop_body | exit)
        let module = make_module(vec![make_function(
            1,
            "main",
            vec![
                make_block(
                    1,
                    vec![Instruction::new(
                        InstId::new(100),
                        Operation::Br {
                            target: BlockId::new(2),
                        },
                    )],
                ),
                make_block(
                    2,
                    vec![
                        Instruction::new(InstId::new(200), Operation::Alloca { size_bytes: None })
                            .with_dst(ValueId::new(10)),
                        Instruction::new(
                            InstId::new(201),
                            Operation::CondBr {
                                then_target: BlockId::new(2),
                                else_target: BlockId::new(3),
                            },
                        ),
                    ],
                ),
                make_block(3, vec![Instruction::new(InstId::new(300), Operation::Ret)]),
            ],
        )]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = IdentityProblem { module };
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Zero should reach loop body and exit.
        assert!(result.holds_at(InstId::new(200), &TestFact::Zero));
        assert!(result.holds_at(InstId::new(300), &TestFact::Zero));
    }

    #[test]
    fn interprocedural_taint_through_call() {
        // main calls helper(tainted_val), helper returns it, main uses result.
        let source = make_declaration(3, "getenv");
        let helper = make_function_with_params(
            2,
            "helper",
            vec![AirParam::new(ValueId::new(30), 0)],
            vec![make_block(
                2,
                vec![
                    Instruction::new(InstId::new(200), Operation::Ret)
                        .with_operands(vec![ValueId::new(30)]),
                ],
            )],
        );
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    // Call getenv (external) — generates taint on dst.
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(3),
                        },
                    )
                    .with_dst(ValueId::new(10)),
                    // Call helper(tainted_value) — should propagate taint through.
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
        let problem = GenKillProblem {
            module,
            source_func: FunctionId::new(3),
            sanitizer_func: FunctionId::new(99),
        };
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Taint should be generated at getenv call.
        assert!(result.holds_at(InstId::new(100), &TestFact::Tainted(ValueId::new(10))));
        // Taint should propagate through helper and back.
        assert!(result.holds_at(InstId::new(102), &TestFact::Tainted(ValueId::new(11))));
    }

    #[test]
    fn zero_fact_reaches_all_reachable_points() {
        let module = make_module(vec![make_function(
            1,
            "main",
            vec![
                make_block(
                    1,
                    vec![
                        Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
                            .with_dst(ValueId::new(10)),
                        Instruction::new(
                            InstId::new(101),
                            Operation::Br {
                                target: BlockId::new(2),
                            },
                        ),
                    ],
                ),
                make_block(2, vec![Instruction::new(InstId::new(200), Operation::Ret)]),
            ],
        )]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = IdentityProblem { module };
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        assert!(result.holds_at(InstId::new(100), &TestFact::Zero));
        assert!(result.holds_at(InstId::new(101), &TestFact::Zero));
        assert!(result.holds_at(InstId::new(200), &TestFact::Zero));
    }

    #[test]
    fn max_iterations_limit_triggers() {
        let module = make_module(vec![make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![Instruction::new(InstId::new(100), Operation::Ret)],
            )],
        )]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = IdentityProblem { module };
        let config = IfdsConfig {
            max_iterations: 0, // Immediately hit limit.
            ..IfdsConfig::default()
        };

        let result = solve_ifds(&problem, &icfg, &cg, &config);
        assert!(result.diagnostics.reached_limit);
    }

    #[test]
    fn max_facts_per_point_limits_propagation() {
        let module = make_module(vec![make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![Instruction::new(InstId::new(100), Operation::Ret)],
            )],
        )]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let problem = IdentityProblem { module };
        let config = IfdsConfig {
            max_facts_per_point: 1,
            ..IfdsConfig::default()
        };

        let result = solve_ifds(&problem, &icfg, &cg, &config);
        // With limit of 1, at most 1 fact per point.
        for facts in result.facts.values() {
            assert!(facts.len() <= 1);
        }
    }
}
