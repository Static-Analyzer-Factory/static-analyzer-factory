//! Nullness tracking analysis for pointer values.
//!
//! This module tracks which pointers may be null at each program point.
//! Used by PTABen's UNSAFE_LOAD/SAFE_LOAD oracles.
//!
//! # Lattice
//!
//! ```text
//!        Top (MaybeNull)
//!       /            \
//!   Null            NotNull
//!       \            /
//!        Bottom (unreachable)
//! ```

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirBlock, AirFunction, AirModule, Constant, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, LocId, ValueId};
use saf_core::spec::SpecRegistry;

use super::function_properties;
use serde::{Deserialize, Serialize};

use super::interprocedural::FunctionSummary;
use super::pta_integration::PtaIntegration;

// ---------------------------------------------------------------------------
// Nullness Domain
// ---------------------------------------------------------------------------

/// Nullness state for a pointer value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum Nullness {
    /// Unreachable or undefined.
    #[default]
    Bottom,
    /// Definitely null (e.g., `ptr = NULL`).
    Null,
    /// Definitely not null (e.g., `ptr = malloc()`).
    NotNull,
    /// May be null or not null (e.g., after branch merge).
    MaybeNull,
}

impl Nullness {
    /// Join two nullness states (least upper bound).
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        use Nullness::{Bottom, MaybeNull, NotNull, Null};
        match (self, other) {
            (Bottom, x) | (x, Bottom) => x,
            (Null, Null) => Null,
            (NotNull, NotNull) => NotNull,
            (MaybeNull, _) | (_, MaybeNull) | (Null, NotNull) | (NotNull, Null) => MaybeNull,
        }
    }

    /// Check if this state is definitely null.
    #[must_use]
    pub fn is_definitely_null(self) -> bool {
        matches!(self, Nullness::Null)
    }

    /// Check if this state is definitely not null.
    #[must_use]
    pub fn is_definitely_not_null(self) -> bool {
        matches!(self, Nullness::NotNull)
    }

    /// Check if this state may be null.
    #[must_use]
    pub fn may_be_null(self) -> bool {
        matches!(self, Nullness::Null | Nullness::MaybeNull)
    }
}

// ---------------------------------------------------------------------------
// Nullness State
// ---------------------------------------------------------------------------

/// Nullness state at a program point.
#[derive(Debug, Clone, Default)]
pub struct NullnessState {
    /// Nullness for each tracked pointer value.
    values: BTreeMap<ValueId, Nullness>,
    /// Nullness for values stored at memory locations (keyed by pointer `ValueId`).
    /// Enables tracking nullness through store/load sequences.
    memory: BTreeMap<ValueId, Nullness>,
    /// Nullness for values stored at abstract memory locations (keyed by `LocId`).
    /// Used when PTA is available for alias-aware memory tracking.
    loc_memory: BTreeMap<LocId, Nullness>,
}

impl NullnessState {
    /// Create a new empty state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the nullness of a value.
    ///
    /// Returns `MaybeNull` for untracked values (conservative assumption).
    #[must_use]
    pub fn get(&self, value: ValueId) -> Nullness {
        self.values
            .get(&value)
            .copied()
            .unwrap_or(Nullness::MaybeNull)
    }

    /// Get the nullness of a value, returning `Bottom` for untracked values.
    ///
    /// Use this when you need to distinguish between "untracked" and "may be null".
    #[must_use]
    pub fn get_raw(&self, value: ValueId) -> Nullness {
        self.values.get(&value).copied().unwrap_or(Nullness::Bottom)
    }

    /// Set the nullness of a value.
    pub fn set(&mut self, value: ValueId, nullness: Nullness) {
        self.values.insert(value, nullness);
    }

    /// Store nullness at a memory location.
    pub fn store(&mut self, ptr: ValueId, nullness: Nullness) {
        self.memory.insert(ptr, nullness);
    }

    /// Load nullness from a memory location.
    #[must_use]
    pub fn load(&self, ptr: ValueId) -> Option<Nullness> {
        self.memory.get(&ptr).copied()
    }

    /// Invalidate all memory (conservative: after function call or unknown store).
    pub fn invalidate_all_memory(&mut self) {
        self.memory.clear();
    }

    // =========================================================================
    // LocId-based memory operations (for PTA-integrated analysis)
    // =========================================================================

    /// Store nullness at an abstract memory location (strong update).
    ///
    /// Replaces any existing value at the location.
    pub fn store_loc(&mut self, loc: LocId, nullness: Nullness) {
        self.loc_memory.insert(loc, nullness);
    }

    /// Store nullness at an abstract memory location (weak update).
    ///
    /// Joins with any existing value at the location.
    pub fn store_loc_weak(&mut self, loc: LocId, nullness: Nullness) {
        let existing = self
            .loc_memory
            .get(&loc)
            .copied()
            .unwrap_or(Nullness::Bottom);
        self.loc_memory.insert(loc, existing.join(nullness));
    }

    /// Load nullness from an abstract memory location.
    #[must_use]
    pub fn load_loc(&self, loc: LocId) -> Option<Nullness> {
        self.loc_memory.get(&loc).copied()
    }

    /// Load nullness from a set of abstract memory locations (join all).
    ///
    /// Used for alias-aware loads when a pointer may point to multiple locations.
    /// Returns `(nullness, any_tracked)` where `any_tracked` indicates whether
    /// at least one location was found in the `loc_memory` map.
    #[must_use]
    pub fn load_locs(&self, locs: &BTreeSet<LocId>) -> (Nullness, bool) {
        let mut result = Nullness::Bottom;
        let mut any_tracked = false;
        for loc in locs {
            if let Some(nullness) = self.loc_memory.get(loc) {
                result = result.join(*nullness);
                any_tracked = true;
            } else {
                // Unknown location - conservatively assume MaybeNull
                result = result.join(Nullness::MaybeNull);
            }
        }
        (result, any_tracked)
    }

    /// Invalidate a specific abstract memory location.
    pub fn invalidate_loc(&mut self, loc: LocId) {
        self.loc_memory.remove(&loc);
    }

    /// Invalidate a set of abstract memory locations.
    pub fn invalidate_locs(&mut self, locs: &BTreeSet<LocId>) {
        for loc in locs {
            self.loc_memory.remove(loc);
        }
    }

