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
use saf_core::id::make_id;
use saf_core::ids::{BlockId, TypeId, ValueId};

use crate::fast_paths::cfg_has_loops;

/// Maximum recursion depth when expanding an SSA value into an affine form.
/// Bounds work on pathological IR; exceeding it ⇒ treat the value as opaque.
const AFFINE_DEPTH_LIMIT: u32 = 64;

/// Cap on the number of enumerated header→latch paths of one loop body. A body
/// with more control-flow paths than this is not modelled path-sensitively
/// (⇒ abstain / fall back to the single-transition havoc model) — this bounds the
/// per-loop Z3 work and keeps the pass well within the SV-COMP time budget.
const MAX_PATHS: usize = 8;

/// Cap on the lexicographic ranking depth (number of greedy rounds). Real
/// programs almost never need a deeper lexicographic tuple; exceeding it abstains.
const MAX_LEX_ROUNDS: usize = 4;

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
    // Every natural loop must be ranked; any failure (unmodelable loop, or Z3
    // `Unsat`/`Unknown`) abstains the whole function.
    loops.iter().all(|li| loop_is_ranked(func, module, cfg, li))
}

/// Rank a single natural loop.
///
/// First tries the **path-sensitive** model — enumerate every header→latch path,
/// derive each path's guard + affine transition, and synthesize a (possibly
/// **lexicographic**) linear ranking function that is bounded and decreasing on
/// every path. This proves multi-path loops (`if (…) i++; else i+=2;`) and
/// phase/lexicographic loops (`while (x>=0 && y>=0){ y--; if (y<0){ x--; y=*; } }`)
/// that a single conjunctive transition cannot.
///
/// If the path-sensitive model does not apply (a **nested** inner loop in the
/// body, too many paths) or fails to rank, falls back to the single-transition
/// **havoc** model ([`build_loop_model`] + [`synthesize_ranking_function`]), which
/// models an inner loop's effect as havoc and handles the common counter loops.
/// Both are complete, sound termination proofs (Farkas over a superset relation),
/// so a `true` here never ranks a non-terminating loop.
fn loop_is_ranked(func: &AirFunction, module: &AirModule, cfg: &Cfg, li: &LoopInfo) -> bool {
    multipath_ranked(func, module, cfg, li)
        || build_loop_model(func, module, li)
            .is_some_and(|model| synthesize_ranking_function(&model))
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
    bounds_from_type(sym, type_id, signs, module)
}

