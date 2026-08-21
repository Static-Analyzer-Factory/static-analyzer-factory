//! Scalar-alloca → SSA promotion (a self-contained `mem2reg`) run **before**
//! ranking, so `-O0`-spilled loop counters (`*i`) and spilled recursion
//! parameters become affine SSA state the [`crate::ranking`] synthesizer can rank.
//!
//! # Why this exists
//!
//! The ranking pipeline ([`crate::ranking::resolve_affine`] and friends) treats a
//! `Load` as an **opaque free leaf**: a memory-backed induction variable
//! (`for (i = 0; i < n; i++)` compiled at `-O0`, where `i` lives in a stack slot
//! read/written by `load`/`store`) therefore carries no affine transition, so the
//! loop's rank cannot be synthesized and the whole termination proof abstains. The
//! same holds for a self-recursive function whose parameter `n` is spilled to a
//! stack slot: the recursive-call argument `n - 1` is `load; sub 1`, whose `load`
//! is opaque. LLVM's own `mem2reg` promotes most such slots at ingestion, but it
//! refuses some cells (e.g. `alloca()`-library-call results, or slots surviving a
//! non-canonical `-O0` shape), and re-running it there is out of SAF's control.
//! This pass is an **AIR-level backstop**: any fixed-size integer scalar slot whose
//! address provably never escapes is promoted to SSA phis, exposing its affine
//! update to the ranker.
//!
//! # Soundness
//!
//! Promotion is the classic, semantics-preserving `mem2reg` transformation
//! (Cytron et al., *Efficiently Computing Static Single Assignment Form*, TOPLAS
//! 1991): SSA phis at the iterated dominance frontier of the slot's stores, then a
//! dominator-tree renaming that replaces each load by its reaching definition.
//! Promotion is sound **only** when the slot is not aliased, so we promote a slot
//! `p` (an [`Operation::Alloca`] result) **only** when a **fail-closed escape gate**
//! holds — *every* use of `p` is either
//!
//! - the pointer operand of a [`Operation::Load`] (`operands[0]`), or
//! - the pointer operand of a [`Operation::Store`] (`operands[1]`), and **not** its
//!   stored-value operand (`operands[0]`).
//!
//! Any other use — feeding a `Gep`, a `Call`, a `Cast`, being *stored as a value*
//! (its address taken), or appearing in any block unreachable from entry — makes
//! the slot ineligible and it is left untouched. We further require the slot to be
//! a **fixed-size integer scalar**: every load shares one integer `result_type`,
//! and every stored value's width matches it (a constant's `bits`, an
//! instruction's `result_type`, or a parameter's `param_type`); a store whose value
//! type cannot be confirmed abstains the whole slot. Under this gate the slot holds
//! exactly one scalar value with no aliasing, so replacing loads by reaching stores
//! preserves the program's semantics — the promoted function is behaviorally
//! identical, and any ranking proof over it is a proof over the original.
//!
//! ## Why this adds recall over LLVM's own `mem2reg`
//!
//! SAF ingests already-`mem2reg`'d IR, so ordinary `-O0` counters are usually
//! promoted before AIR. LLVM's `mem2reg`, however, deliberately **refuses**
//! `volatile` slots (their accesses are observable) — yet SAF's frontend erases
//! the `volatile` marker, so a `volatile int i` counter reaches AIR as plain
//! `load`/`store` and the ranker sees only opaque loads (⇒ abstain). Promoting it
//! here is **sound for SV-COMP's semantics**: `i` has automatic storage and (by the
//! escape gate) an address that never leaves the function, so no other execution
//! context — thread, signal handler, or device mapping — can observe or mutate it;
//! its value sequence is therefore identical to a non-`volatile` local, and a loop
//! ranked over it genuinely terminates. Environment nondeterminism in SV-COMP is
//! modelled by `__VERIFIER_nondet_*` calls, never by a `volatile` local; a slot
//! actually reassigned from a nondet call stores an opaque value, which
//! [`crate::ranking`] resolves to a free symbol and cannot rank (it abstains). The
//! same promotion also backstops any other non-escaping scalar that survives
//! LLVM's pipeline (e.g. an `alloca`-family cell the toolchain leaves in memory).
//!
//! Because the transformation is semantics-preserving and only ever *adds* affine
//! structure the ranker may exploit (never removes a guard, never changes the CFG:
//! phis are inserted at block heads, loads/stores/allocas removed — no block or
//! edge is added or deleted), it can only turn abstentions into sound `true`s; it
//! can never introduce a wrong verdict. When no slot is eligible the function is
//! returned unchanged (a structural clone), so existing proofs never regress.

use std::collections::{BTreeMap, BTreeSet};

use saf_analysis::cfg::Cfg;
use saf_analysis::z3_utils::compute_dominators;
use saf_core::air::{AirFunction, AirModule, AirType, Constant, Instruction, Operation};
use saf_core::ids::{BlockId, InstId, TypeId, ValueId};

