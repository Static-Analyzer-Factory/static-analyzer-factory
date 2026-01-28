//! Generic transfer function trait for abstract interpretation.
//!
//! The [`TransferFn`] trait defines how instructions transform abstract states,
//! allowing custom abstract domains to plug into the generic fixpoint solver.
//! [`IntervalTransfer`] wraps the existing interval transfer function as the
//! built-in implementation.

use std::collections::BTreeMap;

use saf_core::air::{AirFunction, AirModule, Instruction};
use saf_core::ids::ValueId;

use super::domain::AbstractDomain;
use super::interval::Interval;
use super::state::AbstractState;
use super::transfer::{TransferContext, build_constant_map, transfer_instruction_with_context};

/// A generic abstract state mapping values to domain elements.
///
/// This is the generic version of [`super::state::AbstractState`], parameterized
/// over the abstract domain. Values not in the map are implicitly top (unknown).
pub type GenericState<D> = BTreeMap<ValueId, D>;

/// Trait for abstract transfer functions over a specific domain.
///
/// Implementors define how each AIR instruction transforms the abstract state.
/// The fixpoint solver calls `transfer_instruction` for each instruction in a
/// basic block, threading the state through sequentially.
///
/// # Type Parameters
///
/// * `D` - The abstract domain type (e.g., [`super::interval::Interval`])
///
/// # Example
///
/// ```ignore
/// struct MyTransfer;
///
/// impl TransferFn<MyDomain> for MyTransfer {
///     fn transfer_instruction(
///         &self,
///         inst: &Instruction,
///         state: &mut GenericState<MyDomain>,
///         func: &AirFunction,
///         module: &AirModule,
///     ) {
///         // Apply domain-specific semantics for each instruction
///         match &inst.op {
///             Operation::BinaryOp { kind } => { /* ... */ }
///             _ => {}
///         }
///     }
/// }
/// ```
pub trait TransferFn<D: AbstractDomain>: Send + Sync {
    /// Apply the transfer function for a single instruction.
    ///
    /// Mutates `state` in place to reflect the effect of executing `inst`.
    /// The function and module context are provided for looking up
    /// constants, function signatures, and other contextual information.
    fn transfer_instruction(
        &self,
        inst: &Instruction,
        state: &mut GenericState<D>,
        func: &AirFunction,
        module: &AirModule,
    );
}

/// Adapter that wraps the existing interval transfer function as a [`TransferFn`].
///
/// This bridges the specialized [`transfer_instruction_with_context`] (which
/// operates on [`AbstractState`]) into the generic trait interface. It converts
/// the `GenericState<Interval>` to/from `AbstractState` around each call.
///
/// For full-featured interval analysis with memory tracking and PTA integration,
/// use the specialized [`transfer_instruction_with_context`] directly. This
/// wrapper provides the value-level (SSA register) transfer semantics through
/// the generic interface, which is sufficient for simple intraprocedural analysis.
pub struct IntervalTransfer {
    /// Pre-computed constant map from the module.
    constant_map: BTreeMap<ValueId, Interval>,
}

impl IntervalTransfer {
    /// Create a new `IntervalTransfer` for the given module.
    ///
    /// Pre-computes the constant map from the module's constant table.
    #[must_use]
    pub fn new(module: &AirModule) -> Self {
        Self {
            constant_map: build_constant_map(module),
        }
    }

    /// Create an `IntervalTransfer` with a pre-computed constant map.
    #[must_use]
    pub fn with_constant_map(constant_map: BTreeMap<ValueId, Interval>) -> Self {
        Self { constant_map }
    }
}

