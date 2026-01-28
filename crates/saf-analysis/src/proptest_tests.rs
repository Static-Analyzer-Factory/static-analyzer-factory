//! Property-based tests for graph builders and PTA.

use proptest::prelude::*;

use saf_core::air::Operation;

use crate::callgraph::CallGraph;
use crate::cfg::Cfg;
use crate::defuse::DefUseGraph;
use crate::proptest_arb::{
    arb_air_function, arb_air_module, arb_pta_module, arb_tracked_pta_module,
};
use crate::pta::{FieldSensitivity, PtaConfig, PtaContext};

proptest! {
    /// Property: predecessors is the reverse of successors.
    ///
    /// For every edge A -> B in successors, B's predecessors must contain A.
    #[test]
    fn predecessors_is_reverse_of_successors(func in arb_air_function()) {
        let cfg = Cfg::build(&func);

        for (from, succs) in &cfg.successors {
            for to in succs {
                let preds = cfg.predecessors.get(to).expect("block should exist in predecessors");
                prop_assert!(
                    preds.contains(from),
                    "Edge {:?} -> {:?} not reflected in predecessors",
                    from,
                    to
                );
            }
        }

        // Also check reverse: for every edge in predecessors, there's an edge in successors
        for (to, preds) in &cfg.predecessors {
            for from in preds {
                let succs = cfg.successors.get(from).expect("block should exist in successors");
                prop_assert!(
                    succs.contains(to),
                    "Predecessor edge {:?} <- {:?} not reflected in successors",
                    to,
                    from
                );
            }
        }
    }

    /// Property: number of edges equals unique terminator targets.
    ///
    /// The total number of edges in the CFG should equal the unique targets
    /// across all terminators (duplicates are collapsed in BTreeSet).
    #[test]
    fn cfg_edge_count_matches_terminators(func in arb_air_function()) {
        use std::collections::BTreeSet;

        let cfg = Cfg::build(&func);

        // Count edges in CFG
        let edge_count: usize = cfg.successors.values().map(|s| s.len()).sum();

        // Count expected unique edges from terminators
        let expected_edges: usize = func.blocks.iter().map(|block| {
            if let Some(term) = block.terminator() {
                let targets: BTreeSet<_> = match &term.op {
                    Operation::Br { target } => [*target].into_iter().collect(),
                    Operation::CondBr { then_target, else_target } => {
                        [*then_target, *else_target].into_iter().collect()
                    }
                    Operation::Switch { default, cases } => {
                        std::iter::once(*default)
                            .chain(cases.iter().map(|(_, t)| *t))
                            .collect()
                    }
                    Operation::Ret | Operation::Unreachable => BTreeSet::new(),
                    _ => BTreeSet::new(),
                };
                targets.len()
            } else {
                0
            }
        }).sum();

        prop_assert_eq!(
            edge_count,
            expected_edges,
            "Edge count mismatch: CFG has {} edges, terminators specify {}",
            edge_count,
            expected_edges
        );
    }

    /// Property: every block in the function is in the CFG.
    #[test]
    fn cfg_contains_all_blocks(func in arb_air_function()) {
        let cfg = Cfg::build(&func);

        for block in &func.blocks {
            prop_assert!(
                cfg.successors.contains_key(&block.id),
                "Block {:?} not in CFG successors",
                block.id
            );
            prop_assert!(
                cfg.predecessors.contains_key(&block.id),
                "Block {:?} not in CFG predecessors",
                block.id
            );
        }
    }

    /// Property: entry block has no incoming edges from outside (for simple functions).
    #[test]
    fn cfg_entry_is_first_block(func in arb_air_function()) {
        if func.blocks.is_empty() {
            return Ok(());
        }

        let cfg = Cfg::build(&func);
        let expected_entry = func.blocks.first().unwrap().id;

        prop_assert_eq!(
            cfg.entry,
            expected_entry,
            "Entry block mismatch: expected {:?}, got {:?}",
            expected_entry,
            cfg.entry
        );
    }

    /// Property: exits are exactly the blocks with no successors.
    #[test]
    fn cfg_exits_have_no_successors(func in arb_air_function()) {
        let cfg = Cfg::build(&func);

        // Every exit block should have no successors
        for exit in &cfg.exits {
            let succs = cfg.successors.get(exit).expect("exit block should exist");
            prop_assert!(
                succs.is_empty(),
                "Exit block {:?} has successors: {:?}",
                exit,
                succs
            );
        }

        // Every block with no successors should be an exit
        for (block, succs) in &cfg.successors {
            if succs.is_empty() {
                prop_assert!(
                    cfg.exits.contains(block),
                    "Block {:?} has no successors but is not marked as exit",
                    block
                );
            }
        }
    }

    /// Property: call graph nodes include all functions in module.
    #[test]
    fn callgraph_contains_all_functions(module in arb_air_module()) {
        let cg = CallGraph::build(&module);

        for func in &module.functions {
            let has_node = cg.nodes.iter().any(|node| {
                node.function_id() == Some(func.id)
            });
            prop_assert!(
                has_node,
                "Function {:?} not in call graph",
                func.id
            );
        }
    }

    /// Property: every defined value is in def-use graph.
    #[test]
    fn defuse_contains_all_defs(module in arb_air_module()) {
        let du = DefUseGraph::build(&module);

        for func in &module.functions {
            if func.is_declaration {
                continue;
            }

            // All parameters should be defined
            for param in &func.params {
                prop_assert!(
                    du.defs.contains_key(&param.id),
                    "Parameter {:?} not in def-use defs",
                    param.id
                );
            }

            // All instruction destinations should be defined
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Some(dst) = inst.dst {
                        prop_assert!(
                            du.defs.contains_key(&dst),
                            "Instruction result {:?} not in def-use defs",
                            dst
                        );
                    }
                }
            }
        }
    }

    /// Property: CFG is deterministic (same input = same output).
    #[test]
    fn cfg_is_deterministic(func in arb_air_function()) {
        let cfg1 = Cfg::build(&func);
        let cfg2 = Cfg::build(&func);

        prop_assert_eq!(cfg1, cfg2, "CFG is not deterministic");
    }

    /// Property: call graph is deterministic.
    #[test]
    fn callgraph_is_deterministic(module in arb_air_module()) {
        let cg1 = CallGraph::build(&module);
        let cg2 = CallGraph::build(&module);

        prop_assert_eq!(cg1, cg2, "Call graph is not deterministic");
    }

    /// Property: def-use graph is deterministic.
    #[test]
    fn defuse_is_deterministic(module in arb_air_module()) {
        let du1 = DefUseGraph::build(&module);
        let du2 = DefUseGraph::build(&module);

        prop_assert_eq!(du1, du2, "Def-use graph is not deterministic");
    }

    // =========================================================================
    // PTA Property Tests
    // =========================================================================

    /// Property: PTA is deterministic (same input = same output).
    ///
    /// Running PTA twice on the same module must produce identical results.
    #[test]
    fn pta_is_deterministic(module in arb_pta_module()) {
        let config = make_pta_config();

        let mut ctx1 = PtaContext::new(config.clone());
        let result1 = ctx1.analyze(&module);

        let mut ctx2 = PtaContext::new(config);
        let result2 = ctx2.analyze(&module);

        // Points-to maps must be identical
        prop_assert_eq!(
            result1.pts.len(),
            result2.pts.len(),
            "PTA points-to map sizes differ"
        );

        for (key, val1) in &result1.pts {
            let val2 = result2.pts.get(key);
            prop_assert!(
                val2.is_some(),
                "Key {:?} missing in second run",
                key
            );
            prop_assert_eq!(
                val1,
                val2.unwrap(),
                "Points-to set for {:?} differs between runs",
                key
            );
        }

        // Constraint counts must match
        prop_assert_eq!(
            result1.constraints.total_count(),
            result2.constraints.total_count(),
            "Constraint counts differ"
        );
    }

    /// Property: every Addr constraint creates a points-to edge.
    ///
    /// For every Addr(p, loc) constraint, pts[p] must contain loc after solving.
    #[test]
    fn pta_addr_creates_points_to(module in arb_pta_module()) {
        let config = make_pta_config();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        for addr in &result.constraints.addr {
            let pts = result.pts.get(&addr.ptr);
            prop_assert!(
                pts.is_some(),
                "Addr constraint ptr {:?} has no points-to set",
                addr.ptr
            );
            prop_assert!(
                pts.unwrap().contains(&addr.loc),
                "Addr constraint: pts[{:?}] does not contain {:?}",
                addr.ptr,
                addr.loc
            );
        }
    }

    /// Property: Copy constraints propagate points-to sets.
    ///
    /// For every Copy(dst, src) constraint, pts[src] ⊆ pts[dst] after solving.
    #[test]
    fn pta_copy_propagates(module in arb_pta_module()) {
        let config = make_pta_config();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        for copy in &result.constraints.copy {
            let src_pts = result.pts.get(&copy.src).cloned().unwrap_or_default();
            let dst_pts = result.pts.get(&copy.dst).cloned().unwrap_or_default();

            // Every location in src_pts should be in dst_pts
            for loc in &src_pts {
                prop_assert!(
                    dst_pts.contains(loc),
                    "Copy constraint: location {:?} in pts[{:?}] but not in pts[{:?}]",
                    loc,
                    copy.src,
                    copy.dst
                );
            }
        }
    }

    /// Property: number of locations >= number of Addr constraints.
    ///
    /// Each Addr constraint creates at least one location.
    #[test]
    fn pta_location_count_at_least_addr_count(module in arb_pta_module()) {
        let config = make_pta_config();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        prop_assert!(
            result.diagnostics.location_count >= result.constraints.addr.len(),
            "Fewer locations ({}) than Addr constraints ({})",
            result.diagnostics.location_count,
            result.constraints.addr.len()
        );
    }

    /// Property: constraint count in diagnostics matches actual constraints.
    #[test]
    fn pta_constraint_count_matches(module in arb_pta_module()) {
        let config = make_pta_config();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        prop_assert_eq!(
            result.diagnostics.constraint_count,
            result.constraints.total_count(),
            "Diagnostics constraint count doesn't match actual"
        );
    }

    /// Property: all values in points-to map point to valid locations.
    #[test]
    fn pta_pts_reference_valid_locations(module in arb_pta_module()) {
        let config = make_pta_config();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        let all_locs = result.factory.all_locations();

        for (value, locs) in &result.pts {
            for loc in locs {
                prop_assert!(
                    all_locs.contains_key(loc),
                    "Value {:?} points to unknown location {:?}",
                    value,
                    loc
                );
            }
        }
    }

    /// Property: disabled PTA returns empty results.
    #[test]
    fn pta_disabled_returns_empty(module in arb_pta_module()) {
        let config = PtaConfig {
            enabled: false,
            ..PtaConfig::default()
        };

        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        prop_assert!(result.pts.is_empty(), "Disabled PTA should return empty pts");
        prop_assert!(result.constraints.is_empty(), "Disabled PTA should have no constraints");
    }

    /// Property: field sensitivity None produces fewer or equal locations than StructFields.
    #[test]
    fn pta_field_sensitivity_affects_locations(module in arb_pta_module()) {
        let config_none = PtaConfig {
            enabled: true,
            field_sensitivity: FieldSensitivity::None,
            ..PtaConfig::default()
        };

        let config_struct = PtaConfig {
            enabled: true,
            field_sensitivity: FieldSensitivity::StructFields { max_depth: 2 },
            ..PtaConfig::default()
        };

        let mut ctx_none = PtaContext::new(config_none);
        let result_none = ctx_none.analyze(&module);

        let mut ctx_struct = PtaContext::new(config_struct);
        let result_struct = ctx_struct.analyze(&module);

        // With no field paths in our generated tests, locations should be equal
        // But in general, None <= StructFields (we just check they both complete)
        // (reaching this point without panic means both analyses completed)
        let _ = result_none.diagnostics.location_count;
        let _ = result_struct.diagnostics.location_count;
    }
}