    /// Invalidate all abstract memory locations.
    pub fn invalidate_all_loc_memory(&mut self) {
        self.loc_memory.clear();
    }

    /// Join with another state.
    pub fn join_with(&mut self, other: &Self) {
        // Join values
        for (&value, &other_nullness) in &other.values {
            let current = self.get(value);
            self.set(value, current.join(other_nullness));
        }
        // Also need to handle values in self but not in other
        for &value in self.values.keys().collect::<Vec<_>>() {
            if !other.values.contains_key(&value) {
                // If other doesn't have this value, it's effectively Bottom
                // So we keep our current value
            }
        }

        // Join memory states: keep entries present in both with joined values
        let self_mem_keys: std::collections::BTreeSet<ValueId> =
            self.memory.keys().copied().collect();
        let other_mem_keys: std::collections::BTreeSet<ValueId> =
            other.memory.keys().copied().collect();

        // For keys in both, join the values
        for key in self_mem_keys.intersection(&other_mem_keys) {
            let self_val = self.memory.get(key).copied().unwrap_or(Nullness::Bottom);
            let other_val = other.memory.get(key).copied().unwrap_or(Nullness::Bottom);
            self.memory.insert(*key, self_val.join(other_val));
        }

        // For keys only in other, add them (self had Bottom)
        for key in other_mem_keys.difference(&self_mem_keys) {
            if let Some(&val) = other.memory.get(key) {
                self.memory.insert(*key, val);
            }
        }

        // For keys only in self, keep them as-is (other had Bottom)

        // Join loc_memory states
        let self_loc_keys: BTreeSet<LocId> = self.loc_memory.keys().copied().collect();
        let other_loc_keys: BTreeSet<LocId> = other.loc_memory.keys().copied().collect();

        for key in self_loc_keys.intersection(&other_loc_keys) {
            let self_val = self
                .loc_memory
                .get(key)
                .copied()
                .unwrap_or(Nullness::Bottom);
            let other_val = other
                .loc_memory
                .get(key)
                .copied()
                .unwrap_or(Nullness::Bottom);
            self.loc_memory.insert(*key, self_val.join(other_val));
        }

        for key in other_loc_keys.difference(&self_loc_keys) {
            if let Some(&val) = other.loc_memory.get(key) {
                self.loc_memory.insert(*key, val);
            }
        }
    }

    /// Check if this state equals another.
    #[must_use]
    pub fn equals(&self, other: &Self) -> bool {
        self.values == other.values
            && self.memory == other.memory
            && self.loc_memory == other.loc_memory
    }

    /// Get all tracked values.
    #[must_use]
    pub fn values(&self) -> &BTreeMap<ValueId, Nullness> {
        &self.values
    }
}

// ---------------------------------------------------------------------------
// Nullness Analysis Result
// ---------------------------------------------------------------------------

/// Result of nullness analysis.
#[derive(Debug, Clone)]
pub struct NullnessResult {
    /// Nullness state before each instruction.
    inst_states: BTreeMap<InstId, NullnessState>,
    /// Nullness state at block entry.
    block_states: BTreeMap<BlockId, NullnessState>,
}

impl NullnessResult {
    /// Create a new result.
    pub(crate) fn new(
        inst_states: BTreeMap<InstId, NullnessState>,
        block_states: BTreeMap<BlockId, NullnessState>,
    ) -> Self {
        Self {
            inst_states,
            block_states,
        }
    }

    /// Get nullness of a value before an instruction.
    ///
    /// Returns `MaybeNull` if the instruction state is not found or the value
    /// is untracked (conservative assumption for soundness).
    #[must_use]
    pub fn nullness_at(&self, inst: InstId, value: ValueId) -> Nullness {
        self.inst_states
            .get(&inst)
            .map_or(Nullness::MaybeNull, |s| s.get(value))
    }

    /// Get nullness of a value before an instruction, returning `Bottom` for missing states.
    ///
    /// Use this when you need to distinguish between "unreachable/untracked" and "may be null".
    #[must_use]
    pub fn nullness_at_raw(&self, inst: InstId, value: ValueId) -> Nullness {
        self.inst_states
            .get(&inst)
            .map_or(Nullness::Bottom, |s| s.get_raw(value))
    }

    /// Get the full nullness state before an instruction.
    #[must_use]
    pub fn state_at_inst(&self, inst: InstId) -> Option<&NullnessState> {
        self.inst_states.get(&inst)
    }

    /// Get the nullness state at block entry.
    #[must_use]
    pub fn state_at_block(&self, block: BlockId) -> Option<&NullnessState> {
        self.block_states.get(&block)
    }
}

// ---------------------------------------------------------------------------
// Nullness Analysis
// ---------------------------------------------------------------------------

/// Configuration for nullness analysis.
#[derive(Debug, Clone)]
pub struct NullnessConfig {
    /// Maximum iterations before giving up.
    pub max_iterations: usize,
}

impl Default for NullnessConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// Nullness Context
// ---------------------------------------------------------------------------

/// Context for nullness analysis with optional precision enhancements.
pub struct NullnessContext<'a> {
    /// Optional pointer analysis for alias-aware memory tracking.
    pub pta: Option<&'a PtaIntegration<'a>>,
    /// Optional function specs for return nullness, aliasing, purity.
    pub specs: Option<&'a SpecRegistry>,
    /// Optional interprocedural summaries for defined function return nullness.
    pub summaries: Option<&'a BTreeMap<FunctionId, FunctionSummary>>,
}

impl NullnessContext<'_> {
    /// Create an empty context (baseline analysis).
    #[must_use]
    pub fn new() -> Self {
        Self {
            pta: None,
            specs: None,
            summaries: None,
        }
    }
}

impl Default for NullnessContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Public Entry Points
// ---------------------------------------------------------------------------

