//! Linear ranking-function synthesis for single natural loops
//! (Podelski–Rybalchenko style; termination-TRUE recall, R7 follow-on).
//!
//! This module upgrades the termination proof's **(L)** condition from
//! "loop-free reachable CFGs" (plan 201) to "every natural loop admits a linear
//! ranking function". A **linear ranking function** for a loop is an affine map
//! `f(x) = c0 + Σ cᵢ·xᵢ` over the loop's integer state such that on every
//! continuing transition
//!
//! - **(bounded)** `f(x) ≥ 0`, and
//! - **(decreasing)** `f(x) − f(x′) ≥ 1`.
//!
//! Such an `f` maps every reachable loop state to a non-negative integer that
//! strictly decreases (by at least 1) each iteration, so the loop runs finitely
//! many times — a **complete, sound termination proof** requiring no witness.
//!
//! # Soundness (why a synthesized `f` never yields a wrong `true`)
//!
//! We model the loop by (a) an **over-approximate guard** `G` — a conjunction of
//! affine inequalities that every *continuing* state provably satisfies — and (b)
//! an **exact affine transition** `x′ = next(x)` for a subset of *stable* state
//! variables (those whose update provably does not wrap the machine integer
//! range on `G`). Because
//!
//! - `G` is a **superset** of the real continuing-state guard (we only ever add
//!   necessary conditions; anything uncertain is dropped, which only weakens `G`),
//!   and
//! - each stable variable's affine `next` **equals** the machine update on `G`
//!   (verified by an explicit overflow-freedom SMT query), and
//! - all other program values are treated as **free universally-quantified
//!   symbols** (so the proof must hold for *every* value they could take),
//!
//! any `f` we synthesize ranks a relation that *contains* the real loop
//! transition. Ranking a superset ranks the real loop, so termination follows.
//! The synthesis itself is discharged by **Farkas' lemma**: the two universally
//! quantified implications are turned into a purely existential linear feasibility
//! problem over the ranking coefficients and non-negative Farkas multipliers, and
//! solved with Z3 (`QF_LIA`). A `Sat` answer *exhibits* an `f`; we treat only
//! `Sat` as a proof (`Unsat`/`Unknown` ⇒ abstain).
//!
//! # Scope (deliberately conservative — abstain, never guess)
//!
//! We handle a function's loops when the CFG is **reducible** (every cycle is a
//! natural loop identified by a dominance back-edge) and **each natural loop has
//! its own distinct header with a single latch**. This covers the common
//! sequential-loops (`for … ; for …`) and nested-loops (`for { for } `) shapes:
//! each natural loop is ranked *independently* by its own linear ranking function.
//! We abstain (⇒ `unknown`, score 0, never a wrong verdict) on:
//!
//! - **irreducible** CFGs — a cycle with no dominance back-edge would be missed
//!   by natural-loop enumeration, so we reject unless removing all back-edges
//!   leaves an acyclic graph;
//! - **multi-latch** loops — a single header with two back-edges can oscillate
//!   (each latch transition individually ranked ⇏ the loop terminates), so we
//!   require one latch per header;
//! - unresolved indirect control flow and any non-affine update.
//!
//! ## Why per-loop ranking on a reducible CFG is sound
//!
//! In a reducible CFG every cycle passes through exactly one natural-loop header,
//! and natural loops are either disjoint or properly nested. If every natural loop
//! admits a ranking function then no infinite execution exists: an infinite run
//! would traverse some loop's back-edge infinitely often, contradicting that
//! loop's strictly-decreasing, bounded-below rank. When ranking an *outer* loop we
//! model an inner loop's effect on state as a free (universally-quantified) havoc,
//! so the outer proof holds for every inner outcome; the inner loop's own ranking
//! function separately guarantees it runs finitely. Follow-ons (lexicographic /
//! multiphase ranking, multi-latch joint ranking) can widen this later.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_analysis::cfg::Cfg;
use saf_analysis::z3_utils::compute_dominators;
use saf_core::air::{AirFunction, AirModule, BinaryOp, CastKind, Constant, Operation};
use saf_core::ids::{BlockId, TypeId, ValueId};

/// Maximum recursion depth when expanding an SSA value into an affine form.
/// Bounds work on pathological IR; exceeding it ⇒ treat the value as opaque.
const AFFINE_DEPTH_LIMIT: u32 = 64;

/// Deterministic Z3 knobs (mirror `saf-analysis`'s solver): a wall-clock backstop,
/// a deterministic work budget, and a fixed random seed (NFR-DET).
const Z3_TIMEOUT_MS: u32 = 2_000;
const Z3_RLIMIT: u32 = 2_000_000;
const Z3_SEED: u32 = 42;

/// Public entry point: does every natural loop in `func` admit a linear ranking
/// function (⇒ the function's loops all terminate)?
///
/// Returns `true` **only** when the CFG is reducible, every natural loop has a
/// distinct single-latch header, and a ranking function is synthesized for *each*
/// natural loop. Abstains (`false`) on anything outside the supported shape or
/// when synthesis fails for any loop — the caller maps abstain to `unknown`, so a
/// `false` here is never a verdict, only "not proven".
///
/// Precondition: `func` is a defined function whose CFG *has* a loop (the caller
/// checks `cfg_has_loops` first); a loop-free function is handled upstream.
#[must_use]
pub fn loops_are_ranked(func: &AirFunction, module: &AirModule, cfg: &Cfg) -> bool {
    let Some(loops) = extract_natural_loops(func, cfg) else {
        return false;
    };
    // Every natural loop must build an affine model *and* be ranked; any failure
    // (unmodelable loop, or Z3 `Unsat`/`Unknown`) abstains the whole function.
    loops.iter().all(|li| {
        build_loop_model(func, module, li).is_some_and(|model| synthesize_ranking_function(&model))
    })
}

// ---------------------------------------------------------------------------
// Loop extraction
// ---------------------------------------------------------------------------

/// A single natural loop identified by its unique back-edge `latch → header`.
struct LoopInfo {
    header: BlockId,
    latch: BlockId,
    /// The natural-loop block set (header plus every node that reaches the latch
    /// without passing through the header).
    body: BTreeSet<BlockId>,
    /// Immediate-dominator map for the function CFG (for dominance queries).
    idom: BTreeMap<BlockId, BlockId>,
}

