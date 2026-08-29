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
//! A refinement on top of the raw guard set is the **input→error chop**
//! ([`Slice::input_guard_constants`]): a forward data-dependence taint from the
//! scalar `__VERIFIER_nondet_*` sources, intersected with the backward error slice,
//! isolates exactly the constants a nondet INPUT can be steered to match (the input
//! *alphabet*, `if (input == 5)`). On a large state machine this is a small, high-
//! value subset of the guard constants — the rest compare against internal state no
//! input can directly satisfy — so front-loading the chop keeps the reachable
//! alphabet ahead of the dictionary cap and gives the sequence-seed generator the
//! true event alphabet to lay as a chain.
//!
//! The slice never changes what confirms a verdict. Its products —
//! [`slice_directed_dictionary`] (a fuzz dictionary that front-loads the chop then
//! the guard constants so they survive the size cap and are tried first) and
//! [`sequence_seeds`] (byte buffers that lay distinct guard constants at
//! consecutive nondet-read offsets, so a *chain* of distinct-valued guards is
//! satisfied in a single input that single-value tiling cannot build) — only
//! *steer the blind search*. The verdict is still produced by native replay on
//! the ORIGINAL, unsliced program (contract **R6**): a slice can legitimately
//! drop a constraint the real program enforces, so it must never be the thing
//! that runs. An empty slice yields a dictionary byte-identical to the plain
//! whole-module harvest, so wiring this in can only ever *add* reach, never
//! regress a previously-confirmed task or manufacture a wrong FALSE.

use crate::fuzz::{INPUT_LEN, MAX_DICT_ENTRIES, XorShift64, harvest_dictionary};
use crate::property::{ASSUME_FUNCTIONS, REACH_ERROR_NAMES};
use saf_core::air::{AirBlock, AirFunction, AirModule, BinaryOp, Constant, Operation};
use saf_core::ids::{BlockId, InstId, ValueId};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// The result of a backward slice from the `reach_error` criteria.
// `guard_floats: Vec<f64>` precludes `Eq` (f64 is only `PartialEq`); the slice is
// compared structurally only in tests, where `PartialEq` suffices.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Slice {
    /// Instructions that (transitively) influence whether a criterion is
    /// reached, by data or control dependence. Deterministic ([`BTreeSet`]).
    pub instructions: BTreeSet<InstId>,
    /// Integer constants that appear in comparisons / switch cases controlling
    /// the criteria, ordered by data-flow proximity to the criteria (nearest
    /// first) and deduplicated. These are the values a nondet input must match
    /// to cross a guard on the path to the error.
    pub guard_constants: Vec<i64>,
    /// Floating-point constants that appear in comparisons / arithmetic
    /// controlling the criteria (the `-0.8`, `0.1` boundaries of a
    /// `assume(x > -0.8 && x < 0.8); if (!(g(x) < 0.1)) reach_error();` guard),
    /// ordered nearest-the-error first and deduplicated by IEEE-754 bit pattern.
    /// These steer the FLOAT byte-stream fuzzer toward the narrow in-range inputs
    /// a blind random float almost never lands on (a random `u32`/`u64`
    /// reinterpreted as a float is overwhelmingly a huge magnitude / NaN / inf).
    pub guard_floats: Vec<f64>,
    /// The **input→error chop**: integer constants compared against a value that
    /// is (transitively) data-dependent on a scalar `__VERIFIER_nondet_*` read AND
    /// whose comparison is error-relevant (in [`Slice::instructions`]). These are
    /// the constants a nondet INPUT can actually be steered to match — the input
    /// *alphabet* of the program (`if (input == 5)`, `switch (nondet()) { case 7: }`).
    ///
    /// A [`Slice::guard_constants`] entry is any constant on any error-relevant
    /// comparison, which on a large state machine mixes the controllable input
    /// alphabet in with hundreds of constants compared against internal state
    /// (`if (a29 == 5)`) that no input can directly satisfy. Front-loading the chop
    /// puts the values the fuzzer can actually reach a guard with first (surviving
    /// the dictionary cap) and gives the sequence-seed generator the true input
    /// alphabet to lay as a chain. Ordered by first appearance in a deterministic
    /// module walk and deduplicated. Empty when the program has no scalar nondet
    /// source, so wiring it in never changes an input-free program's search.
    pub input_guard_constants: Vec<i64>,
    /// The **input event alphabet**: the bare set of constants an input-tainted
    /// value is directly *equality*-compared against (`input == 5`, `input != 3`)
    /// or matched by a `switch (input)` case — WITHOUT the ±1 strict-crossing
    /// expansion that [`Slice::input_guard_constants`] carries. For an event-driven
    /// state machine (RERS/`eca-*`) whose driver validates every read against a
    /// fixed symbol set (`if (input != 1 && … && input != 7) return;`) this is
    /// exactly the set of *valid event codes*: any other value halts the driver on
    /// the first read, so a blind byte fuzzer — which essentially never produces a
    /// 4-byte value inside `{1..7}` — cannot walk the automaton at all. Saturating
    /// the whole input buffer with symbols drawn from THIS set (see
    /// [`alphabet_walk_seeds`]) gives the fuzzer long *valid* event sequences to
    /// mutate. Ordered by first appearance and deduplicated; empty when the program
    /// has no scalar nondet source, so it never perturbs an input-free search.
    pub input_alphabet: Vec<i64>,
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

