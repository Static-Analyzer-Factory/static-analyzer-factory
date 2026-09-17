//! Sparse Conditional Constant Propagation (SCCP) pre-pass.
//!
//! Runs before the main fixpoint solver. Produces:
//! - `constants`: values proven to be a single constant
//! - `dead_blocks`: blocks proven unreachable via constant branch resolution
//!
//! Algorithm: Wegman & Zadeck (POPL 1991) adapted for AIR.
//! Two worklists: SSA edges (value propagation) and CFG edges (reachability).
//! Three-level lattice: Top (unknown) -> Constant(i128) -> Bottom (overdetermined).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::air::{AirFunction, AirModule, AirType, BinaryOp, CastKind, Constant, Operation};
use saf_core::ids::{BlockId, TypeId, ValueId};

// =============================================================================
// Lattice
// =============================================================================

/// Three-level SCCP lattice value.
///
/// Ordering: `Top` (unknown) > `Constant(c)` > `Bottom` (overdetermined).
/// The meet operation lowers values in this lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SccpValue {
    /// Not yet analyzed — may become any constant.
    Top,
    /// Proven to be exactly this integer constant.
    Constant(i128),
    /// Proven to have multiple possible values (overdetermined).
    Bottom,
}

impl SccpValue {
    /// Lattice meet: greatest lower bound.
    ///
    /// - `Top` meet X = X (Top is identity)
    /// - `Constant(c)` meet `Constant(c)` = `Constant(c)` (same constant is stable)
    /// - `Constant(a)` meet `Constant(b)` = `Bottom` (different constants overdetermine)
    /// - `Bottom` meet X = `Bottom` (Bottom absorbs)
    #[must_use]
    pub fn meet(self, other: Self) -> Self {
        match (self, other) {
            (Self::Top, x) | (x, Self::Top) => x,
            (Self::Bottom, _) | (_, Self::Bottom) => Self::Bottom,
            (Self::Constant(a), Self::Constant(b)) => {
                if a == b {
                    Self::Constant(a)
                } else {
                    Self::Bottom
                }
            }
        }
    }

    /// Extract the constant value, if this is a `Constant`.
    #[must_use]
    pub fn as_constant(self) -> Option<i128> {
        match self {
            Self::Constant(c) => Some(c),
            _ => None,
        }
    }
}

// =============================================================================
// Result
// =============================================================================

/// Results from an SCCP analysis pass.
#[derive(Debug, Clone, Default)]
pub struct SccpResult {
    /// Values proven to be a single integer constant.
    pub constants: BTreeMap<ValueId, i128>,
    /// Blocks proven unreachable via constant branch resolution.
    pub dead_blocks: BTreeSet<BlockId>,
}

// =============================================================================
// Module-level entry point
// =============================================================================

/// Run SCCP on all non-declaration functions in a module.
///
/// Merges per-function results into a single `SccpResult`.
/// Detects read-only globals (never stored to) and propagates their
/// initial values through `Load` instructions.
#[must_use]
pub fn run_sccp_module(module: &AirModule) -> SccpResult {
    let read_only_globals = build_read_only_globals(module);

    // Signed result widths, so an arithmetic fold can be normalised back to the
    // value's OWN width. This is the same lattice invariant `evaluate_cast`
    // protects, and `evaluate_binary` violated it: it computes in `i128`, which
    // does not wrap where the value does. (plans/213 M2)
    let int_widths: BTreeMap<TypeId, u8> = module
        .types
        .iter()
        .filter_map(|(tid, ty)| match ty {
            AirType::Integer { bits } => u8::try_from(*bits).ok().map(|b| (*tid, b)),
            _ => None,
        })
        .collect();

    let mut result = SccpResult::default();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let func_result =
            run_sccp_function_with_widths(func, &module.constants, &read_only_globals, &int_widths);
        result.constants.extend(func_result.constants);
        result.dead_blocks.extend(func_result.dead_blocks);
    }
    result
}

/// Build a map of read-only global addresses to their initial constant values.
///
/// A global is "read-only" if its address never appears as the pointer operand
/// of a `Store` instruction anywhere in the module. For such globals, `Load`
/// from their address always returns the initialization value.
///
/// This handles patterns like `static int staticTrue = 1` and
/// `const int GLOBAL_CONST_FIVE = 5` in the Juliet test suite.
fn build_read_only_globals(module: &AirModule) -> BTreeMap<ValueId, SccpValue> {
    // Collect all global addresses that are stored to
    let mut stored_to: BTreeSet<ValueId> = BTreeSet::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if matches!(inst.op, Operation::Store) {
                    // Store operands: [value, pointer]
                    if let Some(&ptr) = inst.operands.get(1) {
                        stored_to.insert(ptr);
                    }
                }
            }
        }
    }

    // Build map for globals that are never stored to and have an init value
    let mut read_only = BTreeMap::new();
    for global in &module.globals {
        if stored_to.contains(&global.id) {
            continue;
        }
        if let Some(ref init) = global.init {
            let val = constant_to_sccp(init);
            if !matches!(val, SccpValue::Bottom) {
                read_only.insert(global.id, val);
            }
        }
    }
    read_only
}