/// Identify *every* natural loop of the function (one per dominance back-edge), or
/// `None` if the CFG is not in the supported shape:
///
/// - **irreducible** — removing the dominance back-edges does not leave an acyclic
///   graph, so some cycle has no natural-loop header and would be silently missed;
/// - **multi-latch** — a single header has more than one back-edge, which can
///   oscillate (per-latch ranking is unsound), so we require one latch per header;
/// - no back-edge at all (caller guarantees `cfg_has_loops`, so this means an
///   irreducible cycle — already covered by the reducibility check).
fn extract_natural_loops(func: &AirFunction, cfg: &Cfg) -> Option<Vec<LoopInfo>> {
    let idom = compute_dominators(cfg);

    // A back-edge `u → v` is a CFG edge whose head `v` dominates its tail `u`.
    let mut back_edges: Vec<(BlockId, BlockId)> = Vec::new();
    for (&u, succs) in &cfg.successors {
        for &v in succs {
            if dominates(v, u, &idom) {
                back_edges.push((u, v));
            }
        }
    }
    if back_edges.is_empty() {
        return None;
    }

    // Reducibility: with all dominance back-edges removed, a reducible CFG is
    // acyclic. If a cycle survives, some cycle has no natural-loop header (an
    // irreducible retreating edge) and enumerating natural loops would miss it —
    // ranking the rest and returning `true` would be unsound. Abstain.
    let back_set: BTreeSet<(BlockId, BlockId)> = back_edges.iter().copied().collect();
    if forward_graph_has_cycle(cfg, &back_set) {
        return None;
    }

    // Multi-latch guard: two back-edges into the same header form one loop with two
    // latch transitions; ranking each independently does not prove the loop
    // terminates (it may alternate latches forever). Require one latch per header.
    let mut headers: BTreeSet<BlockId> = BTreeSet::new();
    for &(_, header) in &back_edges {
        if !headers.insert(header) {
            return None;
        }
    }

    let mut loops = Vec::with_capacity(back_edges.len());
    for (latch, header) in back_edges {
        // Sanity: the header block must actually exist in the function.
        func.blocks.iter().find(|b| b.id == header)?;

        // Natural-loop body: {header} ∪ {nodes that can reach `latch` without going
        // through `header`}. Reverse BFS from `latch`, never expanding past
        // `header`. For a nested loop this yields the *whole* outer body (inner
        // blocks included); the inner loop's effect is over-approximated as havoc
        // during outer-loop synthesis, and the inner loop is ranked on its own.
        let mut body: BTreeSet<BlockId> = BTreeSet::new();
        body.insert(header);
        let mut queue: VecDeque<BlockId> = VecDeque::new();
        if latch != header {
            body.insert(latch);
            queue.push_back(latch);
        }
        while let Some(b) = queue.pop_front() {
            if let Some(preds) = cfg.predecessors.get(&b) {
                for &p in preds {
                    if p != header && body.insert(p) {
                        queue.push_back(p);
                    }
                }
            }
        }

        loops.push(LoopInfo {
            header,
            latch,
            body,
            idom: idom.clone(),
        });
    }
    Some(loops)
}

/// Does the CFG contain a cycle once the dominance back-edges are removed? A
/// reducible CFG becomes a DAG; a surviving cycle proves irreducibility (an
/// unhandled retreating edge), on which we must abstain.
fn forward_graph_has_cycle(cfg: &Cfg, back_set: &BTreeSet<(BlockId, BlockId)>) -> bool {
    // Iterative three-colour DFS over the back-edge-free graph. `on_stack` marks
    // the current DFS path; re-entering an on-stack node is a forward cycle.
    let mut visited: BTreeSet<BlockId> = BTreeSet::new();
    let mut on_stack: BTreeSet<BlockId> = BTreeSet::new();
    // Stack frames: (node, whether we've begun expanding it).
    let mut stack: Vec<(BlockId, bool)> = Vec::new();

    for &start in cfg.successors.keys() {
        if visited.contains(&start) {
            continue;
        }
        stack.push((start, false));
        while let Some((node, expanded)) = stack.pop() {
            if expanded {
                on_stack.remove(&node);
                continue;
            }
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);
            on_stack.insert(node);
            // Post-visit marker to pop `node` off the path once its subtree is done.
            stack.push((node, true));
            if let Some(succs) = cfg.successors.get(&node) {
                for &s in succs {
                    if back_set.contains(&(node, s)) {
                        continue; // skip the removed back-edge
                    }
                    if on_stack.contains(&s) {
                        return true;
                    }
                    if !visited.contains(&s) {
                        stack.push((s, false));
                    }
                }
            }
        }
    }
    false
}

/// Does block `a` dominate block `b` (walk `b`'s immediate-dominator chain to the
/// entry; `a` dominates `b` iff it appears on the chain, including `b` itself)?
fn dominates(a: BlockId, b: BlockId, idom: &BTreeMap<BlockId, BlockId>) -> bool {
    let mut cur = b;
    loop {
        if cur == a {
            return true;
        }
        match idom.get(&cur) {
            Some(&parent) if parent != cur => cur = parent,
            // Reached the entry (its idom is itself or absent).
            _ => return false,
        }
    }
}

// ---------------------------------------------------------------------------
// Affine expressions over leaf symbols
// ---------------------------------------------------------------------------

/// An affine form `Σ coeff·symbol + constant` over *leaf* `ValueId` symbols
/// (loop state variables and universally-quantified parameters). All coefficients
/// are exact `i128`; extraction abstains before any value could overflow `i128`.
#[derive(Clone, Debug, Default)]
struct Affine {
    terms: BTreeMap<ValueId, i128>,
    constant: i128,
}

impl Affine {
    fn constant(c: i128) -> Self {
        Self {
            terms: BTreeMap::new(),
            constant: c,
        }
    }
    fn symbol(v: ValueId) -> Self {
        let mut terms = BTreeMap::new();
        terms.insert(v, 1);
        Self { terms, constant: 0 }
    }
    fn add(&self, other: &Self) -> Option<Self> {
        let mut out = self.clone();
        out.constant = out.constant.checked_add(other.constant)?;
        for (&k, &c) in &other.terms {
            let e = out.terms.entry(k).or_insert(0);
            *e = e.checked_add(c)?;
        }
        out.terms.retain(|_, c| *c != 0);
        Some(out)
    }
    fn sub(&self, other: &Self) -> Option<Self> {
        self.add(&other.scale(-1)?)
    }
    fn scale(&self, k: i128) -> Option<Self> {
        let mut out = Affine::constant(self.constant.checked_mul(k)?);
        for (&s, &c) in &self.terms {
            let nc = c.checked_mul(k)?;
            if nc != 0 {
                out.terms.insert(s, nc);
            }
        }
        Some(out)
    }
    /// If this form is a bare constant (no symbols), return it.
    fn as_constant(&self) -> Option<i128> {
        self.terms.is_empty().then_some(self.constant)
    }
}

/// Signedness of an integer SSA value, inferred from how the program *uses* it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sign {
    Signed,
    Unsigned,
    Unknown,
}

/// Minimal per-value definition record for affine expansion.
struct Def {
    op: Operation,
    operands: Vec<ValueId>,
    in_loop: bool,
    result_type: Option<TypeId>,
}

// ---------------------------------------------------------------------------
// Loop model
// ---------------------------------------------------------------------------

/// A single linear region constraint `Σ aᵢ·zᵢ + b ≥ 0`.
#[derive(Clone)]
struct Constraint {
    coeffs: BTreeMap<ValueId, i128>,
    constant: i128,
}

/// The extracted, fully-affine model of one natural loop, ready for synthesis.
struct LoopModel {
    /// Stable state variables (header phis whose affine `next` never wraps on the
    /// region): symbol ↦ its `next` affine form. These may carry a non-zero
    /// ranking coefficient and drive the decrease.
    state: BTreeMap<ValueId, Affine>,
    /// Loop-invariant parameter symbols (defined outside the loop / function
    /// params). Eligible for a ranking coefficient (identity transition ⇒ zero net
    /// contribution to the decrease) but only ever help bound `f` from below.
    invariants: BTreeSet<ValueId>,
    /// Over-approximate guard + type-bound region: every continuing state
    /// satisfies all of these.
    region: Vec<Constraint>,
    /// Every leaf symbol appearing anywhere in the model (for the per-symbol
    /// Farkas identity).
    universe: BTreeSet<ValueId>,
}

