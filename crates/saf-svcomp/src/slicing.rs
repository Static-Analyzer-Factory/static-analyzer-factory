//! Backward AIR program slicing for the `unreach-call` FALSE pipeline.
//!
//! This module is **pure** (no I/O, no subprocess). It computes a backward
//! program slice from every `reach_error` criterion — following data
//! dependences through instruction operands and control dependences through the
//! branch conditions that gate the criteria — and extracts the *guard constants*
//! that a nondet input must match to steer execution toward the error.
//!
//! `__VERIFIER_assume` / `__CPROVER_assume` call sites are added as a SECONDARY
//! slicing criterion: a range guard such as `assume(x >= 5 && x <= 10)` does not
//! feed `reach_error` by data flow, so a naive error-only slice would drop it and
//! the fuzzer would waste its budget generating inputs the runtime `assume`
//! immediately rejects (a violation of contract **R4**). Keeping the assume
//! conditions in the slice surfaces their boundary constants for steering.
//!
//! # What the slice is used for (and why it stays SOUND)
//!
//! The slice never changes what confirms a verdict. Its two products —
//! [`slice_directed_dictionary`] (a fuzz dictionary that front-loads the guard
//! constants so they survive the size cap and are tried first) and
//! [`sequence_seeds`] (byte buffers that lay distinct guard constants at
//! consecutive nondet-read offsets, so a *chain* of distinct-valued guards is
//! satisfied in a single input that single-value tiling cannot build) — only
//! *steer the blind search*. The verdict is still produced by native replay on
//! the ORIGINAL, unsliced program (contract **R6**): a slice can legitimately
//! drop a constraint the real program enforces, so it must never be the thing
//! that runs. An empty slice yields a dictionary byte-identical to the plain
//! whole-module harvest, so wiring this in can only ever *add* reach, never
//! regress a previously-confirmed task or manufacture a wrong FALSE.

use crate::fuzz::{INPUT_LEN, MAX_DICT_ENTRIES, harvest_dictionary};
use crate::property::{ASSUME_FUNCTIONS, REACH_ERROR_NAMES};
use saf_core::air::{AirBlock, AirFunction, AirModule, BinaryOp, Constant, Operation};
use saf_core::ids::{BlockId, InstId, ValueId};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// The result of a backward slice from the `reach_error` criteria.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Slice {
    /// Instructions that (transitively) influence whether a criterion is
    /// reached, by data or control dependence. Deterministic ([`BTreeSet`]).
    pub instructions: BTreeSet<InstId>,
    /// Integer constants that appear in comparisons / switch cases controlling
    /// the criteria, ordered by data-flow proximity to the criteria (nearest
    /// first) and deduplicated. These are the values a nondet input must match
    /// to cross a guard on the path to the error.
    pub guard_constants: Vec<i64>,
}

/// A per-module index used during slicing: the defining instruction of every
/// SSA value, the operand list of every instruction, and the constant value of
/// every constant operand.
struct DefUse<'m> {
    /// `dst value -> defining instruction`.
    def_of: BTreeMap<ValueId, InstId>,
    /// `instruction id -> the instruction`.
    inst: BTreeMap<InstId, &'m saf_core::air::Instruction>,
}

impl<'m> DefUse<'m> {
    fn build(module: &'m AirModule) -> Self {
        let mut def_of = BTreeMap::new();
        let mut inst = BTreeMap::new();
        for func in &module.functions {
            for block in &func.blocks {
                for i in &block.instructions {
                    if let Some(dst) = i.dst {
                        def_of.insert(dst, i.id);
                    }
                    inst.insert(i.id, i);
                }
            }
        }
        Self { def_of, inst }
    }
}

/// Look up the integer value of a value id if it is a module constant.
fn const_int(module: &AirModule, v: ValueId) -> Option<i64> {
    match module.constants.get(&v) {
        Some(Constant::Int { value, .. }) => Some(*value),
        Some(Constant::BigInt { value, .. }) => value.parse::<i64>().ok(),
        _ => None,
    }
}

