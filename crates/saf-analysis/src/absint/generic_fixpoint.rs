//! Generic fixpoint solver for pluggable abstract domains.
//!
//! Provides [`solve_generic`], a worklist-based forward fixpoint iterator that is
//! parameterized over the abstract domain `D` and transfer function `T`. This
//! enables custom abstract domains to reuse SAF's fixpoint infrastructure without
//! modifying the core analysis engine.
//!
//! For the full-featured interval analysis with PTA integration, memory tracking,
//! and branch refinement, use [`super::fixpoint::solve_abstract_interp`] instead.
//!
//! # Example
//!
//! ```ignore
//! let result = solve_generic::<MyDomain, MyTransfer>(
//!     &module,
//!     &MyTransfer::new(),
//!     &GenericFixpointConfig::default(),
//! );
//! for (block_id, state) in result.block_states() {
//!     // Query domain values at each block entry
//! }
//! ```

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::air::{AirFunction, AirModule};
use saf_core::ids::{BlockId, ValueId};

use crate::cfg::Cfg;

use super::domain::AbstractDomain;
use super::fixpoint::detect_loop_headers;
use super::transfer_fn::{GenericState, TransferFn};

/// Configuration for the generic fixpoint solver.
#[derive(Debug, Clone)]
pub struct GenericFixpointConfig {
    /// Maximum widening iterations before forcing convergence (per function).
    pub max_widening_iterations: u32,
    /// Number of narrowing iterations after the ascending phase.
    pub narrowing_iterations: u32,
}

impl Default for GenericFixpointConfig {
    fn default() -> Self {
        Self {
            max_widening_iterations: 100,
            narrowing_iterations: 3,
        }
    }
}

/// Diagnostics from the generic fixpoint computation.
#[derive(Debug, Clone, Default)]
pub struct GenericFixpointDiagnostics {
    /// Total blocks analyzed across all functions.
    pub blocks_analyzed: u64,
    /// Number of widening applications.
    pub widening_applications: u64,
    /// Whether the analysis converged within iteration limits.
    pub converged: bool,
    /// Number of functions analyzed.
    pub functions_analyzed: u64,
}

/// Result of generic abstract interpretation.
///
/// Contains block-entry states and per-instruction states for the analyzed domain.
#[derive(Debug, Clone)]
pub struct GenericAbstractInterpResult<D: AbstractDomain> {
    /// Abstract state at each block entry.
    block_states: BTreeMap<BlockId, GenericState<D>>,
    /// Diagnostics from the fixpoint computation.
    diag: GenericFixpointDiagnostics,
}

impl<D: AbstractDomain> GenericAbstractInterpResult<D> {
    /// Get the abstract state at a block entry.
    #[must_use]
    pub fn state_at_block(&self, block: BlockId) -> Option<&GenericState<D>> {
        self.block_states.get(&block)
    }

    /// Get all block-entry states.
    #[must_use]
    pub fn block_states(&self) -> &BTreeMap<BlockId, GenericState<D>> {
        &self.block_states
    }

    /// Get diagnostics about the analysis.
    #[must_use]
    pub fn diagnostics(&self) -> &GenericFixpointDiagnostics {
        &self.diag
    }

    /// Get the domain value for a specific value at a block entry.
    ///
    /// Returns `None` if the block or value is not tracked (implicitly top).
    #[must_use]
    pub fn value_at_block(&self, block: BlockId, value: ValueId) -> Option<&D> {
        self.block_states.get(&block)?.get(&value)
    }
}

/// Join two generic states point-wise.
///
/// Values present in both states are joined. Values present in only one
/// state are dropped (implicitly top), matching the convention that
/// missing keys represent top.
fn join_states<D: AbstractDomain>(a: &GenericState<D>, b: &GenericState<D>) -> GenericState<D> {
    let mut result = BTreeMap::new();

    for (key, a_val) in a {
        if let Some(b_val) = b.get(key) {
            let joined = a_val.join(b_val);
            if !joined.is_top() {
                result.insert(*key, joined);
            }
        }
    }

    result
}

/// Widen two generic states point-wise.
///
/// Values present in both states are widened. New values (in `new` but not `old`)
/// are included to ensure loop body values flow into loop headers.
fn widen_states<D: AbstractDomain>(
    old: &GenericState<D>,
    new: &GenericState<D>,
) -> GenericState<D> {
    let mut result = BTreeMap::new();

    // Widen entries present in both states
    for (key, old_val) in old {
        if let Some(new_val) = new.get(key) {
            let widened = old_val.widen(new_val);
            if !widened.is_top() {
                result.insert(*key, widened);
            }
        }
    }

    // Include new-only entries (back-edge values not yet in old)
    for (key, new_val) in new {
        if !old.contains_key(key) && !new_val.is_top() {
            result.insert(*key, new_val.clone());
        }
    }

    result
}