/// Integer range `[lo, hi]` for `sym` given an explicit `type_id` and its inferred
/// signedness. Used for values whose type is not on a [`Def`] record — notably
/// **function parameters** (their `TypeId` lives on [`AirParam`], not on any
/// instruction) in the recursion model. `None` ⇒ no representable bound (kept free).
fn bounds_from_type(
    sym: ValueId,
    type_id: TypeId,
    signs: &BTreeMap<ValueId, Sign>,
    module: &AirModule,
) -> Option<(i128, i128)> {
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
// Path-sensitive (disjunctive + lexicographic) ranking
// ---------------------------------------------------------------------------
//
// A loop body with internal branches has *several* continuing transitions (one
// per header→latch path). The single-transition model above resolves only the
// back-edge phi value, which for a multi-path loop is a merge (⇒ opaque) — so it
// abstains. Here we enumerate every path, build a per-path guard + affine
// transition, and synthesize a **lexicographic** linear ranking function
// (Podelski–Rybalchenko transition invariants; Cook/Bradley–Manna–Sipma greedy
// lexicographic synthesis): a tuple `(f₁,…,f_k)` such that every path strictly
// decreases some `f_i` while all `f_j` (j<i) do not increase, each `f_i` bounded
// below where it decides. A `k=1` tuple is the ordinary single linear function.
//
// # Soundness
//
// Identical to the single model: each path's transition is modelled by an affine
// relation over a region that is a **superset** of the real continuing states —
// guards are only-ever-necessary conditions (anything non-affine is dropped,
// weakening the region), each affine `next` is overflow-checked to equal the
// machine update (else demoted to a free in-range **havoc** symbol), and all
// other values are universally quantified. Because we enumerate **every** path of
// an acyclic body (bailing to the havoc model on any nested inner loop, and
// abstaining when the path count exceeds [`MAX_PATHS`]), the union of modelled
// transitions contains the real loop transition. A lexicographic ranking of a
// superset ranks the real loop, so termination follows and no wrong `true` is
// possible. Each greedy round's `f_r` is bounded-below and non-increasing on all
// branches still present, and strictly decreasing on the one it removes, which is
// exactly the lexicographic-ranking soundness condition.

/// A path's raw guards + un-classified affine transitions, before overflow
/// classification and universe/type-bound assembly (a build-time intermediate).
struct RawBranch {
    guards: Vec<Constraint>,
    raw_next: BTreeMap<ValueId, Option<Affine>>,
}

/// One continuing transition of a loop: the path's guard region plus, for every
/// header phi, the affine value it takes at the latch along this path (an
/// overflow-safe affine form, or a free in-range havoc symbol).
struct Branch {
    /// Type bounds + this path's (necessary) guard constraints, each `expr ≥ 0`.
    region: Vec<Constraint>,
    /// header phi ↦ its affine `next` on this path (havoc phis map to a fresh
    /// free symbol that is type-bounded in `region`).
    next: BTreeMap<ValueId, Affine>,
}

/// The path-sensitive model of one natural loop: the ranking template (symbols
/// eligible for a coefficient), the full leaf universe, and one [`Branch`] per
/// enumerated header→latch path.
struct MultiPathModel {
    /// Symbols that may carry a ranking coefficient: header phis ∪ loop-invariant
    /// parameters.
    template: BTreeSet<ValueId>,
    /// Every leaf symbol appearing anywhere in the model (for the Farkas identity
    /// and the Z3 variable index).
    universe: BTreeSet<ValueId>,
    branches: Vec<Branch>,
}

/// Try to prove `li` terminates path-sensitively. Returns `false` (⇒ fall back to
/// the havoc model) when the model does not apply (nested inner loop, too many
/// paths, no affine phi) or no lexicographic ranking is found.
fn multipath_ranked(func: &AirFunction, module: &AirModule, cfg: &Cfg, li: &LoopInfo) -> bool {
    match build_multipath_model(func, module, cfg, li) {
        Some(model) => greedy_lex_rank(&model),
        None => false,
    }
}

/// Greedy lexicographic synthesis over a [`MultiPathModel`]: each round find one
/// `f` that is bounded (`f ≥ 0`) and non-increasing (`f − f′ ≥ 0`) on every
/// remaining branch and strictly decreasing (`f − f′ ≥ 1`) on one of them; remove
/// that branch. All branches removed within [`MAX_LEX_ROUNDS`] ⇒ the branches
/// jointly admit a lexicographic ranking tuple (⇒ termination). Shared by the
/// loop path-sensitive model and the recursion model — a `MultiPathModel` only
/// records per-transition guards + affine `next`, so the same driver ranks a
/// loop's back-edge transitions or a self-recursive function's call-site
/// transitions identically.
fn greedy_lex_rank(model: &MultiPathModel) -> bool {
    if model.branches.is_empty() {
        return false;
    }
    let mut remaining: Vec<usize> = (0..model.branches.len()).collect();
    let mut rounds = 0;
    while !remaining.is_empty() {
        if rounds >= MAX_LEX_ROUNDS {
            return false;
        }
        let mut removed_pos = None;
        for (pos, &strict) in remaining.iter().enumerate() {
            if synthesize_round(model, &remaining, strict) {
                removed_pos = Some(pos);
                break;
            }
        }
        match removed_pos {
            Some(pos) => {
                remaining.remove(pos);
                rounds += 1;
            }
            None => return false,
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Recursion ranking (self-recursive functions)
// ---------------------------------------------------------------------------

/// Does the **self-recursive** function `func` admit a linear ranking function
/// over its integer parameters that bounds the recursion depth (⇒ the recursion
/// terminates)?
///
/// The recursion is modelled as a [`MultiPathModel`] exactly like a loop's
/// back-edge transitions, but the ranking *state* is the function's integer
/// parameters and each transition is one **entry → recursive-call-site path**:
/// its guard is the conjunction of branch conditions along the path (a necessary
/// condition on any frame that reaches that call — hence an over-approximation),
/// and its `next` maps each parameter to the affine form of the corresponding
/// call argument (or a type-bounded havoc symbol when the argument is not exactly
/// affine). A ranking `f` that is `≥ 0` on every call's guard and strictly
/// decreases by `≥ 1` from parameters to arguments bounds every root-to-leaf
/// chain of recursive calls; with a finite number of call sites the whole call
/// tree is finite ⇒ the recursion terminates. This is a **complete, sound**
/// proof (Farkas over a superset relation, `Sat`-only), so it never yields a
/// wrong `true`.
///
/// Restricted to a **loop-free** body (so entry→call paths are finite and
/// enumerable); the caller already requires each reachable function's loops to be
/// independently ranked. Abstains (`false` ⇒ `unknown`) on anything else.
///
/// Precondition: `func` is a defined function that directly calls itself (the
/// caller identified it as a size-1 self-recursive call-graph SCC).
#[must_use]
pub fn recursion_is_ranked(func: &AirFunction, module: &AirModule, cfg: &Cfg) -> bool {
    match build_recursion_model(func, module, cfg) {
        Some(model) => greedy_lex_rank(&model),
        None => false,
    }
}

/// Build the recursion [`MultiPathModel`] for a self-recursive function, or `None`
/// (⇒ abstain) when the shape is unsupported (looping body, no integer parameter,
/// too many entry→call paths, or a non-affine ranking-relevant argument).
// NOTE: one cohesive extraction pipeline (reject loops → collect params →
// enumerate entry→call paths → per-path guards/transitions → classify overflow →
// assemble); splitting it would only scatter the shared `defs`/`universe`/bounds
// state, mirroring `build_multipath_model`.
#[allow(clippy::too_many_lines)]
fn build_recursion_model(
    func: &AirFunction,
    module: &AirModule,
    cfg: &Cfg,
) -> Option<MultiPathModel> {
    // A looping body would make entry→call path enumeration incomplete (an
    // unbounded loop between entry and the recursive call is not path-enumerable);
    // abstain and let the loop-ranking gate handle loops separately.
    if cfg_has_loops(cfg) {
        return None;
    }

    let signs = infer_signs(func);

    // Ranking state = the function's integer parameters (their `TypeId` lives on
    // `AirParam`, not on any instruction). Non-integer params (pointers, floats)
    // cannot carry a ranking coefficient and are treated as opaque leaves.
    let mut param_ids: BTreeSet<ValueId> = BTreeSet::new();
    let mut param_type_of: BTreeMap<ValueId, TypeId> = BTreeMap::new();
    let mut ordered: Vec<&saf_core::air::AirParam> = func.params.iter().collect();
    ordered.sort_by_key(|p| p.index);
    for p in &ordered {
        if let Some(ty) = p.param_type {
            if int_width(module, ty).is_some() {
                param_ids.insert(p.id);
                param_type_of.insert(p.id, ty);
            }
        }
    }
    if param_ids.is_empty() {
        return None;
    }

    // The whole (loop-free) body is the affine "region": every def is expandable,
    // and the parameters are the leaves (played by `header_phis` in the shared
    // helpers, which treat that set as un-expanded ranking leaves).
    let all_blocks: BTreeSet<BlockId> = func.blocks.iter().map(|b| b.id).collect();
    let defs = index_defs(func, &all_blocks);
    let block_of = index_block_of(func);
    let header_phis = param_ids.clone();

    // Recursive call sites: a `CallDirect` to `func` itself. Operands are the call
    // arguments (positional, matching `AirParam::index`).
    let mut sites: Vec<(BlockId, Vec<ValueId>)> = Vec::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::CallDirect { callee } = &inst.op {
                if *callee == func.id {
                    sites.push((block.id, inst.operands.clone()));
                }
            }
        }
    }
    if sites.is_empty() {
        return None;
    }

    let entry = func.entry_block.unwrap_or(func.blocks.first()?.id);

    // --- Pass 1: one raw transition per (call site × entry→site path). ---------
    let mut raws: Vec<RawBranch> = Vec::new();
    let mut universe: BTreeSet<ValueId> = param_ids.clone();
    for (cb, args) in &sites {
        // Every simple path from the entry to the call-site block (the body minus
        // no back-edges is already acyclic — we rejected loops above).
        let paths = enumerate_body_paths(cfg, &all_blocks, entry, *cb)?;
        for path in &paths {
            let path_pred = path_pred_map(path);
            let mut guards: Vec<Constraint> = Vec::new();
            // Necessary stay-conditions of the branches entering the call block.
            for pair in path.windows(2) {
                if let Some(block) = func.blocks.iter().find(|b| b.id == pair[0]) {
                    add_path_guards(
                        &mut guards,
                        &mut universe,
                        block,
                        pair[1],
                        &defs,
                        &block_of,
                        &header_phis,
                        module,
                        &path_pred,
                    );
                }
            }
            // Transition: parameter i ↦ affine form of argument i on this path.
            let mut raw_next: BTreeMap<ValueId, Option<Affine>> = BTreeMap::new();
            for p in &ordered {
                if !param_ids.contains(&p.id) {
                    continue;
                }
                let a = args.get(p.index as usize).and_then(|&arg| {
                    resolve_affine_path(arg, &defs, &block_of, &header_phis, module, &path_pred, 0)
                });
                if let Some(ref aff) = a {
                    universe.extend(aff.terms.keys().copied());
                }
                raw_next.insert(p.id, a);
            }
            raws.push(RawBranch { guards, raw_next });
        }
    }
    if raws.is_empty() {
        return None;
    }

    // Invariant params: leaf symbols that are neither ranking state nor defined in
    // the function (globals, opaque leaves) — eligible for a ranking coefficient.
    let mut invariants: BTreeSet<ValueId> = BTreeSet::new();
    for &sym in &universe {
        if !param_ids.contains(&sym) && !defs.contains_key(&sym) {
            invariants.insert(sym);
        }
    }

    // Type bounds: parameters use their `AirParam` type; every other integer leaf
    // uses its def's result type. True facts about the real values, shared by
    // every branch region.
    let mut base_bounds: Vec<Constraint> = Vec::new();
    let mut bound_of: BTreeMap<ValueId, (i128, i128)> = BTreeMap::new();
    for &sym in &universe {
        let bound = match param_type_of.get(&sym) {
            Some(&ty) => bounds_from_type(sym, ty, &signs, module),
            None => type_bounds(sym, &defs, &signs, module),
        };
        if let Some((lo, hi)) = bound {
            bound_of.insert(sym, (lo, hi));
            base_bounds.push(Constraint {
                coeffs: BTreeMap::from([(sym, 1)]),
                constant: -lo,
            });
            base_bounds.push(Constraint {
                coeffs: BTreeMap::from([(sym, -1)]),
                constant: hi,
            });
        }
    }

    // --- Pass 2: classify each argument transition (affine-stable vs havoc). ----
    let mut fresh_syms: BTreeSet<ValueId> = BTreeSet::new();
    let mut branches: Vec<Branch> = Vec::with_capacity(raws.len());
    for (bi, raw) in raws.into_iter().enumerate() {
        let mut check_region = raw.guards;
        check_region.extend(base_bounds.iter().cloned());

        let mut next: BTreeMap<ValueId, Affine> = BTreeMap::new();
        let mut fresh_bounds: Vec<Constraint> = Vec::new();
        for (&param, cand) in &raw.raw_next {
            // Stable iff the affine argument provably never wraps its type range on
            // the region (⇒ affine model == machine value). Otherwise havoc: a
            // fresh type-bounded free symbol (ranking must hold for every value).
            let stable = match (cand, bound_of.get(&param)) {
                (Some(a), Some(&(lo, hi)))
                    if next_never_overflows(a, lo, hi, &check_region, &universe) =>
                {
                    Some(a.clone())
                }
                _ => None,
            };
            if let Some(a) = stable {
                next.insert(param, a);
            } else {
                let fresh = havoc_id(param, bi);
                fresh_syms.insert(fresh);
                if let Some(&(lo, hi)) = bound_of.get(&param) {
                    fresh_bounds.push(Constraint {
                        coeffs: BTreeMap::from([(fresh, 1)]),
                        constant: -lo,
                    });
                    fresh_bounds.push(Constraint {
                        coeffs: BTreeMap::from([(fresh, -1)]),
                        constant: hi,
                    });
                }
                next.insert(param, Affine::symbol(fresh));
            }
        }
        check_region.extend(fresh_bounds);
        branches.push(Branch {
            region: check_region,
            next,
        });
    }
    universe.extend(fresh_syms);

    let template: BTreeSet<ValueId> = param_ids.iter().chain(invariants.iter()).copied().collect();
    Some(MultiPathModel {
        template,
        universe,
        branches,
    })
}

/// Build the path-sensitive [`MultiPathModel`], or `None` (⇒ abstain / fall back)
/// when the loop is not in the supported path-sensitive shape.
// NOTE: this is one cohesive extraction pipeline (reject-nested → collect phis →
// enumerate paths → per-path guards/transitions → classify overflow → assemble);
// splitting it would only scatter the shared `defs`/`universe`/`bound_of` state.
#[allow(clippy::too_many_lines)]
fn build_multipath_model(
    func: &AirFunction,
    module: &AirModule,
    cfg: &Cfg,
    li: &LoopInfo,
) -> Option<MultiPathModel> {
    let defs = index_defs(func, &li.body);
    let block_of = index_block_of(func);
    let signs = infer_signs(func);

    // Reject a **nested inner loop**: the only dominance back-edge inside the body
    // may be this loop's own `latch → header`. Any other in-body back-edge is an
    // inner loop whose many iterations path enumeration would miss (unsound) — bail
    // to the single-transition havoc model, which over-approximates it as havoc.
    // (The function is reducible — `extract_natural_loops` rejected irreducible
    // CFGs — so every in-body cycle has a dominance back-edge and is caught here.)
    for (&u, succs) in &cfg.successors {
        if !li.body.contains(&u) {
            continue;
        }
        for &v in succs {
            if li.body.contains(&v)
                && dominates(v, u, &li.idom)
                && !(u == li.latch && v == li.header)
            {
                return None;
            }
        }
    }

    // Header phis and their back-edge (latch) incoming value.
    let header_block = func.blocks.iter().find(|b| b.id == li.header)?;
    let mut phi_next: BTreeMap<ValueId, ValueId> = BTreeMap::new();
    for inst in &header_block.instructions {
        if let Operation::Phi { incoming } = &inst.op {
            let Some(dst) = inst.dst else { continue };
            if let Some((_, v)) = incoming.iter().find(|(pred, _)| *pred == li.latch) {
                phi_next.insert(dst, *v);
            }
        }
    }
    if phi_next.is_empty() {
        return None;
    }
    let header_phis: BTreeSet<ValueId> = phi_next.keys().copied().collect();

    // Enumerate every header→latch path of the (acyclic) body.
    let paths = enumerate_body_paths(cfg, &li.body, li.header, li.latch)?;
    let latch_block = func.blocks.iter().find(|b| b.id == li.latch)?;

    // --- Pass 1: per-path guards + raw affine transitions; collect the universe.
    let mut raws: Vec<RawBranch> = Vec::with_capacity(paths.len());
    let mut universe: BTreeSet<ValueId> = header_phis.clone();
    for path in &paths {
        let path_pred = path_pred_map(path);
        let mut guards: Vec<Constraint> = Vec::new();
        // Necessary stay-conditions along the path's internal/exit branches …
        for pair in path.windows(2) {
            if let Some(block) = func.blocks.iter().find(|b| b.id == pair[0]) {
                add_path_guards(
                    &mut guards,
                    &mut universe,
                    block,
                    pair[1],
                    &defs,
                    &block_of,
                    &header_phis,
                    module,
                    &path_pred,
                );
            }
        }
        // … plus the back-edge branch itself (`latch → header`): in a `do…while`
        // the continuing condition lives here.
        add_path_guards(
            &mut guards,
            &mut universe,
            latch_block,
            li.header,
            &defs,
            &block_of,
            &header_phis,
            module,
            &path_pred,
        );

        let mut raw_next: BTreeMap<ValueId, Option<Affine>> = BTreeMap::new();
        for (&phi, &nv) in &phi_next {
            let a = resolve_affine_path(nv, &defs, &block_of, &header_phis, module, &path_pred, 0);
            if let Some(ref aff) = a {
                universe.extend(aff.terms.keys().copied());
            }
            raw_next.insert(phi, a);
        }
        raws.push(RawBranch { guards, raw_next });
    }

    // Invariant params: leaf symbols that are neither header phis nor defined
    // inside the loop (function params, nondet results, globals) — eligible for a
    // ranking coefficient (identity transition ⇒ they only bound `f` below).
    let mut invariants: BTreeSet<ValueId> = BTreeSet::new();
    for &sym in &universe {
        if !header_phis.contains(&sym) && !defs.get(&sym).is_some_and(|d| d.in_loop) {
            invariants.insert(sym);
        }
    }

    // Type bounds for every integer leaf with a known signedness/width — true facts
    // about the real state, shared by every branch region.
    let mut base_bounds: Vec<Constraint> = Vec::new();
    let mut bound_of: BTreeMap<ValueId, (i128, i128)> = BTreeMap::new();
    for &sym in &universe {
        if let Some((lo, hi)) = type_bounds(sym, &defs, &signs, module) {
            bound_of.insert(sym, (lo, hi));
            base_bounds.push(Constraint {
                coeffs: BTreeMap::from([(sym, 1)]),
                constant: -lo,
            });
            base_bounds.push(Constraint {
                coeffs: BTreeMap::from([(sym, -1)]),
                constant: hi,
            });
        }
    }

    // --- Pass 2: classify each transition (affine-stable vs havoc) and assemble
    // the final per-branch regions.
    let mut fresh_syms: BTreeSet<ValueId> = BTreeSet::new();
    let mut branches: Vec<Branch> = Vec::with_capacity(raws.len());
    for (bi, raw) in raws.into_iter().enumerate() {
        // Region used to *check* overflow-freedom (guards + type bounds only).
        let mut check_region = raw.guards;
        check_region.extend(base_bounds.iter().cloned());

        let mut next: BTreeMap<ValueId, Affine> = BTreeMap::new();
        let mut fresh_bounds: Vec<Constraint> = Vec::new();
        for (&phi, cand) in &raw.raw_next {
            // A phi is stable on this path iff its affine `next` provably never
            // wraps its type range on the path region (⇒ affine model == machine
            // update). Otherwise it is havoc: a fresh free symbol, type-bounded, so
            // the ranking must hold for *every* in-range successor value.
            let stable = match (cand, bound_of.get(&phi)) {
                (Some(a), Some(&(lo, hi)))
                    if next_never_overflows(a, lo, hi, &check_region, &universe) =>
                {
                    Some(a.clone())
                }
                _ => None,
            };
            if let Some(a) = stable {
                next.insert(phi, a);
            } else {
                let fresh = havoc_id(phi, bi);
                fresh_syms.insert(fresh);
                if let Some(&(lo, hi)) = bound_of.get(&phi) {
                    fresh_bounds.push(Constraint {
                        coeffs: BTreeMap::from([(fresh, 1)]),
                        constant: -lo,
                    });
                    fresh_bounds.push(Constraint {
                        coeffs: BTreeMap::from([(fresh, -1)]),
                        constant: hi,
                    });
                }
                next.insert(phi, Affine::symbol(fresh));
            }
        }
        check_region.extend(fresh_bounds);
        branches.push(Branch {
            region: check_region,
            next,
        });
    }
    universe.extend(fresh_syms);

    let template: BTreeSet<ValueId> = header_phis
        .iter()
        .chain(invariants.iter())
        .copied()
        .collect();
    Some(MultiPathModel {
        template,
        universe,
        branches,
    })
}

/// Build `ValueId → defining block` for path-sensitive phi resolution.
fn index_block_of(func: &AirFunction) -> BTreeMap<ValueId, BlockId> {
    let mut out = BTreeMap::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                out.insert(dst, block.id);
            }
        }
    }
    out
}

