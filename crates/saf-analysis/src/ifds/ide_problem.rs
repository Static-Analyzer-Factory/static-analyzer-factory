//! IDE problem trait definition.
//!
//! The `IdeProblem` trait extends [`IfdsProblem`] with edge functions and a value
//! lattice, enabling analyses that track not just "which facts hold" but "what
//! value does each fact have." This follows the IDE framework from Sagiv, Reps &
//! Horwitz (TCS'96).

use saf_core::air::{AirFunction, Instruction};

use super::edge_fn::BuiltinEdgeFn;
use super::lattice::Lattice;
use super::problem::IfdsProblem;

/// An IDE (Interprocedural Distributive Environment) problem.
///
/// Extends [`IfdsProblem`] with:
/// - A value lattice `V` associated with each data-flow fact
/// - Four edge function factories mapping program edges to `L -> L` functions
///
/// # Edge Function Categories
///
/// - `normal_edge_fn` — non-call intra-procedural statements
/// - `call_edge_fn` — caller to callee entry
/// - `return_edge_fn` — callee exit to caller return point
/// - `call_to_return_edge_fn` — facts that bypass the callee
///
/// Each factory returns a [`BuiltinEdgeFn<V>`] describing how the value
/// transforms along that edge. The solver composes these along paths and
/// joins them at merge points.
pub trait IdeProblem: IfdsProblem {
    /// The value lattice type.
    type Value: Lattice;

    /// Edge function for normal (non-call) intra-procedural flow.
    ///
    /// Given the instruction, source fact, and successor fact (from `normal_flow`),
    /// returns the edge function describing how the value transforms.
    fn normal_edge_fn(
        &self,
        inst: &Instruction,
        src_fact: &Self::Fact,
        succ_fact: &Self::Fact,
    ) -> BuiltinEdgeFn<Self::Value>;

    /// Edge function for call flow (caller to callee entry).
    fn call_edge_fn(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        src_fact: &Self::Fact,
        dest_fact: &Self::Fact,
    ) -> BuiltinEdgeFn<Self::Value>;

    /// Edge function for return flow (callee exit to caller return site).
    fn return_edge_fn(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        exit_inst: &Instruction,
        exit_fact: &Self::Fact,
        ret_fact: &Self::Fact,
    ) -> BuiltinEdgeFn<Self::Value>;

    /// Edge function for call-to-return flow (bypassing the callee).
    fn call_to_return_edge_fn(
        &self,
        call_site: &Instruction,
        src_fact: &Self::Fact,
        ret_fact: &Self::Fact,
    ) -> BuiltinEdgeFn<Self::Value>;

    /// The top value of the lattice (initial value for all facts).
    fn top_value(&self) -> Self::Value;

    /// The bottom value of the lattice.
    fn bottom_value(&self) -> Self::Value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifds::lattice::Lattice;
    use crate::ifds::problem::IfdsProblem;
    use saf_core::air::{AirBlock, AirModule, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId};
    use std::collections::{BTreeMap, BTreeSet};

    /// Trivial IDE problem: all identity edge functions, two-point lattice.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TrivialValue {
        Bottom,
        Top,
    }

    impl Lattice for TrivialValue {
        fn top() -> Self {
            TrivialValue::Top
        }
        fn bottom() -> Self {
            TrivialValue::Bottom
        }
        fn join(&self, other: &Self) -> Self {
            if *self == TrivialValue::Top || *other == TrivialValue::Top {
                TrivialValue::Top
            } else {
                TrivialValue::Bottom
            }
        }
        fn meet(&self, other: &Self) -> Self {
            if *self == TrivialValue::Bottom || *other == TrivialValue::Bottom {
                TrivialValue::Bottom
            } else {
                TrivialValue::Top
            }
        }
        fn leq(&self, other: &Self) -> bool {
            self.join(other) == *other
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TrivialFact {
        Zero,
    }

    struct TrivialIdeProblem {
        module: AirModule,
    }

    impl IfdsProblem for TrivialIdeProblem {
        type Fact = TrivialFact;

        fn zero_value(&self) -> Self::Fact {
            TrivialFact::Zero
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

    impl IdeProblem for TrivialIdeProblem {
        type Value = TrivialValue;

        fn normal_edge_fn(
            &self,
            _inst: &Instruction,
            _src_fact: &Self::Fact,
            _succ_fact: &Self::Fact,
        ) -> BuiltinEdgeFn<Self::Value> {
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
            TrivialValue::Top
        }

        fn bottom_value(&self) -> Self::Value {
            TrivialValue::Bottom
        }
    }

    fn make_module() -> AirModule {
        AirModule {
            id: ModuleId::derive(b"test_ide"),
            name: Some("test_ide".to_string()),
            functions: vec![saf_core::air::AirFunction {
                id: FunctionId::new(1),
                name: "main".to_string(),
                params: Vec::new(),
                blocks: vec![AirBlock {
                    id: BlockId::new(1),
                    label: None,
                    instructions: vec![Instruction::new(InstId::new(1), Operation::Ret)],
                }],
                entry_block: None,
                is_declaration: false,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            }],
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

    #[test]
    fn trivial_ide_problem_compiles() {
        let module = make_module();
        let problem = TrivialIdeProblem { module };
        assert_eq!(problem.zero_value(), TrivialFact::Zero);
        assert_eq!(problem.top_value(), TrivialValue::Top);
        assert_eq!(problem.bottom_value(), TrivialValue::Bottom);
    }

    #[test]
    fn trivial_ide_edge_fns_are_identity() {
        let module = make_module();
        let problem = TrivialIdeProblem { module };
        let inst = Instruction::new(InstId::new(1), Operation::Ret);
        let fact = TrivialFact::Zero;
        let ef = problem.normal_edge_fn(&inst, &fact, &fact);
        assert!(ef.is_identity());
    }
}