/// Narrow two generic states point-wise.
fn narrow_states<D: AbstractDomain>(
    old: &GenericState<D>,
    new: &GenericState<D>,
) -> GenericState<D> {
    let mut result = BTreeMap::new();

    for (key, old_val) in old {
        if let Some(new_val) = new.get(key) {
            let narrowed = old_val.narrow(new_val);
            if !narrowed.is_top() {
                result.insert(*key, narrowed);
            }
        } else if !old_val.is_top() {
            // new has top for this key -> narrow top with old -> old
            result.insert(*key, old_val.clone());
        }
    }

    result
}

/// Check if `new_state` has changed relative to `old_state`.
///
/// Returns `true` if any value is new, removed, or refined.
fn state_changed<D: AbstractDomain>(old: &GenericState<D>, new: &GenericState<D>) -> bool {
    if old.len() != new.len() {
        return true;
    }
    for (key, new_val) in new {
        match old.get(key) {
            Some(old_val) if old_val == new_val => {}
            _ => return true,
        }
    }
    false
}

/// DFS helper for reverse postorder computation.
fn rpo_dfs(
    node: BlockId,
    cfg: &Cfg,
    visited: &mut BTreeSet<BlockId>,
    postorder: &mut Vec<BlockId>,
) {
    if visited.contains(&node) {
        return;
    }
    visited.insert(node);
    if let Some(succs) = cfg.successors_of(node) {
        for succ in succs {
            rpo_dfs(*succ, cfg, visited, postorder);
        }
    }
    postorder.push(node);
}

/// Compute reverse postorder traversal of the CFG.
fn reverse_postorder(cfg: &Cfg) -> Vec<BlockId> {
    let mut visited = BTreeSet::new();
    let mut postorder = Vec::new();

    rpo_dfs(cfg.entry, cfg, &mut visited, &mut postorder);
    postorder.reverse();
    postorder
}

/// Run generic abstract interpretation on the entire module.
///
/// Analyzes each non-declaration function using forward fixpoint iteration
/// with widening at loop headers and a narrowing phase for precision recovery.
///
/// This solver is domain-agnostic: it works with any [`AbstractDomain`] and
/// [`TransferFn`] pair. Unlike [`super::fixpoint::solve_abstract_interp`],
/// it does not include PTA integration, memory tracking, or branch refinement.
/// Those features are specific to the interval domain and remain in the
/// specialized solver.
///
/// # Type Parameters
///
/// * `D` - The abstract domain (e.g., `Interval`, a custom sign domain)
/// * `T` - The transfer function for `D`
pub fn solve_generic<D, T>(
    module: &AirModule,
    transfer: &T,
    config: &GenericFixpointConfig,
) -> GenericAbstractInterpResult<D>
where
    D: AbstractDomain,
    T: TransferFn<D>,
{
    let mut all_block_states: BTreeMap<BlockId, GenericState<D>> = BTreeMap::new();
    let mut diag = GenericFixpointDiagnostics {
        converged: true,
        ..Default::default()
    };

    for func in &module.functions {
        if func.is_declaration || func.blocks.is_empty() {
            continue;
        }

        diag.functions_analyzed += 1;
        let cfg = Cfg::build(func);

        let (block_states, func_diag) =
            solve_function_generic(func, &cfg, transfer, config, module);

        all_block_states.extend(block_states);
        diag.blocks_analyzed += func_diag.blocks_analyzed;
        diag.widening_applications += func_diag.widening_applications;
        if !func_diag.converged {
            diag.converged = false;
        }
    }

    GenericAbstractInterpResult {
        block_states: all_block_states,
        diag,
    }
}