/// True for an integer-comparison binary op (the guards a nondet input crosses).
fn is_icmp(op: &Operation) -> Option<BinaryOp> {
    match op {
        Operation::BinaryOp { kind } => matches!(
            kind,
            BinaryOp::ICmpEq
                | BinaryOp::ICmpNe
                | BinaryOp::ICmpUgt
                | BinaryOp::ICmpUge
                | BinaryOp::ICmpUlt
                | BinaryOp::ICmpUle
                | BinaryOp::ICmpSgt
                | BinaryOp::ICmpSge
                | BinaryOp::ICmpSlt
                | BinaryOp::ICmpSle
        )
        .then_some(*kind),
        _ => None,
    }
}

/// Whether a comparison is strict (`<`, `>`, `!=`), so that the *crossing* value
/// is the constant plus/minus one rather than the constant itself.
fn is_strict(kind: BinaryOp) -> bool {
    matches!(
        kind,
        BinaryOp::ICmpNe
            | BinaryOp::ICmpUgt
            | BinaryOp::ICmpUlt
            | BinaryOp::ICmpSgt
            | BinaryOp::ICmpSlt
    )
}

/// Compute a backward slice from every `reach_error` criterion (with
/// `__VERIFIER_assume` sites as a secondary criterion). See the module docs.
///
/// The traversal is a whole-module backward reachability over the SSA def-use
/// graph: starting from the operands of the criteria, it pulls in each defining
/// instruction and enqueues that instruction's operands, and it treats every
/// conditional branch / switch as control-relevant (its condition is enqueued).
/// This is an *over-approximation* of the true PDG slice — deliberately so, since
/// the slice only steers input generation and over-inclusion merely surfaces a
/// few extra (harmless) steering constants. Guard constants are collected in
/// breadth-first (nearest-the-error-first) order.
#[must_use]
pub fn backward_slice(module: &AirModule) -> Slice {
    let du = DefUse::build(module);

    // Criteria: reach_error call sites (primary) + assume call sites (secondary).
    let mut criteria: Vec<InstId> = Vec::new();
    for func in &module.functions {
        for block in &func.blocks {
            for i in &block.instructions {
                if let Operation::CallDirect { callee } = &i.op {
                    if let Some(t) = module.function(*callee) {
                        if REACH_ERROR_NAMES.contains(&t.name.as_str())
                            || ASSUME_FUNCTIONS.contains(&t.name.as_str())
                        {
                            criteria.push(i.id);
                        }
                    }
                }
            }
        }
    }

    // Seed the worklist with the criteria's operands, plus every conditional
    // terminator's condition (control dependence, over-approximated).
    let mut queue: VecDeque<ValueId> = VecDeque::new();
    let mut seen_val: BTreeSet<ValueId> = BTreeSet::new();
    let push = |q: &mut VecDeque<ValueId>, s: &mut BTreeSet<ValueId>, v: ValueId| {
        if s.insert(v) {
            q.push_back(v);
        }
    };

    for c in &criteria {
        if let Some(i) = du.inst.get(c) {
            for &op in &i.operands {
                push(&mut queue, &mut seen_val, op);
            }
        }
    }
    for func in &module.functions {
        for block in &func.blocks {
            for i in &block.instructions {
                match &i.op {
                    Operation::CondBr { .. } | Operation::Switch { .. } => {
                        for &op in &i.operands {
                            push(&mut queue, &mut seen_val, op);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut slice = Slice::default();
    let mut guard_seen: BTreeSet<i64> = BTreeSet::new();
    let record_guard = |slice: &mut Slice, seen: &mut BTreeSet<i64>, v: i64| {
        if seen.insert(v) {
            slice.guard_constants.push(v);
        }
    };

    // Backward closure over data dependences.
    while let Some(val) = queue.pop_front() {
        let Some(&def) = du.def_of.get(&val) else {
            continue;
        };
        slice.instructions.insert(def);
        let Some(inst) = du.inst.get(&def) else {
            continue;
        };
        // Harvest guard constants from EVERY control-relevant instruction, not
        // just the comparison itself. An opaque guard `if ((x ^ C) == 0)` compares
        // against 0 — the load-bearing constant `C` is an operand of the XOR that
        // feeds the comparison, so restricting to icmp operands would miss it. The
        // BFS visits the comparison before its arithmetic operands, so constants
        // are recorded roughly nearest-the-error first.
        let icmp_kind = is_icmp(&inst.op);
        for &op in &inst.operands {
            if let Some(k) = const_int(module, op) {
                record_guard(&mut slice, &mut guard_seen, k);
                // For a STRICT comparison the value that crosses the guard is one
                // past the constant; add both directions (cheap, deduped).
                if let Some(kind) = icmp_kind {
                    if is_strict(kind) {
                        if let Some(up) = k.checked_add(1) {
                            record_guard(&mut slice, &mut guard_seen, up);
                        }
                        if let Some(dn) = k.checked_sub(1) {
                            record_guard(&mut slice, &mut guard_seen, dn);
                        }
                    }
                }
            }
            push(&mut queue, &mut seen_val, op);
        }
    }

    // Harvest switch case labels as guard constants. Switch discriminants were
    // already enqueued as control-relevant above; the case values are the
    // constants an input must match to take a case edge toward the error.
    // Over-inclusion is harmless (steering only), matching the CondBr treatment.
    for func in &module.functions {
        for block in &func.blocks {
            for i in &block.instructions {
                if let Operation::Switch { cases, .. } = &i.op {
                    for (case_val, _) in cases {
                        record_guard(&mut slice, &mut guard_seen, *case_val);
                    }
                }
            }
        }
    }

    slice
}

/// Successor blocks of `block`'s terminator (empty for `ret` / `unreachable` /
/// no terminator). Mirrors the CFG edges an execution can follow out of `block`.
fn block_successors(block: &AirBlock) -> Vec<BlockId> {
    match block.terminator().map(|t| &t.op) {
        Some(Operation::Br { target }) => vec![*target],
        Some(Operation::CondBr {
            then_target,
            else_target,
        }) => vec![*then_target, *else_target],
        Some(Operation::Switch { default, cases }) => {
            let mut out = vec![*default];
            out.extend(cases.iter().map(|(_, t)| *t));
            out
        }
        _ => Vec::new(),
    }
}

/// The **coarse abort-prune** set: blocks of `func` from which a direct
/// `reach_error` / `__VERIFIER_error` call site is reachable along CFG successor
/// edges (back edges included). This is the block-level backward slice from the
/// `unreach-call` criteria — the executable-slice's "what can still reach the
/// error" projection.
///
/// # Why an engine may soundly ABORT any path leaving this set
///
/// If a block `B` is not in the returned set, then no CFG path `B → … → error`
/// exists, so *no* execution passing through `B` can reach the error — the path
/// is dead for `unreach-call`. Conversely, every block that lies on a real path
/// to the error is, by construction, in the set (it reaches the error along that
/// very path). Therefore dropping a successor outside the set removes only dead
/// search and can never discard a genuine witness path. A symbolic/BMC engine
/// bounded by a step/state/solver budget can spend that budget on error-relevant
/// paths instead, finishing on programs whose error-irrelevant branching would
/// otherwise exhaust it — a pure scalability multiplier, never a soundness or
/// recall change (dead subtrees only ever spawn dead subtrees).
///
/// Intraprocedural and consistent with the SE lever's error detection (both key
/// on [`REACH_ERROR_NAMES`] direct calls); an error inside a callee is invisible
/// here, matching the caller's own direct-call gate.
#[must_use]
pub fn error_reaching_blocks(func: &AirFunction, module: &AirModule) -> BTreeSet<BlockId> {
    let mut preds: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
    let mut queue: VecDeque<BlockId> = VecDeque::new();
    let mut reaching: BTreeSet<BlockId> = BTreeSet::new();

    for block in &func.blocks {
        for succ in block_successors(block) {
            preds.entry(succ).or_default().push(block.id);
        }
        let has_error = block.instructions.iter().any(|i| {
            matches!(&i.op, Operation::CallDirect { callee }
                if module
                    .function(*callee)
                    .is_some_and(|t| REACH_ERROR_NAMES.contains(&t.name.as_str())))
        });
        if has_error && reaching.insert(block.id) {
            queue.push_back(block.id);
        }
    }

    while let Some(b) = queue.pop_front() {
        if let Some(ps) = preds.get(&b) {
            for &p in ps {
                if reaching.insert(p) {
                    queue.push_back(p);
                }
            }
        }
    }
    reaching
}

/// Build a fuzz dictionary that FRONT-LOADS the slice's guard constants (so they
/// survive the [`MAX_DICT_ENTRIES`] cap and are tried before generic constants),
/// then appends the plain whole-module harvest for everything else. Deterministic
/// and deduplicated. When the slice has no guard constants this is byte-identical
/// (as a set) to [`harvest_dictionary`], so it never regresses.
#[must_use]
pub fn slice_directed_dictionary(module: &AirModule, slice: &Slice) -> Vec<i64> {
    let mut out: Vec<i64> = Vec::with_capacity(MAX_DICT_ENTRIES);
    let mut seen: BTreeSet<i64> = BTreeSet::new();
    for &g in &slice.guard_constants {
        if out.len() >= MAX_DICT_ENTRIES {
            break;
        }
        if seen.insert(g) {
            out.push(g);
        }
    }
    for v in harvest_dictionary(module) {
        if out.len() >= MAX_DICT_ENTRIES {
            break;
        }
        if seen.insert(v) {
            out.push(v);
        }
    }
    out
}

/// Cap on how many guard constants a single sequence seed lays out (bounds the
/// buffer's stride depth; deeper guard chains than this fall back to mutation).
const MAX_SEQUENCE_GUARDS: usize = 8;

/// Cap on the number of sequence seeds produced (keeps the extra native runs
/// bounded within the fuzz time budget).
const MAX_SEQUENCE_SEEDS: usize = 16;

/// Lay `guard_constants` at consecutive little-endian offsets of a fresh
/// [`INPUT_LEN`] buffer, at stride `width`, starting at byte `start`.
fn layout(order: &[i64], width: usize, start: usize) -> Vec<u8> {
    let mut buf = vec![0u8; INPUT_LEN];
    let mut off = start;
    for &g in order {
        if off + width > INPUT_LEN {
            break;
        }
        #[allow(clippy::cast_sign_loss)]
        let bits = g as u64;
        for i in 0..width {
            #[allow(clippy::cast_possible_truncation)]
            let b = (bits >> (8 * i)) as u8;
            buf[off + i] = b;
        }
        off += width;
    }
    buf
}

/// Build seed buffers that place a *chain* of distinct guard constants at
/// consecutive nondet-read offsets, so a multi-guard reach
/// (`x=nondet(); if(g(x)){ y=nondet(); if(h(y)) reach_error(); }`) is satisfied
/// by one input. Single-value seed tiling ([`crate::fuzz::seed_corpus`]) cannot
/// build such an input (one value everywhere ⇒ every read sees the same value);
/// the mutation loop can only stumble on the joint assignment slowly.
///
/// The static slice does not tell us which read consumes which constant, nor the
/// exact read-order-to-byte-offset mapping (it depends on each read's width), so
/// we emit a small deterministic spread of layouts:
/// - every ORDERED PAIR of distinct guard constants at the first two 4-byte read
///   slots — this directly covers the dominant two-guard `int` chain regardless of
///   which constant the first read must match;
/// - the full guard sequence laid forward and reversed at 4- and 8-byte strides —
///   for longer chains and wider reads.
///
/// `0` is dropped from the layout constants: laying zero into an already-zero
/// buffer is a no-op that only misaligns a chain. Native replay on the ORIGINAL
/// program is still the sole arbiter (R6), so an ill-fitting layout simply fails
/// to reach and is discarded — this can only add reach, never a wrong FALSE.
#[must_use]
pub fn sequence_seeds(guard_constants: &[i64]) -> Vec<Vec<u8>> {
    // Distinct nonzero guard constants, order-preserving (nearest-error first).
    let mut nz: Vec<i64> = Vec::new();
    let mut seen_c: BTreeSet<i64> = BTreeSet::new();
    for &g in guard_constants {
        if g != 0 && seen_c.insert(g) {
            nz.push(g);
        }
    }
    // A chain needs at least two distinct guards to be worth a bespoke layout;
    // single guards are already covered by the tiled seed corpus.
    if nz.len() < 2 {
        return Vec::new();
    }
    nz.truncate(MAX_SEQUENCE_GUARDS);

    let mut seeds: Vec<Vec<u8>> = Vec::new();
    let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
    let add = |seeds: &mut Vec<Vec<u8>>, seen: &mut BTreeSet<Vec<u8>>, buf: Vec<u8>| {
        if seeds.len() < MAX_SEQUENCE_SEEDS && seen.insert(buf.clone()) {
            seeds.push(buf);
        }
    };

    // Ordered pairs at the first two 4-byte slots (the two-guard `int` chain).
    for i in 0..nz.len() {
        for j in 0..nz.len() {
            if i == j {
                continue;
            }
            add(&mut seeds, &mut seen, layout(&[nz[i], nz[j]], 4, 0));
        }
    }

    // Full-sequence layouts for longer chains / wider reads.
    let forward = nz.clone();
    let mut reversed = forward.clone();
    reversed.reverse();
    for order in [&forward, &reversed] {
        for width in [4usize, 8] {
            add(&mut seeds, &mut seen, layout(order, width, 0));
        }
    }
    seeds
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};
    use std::collections::BTreeMap;

    fn decl(id: u128, name: &str) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    /// `main` computes `cmp = (x == 500)` then `reach_error()`; slice must pick
    /// up the guard constant 500.
    #[test]
    fn slice_harvests_the_guard_constant() {
        let reach_id = FunctionId::new(1);
        let x = ValueId::new(10);
        let five_hundred = ValueId::new(11);
        let cmp = ValueId::new(12);

        // cmp = icmp eq x, 500
        let icmp = Instruction {
            id: InstId::new(100),
            op: Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            operands: vec![x, five_hundred],
            dst: Some(cmp),
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };
        // br cmp, then, else  (condition references cmp)
        let condbr = Instruction {
            id: InstId::new(101),
            op: Operation::CondBr {
                then_target: BlockId::new(21),
                else_target: BlockId::new(22),
            },
            operands: vec![cmp],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };
        // call reach_error()
        let call = Instruction {
            id: InstId::new(102),
            op: Operation::CallDirect { callee: reach_id },
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };
        let mut block = AirBlock::new(BlockId::new(20));
        block.instructions = vec![icmp, condbr, call];

        let main = AirFunction {
            id: FunctionId::new(2),
            name: "main".to_string(),
            params: vec![],
            blocks: vec![block],
            entry_block: Some(BlockId::new(20)),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(decl(1, "reach_error"));
        m.functions.push(main);
        m.constants.insert(x, Constant::int(0, 32)); // x placeholder (not const-int guard)
        m.constants.insert(five_hundred, Constant::int(500, 32));

        let slice = backward_slice(&m);
        assert!(
            slice.guard_constants.contains(&500),
            "guard constant 500 must be harvested, got {:?}",
            slice.guard_constants
        );
        assert!(
            slice.instructions.contains(&InstId::new(100)),
            "icmp sliced"
        );
    }

    /// CFG: bb0 -CondBr-> bb1(reach_error) / bb2 -Br-> bb3(ret). Only bb0 and bb1
    /// can reach the error; bb2/bb3 are a dead subtree that a solver may abort.
    #[test]
    fn error_reaching_blocks_prunes_dead_subtree() {
        let reach_id = FunctionId::new(1);
        let (bb0, bb1, bb2, bb3) = (
            BlockId::new(20),
            BlockId::new(21),
            BlockId::new(22),
            BlockId::new(23),
        );

        let term = |id: u128, op: Operation| Instruction {
            id: InstId::new(id),
            op,
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };

        let mut b0 = AirBlock::new(bb0);
        b0.instructions = vec![Instruction {
            id: InstId::new(100),
            op: Operation::CondBr {
                then_target: bb1,
                else_target: bb2,
            },
            operands: vec![ValueId::new(9)],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }];
        let mut b1 = AirBlock::new(bb1);
        b1.instructions = vec![
            Instruction {
                id: InstId::new(101),
                op: Operation::CallDirect { callee: reach_id },
                operands: vec![],
                dst: None,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            },
            term(102, Operation::Ret),
        ];
        let mut b2 = AirBlock::new(bb2);
        b2.instructions = vec![term(103, Operation::Br { target: bb3 })];
        let mut b3 = AirBlock::new(bb3);
        b3.instructions = vec![term(104, Operation::Ret)];

        let func = AirFunction {
            id: FunctionId::new(2),
            name: "main".to_string(),
            params: vec![],
            blocks: vec![b0, b1, b2, b3],
            entry_block: Some(bb0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(decl(1, "reach_error"));
        m.functions.push(func.clone());

        let reaching = error_reaching_blocks(&func, &m);
        assert!(reaching.contains(&bb0), "entry reaches error");
        assert!(reaching.contains(&bb1), "error block reaches itself");
        assert!(!reaching.contains(&bb2), "dead subtree pruned");
        assert!(!reaching.contains(&bb3), "dead subtree pruned");
    }

    /// A back edge into the error path must keep the loop body in the reaching set.
    #[test]
    fn error_reaching_blocks_follows_back_edges() {
        let reach_id = FunctionId::new(1);
        let (bb0, bb1) = (BlockId::new(30), BlockId::new(31));
        // bb0 -Br-> bb1 ; bb1 -CondBr-> bb0 (back edge) / bb1-self is error.
        let mut b0 = AirBlock::new(bb0);
        b0.instructions = vec![Instruction {
            id: InstId::new(200),
            op: Operation::Br { target: bb1 },
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }];
        let mut b1 = AirBlock::new(bb1);
        b1.instructions = vec![
            Instruction {
                id: InstId::new(201),
                op: Operation::CallDirect { callee: reach_id },
                operands: vec![],
                dst: None,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            },
            Instruction {
                id: InstId::new(202),
                op: Operation::CondBr {
                    then_target: bb0,
                    else_target: bb1,
                },
                operands: vec![ValueId::new(9)],
                dst: None,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            },
        ];
        let func = AirFunction {
            id: FunctionId::new(2),
            name: "main".to_string(),
            params: vec![],
            blocks: vec![b0, b1],
            entry_block: Some(bb0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(decl(1, "reach_error"));
        m.functions.push(func.clone());
        let reaching = error_reaching_blocks(&func, &m);
        assert!(
            reaching.contains(&bb0),
            "predecessor of error via back edge"
        );
        assert!(reaching.contains(&bb1), "error block");
    }

    #[test]
    fn dictionary_front_loads_guard_constants() {
        let mut m = AirModule::new(ModuleId::new(1));
        // A generic constant that would otherwise crowd the dictionary.
        m.constants.insert(ValueId::new(1), Constant::int(4242, 32));
        let slice = Slice {
            instructions: BTreeSet::new(),
            guard_constants: vec![777, 888],
        };
        let dict = slice_directed_dictionary(&m, &slice);
        assert_eq!(dict[0], 777, "guard constant first");
        assert_eq!(dict[1], 888, "guard constant second");
        assert!(dict.contains(&4242), "module constant still present");
        // No duplicates.
        let mut sorted = dict.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), dict.len(), "dictionary is deduplicated");
    }

    #[test]
    fn empty_slice_dictionary_matches_plain_harvest_as_a_set() {
        let mut m = AirModule::new(ModuleId::new(1));
        m.constants.insert(ValueId::new(1), Constant::int(4242, 32));
        let empty = Slice::default();
        let a: BTreeSet<i64> = slice_directed_dictionary(&m, &empty).into_iter().collect();
        let b: BTreeSet<i64> = harvest_dictionary(&m).into_iter().collect();
        assert_eq!(a, b, "empty slice must not change the dictionary content");
    }

    #[test]
    fn sequence_seeds_lay_the_chain_at_consecutive_offsets() {
        let seeds = sequence_seeds(&[5, 17]);
        assert!(!seeds.is_empty());
        // The forward 4-byte layout puts 5 at offset 0 and 17 at offset 4.
        let want = {
            let mut b = vec![0u8; INPUT_LEN];
            b[0..4].copy_from_slice(&5u32.to_le_bytes());
            b[4..8].copy_from_slice(&17u32.to_le_bytes());
            b
        };
        assert!(
            seeds.contains(&want),
            "a forward 4-byte chain layout must be present"
        );
        // Reversed order is also produced (17 at offset 0, 5 at offset 4).
        let want_rev = {
            let mut b = vec![0u8; INPUT_LEN];
            b[0..4].copy_from_slice(&17u32.to_le_bytes());
            b[4..8].copy_from_slice(&5u32.to_le_bytes());
            b
        };
        assert!(seeds.contains(&want_rev), "reversed layout present");
    }

    #[test]
    fn sequence_seeds_need_a_chain() {
        assert!(sequence_seeds(&[]).is_empty());
        assert!(
            sequence_seeds(&[42]).is_empty(),
            "a single guard is not a chain"
        );
    }

    #[test]
    fn sequence_seeds_are_bounded_and_deterministic() {
        let guards: Vec<i64> = (1..=32).collect();
        let a = sequence_seeds(&guards);
        let b = sequence_seeds(&guards);
        assert_eq!(a, b, "deterministic");
        assert!(a.len() <= MAX_SEQUENCE_SEEDS, "seed count bounded");
        assert!(a.iter().all(|s| s.len() == INPUT_LEN));
    }
}