// =============================================================================
// Function-level solver (dual-worklist algorithm)
// =============================================================================

/// Run SCCP on a single function.
///
/// Implements the dual-worklist Wegman-Zadeck algorithm:
/// 1. CFG worklist tracks newly executable blocks.
/// 2. SSA worklist tracks values whose lattice state changed.
///
/// The `module_constants` map provides constant values for `ValueId`s that
/// correspond to module-level constants (e.g., literal integers in operands).
/// The `read_only_globals` map provides constant values for globals that are
/// never stored to — `Load` from these addresses returns the init value.
#[must_use]
/// Back-compat entry point: no type-width table, so any arithmetic fold whose
/// result width is unknown drops to `Bottom` rather than claiming a value the
/// program may not hold. Prefer [`run_sccp_function_with_widths`].
pub fn run_sccp_function(
    func: &AirFunction,
    module_constants: &BTreeMap<ValueId, Constant>,
    read_only_globals: &BTreeMap<ValueId, SccpValue>,
) -> SccpResult {
    run_sccp_function_with_widths(func, module_constants, read_only_globals, &BTreeMap::new())
}

/// SCCP over one function, with the module's integer result widths so binary
/// folds can be normalised to the value's own width (plans/213 M2).
pub fn run_sccp_function_with_widths(
    func: &AirFunction,
    module_constants: &BTreeMap<ValueId, Constant>,
    read_only_globals: &BTreeMap<ValueId, SccpValue>,
    int_widths: &BTreeMap<TypeId, u8>,
) -> SccpResult {
    // Value lattice: all values start at Top (unknown).
    let mut values: BTreeMap<ValueId, SccpValue> = BTreeMap::new();

    // Executable edges (from_block, to_block) — tracks which CFG edges have been taken.
    let mut executable_edges: BTreeSet<(BlockId, BlockId)> = BTreeSet::new();

    // Executable blocks — blocks reached by at least one executable edge.
    let mut executable_blocks: BTreeSet<BlockId> = BTreeSet::new();

    // Worklists.
    let mut cfg_worklist: VecDeque<BlockId> = VecDeque::new();
    let mut ssa_worklist: VecDeque<ValueId> = VecDeque::new();

    // Seed module constants into the value lattice.
    for (vid, constant) in module_constants {
        let sccp_val = constant_to_sccp(constant);
        values.insert(*vid, sccp_val);
    }

    // Mark entry block executable.
    let entry_id = match func.entry_block {
        Some(id) => id,
        None => {
            if let Some(first) = func.blocks.first() {
                first.id
            } else {
                return SccpResult::default();
            }
        }
    };
    executable_blocks.insert(entry_id);
    cfg_worklist.push_back(entry_id);

    // Main solver loop.
    let mut iterations = 0;
    let max_iterations = 10_000;

    while (!cfg_worklist.is_empty() || !ssa_worklist.is_empty()) && iterations < max_iterations {
        iterations += 1;

        // Process CFG worklist: evaluate all instructions in newly executable blocks.
        while let Some(block_id) = cfg_worklist.pop_front() {
            if let Some(block) = func.block(block_id) {
                for inst in &block.instructions {
                    evaluate_instruction(
                        inst,
                        block_id,
                        &mut values,
                        module_constants,
                        read_only_globals,
                        &mut ssa_worklist,
                        &mut executable_edges,
                        &mut executable_blocks,
                        &mut cfg_worklist,
                        int_widths,
                    );
                }
            }
        }

        // Process SSA worklist: re-evaluate instructions that use changed values.
        while let Some(changed_vid) = ssa_worklist.pop_front() {
            // Find all instructions that use this value and are in executable blocks.
            for block in &func.blocks {
                if !executable_blocks.contains(&block.id) {
                    continue;
                }
                for inst in &block.instructions {
                    let uses_value = inst.operands.contains(&changed_vid)
                        || matches!(&inst.op, Operation::Phi { incoming } if incoming.iter().any(|(_, v)| *v == changed_vid));
                    if uses_value {
                        evaluate_instruction(
                            inst,
                            block.id,
                            &mut values,
                            module_constants,
                            read_only_globals,
                            &mut ssa_worklist,
                            &mut executable_edges,
                            &mut executable_blocks,
                            &mut cfg_worklist,
                            int_widths,
                        );
                    }
                }
            }
        }
    }

    // Collect results.
    let mut result = SccpResult::default();

    // Gather proven constants.
    for (vid, val) in &values {
        if let Some(c) = val.as_constant() {
            result.constants.insert(*vid, c);
        }
    }

    // Gather dead blocks (exist in function but never became executable).
    for block in &func.blocks {
        if !executable_blocks.contains(&block.id) {
            result.dead_blocks.insert(block.id);
        }
    }

    result
}