/// Run nullness analysis on a module with configurable context.
///
/// This is the unified entry point for all nullness analysis variants.
/// Use `NullnessContext` to control which precision enhancements are enabled:
/// - `pta`: Alias-aware memory tracking (strong/weak updates, alias-aware loads)
/// - `specs`: Function specifications for return nullness, aliasing, purity
/// - `summaries`: Interprocedural return nullness for defined functions
#[must_use]
pub fn analyze_nullness_with_context(
    module: &AirModule,
    config: &NullnessConfig,
    ctx: &NullnessContext<'_>,
) -> NullnessResult {
    let mut block_states = BTreeMap::new();
    let mut inst_states = BTreeMap::new();

    // Pass 1: analyze all functions with default MaybeNull parameters
    for func in &module.functions {
        if func.is_declaration || func.blocks.is_empty() {
            continue;
        }
        analyze_function_impl(
            func,
            module,
            config,
            ctx,
            &mut block_states,
            &mut inst_states,
            None,
        );
    }

    // Pass 2: collect caller argument nullness and re-analyze tightened functions
    let param_nullness = collect_caller_param_nullness(module, &inst_states);
    if !param_nullness.is_empty() {
        for func in &module.functions {
            if func.is_declaration || func.blocks.is_empty() {
                continue;
            }
            if let Some(pn) = param_nullness.get(&func.id) {
                // Only re-analyze if at least one param tightened to NotNull
                let any_tightened = pn.contains(&Nullness::NotNull);
                if any_tightened {
                    analyze_function_impl(
                        func,
                        module,
                        config,
                        ctx,
                        &mut block_states,
                        &mut inst_states,
                        Some(pn),
                    );
                }
            }
        }
    }

    NullnessResult::new(inst_states, block_states)
}

/// Run nullness analysis on a module.
///
/// Performs a forward dataflow analysis tracking pointer nullness.
#[must_use]
pub fn analyze_nullness(module: &AirModule, config: &NullnessConfig) -> NullnessResult {
    analyze_nullness_with_context(module, config, &NullnessContext::new())
}

/// Run nullness analysis on a module with PTA integration.
///
/// Uses alias information for:
/// - Strong updates when pointer has singleton points-to set
/// - Weak updates when pointer may point to multiple locations
/// - Alias-aware loads (join from all aliased locations)
#[must_use]
pub fn analyze_nullness_with_pta(
    module: &AirModule,
    config: &NullnessConfig,
    pta: &PtaIntegration<'_>,
) -> NullnessResult {
    analyze_nullness_with_context(
        module,
        config,
        &NullnessContext {
            pta: Some(pta),
            specs: None,
            summaries: None,
        },
    )
}

/// Run nullness analysis on a module with PTA and function specs.
///
/// Uses alias information for strong/weak updates and function specs for
/// library function modeling (return nullness, parameter aliasing, purity).
#[must_use]
pub fn analyze_nullness_with_pta_and_specs(
    module: &AirModule,
    config: &NullnessConfig,
    pta: &PtaIntegration<'_>,
    specs: &SpecRegistry,
) -> NullnessResult {
    analyze_nullness_with_context(
        module,
        config,
        &NullnessContext {
            pta: Some(pta),
            specs: Some(specs),
            summaries: None,
        },
    )
}

/// Run nullness analysis with PTA, specs, and interprocedural function summaries.
///
/// Extends `analyze_nullness_with_pta_and_specs` with interprocedural return
/// nullness tracking. When a defined function has a computed return nullness
/// summary, call sites use that instead of defaulting to `MaybeNull`.
#[must_use]
pub fn analyze_nullness_with_pta_specs_and_summaries(
    module: &AirModule,
    config: &NullnessConfig,
    pta: &PtaIntegration<'_>,
    specs: &SpecRegistry,
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
) -> NullnessResult {
    analyze_nullness_with_context(
        module,
        config,
        &NullnessContext {
            pta: Some(pta),
            specs: Some(specs),
            summaries: Some(summaries),
        },
    )
}

// ---------------------------------------------------------------------------
// Interprocedural Parameter Nullness Collection
// ---------------------------------------------------------------------------

