//! Taint analysis as an IFDS problem.
//!
//! Encodes source/sink/sanitizer-based taint tracking as an IFDS problem.
//! Sources generate taint facts, sanitizers kill them, and the flow functions
//! propagate taint through copies, casts, binary ops, phi nodes, GEP, and
//! interprocedural argument/return passing.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirFunction, AirModule, Instruction, Operation};
use saf_core::ids::{FunctionId, ValueId};

use crate::selector::Selector;

use super::matches_name;
use super::problem::IfdsProblem;

/// A taint data-flow fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaintFact {
    /// The zero (tautology) fact.
    Zero,
    /// A tainted value.
    Tainted(ValueId),
}

/// Taint analysis as an IFDS problem.
///
/// Sources: calls to functions matching source selectors generate taint on
/// the call's return value (dst).
/// Sinks: arguments to functions matching sink selectors (checked externally).
/// Sanitizers: calls to functions matching sanitizer selectors kill taint.
/// Propagation: through Copy, Cast, `BinaryOp`, Select, Phi, GEP, call args, returns.
pub struct TaintIfdsProblem<'a> {
    module: &'a AirModule,
    /// Functions whose call return values are taint sources.
    source_funcs: BTreeSet<FunctionId>,
    /// Functions whose calls kill taint (sanitizers).
    sanitizer_funcs: BTreeSet<FunctionId>,
    /// Maps values derived from function parameters to (function_id, param_index).
    /// Enables return_flow to propagate taint back through pointer parameters
    /// whose contents were modified by the callee (e.g. C++ constructors storing
    /// to `this->field`).
    param_derived: BTreeMap<ValueId, (FunctionId, usize)>,
}

impl<'a> TaintIfdsProblem<'a> {
    /// Create a new taint IFDS problem.
    ///
    /// `sources` and `sanitizers` are selectors identifying source/sanitizer functions.
    /// Sinks are not used in the IFDS problem itself — they are checked on the result.
    pub fn new(module: &'a AirModule, sources: &[Selector], sanitizers: &[Selector]) -> Self {
        // Resolve source functions: find FunctionIds that match source selectors.
        let mut source_funcs = BTreeSet::new();
        for sel in sources {
            // Identify source function IDs from CallTo/FunctionReturn selectors.
            match sel {
                Selector::CallTo { callee } | Selector::FunctionReturn { function: callee } => {
                    for func in &module.functions {
                        if matches_name(&func.name, callee) {
                            source_funcs.insert(func.id);
                        }
                    }
                }
                _ => {}
            }
        }

        // Resolve sanitizer functions.
        let mut sanitizer_funcs = BTreeSet::new();
        for sel in sanitizers {
            match sel {
                Selector::CallTo { callee }
                | Selector::FunctionReturn { function: callee }
                | Selector::ArgTo { callee, .. } => {
                    for func in &module.functions {
                        if matches_name(&func.name, callee) {
                            sanitizer_funcs.insert(func.id);
                        }
                    }
                }
                _ => {}
            }
        }

        // Build param_derived map: for each function, trace parameter values
        // through store→load→GEP chains to identify values derived from parameters.
        let mut param_derived = BTreeMap::new();
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }

            // Step 1: Register formal parameters.
            for (idx, param) in func.params.iter().enumerate() {
                param_derived.insert(param.id, (func.id, idx));
            }

            // Step 2: Find stores of parameter-derived values to allocas.
            let mut alloca_to_param: BTreeMap<ValueId, usize> = BTreeMap::new();
            for block in &func.blocks {
                for inst in &block.instructions {
                    if matches!(inst.op, Operation::Store) && inst.operands.len() >= 2 {
                        let stored_val = inst.operands[0];
                        let ptr = inst.operands[1];
                        if let Some(&(fid, idx)) = param_derived.get(&stored_val) {
                            if fid == func.id {
                                alloca_to_param.insert(ptr, idx);
                            }
                        }
                    }
                }
            }