// =============================================================================
// Helpers
// =============================================================================

/// Convert an AIR `Constant` to an `SccpValue`.
fn constant_to_sccp(constant: &Constant) -> SccpValue {
    match constant {
        Constant::Int { value, .. } => SccpValue::Constant(i128::from(*value)),
        Constant::BigInt { value, .. } => {
            if let Ok(v) = value.parse::<i128>() {
                SccpValue::Constant(v)
            } else {
                SccpValue::Bottom
            }
        }
        Constant::Null | Constant::ZeroInit => SccpValue::Constant(0),
        // Float, String, Undef, Aggregate, GlobalRef — not integer-foldable.
        _ => SccpValue::Bottom,
    }
}

/// Look up the SCCP value for a `ValueId`.
///
/// Checks the local lattice first, then falls back to module constants.
/// Returns `Top` if the value has never been seen (optimistic assumption).
fn lookup_value(
    vid: ValueId,
    values: &BTreeMap<ValueId, SccpValue>,
    module_constants: &BTreeMap<ValueId, Constant>,
) -> SccpValue {
    if let Some(&val) = values.get(&vid) {
        return val;
    }
    if let Some(constant) = module_constants.get(&vid) {
        return constant_to_sccp(constant);
    }
    SccpValue::Top
}

/// Meet-update a value in the lattice. If the value changes (lowers), add it
/// to the SSA worklist for re-propagation.
fn update_value(
    vid: ValueId,
    new_val: SccpValue,
    values: &mut BTreeMap<ValueId, SccpValue>,
    ssa_worklist: &mut VecDeque<ValueId>,
) {
    let old = values.get(&vid).copied().unwrap_or(SccpValue::Top);
    let met = old.meet(new_val);
    if met != old {
        values.insert(vid, met);
        ssa_worklist.push_back(vid);
    }
}

/// Mark a CFG edge executable and schedule the target for (re-)evaluation.
///
/// Scheduling on a newly-executable BLOCK is not enough: a `Phi` meets only over
/// the incoming edges that are executable *at the moment it is evaluated*
/// (see the `Operation::Phi` arm), so when a NEW EDGE reaches a block that was
/// already executable, that block's phis were computed over a strictly smaller
/// edge set and are now stale. Wegman-Zadeck requires re-visiting them; dropping
/// that obligation freezes a phi at whatever it meant on its first evaluation.
///
/// This was mechanism "M4" (`plans/213-movement1-RESULTS.md`), and it reached
/// production as a **wrong TRUE**. A dispatch chain
/// `s=8466 -> 8496 -> 8512 -> INT_MAX` froze its join phi at the value it had
/// before the deepest arm's edge became executable, SCCP published that frozen
/// singleton, the interval solver adopted it through `constant_map`, and
/// `saf verify --property no-overflow` answered `true` on a program that
/// overflows. The literal incoming values never wake the SSA worklist either —
/// module literals are seeded once and never re-pushed — so the edge set is the
/// only thing that can trigger the re-visit.
fn mark_edge_executable(
    from: BlockId,
    to: BlockId,
    edges: &mut BTreeSet<(BlockId, BlockId)>,
    blocks: &mut BTreeSet<BlockId>,
    worklist: &mut VecDeque<BlockId>,
) {
    let edge_is_new = edges.insert((from, to));
    let block_is_new = blocks.insert(to);
    // Either condition obliges a visit: a new block has never been evaluated, and
    // a new edge invalidates the phis of one that has.
    if block_is_new || edge_is_new {
        worklist.push_back(to);
    }
}