/// Build the affine [`LoopModel`], or `None` if any part is not exactly
/// expressible (⇒ abstain).
fn build_loop_model(func: &AirFunction, module: &AirModule, li: &LoopInfo) -> Option<LoopModel> {
    let defs = index_defs(func, &li.body);
    let signs = infer_signs(func);

    // Header phis and their back-edge (latch) incoming value.
    let header_block = func.blocks.iter().find(|b| b.id == li.header)?;
    let mut phi_next: BTreeMap<ValueId, ValueId> = BTreeMap::new();
    for inst in &header_block.instructions {
        if let Operation::Phi { incoming } = &inst.op {
            let Some(dst) = inst.dst else { continue };
            // The value that flows in along the back-edge (from the latch).
            let mut latch_val = None;
            for (pred, val) in incoming {
                if *pred == li.latch {
                    latch_val = Some(*val);
                }
            }
            if let Some(v) = latch_val {
                phi_next.insert(dst, v);
            }
        }
    }
    if phi_next.is_empty() {
        return None;
    }

    let header_phis: BTreeSet<ValueId> = phi_next.keys().copied().collect();

    // --- Guards (over-approximate; only necessary stay-conditions) -----------
    let mut region: Vec<Constraint> = Vec::new();
    let mut universe: BTreeSet<ValueId> = BTreeSet::new();
    for &b in &li.body {
        // Only exit branches that *dominate the latch* impose a necessary
        // continuing condition (every iteration's path to the back-edge passes
        // through them). Others are dropped — a strictly weaker (sound) guard.
        if !dominates(b, li.latch, &li.idom) {
            continue;
        }
        if let Some(cons) = guard_constraints_for_block(func, module, &defs, &header_phis, li, b) {
            for c in cons {
                for k in c.coeffs.keys() {
                    universe.insert(*k);
                }
                region.push(c);
            }
        }
    }

    // --- Classify leaf symbols + collect type bounds -------------------------
    // Resolve each phi's `next`; a phi with a non-affine next, or one that may
    // overflow, is demoted to an opaque (varying) parameter — zero ranking
    // coefficient, no modeled update, but still a valid free symbol.
    let mut state: BTreeMap<ValueId, Affine> = BTreeMap::new();
    let mut next_forms: BTreeMap<ValueId, Affine> = BTreeMap::new();
    for (&phi, &nv) in &phi_next {
        if let Some(a) = resolve_affine(nv, &defs, &header_phis, module, 0) {
            for k in a.terms.keys() {
                universe.insert(*k);
            }
            next_forms.insert(phi, a);
        }
    }
    universe.extend(header_phis.iter().copied());

    // Type bounds for every integer leaf symbol with a *known* signedness and a
    // representable width — a valid fact about the real state, so adding it only
    // shrinks the region by truths (keeping it a superset of real states).
    let mut bounds: BTreeMap<ValueId, (i128, i128)> = BTreeMap::new();
    for &sym in &universe {
        if let Some((lo, hi)) = type_bounds(sym, &defs, &signs, module) {
            bounds.insert(sym, (lo, hi));
            region.push(Constraint {
                coeffs: BTreeMap::from([(sym, 1)]),
                constant: -lo,
            }); // sym - lo ≥ 0
            region.push(Constraint {
                coeffs: BTreeMap::from([(sym, -1)]),
                constant: hi,
            }); // hi - sym ≥ 0
        }
    }

    // Overflow-freedom: a phi is a *stable* state var iff its `next` provably
    // stays within its type range on the region (so the affine model equals the
    // machine update). Requires a known type bound for the phi itself.
    for (&phi, next) in &next_forms {
        let Some(&(lo, hi)) = bounds.get(&phi) else {
            continue;
        };
        if next_never_overflows(next, lo, hi, &region, &universe) {
            state.insert(phi, next.clone());
        }
    }
    if state.is_empty() {
        return None;
    }

    // Invariant params = leaf symbols that are neither header phis nor defined
    // inside the loop (function params, nondet results, globals). They may carry a
    // ranking coefficient; everything else defaults to a zero-coefficient free
    // symbol.
    let mut invariants: BTreeSet<ValueId> = BTreeSet::new();
    for &sym in &universe {
        if header_phis.contains(&sym) {
            continue;
        }
        let inside = defs.get(&sym).is_some_and(|d| d.in_loop);
        if !inside {
            invariants.insert(sym);
        }
    }

    Some(LoopModel {
        state,
        invariants,
        region,
        universe,
    })
}

/// Build a `ValueId → Def` index for the function, flagging whether each
/// definition sits inside the loop body.
fn index_defs(func: &AirFunction, body: &BTreeSet<BlockId>) -> BTreeMap<ValueId, Def> {
    let mut defs = BTreeMap::new();
    for block in &func.blocks {
        let in_loop = body.contains(&block.id);
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                defs.insert(
                    dst,
                    Def {
                        op: inst.op.clone(),
                        operands: inst.operands.clone(),
                        in_loop,
                        result_type: inst.result_type,
                    },
                );
            }
        }
    }
    defs
}

/// Expand SSA value `v` into an affine form over leaf symbols (header phis and
/// values not further expandable), or `None` if it is not affine.
fn resolve_affine(
    v: ValueId,
    defs: &BTreeMap<ValueId, Def>,
    header_phis: &BTreeSet<ValueId>,
    module: &AirModule,
    depth: u32,
) -> Option<Affine> {
    if depth > AFFINE_DEPTH_LIMIT {
        return None;
    }
    // A recognized integer constant folds to a bare constant.
    if let Some(Constant::Int { value, .. }) = module.constants.get(&v) {
        return Some(Affine::constant(i128::from(*value)));
    }
    // Header phis are the loop's state leaves — never expanded further.
    if header_phis.contains(&v) {
        return Some(Affine::symbol(v));
    }
    let Some(def) = defs.get(&v) else {
        // No definition in this function (function param / external) ⇒ a leaf.
        return Some(Affine::symbol(v));
    };
    // Values defined *outside* the loop are loop-invariant ⇒ opaque leaves.
    if !def.in_loop {
        return Some(Affine::symbol(v));
    }
    match &def.op {
        Operation::BinaryOp {
            kind: kind @ (BinaryOp::Add | BinaryOp::Sub),
        } => {
            let a = resolve_affine(*def.operands.first()?, defs, header_phis, module, depth + 1)?;
            let b = resolve_affine(*def.operands.get(1)?, defs, header_phis, module, depth + 1)?;
            if matches!(kind, BinaryOp::Add) {
                a.add(&b)
            } else {
                a.sub(&b)
            }
        }
        Operation::BinaryOp {
            kind: BinaryOp::Mul,
        } => {
            let a = resolve_affine(*def.operands.first()?, defs, header_phis, module, depth + 1)?;
            let b = resolve_affine(*def.operands.get(1)?, defs, header_phis, module, depth + 1)?;
            // Affine only if at least one factor is a constant.
            if let Some(k) = b.as_constant() {
                a.scale(k)
            } else if let Some(k) = a.as_constant() {
                b.scale(k)
            } else {
                None
            }
        }
        // Value-preserving casts and copies pass the value through unchanged:
        // `zext`/`sext` preserve the (unsigned/signed) integer value.
        Operation::Cast {
            kind: CastKind::ZExt | CastKind::SExt,
            ..
        }
        | Operation::Copy
        | Operation::Freeze => {
            resolve_affine(*def.operands.first()?, defs, header_phis, module, depth + 1)
        }
        // Anything else (Load, Call, non-header Phi, Trunc, …) is opaque: treat as
        // a fresh free leaf symbol (sound — universally quantified downstream).
        _ => Some(Affine::symbol(v)),
    }
}