/// Bound on the reaching-definition chase when a stored value is itself a
/// (removed) load result. Chases resolve in dominator order so a real chain is
/// short; the cap is only a guard against a pathological self-referential map.
const CHASE_LIMIT: usize = 1 << 16;

/// Promote every function's non-escaped fixed-size integer scalar allocas to SSA
/// phis, returning a new module. Functions with nothing to promote are cloned
/// unchanged, so the result is byte-identical to the input when no slot is
/// eligible (determinism / no-regression).
#[must_use]
pub fn promote_module(module: &AirModule) -> AirModule {
    let mut out = module.clone();
    for func in &mut out.functions {
        if let Some(promoted) = promote_function(func, module) {
            *func = promoted;
        }
    }
    out
}

/// Promote one function's eligible scalar allocas. Returns `Some(new_func)` when
/// at least one slot was promoted, or `None` when nothing is eligible (the caller
/// then keeps the original — avoids a needless clone).
#[must_use]
pub fn promote_function(func: &AirFunction, module: &AirModule) -> Option<AirFunction> {
    if func.is_declaration || func.blocks.is_empty() {
        return None;
    }
    // Cheap pre-filter: skip the CFG/dominator work entirely when the function has
    // no fixed-size alloca to promote (the common case).
    if !func.blocks.iter().any(|b| {
        b.instructions.iter().any(|i| {
            matches!(
                i.op,
                Operation::Alloca {
                    size_bytes: Some(_)
                }
            )
        })
    }) {
        return None;
    }

    let cfg = Cfg::build(func);
    let idom = compute_dominators(&cfg);
    // Reachable-from-entry blocks: entry plus every block with an immediate
    // dominator. Slots touched in an unreachable block are not promoted.
    let mut reachable: BTreeSet<BlockId> = idom.keys().copied().collect();
    reachable.insert(cfg.entry);

    let slots = eligible_slots(func, module, &reachable);
    if slots.is_empty() {
        return None;
    }

    // Iterated dominance frontier of each slot's store-blocks ⇒ phi placement.
    let df = dominance_frontiers(&cfg, &idom);
    // phi_at[block] = the slots that get a phi at `block`'s head (sorted by slot id).
    let mut phi_at: BTreeMap<BlockId, Vec<ValueId>> = BTreeMap::new();
    // phi_dst[(block, slot)] = the fresh SSA value id for that phi.
    let mut phi_dst: BTreeMap<(BlockId, ValueId), ValueId> = BTreeMap::new();
    for (&slot, info) in &slots {
        let blocks = iterated_dominance_frontier(&info.store_blocks, &df, &reachable);
        for b in blocks {
            phi_at.entry(b).or_default().push(slot);
            phi_dst.insert((b, slot), fresh_phi_value(func, b, slot));
        }
    }
    for v in phi_at.values_mut() {
        v.sort_unstable();
    }

    // Rename: dominator-tree walk producing (a) load-result → reaching-value
    // substitutions and (b) each inserted phi's incoming operands.
    let children = dom_children(&idom, &reachable);
    let mut subst: BTreeMap<ValueId, ValueId> = BTreeMap::new();
    let mut incoming: BTreeMap<(BlockId, ValueId), Vec<(BlockId, ValueId)>> = BTreeMap::new();
    let block_index: BTreeMap<BlockId, usize> = func
        .blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.id, i))
        .collect();
    rename(
        cfg.entry,
        BTreeMap::new(),
        func,
        &cfg,
        &children,
        &block_index,
        &slots,
        &phi_at,
        &phi_dst,
        &mut subst,
        &mut incoming,
    );

    Some(rebuild(func, &slots, &phi_at, &phi_dst, &incoming, &subst))
}

/// Metadata for one promotable slot.
struct SlotInfo {
    /// The scalar integer element type of the slot (every load's `result_type`).
    elem_type: TypeId,
    /// Blocks that contain a `store` to this slot (the SSA def-blocks).
    store_blocks: BTreeSet<BlockId>,
}

/// A single use of an alloca pointer, classified by the escape gate.
enum PtrUse {
    /// Pointer operand of a load — permitted.
    Load,
    /// Pointer operand of a store (with the given stored-value id) — permitted.
    Store(ValueId),
    /// Anything else (Gep base, call arg, cast, stored-as-value, …) — escape.
    Escape,
}