/// Evaluate a single instruction, updating the lattice and worklists.
// NOTE: This function implements the SCCP instruction evaluation for all
// AIR operations as a single cohesive unit. Splitting would obscure the algorithm.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
/// Restore the lattice invariant [`evaluate_cast`] documents: the `i128` inside a
/// [`SccpValue::Constant`] is the SIGNED interpretation of the value at its OWN
/// bit-width.
///
/// [`evaluate_binary`] computes in `i128` with `wrapping_*`, which wraps at 128
/// bits — not where the value actually wraps. So `add i32 -2147483648,
/// -2147483648` yields `-4294967296`, a value no `i32` can hold, whose true value
/// is `0`. A constant that lies makes `CondBr` mark a LIVE block dead, and
/// `prove_unreachable` then reads the pre-seeded ⊥ as a proof. Witness:
/// `tests/programs/c/svcomp/unreach_false_wrapped_add.c`. (plans/213 M2)
///
/// * Comparisons already produce a normal `i1` 0/1.
/// * `And`/`Or`/`Xor` of two operands already normalised at width `b` stay within
///   `b` — their `i128` sign-extensions agree bit for bit — so they need nothing.
/// * `Add`/`Sub`/`Mul`/`SDiv`/`SRem` can leave the width and must be wrapped.
///
/// Where the width is unknown no claim is justified, so the fold drops to
/// `Bottom` — this lattice's "overdetermined". That is the same conservative
/// escape the `ZExt` arm of `evaluate_cast` uses, and `Bottom` can only REMOVE
/// blocks from `dead_blocks`, never add them, so it cannot manufacture a ⊥.
fn normalize_binary_result(kind: BinaryOp, v: SccpValue, bits: Option<u8>) -> SccpValue {
    if !matches!(
        kind,
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::SDiv | BinaryOp::SRem
    ) {
        return v;
    }
    let SccpValue::Constant(c) = v else {
        return v;
    };
    let Some(b) = bits else {
        return SccpValue::Bottom;
    };
    if !(1..128).contains(&b) {
        return SccpValue::Bottom;
    }
    let mask = (1i128 << b) - 1;
    let masked = c & mask;
    let sign_bit = 1i128 << (b - 1);
    SccpValue::Constant(if masked & sign_bit == 0 {
        masked
    } else {
        masked | !mask
    })
}