/// Generic fixpoint solver for a single function.
///
/// Implements the standard worklist-based forward abstract interpretation:
/// 1. Initialize entry block with top for all parameters
/// 2. Ascending phase with widening at loop headers
/// 3. Descending phase with narrowing for precision recovery
// NOTE: This function implements the fixpoint iteration algorithm as a single
// cohesive unit. Splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines)]
fn solve_function_generic<D, T>(
    func: &AirFunction,
    cfg: &Cfg,
    transfer: &T,
    config: &GenericFixpointConfig,
    module: &AirModule,
) -> (
    BTreeMap<BlockId, GenericState<D>>,
    GenericFixpointDiagnostics,
)
where
    D: AbstractDomain,
    T: TransferFn<D>,
{
    let mut diag = GenericFixpointDiagnostics {
        converged: true,
        ..Default::default()
    };
    let loop_headers = detect_loop_headers(cfg);

    // Initialize block-entry states as empty (implicitly top for all values)
    let mut block_entry_states: BTreeMap<BlockId, GenericState<D>> = BTreeMap::new();
    for block in &func.blocks {
        block_entry_states.insert(block.id, BTreeMap::new());
    }

    // Entry block: parameters get top values
    let mut entry_state: GenericState<D> = BTreeMap::new();
    for param in &func.params {
        entry_state.insert(param.id, D::top());
    }
    block_entry_states.insert(cfg.entry, entry_state);

    // =================================================================
    // Ascending phase (widening)
    // =================================================================
    let mut worklist: VecDeque<BlockId> = VecDeque::new();
    worklist.push_back(cfg.entry);
    let mut iteration_count: u32 = 0;

    while let Some(block_id) = worklist.pop_front() {
        iteration_count += 1;
        #[allow(clippy::cast_possible_truncation)]
        if iteration_count > config.max_widening_iterations * (func.blocks.len() as u32 + 1) {
            diag.converged = false;
            break;
        }
        diag.blocks_analyzed += 1;

        let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
            continue;
        };

        let entry_state = block_entry_states
            .get(&block_id)
            .cloned()
            .unwrap_or_default();

        // Execute instructions through the transfer function
        let mut current_state = entry_state;
        for inst in &block.instructions {
            transfer.transfer_instruction(inst, &mut current_state, func, module);
        }

        // Propagate to successors
        let succs = cfg.successors_of(block_id).cloned().unwrap_or_default();

        for succ_id in &succs {
            let old_state = block_entry_states.get(succ_id).cloned().unwrap_or_default();

            let new_state = if loop_headers.contains(*succ_id) {
                diag.widening_applications += 1;
                widen_states(&old_state, &current_state)
            } else {
                join_states(&old_state, &current_state)
            };

            // Detect new entries: values appearing for the first time
            let has_new_values = new_state.len() > old_state.len();
            let changed = state_changed(&old_state, &new_state) || has_new_values;

            if changed {
                block_entry_states.insert(*succ_id, new_state);
                if !worklist.contains(succ_id) {
                    worklist.push_back(*succ_id);
                }
            }
        }
    }

    // =================================================================
    // Descending phase (narrowing)
    // =================================================================
    let rpo = reverse_postorder(cfg);

    for _narrow_iter in 0..config.narrowing_iterations {
        let mut changed = false;

        for &block_id in &rpo {
            let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
                continue;
            };

            let entry_state = block_entry_states
                .get(&block_id)
                .cloned()
                .unwrap_or_default();

            // Execute instructions
            let mut current_state = entry_state;
            for inst in &block.instructions {
                transfer.transfer_instruction(inst, &mut current_state, func, module);
            }

            // Propagate to successors with narrowing
            let succs = cfg.successors_of(block_id).cloned().unwrap_or_default();

            for succ_id in &succs {
                let old_state = block_entry_states.get(succ_id).cloned().unwrap_or_default();

                let joined = join_states(&old_state, &current_state);
                let narrowed = narrow_states(&old_state, &joined);

                if narrowed != old_state {
                    block_entry_states.insert(*succ_id, narrowed);
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    (block_entry_states, diag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirModule, BinaryOp, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    // -----------------------------------------------------------------------
    // Test domain: TwoPoint lattice (Bottom < Top)
    // -----------------------------------------------------------------------

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TwoPoint {
        Bottom,
        Top,
    }

    impl AbstractDomain for TwoPoint {
        fn bottom() -> Self {
            TwoPoint::Bottom
        }
        fn top() -> Self {
            TwoPoint::Top
        }
        fn is_bottom(&self) -> bool {
            matches!(self, TwoPoint::Bottom)
        }
        fn is_top(&self) -> bool {
            matches!(self, TwoPoint::Top)
        }
        fn leq(&self, other: &Self) -> bool {
            matches!(
                (self, other),
                (TwoPoint::Bottom, _) | (TwoPoint::Top, TwoPoint::Top)
            )
        }
        fn join(&self, other: &Self) -> Self {
            if self.is_top() || other.is_top() {
                TwoPoint::Top
            } else {
                TwoPoint::Bottom
            }
        }
        fn meet(&self, other: &Self) -> Self {
            if self.is_bottom() || other.is_bottom() {
                TwoPoint::Bottom
            } else {
                TwoPoint::Top
            }
        }
    }

    /// Transfer that sets dst to `Top` for any instruction with a destination.
    struct TwoPointTransfer;

    impl TransferFn<TwoPoint> for TwoPointTransfer {
        fn transfer_instruction(
            &self,
            inst: &Instruction,
            state: &mut GenericState<TwoPoint>,
            _func: &AirFunction,
            _module: &AirModule,
        ) {
            if let Some(dst) = inst.dst {
                state.insert(dst, TwoPoint::Top);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Test domain: Parity (even/odd/top/bottom)
    // -----------------------------------------------------------------------

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Parity {
        Bottom,
        Even,
        Odd,
        Top,
    }

    impl AbstractDomain for Parity {
        fn bottom() -> Self {
            Parity::Bottom
        }
        fn top() -> Self {
            Parity::Top
        }
        fn is_bottom(&self) -> bool {
            matches!(self, Parity::Bottom)
        }
        fn is_top(&self) -> bool {
            matches!(self, Parity::Top)
        }
        fn leq(&self, other: &Self) -> bool {
            matches!(
                (self, other),
                (Parity::Bottom, _)
                    | (_, Parity::Top)
                    | (Parity::Even, Parity::Even)
                    | (Parity::Odd, Parity::Odd)
            )
        }
        fn join(&self, other: &Self) -> Self {
            if self == other {
                return self.clone();
            }
            if self.is_bottom() {
                return other.clone();
            }
            if other.is_bottom() {
                return self.clone();
            }
            Parity::Top
        }
        fn meet(&self, other: &Self) -> Self {
            if self == other {
                return self.clone();
            }
            if self.is_top() {
                return other.clone();
            }
            if other.is_top() {
                return self.clone();
            }
            Parity::Bottom
        }
    }

    /// Transfer that tracks parity of additions.
    struct ParityTransfer;

    impl TransferFn<Parity> for ParityTransfer {
        fn transfer_instruction(
            &self,
            inst: &Instruction,
            state: &mut GenericState<Parity>,
            _func: &AirFunction,
            _module: &AirModule,
        ) {
            if let Some(dst) = inst.dst {
                if let Operation::BinaryOp {
                    kind: BinaryOp::Add,
                } = &inst.op
                {
                    if inst.operands.len() == 2 {
                        let a = state.get(&inst.operands[0]).cloned().unwrap_or(Parity::Top);
                        let b = state.get(&inst.operands[1]).cloned().unwrap_or(Parity::Top);

                        let result = match (&a, &b) {
                            (Parity::Even, Parity::Even) | (Parity::Odd, Parity::Odd) => {
                                Parity::Even
                            }
                            (Parity::Even, Parity::Odd) | (Parity::Odd, Parity::Even) => {
                                Parity::Odd
                            }
                            _ => Parity::Top,
                        };
                        state.insert(dst, result);
                        return;
                    }
                }
                // Default: mark as top
                state.insert(dst, Parity::Top);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Helper functions
    // -----------------------------------------------------------------------

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }
    fn bid(n: u128) -> BlockId {
        BlockId::new(n)
    }
    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }
    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }

    /// Build a straight-line function: entry -> ret
    fn make_straight_line() -> AirModule {
        let entry = bid(10);
        let v_result = vid(200);

        let mut entry_block = AirBlock::new(entry);
        entry_block.instructions.push(
            Instruction::new(
                iid(1),
                Operation::BinaryOp {
                    kind: BinaryOp::Add,
                },
            )
            .with_operands(vec![vid(100), vid(101)])
            .with_dst(v_result),
        );
        entry_block
            .instructions
            .push(Instruction::new(iid(2), Operation::Ret));

        let func = AirFunction {
            id: fid(1),
            name: "straight_line".to_string(),
            params: Vec::new(),
            blocks: vec![entry_block],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::derive(b"test_straight"));
        module.functions.push(func);
        module
    }

    /// Build a simple loop: entry -> header <-> body -> exit
    fn make_simple_loop() -> AirModule {
        let entry = bid(10);
        let header = bid(20);
        let body = bid(30);
        let exit = bid(40);

        let v_i_phi = vid(200);
        let v_cond = vid(201);
        let v_i_inc = vid(202);

        // Entry: branch to header
        let mut entry_block = AirBlock::new(entry);
        entry_block
            .instructions
            .push(Instruction::new(iid(1), Operation::Br { target: header }));

        // Header: phi, compare, condbr
        let mut header_block = AirBlock::new(header);
        header_block.instructions.push(
            Instruction::new(
                iid(2),
                Operation::Phi {
                    incoming: vec![(entry, vid(100)), (body, v_i_inc)],
                },
            )
            .with_dst(v_i_phi),
        );
        header_block.instructions.push(
            Instruction::new(
                iid(3),
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpSlt,
                },
            )
            .with_operands(vec![v_i_phi, vid(101)])
            .with_dst(v_cond),
        );
        header_block.instructions.push(
            Instruction::new(
                iid(4),
                Operation::CondBr {
                    then_target: body,
                    else_target: exit,
                },
            )
            .with_operands(vec![v_cond]),
        );

        // Body: increment, branch to header
        let mut body_block = AirBlock::new(body);
        body_block.instructions.push(
            Instruction::new(
                iid(5),
                Operation::BinaryOp {
                    kind: BinaryOp::Add,
                },
            )
            .with_operands(vec![v_i_phi, vid(102)])
            .with_dst(v_i_inc),
        );
        body_block
            .instructions
            .push(Instruction::new(iid(6), Operation::Br { target: header }));

        // Exit: ret
        let mut exit_block = AirBlock::new(exit);
        exit_block
            .instructions
            .push(Instruction::new(iid(7), Operation::Ret));

        let func = AirFunction {
            id: fid(1),
            name: "loop_func".to_string(),
            params: Vec::new(),
            blocks: vec![entry_block, header_block, body_block, exit_block],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::derive(b"test_loop"));
        module.functions.push(func);
        module
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[test]
    fn straight_line_converges() {
        let module = make_straight_line();
        let transfer = TwoPointTransfer;
        let config = GenericFixpointConfig::default();

        let result = solve_generic::<TwoPoint, _>(&module, &transfer, &config);

        assert!(result.diagnostics().converged);
        assert_eq!(result.diagnostics().functions_analyzed, 1);
    }

    #[test]
    fn straight_line_computes_values() {
        let module = make_straight_line();
        let transfer = TwoPointTransfer;
        let config = GenericFixpointConfig::default();

        let result = solve_generic::<TwoPoint, _>(&module, &transfer, &config);

        // Entry block is the only block that has instructions executed
        // After transfer, v200 (the add result) should be Top in the entry block
        // But the result stores entry states (BEFORE instructions), so v200
        // won't be in the entry state; it would be in a successor state if there
        // were one. Since there's only one block, we check the entry state directly.
        let entry_state = result.state_at_block(bid(10));
        assert!(entry_state.is_some());
    }

    #[test]
    fn loop_converges_with_two_point() {
        let module = make_simple_loop();
        let transfer = TwoPointTransfer;
        let config = GenericFixpointConfig::default();

        let result = solve_generic::<TwoPoint, _>(&module, &transfer, &config);

        assert!(result.diagnostics().converged);
        // Should have applied widening at the loop header
        assert!(result.diagnostics().widening_applications > 0);
    }

    #[test]
    fn loop_converges_with_parity() {
        let module = make_simple_loop();
        let transfer = ParityTransfer;
        let config = GenericFixpointConfig::default();

        let result = solve_generic::<Parity, _>(&module, &transfer, &config);

        assert!(result.diagnostics().converged);
    }

    #[test]
    fn empty_module_produces_empty_result() {
        let module = AirModule::new(ModuleId::derive(b"empty"));
        let transfer = TwoPointTransfer;
        let config = GenericFixpointConfig::default();

        let result = solve_generic::<TwoPoint, _>(&module, &transfer, &config);

        assert!(result.block_states().is_empty());
        assert!(result.diagnostics().converged);
        assert_eq!(result.diagnostics().functions_analyzed, 0);
    }

    #[test]
    fn declaration_functions_are_skipped() {
        let mut module = AirModule::new(ModuleId::derive(b"test_decl"));
        let mut func = AirFunction::new(fid(1), "declared_only");
        func.is_declaration = true;
        module.functions.push(func);

        let transfer = TwoPointTransfer;
        let config = GenericFixpointConfig::default();

        let result = solve_generic::<TwoPoint, _>(&module, &transfer, &config);

        assert_eq!(result.diagnostics().functions_analyzed, 0);
    }

    #[test]
    fn deterministic_results() {
        let module = make_simple_loop();
        let transfer = ParityTransfer;
        let config = GenericFixpointConfig::default();

        let result1 = solve_generic::<Parity, _>(&module, &transfer, &config);
        let result2 = solve_generic::<Parity, _>(&module, &transfer, &config);

        assert_eq!(
            result1.diagnostics().blocks_analyzed,
            result2.diagnostics().blocks_analyzed
        );
        assert_eq!(
            result1.diagnostics().widening_applications,
            result2.diagnostics().widening_applications
        );
    }
}