/// Identify the function's promotable scalar allocas (see the module-level escape
/// and type gates). Returns a map slot-pointer → [`SlotInfo`], sorted by id.
fn eligible_slots(
    func: &AirFunction,
    module: &AirModule,
    reachable: &BTreeSet<BlockId>,
) -> BTreeMap<ValueId, SlotInfo> {
    // Candidate allocas: fixed-size (`size_bytes` known) alloca results.
    let mut candidates: BTreeSet<ValueId> = BTreeSet::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::Alloca {
                size_bytes: Some(_),
            } = inst.op
            {
                if let Some(dst) = inst.dst {
                    candidates.insert(dst);
                }
            }
        }
    }
    if candidates.is_empty() {
        return BTreeMap::new();
    }

    // Value → integer bit width for stored-value type confirmation.
    let vtype = value_widths(func, module);

    // Per-candidate accumulation while scanning every instruction/operand.
    let mut escaped: BTreeSet<ValueId> = BTreeSet::new();
    let mut load_type: BTreeMap<ValueId, Option<TypeId>> = BTreeMap::new(); // None ⇒ inconsistent
    let mut store_blocks: BTreeMap<ValueId, BTreeSet<BlockId>> = BTreeMap::new();
    let mut has_load: BTreeSet<ValueId> = BTreeSet::new();
    let mut bad_store_type: BTreeSet<ValueId> = BTreeSet::new();

    for block in &func.blocks {
        let block_reachable = reachable.contains(&block.id);
        for inst in &block.instructions {
            for (ptr, use_) in classify_ptr_uses(inst, &candidates) {
                if !block_reachable {
                    // A touch in an unreachable block ⇒ do not promote this slot.
                    escaped.insert(ptr);
                    continue;
                }
                match use_ {
                    PtrUse::Escape => {
                        escaped.insert(ptr);
                    }
                    PtrUse::Load => {
                        has_load.insert(ptr);
                        let entry = load_type.entry(ptr).or_insert_with(|| inst.result_type);
                        // A load whose type disagrees with a prior load, or which
                        // has no known integer type, disqualifies the slot.
                        if *entry != inst.result_type || inst.result_type.is_none() {
                            *entry = None;
                        }
                    }
                    PtrUse::Store(val) => {
                        store_blocks.entry(ptr).or_default().insert(block.id);
                        // The stored value's width must match the slot's scalar
                        // width; an unconfirmable store type abstains the slot.
                        if !store_value_ok(val, module, &vtype) {
                            bad_store_type.insert(ptr);
                        }
                    }
                }
            }
        }
    }

    let mut out = BTreeMap::new();
    for ptr in candidates {
        if escaped.contains(&ptr) || bad_store_type.contains(&ptr) {
            continue;
        }
        // Need at least one load (a benefit) and one store (an initializer).
        if !has_load.contains(&ptr) || !store_blocks.contains_key(&ptr) {
            continue;
        }
        let Some(Some(elem)) = load_type.get(&ptr).copied() else {
            continue; // no consistent load type
        };
        // The element type must be a representable integer (ranking's domain) and
        // its width must equal every stored value's width.
        let Some(w) = int_width(module, elem) else {
            continue;
        };
        // Confirm no store's confirmed width contradicts the load width.
        let blocks = store_blocks.get(&ptr).cloned().unwrap_or_default();
        if !stores_match_width(&ptr, w, func, module, &vtype) {
            continue;
        }
        out.insert(
            ptr,
            SlotInfo {
                elem_type: elem,
                store_blocks: blocks,
            },
        );
    }
    out
}

/// Classify each occurrence of a candidate alloca pointer in `inst`'s operands.
/// A pointer may appear once (the common case); returns every occurrence so a
/// stored-as-value self-reference (`store p, p`) is caught as an escape.
fn classify_ptr_uses(inst: &Instruction, candidates: &BTreeSet<ValueId>) -> Vec<(ValueId, PtrUse)> {
    let mut uses = Vec::new();
    match &inst.op {
        Operation::Load => {
            for (i, &op) in inst.operands.iter().enumerate() {
                if candidates.contains(&op) {
                    // Only operand[0] is the load pointer; any other operand slot
                    // referencing the alloca is unexpected ⇒ escape.
                    uses.push((op, if i == 0 { PtrUse::Load } else { PtrUse::Escape }));
                }
            }
        }
        Operation::Store => {
            let val = inst.operands.first().copied();
            for (i, &op) in inst.operands.iter().enumerate() {
                if candidates.contains(&op) {
                    // operand[1] is the store pointer; operand[0] (stored value) or
                    // any other slot referencing the alloca is an address escape.
                    if i == 1 && Some(op) != val {
                        uses.push((op, PtrUse::Store(val.unwrap_or(op))));
                    } else {
                        uses.push((op, PtrUse::Escape));
                    }
                }
            }
        }
        // The alloca-defining instruction itself is not a "use".
        Operation::Alloca { .. } => {}
        // Any other instruction that names the alloca pointer is an escape.
        _ => {
            for &op in &inst.operands {
                if candidates.contains(&op) {
                    uses.push((op, PtrUse::Escape));
                }
            }
        }
    }
    uses
}

/// Map every value with a known integer bit width (instruction results and
/// parameters) to that width, for stored-value type confirmation.
fn value_widths(func: &AirFunction, module: &AirModule) -> BTreeMap<ValueId, u16> {
    let mut out = BTreeMap::new();
    for p in &func.params {
        if let Some(t) = p.param_type {
            if let Some(w) = int_width(module, t) {
                out.insert(p.id, w);
            }
        }
    }
    for block in &func.blocks {
        for inst in &block.instructions {
            if let (Some(dst), Some(t)) = (inst.dst, inst.result_type) {
                if let Some(w) = int_width(module, t) {
                    out.insert(dst, w);
                }
            }
        }
    }
    out
}