// One SSA-value dispatch over every AIR operation, so it is inherently wide and
// long; splitting it would scatter the lattice rules that must stay legible as a
// unit. `int_widths` is the tenth parameter, added so a binary fold can be
// normalised to its result width (plans/213 M2) — the alternative, a context
// struct, would be threaded through only this one call site.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn evaluate_instruction(
    inst: &saf_core::air::Instruction,
    block_id: BlockId,
    values: &mut BTreeMap<ValueId, SccpValue>,
    module_constants: &BTreeMap<ValueId, Constant>,
    read_only_globals: &BTreeMap<ValueId, SccpValue>,
    ssa_worklist: &mut VecDeque<ValueId>,
    executable_edges: &mut BTreeSet<(BlockId, BlockId)>,
    executable_blocks: &mut BTreeSet<BlockId>,
    cfg_worklist: &mut VecDeque<BlockId>,
    int_widths: &BTreeMap<TypeId, u8>,
) {
    match &inst.op {
        // -----------------------------------------------------------------
        // Phi: meet over executable predecessors
        // -----------------------------------------------------------------
        Operation::Phi { incoming } => {
            if let Some(dst) = inst.dst {
                let mut result = SccpValue::Top;
                for (pred_block, val_id) in incoming {
                    // Only consider values from executable predecessors.
                    if executable_edges.contains(&(*pred_block, block_id)) {
                        let val = lookup_value(*val_id, values, module_constants);
                        result = result.meet(val);
                    }
                }
                update_value(dst, result, values, ssa_worklist);
            }
        }

        // -----------------------------------------------------------------
        // Binary operations
        // -----------------------------------------------------------------
        Operation::BinaryOp { kind } => {
            if let Some(dst) = inst.dst {
                if inst.operands.len() >= 2 {
                    let lhs = lookup_value(inst.operands[0], values, module_constants);
                    let rhs = lookup_value(inst.operands[1], values, module_constants);
                    let result = normalize_binary_result(
                        *kind,
                        evaluate_binary(*kind, lhs, rhs),
                        inst.result_type.and_then(|t| int_widths.get(&t).copied()),
                    );
                    update_value(dst, result, values, ssa_worklist);
                } else {
                    update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                }
            }
        }

        // -----------------------------------------------------------------
        // Cast operations
        // -----------------------------------------------------------------
        Operation::Cast { kind, target_bits } => {
            if let Some(dst) = inst.dst {
                if let Some(&src_vid) = inst.operands.first() {
                    let src = lookup_value(src_vid, values, module_constants);
                    let result = evaluate_cast(*kind, src, *target_bits);
                    update_value(dst, result, values, ssa_worklist);
                } else {
                    update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                }
            }
        }

        // -----------------------------------------------------------------
        // Conditional branch
        // -----------------------------------------------------------------
        Operation::CondBr {
            then_target,
            else_target,
        } => {
            let cond = inst
                .operands
                .first()
                .map(|vid| lookup_value(*vid, values, module_constants));

            if let Some(SccpValue::Constant(c)) = cond {
                if c != 0 {
                    mark_edge_executable(
                        block_id,
                        *then_target,
                        executable_edges,
                        executable_blocks,
                        cfg_worklist,
                    );
                } else {
                    mark_edge_executable(
                        block_id,
                        *else_target,
                        executable_edges,
                        executable_blocks,
                        cfg_worklist,
                    );
                }
            } else {
                // Top, Bottom, or malformed — conservatively mark both.
                mark_edge_executable(
                    block_id,
                    *then_target,
                    executable_edges,
                    executable_blocks,
                    cfg_worklist,
                );
                mark_edge_executable(
                    block_id,
                    *else_target,
                    executable_edges,
                    executable_blocks,
                    cfg_worklist,
                );
            }
        }

        // -----------------------------------------------------------------
        // Switch
        // -----------------------------------------------------------------
        Operation::Switch { default, cases } => {
            let disc = inst
                .operands
                .first()
                .map(|vid| lookup_value(*vid, values, module_constants));

            if let Some(SccpValue::Constant(c)) = disc {
                // Find the matching case, or fall through to default.
                let target = cases
                    .iter()
                    .find(|(val, _)| i128::from(*val) == c)
                    .map_or(*default, |(_, tgt)| *tgt);
                mark_edge_executable(
                    block_id,
                    target,
                    executable_edges,
                    executable_blocks,
                    cfg_worklist,
                );
            } else {
                // Top, Bottom, or malformed — mark all targets.
                mark_edge_executable(
                    block_id,
                    *default,
                    executable_edges,
                    executable_blocks,
                    cfg_worklist,
                );
                for (_, tgt) in cases {
                    mark_edge_executable(
                        block_id,
                        *tgt,
                        executable_edges,
                        executable_blocks,
                        cfg_worklist,
                    );
                }
            }
        }

        // -----------------------------------------------------------------
        // Unconditional branch
        // -----------------------------------------------------------------
        Operation::Br { target } => {
            mark_edge_executable(
                block_id,
                *target,
                executable_edges,
                executable_blocks,
                cfg_worklist,
            );
        }

        // -----------------------------------------------------------------
        // Copy (identity)
        // -----------------------------------------------------------------
        Operation::Copy => {
            if let Some(dst) = inst.dst {
                if let Some(&src_vid) = inst.operands.first() {
                    let src = lookup_value(src_vid, values, module_constants);
                    update_value(dst, src, values, ssa_worklist);
                } else {
                    update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                }
            }
        }

        // -----------------------------------------------------------------
        // Load: check if loading from a read-only global with known init
        // -----------------------------------------------------------------
        Operation::Load => {
            if let Some(dst) = inst.dst {
                if let Some(&ptr) = inst.operands.first() {
                    if let Some(&global_val) = read_only_globals.get(&ptr) {
                        // Loading from a read-only global — use its init value
                        update_value(dst, global_val, values, ssa_worklist);
                    } else {
                        update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                    }
                } else {
                    update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                }
            }
        }

        // -----------------------------------------------------------------
        // Default: Store, Call, Alloca, Gep, etc. — overdetermined
        // -----------------------------------------------------------------
        _ => {
            if let Some(dst) = inst.dst {
                update_value(dst, SccpValue::Bottom, values, ssa_worklist);
            }
        }
    }
}