impl TransferFn<Interval> for IntervalTransfer {
    fn transfer_instruction(
        &self,
        inst: &Instruction,
        state: &mut GenericState<Interval>,
        _func: &AirFunction,
        module: &AirModule,
    ) {
        // Build a temporary AbstractState from the generic state map
        let mut abs_state = AbstractState::new();
        for (vid, interval) in state.iter() {
            abs_state.set(*vid, interval.clone());
        }

        // Delegate to the existing transfer function with default (no-PTA) context
        let ctx = TransferContext::default();
        transfer_instruction_with_context(inst, &mut abs_state, &self.constant_map, module, &ctx);

        // Sync changes back: update state with any values that changed
        // We iterate over the AbstractState entries and update the generic state.
        // New entries are inserted; existing entries are updated.
        for (vid, interval) in abs_state.entries() {
            state.insert(*vid, interval.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirFunction, AirModule, BinaryOp, Constant, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    // =========================================================================
    // Generic TransferFn trait tests (custom domain)
    // =========================================================================

    /// Simple counting domain: tracks whether a value has been assigned.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Assigned {
        /// Not yet assigned (bottom).
        Unassigned,
        /// Has been assigned.
        Assigned,
    }

    impl AbstractDomain for Assigned {
        fn bottom() -> Self {
            Assigned::Unassigned
        }
        fn top() -> Self {
            Assigned::Assigned
        }
        fn is_bottom(&self) -> bool {
            matches!(self, Assigned::Unassigned)
        }
        fn is_top(&self) -> bool {
            matches!(self, Assigned::Assigned)
        }
        fn leq(&self, other: &Self) -> bool {
            matches!(
                (self, other),
                (Assigned::Unassigned, _) | (Assigned::Assigned, Assigned::Assigned)
            )
        }
        fn join(&self, other: &Self) -> Self {
            if self.is_bottom() {
                return other.clone();
            }
            if other.is_bottom() {
                return self.clone();
            }
            Assigned::Assigned
        }
        fn meet(&self, other: &Self) -> Self {
            if self.is_top() {
                return other.clone();
            }
            if other.is_top() {
                return self.clone();
            }
            Assigned::Unassigned
        }
    }

    /// Transfer function that marks destination values as `Assigned`.
    struct AssignedTransfer;

    impl TransferFn<Assigned> for AssignedTransfer {
        fn transfer_instruction(
            &self,
            inst: &Instruction,
            state: &mut GenericState<Assigned>,
            _func: &AirFunction,
            _module: &AirModule,
        ) {
            if let Some(dst) = inst.dst {
                state.insert(dst, Assigned::Assigned);
            }
        }
    }

    fn make_test_func() -> AirFunction {
        AirFunction::new(FunctionId::new(1), "test")
    }

    fn make_test_module() -> AirModule {
        AirModule::new(ModuleId::derive(b"test"))
    }

    #[test]
    fn transfer_fn_marks_assigned() {
        let func = make_test_func();
        let module = make_test_module();

        let transfer = AssignedTransfer;
        let mut state: GenericState<Assigned> = BTreeMap::new();

        // Instruction that assigns to v100
        let inst = Instruction::new(
            InstId::new(1),
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
        )
        .with_operands(vec![ValueId::new(10), ValueId::new(11)])
        .with_dst(ValueId::new(100));

        transfer.transfer_instruction(&inst, &mut state, &func, &module);

        assert_eq!(state.get(&ValueId::new(100)), Some(&Assigned::Assigned));
        // Operands are not assigned by this instruction
        assert_eq!(state.get(&ValueId::new(10)), None);
    }

    #[test]
    fn transfer_fn_no_dst_leaves_state_unchanged() {
        let func = make_test_func();
        let module = make_test_module();

        let transfer = AssignedTransfer;
        let mut state: GenericState<Assigned> = BTreeMap::new();

        // Instruction with no destination (e.g., branch)
        let inst = Instruction::new(
            InstId::new(1),
            Operation::Br {
                target: BlockId::new(2),
            },
        );

        transfer.transfer_instruction(&inst, &mut state, &func, &module);

        assert!(state.is_empty());
    }

    // =========================================================================
    // IntervalTransfer wrapper tests
    // =========================================================================

    /// Build a module with two integer constants for testing.
    fn make_module_with_constants() -> AirModule {
        let mut module = AirModule::new(ModuleId::derive(b"test"));
        // Register constants: v10 = 3, v11 = 7
        module
            .constants
            .insert(ValueId::new(10), Constant::Int { value: 3, bits: 32 });
        module
            .constants
            .insert(ValueId::new(11), Constant::Int { value: 7, bits: 32 });
        module
    }

    #[test]
    fn interval_transfer_add_constants() {
        let module = make_module_with_constants();
        let func = make_test_func();
        let transfer = IntervalTransfer::new(&module);
        let mut state: GenericState<Interval> = BTreeMap::new();

        // v100 = add v10(=3), v11(=7) => [10, 10]
        let inst = Instruction::new(
            InstId::new(1),
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
        )
        .with_operands(vec![ValueId::new(10), ValueId::new(11)])
        .with_dst(ValueId::new(100));

        transfer.transfer_instruction(&inst, &mut state, &func, &module);

        let result = state.get(&ValueId::new(100));
        assert!(result.is_some(), "destination should have an interval");
        let interval = result.unwrap();
        assert_eq!(interval.lo(), 10);
        assert_eq!(interval.hi(), 10);
    }

    #[test]
    fn interval_transfer_preserves_existing_state() {
        let module = make_module_with_constants();
        let func = make_test_func();
        let transfer = IntervalTransfer::new(&module);

        let mut state: GenericState<Interval> = BTreeMap::new();
        // Pre-existing value in state
        state.insert(ValueId::new(50), Interval::new(0, 100, 32));

        // v100 = add v10(=3), v11(=7)
        let inst = Instruction::new(
            InstId::new(1),
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
        )
        .with_operands(vec![ValueId::new(10), ValueId::new(11)])
        .with_dst(ValueId::new(100));

        transfer.transfer_instruction(&inst, &mut state, &func, &module);

        // Pre-existing value should still be there
        assert!(state.contains_key(&ValueId::new(50)));
        // New value should be computed
        assert!(state.contains_key(&ValueId::new(100)));
    }

    #[test]
    fn interval_transfer_send_sync() {
        // Verify that IntervalTransfer satisfies Send + Sync (required by trait bound)
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<IntervalTransfer>();
    }
}