/// A deterministic fresh **havoc** symbol for header phi `phi` on branch `bi`
/// (used when the phi's `next` on that path is non-affine or may overflow).
fn havoc_id(phi: ValueId, bi: usize) -> ValueId {
    let mut bytes = format!("{phi:?}").into_bytes();
    bytes.extend_from_slice(&bi.to_le_bytes());
    ValueId(make_id("ranking_havoc", &bytes))
}

/// `block ↦ its predecessor on this path` (the header, at index 0, has none).
fn path_pred_map(path: &[BlockId]) -> BTreeMap<BlockId, BlockId> {
    let mut out = BTreeMap::new();
    for pair in path.windows(2) {
        out.insert(pair[1], pair[0]);
    }
    out
}

/// Enumerate **every** simple path from `header` to `latch` through the loop body,
/// excluding the `latch → header` back-edge (a continuing transition ends *at* the
/// latch, about to take the back-edge). The body minus that back-edge is acyclic
/// (the caller rejected nested loops), so simple-path enumeration is complete.
/// Returns `None` if the path count would exceed [`MAX_PATHS`] (⇒ abstain).
fn enumerate_body_paths(
    cfg: &Cfg,
    body: &BTreeSet<BlockId>,
    header: BlockId,
    latch: BlockId,
) -> Option<Vec<Vec<BlockId>>> {
    if header == latch {
        // A single-block loop: the header *is* the latch; its self back-edge's
        // guard is captured separately from the `latch → header` branch.
        return Some(vec![vec![header]]);
    }
    let mut out: Vec<Vec<BlockId>> = Vec::new();
    let mut path = vec![header];
    let mut on_path: BTreeSet<BlockId> = BTreeSet::from([header]);
    if dfs_body_paths(header, cfg, body, latch, &mut path, &mut on_path, &mut out) {
        Some(out)
    } else {
        None
    }
}