/// Evaluate a binary operation when both operands may be constant.
fn evaluate_binary(kind: BinaryOp, lhs: SccpValue, rhs: SccpValue) -> SccpValue {
    match (lhs, rhs) {
        (SccpValue::Constant(a), SccpValue::Constant(b)) => {
            let result = match kind {
                BinaryOp::Add => Some(a.wrapping_add(b)),
                BinaryOp::Sub => Some(a.wrapping_sub(b)),
                BinaryOp::Mul => Some(a.wrapping_mul(b)),
                BinaryOp::And => Some(a & b),
                BinaryOp::Or => Some(a | b),
                BinaryOp::Xor => Some(a ^ b),
                BinaryOp::ICmpEq => Some(i128::from(a == b)),
                BinaryOp::ICmpNe => Some(i128::from(a != b)),
                BinaryOp::ICmpSlt => Some(i128::from(a < b)),
                BinaryOp::ICmpSle => Some(i128::from(a <= b)),
                BinaryOp::ICmpSgt => Some(i128::from(a > b)),
                BinaryOp::ICmpSge => Some(i128::from(a >= b)),
                BinaryOp::SDiv => {
                    if b != 0 {
                        Some(a.wrapping_div(b))
                    } else {
                        None
                    }
                }
                BinaryOp::SRem => {
                    if b != 0 {
                        Some(a.wrapping_rem(b))
                    } else {
                        None
                    }
                }
                // Unsigned comparisons, float ops, shifts — not folded here.
                _ => None,
            };
            result.map_or(SccpValue::Bottom, SccpValue::Constant)
        }
        // If either operand is Bottom, the result is Bottom.
        (SccpValue::Bottom, _) | (_, SccpValue::Bottom) => SccpValue::Bottom,
        // If either operand is still Top, we don't know yet — stay Top.
        _ => SccpValue::Top,
    }
}