            // Step 3: Find loads from parameter-storing allocas.
            for block in &func.blocks {
                for inst in &block.instructions {
                    if matches!(inst.op, Operation::Load) && !inst.operands.is_empty() {
                        let ptr = inst.operands[0];
                        if let Some(&idx) = alloca_to_param.get(&ptr) {
                            if let Some(dst) = inst.dst {
                                param_derived.insert(dst, (func.id, idx));
                            }
                        }
                    }
                }
            }

            // Step 4: Find GEPs from parameter-derived bases.
            for block in &func.blocks {
                for inst in &block.instructions {
                    if matches!(inst.op, Operation::Gep { .. }) && !inst.operands.is_empty() {
                        let base = inst.operands[0];
                        if let Some(&(fid, idx)) = param_derived.get(&base) {
                            if fid == func.id {
                                if let Some(dst) = inst.dst {
                                    param_derived.insert(dst, (fid, idx));
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            module,
            source_funcs,
            sanitizer_funcs,
            param_derived,
        }
    }

    /// Check if a function is a taint source.
    fn is_source(&self, func_id: FunctionId) -> bool {
        self.source_funcs.contains(&func_id)
    }

    /// Check if a function is a sanitizer.
    fn is_sanitizer(&self, func_id: FunctionId) -> bool {
        self.sanitizer_funcs.contains(&func_id)
    }
}

impl IfdsProblem for TaintIfdsProblem<'_> {
    type Fact = TaintFact;

    fn zero_value(&self) -> Self::Fact {
        TaintFact::Zero
    }

    fn module(&self) -> &AirModule {
        self.module
    }

    fn initial_seeds(&self) -> BTreeMap<FunctionId, BTreeSet<Self::Fact>> {
        // No additional seeds — taint is generated at source call sites via normal_flow.
        BTreeMap::new()
    }

    fn normal_flow(&self, inst: &Instruction, fact: &Self::Fact) -> BTreeSet<Self::Fact> {
        match &inst.op {
            // Source call: zero generates Tainted(dst).
            Operation::CallDirect { callee } if self.is_source(*callee) => {
                if let TaintFact::Zero = fact {
                    let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                    if let Some(dst) = inst.dst {
                        result.insert(TaintFact::Tainted(dst));
                    }
                    return result;
                }
                // Sanitizer kills taint.
                if self.is_sanitizer(*callee) {
                    if let TaintFact::Tainted(_) = fact {
                        return BTreeSet::new();
                    }
                }
                [fact.clone()].into_iter().collect()
            }
            // Sanitizer call: kill taint facts.
            Operation::CallDirect { callee } if self.is_sanitizer(*callee) => {
                if let TaintFact::Tainted(_) = fact {
                    return BTreeSet::new();
                }
                [fact.clone()].into_iter().collect()
            }
            // Store: if the stored value is tainted, mark the pointer as tainted.
            // operand[0] = value, operand[1] = pointer.
            Operation::Store => {
                if let TaintFact::Tainted(v) = fact {
                    if inst.operands.len() >= 2 && inst.operands[0] == *v {
                        let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                        result.insert(TaintFact::Tainted(inst.operands[1]));
                        return result;
                    }
                }
                [fact.clone()].into_iter().collect()
            }
            // Operations that propagate taint from operands to dst.
            Operation::Copy
            | Operation::Freeze
            | Operation::Cast { .. }
            | Operation::BinaryOp { .. }
            | Operation::Gep { .. }
            | Operation::Load => propagate_through_operands(inst, fact),
            // Select: propagate taint from true/false values to dst.
            Operation::Select => {
                if let TaintFact::Tainted(v) = fact {
                    // Operand 1 (true val) or operand 2 (false val).
                    if inst.operands.len() >= 3
                        && (inst.operands[1] == *v || inst.operands[2] == *v)
                    {
                        let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                        if let Some(dst) = inst.dst {
                            result.insert(TaintFact::Tainted(dst));
                        }
                        return result;
                    }
                }
                [fact.clone()].into_iter().collect()
            }
            // Phi: propagate taint from incoming values to dst.
            Operation::Phi { incoming } => {
                if let TaintFact::Tainted(v) = fact {
                    for (_, val) in incoming {
                        if val == v {
                            let mut result: BTreeSet<Self::Fact> =
                                [fact.clone()].into_iter().collect();
                            if let Some(dst) = inst.dst {
                                result.insert(TaintFact::Tainted(dst));
                            }
                            return result;
                        }
                    }
                }
                [fact.clone()].into_iter().collect()
            }
            // All other operations: pass through unchanged.
            _ => [fact.clone()].into_iter().collect(),
        }
    }

    fn call_flow(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact> {
        match fact {
            TaintFact::Zero => {
                // Always propagate zero to callees.
                [TaintFact::Zero].into_iter().collect()
            }
            TaintFact::Tainted(v) => {
                // Map tainted arguments to callee parameters.
                let mut result = BTreeSet::new();
                for (i, operand) in call_site.operands.iter().enumerate() {
                    if operand == v {
                        if let Some(param) = callee.params.get(i) {
                            result.insert(TaintFact::Tainted(param.id));
                        }
                    }
                }
                result
            }
        }
    }

    fn return_flow(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        exit_inst: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact> {
        match fact {
            TaintFact::Zero => BTreeSet::new(),
            TaintFact::Tainted(v) => {
                let mut result = BTreeSet::new();

                // If the return value is tainted, map to call site's dst.
                if exit_inst.operands.contains(v) {
                    if let Some(dst) = call_site.dst {
                        result.insert(TaintFact::Tainted(dst));
                    }
                }

                // If a parameter-derived value is tainted, map back to the
                // actual argument. This handles pointer parameters whose
                // contents were modified by the callee (e.g. C++ constructors
                // storing tainted data to this->field via GEP+Store).
                if let Some(&(func_id, param_idx)) = self.param_derived.get(v) {
                    if func_id == callee.id {
                        if let Some(actual_arg) = call_site.operands.get(param_idx) {
                            result.insert(TaintFact::Tainted(*actual_arg));
                        }
                    }
                }

                result
            }
        }
    }

    fn call_to_return_flow(
        &self,
        call_site: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact> {
        match fact {
            TaintFact::Zero => [TaintFact::Zero].into_iter().collect(),
            TaintFact::Tainted(v) => {
                // Pass through facts not related to call arguments.
                if call_site.operands.contains(v) {
                    // This fact is passed as an argument — handled by call_flow.
                    // Don't duplicate at return site.
                    return BTreeSet::new();
                }
                [fact.clone()].into_iter().collect()
            }
        }
    }
}

/// Propagate taint through instructions that copy/transform operands to dst.
fn propagate_through_operands(inst: &Instruction, fact: &TaintFact) -> BTreeSet<TaintFact> {
    if let TaintFact::Tainted(v) = fact {
        if inst.operands.contains(v) {
            let mut result: BTreeSet<TaintFact> = [fact.clone()].into_iter().collect();
            if let Some(dst) = inst.dst {
                result.insert(TaintFact::Tainted(dst));
            }
            return result;
        }
    }
    [fact.clone()].into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callgraph::CallGraph;
    use crate::icfg::Icfg;
    use crate::ifds::config::IfdsConfig;
    use crate::ifds::solver::solve_ifds;
    use saf_core::air::{AirBlock, AirParam};
    use saf_core::ids::{BlockId, InstId, ModuleId};

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

    #[test]
    fn simple_source_to_sink_taint() {
        // getenv() → system(): taint should flow from getenv dst to system arg.
        let getenv = make_declaration(2, "getenv");
        let system = make_declaration(3, "system");
        let main = make_function(
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
                    .with_operands(vec![ValueId::new(10)]),
                    Instruction::new(InstId::new(102), Operation::Ret),
                ],
            )],
        );
        let module = make_module(vec![main, getenv, system]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let sources = vec![Selector::call_to("getenv")];
        let sanitizers: Vec<Selector> = vec![];
        let problem = TaintIfdsProblem::new(&module, &sources, &sanitizers);
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Taint should be generated at getenv call.
        assert!(result.holds_at(InstId::new(100), &TaintFact::Tainted(ValueId::new(10))));
        // Taint should reach system call.
        assert!(result.holds_at(InstId::new(101), &TaintFact::Tainted(ValueId::new(10))));
    }

    #[test]
    fn interprocedural_taint_through_helper() {
        // getenv() → helper(x) → system(result)
        let getenv = make_declaration(3, "getenv");
        let system = make_declaration(4, "system");
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
                    Instruction::new(
                        InstId::new(102),
                        Operation::CallDirect {
                            callee: FunctionId::new(4),
                        },
                    )
                    .with_operands(vec![ValueId::new(11)]),
                    Instruction::new(InstId::new(103), Operation::Ret),
                ],
            )],
        );
        let module = make_module(vec![main, helper, getenv, system]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let sources = vec![Selector::call_to("getenv")];
        let sanitizers: Vec<Selector> = vec![];
        let problem = TaintIfdsProblem::new(&module, &sources, &sanitizers);
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Taint at getenv call.
        assert!(result.holds_at(InstId::new(100), &TaintFact::Tainted(ValueId::new(10))));
        // Taint propagated through helper and returned.
        assert!(result.holds_at(InstId::new(102), &TaintFact::Tainted(ValueId::new(11))));
    }

    #[test]
    fn sanitizer_kills_taint() {
        // getenv() → sanitize() → system(): taint should be killed.
        let getenv = make_declaration(2, "getenv");
        let sanitize = make_declaration(3, "sanitize");
        let system = make_declaration(4, "system");
        let main = make_function(
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
                    Instruction::new(
                        InstId::new(102),
                        Operation::CallDirect {
                            callee: FunctionId::new(4),
                        },
                    )
                    .with_operands(vec![ValueId::new(11)]),
                    Instruction::new(InstId::new(103), Operation::Ret),
                ],
            )],
        );
        let module = make_module(vec![main, getenv, sanitize, system]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let sources = vec![Selector::call_to("getenv")];
        let sanitizers = vec![Selector::call_to("sanitize")];
        let problem = TaintIfdsProblem::new(&module, &sources, &sanitizers);
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Taint generated at getenv.
        assert!(result.holds_at(InstId::new(100), &TaintFact::Tainted(ValueId::new(10))));
        // Taint killed at sanitize — should NOT reach system call.
        assert!(!result.holds_at(InstId::new(102), &TaintFact::Tainted(ValueId::new(10))));
    }

    #[test]
    fn no_false_taint_on_clean_path() {
        // No source calls — no taint should exist.
        let module = make_module(vec![make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
                        .with_dst(ValueId::new(10)),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        )]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let sources = vec![Selector::call_to("getenv")];
        let sanitizers: Vec<Selector> = vec![];
        let problem = TaintIfdsProblem::new(&module, &sources, &sanitizers);
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Only zero fact should exist — no taint.
        for facts in result.facts.values() {
            for f in facts {
                assert!(
                    matches!(f, TaintFact::Zero),
                    "expected only Zero facts, found: {f:?}"
                );
            }
        }
    }

    #[test]
    fn multiple_independent_taint_flows() {
        // Two independent getenv calls, both should generate taint.
        let getenv = make_declaration(2, "getenv");
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
                                callee: FunctionId::new(2),
                            },
                        )
                        .with_dst(ValueId::new(11)),
                        Instruction::new(InstId::new(102), Operation::Ret),
                    ],
                )],
            ),
            getenv,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let sources = vec![Selector::call_to("getenv")];
        let sanitizers: Vec<Selector> = vec![];
        let problem = TaintIfdsProblem::new(&module, &sources, &sanitizers);
        let config = IfdsConfig::default();

        let result = solve_ifds(&problem, &icfg, &cg, &config);

        // Both taint facts should reach the ret.
        assert!(result.holds_at(InstId::new(102), &TaintFact::Tainted(ValueId::new(10))));
        assert!(result.holds_at(InstId::new(102), &TaintFact::Tainted(ValueId::new(11))));
    }
}