/// Extract the necessary stay-condition(s) imposed by block `b`'s exit branch,
/// as region constraints `expr ≥ 0`. Returns `None` if `b` is not an
/// affine-resolvable single-exit loop branch.
fn guard_constraints_for_block(
    func: &AirFunction,
    module: &AirModule,
    defs: &BTreeMap<ValueId, Def>,
    header_phis: &BTreeSet<ValueId>,
    li: &LoopInfo,
    b: BlockId,
) -> Option<Vec<Constraint>> {
    let block = func.blocks.iter().find(|bb| bb.id == b)?;
    let term = block.terminator()?;
    let Operation::CondBr {
        then_target,
        else_target,
    } = &term.op
    else {
        return None;
    };
    let then_in = li.body.contains(then_target);
    let else_in = li.body.contains(else_target);
    // Only a *single-exit* branch imposes a necessary continuing condition.
    let stay_when_true = match (then_in, else_in) {
        (true, false) => true,
        (false, true) => false,
        // Both-in (internal branch) or both-out (not a loop exit) ⇒ no constraint.
        _ => return None,
    };
    let cond = *term.operands.first()?;
    let def = defs.get(&cond)?;
    let Operation::BinaryOp { kind } = &def.op else {
        return None;
    };
    let lhs = resolve_affine(*def.operands.first()?, defs, header_phis, module, 0)?;
    let rhs = resolve_affine(*def.operands.get(1)?, defs, header_phis, module, 0)?;
    comparison_constraints(*kind, &lhs, &rhs, stay_when_true)
}

/// Turn an integer comparison (taken in the given polarity) into region
/// constraints `expr ≥ 0`. Over the integers, strict `<` becomes `≥ 1`.
/// Signed/unsigned share the same math form here: the constraint is a *necessary*
/// condition on in-range continuing states either way, so it stays a sound
/// superset guard. Non-order predicates that are not affine half-spaces (`!=`,
/// and `==` under negation) are dropped.
fn comparison_constraints(
    kind: BinaryOp,
    lhs: &Affine,
    rhs: &Affine,
    taken_true: bool,
) -> Option<Vec<Constraint>> {
    use BinaryOp::{
        ICmpEq, ICmpNe, ICmpSge, ICmpSgt, ICmpSle, ICmpSlt, ICmpUge, ICmpUgt, ICmpUle, ICmpUlt,
    };
    // Normalize to the effective predicate actually taken on the stay-edge.
    let effective = if taken_true {
        kind
    } else {
        match kind {
            ICmpSlt => ICmpSge,
            ICmpSle => ICmpSgt,
            ICmpSgt => ICmpSle,
            ICmpSge => ICmpSlt,
            ICmpUlt => ICmpUge,
            ICmpUle => ICmpUgt,
            ICmpUgt => ICmpUle,
            ICmpUge => ICmpUlt,
            // Negated equality / inequality: not an affine half-space we use.
            ICmpEq => ICmpNe,
            ICmpNe => ICmpEq,
            other => other,
        }
    };
    // Build `expr ≥ 0` forms. For `a < b` use `b - a - 1 ≥ 0` (integers).
    let cons = |a: &Affine, b: &Affine, minus: i128| -> Option<Constraint> {
        let e = b.sub(a)?.add(&Affine::constant(minus))?;
        Some(Constraint {
            coeffs: e.terms,
            constant: e.constant,
        })
    };
    match effective {
        ICmpSlt | ICmpUlt => Some(vec![cons(lhs, rhs, -1)?]), // b - a - 1 ≥ 0
        ICmpSle | ICmpUle => Some(vec![cons(lhs, rhs, 0)?]),  // b - a ≥ 0
        ICmpSgt | ICmpUgt => Some(vec![cons(rhs, lhs, -1)?]), // a - b - 1 ≥ 0
        ICmpSge | ICmpUge => Some(vec![cons(rhs, lhs, 0)?]),  // a - b ≥ 0
        ICmpEq => Some(vec![cons(lhs, rhs, 0)?, cons(rhs, lhs, 0)?]), // a = b
        // `!=` is not convex; drop it (sound — weaker guard).
        _ => Some(vec![]),
    }
}