/// Evaluate a cast operation when the source may be constant.
///
/// **Lattice invariant:** the `i128` inside a [`SccpValue::Constant`] is the
/// SIGNED interpretation of the value at its OWN bit-width. Every arm below
/// exists to preserve that; breaking it is a wrong-TRUE generator, not merely an
/// imprecision (plans/213).
///
/// Why it is that severe: a mis-folded constant lets [`evaluate_binary`] decide
/// an `ICmp` whose real value is unknown. A `CondBr` on a constant condition
/// marks one edge non-executable, so every block behind it is collected as dead;
/// `fixpoint.rs` skips a dead block BEFORE propagating to its successors, so a
/// live successor keeps the ⊥ every block is pre-seeded with; and
/// `prove_unreachable` reads that ⊥ as a proof the error is unreachable.
///
/// * `SExt` and integer `Bitcast` preserve the signed value, so they really are
///   pass-throughs.
/// * `ZExt` does **not**: `zext i16 -1 to i32` is 65535. Computing that needs the
///   SOURCE width, and `Operation::Cast` carries only `target_bits`. A
///   non-negative source is unchanged by zero-extension and still folds; a
///   negative one cannot be folded soundly and drops to `Bottom`
///   (overdetermined), which is this lattice's "unknown".
/// * `Trunc` keeps the low `bits` and then REINTERPRETS them as a signed value of
///   that width: `trunc i32 -1 to i16` is −1, and `trunc i32 255 to i8` is −1,
///   because `0xFF` as a signed `i8` is −1.
fn evaluate_cast(kind: CastKind, src: SccpValue, target_bits: Option<u8>) -> SccpValue {
    match kind {
        CastKind::SExt | CastKind::Bitcast => src,
        CastKind::ZExt => match src {
            // Cannot zero-extend without the source width; refuse to fold.
            SccpValue::Constant(v) if v < 0 => SccpValue::Bottom,
            _ => src,
        },
        CastKind::Trunc => match (src, target_bits) {
            (SccpValue::Constant(v), Some(bits)) if (1..128).contains(&bits) => {
                let mask = (1i128 << bits) - 1;
                let masked = v & mask;
                let sign_bit = 1i128 << (bits - 1);
                SccpValue::Constant(if masked & sign_bit == 0 {
                    masked
                } else {
                    masked | !mask
                })
            }
            // A constant we cannot narrow soundly must not keep its wider value:
            // `trunc <unknown width>` of 256 is not 256.
            (SccpValue::Constant(_), _) => SccpValue::Bottom,
            // Top / Bottom carry no value to corrupt.
            _ => src,
        },
        // Float-to-int, ptr-to-int, etc. — not folded.
        _ => SccpValue::Bottom,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Lattice tests ----

    #[test]
    fn sccp_lattice_meet_top_with_const() {
        assert_eq!(
            SccpValue::Top.meet(SccpValue::Constant(5)),
            SccpValue::Constant(5)
        );
    }

    #[test]
    fn sccp_lattice_meet_same_const() {
        assert_eq!(
            SccpValue::Constant(5).meet(SccpValue::Constant(5)),
            SccpValue::Constant(5)
        );
    }

    #[test]
    fn sccp_lattice_meet_different_const() {
        assert_eq!(
            SccpValue::Constant(5).meet(SccpValue::Constant(3)),
            SccpValue::Bottom
        );
    }

    #[test]
    fn sccp_lattice_meet_bottom_absorbs() {
        assert_eq!(
            SccpValue::Bottom.meet(SccpValue::Constant(5)),
            SccpValue::Bottom
        );
    }

    // ---- Solver tests ----

    #[test]
    fn sccp_run_trivial_function() {
        use saf_core::air::{AirBlock, Instruction};
        use saf_core::ids::{FunctionId, InstId, ModuleId};

        let block_id = BlockId::new(1);
        let func_id = FunctionId::new(2);
        let mut block = AirBlock::new(block_id);
        block.instructions = vec![Instruction::new(InstId::new(3), Operation::Ret)];
        let mut func = AirFunction::new(func_id, "test");
        func.blocks = vec![block];
        func.entry_block = Some(block_id);
        func.rebuild_block_index();

        let module = AirModule::new(ModuleId::new(0));
        // Module has no functions added, test with standalone function
        let result = run_sccp_function(&func, &module.constants, &BTreeMap::new());
        assert!(result.dead_blocks.is_empty());
    }

    #[test]
    fn sccp_detects_dead_else_branch() {
        use saf_core::air::*;
        use saf_core::ids::*;

        let const_5_id = ValueId::new(100);
        let five_id = ValueId::new(101);
        let cond_id = ValueId::new(102);

        let entry_id = BlockId::new(1);
        let then_id = BlockId::new(2);
        let else_id = BlockId::new(3);

        let mut entry = AirBlock::with_label(entry_id, "entry");
        entry.instructions = vec![
            Instruction::new(
                InstId::new(10),
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpEq,
                },
            )
            .with_operands(vec![const_5_id, five_id])
            .with_dst(cond_id),
            Instruction::new(
                InstId::new(11),
                Operation::CondBr {
                    then_target: then_id,
                    else_target: else_id,
                },
            )
            .with_operands(vec![cond_id]),
        ];

        let mut then_block = AirBlock::with_label(then_id, "then");
        then_block.instructions = vec![Instruction::new(InstId::new(12), Operation::Ret)];

        let mut else_block = AirBlock::with_label(else_id, "else");
        else_block.instructions = vec![Instruction::new(InstId::new(13), Operation::Ret)];

        let mut func = AirFunction::new(FunctionId::new(1), "test_dead_else");
        func.blocks = vec![entry, then_block, else_block];
        func.entry_block = Some(entry_id);
        func.rebuild_block_index();

        let mut module_constants = BTreeMap::new();
        module_constants.insert(const_5_id, Constant::Int { value: 5, bits: 32 });
        module_constants.insert(five_id, Constant::Int { value: 5, bits: 32 });

        let result = run_sccp_function(&func, &module_constants, &BTreeMap::new());

        assert!(
            result.dead_blocks.contains(&else_id),
            "else block should be dead"
        );
        assert!(
            !result.dead_blocks.contains(&then_id),
            "then block should be alive"
        );
        assert_eq!(result.constants.get(&cond_id), Some(&1));
    }

    // ---- Cast evaluation tests ----

    #[test]
    fn evaluate_cast_trunc_masks_bits() {
        // 256 truncated to i8 should become 0 (256 & 0xFF = 0)
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(256), Some(8)),
            SccpValue::Constant(0)
        );
    }

    #[test]
    fn evaluate_cast_trunc_reinterprets_in_range_value_as_signed() {
        // CORRECTED (plans/213): 255 truncated to i8 is NOT 255. The low 8 bits
        // are `0xFF`, and this lattice stores the SIGNED interpretation at the
        // value's own width, so the result is -1. The old expectation encoded the
        // very invariant violation that let a mis-folded constant kill a live
        // branch and turn a reachable error into a wrong PROVE.
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(255), Some(8)),
            SccpValue::Constant(-1)
        );
    }

    #[test]
    fn evaluate_cast_trunc_no_target_bits_refuses_to_fold() {
        // CORRECTED (plans/213): passing an un-narrowable constant through is
        // unsound -- `trunc` of 256 is 0 at i8 and 256 at i16, so keeping 256
        // asserts a value the program may not have. Without the width the only
        // sound answer is `Bottom` (this lattice's "unknown").
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(256), None),
            SccpValue::Bottom
        );
    }

    #[test]
    fn evaluate_cast_trunc_non_constant_passes_through() {
        // Non-constant values pass through unchanged
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Top, Some(8)),
            SccpValue::Top
        );
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Bottom, Some(8)),
            SccpValue::Bottom
        );
    }

    #[test]
    fn evaluate_cast_zext_sext_bitcast_pass_through() {
        // Extension and bitcast always pass through
        assert_eq!(
            evaluate_cast(CastKind::ZExt, SccpValue::Constant(42), Some(64)),
            SccpValue::Constant(42)
        );
        assert_eq!(
            evaluate_cast(CastKind::SExt, SccpValue::Constant(42), Some(64)),
            SccpValue::Constant(42)
        );
        assert_eq!(
            evaluate_cast(CastKind::Bitcast, SccpValue::Constant(42), Some(64)),
            SccpValue::Constant(42)
        );
    }

    // =========================================================================
    // plans/213 -- SCCP cast folding must preserve the lattice invariant
    // "the i128 in `SccpValue::Constant` is the SIGNED interpretation of the
    // value at its OWN width".
    //
    // Breaking it is not merely imprecise, it is a WRONG TRUE generator. A
    // mis-folded constant makes `evaluate_binary` decide an ICmp that is really
    // unknown; `CondBr` with a constant condition then marks one edge
    // non-executable and every block behind it DEAD; `fixpoint.rs` skips dead
    // blocks BEFORE propagating to their successors, so a live successor keeps
    // the ⊥ every block is pre-seeded with; and `prove_unreachable` reads that ⊥
    // as a proof that the error is unreachable. -32 points, uncapped by dedup.
    // =========================================================================

    /// RED. `zext i16 -1 to i32` is **65535**, not −1. `evaluate_cast` is handed
    /// only `target_bits`, never the SOURCE width, so it cannot compute the
    /// zero-extended value — and must therefore refuse to fold rather than pass
    /// the signed value through. Witness: `bitvector-regression/signextension-1`.
    #[test]
    fn evaluate_cast_zext_of_negative_does_not_pass_through() {
        assert_ne!(
            evaluate_cast(CastKind::ZExt, SccpValue::Constant(-1), Some(32)),
            SccpValue::Constant(-1),
            "zext of a negative constant is NOT the same signed value"
        );
    }

    /// RED, the same hole reached through a narrow unsigned type: LLVM stores
    /// `unsigned char c = 200` as `i8 -56`, so `zext i8 -56 to i32` must not fold
    /// to −56 (the real value is 200).
    #[test]
    fn evaluate_cast_zext_of_narrow_negative_does_not_pass_through() {
        assert_ne!(
            evaluate_cast(CastKind::ZExt, SccpValue::Constant(-56), Some(32)),
            SccpValue::Constant(-56)
        );
    }

    /// RED. `trunc i32 -1 to i16` is **−1**, not 65535: the low 16 bits are
    /// `0xFFFF`, whose SIGNED i16 interpretation is −1. Masking without
    /// sign-normalising leaves a value that no longer means what the lattice says
    /// it means. Witness: `m1probe/t1.c`.
    #[test]
    fn evaluate_cast_trunc_sign_normalizes_negative() {
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(-1), Some(16)),
            SccpValue::Constant(-1)
        );
    }

    /// RED, the other direction: a POSITIVE wide constant whose low bits have the
    /// sign bit set truncates to a NEGATIVE narrow value. `trunc i32 65535 to i16`
    /// is −1.
    #[test]
    fn evaluate_cast_trunc_wraps_positive_to_negative() {
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(65535), Some(16)),
            SccpValue::Constant(-1)
        );
    }

    /// RED. `trunc i32 200 to i8` is −56 (`0xC8` as signed i8), not 200.
    #[test]
    fn evaluate_cast_trunc_byte_sign_boundary() {
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(200), Some(8)),
            SccpValue::Constant(-56)
        );
    }

    /// GUARD, must stay green. Zero-extending a NON-negative constant really is
    /// the identity, and that is the overwhelmingly common case — the fix must
    /// not cost this precision.
    #[test]
    fn evaluate_cast_zext_of_nonnegative_still_passes_through() {
        assert_eq!(
            evaluate_cast(CastKind::ZExt, SccpValue::Constant(42), Some(64)),
            SccpValue::Constant(42)
        );
        assert_eq!(
            evaluate_cast(CastKind::ZExt, SccpValue::Constant(0), Some(8)),
            SccpValue::Constant(0)
        );
    }

    /// GUARD, must stay green. `SExt` DOES preserve the signed value, so it is a
    /// genuine pass-through even for negatives. This is what makes
    /// `loop-simple`-style `sext` programs analyse correctly today.
    #[test]
    fn evaluate_cast_sext_of_negative_still_passes_through() {
        assert_eq!(
            evaluate_cast(CastKind::SExt, SccpValue::Constant(-1), Some(32)),
            SccpValue::Constant(-1)
        );
    }
}