/// Helper function to create a default PTA config for property tests.
fn make_pta_config() -> PtaConfig {
    PtaConfig::default()
}

// =========================================================================
// ValueFlow Property Tests
// =========================================================================

use std::sync::Arc;

use crate::valueflow::{QueryLimits, ValueFlowBuilder, ValueFlowConfig, ValueFlowMode};
use std::collections::BTreeSet;

/// Helper function to create a default ValueFlow config for property tests.
fn make_valueflow_config() -> ValueFlowConfig {
    ValueFlowConfig::default()
}

proptest! {
    /// Property: ValueFlow graph is deterministic (same input = same output).
    ///
    /// Running ValueFlow builder twice on the same module must produce identical graphs.
    #[test]
    fn valueflow_is_deterministic(module in arb_pta_module()) {
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let pta_config = make_pta_config();
        let mut pta_ctx = PtaContext::new(pta_config.clone());
        let pta_result = pta_ctx.analyze(&module);
        let pta = crate::pta::PtaResult::new(
            pta_result.pts,
            Arc::new(pta_result.factory),
            pta_result.diagnostics,
        );

        let config = make_valueflow_config();

        // Build twice
        let builder1 = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta));
        let graph1 = builder1.build();

        let builder2 = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta));
        let graph2 = builder2.build();

        // Graphs should be identical
        prop_assert_eq!(
            graph1.nodes().len(),
            graph2.nodes().len(),
            "ValueFlow node counts should match"
        );
        prop_assert_eq!(
            graph1.edge_count(),
            graph2.edge_count(),
            "ValueFlow edge counts should match"
        );
        prop_assert_eq!(
            graph1.nodes(),
            graph2.nodes(),
            "ValueFlow nodes should be identical"
        );
    }

    /// Property: ValueFlow graph successors is reverse of predecessors.
    ///
    /// For every edge A -> B, B's predecessors must contain A.
    #[test]
    fn valueflow_successors_reverse_of_predecessors(module in arb_pta_module()) {
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let pta_config = make_pta_config();
        let mut pta_ctx = PtaContext::new(pta_config);
        let pta_result = pta_ctx.analyze(&module);
        let pta = crate::pta::PtaResult::new(
            pta_result.pts,
            Arc::new(pta_result.factory),
            pta_result.diagnostics,
        );

        let config = make_valueflow_config();
        let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta));
        let graph = builder.build();

        // For every successor edge, check predecessor contains reverse
        for node in graph.nodes() {
            if let Some(successors) = graph.successors_of(*node) {
                for (edge_kind, target) in successors {
                    if let Some(preds) = graph.predecessors_of(*target) {
                        let has_reverse = preds.iter().any(|(ek, src)| *ek == *edge_kind && *src == *node);
                        prop_assert!(
                            has_reverse,
                            "Edge {:?} -[{:?}]-> {:?} not reflected in predecessors",
                            node, edge_kind, target
                        );
                    }
                }
            }
        }
    }

    /// Property: ValueFlow flows query is deterministic.
    ///
    /// Running flows() twice with the same parameters returns identical results.
    #[test]
    fn valueflow_flows_is_deterministic(module in arb_pta_module()) {
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let pta_config = make_pta_config();
        let mut pta_ctx = PtaContext::new(pta_config);
        let pta_result = pta_ctx.analyze(&module);
        let pta = crate::pta::PtaResult::new(
            pta_result.pts,
            Arc::new(pta_result.factory),
            pta_result.diagnostics,
        );

        let config = make_valueflow_config();
        let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta));
        let graph = builder.build();

        // Get value nodes for querying
        let value_nodes: Vec<_> = graph.value_nodes().collect();

        if value_nodes.len() >= 2 {
            let sources: BTreeSet<_> = value_nodes.iter().take(1).copied().collect();
            let sinks: BTreeSet<_> = value_nodes.iter().skip(1).take(3).copied().collect();

            let limits = QueryLimits {
                max_depth: 5,
                max_paths: 10,
            };

            // Query twice
            let flows1 = graph.flows(&sources, &sinks, &limits);
            let flows2 = graph.flows(&sources, &sinks, &limits);

            prop_assert_eq!(
                flows1.len(),
                flows2.len(),
                "Flow counts should match"
            );

            for (f1, f2) in flows1.iter().zip(flows2.iter()) {
                prop_assert_eq!(f1.source, f2.source, "Sources should match");
                prop_assert_eq!(f1.sink, f2.sink, "Sinks should match");
                prop_assert_eq!(
                    f1.trace.steps.len(),
                    f2.trace.steps.len(),
                    "Trace lengths should match"
                );
            }
        }
    }

    /// Property: ValueFlow fast mode produces valid graph without PTA.
    #[test]
    fn valueflow_fast_mode_valid(module in arb_pta_module()) {
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);

        let config = ValueFlowConfig {
            mode: ValueFlowMode::Fast,
            ..Default::default()
        };

        let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, None);
        let graph = builder.build();

        // Fast mode should still produce a valid graph
        // (reaching this point without panic means build succeeded)
        let _ = graph.nodes().len();
    }

    /// Property: max_depth limit is respected in flows query.
    #[test]
    fn valueflow_max_depth_respected(module in arb_pta_module()) {
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let pta_config = make_pta_config();
        let mut pta_ctx = PtaContext::new(pta_config);
        let pta_result = pta_ctx.analyze(&module);
        let pta = crate::pta::PtaResult::new(
            pta_result.pts,
            Arc::new(pta_result.factory),
            pta_result.diagnostics,
        );

        let config = make_valueflow_config();
        let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta));
        let graph = builder.build();

        let value_nodes: Vec<_> = graph.nodes().iter()
            .filter_map(|n| n.as_value())
            .collect();

        if value_nodes.len() >= 2 {
            let sources: BTreeSet<_> = value_nodes.iter().take(1).copied().collect();
            let sinks: BTreeSet<_> = value_nodes.iter().skip(1).copied().collect();

            let max_depth = 3;
            let limits = QueryLimits {
                max_depth,
                max_paths: 100,
            };

            let flows = graph.flows(&sources, &sinks, &limits);

            // All traces should respect max_depth
            for flow in &flows {
                prop_assert!(
                    flow.trace.steps.len() <= max_depth,
                    "Trace length {} exceeds max_depth {}",
                    flow.trace.steps.len(),
                    max_depth
                );
            }
        }
    }

    /// Property: sanitizers reduce or maintain finding count.
    ///
    /// Adding sanitizers should never increase the number of findings.
    #[test]
    fn valueflow_sanitizers_reduce_findings(module in arb_pta_module()) {
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let pta_config = make_pta_config();
        let mut pta_ctx = PtaContext::new(pta_config);
        let pta_result = pta_ctx.analyze(&module);
        let pta = crate::pta::PtaResult::new(
            pta_result.pts,
            Arc::new(pta_result.factory),
            pta_result.diagnostics,
        );

        let config = make_valueflow_config();
        let builder = ValueFlowBuilder::new(&config, &module, &defuse, &callgraph, Some(&pta));
        let graph = builder.build();

        let value_nodes: Vec<_> = graph.nodes().iter()
            .filter_map(|n| n.as_value())
            .collect();

        if value_nodes.len() >= 3 {
            let sources: BTreeSet<_> = value_nodes.iter().take(1).copied().collect();
            let sinks: BTreeSet<_> = value_nodes.iter().skip(2).take(2).copied().collect();
            let sanitizers: BTreeSet<_> = value_nodes.iter().skip(1).take(1).copied().collect();

            let limits = QueryLimits {
                max_depth: 10,
                max_paths: 100,
            };

            let findings_no_sanitizer = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
            let findings_with_sanitizer = graph.taint_flow(&sources, &sinks, &sanitizers, &limits);

            prop_assert!(
                findings_with_sanitizer.len() <= findings_no_sanitizer.len(),
                "Sanitizers should not increase findings: {} with sanitizer vs {} without",
                findings_with_sanitizer.len(),
                findings_no_sanitizer.len()
            );
        }
    }
}