/// Is a store's value operand type-compatible with an integer scalar slot? A
/// constant carries its own `bits`; an SSA value / parameter must have a known
/// integer width (checked against the slot width separately). `ZeroInit`/`Undef`
/// are accepted (they take on the slot's own width). Anything whose width is not
/// derivable (a non-int constant, a value with no `result_type`) abstains.
fn store_value_ok(val: ValueId, module: &AirModule, vtype: &BTreeMap<ValueId, u16>) -> bool {
    if let Some(c) = module.constants.get(&val) {
        return matches!(
            c,
            Constant::Int { .. } | Constant::BigInt { .. } | Constant::ZeroInit | Constant::Undef
        );
    }
    vtype.contains_key(&val)
}

/// Confirm every store into slot `ptr` writes a value whose (known) width equals
/// the slot width `w`. Re-scans stores; a constant's `bits` and a value's tracked
/// width must both equal `w` where known (`ZeroInit`/`Undef` impose no width).
fn stores_match_width(
    ptr: &ValueId,
    w: u16,
    func: &AirFunction,
    module: &AirModule,
    vtype: &BTreeMap<ValueId, u16>,
) -> bool {
    for block in &func.blocks {
        for inst in &block.instructions {
            if !matches!(inst.op, Operation::Store) {
                continue;
            }
            if inst.operands.get(1) != Some(ptr) {
                continue;
            }
            let Some(&val) = inst.operands.first() else {
                return false;
            };
            match module.constants.get(&val) {
                Some(Constant::Int { bits, .. } | Constant::BigInt { bits, .. }) => {
                    if u16::from(*bits) != w {
                        return false;
                    }
                }
                Some(Constant::ZeroInit | Constant::Undef) => {}
                Some(_) => return false,
                None => match vtype.get(&val) {
                    Some(&vw) if vw == w => {}
                    _ => return false,
                },
            }
        }
    }
    true
}