/// Infer each integer value's signedness from the operations that consume it.
fn infer_signs(func: &AirFunction) -> BTreeMap<ValueId, Sign> {
    let mut signed: BTreeSet<ValueId> = BTreeSet::new();
    let mut unsigned: BTreeSet<ValueId> = BTreeSet::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            // `hint`: Some(true) ⇒ its operands are used signed, Some(false) ⇒
            // unsigned. `ops` is the operand slice those hints apply to.
            let (hint, ops): (Option<bool>, &[ValueId]) = match &inst.op {
                Operation::BinaryOp { kind } => {
                    let h = match kind {
                        BinaryOp::ICmpSgt
                        | BinaryOp::ICmpSge
                        | BinaryOp::ICmpSlt
                        | BinaryOp::ICmpSle
                        | BinaryOp::SDiv
                        | BinaryOp::SRem
                        | BinaryOp::AShr => Some(true),
                        BinaryOp::ICmpUgt
                        | BinaryOp::ICmpUge
                        | BinaryOp::ICmpUlt
                        | BinaryOp::ICmpUle
                        | BinaryOp::UDiv
                        | BinaryOp::URem
                        | BinaryOp::LShr => Some(false),
                        _ => None,
                    };
                    (h, inst.operands.as_slice())
                }
                Operation::Cast {
                    kind: CastKind::SExt,
                    ..
                } => (Some(true), inst.operands.get(..1).unwrap_or_default()),
                Operation::Cast {
                    kind: CastKind::ZExt,
                    ..
                } => (Some(false), inst.operands.get(..1).unwrap_or_default()),
                _ => (None, [].as_slice()),
            };
            if let Some(is_signed) = hint {
                for &op in ops {
                    if is_signed {
                        signed.insert(op);
                    } else {
                        unsigned.insert(op);
                    }
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    for &v in signed.union(&unsigned) {
        let sign = match (signed.contains(&v), unsigned.contains(&v)) {
            (true, false) => Sign::Signed,
            (false, true) => Sign::Unsigned,
            _ => Sign::Unknown,
        };
        out.insert(v, sign);
    }
    out
}

/// Integer type range `[lo, hi]` for `sym`, if its width is known (≤ 64 bits) and
/// its signedness is unambiguous. Returns `None` when the bounds cannot be
/// represented (⇒ no bound is asserted; the symbol stays free — sound).
fn type_bounds(
    sym: ValueId,
    defs: &BTreeMap<ValueId, Def>,
    signs: &BTreeMap<ValueId, Sign>,
    module: &AirModule,
) -> Option<(i128, i128)> {
    let type_id = defs.get(&sym).and_then(|d| d.result_type)?;
    let width = i128::from(int_width(module, type_id)?);
    if width == 0 || width > 64 {
        return None;
    }
    match signs.get(&sym).copied().unwrap_or(Sign::Unknown) {
        Sign::Signed => {
            let hi = (1i128 << (width - 1)) - 1;
            let lo = -(1i128 << (width - 1));
            Some((lo, hi))
        }
        Sign::Unsigned => {
            let hi = (1i128 << width) - 1;
            Some((0, hi))
        }
        Sign::Unknown => None,
    }
}

/// Resolve a `TypeId`'s integer bit width from the module type table.
fn int_width(module: &AirModule, type_id: TypeId) -> Option<u16> {
    match module.types.get(&type_id) {
        Some(saf_core::air::AirType::Integer { bits }) => Some(*bits),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Z3: overflow-freedom check + Farkas synthesis
// ---------------------------------------------------------------------------

/// Fresh deterministic Z3 solver.
fn new_solver() -> z3::Solver {
    let solver = z3::Solver::new();
    let mut params = z3::Params::new();
    params.set_u32("timeout", Z3_TIMEOUT_MS);
    params.set_u32("rlimit", Z3_RLIMIT);
    params.set_u32("random_seed", Z3_SEED);
    solver.set_params(&params);
    solver
}

/// Deterministic Z3 variable name for a symbol (index in the sorted universe).
fn sym_name(idx: usize) -> String {
    format!("z{idx}")
}

/// Build the Z3 `Int` for an affine form over the concrete symbol variables.
fn affine_to_z3(
    a: &Affine,
    index: &BTreeMap<ValueId, usize>,
    vars: &BTreeMap<usize, z3::ast::Int>,
) -> Option<z3::ast::Int> {
    let mut acc = z3::ast::Int::from_i64(i128_to_i64(a.constant)?);
    for (&s, &c) in &a.terms {
        let idx = *index.get(&s)?;
        let var = vars.get(&idx)?;
        acc += var * z3::ast::Int::from_i64(i128_to_i64(c)?);
    }
    Some(acc)
}

/// Does `next` provably stay within `[lo, hi]` on the region (⇒ no machine wrap)?
/// Sound direction: `Unsat` of "next is out of range" ⇒ never overflows.
fn next_never_overflows(
    next: &Affine,
    lo: i128,
    hi: i128,
    region: &[Constraint],
    universe: &BTreeSet<ValueId>,
) -> bool {
    let index: BTreeMap<ValueId, usize> = universe.iter().copied().zip(0..).collect();
    let vars: BTreeMap<usize, z3::ast::Int> = (0..universe.len())
        .map(|i| (i, z3::ast::Int::new_const(sym_name(i))))
        .collect();
    let (Some(lo64), Some(hi64)) = (i128_to_i64(lo), i128_to_i64(hi)) else {
        return false;
    };
    // Check both out-of-range directions; either satisfiable ⇒ overflow possible.
    for below in [true, false] {
        let solver = new_solver();
        for c in region {
            let Some(b) = constraint_to_z3(c, &index, &vars) else {
                return false;
            };
            solver.assert(&b);
        }
        let Some(nz) = affine_to_z3(next, &index, &vars) else {
            return false;
        };
        let bound = if below {
            nz.lt(z3::ast::Int::from_i64(lo64))
        } else {
            nz.gt(z3::ast::Int::from_i64(hi64))
        };
        solver.assert(&bound);
        match solver.check() {
            z3::SatResult::Unsat => {}
            // Sat (overflow reachable) or Unknown (cannot prove safe) ⇒ not stable.
            _ => return false,
        }
    }
    true
}

/// Assert `Σ aᵢ·zᵢ + b ≥ 0` as a Z3 `Bool`.
fn constraint_to_z3(
    c: &Constraint,
    index: &BTreeMap<ValueId, usize>,
    vars: &BTreeMap<usize, z3::ast::Int>,
) -> Option<z3::ast::Bool> {
    let a = Affine {
        terms: c.coeffs.clone(),
        constant: c.constant,
    };
    let e = affine_to_z3(&a, index, vars)?;
    Some(e.ge(z3::ast::Int::from_i64(0)))
}

/// Synthesize a linear ranking function for the model via a Farkas-lemma
/// reduction to `QF_LIA` feasibility. Returns `true` iff Z3 finds one (`Sat`).
///
/// Unknowns: ranking coefficients `c_m` (for template symbols = state ∪
/// invariants), constant `c0`, a strictly-positive decrease `δ ≥ 1`, and a
/// non-negative Farkas multiplier per region constraint for each of the two
/// requirements. For a requirement `region ⇒ L ≥ k`, Farkas gives the identity
/// `L(z) − k = Σⱼ λⱼ·exprⱼ(z) + s` with `λ, s ≥ 0`; matching coefficients
/// per symbol and the constant term yields purely linear constraints.
fn synthesize_ranking_function(model: &LoopModel) -> bool {
    // Template symbols may carry a ranking coefficient; the universe is every
    // symbol appearing in the region / transition (for the per-symbol identity).
    let template: BTreeSet<ValueId> = model
        .state
        .keys()
        .copied()
        .chain(model.invariants.iter().copied())
        .collect();
    let universe: Vec<ValueId> = model.universe.iter().copied().collect();

    let solver = new_solver();

    // Ranking coefficients c_m (template only) and constant c0. Names are given a
    // distinct `coef_`/`rank_` prefix so no coefficient can alias the constant or a
    // Farkas multiplier — z3 0.19's global context makes same-named consts identical.
    let mut coeff: BTreeMap<ValueId, z3::ast::Int> = BTreeMap::new();
    for (i, m) in template.iter().enumerate() {
        coeff.insert(*m, z3::ast::Int::new_const(format!("coef_{i}")));
    }
    let c0 = z3::ast::Int::new_const("rank_const");
    let delta = z3::ast::Int::new_const("rank_delta");
    solver.assert(delta.ge(z3::ast::Int::from_i64(1)));

    // Coefficient of template symbol `m` in the ranking form `f` (0 outside).
    let coeff_f = |m: &ValueId| -> z3::ast::Int {
        coeff
            .get(m)
            .cloned()
            .unwrap_or_else(|| z3::ast::Int::from_i64(0))
    };

    // Requirement A: region ⇒ f ≥ 0  (L_A = f, k = 0).
    // Requirement B: region ⇒ f(x) − f(x′) ≥ δ  (L_B, k = δ).
    //
    // L_A coeff on symbol m = coeff_f(m); L_A const = c0.
    // L_B coeff on symbol m = Σ_{state i} c_i·([i == m] − next_i[m]);
    // L_B const            = −Σ_{state i} c_i·next_i.const.
    let zero = || z3::ast::Int::from_i64(0);

    // Per-requirement Farkas multipliers λ ≥ 0.
    let make_lambdas = |tag: &str| -> Vec<z3::ast::Int> {
        (0..model.region.len())
            .map(|j| {
                let l = z3::ast::Int::new_const(format!("lam_{tag}_{j}"));
                solver.assert(l.ge(z3::ast::Int::from_i64(0)));
                l
            })
            .collect()
    };
    let lam_a = make_lambdas("a");
    let lam_b = make_lambdas("b");

    // Σⱼ λⱼ·aⱼ(m): the region's contribution to symbol m's coefficient.
    let region_coeff = |lams: &[z3::ast::Int], m: &ValueId| -> Option<z3::ast::Int> {
        let mut acc = zero();
        for (j, c) in model.region.iter().enumerate() {
            if let Some(&a) = c.coeffs.get(m) {
                acc += &lams[j] * z3::ast::Int::from_i64(i128_to_i64(a)?);
            }
        }
        Some(acc)
    };
    // Σⱼ λⱼ·bⱼ: the region's constant contribution.
    let region_const = |lams: &[z3::ast::Int]| -> Option<z3::ast::Int> {
        let mut acc = zero();
        for (j, c) in model.region.iter().enumerate() {
            acc += &lams[j] * z3::ast::Int::from_i64(i128_to_i64(c.constant)?);
        }
        Some(acc)
    };

    // Per-symbol coefficient identities.
    for m in &universe {
        // Requirement A: coeff_f(m) == Σ λ^A aⱼ(m).
        let Some(ra) = region_coeff(&lam_a, m) else {
            return false;
        };
        solver.assert(coeff_f(m).eq(ra));

        // Requirement B: L_B coeff(m) == Σ λ^B aⱼ(m).
        let mut lb_m = zero();
        for (i, next_i) in &model.state {
            let ci = coeff_f(i);
            let indicator = i128::from(i == m);
            let in_next = next_i.terms.get(m).copied().unwrap_or(0);
            let Some(k) = i128_to_i64(indicator - in_next) else {
                return false;
            };
            lb_m += &ci * z3::ast::Int::from_i64(k);
        }
        let Some(rb) = region_coeff(&lam_b, m) else {
            return false;
        };
        solver.assert(lb_m.eq(rb));
    }

    // Constant-term inequalities: const_L − k − Σ λ bⱼ ≥ 0.
    // Requirement A: c0 − 0 − Σ λ^A bⱼ ≥ 0.
    let Some(const_a) = region_const(&lam_a) else {
        return false;
    };
    solver.assert((c0 - const_a).ge(zero()));

    // Requirement B: (−Σ_i c_i·next_i.const) − δ − Σ λ^B bⱼ ≥ 0.
    let mut lb_const = zero();
    for (i, next_i) in &model.state {
        let ci = coeff_f(i);
        let Some(k) = i128_to_i64(-next_i.constant) else {
            return false;
        };
        lb_const += &ci * z3::ast::Int::from_i64(k);
    }
    let Some(const_b) = region_const(&lam_b) else {
        return false;
    };
    solver.assert((lb_const - delta - const_b).ge(zero()));

    matches!(solver.check(), z3::SatResult::Sat)
}

/// Narrow an `i128` to `i64`, or `None` if it does not fit (⇒ abstain).
fn i128_to_i64(v: i128) -> Option<i64> {
    i64::try_from(v).ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use saf_core::air::{AirBlock, AirFunction, AirModule, AirType, Instruction};
    use saf_core::id::make_id;
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, TypeId, ValueId};

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

    /// A value-producing instruction with a result type.
    fn vinst(
        id: &str,
        op: Operation,
        dst: ValueId,
        operands: Vec<ValueId>,
        ty: TypeId,
    ) -> Instruction {
        Instruction {
            id: iid(id),
            op,
            operands,
            dst: Some(dst),
            span: None,
            symbol: None,
            result_type: Some(ty),
            extensions: BTreeMap::new(),
        }
    }
    /// A terminator (no dst / result).
    fn term(id: &str, op: Operation, operands: Vec<ValueId>) -> Instruction {
        Instruction {
            id: iid(id),
            op,
            operands,
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    /// The compare bound: a program constant or a loop-invariant nondet value.
    #[derive(Clone, Copy)]
    enum Bound {
        Const(i64),
        Nondet,
    }

    /// Build a single-counter `while (x CMP bound) x += step;` loop as a module,
    /// with `x` typed `i32`. `cmp`'s signedness drives the type-bound inference.
    #[allow(clippy::too_many_lines, clippy::many_single_char_names)]
    fn counter_loop(cmp: BinaryOp, bound: Bound, step: i64) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });
        let mut constants = BTreeMap::new();

        let b0 = bid("entry");
        let h = bid("header");
        let l = bid("latch");
        let e = bid("exit");

        let x = vid("x"); // header phi
        let xn = vid("xn"); // latch: x + step
        let cval = vid("c"); // guard compare result
        let x_init = vid("x_init");
        let step_v = vid("step");
        constants.insert(x_init, Constant::Int { value: 0, bits: 32 });
        constants.insert(
            step_v,
            Constant::Int {
                value: step,
                bits: 32,
            },
        );

        // Bound operand.
        let (bound_v, entry_extra): (ValueId, Vec<Instruction>) = match bound {
            Bound::Const(c) => {
                let bv = vid("bound_const");
                constants.insert(bv, Constant::Int { value: c, bits: 32 });
                (bv, vec![])
            }
            Bound::Nondet => {
                let bv = vid("n");
                // A loop-invariant nondet value defined in the entry block.
                let call = vinst(
                    "n_call",
                    Operation::CallDirect {
                        callee: FunctionId(make_id("func", b"__VERIFIER_nondet_int")),
                    },
                    bv,
                    vec![],
                    i32t,
                );
                (bv, vec![call])
            }
        };

        // entry: [nondet?] ; br header
        let mut entry = AirBlock::new(b0);
        entry.instructions.extend(entry_extra);
        entry
            .instructions
            .push(term("br_entry", Operation::Br { target: h }, vec![]));

        // header: phi x ; c = icmp cmp(x, bound) ; condbr c -> latch/exit
        let mut header = AirBlock::new(h);
        header.instructions.push(vinst(
            "phi_x",
            Operation::Phi {
                incoming: vec![(b0, x_init), (l, xn)],
            },
            x,
            vec![],
            i32t,
        ));
        header.instructions.push(vinst(
            "cmp",
            Operation::BinaryOp { kind: cmp },
            cval,
            vec![x, bound_v],
            i1t,
        ));
        header.instructions.push(term(
            "condbr",
            Operation::CondBr {
                then_target: l,
                else_target: e,
            },
            vec![cval],
        ));

        // latch: xn = add(x, step) ; br header
        let mut latch = AirBlock::new(l);
        latch.instructions.push(vinst(
            "add",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            xn,
            vec![x, step_v],
            i32t,
        ));
        latch
            .instructions
            .push(term("br_latch", Operation::Br { target: h }, vec![]));

        // exit: ret x
        let mut exit = AirBlock::new(e);
        exit.instructions.push(term("ret", Operation::Ret, vec![x]));

        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![entry, header, latch, exit],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        AirModule {
            id: ModuleId(make_id("module", b"t")),
            name: Some("t".to_string()),
            functions: vec![func],
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

    fn ranked(m: &AirModule) -> bool {
        let func = &m.functions[0];
        let cfg = Cfg::build(func);
        loops_are_ranked(func, m, &cfg)
    }

    #[test]
    fn count_down_unsigned_is_ranked() {
        // while (x > 0) x += -1;   (unsigned)  →  f = x
        let m = counter_loop(BinaryOp::ICmpUgt, Bound::Const(0), -1);
        assert!(ranked(&m));
    }

    #[test]
    fn count_up_strict_symbolic_bound_is_ranked() {
        // for (i = 0; i < n; i++)  (signed)  →  f = n - i.
        let m = counter_loop(BinaryOp::ICmpSlt, Bound::Nondet, 1);
        assert!(ranked(&m));
    }

    #[test]
    fn count_up_strict_const_bound_is_ranked() {
        // while (i < 100) i++  (signed)  →  f = 100 - i.
        let m = counter_loop(BinaryOp::ICmpSlt, Bound::Const(100), 1);
        assert!(ranked(&m));
    }

    #[test]
    fn count_down_to_const_signed_is_ranked() {
        // while (i > 5) i--  (signed)  →  f = i - 5.
        let m = counter_loop(BinaryOp::ICmpSgt, Bound::Const(5), -1);
        assert!(ranked(&m));
    }

    #[test]
    fn le_bound_abstains_due_to_intmax_overflow() {
        // for (i = 0; i <= n; i++): at i == n == INT_MAX the increment overflows,
        // so the loop can be non-terminating — the overflow check must reject it.
        let m = counter_loop(BinaryOp::ICmpSle, Bound::Nondet, 1);
        assert!(!ranked(&m));
    }

    #[test]
    fn ge_bound_abstains_due_to_intmin_overflow() {
        // while (i >= n) i-- : at i == n == INT_MIN the decrement underflows.
        let m = counter_loop(BinaryOp::ICmpSge, Bound::Nondet, -1);
        assert!(!ranked(&m));
    }

    #[test]
    fn wrong_direction_step_not_ranked() {
        // while (i < n) i-- : i moves away from the bound; no ranking function.
        let m = counter_loop(BinaryOp::ICmpSlt, Bound::Nondet, -1);
        assert!(!ranked(&m));
    }

    #[test]
    fn ne_guard_alone_not_ranked() {
        // while (i != n) i++ : `!=` is not a convex half-space; with only a type
        // bound the increment can overflow ⇒ abstain (sound).
        let m = counter_loop(BinaryOp::ICmpNe, Bound::Nondet, 1);
        assert!(!ranked(&m));
    }

    #[test]
    fn infinite_loop_no_exit_branch_abstains() {
        // A back-edge with no conditional exit (`while (1) x++;`): the increment
        // overflows the type range, so no ranking function exists.
        let i32t = tid("i32");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        let mut constants = BTreeMap::new();
        let b0 = bid("entry");
        let h = bid("header");
        let x = vid("x");
        let xn = vid("xn");
        let x_init = vid("x_init");
        let one = vid("one");
        constants.insert(x_init, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });

        let mut entry = AirBlock::new(b0);
        entry
            .instructions
            .push(term("br0", Operation::Br { target: h }, vec![]));
        let mut header = AirBlock::new(h);
        header.instructions.push(vinst(
            "phi_x",
            Operation::Phi {
                incoming: vec![(b0, x_init), (h, xn)],
            },
            x,
            vec![],
            i32t,
        ));
        header.instructions.push(vinst(
            "add",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            xn,
            vec![x, one],
            i32t,
        ));
        header
            .instructions
            .push(term("br1", Operation::Br { target: h }, vec![]));
        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![entry, header],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let m = AirModule {
            id: ModuleId(make_id("module", b"t")),
            name: Some("t".to_string()),
            functions: vec![func],
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants,
            types,
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };
        assert!(!ranked(&m));
    }

    #[test]
    fn two_back_edges_abstain() {
        // Two distinct back-edges to the same header ⇒ not a single natural loop.
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });
        let mut constants = BTreeMap::new();
        let b0 = bid("entry");
        let h = bid("header");
        let m1 = bid("mid1");
        let m2 = bid("mid2");
        let e = bid("exit");
        let x = vid("x");
        let cval = vid("c");
        let zero = vid("zero");
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });

        let mut entry = AirBlock::new(b0);
        entry
            .instructions
            .push(term("br0", Operation::Br { target: h }, vec![]));
        // header: phi ; c = icmp ; condbr -> mid1 / exit
        let mut header = AirBlock::new(h);
        header.instructions.push(vinst(
            "phi_x",
            Operation::Phi {
                incoming: vec![(b0, zero), (m1, x), (m2, x)],
            },
            x,
            vec![],
            i32t,
        ));
        header.instructions.push(vinst(
            "cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            cval,
            vec![x, zero],
            i1t,
        ));
        header.instructions.push(term(
            "condbr",
            Operation::CondBr {
                then_target: m1,
                else_target: e,
            },
            vec![cval],
        ));
        // mid1: condbr -> header / mid2   (back-edge 1: mid1 -> header)
        let mut mid1 = AirBlock::new(m1);
        mid1.instructions.push(term(
            "cb1",
            Operation::CondBr {
                then_target: h,
                else_target: m2,
            },
            vec![cval],
        ));
        // mid2: br header   (back-edge 2: mid2 -> header)
        let mut mid2 = AirBlock::new(m2);
        mid2.instructions
            .push(term("br2", Operation::Br { target: h }, vec![]));
        let mut exit = AirBlock::new(e);
        exit.instructions.push(term("ret", Operation::Ret, vec![x]));

        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![entry, header, mid1, mid2, exit],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let m = AirModule {
            id: ModuleId(make_id("module", b"t")),
            name: Some("t".to_string()),
            functions: vec![func],
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants,
            types,
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };
        assert!(!ranked(&m));
    }

    #[test]
    fn scaled_step_const_bound_is_ranked() {
        // while (i < 100) i += 2  (signed): f = 100 - i decreases by 2 ≥ 1, and
        // i + 2 ≤ 101 never overflows i32 ⇒ ranked.
        let m = counter_loop(BinaryOp::ICmpSlt, Bound::Const(100), 2);
        assert!(ranked(&m));
    }

    #[test]
    fn scaled_step_symbolic_bound_abstains() {
        // while (i < n) i += 2: at i == n − 1 == INT_MAX − 1 the increment can
        // overflow, so the non-unit step with a symbolic bound must abstain.
        let m = counter_loop(BinaryOp::ICmpSlt, Bound::Nondet, 2);
        assert!(!ranked(&m));
    }

    // --- Multi-loop decomposition -------------------------------------------

    /// Build an i32 count-up loop `for (v = 0; v < bound; v += step)` as a chain of
    /// four blocks `hdr → latch`, wired between `entry_pred` (fall-in) and `exit`
    /// (fall-out). Returns the header phi/latch block ids plus the four blocks and
    /// the constants they introduce. `tag` disambiguates value/block names so
    /// several such loops can coexist in one function.
    #[allow(clippy::too_many_arguments)]
    fn build_count_up(
        tag: &str,
        i32t: TypeId,
        i1t: TypeId,
        entry_pred: BlockId,
        exit: BlockId,
        bound: i64,
        step: i64,
        constants: &mut BTreeMap<ValueId, Constant>,
    ) -> (BlockId, Vec<AirBlock>) {
        let h = bid(&format!("h_{tag}"));
        let l = bid(&format!("l_{tag}"));
        let v = vid(&format!("v_{tag}"));
        let vn = vid(&format!("vn_{tag}"));
        let cval = vid(&format!("c_{tag}"));
        let v0 = vid(&format!("v0_{tag}"));
        let stepv = vid(&format!("step_{tag}"));
        let boundv = vid(&format!("bound_{tag}"));
        constants.insert(v0, Constant::Int { value: 0, bits: 32 });
        constants.insert(
            stepv,
            Constant::Int {
                value: step,
                bits: 32,
            },
        );
        constants.insert(
            boundv,
            Constant::Int {
                value: bound,
                bits: 32,
            },
        );

        let mut header = AirBlock::new(h);
        header.instructions.push(vinst(
            &format!("phi_{tag}"),
            Operation::Phi {
                incoming: vec![(entry_pred, v0), (l, vn)],
            },
            v,
            vec![],
            i32t,
        ));
        header.instructions.push(vinst(
            &format!("cmp_{tag}"),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            cval,
            vec![v, boundv],
            i1t,
        ));
        header.instructions.push(term(
            &format!("condbr_{tag}"),
            Operation::CondBr {
                then_target: l,
                else_target: exit,
            },
            vec![cval],
        ));

        let mut latch = AirBlock::new(l);
        latch.instructions.push(vinst(
            &format!("add_{tag}"),
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vn,
            vec![v, stepv],
            i32t,
        ));
        latch.instructions.push(term(
            &format!("br_{tag}"),
            Operation::Br { target: h },
            vec![],
        ));

        (h, vec![header, latch])
    }

    fn module_of(func: AirFunction, constants: BTreeMap<ValueId, Constant>) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });
        AirModule {
            id: ModuleId(make_id("module", b"t")),
            name: Some("t".to_string()),
            functions: vec![func],
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

    #[test]
    fn two_sequential_loops_both_ranked() {
        // for (x=0;x<10;x++){}  for (y=0;y<20;y++){}  — two distinct-header loops,
        // each individually ranked ⇒ the function terminates.
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let entry = bid("entry");
        let mid = bid("mid");
        let exit = bid("exit");

        let (h1, l1_blocks) = build_count_up("a", i32t, i1t, entry, mid, 10, 1, &mut constants);
        let (h2, l2_blocks) = build_count_up("b", i32t, i1t, mid, exit, 20, 1, &mut constants);

        let mut e = AirBlock::new(entry);
        e.instructions
            .push(term("br_e", Operation::Br { target: h1 }, vec![]));
        // `mid` is the fall-out of loop 1 and the pre-header of loop 2.
        let mut midb = AirBlock::new(mid);
        midb.instructions
            .push(term("br_mid", Operation::Br { target: h2 }, vec![]));
        let mut exitb = AirBlock::new(exit);
        exitb.instructions.push(term("ret", Operation::Ret, vec![]));

        let mut blocks = vec![e];
        blocks.extend(l1_blocks);
        blocks.push(midb);
        blocks.extend(l2_blocks);
        blocks.push(exitb);

        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        assert!(ranked(&module_of(func, constants)));
    }

    #[test]
    fn one_ranked_one_infinite_abstains() {
        // for (x=0;x<10;x++){}  while (1) z++;  — the second loop has no exit and
        // its counter overflows, so it is not ranked ⇒ the whole function abstains.
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let entry = bid("entry");
        let mid = bid("mid");
        let exit = bid("exit"); // unreachable, but keeps the loop1 exit target valid

        let (h1, l1_blocks) = build_count_up("a", i32t, i1t, entry, mid, 10, 1, &mut constants);

        let hz = bid("hz");
        let z = vid("z");
        let zn = vid("zn");
        let z0 = vid("z0");
        let one = vid("one");
        constants.insert(z0, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });

        let mut e = AirBlock::new(entry);
        e.instructions
            .push(term("br_e", Operation::Br { target: h1 }, vec![]));
        let mut midb = AirBlock::new(mid);
        midb.instructions
            .push(term("br_mid", Operation::Br { target: hz }, vec![]));
        // hz: phi z ; zn = z + 1 ; br hz   (infinite)
        let mut hzb = AirBlock::new(hz);
        hzb.instructions.push(vinst(
            "phi_z",
            Operation::Phi {
                incoming: vec![(mid, z0), (hz, zn)],
            },
            z,
            vec![],
            i32t,
        ));
        hzb.instructions.push(vinst(
            "add_z",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            zn,
            vec![z, one],
            i32t,
        ));
        hzb.instructions
            .push(term("br_z", Operation::Br { target: hz }, vec![]));
        let mut exitb = AirBlock::new(exit);
        exitb.instructions.push(term("ret", Operation::Ret, vec![]));

        let mut blocks = vec![e];
        blocks.extend(l1_blocks);
        blocks.push(midb);
        blocks.push(hzb);
        blocks.push(exitb);

        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        assert!(!ranked(&module_of(func, constants)));
    }

    #[test]
    fn nested_loops_both_ranked() {
        // for (i=0;i<10;i++) for (j=0;j<20;j++) {}  — properly nested; the inner
        // loop resets each outer iteration. Both natural loops rank independently
        // (outer models the inner as havoc) ⇒ terminates.
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let entry = bid("entry");
        let ho = bid("ho");
        let pre_i = bid("pre_i");
        let hi = bid("hi");
        let li = bid("li");
        let lo = bid("lo");
        let exit = bid("exit");

        let i = vid("i");
        let i_n = vid("i_n");
        let ci = vid("ci");
        let i0 = vid("i0");
        let one = vid("one");
        let no = vid("no");
        let j = vid("j");
        let j_n = vid("j_n");
        let cj = vid("cj");
        let j0 = vid("j0");
        let ni = vid("ni");
        constants.insert(i0, Constant::Int { value: 0, bits: 32 });
        constants.insert(j0, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });
        constants.insert(
            no,
            Constant::Int {
                value: 10,
                bits: 32,
            },
        );
        constants.insert(
            ni,
            Constant::Int {
                value: 20,
                bits: 32,
            },
        );

        let mut e = AirBlock::new(entry);
        e.instructions
            .push(term("br_e", Operation::Br { target: ho }, vec![]));

        // ho: phi i [(entry,0),(lo,i_n)] ; ci = i < 10 ; condbr -> pre_i / exit
        let mut hob = AirBlock::new(ho);
        hob.instructions.push(vinst(
            "phi_i",
            Operation::Phi {
                incoming: vec![(entry, i0), (lo, i_n)],
            },
            i,
            vec![],
            i32t,
        ));
        hob.instructions.push(vinst(
            "cmp_i",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            ci,
            vec![i, no],
            i1t,
        ));
        hob.instructions.push(term(
            "condbr_i",
            Operation::CondBr {
                then_target: pre_i,
                else_target: exit,
            },
            vec![ci],
        ));

        // pre_i: br hi   (inner pre-header, inside the outer body)
        let mut pib = AirBlock::new(pre_i);
        pib.instructions
            .push(term("br_pi", Operation::Br { target: hi }, vec![]));

        // hi: phi j [(pre_i,0),(li,j_n)] ; cj = j < 20 ; condbr -> li / lo
        let mut hib = AirBlock::new(hi);
        hib.instructions.push(vinst(
            "phi_j",
            Operation::Phi {
                incoming: vec![(pre_i, j0), (li, j_n)],
            },
            j,
            vec![],
            i32t,
        ));
        hib.instructions.push(vinst(
            "cmp_j",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            cj,
            vec![j, ni],
            i1t,
        ));
        hib.instructions.push(term(
            "condbr_j",
            Operation::CondBr {
                then_target: li,
                else_target: lo,
            },
            vec![cj],
        ));

        // li: j_n = j + 1 ; br hi
        let mut lib = AirBlock::new(li);
        lib.instructions.push(vinst(
            "add_j",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            j_n,
            vec![j, one],
            i32t,
        ));
        lib.instructions
            .push(term("br_li", Operation::Br { target: hi }, vec![]));

        // lo: i_n = i + 1 ; br ho
        let mut lob = AirBlock::new(lo);
        lob.instructions.push(vinst(
            "add_i",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            i_n,
            vec![i, one],
            i32t,
        ));
        lob.instructions
            .push(term("br_lo", Operation::Br { target: ho }, vec![]));

        let mut exitb = AirBlock::new(exit);
        exitb.instructions.push(term("ret", Operation::Ret, vec![]));

        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![e, hob, pib, hib, lib, lob, exitb],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        assert!(ranked(&module_of(func, constants)));
    }
}