/// DFS helper for [`enumerate_body_paths`]; returns `false` once the [`MAX_PATHS`]
/// cap is exceeded (deterministic: successors iterate in sorted `BTreeSet` order).
fn dfs_body_paths(
    node: BlockId,
    cfg: &Cfg,
    body: &BTreeSet<BlockId>,
    latch: BlockId,
    path: &mut Vec<BlockId>,
    on_path: &mut BTreeSet<BlockId>,
    out: &mut Vec<Vec<BlockId>>,
) -> bool {
    if node == latch {
        out.push(path.clone());
        return out.len() <= MAX_PATHS;
    }
    if let Some(succs) = cfg.successors.get(&node) {
        for &s in succs {
            if !body.contains(&s) || on_path.contains(&s) {
                continue;
            }
            path.push(s);
            on_path.insert(s);
            let ok = dfs_body_paths(s, cfg, body, latch, path, on_path, out);
            path.pop();
            on_path.remove(&s);
            if !ok {
                return false;
            }
        }
    }
    true
}

/// Add the necessary stay-condition imposed by taking the `block → next_block`
/// edge (a `CondBr` in the taken polarity), path-sensitively resolved. Non-affine
/// conditions and non-`CondBr` terminators contribute nothing (a sound weakening).
#[allow(clippy::too_many_arguments)]
fn add_path_guards(
    region: &mut Vec<Constraint>,
    universe: &mut BTreeSet<ValueId>,
    block: &saf_core::air::AirBlock,
    next_block: BlockId,
    defs: &BTreeMap<ValueId, Def>,
    block_of: &BTreeMap<ValueId, BlockId>,
    header_phis: &BTreeSet<ValueId>,
    module: &AirModule,
    path_pred: &BTreeMap<BlockId, BlockId>,
) {
    let Some(term) = block.terminator() else {
        return;
    };
    let Operation::CondBr {
        then_target,
        else_target,
    } = &term.op
    else {
        return;
    };
    let taken_true = if next_block == *then_target {
        true
    } else if next_block == *else_target {
        false
    } else {
        return;
    };
    let Some(&cond) = term.operands.first() else {
        return;
    };
    resolve_bool_guard(
        region,
        universe,
        cond,
        taken_true,
        defs,
        block_of,
        header_phis,
        module,
        path_pred,
        0,
    );
}

/// Emit into `region` the affine constraints implied by boolean value `cond`
/// holding polarity `want_true` along the current path.
///
/// The direct case is a comparison ([`comparison_constraints`]). But a
/// `while (A && B)` / `while (A || B)` header condition is *not* a single
/// comparison: clang short-circuits it into an `i1` **phi** (`%c = phi [false, …],
/// [%b, …]`), and a `!x` guard into `xor %x, true`. Without following these, the
/// second conjunct's guard is silently dropped, so the loop's true bound is
/// invisible and an otherwise-rankable loop abstains. This resolver follows, along
/// the *current path*:
///
/// - an **`i1` phi** to its incoming value from the path predecessor of the phi's
///   block (the concrete branch taken here), same polarity;
/// - a logical **`and`** taken *true* ⇒ both operands are true; a logical **`or`**
///   taken *false* ⇒ both operands are false (the only convex, sound directions —
///   the other direction is dropped);
/// - a **`xor c, 1`** negation ⇒ recurse on the other operand with flipped
///   polarity;
/// - a **copy / freeze / integer cast** ⇒ recurse on the source, same polarity.
///
/// Every emitted constraint is a *necessary* condition of the branch being taken,
/// so `region` stays a sound over-approximation (superset) of the reachable
/// continuing states — adding it only shrinks the region, never drops a real
/// transition. A boolean **constant** that contradicts `want_true` proves this
/// path is infeasible; we record an unsatisfiable `−1 ≥ 0` so the branch region is
/// empty and vacuously ranked (a `while (A && B)` body entered from the `A`-false
/// side is such an infeasible path).
// NOTE: one cohesive boolean-guard resolution (constant → comparison → phi →
// and/or → xor → carrier); splitting the match arms into helpers would only
// scatter the shared `recurse` closure and the `defs`/`path_pred` context.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn resolve_bool_guard(
    region: &mut Vec<Constraint>,
    universe: &mut BTreeSet<ValueId>,
    cond: ValueId,
    want_true: bool,
    defs: &BTreeMap<ValueId, Def>,
    block_of: &BTreeMap<ValueId, BlockId>,
    header_phis: &BTreeSet<ValueId>,
    module: &AirModule,
    path_pred: &BTreeMap<BlockId, BlockId>,
    depth: u32,
) {
    if depth > AFFINE_DEPTH_LIMIT {
        return;
    }
    // A boolean constant: consistent with `want_true` ⇒ vacuous (no constraint);
    // contradictory ⇒ this path is infeasible, so make the region unsatisfiable.
    if let Some(Constant::Int { value, .. }) = module.constants.get(&cond) {
        if (*value != 0) != want_true {
            region.push(Constraint {
                coeffs: BTreeMap::new(),
                constant: -1,
            });
        }
        return;
    }
    let Some(def) = defs.get(&cond) else {
        return; // opaque leaf (param / nondet / global) ⇒ no usable constraint.
    };
    let recurse =
        |region: &mut Vec<Constraint>, universe: &mut BTreeSet<ValueId>, v: ValueId, want: bool| {
            resolve_bool_guard(
                region,
                universe,
                v,
                want,
                defs,
                block_of,
                header_phis,
                module,
                path_pred,
                depth + 1,
            );
        };
    match &def.op {
        // Direct integer comparison — the base case.
        Operation::BinaryOp { kind } if is_int_comparison(*kind) => {
            let (Some(&lo), Some(&ro)) = (def.operands.first(), def.operands.get(1)) else {
                return;
            };
            let (Some(lhs), Some(rhs)) = (
                resolve_affine_path(lo, defs, block_of, header_phis, module, path_pred, 0),
                resolve_affine_path(ro, defs, block_of, header_phis, module, path_pred, 0),
            ) else {
                return;
            };
            if let Some(cons) = comparison_constraints(*kind, &lhs, &rhs, want_true) {
                for c in cons {
                    universe.extend(c.coeffs.keys().copied());
                    region.push(c);
                }
            }
        }
        // `i1` short-circuit phi (the `&&` / `||` lowering): resolve to the value
        // incoming from this path's predecessor of the phi's block, same polarity.
        Operation::Phi { incoming } => {
            let Some(blk) = block_of.get(&cond) else {
                return;
            };
            let Some(pred) = path_pred.get(blk) else {
                return;
            };
            if let Some((_, val)) = incoming.iter().find(|(p, _)| p == pred) {
                recurse(region, universe, *val, want_true);
            }
        }
        // Logical `and` taken true ⇒ both conjuncts true; logical `or` taken false
        // ⇒ both disjuncts false. The opposite directions are non-convex (drop).
        Operation::BinaryOp {
            kind: BinaryOp::And,
        } if want_true => {
            for &op in def.operands.iter().take(2) {
                recurse(region, universe, op, true);
            }
        }
        Operation::BinaryOp { kind: BinaryOp::Or } if !want_true => {
            for &op in def.operands.iter().take(2) {
                recurse(region, universe, op, false);
            }
        }
        // `xor v, 1` is `!v` on an `i1`; recurse on `v` with flipped polarity.
        Operation::BinaryOp {
            kind: BinaryOp::Xor,
        } => {
            let a = def.operands.first().copied();
            let b = def.operands.get(1).copied();
            let const_bit = |v: Option<ValueId>| {
                v.and_then(|v| match module.constants.get(&v) {
                    Some(Constant::Int { value, .. }) => Some(*value & 1 == 1),
                    _ => None,
                })
            };
            match (const_bit(a), const_bit(b)) {
                // `xor v, true` ⇒ negate `v`; `xor v, false` ⇒ passthrough.
                (Some(bit), _) => {
                    if let Some(v) = b {
                        recurse(region, universe, v, want_true ^ bit);
                    }
                }
                (_, Some(bit)) => {
                    if let Some(v) = a {
                        recurse(region, universe, v, want_true ^ bit);
                    }
                }
                _ => {}
            }
        }
        // Value-preserving carriers of an `i1`: recurse on the source unchanged.
        Operation::Copy
        | Operation::Freeze
        | Operation::Cast {
            kind: CastKind::ZExt | CastKind::SExt | CastKind::Trunc,
            ..
        } => {
            if let Some(&src) = def.operands.first() {
                recurse(region, universe, src, want_true);
            }
        }
        _ => {}
    }
}