/// Integer bit width of a `TypeId`, or `None` for a non-integer / unknown type.
fn int_width(module: &AirModule, type_id: TypeId) -> Option<u16> {
    match module.types.get(&type_id) {
        Some(AirType::Integer { bits }) => Some(*bits),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Dominance frontiers + iterated dominance frontier (Cytron et al.)
// ---------------------------------------------------------------------------

/// Compute the dominance frontier of every block from the immediate-dominator map
/// (Cytron's DF algorithm): for each join block `b` and each predecessor `p`, walk
/// up the dominator tree from `p` adding `b` until reaching `idom(b)`.
fn dominance_frontiers(
    cfg: &Cfg,
    idom: &BTreeMap<BlockId, BlockId>,
) -> BTreeMap<BlockId, BTreeSet<BlockId>> {
    let mut df: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
    for (&b, preds) in &cfg.predecessors {
        if preds.len() < 2 {
            continue;
        }
        let Some(&idom_b) = idom.get(&b) else {
            continue; // entry / unreachable — no frontier contribution
        };
        for &p in preds {
            let mut runner = p;
            // Only walk reachable nodes (those present in `idom` or the entry).
            while runner != idom_b {
                df.entry(runner).or_default().insert(b);
                match idom.get(&runner) {
                    Some(&next) if next != runner => runner = next,
                    _ => break, // reached entry (no idom) or a fixpoint
                }
            }
        }
    }
    df
}

/// Iterated dominance frontier of a set of definition blocks — the phi-placement
/// set (worklist over DF closure), restricted to reachable blocks.
fn iterated_dominance_frontier(
    defs: &BTreeSet<BlockId>,
    df: &BTreeMap<BlockId, BTreeSet<BlockId>>,
    reachable: &BTreeSet<BlockId>,
) -> BTreeSet<BlockId> {
    let mut phi: BTreeSet<BlockId> = BTreeSet::new();
    let mut worklist: Vec<BlockId> = defs.iter().copied().collect();
    while let Some(b) = worklist.pop() {
        let Some(frontier) = df.get(&b) else { continue };
        for &f in frontier {
            if reachable.contains(&f) && phi.insert(f) {
                worklist.push(f);
            }
        }
    }
    phi
}

/// Dominator-tree children of each block (`{c : idom(c) == b}`), reachable only,
/// each child list sorted for a deterministic traversal.
fn dom_children(
    idom: &BTreeMap<BlockId, BlockId>,
    reachable: &BTreeSet<BlockId>,
) -> BTreeMap<BlockId, Vec<BlockId>> {
    let mut children: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
    for (&b, &d) in idom {
        if b != d && reachable.contains(&b) {
            children.entry(d).or_default().push(b);
        }
    }
    for v in children.values_mut() {
        v.sort_unstable();
    }
    children
}

// ---------------------------------------------------------------------------
// Rename
// ---------------------------------------------------------------------------

/// Dominator-tree renaming pass. `cur` maps each slot to its reaching SSA value at
/// the current program point; it is passed by value (cloned per child) to restore
/// on the way back up the tree. Fills `subst` (load-result → reaching value) and
/// `incoming` (inserted-phi operands).
#[allow(clippy::too_many_arguments)]
fn rename(
    b: BlockId,
    mut cur: BTreeMap<ValueId, ValueId>,
    func: &AirFunction,
    cfg: &Cfg,
    children: &BTreeMap<BlockId, Vec<BlockId>>,
    block_index: &BTreeMap<BlockId, usize>,
    slots: &BTreeMap<ValueId, SlotInfo>,
    phi_at: &BTreeMap<BlockId, Vec<ValueId>>,
    phi_dst: &BTreeMap<(BlockId, ValueId), ValueId>,
    subst: &mut BTreeMap<ValueId, ValueId>,
    incoming: &mut BTreeMap<(BlockId, ValueId), Vec<(BlockId, ValueId)>>,
) {
    // Phis inserted at this block become the reaching definition for their slot.
    if let Some(ps) = phi_at.get(&b) {
        for &slot in ps {
            if let Some(&dst) = phi_dst.get(&(b, slot)) {
                cur.insert(slot, dst);
            }
        }
    }

    // Walk the block's original instructions in order.
    if let Some(&bi) = block_index.get(&b) {
        for inst in &func.blocks[bi].instructions {
            match &inst.op {
                Operation::Load => {
                    if let Some(&ptr) = inst.operands.first() {
                        if slots.contains_key(&ptr) {
                            let reaching =
                                cur.get(&ptr).copied().unwrap_or_else(|| undef_value(ptr));
                            if let Some(dst) = inst.dst {
                                subst.insert(dst, reaching);
                            }
                        }
                    }
                }
                Operation::Store => {
                    if let (Some(&val), Some(&ptr)) = (inst.operands.first(), inst.operands.get(1))
                    {
                        if slots.contains_key(&ptr) {
                            cur.insert(ptr, chase(subst, val));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Fill each successor's inserted-phi incoming with this block's exit value.
    if let Some(succs) = cfg.successors.get(&b) {
        for &s in succs {
            if let Some(ps) = phi_at.get(&s) {
                for &slot in ps {
                    let v = cur.get(&slot).copied().unwrap_or_else(|| undef_value(slot));
                    incoming.entry((s, slot)).or_default().push((b, v));
                }
            }
        }
    }

    if let Some(cs) = children.get(&b) {
        for &c in cs {
            rename(
                c,
                cur.clone(),
                func,
                cfg,
                children,
                block_index,
                slots,
                phi_at,
                phi_dst,
                subst,
                incoming,
            );
        }
    }
}

/// Follow `subst` from `v` to a value not further substituted (a phi result,
/// constant, parameter, undef sentinel, or non-promoted SSA value). Bounded.
fn chase(subst: &BTreeMap<ValueId, ValueId>, mut v: ValueId) -> ValueId {
    let mut steps = 0;
    while let Some(&next) = subst.get(&v) {
        if next == v || steps >= CHASE_LIMIT {
            break;
        }
        v = next;
        steps += 1;
    }
    v
}

// ---------------------------------------------------------------------------
// Rebuild
// ---------------------------------------------------------------------------

/// Materialize the promoted function: drop the promoted allocas and their
/// loads/stores, insert the phis at block heads, and remap every remaining use of
/// a removed load result to its reaching value.
fn rebuild(
    func: &AirFunction,
    slots: &BTreeMap<ValueId, SlotInfo>,
    phi_at: &BTreeMap<BlockId, Vec<ValueId>>,
    phi_dst: &BTreeMap<(BlockId, ValueId), ValueId>,
    incoming: &BTreeMap<(BlockId, ValueId), Vec<(BlockId, ValueId)>>,
    subst: &BTreeMap<ValueId, ValueId>,
) -> AirFunction {
    let mut new_func = func.clone();
    for block in &mut new_func.blocks {
        let mut insts: Vec<Instruction> = Vec::new();

        // 1. Inserted phis at the block head (sorted slot order = deterministic).
        if let Some(ps) = phi_at.get(&block.id) {
            for &slot in ps {
                let Some(&dst) = phi_dst.get(&(block.id, slot)) else {
                    continue;
                };
                let mut inc: Vec<(BlockId, ValueId)> =
                    incoming.get(&(block.id, slot)).cloned().unwrap_or_default();
                // Resolve any incoming value that is itself a removed load result,
                // then sort by predecessor for determinism.
                for (_, v) in &mut inc {
                    *v = chase(subst, *v);
                }
                inc.sort_unstable();
                let mut phi = Instruction::new(
                    fresh_phi_inst(func, block.id, slot),
                    Operation::Phi { incoming: inc },
                );
                phi.dst = Some(dst);
                phi.result_type = Some(slots[&slot].elem_type);
                insts.push(phi);
            }
        }

        // 2. Original instructions, minus the promoted allocas/loads/stores, with
        //    every operand remapped through the load-result substitution.
        for inst in &block.instructions {
            if is_removed(inst, slots) {
                continue;
            }
            let mut ni = inst.clone();
            for op in &mut ni.operands {
                *op = chase(subst, *op);
            }
            if let Operation::Phi { incoming } = &mut ni.op {
                for (_, v) in incoming.iter_mut() {
                    *v = chase(subst, *v);
                }
            }
            insts.push(ni);
        }

        block.instructions = insts;
    }
    new_func
}

/// Is `inst` a promoted alloca, or a load/store targeting a promoted slot (all
/// removed by promotion)?
fn is_removed(inst: &Instruction, slots: &BTreeMap<ValueId, SlotInfo>) -> bool {
    match &inst.op {
        Operation::Alloca { .. } => inst.dst.is_some_and(|d| slots.contains_key(&d)),
        Operation::Load => inst.operands.first().is_some_and(|p| slots.contains_key(p)),
        Operation::Store => inst.operands.get(1).is_some_and(|p| slots.contains_key(p)),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Deterministic fresh ids
// ---------------------------------------------------------------------------

/// Deterministic fresh SSA value id for the phi of `slot` at `block`.
fn fresh_phi_value(func: &AirFunction, block: BlockId, slot: ValueId) -> ValueId {
    ValueId::derive(&mix(
        b"saf-promote-phi-val",
        func.id.raw(),
        block.raw(),
        slot.raw(),
    ))
}

/// Deterministic fresh instruction id for the phi of `slot` at `block`.
fn fresh_phi_inst(func: &AirFunction, block: BlockId, slot: ValueId) -> InstId {
    InstId::derive(&mix(
        b"saf-promote-phi-inst",
        func.id.raw(),
        block.raw(),
        slot.raw(),
    ))
}

/// Deterministic undef sentinel value for `slot` (a load with no reaching def, or
/// a phi edge with no reaching def). It is defined nowhere and is not a constant,
/// so the ranker treats it as a free opaque leaf — sound.
fn undef_value(slot: ValueId) -> ValueId {
    ValueId::derive(&mix(b"saf-promote-undef", slot.raw(), 0, 0))
}

/// Mix a domain tag and three `u128`s into a byte buffer for deterministic id
/// derivation.
fn mix(tag: &[u8], a: u128, b: u128, c: u128) -> Vec<u8> {
    let mut v = Vec::with_capacity(tag.len() + 48);
    v.extend_from_slice(tag);
    v.extend_from_slice(&a.to_le_bytes());
    v.extend_from_slice(&b.to_le_bytes());
    v.extend_from_slice(&c.to_le_bytes());
    v
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use saf_core::air::{AirBlock, AirFunction, AirModule, AirParam, BinaryOp};
    use saf_core::id::make_id;
    use saf_core::ids::{FunctionId, ModuleId};

    fn vid(n: &str) -> ValueId {
        ValueId(make_id("value", n.as_bytes()))
    }
    fn bid(n: &str) -> BlockId {
        BlockId(make_id("block", n.as_bytes()))
    }
    fn iid(n: &str) -> InstId {
        InstId(make_id("inst", n.as_bytes()))
    }
    fn tid(n: &str) -> TypeId {
        TypeId(make_id("type", n.as_bytes()))
    }
    fn fid(n: &str) -> FunctionId {
        FunctionId(make_id("func", n.as_bytes()))
    }

    /// An instruction producing `dst` of type `ty`.
    fn vinst(
        id: &str,
        op: Operation,
        dst: ValueId,
        operands: Vec<ValueId>,
        ty: TypeId,
    ) -> Instruction {
        let mut i = Instruction::new(iid(id), op);
        i.operands = operands;
        i.dst = Some(dst);
        i.result_type = Some(ty);
        i
    }
    /// A void instruction (store / terminator) with no result.
    fn vinst_void(id: &str, op: Operation, operands: Vec<ValueId>) -> Instruction {
        let mut i = Instruction::new(iid(id), op);
        i.operands = operands;
        i
    }

    /// Build a `-O0`-style memory-backed `for (i = 0; i < 100; i++)` loop in `main`:
    /// the counter lives in an `alloca` read/written by `load`/`store`. When
    /// `escape` is set, the slot's address is also passed to a (defined) callee, so
    /// the escape gate must refuse promotion.
    fn mem_counter_loop(escape: bool) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });

        let mut constants = BTreeMap::new();
        let c0 = vid("c0");
        let c1 = vid("c1");
        let c100 = vid("c100");
        constants.insert(c0, Constant::Int { value: 0, bits: 32 });
        constants.insert(c1, Constant::Int { value: 1, bits: 32 });
        constants.insert(
            c100,
            Constant::Int {
                value: 100,
                bits: 32,
            },
        );

        let b0 = bid("entry");
        let h = bid("header");
        let l = bid("body");
        let e = bid("exit");

        let iaddr = vid("i.addr");
        let i0 = vid("i0");
        let cval = vid("c");
        let i1v = vid("i1");
        let i2 = vid("i2");

        // entry: %i.addr = alloca i32 ; [call foo(%i.addr)] ; store 0 ; br header
        let mut entry = AirBlock::new(b0);
        entry.instructions.push(vinst(
            "alloca",
            Operation::Alloca {
                size_bytes: Some(4),
            },
            iaddr,
            vec![],
            i32t,
        ));
        if escape {
            entry.instructions.push(vinst_void(
                "escape_call",
                Operation::CallDirect { callee: fid("foo") },
                vec![iaddr],
            ));
        }
        entry
            .instructions
            .push(vinst_void("store_init", Operation::Store, vec![c0, iaddr]));
        entry
            .instructions
            .push(vinst_void("br0", Operation::Br { target: h }, vec![]));

        // header: %i0 = load %i.addr ; %c = icmp slt %i0, 100 ; condbr
        let mut header = AirBlock::new(h);
        header
            .instructions
            .push(vinst("load_h", Operation::Load, i0, vec![iaddr], i32t));
        header.instructions.push(vinst(
            "cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            cval,
            vec![i0, c100],
            i1t,
        ));
        header.instructions.push(vinst_void(
            "condbr",
            Operation::CondBr {
                then_target: l,
                else_target: e,
            },
            vec![cval],
        ));

        // body: %i1 = load %i.addr ; %i2 = add %i1, 1 ; store %i2 ; br header
        let mut body = AirBlock::new(l);
        body.instructions
            .push(vinst("load_b", Operation::Load, i1v, vec![iaddr], i32t));
        body.instructions.push(vinst(
            "add",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            i2,
            vec![i1v, c1],
            i32t,
        ));
        body.instructions
            .push(vinst_void("store_i", Operation::Store, vec![i2, iaddr]));
        body.instructions
            .push(vinst_void("br1", Operation::Br { target: h }, vec![]));

        // exit: ret
        let mut exit = AirBlock::new(e);
        exit.instructions
            .push(vinst_void("ret", Operation::Ret, vec![]));

        let main = AirFunction {
            id: fid("main"),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![entry, header, body, exit],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut functions = vec![main];
        if escape {
            // A defined, loop-free `void foo(int*)` — keeps the (E)/(A) gates happy
            // so the only reason termination could fail is the un-promoted counter.
            let mut fblock = AirBlock::new(bid("foo_entry"));
            fblock
                .instructions
                .push(vinst_void("foo_ret", Operation::Ret, vec![]));
            functions.push(AirFunction {
                id: fid("foo"),
                name: "foo".to_string(),
                params: vec![AirParam {
                    id: vid("foo_p"),
                    name: None,
                    index: 0,
                    param_type: None,
                }],
                blocks: vec![fblock],
                entry_block: None,
                is_declaration: false,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            });
        }

        AirModule {
            id: ModuleId(make_id("module", b"t")),
            name: Some("t".to_string()),
            functions,
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants,
            types,
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    fn count_ops(f: &AirFunction, pred: impl Fn(&Operation) -> bool) -> usize {
        f.blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .filter(|i| pred(&i.op))
            .count()
    }

    #[test]
    fn promotes_memory_backed_counter_to_phi() {
        let m = mem_counter_loop(false);
        let main = &m.functions[0];
        let promoted = promote_function(main, &m).expect("counter slot should promote");

        // The alloca and all its loads/stores are gone.
        assert_eq!(
            count_ops(&promoted, |o| matches!(o, Operation::Alloca { .. })),
            0
        );
        assert_eq!(count_ops(&promoted, |o| matches!(o, Operation::Load)), 0);
        assert_eq!(count_ops(&promoted, |o| matches!(o, Operation::Store)), 0);
        // A phi was inserted at the loop header.
        let phis = count_ops(&promoted, |o| matches!(o, Operation::Phi { .. }));
        assert_eq!(phis, 1, "expected exactly one header phi");

        // The header phi's back-edge (body) incoming is the increment; its
        // preheader (entry) incoming is the initial constant.
        let header = promoted
            .blocks
            .iter()
            .find(|b| b.id == bid("header"))
            .unwrap();
        let phi = header
            .instructions
            .iter()
            .find_map(|i| match &i.op {
                Operation::Phi { incoming } => Some(incoming.clone()),
                _ => None,
            })
            .unwrap();
        assert!(incoming_has(&phi, bid("entry"), vid("c0")));
        assert!(incoming_has(&phi, bid("body"), vid("i2")));
    }

    fn incoming_has(inc: &[(BlockId, ValueId)], pred: BlockId, val: ValueId) -> bool {
        inc.iter().any(|&(p, v)| p == pred && v == val)
    }

    #[test]
    fn promoted_loop_is_ranked_end_to_end() {
        // The memory-backed loop is not ranked as-is (loads are opaque), but
        // promotion exposes the affine counter so the program proves terminating.
        let m = mem_counter_loop(false);

        // Without promotion the ranker abstains.
        let main = &m.functions[0];
        let cfg = Cfg::build(main);
        assert!(
            !crate::ranking::loops_are_ranked(main, &m, &cfg),
            "raw memory-backed loop must be opaque to the ranker (no phi)"
        );

        // With promotion wired into the termination proof, it succeeds.
        assert!(crate::termination::program_structurally_terminates(&m));
    }

    #[test]
    fn escaped_slot_is_not_promoted() {
        // MANDATORY negative: a slot whose address escapes (passed to a call) must
        // not promote, so the counter stays memory-backed and termination abstains.
        let m = mem_counter_loop(true);
        let main = &m.functions[0];
        assert!(
            promote_function(main, &m).is_none(),
            "an address-escaped slot must NOT promote"
        );
        // End-to-end: the proof soundly abstains (no wrong `true`).
        assert!(!crate::termination::program_structurally_terminates(&m));
    }

    #[test]
    fn straight_line_scalar_is_promoted_without_phi() {
        // int x = 5; x = x + 1; return x;  — one block, no phi needed.
        let i32t = tid("i32");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        let mut constants = BTreeMap::new();
        let c5 = vid("c5");
        let c1 = vid("c1s");
        constants.insert(c5, Constant::Int { value: 5, bits: 32 });
        constants.insert(c1, Constant::Int { value: 1, bits: 32 });

        let x = vid("x.addr");
        let l0 = vid("l0");
        let s1 = vid("s1");

        let mut blk = AirBlock::new(bid("sb_entry"));
        blk.instructions.push(vinst(
            "sb_alloca",
            Operation::Alloca {
                size_bytes: Some(4),
            },
            x,
            vec![],
            i32t,
        ));
        blk.instructions
            .push(vinst_void("sb_store0", Operation::Store, vec![c5, x]));
        blk.instructions
            .push(vinst("sb_load", Operation::Load, l0, vec![x], i32t));
        blk.instructions.push(vinst(
            "sb_add",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            s1,
            vec![l0, c1],
            i32t,
        ));
        blk.instructions
            .push(vinst_void("sb_store1", Operation::Store, vec![s1, x]));
        blk.instructions
            .push(vinst_void("sb_ret", Operation::Ret, vec![s1]));

        let f = AirFunction {
            id: fid("sb"),
            name: "sb".to_string(),
            params: Vec::new(),
            blocks: vec![blk],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let m = AirModule {
            id: ModuleId(make_id("module", b"sb")),
            name: None,
            functions: vec![f],
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants,
            types,
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };

        let promoted = promote_function(&m.functions[0], &m).expect("scalar should promote");
        assert_eq!(
            count_ops(&promoted, |o| matches!(
                o,
                Operation::Alloca { .. } | Operation::Load | Operation::Store
            )),
            0
        );
        assert_eq!(
            count_ops(&promoted, |o| matches!(o, Operation::Phi { .. })),
            0,
            "single block needs no phi"
        );
        // The `add` now reads the initial constant directly: add(c5, 1).
        let add = promoted.blocks[0]
            .instructions
            .iter()
            .find(|i| {
                matches!(
                    i.op,
                    Operation::BinaryOp {
                        kind: BinaryOp::Add
                    }
                )
            })
            .unwrap();
        assert_eq!(add.operands, vec![c5, c1]);
    }

    #[test]
    fn width_mismatched_store_blocks_promotion() {
        // A slot loaded as i32 but stored an i64 value must not promote (type pun).
        let i32t = tid("i32");
        let i64t = tid("i64");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i64t, AirType::Integer { bits: 64 });
        let mut constants = BTreeMap::new();
        let cbig = vid("cbig");
        constants.insert(cbig, Constant::Int { value: 7, bits: 64 });

        let x = vid("pun.addr");
        let l0 = vid("pun_l0");
        let mut blk = AirBlock::new(bid("pun_entry"));
        blk.instructions.push(vinst(
            "pun_alloca",
            Operation::Alloca {
                size_bytes: Some(4),
            },
            x,
            vec![],
            i32t,
        ));
        blk.instructions
            .push(vinst_void("pun_store", Operation::Store, vec![cbig, x]));
        blk.instructions
            .push(vinst("pun_load", Operation::Load, l0, vec![x], i32t));
        blk.instructions
            .push(vinst_void("pun_ret", Operation::Ret, vec![l0]));

        let f = AirFunction {
            id: fid("pun"),
            name: "pun".to_string(),
            params: Vec::new(),
            blocks: vec![blk],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let m = AirModule {
            id: ModuleId(make_id("module", b"pun")),
            name: None,
            functions: vec![f],
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants,
            types,
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };
        assert!(
            promote_function(&m.functions[0], &m).is_none(),
            "width-mismatched (type-punned) slot must not promote"
        );
    }

    #[test]
    fn no_alloca_function_is_unchanged() {
        // A function with no alloca returns `None` (no needless clone).
        let m = mem_counter_loop(false);
        // foo-less clean module's `main` has an alloca, so build a trivial fn.
        let mut blk = AirBlock::new(bid("triv_entry"));
        blk.instructions
            .push(vinst_void("triv_ret", Operation::Ret, vec![]));
        let f = AirFunction {
            id: fid("triv"),
            name: "triv".to_string(),
            params: Vec::new(),
            blocks: vec![blk],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        assert!(promote_function(&f, &m).is_none());
    }
}