/// Look up the floating-point value of a value id if it is a module float
/// constant (stored as `f64`; lossless for `f32` sources — see [`Constant::Float`]).
fn const_float(module: &AirModule, v: ValueId) -> Option<f64> {
    match module.constants.get(&v) {
        Some(Constant::Float { value, .. }) => Some(*value),
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

/// Whether `name` is a scalar-INTEGER `__VERIFIER_nondet_*` read — a taint source
/// for the forward input slice. Float / double / pointer nondet are excluded: their
/// comparison constants are steered by the dedicated float pipeline / pointer shim,
/// not the integer dictionary, so mixing them into the integer chop would be noise.
fn is_int_nondet(name: &str) -> bool {
    name.starts_with("__VERIFIER_nondet_")
        && !name.ends_with("_float")
        && !name.ends_with("_double")
        && !name.ends_with("_pointer")
}

/// Forward data-dependence taint from every scalar-integer `__VERIFIER_nondet_*`
/// result, over the SSA def-use graph. A value is tainted if it is such a result
/// or is produced by a value op (arithmetic / cast / copy / phi / select / gep …)
/// that uses a tainted operand. Propagation STOPS at comparisons (`icmp`): an
/// `icmp`'s result is a boolean condition, not an input value whose alphabet we are
/// tracking. Deterministic ([`BTreeSet`] / [`VecDeque`] worklist).
///
/// This is the *forward* half of the input→error chop; intersecting the tainted set
/// with the backward error slice isolates the constants an input can be steered to
/// match. Being an over-approximation is harmless — the chop only orders steering
/// constants and native replay on the ORIGINAL program stays the sole arbiter (R6).
fn nondet_tainted_values(module: &AirModule, du: &DefUse) -> BTreeSet<ValueId> {
    // Forward use-map: value -> instructions that read it as an operand.
    let mut uses: BTreeMap<ValueId, Vec<InstId>> = BTreeMap::new();
    for func in &module.functions {
        for block in &func.blocks {
            for i in &block.instructions {
                for &op in &i.operands {
                    uses.entry(op).or_default().push(i.id);
                }
            }
        }
    }

    let mut tainted: BTreeSet<ValueId> = BTreeSet::new();
    let mut queue: VecDeque<ValueId> = VecDeque::new();
    // Seeds: dsts of scalar-integer nondet calls.
    for func in &module.functions {
        for block in &func.blocks {
            for i in &block.instructions {
                if let Operation::CallDirect { callee } = &i.op {
                    if let (Some(dst), Some(t)) = (i.dst, module.function(*callee)) {
                        if is_int_nondet(&t.name) && tainted.insert(dst) {
                            queue.push_back(dst);
                        }
                    }
                }
            }
        }
    }

    while let Some(v) = queue.pop_front() {
        let Some(consumers) = uses.get(&v) else {
            continue;
        };
        for &uid in consumers {
            let Some(inst) = du.inst.get(&uid) else {
                continue;
            };
            // Stop at comparisons — the taint we care about is the input VALUE, not
            // the boolean it feeds into.
            if is_icmp(&inst.op).is_some() {
                continue;
            }
            if let Some(dst) = inst.dst {
                if tainted.insert(dst) {
                    queue.push_back(dst);
                }
            }
        }
    }
    tainted
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
    // Float guards are deduplicated on the IEEE-754 bit pattern (`f64` is not
    // `Ord`/`Eq`-hashable; two syntactically distinct writes of the same value
    // share a pattern and must collapse to one steering constant).
    let mut float_seen: BTreeSet<u64> = BTreeSet::new();
    let record_float = |slice: &mut Slice, seen: &mut BTreeSet<u64>, v: f64| {
        if seen.insert(v.to_bits()) {
            slice.guard_floats.push(v);
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
            if let Some(f) = const_float(module, op) {
                record_float(&mut slice, &mut float_seen, f);
            }
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

    // Input→error chop: the constants an actual nondet INPUT can be steered to
    // match (see [`harvest_input_chop`]).
    harvest_input_chop(module, &du, &mut slice);

    slice
}

/// Populate [`Slice::input_guard_constants`] — the input→error chop. Forward-taint
/// from the scalar-integer nondet sources, then harvest the constant operand of
/// every error-relevant comparison whose OTHER operand is tainted (and every case
/// value of a switch on a tainted discriminant). These front-load the dictionary
/// and drive the sequence-seed chain ahead of the non-input state constants that
/// dominate a large automaton's guard set. No-op when the program has no scalar
/// nondet source, so the refinement never perturbs an input-free search.
fn harvest_input_chop(module: &AirModule, du: &DefUse, slice: &mut Slice) {
    let tainted = nondet_tainted_values(module, du);
    if tainted.is_empty() {
        return;
    }
    let mut input_seen: BTreeSet<i64> = BTreeSet::new();
    let mut record_input = |slice: &mut Slice, v: i64| {
        if input_seen.insert(v) {
            slice.input_guard_constants.push(v);
        }
    };
    // The bare event alphabet (no ±1 expansion): a slot value that halts the driver
    // must never appear in a saturation seed, so the ±1 neighbours are excluded here.
    let mut alpha_seen: BTreeSet<i64> = BTreeSet::new();
    let mut record_alpha = |slice: &mut Slice, v: i64| {
        if alpha_seen.insert(v) {
            slice.input_alphabet.push(v);
        }
    };
    for func in &module.functions {
        for block in &func.blocks {
            for i in &block.instructions {
                match &i.op {
                    // A comparison against a tainted operand: the constant is the
                    // value the input must reach. Gate on error-relevance (in the
                    // backward slice) so dead-code comparisons don't pollute the
                    // alphabet.
                    Operation::BinaryOp { .. }
                        if is_icmp(&i.op).is_some()
                            && slice.instructions.contains(&i.id)
                            && i.operands.iter().any(|op| tainted.contains(op)) =>
                    {
                        let kind = is_icmp(&i.op);
                        // Equality/disequality comparisons pin the input to (or away
                        // from) an EXACT event code — the bare constant is a valid
                        // alphabet symbol. Ordering comparisons (`<`, `>`) do not name
                        // a single symbol, so they seed only the steering set.
                        let is_eq = matches!(kind, Some(BinaryOp::ICmpEq | BinaryOp::ICmpNe));
                        for &op in &i.operands {
                            let Some(k) = const_int(module, op) else {
                                continue;
                            };
                            record_input(slice, k);
                            if is_eq {
                                record_alpha(slice, k);
                            }
                            if kind.is_some_and(is_strict) {
                                if let Some(up) = k.checked_add(1) {
                                    record_input(slice, up);
                                }
                                if let Some(dn) = k.checked_sub(1) {
                                    record_input(slice, dn);
                                }
                            }
                        }
                    }
                    // A switch on a tainted discriminant: each case value is a
                    // controllable input target and a valid alphabet symbol.
                    Operation::Switch { cases, .. }
                        if i.operands.iter().any(|op| tainted.contains(op)) =>
                    {
                        for (case_val, _) in cases {
                            record_input(slice, *case_val);
                            record_alpha(slice, *case_val);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
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
    // The input→error chop first: the values an input can actually be steered to
    // match survive the cap and are tried before the (much larger) full guard set.
    for &g in &slice.input_guard_constants {
        if out.len() >= MAX_DICT_ENTRIES {
            break;
        }
        if seen.insert(g) {
            out.push(g);
        }
    }
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

/// Cap on the number of alphabet-saturation walk seeds produced (bounds the extra
/// native runs added at the start of the fuzz budget).
const MAX_WALK_SEEDS: usize = 24;

/// Number of distinct deterministic pseudo-random walks generated over the event
/// alphabet (each at the two most common read widths).
const NUM_RANDOM_WALKS: usize = 10;

/// Build **input-alphabet saturation seeds** for an input-driven state machine.
///
/// `alphabet` is the [`Slice::input_alphabet`] — the set of valid event codes a
/// nondet input is equality-compared against. An event-driven driver (RERS /
/// `eca-*`) reads one input per loop iteration and *halts on the first out-of-
/// alphabet value* (`if (input != 1 && … && input != 7) return;`), so the automaton
/// is only walked by a stream of valid symbols. A blind byte fuzzer essentially
/// never produces a 4-byte value inside a 7-element set, so it stalls at the first
/// read; the tiled single-value seeds ([`crate::fuzz::seed_corpus`]) only ever feed
/// ONE symbol at every read (a constant walk), and [`sequence_seeds`] lays a short
/// distinct-valued prefix then zeros (an out-of-alphabet value that halts the
/// driver). Neither can build a LONG *varied* valid sequence.
///
/// This fills the WHOLE [`INPUT_LEN`] buffer with symbols drawn from the alphabet,
/// at the common integer read widths, in two families:
/// - **cyclic tilings** (`1,2,…,k,1,2,…`) at 1-, 2-, and 4-byte strides — a
///   deterministic round-robin walk that covers every symbol evenly;
/// - **deterministic pseudo-random walks** (fixed-seed [`XorShift64`]) that assign
///   an independent alphabet symbol to every read slot — a diverse population of
///   valid event sequences for the mutator to refine (its dictionary-window
///   operator, front-loaded with the same alphabet, keeps mutants in-alphabet).
///
/// Returns empty when the alphabet has fewer than two distinct symbols (a single
/// symbol is already covered by constant tiling) — so an input-free or single-guard
/// program gets no extra seeds. Purely steers the blind search; native replay on the
/// ORIGINAL program stays the sole FALSE arbiter (**R6**), so a seed that fails to
/// reach is simply discarded and this can only ADD reach, never a wrong FALSE.
#[must_use]
pub fn alphabet_walk_seeds(alphabet: &[i64]) -> Vec<Vec<u8>> {
    // Distinct symbols, order-preserving (nearest-error first).
    let mut syms: Vec<i64> = Vec::new();
    let mut seen_s: BTreeSet<i64> = BTreeSet::new();
    for &a in alphabet {
        if seen_s.insert(a) {
            syms.push(a);
        }
    }
    if syms.len() < 2 {
        return Vec::new();
    }

    let mut seeds: Vec<Vec<u8>> = Vec::new();
    let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
    let add = |seeds: &mut Vec<Vec<u8>>, seen: &mut BTreeSet<Vec<u8>>, buf: Vec<u8>| {
        if seeds.len() < MAX_WALK_SEEDS && seen.insert(buf.clone()) {
            seeds.push(buf);
        }
    };
    // Write `val` little-endian into `buf` at `slot`-th `width`-byte read offset.
    let put = |buf: &mut [u8], slot: usize, width: usize, val: i64| {
        #[allow(clippy::cast_sign_loss)]
        let bits = val as u64;
        for b in 0..width {
            let off = slot * width + b;
            if off < buf.len() {
                #[allow(clippy::cast_possible_truncation)]
                let byte = (bits >> (8 * b)) as u8;
                buf[off] = byte;
            }
        }
    };

    // Cyclic round-robin tilings across the whole buffer.
    for &width in &[1usize, 2, 4] {
        let mut buf = vec![0u8; INPUT_LEN];
        let slots = INPUT_LEN / width;
        for slot in 0..slots {
            put(&mut buf, slot, width, syms[slot % syms.len()]);
        }
        add(&mut seeds, &mut seen, buf);
    }

    // Deterministic pseudo-random walks: every read slot gets an independent symbol.
    // Width 4 is the dominant `int` read width; a couple of width-1 walks cover
    // `char`-driven machines. Fixed seeds keep the corpus byte-identical run to run.
    for w in 0..NUM_RANDOM_WALKS {
        let widths: &[usize] = if w < 2 { &[4, 1] } else { &[4] };
        for &width in widths {
            #[allow(clippy::cast_possible_truncation)]
            let rng_seed = 0x5AF3_A1FA_0000_u64 ^ ((w as u64) << 8) ^ (width as u64);
            let mut rng = XorShift64::new(rng_seed);
            let mut buf = vec![0u8; INPUT_LEN];
            let slots = INPUT_LEN / width;
            for slot in 0..slots {
                put(&mut buf, slot, width, syms[rng.below(syms.len())]);
            }
            add(&mut seeds, &mut seen, buf);
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

    /// `input = nondet(); if (input == 7) reach_error();` with an UNRELATED
    /// `if (g == 5)` guard where `g` is not input-derived. The input→error chop
    /// must contain the input-compared `7` but NOT the state-compared `5`, while
    /// the raw guard set contains both.
    #[test]
    fn chop_isolates_the_input_alphabet() {
        let reach_id = FunctionId::new(1);
        let nondet_id = FunctionId::new(3);
        let input = ValueId::new(10);
        let seven = ValueId::new(11);
        let cmp_in = ValueId::new(12);
        let g = ValueId::new(13); // an un-tainted state value (a fn param).
        let five = ValueId::new(14);
        let cmp_st = ValueId::new(15);

        let mk =
            |id: u128, op: Operation, operands: Vec<ValueId>, dst: Option<ValueId>| Instruction {
                id: InstId::new(id),
                op,
                operands,
                dst,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            };

        // input = __VERIFIER_nondet_int()
        let call_nd = mk(
            99,
            Operation::CallDirect { callee: nondet_id },
            vec![],
            Some(input),
        );
        // cmp_in = icmp eq input, 7
        let icmp_in = mk(
            100,
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![input, seven],
            Some(cmp_in),
        );
        // cmp_st = icmp eq g, 5   (g is a param, not input-tainted)
        let icmp_st = mk(
            101,
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![g, five],
            Some(cmp_st),
        );
        let condbr = mk(
            102,
            Operation::CondBr {
                then_target: BlockId::new(21),
                else_target: BlockId::new(22),
            },
            vec![cmp_in],
            None,
        );
        let condbr2 = mk(
            103,
            Operation::CondBr {
                then_target: BlockId::new(21),
                else_target: BlockId::new(22),
            },
            vec![cmp_st],
            None,
        );
        let call_err = mk(
            104,
            Operation::CallDirect { callee: reach_id },
            vec![],
            None,
        );

        let mut block = AirBlock::new(BlockId::new(20));
        block.instructions = vec![call_nd, icmp_in, icmp_st, condbr, condbr2, call_err];

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
        m.functions.push(decl(3, "__VERIFIER_nondet_int"));
        m.constants.insert(seven, Constant::int(7, 32));
        m.constants.insert(five, Constant::int(5, 32));

        let slice = backward_slice(&m);
        assert!(
            slice.input_guard_constants.contains(&7),
            "input-compared constant 7 is in the chop, got {:?}",
            slice.input_guard_constants
        );
        assert!(
            !slice.input_guard_constants.contains(&5),
            "state-compared constant 5 must NOT be in the chop, got {:?}",
            slice.input_guard_constants
        );
        // The raw guard set is unrefined and still contains both.
        assert!(slice.guard_constants.contains(&7));
        assert!(slice.guard_constants.contains(&5));
        // The bare event alphabet mirrors the chop: the input-compared 7, not 5.
        assert!(
            slice.input_alphabet.contains(&7),
            "input-compared 7 is in the alphabet, got {:?}",
            slice.input_alphabet
        );
        assert!(
            !slice.input_alphabet.contains(&5),
            "state-compared 5 must NOT be in the alphabet, got {:?}",
            slice.input_alphabet
        );
    }

    /// A strict `!=` disequality against a tainted input contributes the bare symbol
    /// to the alphabet but NOT its ±1 neighbours (a neighbour would halt a driver
    /// that validates the read against the exact symbol set). The steering
    /// `input_guard_constants` still carries the neighbours.
    #[test]
    fn alphabet_excludes_strict_neighbours() {
        let reach_id = FunctionId::new(1);
        let nondet_id = FunctionId::new(3);
        let input = ValueId::new(10);
        let five = ValueId::new(11);
        let cmp = ValueId::new(12);

        let mk =
            |id: u128, op: Operation, operands: Vec<ValueId>, dst: Option<ValueId>| Instruction {
                id: InstId::new(id),
                op,
                operands,
                dst,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            };
        let call_nd = mk(
            99,
            Operation::CallDirect { callee: nondet_id },
            vec![],
            Some(input),
        );
        // cmp = icmp ne input, 5   (strict disequality)
        let icmp = mk(
            100,
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![input, five],
            Some(cmp),
        );
        let condbr = mk(
            101,
            Operation::CondBr {
                then_target: BlockId::new(21),
                else_target: BlockId::new(22),
            },
            vec![cmp],
            None,
        );
        let call_err = mk(
            102,
            Operation::CallDirect { callee: reach_id },
            vec![],
            None,
        );
        let mut block = AirBlock::new(BlockId::new(20));
        block.instructions = vec![call_nd, icmp, condbr, call_err];
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
        m.functions.push(decl(3, "__VERIFIER_nondet_int"));
        m.constants.insert(five, Constant::int(5, 32));

        let slice = backward_slice(&m);
        assert!(
            slice.input_alphabet.contains(&5),
            "bare symbol 5 in the alphabet, got {:?}",
            slice.input_alphabet
        );
        assert!(
            !slice.input_alphabet.contains(&4) && !slice.input_alphabet.contains(&6),
            "±1 neighbours must NOT be in the alphabet, got {:?}",
            slice.input_alphabet
        );
        // But the steering set keeps the neighbours for the strict crossing.
        assert!(slice.input_guard_constants.contains(&4));
        assert!(slice.input_guard_constants.contains(&6));
    }

    #[test]
    fn alphabet_walk_seeds_saturate_the_buffer_with_valid_symbols() {
        // A 3-symbol alphabet {1,2,3}: every produced seed must be full-length and
        // contain ONLY alphabet symbols at 4-byte read slots (so a driver that halts
        // on an out-of-alphabet read is never halted by our saturation seed).
        let seeds = alphabet_walk_seeds(&[1, 2, 3]);
        assert!(
            !seeds.is_empty(),
            "a multi-symbol alphabet yields walk seeds"
        );
        assert!(seeds.len() <= MAX_WALK_SEEDS, "seed count bounded");
        // The width-4 cyclic tiling: slots 0,1,2,3,... = 1,2,3,1,...
        let cyclic4 = {
            let mut b = vec![0u8; INPUT_LEN];
            for slot in 0..(INPUT_LEN / 4) {
                let v = [1u32, 2, 3][slot % 3];
                b[slot * 4..slot * 4 + 4].copy_from_slice(&v.to_le_bytes());
            }
            b
        };
        assert!(
            seeds.contains(&cyclic4),
            "the width-4 cyclic round-robin tiling must be present"
        );
        assert!(seeds.iter().all(|s| s.len() == INPUT_LEN));
    }

    #[test]
    fn alphabet_walk_seeds_need_two_symbols_and_are_deterministic() {
        assert!(alphabet_walk_seeds(&[]).is_empty());
        assert!(
            alphabet_walk_seeds(&[7]).is_empty(),
            "a single symbol is already covered by constant tiling"
        );
        let a = alphabet_walk_seeds(&[1, 2, 3, 4, 5, 6, 7]);
        let b = alphabet_walk_seeds(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(a, b, "deterministic");
    }

    /// The chop propagates through arithmetic: `input = nondet(); y = input + 3;
    /// if (y == 100) reach_error();` — `y` is input-tainted, so `100` is in the chop.
    #[test]
    fn chop_follows_arithmetic_taint() {
        let reach_id = FunctionId::new(1);
        let nondet_id = FunctionId::new(3);
        let input = ValueId::new(10);
        let three = ValueId::new(11);
        let y = ValueId::new(12);
        let hundred = ValueId::new(13);
        let cmp = ValueId::new(14);

        let mk =
            |id: u128, op: Operation, operands: Vec<ValueId>, dst: Option<ValueId>| Instruction {
                id: InstId::new(id),
                op,
                operands,
                dst,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            };
        let call_nd = mk(
            99,
            Operation::CallDirect { callee: nondet_id },
            vec![],
            Some(input),
        );
        let add = mk(
            100,
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![input, three],
            Some(y),
        );
        let icmp = mk(
            101,
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![y, hundred],
            Some(cmp),
        );
        let condbr = mk(
            102,
            Operation::CondBr {
                then_target: BlockId::new(21),
                else_target: BlockId::new(22),
            },
            vec![cmp],
            None,
        );
        let call_err = mk(
            103,
            Operation::CallDirect { callee: reach_id },
            vec![],
            None,
        );
        let mut block = AirBlock::new(BlockId::new(20));
        block.instructions = vec![call_nd, add, icmp, condbr, call_err];
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
        m.functions.push(decl(3, "__VERIFIER_nondet_int"));
        m.constants.insert(three, Constant::int(3, 32));
        m.constants.insert(hundred, Constant::int(100, 32));

        let slice = backward_slice(&m);
        assert!(
            slice.input_guard_constants.contains(&100),
            "arithmetic-tainted comparison constant 100 must be in the chop, got {:?}",
            slice.input_guard_constants
        );
    }

    /// A program with no scalar nondet source has an empty chop, and the dictionary
    /// it produces is unchanged — the refinement never perturbs an input-free search.
    #[test]
    fn no_nondet_source_leaves_chop_empty() {
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(decl(1, "reach_error"));
        m.constants.insert(ValueId::new(1), Constant::int(4242, 32));
        let slice = backward_slice(&m);
        assert!(
            slice.input_guard_constants.is_empty(),
            "no nondet source => empty chop"
        );
    }

    /// `main` computes `cmp = fcmp(x, 0.8)` then branches to `reach_error()`; the
    /// slice must harvest the FLOAT guard constant `0.8`.
    #[test]
    fn slice_harvests_the_float_guard_constant() {
        let reach_id = FunctionId::new(1);
        let x = ValueId::new(10);
        let bound = ValueId::new(11);
        let cmp = ValueId::new(12);

        let fcmp = Instruction {
            id: InstId::new(100),
            op: Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt, // op kind is irrelevant to float harvesting
            },
            operands: vec![x, bound],
            dst: Some(cmp),
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };
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
        block.instructions = vec![fcmp, condbr, call];

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
        m.constants.insert(
            bound,
            Constant::Float {
                value: 0.8,
                bits: 64,
            },
        );

        let slice = backward_slice(&m);
        assert!(
            slice.guard_floats.iter().any(|f| (*f - 0.8).abs() < 1e-12),
            "float guard 0.8 must be harvested, got {:?}",
            slice.guard_floats
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
            guard_floats: vec![],
            input_guard_constants: vec![555],
            input_alphabet: vec![555],
        };
        let dict = slice_directed_dictionary(&m, &slice);
        assert_eq!(dict[0], 555, "input-chop constant is front-loaded first");
        assert_eq!(dict[1], 777, "guard constant next");
        assert_eq!(dict[2], 888, "guard constant next");
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
