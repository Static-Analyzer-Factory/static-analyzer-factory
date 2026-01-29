//! IFDS problem trait definition.
//!
//! The `IfdsProblem` trait encodes an interprocedural data-flow problem in the
//! IFDS (Interprocedural Finite Distributive Subset) framework. Problems operate
//! on a finite domain D with powerset lattice and distributive flow functions.
//! The solver reduces the problem to graph reachability on an exploded supergraph.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirFunction, AirModule, Instruction};
use saf_core::ids::FunctionId;

/// A data-flow problem expressible in the IFDS framework.
///
/// IFDS problems operate on a finite domain D with powerset lattice and
/// distributive flow functions. The solver reduces each problem to graph
/// reachability on an exploded supergraph where each node is
/// `(program_point, data_flow_fact)`.
///
/// # Type Parameters
///
/// The `Fact` associated type is the domain D. It must be:
/// - `Ord` — for deterministic `BTreeSet` storage
/// - `Clone` — facts are copied during propagation
/// - `Debug` — for diagnostics and export
///
/// # Flow Function Categories
///
/// - `normal_flow` — non-call intra-procedural statements
/// - `call_flow` — caller to callee entry (argument passing)
/// - `return_flow` — callee exit to caller return point
/// - `call_to_return_flow` — facts that bypass the callee (locals not passed as args)
pub trait IfdsProblem {
    /// The data-flow fact type (domain D).
    type Fact: Ord + Clone + std::fmt::Debug;

    /// The zero (tautology) fact.
    ///
    /// This special fact is always propagated and represents "this program point
    /// is reachable." Edges from zero generate new facts; edges to zero kill facts.
    fn zero_value(&self) -> Self::Fact;

    /// Reference to the AIR module being analyzed.
    fn module(&self) -> &AirModule;

    /// Initial seeds: functions and facts alive at their entry points.
    ///
    /// The zero fact is automatically seeded at all reachable function entries;
    /// this method should return additional problem-specific initial facts.
    fn initial_seeds(&self) -> BTreeMap<FunctionId, BTreeSet<Self::Fact>>;

    /// Normal (non-call) intra-procedural flow function.
    ///
    /// Given an instruction and an incoming fact, returns the set of facts
    /// that hold after the instruction executes.
    ///
    /// To propagate the fact unchanged, include it in the returned set.
    /// To kill the fact, return an empty set.
    /// To generate new facts, include them in the returned set.
    fn normal_flow(&self, inst: &Instruction, fact: &Self::Fact) -> BTreeSet<Self::Fact>;

    /// Call flow function: facts propagated from call site to callee entry.
    ///
    /// Typically maps tainted arguments to callee parameters.
    fn call_flow(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact>;

    /// Return flow function: facts propagated from callee exit to caller return site.
    ///
    /// Typically maps tainted return values back to the call result.
    fn return_flow(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        exit_inst: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact>;

    /// Call-to-return flow function: facts that skip the callee.
    ///
    /// Handles facts about caller-local state that isn't affected by the call
    /// (e.g., local variables not passed as arguments).
    fn call_to_return_flow(
        &self,
        call_site: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirModule, Operation};
    use saf_core::ids::{BlockId, InstId, ModuleId};

    /// Minimal identity problem that propagates all facts unchanged.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum IdentityFact {
        Zero,
        Value(u64),
    }

    struct IdentityProblem {
        module: AirModule,
    }

    impl IfdsProblem for IdentityProblem {
        type Fact = IdentityFact;

        fn zero_value(&self) -> Self::Fact {
            IdentityFact::Zero
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

    fn make_module() -> AirModule {
        AirModule {
            id: ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions: vec![AirFunction {
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
    fn identity_problem_compiles_and_type_checks() {
        let module = make_module();
        let problem = IdentityProblem { module };
        assert_eq!(problem.zero_value(), IdentityFact::Zero);
        assert!(!problem.module().functions.is_empty());
    }

    #[test]
    fn identity_problem_normal_flow_propagates() {
        let module = make_module();
        let problem = IdentityProblem { module };
        let fact = IdentityFact::Value(42);
        let inst = Instruction::new(InstId::new(1), Operation::Ret);
        let result = problem.normal_flow(&inst, &fact);
        assert!(result.contains(&IdentityFact::Value(42)));
    }
}