/// Collect parameter nullness for each callee by scanning all call sites.
///
/// For each `CallDirect` instruction, joins the caller's argument nullness at
/// that call site into the callee's parameter nullness vector. A parameter is
/// `NotNull` only if ALL callers pass `NotNull` for that argument.
fn collect_caller_param_nullness(
    module: &AirModule,
    inst_states: &BTreeMap<InstId, NullnessState>,
) -> BTreeMap<FunctionId, Vec<Nullness>> {
    let mut result: BTreeMap<FunctionId, Vec<Nullness>> = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration || func.blocks.is_empty() {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                let Operation::CallDirect { callee } = &inst.op else {
                    continue;
                };

                let Some(state) = inst_states.get(&inst.id) else {
                    continue;
                };

                // Look up the callee's parameter count
                let callee_func = module.functions.iter().find(|f| f.id == *callee);
                let Some(callee_func) = callee_func else {
                    continue;
                };
                let param_count = callee_func.params.len();
                if param_count == 0 {
                    continue;
                }

                // Collect argument nullness at this call site
                let mut arg_nullness = Vec::with_capacity(param_count);
                for i in 0..param_count {
                    let nullness = inst.operands.get(i).map_or(Nullness::MaybeNull, |&arg| {
                        if is_null_constant(arg, module) {
                            Nullness::Null
                        } else {
                            state.get(arg)
                        }
                    });
                    arg_nullness.push(nullness);
                }

                // Join with existing data for this callee
                result
                    .entry(*callee)
                    .and_modify(|existing| {
                        for (i, n) in arg_nullness.iter().enumerate() {
                            if let Some(e) = existing.get_mut(i) {
                                *e = e.join(*n);
                            }
                        }
                    })
                    .or_insert(arg_nullness);
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Unified Analysis Implementation
// ---------------------------------------------------------------------------

/// Analyze a single function with the unified transfer function.
fn analyze_function_impl(
    func: &AirFunction,
    module: &AirModule,
    config: &NullnessConfig,
    ctx: &NullnessContext<'_>,
    block_states: &mut BTreeMap<BlockId, NullnessState>,
    inst_states: &mut BTreeMap<InstId, NullnessState>,
    param_nullness: Option<&[Nullness]>,
) {
    if func.blocks.is_empty() {
        return;
    }

    // Initialize entry block with parameters
    let entry_block = func.blocks[0].id;
    let mut entry_state = NullnessState::new();
    for (i, param) in func.params.iter().enumerate() {
        let nullness = param_nullness
            .and_then(|pn| pn.get(i).copied())
            .unwrap_or(Nullness::MaybeNull);
        entry_state.set(param.id, nullness);
    }
    block_states.insert(entry_block, entry_state);

    // Worklist algorithm - start only with entry block
    let mut worklist: Vec<BlockId> = vec![entry_block];
    let mut iterations = 0;

    while let Some(block_id) = worklist.pop() {
        iterations += 1;
        if iterations > config.max_iterations {
            break;
        }

        let block = func.blocks.iter().find(|b| b.id == block_id);
        let Some(block) = block else { continue };

        let mut state = block_states.get(&block_id).cloned().unwrap_or_default();

        for inst in &block.instructions {
            inst_states.insert(inst.id, state.clone());
            transfer_nullness_instruction(inst, module, &mut state, ctx);
        }

        let successors_with_edge = get_block_successors_with_edge(block, func);
        for (succ_id, edge_kind) in successors_with_edge {
            let propagate_state = refine_for_branch(&state, block, edge_kind, module);

            let old_state = block_states.get(&succ_id).cloned();
            let (new_state, changed) = match old_state {
                Some(mut old) => {
                    let prev = old.clone();
                    old.join_with(&propagate_state);
                    let changed = !prev.equals(&old);
                    (old, changed)
                }
                None => (propagate_state, true),
            };

            if changed {
                block_states.insert(succ_id, new_state);
                if !worklist.contains(&succ_id) {
                    worklist.push(succ_id);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Branch Refinement
// ---------------------------------------------------------------------------

/// Edge kind for branch refinement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeKind {
    /// Unconditional or default edge.
    Unconditional,
    /// True branch of a conditional.
    True,
    /// False branch of a conditional.
    False,
}

/// Get successor block IDs with edge kinds from a block's terminator.
fn get_block_successors_with_edge(
    block: &AirBlock,
    func: &AirFunction,
) -> Vec<(BlockId, EdgeKind)> {
    let mut successors = Vec::new();

    // Look at the last instruction (terminator)
    if let Some(last) = block.instructions.last() {
        match &last.op {
            Operation::Br { target } => {
                if let Some(target_block) = func.blocks.iter().find(|b| b.id == *target) {
                    successors.push((target_block.id, EdgeKind::Unconditional));
                }
            }
            Operation::CondBr {
                then_target,
                else_target,
            } => {
                if let Some(t) = func.blocks.iter().find(|b| b.id == *then_target) {
                    successors.push((t.id, EdgeKind::True));
                }
                if let Some(f) = func.blocks.iter().find(|b| b.id == *else_target) {
                    successors.push((f.id, EdgeKind::False));
                }
            }
            Operation::Switch { default, cases } => {
                if let Some(d) = func.blocks.iter().find(|b| b.id == *default) {
                    successors.push((d.id, EdgeKind::Unconditional));
                }
                for (_, target) in cases {
                    if let Some(c) = func.blocks.iter().find(|b| b.id == *target) {
                        successors.push((c.id, EdgeKind::Unconditional));
                    }
                }
            }
            _ => {}
        }
    }

    successors
}

/// Refine nullness state based on branch condition.
///
/// Handles simple null checks and complex conditions with AND/OR:
/// - `if (ptr != null)` - simple null check
/// - `if (a && b)` - AND of null checks, true branch means both NotNull
/// - `if (a || b)` - OR of null checks, false branch means both Null
fn refine_for_branch(
    state: &NullnessState,
    block: &AirBlock,
    edge: EdgeKind,
    module: &AirModule,
) -> NullnessState {
    let mut refined = state.clone();

    // Only refine for true/false edges
    if edge == EdgeKind::Unconditional {
        return refined;
    }

    // Find the conditional branch terminator
    let Some(term) = block.instructions.last() else {
        return refined;
    };

    let Operation::CondBr { .. } = &term.op else {
        return refined;
    };

    // Get the condition operand
    let Some(&cond_value) = term.operands.first() else {
        return refined;
    };

    // Build instruction map for this block
    let inst_map: BTreeMap<ValueId, &Instruction> = block
        .instructions
        .iter()
        .filter_map(|i| i.dst.map(|d| (d, i)))
        .collect();

    // Collect all null check refinements from the condition
    let refinements = collect_null_check_refinements(cond_value, edge, &inst_map, module);

    // Apply value refinements
    for (ptr, nullness) in refinements.values {
        refined.set(ptr, nullness);
    }

    // Apply memory refinements (for values loaded from memory)
    for (mem_loc, nullness) in refinements.memory {
        refined.store(mem_loc, nullness);
    }

    refined
}

/// Represents a null check condition.
#[derive(Debug, Clone, Copy)]
enum NullCheckKind {
    /// `ptr != null` - pointer is checked for non-null
    /// The optional second ValueId is the memory location if ptr was loaded from memory
    IsNotNull(ValueId, Option<ValueId>),
    /// `ptr == null` - pointer is checked for null
    /// The optional second ValueId is the memory location if ptr was loaded from memory
    IsNull(ValueId, Option<ValueId>),
}

/// Refinements to apply after branch analysis.
struct BranchRefinements {
    /// Value refinements: (ValueId, Nullness)
    values: Vec<(ValueId, Nullness)>,
    /// Memory refinements: (memory location ValueId, Nullness)
    memory: Vec<(ValueId, Nullness)>,
}

/// Collect null check refinements from a condition value, handling AND/OR.
///
/// For AND on true branch: all sub-conditions are true → all ptrs NotNull
/// For OR on false branch: all sub-conditions are false → all ptrs Null
fn collect_null_check_refinements(
    cond: ValueId,
    edge: EdgeKind,
    inst_map: &BTreeMap<ValueId, &Instruction>,
    module: &AirModule,
) -> BranchRefinements {
    let mut refinements = BranchRefinements {
        values: Vec::new(),
        memory: Vec::new(),
    };

    // Recursively collect null checks, tracking logical structure
    let checks = extract_null_checks(cond, inst_map, module, 0);

    for (check, logic_path) in checks {
        // Determine the effective edge based on logic path
        // logic_path: 0 = direct, positive = through ANDs, negative = through ORs (inverted)
        let effective_edge = if logic_path >= 0 { edge } else { edge.invert() };

        match (check, effective_edge) {
            // `ptr != null` on true branch or `ptr == null` on false branch → NotNull
            (NullCheckKind::IsNotNull(ptr, mem_loc), EdgeKind::True)
            | (NullCheckKind::IsNull(ptr, mem_loc), EdgeKind::False) => {
                refinements.values.push((ptr, Nullness::NotNull));
                if let Some(loc) = mem_loc {
                    refinements.memory.push((loc, Nullness::NotNull));
                }
            }
            // `ptr != null` on false branch or `ptr == null` on true branch → Null
            (NullCheckKind::IsNotNull(ptr, mem_loc), EdgeKind::False)
            | (NullCheckKind::IsNull(ptr, mem_loc), EdgeKind::True) => {
                refinements.values.push((ptr, Nullness::Null));
                if let Some(loc) = mem_loc {
                    refinements.memory.push((loc, Nullness::Null));
                }
            }
            _ => {}
        }
    }

    refinements
}

/// Extract null checks from a condition, recursing through AND/OR.
///
/// Returns list of (NullCheckKind, logic_path) where logic_path tracks inversions.
fn extract_null_checks(
    cond: ValueId,
    inst_map: &BTreeMap<ValueId, &Instruction>,
    module: &AirModule,
    depth: usize,
) -> Vec<(NullCheckKind, i32)> {
    // Limit recursion depth to avoid infinite loops
    if depth > 10 {
        return Vec::new();
    }

    let Some(inst) = inst_map.get(&cond) else {
        return Vec::new();
    };

    if let Operation::BinaryOp { kind } = &inst.op {
        use saf_core::air::BinaryOp;

        match kind {
            // AND: both operands must be true on true branch
            BinaryOp::And => {
                if inst.operands.len() >= 2 {
                    let mut checks =
                        extract_null_checks(inst.operands[0], inst_map, module, depth + 1);
                    checks.extend(extract_null_checks(
                        inst.operands[1],
                        inst_map,
                        module,
                        depth + 1,
                    ));
                    return checks;
                }
            }

            // OR: at least one operand is true on true branch
            // On FALSE branch of OR, ALL operands are false
            BinaryOp::Or => {
                if inst.operands.len() >= 2 {
                    // For OR on false branch, we can refine all pointers
                    // But for true branch, we can't refine any (only one needs to be true)
                    // We handle this by collecting checks but they only apply on false branch
                    let mut checks = Vec::new();
                    for check in extract_null_checks(inst.operands[0], inst_map, module, depth + 1)
                    {
                        // Mark as coming through OR (negative logic_path)
                        checks.push((check.0, -1));
                    }
                    for check in extract_null_checks(inst.operands[1], inst_map, module, depth + 1)
                    {
                        checks.push((check.0, -1));
                    }
                    return checks;
                }
            }

            // XOR with 1 (or true): logical NOT
            BinaryOp::Xor => {
                if inst.operands.len() >= 2 {
                    // Check if XOR with 1 (logical NOT)
                    let is_not = is_true_constant(inst.operands[1], module)
                        || is_true_constant(inst.operands[0], module);
                    if is_not {
                        let operand = if is_true_constant(inst.operands[1], module) {
                            inst.operands[0]
                        } else {
                            inst.operands[1]
                        };
                        // Invert the logic path for NOT
                        return extract_null_checks(operand, inst_map, module, depth + 1)
                            .into_iter()
                            .map(|(check, path)| (check, -path - 1))
                            .collect();
                    }
                }
            }

            // Null pointer comparison
            BinaryOp::ICmpEq | BinaryOp::ICmpNe => {
                if inst.operands.len() >= 2 {
                    let lhs = inst.operands[0];
                    let rhs = inst.operands[1];

                    // Check if one operand is null
                    let ptr_value = if is_null_constant(rhs, module) {
                        Some(lhs)
                    } else if is_null_constant(lhs, module) {
                        Some(rhs)
                    } else {
                        None
                    };

                    if let Some(ptr) = ptr_value {
                        // Check if the pointer was loaded from memory
                        // If so, we can also refine the memory location
                        let mem_loc = find_load_source(ptr, inst_map);

                        let check = if *kind == BinaryOp::ICmpNe {
                            NullCheckKind::IsNotNull(ptr, mem_loc)
                        } else {
                            NullCheckKind::IsNull(ptr, mem_loc)
                        };
                        return vec![(check, 0)];
                    }
                }
            }

            _ => {}
        }
    }

    Vec::new()
}

/// Find the memory location a value was loaded from, if any.
///
/// Given a ValueId, trace back through the instruction map to find if
/// it was produced by a Load instruction. If so, return the pointer operand.
fn find_load_source(value: ValueId, inst_map: &BTreeMap<ValueId, &Instruction>) -> Option<ValueId> {
    let inst = inst_map.get(&value)?;

    if let Operation::Load = &inst.op {
        // Load has pointer as first operand
        inst.operands.first().copied()
    } else {
        None
    }
}

/// Check if a value is true (1) constant.
fn is_true_constant(value: ValueId, module: &AirModule) -> bool {
    matches!(
        module.constants.get(&value),
        Some(Constant::Int { value: 1, .. })
    )
}

impl EdgeKind {
    /// Invert the edge kind.
    fn invert(self) -> Self {
        match self {
            EdgeKind::True => EdgeKind::False,
            EdgeKind::False => EdgeKind::True,
            EdgeKind::Unconditional => EdgeKind::Unconditional,
        }
    }
}

/// Check if a value is a null constant.
fn is_null_constant(value: ValueId, module: &AirModule) -> bool {
    matches!(
        module.constants.get(&value),
        Some(Constant::Null | Constant::Int { value: 0, .. })
    )
}

// Function property lookups (purity, return aliasing, nullness, noreturn) are
// centralized in the `function_properties` module. Re-export for backwards
// compatibility with external callers that import from `nullness`.
pub use super::function_properties::{
    is_noreturn_from_spec, is_noreturn_with_specs, is_pure_with_spec,
    param_required_nonnull_from_spec, return_aliases_param_from_spec, return_nullness_from_spec,
    returns_alias_param_with_specs, returns_nonnull_with_specs,
};

// ---------------------------------------------------------------------------
// Unified Transfer Function
// ---------------------------------------------------------------------------

/// Apply transfer function for an instruction with configurable context.
///
/// Behavior depends on which context fields are populated:
/// - No PTA: Uses `ValueId`-based memory tracking only
/// - With PTA: Uses alias-aware strong/weak updates and loads
/// - With specs: Uses function specs for return nullness, aliasing, purity
/// - With summaries: Uses interprocedural return nullness for defined functions
fn transfer_nullness_instruction(
    inst: &Instruction,
    module: &AirModule,
    state: &mut NullnessState,
    ctx: &NullnessContext<'_>,
) {
    // Handle Store first (no dst, but modifies memory state)
    if let Operation::Store = &inst.op {
        transfer_store(inst, module, state, ctx);
        return;
    }

    // Handle Memcpy/Memset - modifies memory, returns dest pointer
    if matches!(&inst.op, Operation::Memcpy | Operation::Memset) {
        state.invalidate_all_memory();
        if ctx.pta.is_some() {
            state.invalidate_all_loc_memory();
        }
        if let Some(dst) = inst.dst {
            if let Some(&dest_ptr) = inst.operands.first() {
                state.set(dst, state.get(dest_ptr));
            } else {
                state.set(dst, Nullness::MaybeNull);
            }
        }
        return;
    }

    // Handle void function calls (no return value but may have side effects)
    if inst.dst.is_none() {
        match &inst.op {
            Operation::CallDirect { callee } => {
                transfer_void_call_direct(inst, *callee, module, state, ctx);
            }
            Operation::CallIndirect { .. } => {
                // Conservative: invalidate all memory for unknown void calls
                state.invalidate_all_memory();
                if ctx.pta.is_some() {
                    state.invalidate_all_loc_memory();
                }
            }
            _ => {} // Other void ops (Br, CondBr, Ret, etc.) - no effect on nullness
        }
        return;
    }

    let Some(dst) = inst.dst else { return };

    match &inst.op {
        // Heap/stack allocation and global address are non-null
        Operation::HeapAlloc { .. } | Operation::Alloca { .. } | Operation::Global { .. } => {
            state.set(dst, Nullness::NotNull);
        }

        // Load - context-dependent alias-aware loading
        Operation::Load => {
            transfer_load(inst, dst, module, state, ctx);
        }

        // Copy/Cast - propagate nullness from source
        Operation::Copy | Operation::Cast { .. } => {
            if let Some(&src) = inst.operands.first() {
                if is_null_constant(src, module) {
                    state.set(dst, Nullness::Null);
                } else {
                    state.set(dst, state.get(src));
                }
            }
        }

        // GEP on a non-null base is non-null
        Operation::Gep { .. } => {
            if let Some(&base) = inst.operands.first() {
                state.set(dst, state.get(base));
            }
        }

        // Phi nodes - join all incoming values
        Operation::Phi { incoming } => {
            let mut result = Nullness::Bottom;
            for (_, value) in incoming {
                if is_null_constant(*value, module) {
                    result = result.join(Nullness::Null);
                } else {
                    result = result.join(state.get(*value));
                }
            }
            state.set(dst, result);
        }

        // Select (ternary) - join both branches
        Operation::Select => {
            if inst.operands.len() >= 3 {
                let true_val = if is_null_constant(inst.operands[1], module) {
                    Nullness::Null
                } else {
                    state.get(inst.operands[1])
                };
                let false_val = if is_null_constant(inst.operands[2], module) {
                    Nullness::Null
                } else {
                    state.get(inst.operands[2])
                };
                state.set(dst, true_val.join(false_val));
            }
        }

        // Function calls - context-dependent handling
        Operation::CallDirect { callee } => {
            transfer_call_direct(inst, dst, *callee, module, state, ctx);
        }

        Operation::CallIndirect { .. } => {
            transfer_call_indirect(inst, dst, state, ctx);
        }

        // All other operations: MaybeNull
        _ => {
            state.set(dst, Nullness::MaybeNull);
        }
    }
}

/// Transfer function for Store operation.
fn transfer_store(
    inst: &Instruction,
    module: &AirModule,
    state: &mut NullnessState,
    ctx: &NullnessContext<'_>,
) {
    if inst.operands.len() < 2 {
        return;
    }

    let value_id = inst.operands[0];
    let ptr_id = inst.operands[1];

    let value_nullness = if is_null_constant(value_id, module) {
        Nullness::Null
    } else {
        state.get(value_id)
    };

    // Without PTA: use ValueId-based tracking
    let Some(pta_ref) = ctx.pta else {
        state.store(ptr_id, value_nullness);
        return;
    };

    // With PTA: use alias-aware strong/weak updates
    let pt_set = pta_ref.points_to(ptr_id);
    if pt_set.is_empty() {
        // No PTA info - fall back to ValueId-based tracking
        state.store(ptr_id, value_nullness);
    } else if pta_ref.is_singleton(ptr_id) {
        // Singleton points-to set: strong update
        for loc in &pt_set {
            state.store_loc(*loc, value_nullness);
        }
    } else {
        // Multiple targets: weak update (join)
        for loc in &pt_set {
            state.store_loc_weak(*loc, value_nullness);
        }
    }
}

/// Transfer function for void `CallDirect` (no return value).
///
/// Handles side effects of void function calls:
/// - Deallocators (`free`, `operator delete`): mark first argument as `MaybeNull`
/// - Non-pure functions: invalidate all memory
/// - Pure functions: no effect
fn transfer_void_call_direct(
    inst: &Instruction,
    callee: FunctionId,
    module: &AirModule,
    state: &mut NullnessState,
    ctx: &NullnessContext<'_>,
) {
    let callee_name = module
        .functions
        .iter()
        .find(|f| f.id == callee)
        .map(|f| f.name.as_str());

    // With summaries: use interprocedural mod/ref info for defined functions
    if let Some(summaries) = ctx.summaries {
        let callee_is_defined = module
            .functions
            .iter()
            .any(|f| f.id == callee && !f.is_declaration);

        if callee_is_defined {
            if let Some(summary) = summaries.get(&callee) {
                if summary.may_modify_unknown() {
                    state.invalidate_all_memory();
                    if ctx.pta.is_some() {
                        state.invalidate_all_loc_memory();
                    }
                } else if !summary.modified_locations().is_empty() {
                    state.invalidate_locs(summary.modified_locations());
                }
                return;
            }
        }
    }

    let Some(name) = callee_name else {
        // Unknown function — conservative invalidation
        state.invalidate_all_memory();
        if ctx.pta.is_some() {
            state.invalidate_all_loc_memory();
        }
        return;
    };

    // Deallocator: mark first argument as MaybeNull (dangling after free)
    if function_properties::is_deallocation_with_specs(name, ctx.specs) {
        if let Some(&arg) = inst.operands.first() {
            state.set(arg, Nullness::MaybeNull);
        }
    }

    // Check purity for memory invalidation
    let is_pure = function_properties::is_pure_function_with_specs(name, ctx.specs);
    if !is_pure {
        state.invalidate_all_memory();
        if ctx.pta.is_some() {
            state.invalidate_all_loc_memory();
        }
    }
}

/// Transfer function for Load operation.
fn transfer_load(
    inst: &Instruction,
    dst: ValueId,
    _module: &AirModule,
    state: &mut NullnessState,
    ctx: &NullnessContext<'_>,
) {
    let Some(&ptr) = inst.operands.first() else {
        state.set(dst, Nullness::MaybeNull);
        return;
    };

    // Without PTA: use ValueId-based tracking only
    let Some(pta_ref) = ctx.pta else {
        if let Some(stored_nullness) = state.load(ptr) {
            state.set(dst, stored_nullness);
        } else {
            state.set(dst, Nullness::MaybeNull);
        }
        return;
    };

    // With PTA: check branch refinement first, then use alias-aware loading
    // Branch refinements store definite info in ValueId-based memory
    if let Some(refined_nullness) = state.load(ptr) {
        if refined_nullness == Nullness::NotNull || refined_nullness == Nullness::Null {
            state.set(dst, refined_nullness);
            return;
        }
    }

    // No definite refinement - check PTA-based info
    let pt_set = pta_ref.points_to(ptr);
    if pt_set.is_empty() {
        // No PTA info - use ValueId-based tracking
        if let Some(stored_nullness) = state.load(ptr) {
            state.set(dst, stored_nullness);
        } else {
            state.set(dst, Nullness::MaybeNull);
        }
    } else {
        // Use PTA: join nullness from all aliased locations
        let (loaded_nullness, any_tracked) = state.load_locs(&pt_set);
        if any_tracked {
            state.set(dst, loaded_nullness);
        } else if let Some(stored_nullness) = state.load(ptr) {
            state.set(dst, stored_nullness);
        } else {
            state.set(dst, Nullness::MaybeNull);
        }
    }
}

/// Transfer function for CallDirect operation.
fn transfer_call_direct(
    inst: &Instruction,
    dst: ValueId,
    callee: FunctionId,
    module: &AirModule,
    state: &mut NullnessState,
    ctx: &NullnessContext<'_>,
) {
    let callee_name = module
        .functions
        .iter()
        .find(|f| f.id == callee)
        .map(|f| f.name.as_str());

    // Check if this is a deallocation call (free, operator delete, etc.)
    let is_dealloc = callee_name
        .is_some_and(|name| function_properties::is_deallocation_with_specs(name, ctx.specs));

    // With summaries: try interprocedural summary first for defined functions
    if let Some(summaries) = ctx.summaries {
        let callee_is_defined = module
            .functions
            .iter()
            .any(|f| f.id == callee && !f.is_declaration);

        if callee_is_defined {
            if let Some(summary) = summaries.get(&callee) {
                if let Some(return_nullness) = summary.return_nullness() {
                    state.set(dst, return_nullness);
                } else {
                    state.set(dst, Nullness::MaybeNull);
                }

                // Use mod/ref-based selective invalidation
                if summary.may_modify_unknown() {
                    state.invalidate_all_memory();
                    if ctx.pta.is_some() {
                        state.invalidate_all_loc_memory();
                    }
                } else if !summary.modified_locations().is_empty() {
                    state.invalidate_locs(summary.modified_locations());
                }
                // Pure function - no invalidation needed
                if is_dealloc {
                    apply_deallocation_effect(inst, state, ctx);
                }
                return;
            }
        }
    }

    // With specs: use function specifications
    if let Some(specs) = ctx.specs {
        if let Some(name) = callee_name {
            // Check spec for return-aliases-param
            if let Some(param_idx) = return_aliases_param_from_spec(name, specs) {
                if let Some(&arg) = inst.operands.get(param_idx as usize) {
                    state.set(dst, state.get(arg));
                } else {
                    state.set(dst, Nullness::MaybeNull);
                }
            }
            // Check spec for explicit return nullness
            else if let Some(nullness) = return_nullness_from_spec(name, specs) {
                state.set(dst, nullness);
            }
            // Fall back to hardcoded lists
            else {
                set_return_nullness_from_hardcoded(name, inst, state, dst);
            }

            // Check if function is pure (using spec, then hardcoded)
            let is_side_effect_free = is_pure_with_spec(name, specs);
            if !is_side_effect_free {
                state.invalidate_all_memory();
                if ctx.pta.is_some() {
                    state.invalidate_all_loc_memory();
                }
            }
            if is_dealloc {
                apply_deallocation_effect(inst, state, ctx);
            }
            return;
        }
    }

    // Without specs: use hardcoded handling only
    if let Some(name) = callee_name {
        set_return_nullness_from_hardcoded(name, inst, state, dst);

        let is_side_effect_free = function_properties::is_pure_function_with_specs(name, None);
        if !is_side_effect_free {
            state.invalidate_all_memory();
            if ctx.pta.is_some() {
                state.invalidate_all_loc_memory();
            }
        }
        if is_dealloc {
            apply_deallocation_effect(inst, state, ctx);
        }
    } else {
        state.set(dst, Nullness::MaybeNull);
        state.invalidate_all_memory();
        if ctx.pta.is_some() {
            state.invalidate_all_loc_memory();
        }
    }
}

/// Apply deallocation side effects on the first argument.
///
/// After `free(ptr)`:
/// 1. Mark the ptr value as `MaybeNull` (it's now dangling)
/// 2. With PTA: find the alloca that ptr was loaded from and store `MaybeNull`
///    to that alloca's PTA location, so subsequent loads get `MaybeNull`
fn apply_deallocation_effect(
    inst: &Instruction,
    state: &mut NullnessState,
    ctx: &NullnessContext<'_>,
) {
    let Some(&arg) = inst.operands.first() else {
        return;
    };

    // Mark the ptr value itself as MaybeNull
    state.set(arg, Nullness::MaybeNull);

    // Also store MaybeNull via ValueId-based memory (for non-PTA path)
    state.store(arg, Nullness::MaybeNull);

    // With PTA: propagate MaybeNull to the arg's alloca location
    // The arg was loaded from an alloca; we need to mark that alloca's
    // loc_memory entry so subsequent loads from the same alloca get MaybeNull
    if let Some(pta) = ctx.pta {
        let pt_set = pta.points_to(arg);
        for loc in &pt_set {
            state.store_loc(*loc, Nullness::MaybeNull);
        }
    }
}

/// Set return nullness based on hardcoded function lists.
fn set_return_nullness_from_hardcoded(
    name: &str,
    inst: &Instruction,
    state: &mut NullnessState,
    dst: ValueId,
) {
    if function_properties::returns_first_argument(name) && !inst.operands.is_empty() {
        let first_arg = inst.operands[0];
        state.set(dst, state.get(first_arg));
    } else if function_properties::returns_nonnull(name) {
        state.set(dst, Nullness::NotNull);
    } else {
        state.set(dst, Nullness::MaybeNull);
    }
}

/// Transfer function for CallIndirect operation.
fn transfer_call_indirect(
    inst: &Instruction,
    dst: ValueId,
    state: &mut NullnessState,
    ctx: &NullnessContext<'_>,
) {
    // Without PTA+summaries: conservative handling
    let (Some(pta), Some(summaries)) = (ctx.pta, ctx.summaries) else {
        state.set(dst, Nullness::MaybeNull);
        state.invalidate_all_memory();
        if ctx.pta.is_some() {
            state.invalidate_all_loc_memory();
        }
        return;
    };

    // With PTA+summaries: try to resolve targets
    let Some(&fn_ptr) = inst.operands.first() else {
        state.set(dst, Nullness::MaybeNull);
        state.invalidate_all_memory();
        state.invalidate_all_loc_memory();
        return;
    };

    let targets = pta.resolve_indirect_call(fn_ptr);

    if targets.is_empty() {
        // Unknown targets - full invalidation
        state.set(dst, Nullness::MaybeNull);
        state.invalidate_all_memory();
        state.invalidate_all_loc_memory();
        return;
    }

    // Join return nullness and mod/ref from all possible callees
    let mut return_nullness = Nullness::Bottom;
    let mut any_modifies_unknown = false;
    let mut all_pure = true;
    let mut all_locs_to_invalidate = BTreeSet::new();

    for target in &targets {
        if let Some(summary) = summaries.get(target) {
            // Join return nullness
            if let Some(ret_null) = summary.return_nullness() {
                return_nullness = return_nullness.join(ret_null);
            } else {
                return_nullness = return_nullness.join(Nullness::MaybeNull);
            }

            // Collect mod/ref info
            if summary.may_modify_unknown() {
                any_modifies_unknown = true;
            }
            if !summary.modified_locations().is_empty() || summary.may_modify_unknown() {
                all_pure = false;
            }
            all_locs_to_invalidate.extend(summary.modified_locations().iter().copied());
        } else {
            // No summary for target - conservative
            return_nullness = return_nullness.join(Nullness::MaybeNull);
            any_modifies_unknown = true;
            all_pure = false;
        }
    }

    // Set return nullness
    if return_nullness == Nullness::Bottom {
        state.set(dst, Nullness::MaybeNull);
    } else {
        state.set(dst, return_nullness);
    }

    // Apply mod/ref-based invalidation
    if any_modifies_unknown {
        state.invalidate_all_memory();
        state.invalidate_all_loc_memory();
    } else if !all_pure {
        state.invalidate_locs(&all_locs_to_invalidate);
    }
    // If all_pure, no invalidation needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nullness_join() {
        use Nullness::*;

        assert_eq!(Bottom.join(Null), Null);
        assert_eq!(Bottom.join(NotNull), NotNull);
        assert_eq!(Null.join(NotNull), MaybeNull);
        assert_eq!(MaybeNull.join(Null), MaybeNull);
        assert_eq!(NotNull.join(NotNull), NotNull);
    }

    #[test]
    fn test_nullness_predicates() {
        assert!(Nullness::Null.is_definitely_null());
        assert!(!Nullness::NotNull.is_definitely_null());
        assert!(!Nullness::MaybeNull.is_definitely_null());

        assert!(Nullness::NotNull.is_definitely_not_null());
        assert!(!Nullness::Null.is_definitely_not_null());
        assert!(!Nullness::MaybeNull.is_definitely_not_null());

        assert!(Nullness::Null.may_be_null());
        assert!(Nullness::MaybeNull.may_be_null());
        assert!(!Nullness::NotNull.may_be_null());
    }

    #[test]
    fn test_returns_alias_param_hardcoded_fallback() {
        // Without specs, should fall back to hardcoded list
        assert_eq!(returns_alias_param_with_specs("memcpy", None), Some(0));
        assert_eq!(returns_alias_param_with_specs("strcpy", None), Some(0));
        assert_eq!(returns_alias_param_with_specs("strcat", None), Some(0));
        assert_eq!(returns_alias_param_with_specs("unknown_func", None), None);
    }

    #[test]
    fn test_returns_nonnull_hardcoded_fallback() {
        // Without specs, should fall back to hardcoded list
        assert!(returns_nonnull_with_specs("xmalloc", None));
        assert!(returns_nonnull_with_specs("xrealloc", None));
        assert!(!returns_nonnull_with_specs("malloc", None)); // malloc can return null
        assert!(!returns_nonnull_with_specs("unknown_func", None));
    }

    #[test]
    fn test_is_noreturn_hardcoded_fallback() {
        // Without specs, should fall back to hardcoded list
        assert!(is_noreturn_with_specs("exit", None));
        assert!(is_noreturn_with_specs("abort", None));
        assert!(is_noreturn_with_specs("pthread_exit", None));
        assert!(!is_noreturn_with_specs("printf", None));
        assert!(!is_noreturn_with_specs("unknown_func", None));
    }
}