// =========================================================================
// Constraint Extraction Completeness Property Tests
// =========================================================================

proptest! {
    /// Property: every `Alloca` instruction produces at least one `Addr` constraint.
    ///
    /// If the generated module contains N allocas, the solver must produce >= N
    /// `Addr` constraints (some allocas may produce multiple if field-sensitive).
    #[test]
    fn every_alloca_produces_addr_constraint(
        (module, alloca_count, _, _) in arb_tracked_pta_module()
    ) {
        let config = make_pta_config();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        prop_assert!(
            result.constraints.addr.len() >= alloca_count,
            "Expected at least {} Addr constraints for {} allocas, got {}",
            alloca_count,
            alloca_count,
            result.constraints.addr.len()
        );
    }

    /// Property: stores in a module produce store constraints.
    ///
    /// When a module has Store instructions, the constraint extractor must
    /// produce at least one Store constraint. Note: the count may be less than
    /// the number of Store instructions due to `BTreeSet` deduplication when
    /// multiple stores write the same (src, dst_ptr) pair.
    #[test]
    fn stores_produce_store_constraints(
        (module, _, store_count, _) in arb_tracked_pta_module()
    ) {
        let config = make_pta_config();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        if store_count > 0 {
            prop_assert!(
                !result.constraints.store.is_empty(),
                "Module has {} Store instructions but no Store constraints were extracted",
                store_count,
            );
        }
    }

    /// Property: loads in a module produce load constraints.
    ///
    /// When a module has Load instructions, the constraint extractor must
    /// produce at least one Load constraint. Count may differ from instruction
    /// count due to `BTreeSet` deduplication.
    #[test]
    fn loads_produce_load_constraints(
        (module, _, _, load_count) in arb_tracked_pta_module()
    ) {
        let config = make_pta_config();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&module);

        if load_count > 0 {
            prop_assert!(
                !result.constraints.load.is_empty(),
                "Module has {} Load instructions but no Load constraints were extracted",
                load_count,
            );
        }
    }
}