/// Is `kind` an integer comparison predicate (the ones [`comparison_constraints`]
/// turns into affine half-spaces)?
fn is_int_comparison(kind: BinaryOp) -> bool {
    matches!(
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
}

/// Path-sensitive affine expansion: like [`resolve_affine`], but a **non-header
/// `Phi`** on the path resolves to its incoming value from the path predecessor
/// of the phi's block (so a join-block merge becomes the concrete value taken on
/// *this* path). `None` ⇒ not affine on this path.
#[allow(clippy::too_many_arguments)]
fn resolve_affine_path(
    v: ValueId,
    defs: &BTreeMap<ValueId, Def>,
    block_of: &BTreeMap<ValueId, BlockId>,
    header_phis: &BTreeSet<ValueId>,
    module: &AirModule,
    path_pred: &BTreeMap<BlockId, BlockId>,
    depth: u32,
) -> Option<Affine> {
    if depth > AFFINE_DEPTH_LIMIT {
        return None;
    }
    if let Some(Constant::Int { value, .. }) = module.constants.get(&v) {
        return Some(Affine::constant(i128::from(*value)));
    }
    if header_phis.contains(&v) {
        return Some(Affine::symbol(v));
    }
    let Some(def) = defs.get(&v) else {
        return Some(Affine::symbol(v));
    };
    if !def.in_loop {
        return Some(Affine::symbol(v));
    }
    let recur = |x: ValueId| {
        resolve_affine_path(x, defs, block_of, header_phis, module, path_pred, depth + 1)
    };
    match &def.op {
        Operation::BinaryOp {
            kind: kind @ (BinaryOp::Add | BinaryOp::Sub),
        } => {
            let a = recur(*def.operands.first()?)?;
            let b = recur(*def.operands.get(1)?)?;
            if matches!(kind, BinaryOp::Add) {
                a.add(&b)
            } else {
                a.sub(&b)
            }
        }
        Operation::BinaryOp {
            kind: BinaryOp::Mul,
        } => {
            let a = recur(*def.operands.first()?)?;
            let b = recur(*def.operands.get(1)?)?;
            if let Some(k) = b.as_constant() {
                a.scale(k)
            } else if let Some(k) = a.as_constant() {
                b.scale(k)
            } else {
                None
            }
        }
        Operation::Cast {
            kind: CastKind::ZExt | CastKind::SExt,
            ..
        }
        | Operation::Copy
        | Operation::Freeze => recur(*def.operands.first()?),
        // A non-header phi resolves along the current path to its incoming value
        // from the path predecessor of the phi's own block.
        Operation::Phi { incoming } => {
            let blk = block_of.get(&v)?;
            let Some(pred) = path_pred.get(blk) else {
                // The phi's block is not on this path (defensive) ⇒ opaque leaf.
                return Some(Affine::symbol(v));
            };
            let val = incoming
                .iter()
                .find(|(p, _)| p == pred)
                .map(|(_, val)| *val)?;
            recur(val)
        }
        // Load / Call / Select / Trunc / … ⇒ opaque free leaf (sound).
        _ => Some(Affine::symbol(v)),
    }
}

/// One greedy lexicographic round: is there a single linear `f` that is bounded
/// (`f ≥ 0`) and **non-increasing** (`f − f′ ≥ 0`) on every `remaining` branch and
/// **strictly** decreasing (`f − f′ ≥ 1`) on branch `strict`? Discharged by the
/// same Farkas reduction as [`synthesize_ranking_function`], summed over branches.
fn synthesize_round(model: &MultiPathModel, remaining: &[usize], strict: usize) -> bool {
    let universe: Vec<ValueId> = model.universe.iter().copied().collect();

    let solver = new_solver();

    // Ranking coefficients (template symbols only) + constant + strict decrease δ.
    let mut coeff: BTreeMap<ValueId, z3::ast::Int> = BTreeMap::new();
    for (i, m) in model.template.iter().enumerate() {
        coeff.insert(*m, z3::ast::Int::new_const(format!("coef_{i}")));
    }
    let c0 = z3::ast::Int::new_const("rank_const");
    let delta = z3::ast::Int::new_const("rank_delta");
    solver.assert(delta.ge(z3::ast::Int::from_i64(1)));

    let zero = || z3::ast::Int::from_i64(0);
    let coeff_f = |m: &ValueId| -> z3::ast::Int { coeff.get(m).cloned().unwrap_or_else(zero) };

    for &b in remaining {
        let branch = &model.branches[b];
        let region = &branch.region;

        // Non-negative Farkas multipliers for this branch's two requirements.
        let make_lambdas = |tag: char| -> Vec<z3::ast::Int> {
            (0..region.len())
                .map(|j| {
                    let l = z3::ast::Int::new_const(format!("lam_{tag}_{b}_{j}"));
                    solver.assert(l.ge(z3::ast::Int::from_i64(0)));
                    l
                })
                .collect()
        };
        let lam_a = make_lambdas('a');
        let lam_d = make_lambdas('d');

        let region_coeff = |lams: &[z3::ast::Int], m: &ValueId| -> Option<z3::ast::Int> {
            let mut acc = zero();
            for (j, c) in region.iter().enumerate() {
                if let Some(&a) = c.coeffs.get(m) {
                    acc += &lams[j] * z3::ast::Int::from_i64(i128_to_i64(a)?);
                }
            }
            Some(acc)
        };
        let region_const = |lams: &[z3::ast::Int]| -> Option<z3::ast::Int> {
            let mut acc = zero();
            for (j, c) in region.iter().enumerate() {
                acc += &lams[j] * z3::ast::Int::from_i64(i128_to_i64(c.constant)?);
            }
            Some(acc)
        };

        // Per-symbol coefficient identities (requirement A: f ≥ 0; requirement B:
        // f − f∘next_b ≥ k_b — only header phis change, so only they appear in B).
        for m in &universe {
            let Some(ra) = region_coeff(&lam_a, m) else {
                return false;
            };
            solver.assert(coeff_f(m).eq(ra));

            let mut lb_m = zero();
            for (phi, nxt) in &branch.next {
                let indicator = i128::from(phi == m);
                let in_next = nxt.terms.get(m).copied().unwrap_or(0);
                let Some(k) = i128_to_i64(indicator - in_next) else {
                    return false;
                };
                lb_m += &coeff_f(phi) * z3::ast::Int::from_i64(k);
            }
            let Some(rb) = region_coeff(&lam_d, m) else {
                return false;
            };
            solver.assert(lb_m.eq(rb));
        }

        // Constant-term inequalities: const_L − k − Σ λ·bⱼ ≥ 0.
        let Some(const_a) = region_const(&lam_a) else {
            return false;
        };
        solver.assert((c0.clone() - const_a).ge(zero()));

        let mut lb_const = zero();
        for (phi, nxt) in &branch.next {
            let Some(k) = i128_to_i64(-nxt.constant) else {
                return false;
            };
            lb_const += &coeff_f(phi) * z3::ast::Int::from_i64(k);
        }
        let k_b = if b == strict { delta.clone() } else { zero() };
        let Some(const_d) = region_const(&lam_d) else {
            return false;
        };
        solver.assert((lb_const - k_b - const_d).ge(zero()));
    }

    matches!(solver.check(), z3::SatResult::Sat)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use saf_core::air::{AirBlock, AirFunction, AirModule, AirParam, AirType, Instruction};
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

    // --- Path-sensitive (disjunctive + lexicographic) ------------------------

    /// A `__VERIFIER_nondet_*` call producing an opaque typed value.
    fn ndcall(id: &str, fname: &str, dst: ValueId, ty: TypeId) -> Instruction {
        vinst(
            id,
            Operation::CallDirect {
                callee: FunctionId(make_id("func", fname.as_bytes())),
            },
            dst,
            vec![],
            ty,
        )
    }

    fn main_func(blocks: Vec<AirBlock>, entry: BlockId) -> AirFunction {
        AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    #[test]
    fn multipath_two_increments_const_bound_is_ranked() {
        // while (i < 100) { if (nondet) i += 1; else i += 2; }
        // A single f = 100 - i decreases by 1 or 2 on both paths ⇒ ranked, but the
        // single-transition model sees the join phi as opaque and cannot.
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let entry = bid("entry");
        let h = bid("h");
        let body = bid("body");
        let bthen = bid("bthen");
        let bels = bid("bels");
        let latch = bid("latch");
        let exit = bid("exit");

        let i = vid("i");
        let i_n = vid("i_n");
        let ci = vid("ci");
        let cnd = vid("cnd");
        let ip1 = vid("ip1");
        let ip2 = vid("ip2");
        let i0 = vid("i0");
        let one = vid("one");
        let two = vid("two");
        let hundred = vid("hundred");
        constants.insert(i0, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });
        constants.insert(two, Constant::Int { value: 2, bits: 32 });
        constants.insert(
            hundred,
            Constant::Int {
                value: 100,
                bits: 32,
            },
        );

        let mut eb = AirBlock::new(entry);
        eb.instructions
            .push(term("br_e", Operation::Br { target: h }, vec![]));

        let mut hb = AirBlock::new(h);
        hb.instructions.push(vinst(
            "phi_i",
            Operation::Phi {
                incoming: vec![(entry, i0), (latch, i_n)],
            },
            i,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "cmp_i",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            ci,
            vec![i, hundred],
            i1t,
        ));
        hb.instructions.push(term(
            "cb_h",
            Operation::CondBr {
                then_target: body,
                else_target: exit,
            },
            vec![ci],
        ));

        let mut bb = AirBlock::new(body);
        bb.instructions
            .push(ndcall("nd", "__VERIFIER_nondet_bool", cnd, i1t));
        bb.instructions.push(term(
            "cb_b",
            Operation::CondBr {
                then_target: bthen,
                else_target: bels,
            },
            vec![cnd],
        ));

        let mut tb = AirBlock::new(bthen);
        tb.instructions.push(vinst(
            "add1",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            ip1,
            vec![i, one],
            i32t,
        ));
        tb.instructions
            .push(term("br_t", Operation::Br { target: latch }, vec![]));

        let mut lb = AirBlock::new(bels);
        lb.instructions.push(vinst(
            "add2",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            ip2,
            vec![i, two],
            i32t,
        ));
        lb.instructions
            .push(term("br_l", Operation::Br { target: latch }, vec![]));

        let mut latb = AirBlock::new(latch);
        latb.instructions.push(vinst(
            "phi_in",
            Operation::Phi {
                incoming: vec![(bthen, ip1), (bels, ip2)],
            },
            i_n,
            vec![],
            i32t,
        ));
        latb.instructions
            .push(term("br_lat", Operation::Br { target: h }, vec![]));

        let mut exb = AirBlock::new(exit);
        exb.instructions.push(term("ret", Operation::Ret, vec![i]));

        let func = main_func(vec![eb, hb, bb, tb, lb, latb, exb], entry);
        assert!(ranked(&module_of(func, constants)));
    }

    #[test]
    fn multipath_two_increments_symbolic_bound_abstains() {
        // while (i < n) { if (nondet) i += 1; else i += 2; }  — the `i += 2` path
        // can overflow near INT_MAX (n symbolic), so it is not stable ⇒ abstain.
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let entry = bid("entry");
        let h = bid("h");
        let body = bid("body");
        let bthen = bid("bthen");
        let bels = bid("bels");
        let latch = bid("latch");
        let exit = bid("exit");

        let i = vid("i");
        let i_n = vid("i_n");
        let ci = vid("ci");
        let cnd = vid("cnd");
        let ip1 = vid("ip1");
        let ip2 = vid("ip2");
        let i0 = vid("i0");
        let one = vid("one");
        let two = vid("two");
        let n = vid("n");
        constants.insert(i0, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });
        constants.insert(two, Constant::Int { value: 2, bits: 32 });

        let mut eb = AirBlock::new(entry);
        eb.instructions
            .push(ndcall("nd_n", "__VERIFIER_nondet_int", n, i32t));
        eb.instructions
            .push(term("br_e", Operation::Br { target: h }, vec![]));

        let mut hb = AirBlock::new(h);
        hb.instructions.push(vinst(
            "phi_i",
            Operation::Phi {
                incoming: vec![(entry, i0), (latch, i_n)],
            },
            i,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "cmp_i",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            ci,
            vec![i, n],
            i1t,
        ));
        hb.instructions.push(term(
            "cb_h",
            Operation::CondBr {
                then_target: body,
                else_target: exit,
            },
            vec![ci],
        ));

        let mut bb = AirBlock::new(body);
        bb.instructions
            .push(ndcall("nd", "__VERIFIER_nondet_bool", cnd, i1t));
        bb.instructions.push(term(
            "cb_b",
            Operation::CondBr {
                then_target: bthen,
                else_target: bels,
            },
            vec![cnd],
        ));

        let mut tb = AirBlock::new(bthen);
        tb.instructions.push(vinst(
            "add1",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            ip1,
            vec![i, one],
            i32t,
        ));
        tb.instructions
            .push(term("br_t", Operation::Br { target: latch }, vec![]));

        let mut lb = AirBlock::new(bels);
        lb.instructions.push(vinst(
            "add2",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            ip2,
            vec![i, two],
            i32t,
        ));
        lb.instructions
            .push(term("br_l", Operation::Br { target: latch }, vec![]));

        let mut latb = AirBlock::new(latch);
        latb.instructions.push(vinst(
            "phi_in",
            Operation::Phi {
                incoming: vec![(bthen, ip1), (bels, ip2)],
            },
            i_n,
            vec![],
            i32t,
        ));
        latb.instructions
            .push(term("br_lat", Operation::Br { target: h }, vec![]));

        let mut exb = AirBlock::new(exit);
        exb.instructions.push(term("ret", Operation::Ret, vec![i]));

        let func = main_func(vec![eb, hb, bb, tb, lb, latb, exb], entry);
        assert!(!ranked(&module_of(func, constants)));
    }

    #[test]
    fn lexicographic_two_phase_is_ranked() {
        // while (x >= 0 && y >= 0) { y = y - 1; if (y < 0) { x = x - 1; y = *; } }
        // No single linear ranking function; the lexicographic f = <x, y> works
        // (round 1 ranks x on the inner-if path, round 2 ranks y on the other).
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let entry = bid("entry");
        let h = bid("h");
        let h2 = bid("h2");
        let body = bid("body");
        let bthen = bid("bthen");
        let latch = bid("latch");
        let exit = bid("exit");

        let x = vid("x");
        let y = vid("y");
        let x_n = vid("x_n");
        let y_n = vid("y_n");
        let cx = vid("cx");
        let cy = vid("cy");
        let y1 = vid("y1");
        let c2 = vid("c2");
        let x1 = vid("x1");
        let ynd = vid("ynd");
        let x0 = vid("x0");
        let y0 = vid("y0");
        let zero = vid("zero");
        let one = vid("one");
        constants.insert(x0, Constant::Int { value: 0, bits: 32 });
        constants.insert(y0, Constant::Int { value: 0, bits: 32 });
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });

        let mut eb = AirBlock::new(entry);
        eb.instructions
            .push(term("br_e", Operation::Br { target: h }, vec![]));

        // h: phi x, phi y ; cx = x >= 0 ; condbr -> h2 / exit
        let mut hb = AirBlock::new(h);
        hb.instructions.push(vinst(
            "phi_x",
            Operation::Phi {
                incoming: vec![(entry, x0), (latch, x_n)],
            },
            x,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "phi_y",
            Operation::Phi {
                incoming: vec![(entry, y0), (latch, y_n)],
            },
            y,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "cmp_x",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSge,
            },
            cx,
            vec![x, zero],
            i1t,
        ));
        hb.instructions.push(term(
            "cb_x",
            Operation::CondBr {
                then_target: h2,
                else_target: exit,
            },
            vec![cx],
        ));

        // h2: cy = y >= 0 ; condbr -> body / exit
        let mut h2b = AirBlock::new(h2);
        h2b.instructions.push(vinst(
            "cmp_y",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSge,
            },
            cy,
            vec![y, zero],
            i1t,
        ));
        h2b.instructions.push(term(
            "cb_y",
            Operation::CondBr {
                then_target: body,
                else_target: exit,
            },
            vec![cy],
        ));

        // body: y1 = y - 1 ; c2 = y1 < 0 ; condbr -> bthen / latch
        let mut bb = AirBlock::new(body);
        bb.instructions.push(vinst(
            "sub_y",
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            y1,
            vec![y, one],
            i32t,
        ));
        bb.instructions.push(vinst(
            "cmp_y1",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            c2,
            vec![y1, zero],
            i1t,
        ));
        bb.instructions.push(term(
            "cb_y1",
            Operation::CondBr {
                then_target: bthen,
                else_target: latch,
            },
            vec![c2],
        ));

        // bthen: x1 = x - 1 ; ynd = nondet ; br latch
        let mut tb = AirBlock::new(bthen);
        tb.instructions.push(vinst(
            "sub_x",
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            x1,
            vec![x, one],
            i32t,
        ));
        tb.instructions
            .push(ndcall("nd_y", "__VERIFIER_nondet_int", ynd, i32t));
        tb.instructions
            .push(term("br_t", Operation::Br { target: latch }, vec![]));

        // latch: x_n = phi[body:x, bthen:x1] ; y_n = phi[body:y1, bthen:ynd] ; br h
        let mut latb = AirBlock::new(latch);
        latb.instructions.push(vinst(
            "phi_xn",
            Operation::Phi {
                incoming: vec![(body, x), (bthen, x1)],
            },
            x_n,
            vec![],
            i32t,
        ));
        latb.instructions.push(vinst(
            "phi_yn",
            Operation::Phi {
                incoming: vec![(body, y1), (bthen, ynd)],
            },
            y_n,
            vec![],
            i32t,
        ));
        latb.instructions
            .push(term("br_lat", Operation::Br { target: h }, vec![]));

        let mut exb = AirBlock::new(exit);
        exb.instructions.push(term("ret", Operation::Ret, vec![x]));

        let func = main_func(vec![eb, hb, h2b, bb, tb, latb, exb], entry);
        assert!(ranked(&module_of(func, constants)));
    }

    #[test]
    fn multipath_stutter_path_abstains() {
        // while (x > 0) { if (nondet) x = x - 1; else /* x unchanged */; }
        // The stuttering (`x = x`) path makes no progress ⇒ non-terminating when it
        // is taken forever, so no ranking function exists ⇒ abstain (soundness).
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let entry = bid("entry");
        let h = bid("h");
        let body = bid("body");
        let bthen = bid("bthen");
        let bels = bid("bels");
        let latch = bid("latch");
        let exit = bid("exit");

        let x = vid("x");
        let x_n = vid("x_n");
        let cx = vid("cx");
        let cnd = vid("cnd");
        let xd = vid("xd");
        let x0 = vid("x0");
        let zero = vid("zero");
        let one = vid("one");
        constants.insert(x0, Constant::Int { value: 5, bits: 32 });
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });

        let mut eb = AirBlock::new(entry);
        eb.instructions
            .push(term("br_e", Operation::Br { target: h }, vec![]));

        let mut hb = AirBlock::new(h);
        hb.instructions.push(vinst(
            "phi_x",
            Operation::Phi {
                incoming: vec![(entry, x0), (latch, x_n)],
            },
            x,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "cmp_x",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            cx,
            vec![x, zero],
            i1t,
        ));
        hb.instructions.push(term(
            "cb_h",
            Operation::CondBr {
                then_target: body,
                else_target: exit,
            },
            vec![cx],
        ));

        let mut bb = AirBlock::new(body);
        bb.instructions
            .push(ndcall("nd", "__VERIFIER_nondet_bool", cnd, i1t));
        bb.instructions.push(term(
            "cb_b",
            Operation::CondBr {
                then_target: bthen,
                else_target: bels,
            },
            vec![cnd],
        ));

        let mut tb = AirBlock::new(bthen);
        tb.instructions.push(vinst(
            "sub_x",
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            xd,
            vec![x, one],
            i32t,
        ));
        tb.instructions
            .push(term("br_t", Operation::Br { target: latch }, vec![]));

        let mut elb = AirBlock::new(bels);
        elb.instructions
            .push(term("br_el", Operation::Br { target: latch }, vec![]));

        let mut latb = AirBlock::new(latch);
        latb.instructions.push(vinst(
            "phi_xn",
            Operation::Phi {
                incoming: vec![(bthen, xd), (bels, x)],
            },
            x_n,
            vec![],
            i32t,
        ));
        latb.instructions
            .push(term("br_lat", Operation::Br { target: h }, vec![]));

        let mut exb = AirBlock::new(exit);
        exb.instructions.push(term("ret", Operation::Ret, vec![x]));

        let func = main_func(vec![eb, hb, bb, tb, elb, latb, exb], entry);
        assert!(!ranked(&module_of(func, constants)));
    }

    // --- short-circuit `&&` header guard (i1 phi) ---------------------------

    /// Build the CookSeeZuleger loop
    /// `while (x > 0 && y > 0) { if (nondet) x = x - 1; else { x = *; y = y - 1; } }`,
    /// whose `&&` header condition clang lowers to an `i1` **phi**
    /// (`gp = phi [false, header], [y>0, h2]`). If `strict` is `false` the `else`
    /// branch does `y = y + 1` instead — a genuinely non-terminating loop.
    ///
    /// Ranking needs the lexicographic tuple `(y, x)`: `y` bounds/ranks the
    /// `y`-decreasing branch (whose `x` is havoced), `x` ranks the `x`-decreasing
    /// branch. The `y ≥ 1` bound comes ONLY from the second `&&` conjunct, which is
    /// hidden behind the `i1` phi — so this loop ranks iff the guard resolver
    /// follows that phi.
    #[allow(clippy::too_many_lines)]
    fn cook_see_zuleger(decreasing: bool) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();

        let entry = bid("entry");
        let h = bid("h");
        let h2 = bid("h2");
        let g = bid("g");
        let body = bid("body");
        let pa = bid("pa");
        let pb = bid("pb");
        let latch = bid("latch");
        let exit = bid("exit");

        let x = vid("x");
        let y = vid("y");
        let x0 = vid("x0");
        let y0 = vid("y0");
        let xn = vid("xn");
        let yn = vid("yn");
        let cx = vid("cx");
        let cy = vid("cy");
        let gp = vid("gp");
        let cnd = vid("cnd");
        let xa = vid("xa");
        let xb = vid("xb");
        let yb = vid("yb");
        let zero = vid("zero");
        let one = vid("one");
        let false_c = vid("false_c");
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });
        constants.insert(false_c, Constant::Int { value: 0, bits: 1 });

        // entry: x0 = nondet; y0 = nondet; br h
        let mut eb = AirBlock::new(entry);
        eb.instructions
            .push(ndcall("nd_x0", "__VERIFIER_nondet_int", x0, i32t));
        eb.instructions
            .push(ndcall("nd_y0", "__VERIFIER_nondet_int", y0, i32t));
        eb.instructions
            .push(term("br_e", Operation::Br { target: h }, vec![]));

        // h: phi x, phi y; cx = x > 0; condbr cx -> h2 else g
        let mut hb = AirBlock::new(h);
        hb.instructions.push(vinst(
            "phi_x",
            Operation::Phi {
                incoming: vec![(entry, x0), (latch, xn)],
            },
            x,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "phi_y",
            Operation::Phi {
                incoming: vec![(entry, y0), (latch, yn)],
            },
            y,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "cmp_x",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            cx,
            vec![x, zero],
            i1t,
        ));
        hb.instructions.push(term(
            "cb_h",
            Operation::CondBr {
                then_target: h2,
                else_target: g,
            },
            vec![cx],
        ));

        // h2: cy = y > 0; br g
        let mut h2b = AirBlock::new(h2);
        h2b.instructions.push(vinst(
            "cmp_y",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            cy,
            vec![y, zero],
            i1t,
        ));
        h2b.instructions
            .push(term("br_h2", Operation::Br { target: g }, vec![]));

        // g: gp = phi [false, h], [cy, h2]; condbr gp -> body else exit
        let mut gb = AirBlock::new(g);
        gb.instructions.push(vinst(
            "phi_g",
            Operation::Phi {
                incoming: vec![(h, false_c), (h2, cy)],
            },
            gp,
            vec![],
            i1t,
        ));
        gb.instructions.push(term(
            "cb_g",
            Operation::CondBr {
                then_target: body,
                else_target: exit,
            },
            vec![gp],
        ));

        // body: cnd = nondet_bool; condbr cnd -> pa else pb
        let mut bb = AirBlock::new(body);
        bb.instructions
            .push(ndcall("nd_c", "__VERIFIER_nondet_bool", cnd, i1t));
        bb.instructions.push(term(
            "cb_b",
            Operation::CondBr {
                then_target: pa,
                else_target: pb,
            },
            vec![cnd],
        ));

        // pa: xa = x - 1; br latch   (y unchanged)
        let mut pab = AirBlock::new(pa);
        pab.instructions.push(vinst(
            "sub_x",
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            xa,
            vec![x, one],
            i32t,
        ));
        pab.instructions
            .push(term("br_pa", Operation::Br { target: latch }, vec![]));

        // pb: xb = nondet (havoc x); yb = y ∓ 1; br latch
        let mut pbb = AirBlock::new(pb);
        pbb.instructions
            .push(ndcall("nd_xb", "__VERIFIER_nondet_int", xb, i32t));
        pbb.instructions.push(vinst(
            "step_y",
            Operation::BinaryOp {
                kind: if decreasing {
                    BinaryOp::Sub
                } else {
                    BinaryOp::Add
                },
            },
            yb,
            vec![y, one],
            i32t,
        ));
        pbb.instructions
            .push(term("br_pb", Operation::Br { target: latch }, vec![]));

        // latch: xn = phi [xa, pa], [xb, pb]; yn = phi [y, pa], [yb, pb]; br h
        let mut latb = AirBlock::new(latch);
        latb.instructions.push(vinst(
            "phi_xn",
            Operation::Phi {
                incoming: vec![(pa, xa), (pb, xb)],
            },
            xn,
            vec![],
            i32t,
        ));
        latb.instructions.push(vinst(
            "phi_yn",
            Operation::Phi {
                incoming: vec![(pa, y), (pb, yb)],
            },
            yn,
            vec![],
            i32t,
        ));
        latb.instructions
            .push(term("br_lat", Operation::Br { target: h }, vec![]));

        let mut exb = AirBlock::new(exit);
        exb.instructions.push(term("ret", Operation::Ret, vec![]));

        let func = main_func(vec![eb, hb, h2b, gb, bb, pab, pbb, latb, exb], entry);
        module_of(func, constants)
    }

    #[test]
    fn short_circuit_and_guard_is_ranked() {
        // The `y > 0` bound is reachable only by following the `&&` i1 phi; with it,
        // the lexicographic tuple `(y, x)` ranks the loop.
        assert!(ranked(&cook_see_zuleger(true)));
    }

    #[test]
    fn short_circuit_and_nonterminating_abstains() {
        // Same short-circuit shape, but the `else` branch does `y = y + 1` while
        // havocing `x` — no ranking tuple exists ⇒ abstain (never a wrong `true`).
        assert!(!ranked(&cook_see_zuleger(false)));
    }

    // --- recursion ranking --------------------------------------------------

    /// Build `void f(int n) { if (n CMP bound) f(n + step); }` — a single-call-site
    /// self-recursive function. `cmp`'s signedness drives type-bound inference.
    fn self_rec(cmp: BinaryOp, bound: i64, step: i64) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let bound_v = vid("bound");
        let step_v = vid("step");
        constants.insert(
            bound_v,
            Constant::Int {
                value: bound,
                bits: 32,
            },
        );
        constants.insert(
            step_v,
            Constant::Int {
                value: step,
                bits: 32,
            },
        );

        let f_id = FunctionId(make_id("func", b"f"));
        let n = vid("n");
        let c = vid("c");
        let na = vid("na");
        let entry = bid("f_entry");
        let rec = bid("f_rec");
        let base = bid("f_base");

        let mut eb = AirBlock::new(entry);
        eb.instructions.push(vinst(
            "cmp",
            Operation::BinaryOp { kind: cmp },
            c,
            vec![n, bound_v],
            i1t,
        ));
        eb.instructions.push(term(
            "condbr",
            Operation::CondBr {
                then_target: rec,
                else_target: base,
            },
            vec![c],
        ));

        let mut rb = AirBlock::new(rec);
        rb.instructions.push(vinst(
            "add",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            na,
            vec![n, step_v],
            i32t,
        ));
        rb.instructions.push(term(
            "call",
            Operation::CallDirect { callee: f_id },
            vec![na],
        ));
        rb.instructions
            .push(term("br_rec", Operation::Br { target: base }, vec![]));

        let mut bb = AirBlock::new(base);
        bb.instructions.push(term("ret", Operation::Ret, vec![]));

        let func = AirFunction {
            id: f_id,
            name: "f".to_string(),
            params: vec![AirParam {
                id: n,
                name: None,
                index: 0,
                param_type: Some(i32t),
            }],
            blocks: vec![eb, rb, bb],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        module_of(func, constants)
    }

    fn rec_ranked(m: &AirModule) -> bool {
        let func = &m.functions[0];
        let cfg = Cfg::build(func);
        recursion_is_ranked(func, m, &cfg)
    }

    #[test]
    fn count_down_recursion_is_ranked() {
        // f(n){ if (n > 0) f(n - 1); }  (signed)  →  f = n.
        assert!(rec_ranked(&self_rec(BinaryOp::ICmpSgt, 0, -1)));
    }

    #[test]
    fn count_down_to_const_recursion_is_ranked() {
        // f(n){ if (n > 5) f(n - 1); }  →  f = n - 5.
        assert!(rec_ranked(&self_rec(BinaryOp::ICmpSgt, 5, -1)));
    }

    #[test]
    fn count_up_recursion_not_ranked() {
        // f(n){ if (n > 0) f(n + 1); }  — recurses forever for n > 0; no ranking.
        assert!(!rec_ranked(&self_rec(BinaryOp::ICmpSgt, 0, 1)));
    }

    #[test]
    fn wrong_direction_recursion_not_ranked() {
        // f(n){ if (n < 0) f(n - 1); }  — n moves away from 0; non-terminating.
        assert!(!rec_ranked(&self_rec(BinaryOp::ICmpSlt, 0, -1)));
    }

    #[test]
    fn ne_guard_recursion_not_ranked() {
        // f(n){ if (n != 0) f(n - 1); }  — `!=` is not convex; with only a type
        // bound the argument can decrease past INT_MIN ⇒ abstain (sound: this is
        // non-terminating for a negative signed `n`).
        assert!(!rec_ranked(&self_rec(BinaryOp::ICmpNe, 0, -1)));
    }

    #[test]
    fn identity_arg_recursion_not_ranked() {
        // f(n){ if (n > 0) f(n); } — the argument never changes (step 0), so no
        // ranking function strictly decreases ⇒ abstain (this recurses forever).
        assert!(!rec_ranked(&self_rec(BinaryOp::ICmpSgt, 0, 0)));
    }

    /// Build `void f(int n) { if (n > 1) { f(n - 1); f(n - 2); } }` — a two-call-site
    /// (tree) self-recursion ranked by the single `f = n`.
    fn fib_rec() -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let one = vid("one");
        let two = vid("two");
        let bnd = vid("bnd");
        constants.insert(one, Constant::Int { value: 1, bits: 32 });
        constants.insert(two, Constant::Int { value: 2, bits: 32 });
        constants.insert(bnd, Constant::Int { value: 1, bits: 32 });

        let f_id = FunctionId(make_id("func", b"f"));
        let n = vid("n");
        let c = vid("c");
        let n1 = vid("n1");
        let n2 = vid("n2");
        let entry = bid("f_entry");
        let rec = bid("f_rec");
        let base = bid("f_base");

        let mut eb = AirBlock::new(entry);
        eb.instructions.push(vinst(
            "cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            c,
            vec![n, bnd],
            i1t,
        ));
        eb.instructions.push(term(
            "condbr",
            Operation::CondBr {
                then_target: rec,
                else_target: base,
            },
            vec![c],
        ));

        let mut rb = AirBlock::new(rec);
        rb.instructions.push(vinst(
            "sub1",
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            n1,
            vec![n, one],
            i32t,
        ));
        rb.instructions.push(term(
            "call1",
            Operation::CallDirect { callee: f_id },
            vec![n1],
        ));
        rb.instructions.push(vinst(
            "sub2",
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            n2,
            vec![n, two],
            i32t,
        ));
        rb.instructions.push(term(
            "call2",
            Operation::CallDirect { callee: f_id },
            vec![n2],
        ));
        rb.instructions
            .push(term("br_rec", Operation::Br { target: base }, vec![]));

        let mut bb = AirBlock::new(base);
        bb.instructions.push(term("ret", Operation::Ret, vec![]));

        let func = AirFunction {
            id: f_id,
            name: "f".to_string(),
            params: vec![AirParam {
                id: n,
                name: None,
                index: 0,
                param_type: Some(i32t),
            }],
            blocks: vec![eb, rb, bb],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        module_of(func, constants)
    }

    #[test]
    fn tree_recursion_is_ranked() {
        // Both recursive call sites (n-1 and n-2) strictly decrease f = n under the
        // shared guard n > 1 ⇒ ranked.
        assert!(rec_ranked(&fib_rec()));
    }
}
