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
use saf_core::air::{
    AirFunction, AirModule, AirParam, BinaryOp, CastKind, Constant, Instruction, Operation,
};
use saf_core::id::make_id;
use saf_core::ids::{BlockId, FunctionId, TypeId, ValueId};

use crate::fast_paths::cfg_has_loops;

/// Maximum recursion depth when expanding an SSA value into an affine form.
/// Bounds work on pathological IR; exceeding it ⇒ treat the value as opaque.
const AFFINE_DEPTH_LIMIT: u32 = 64;

/// Cap on the number of **feasible** header→latch paths modelled per loop body
/// (equivalently: recursion entry→call paths). Bounds the per-loop Z3 work and
/// keeps the pass within the SV-COMP time budget; a body with more *feasible*
/// paths than this is not modelled path-sensitively (⇒ abstain / fall back to the
/// single-transition havoc model).
const MAX_PATHS: usize = 8;

/// Cap on the number of **structurally enumerated** header→latch paths, before
/// infeasible ones (short-circuit `&&`/`||` exit shortcuts — see
/// [`path_guard_is_unsat`]) are pruned. A multi-conjunct short-circuit guard fans a
/// single real path into a product of infeasible sibling paths, so this ceiling is
/// higher than [`MAX_PATHS`]: it lets those phantom paths be enumerated *and then
/// discarded* rather than tripping the abstain before the feasible paths are even
/// seen. Still bounded (a body with more than this many *structural* paths abstains
/// outright) so enumeration itself stays cheap.
const MAX_ENUM_PATHS: usize = 64;

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
    // Fast path: a reducible CFG with one latch per header — the full existing
    // machinery applies (path-sensitive multipath model with a havoc fallback that
    // also covers nested loops).
    if let Some(loops) = extract_natural_loops(func, cfg) {
        // Every natural loop must be ranked; any failure (unmodelable loop, or Z3
        // `Unsat`/`Unknown`) abstains the whole function.
        return loops.iter().all(|li| loop_is_ranked(func, module, cfg, li));
    }
    // Additive fallback: a reducible CFG where some header has **more than one**
    // back-edge (a `continue`, or a short-circuit `while (a && b)`). The single-
    // latch extractor rejected it; rank each header's loop with one common
    // lexicographic function over *all* its back-edge transitions (multi-latch
    // soundness: see [`build_multipath_model_multi`]). Irreducible CFGs are still
    // rejected here (⇒ abstain).
    match extract_multilatch_loops(func, cfg) {
        Some(loops) => loops
            .iter()
            .all(|ml| multilatch_loop_is_ranked(func, module, cfg, ml)),
        None => false,
    }
}

/// A natural loop identified by a header and **all** of its back-edge latches
/// (`latchᵢ → header`). Unlike [`LoopInfo`] this admits more than one latch.
struct MultiLatchLoop {
    header: BlockId,
    /// Every latch with a back-edge into `header` (sorted, deduped — deterministic).
    latches: Vec<BlockId>,
    /// The natural-loop body: `{header}` ∪ every node reaching some latch without
    /// passing through `header`.
    body: BTreeSet<BlockId>,
    /// Immediate-dominator map for the function CFG.
    idom: BTreeMap<BlockId, BlockId>,
}

/// Identify every natural loop grouping *all* back-edges by their header (so a
/// header with several latches becomes ONE [`MultiLatchLoop`]), or `None` if the
/// CFG is **irreducible** (removing dominance back-edges leaves a cycle) or has no
/// back-edge. Mirrors [`extract_natural_loops`] but does not reject multi-latch.
fn extract_multilatch_loops(func: &AirFunction, cfg: &Cfg) -> Option<Vec<MultiLatchLoop>> {
    let idom = compute_dominators(cfg);

    // Back-edges `u → v` where the head `v` dominates its tail `u`.
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

    // Reducibility: with all dominance back-edges removed a reducible CFG is acyclic.
    // A surviving cycle is an irreducible retreating edge natural-loop enumeration
    // would miss ⇒ abstain (same guard as the single-latch extractor).
    let back_set: BTreeSet<(BlockId, BlockId)> = back_edges.iter().copied().collect();
    if forward_graph_has_cycle(cfg, &back_set) {
        return None;
    }

    // Group latches by header (BTreeMap ⇒ deterministic header + latch order).
    let mut by_header: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
    for &(latch, header) in &back_edges {
        by_header.entry(header).or_default().insert(latch);
    }

    let mut loops = Vec::with_capacity(by_header.len());
    for (header, latches) in by_header {
        // Sanity: the header must exist.
        func.blocks.iter().find(|b| b.id == header)?;

        // Body: {header} ∪ nodes reaching any latch without passing through header.
        let mut body: BTreeSet<BlockId> = BTreeSet::new();
        body.insert(header);
        let mut queue: VecDeque<BlockId> = VecDeque::new();
        for &latch in &latches {
            if latch != header && body.insert(latch) {
                queue.push_back(latch);
            }
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

        loops.push(MultiLatchLoop {
            header,
            latches: latches.into_iter().collect(),
            body,
            idom: idom.clone(),
        });
    }
    Some(loops)
}

/// Rank one [`MultiLatchLoop`] with a single lexicographic ranking function that is
/// bounded and strictly decreasing on every back-edge transition. Path-sensitive
/// only (no havoc fallback): a nested inner loop, too many paths, or a non-affine
/// update abstains (⇒ `unknown`). Sound — see [`build_multipath_model_multi`].
fn multilatch_loop_is_ranked(
    func: &AirFunction,
    module: &AirModule,
    cfg: &Cfg,
    ml: &MultiLatchLoop,
) -> bool {
    match build_multipath_model_multi(
        func,
        module,
        cfg,
        ml.header,
        &ml.latches,
        &ml.body,
        &ml.idom,
    ) {
        Some(model) => greedy_lex_rank(&model),
        None => false,
    }
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
    let signs = infer_signs(func, module);

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

    // Function parameters carry their declared type on [`AirParam`], not on any
    // instruction [`Def`], so [`type_bounds`] (which reads `defs`) misses them.
    // Index each param's `TypeId` so a param used as a loop bound (`for(;i<y;i++)`
    // with `y` a parameter) still gets its type range — without it the `i+1`
    // overflow check cannot bound `y ≤ INT_MAX`, so the counter is judged unstable
    // and the (otherwise trivially ranked) increasing loop abstains.
    let param_types: BTreeMap<ValueId, TypeId> = func
        .params
        .iter()
        .filter_map(|p| p.param_type.map(|t| (p.id, t)))
        .collect();

    // Type bounds for every integer leaf symbol with a *known* signedness and a
    // representable width — a valid fact about the real state, so adding it only
    // shrinks the region by truths (keeping it a superset of real states).
    let mut bounds: BTreeMap<ValueId, (i128, i128)> = BTreeMap::new();
    for &sym in &universe {
        if let Some((lo, hi)) = type_bounds(sym, &defs, &signs, module).or_else(|| {
            param_types
                .get(&sym)
                .and_then(|&t| bounds_from_type(sym, t, &signs, module))
        }) {
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
        // `!=` is not convex; drop it here (sound — weaker guard). The disjunctive
        // path-split ([`ne_split`]) recovers it precisely for the path-sensitive
        // models by forking the branch into the two convex half-spaces.
        _ => Some(vec![]),
    }
}

/// If the comparison `lhs <pred> rhs`, taken in the `taken_true` polarity, is
/// effectively **`!=`** (a `!=` taken true, or an `==` taken *false*), return the
/// two convex half-spaces `lhs ≥ rhs + 1` and `lhs ≤ rhs − 1` whose **union** is
/// the guard region `lhs ≠ rhs`. Returns `None` for every other predicate (those
/// are already a single convex half-space handled by [`comparison_constraints`]).
///
/// The path-sensitive model forks the branch into two branches — one per
/// half-space — and requires the ranking to hold on **both** (a disjunctive
/// termination argument). This is sound: the union of the two half-spaces is still
/// a superset of the real continuing states on this path (which do satisfy
/// `lhs ≠ rhs`), and requiring a common lexicographic ranking over both covers that
/// union. It is strictly *more precise* than dropping the guard — each half-space
/// contributes the lower/upper bound `!=` alone cannot, which is exactly what makes
/// an otherwise-`havoc` transition (e.g. `n − 1` under `n ≠ 0` with `n` unsigned,
/// where `n ≥ 1` proves no underflow) affine-stable and hence rankable. The `∞`
/// (both-halves-feasible, one-side-diverging) cases still abstain because the
/// diverging half cannot be ranked.
fn ne_split(
    kind: BinaryOp,
    lhs: &Affine,
    rhs: &Affine,
    taken_true: bool,
) -> Option<[Constraint; 2]> {
    use BinaryOp::{ICmpEq, ICmpNe};
    if !matches!((kind, taken_true), (ICmpNe, true) | (ICmpEq, false)) {
        return None;
    }
    // `cons(a, b, m)` builds `b − a + m ≥ 0` (mirrors [`comparison_constraints`]).
    let cons = |a: &Affine, b: &Affine, minus: i128| -> Option<Constraint> {
        let e = b.sub(a)?.add(&Affine::constant(minus))?;
        Some(Constraint {
            coeffs: e.terms,
            constant: e.constant,
        })
    };
    // Half A: `lhs > rhs` ⇒ `lhs − rhs − 1 ≥ 0`. Half B: `lhs < rhs` ⇒
    // `rhs − lhs − 1 ≥ 0`. (Same forms as the strict `>`/`<` arms above.)
    Some([cons(rhs, lhs, -1)?, cons(lhs, rhs, -1)?])
}

/// Infer each integer value's signedness from the operations that consume it.
fn infer_signs(func: &AirFunction, module: &AirModule) -> BTreeMap<ValueId, Sign> {
    let mut signed: BTreeSet<ValueId> = BTreeSet::new();
    let mut unsigned: BTreeSet<ValueId> = BTreeSet::new();
    // Values whose *defining* op yields an unconditionally **non-negative** result
    // (bit pattern in `[0, 2^w−1]`), independent of any operand's sign. These carry
    // a sound `[0, 2^w−1]` range even when no consuming op hints a signedness — the
    // lower bound a `!=`/`==`-else guard needs to make its negative half-space
    // infeasible. See [`def_result_nonneg`].
    let mut nonneg_result: BTreeSet<ValueId> = BTreeSet::new();
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
            if let Some(dst) = inst.dst {
                if def_result_nonneg(inst, module) {
                    nonneg_result.insert(dst);
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    for &v in signed.union(&unsigned).chain(nonneg_result.iter()) {
        // A value used *signed* keeps its (also sound) signed range; a value used
        // *both* ways stays `Unknown` (fail-closed, no bound). Only a value with no
        // consuming signedness hint is promoted to `Unsigned` by a non-negative
        // defining op — a purely additive `[0, 2^w−1]` bound where there was none.
        let sign = match (signed.contains(&v), unsigned.contains(&v)) {
            (true, false) => Sign::Signed,
            (false, true) => Sign::Unsigned,
            (false, false) if nonneg_result.contains(&v) => Sign::Unsigned,
            _ => Sign::Unknown,
        };
        out.insert(v, sign);
    }
    out
}

/// Does `inst`'s defining op produce an unconditionally **non-negative** integer
/// result — a bit pattern in `[0, 2^w−1]` for *every* operand value, so the result
/// can soundly carry the unsigned range `[0, 2^w−1]` regardless of its operands'
/// signedness? Recognized (each provably clears the result's sign bit):
///
/// - **`urem`** (unsigned remainder): the result lies in `[0, divisor)` in every
///   defined execution (an unsigned modulo is never negative).
/// - **`lshr x, k`** with a *constant* shift `1 ≤ k < w`: the top `k` bits become
///   `0`, so the sign bit is clear.
/// - **`and x, m`** with a *constant* mask `m` whose sign bit is clear
///   (`0 ≤ m < 2^(w−1)`): the result's bits are a subset of `m`'s, so `0 ≤ r ≤ m`.
///
/// Fails closed (returns `false`) on anything else — a value that might be negative
/// keeps no bound, so only recall is affected, never soundness. This mirrors the
/// non-negativity reasoning in [`arg_is_nonneg`], but for a *locally defined* result
/// rather than a call argument, and feeds [`infer_signs`]'s `[0, 2^w−1]` bound.
fn def_result_nonneg(inst: &Instruction, module: &AirModule) -> bool {
    let Operation::BinaryOp { kind } = &inst.op else {
        return false;
    };
    let const_int = |v: &ValueId| match module.constants.get(v) {
        Some(Constant::Int { value, bits }) => Some((*value, *bits)),
        _ => None,
    };
    match kind {
        // Unsigned remainder is always in `[0, divisor)` — never negative.
        BinaryOp::URem => true,
        // Logical right shift by a positive constant clears the top bit(s).
        BinaryOp::LShr => {
            let width = inst
                .result_type
                .and_then(|t| int_width(module, t))
                .map_or(0i128, i128::from);
            inst.operands
                .get(1)
                .and_then(const_int)
                .is_some_and(|(k, _)| width > 0 && k >= 1 && i128::from(k) < width)
        }
        // AND with a non-negative constant mask bounds the result to `[0, mask]`.
        BinaryOp::And => inst.operands.iter().any(|op| {
            const_int(op).is_some_and(|(value, bits)| {
                (1..=64).contains(&bits) && value >= 0 && i128::from(value) < (1i128 << (bits - 1))
            })
        }),
        _ => false,
    }
}

/// Leaf symbols that must **keep** their inferred signedness and may not be
/// re-bounded by the unsigned bit-pattern range `[0, 2^w−1]`. A leaf is ineligible
/// when it feeds — directly or through an affine derivation — either:
///
/// - an **ordered** integer comparison (`<`/`<=`/`>`/`>=`, signed *or* unsigned):
///   [`comparison_constraints`] lowers ordered predicates into affine half-spaces
///   *without regard to signedness*, so the lowering is a sound necessary
///   condition only under the integer bound matching how the machine actually
///   reads the value; forcing an unsigned bound could wrongly prune a real signed
///   half (a `< 0` branch) and unsoundly claim termination; or
/// - an **`nsw`** integer op (`add`/`sub`/`mul`/`shl`): a signed overflow there is
///   undefined behavior, so the operand must not be modelled as a defined
///   `mod 2^w` wraparound (a `sub nsw x, 1` from `INT_MIN` is UB, not a step to
///   `INT_MAX`).
///
/// Equality/disequality (`==`/`!=`) is interpretation-independent (bit equality),
/// so an Eq/Ne-only, `nsw`-free value is safe to bound by its bit pattern — which
/// is exactly the lower bound the disjunctive `!=` split ([`ne_split`]) needs to
/// prune its negative half-space.
fn bitpattern_ineligible_leaves(
    func: &AirFunction,
    defs: &BTreeMap<ValueId, Def>,
    header_phis: &BTreeSet<ValueId>,
    module: &AirModule,
) -> BTreeSet<ValueId> {
    let mut out: BTreeSet<ValueId> = BTreeSet::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            let Operation::BinaryOp { kind } = &inst.op else {
                continue;
            };
            let ordered = matches!(
                kind,
                BinaryOp::ICmpSgt
                    | BinaryOp::ICmpSge
                    | BinaryOp::ICmpSlt
                    | BinaryOp::ICmpSle
                    | BinaryOp::ICmpUgt
                    | BinaryOp::ICmpUge
                    | BinaryOp::ICmpUlt
                    | BinaryOp::ICmpUle
            );
            if !(ordered || inst.has_no_signed_wrap()) {
                continue;
            }
            for &op in inst.operands.iter().take(2) {
                match resolve_affine(op, defs, header_phis, module, 0) {
                    Some(aff) => out.extend(aff.terms.keys().copied()),
                    None => {
                        out.insert(op);
                    }
                }
            }
        }
    }
    out
}

/// Promote every leaf in `candidates` whose signedness is currently *unknown*
/// (absent, or explicitly [`Sign::Unknown`]) and that is **not** in `ineligible`
/// to [`Sign::Unsigned`], giving it the sound `[0, 2^w−1]` bit-pattern bound.
///
/// This is unconditionally sound: `[0, 2^w−1]` is a true fact about *every* `w`-bit
/// machine value, and [`bitpattern_ineligible_leaves`] excludes exactly the values
/// whose guards or `nsw` arithmetic would make an unsigned reading unsound. It is
/// also purely additive — only leaves that previously had *no* bound gain one — so
/// a ranking that already succeeded cannot be lost, and the extra lower bound only
/// lets the `!=`/`==`-else split prune its (now-infeasible) negative half.
fn promote_bitpattern_unsigned(
    signs: &mut BTreeMap<ValueId, Sign>,
    candidates: &BTreeSet<ValueId>,
    ineligible: &BTreeSet<ValueId>,
) {
    for &v in candidates {
        if ineligible.contains(&v) {
            continue;
        }
        if !matches!(signs.get(&v), Some(Sign::Signed | Sign::Unsigned)) {
            signs.insert(v, Sign::Unsigned);
        }
    }
}

/// SV-COMP `__VERIFIER_nondet_*` sources that return a **non-negative** value
/// (an unsigned integer or a boolean). A call to one of these is a sound witness
/// that its result — and hence anything it is directly assigned to — lies in the
/// unsigned range `[0, 2^w−1]`, never a negative two's-complement value.
fn is_unsigned_nondet(name: &str) -> bool {
    matches!(
        name,
        "__VERIFIER_nondet_uint"
            | "__VERIFIER_nondet_unsigned"
            | "__VERIFIER_nondet_uchar"
            | "__VERIFIER_nondet_ushort"
            | "__VERIFIER_nondet_ulong"
            | "__VERIFIER_nondet_ulonglong"
            | "__VERIFIER_nondet_u8"
            | "__VERIFIER_nondet_u16"
            | "__VERIFIER_nondet_u32"
            | "__VERIFIER_nondet_u64"
            | "__VERIFIER_nondet_size_t"
            | "__VERIFIER_nondet_bool"
            | "__VERIFIER_nondet_pchar"
    )
}

/// Is call argument `arg` (evaluated in `caller`) provably a **non-negative**
/// integer, using only `caller`-local facts? A non-negative value is one whose
/// two's-complement bit pattern is a genuine `[0, 2^w−1]` unsigned magnitude —
/// so treating the callee parameter it feeds as unsigned is sound. Recognized
/// sources (each unambiguously non-negative regardless of any wider signed
/// interpretation):
///
/// - a non-negative integer constant (`Int`/`ZeroInit`);
/// - the result of an unsigned nondet source ([`is_unsigned_nondet`]);
/// - a zero-extension (`ZExt`) — its high bits are all `0`, so the value is
///   non-negative in both the source and destination widths;
/// - an unconditionally non-negative arithmetic result ([`def_result_nonneg`]):
///   an unsigned remainder (`x % k`), a mask (`x & m`, `m ≥ 0`), or a logical
///   right shift by a positive constant (`x >> k`).
///
/// Anything else (a passed-through parameter, a signed nondet, an arbitrary
/// arithmetic result, `Undef`/`BigInt`) fails closed ⇒ the position is *not*
/// treated as unsigned (only recall is lost, never soundness).
fn arg_is_nonneg(arg: ValueId, caller: &AirFunction, module: &AirModule) -> bool {
    if let Some(c) = module.constants.get(&arg) {
        return match c {
            Constant::Int { value, .. } => *value >= 0,
            Constant::ZeroInit => true,
            _ => false,
        };
    }
    for block in &caller.blocks {
        for inst in &block.instructions {
            if inst.dst != Some(arg) {
                continue;
            }
            return match &inst.op {
                Operation::CallDirect { callee } => module
                    .functions
                    .iter()
                    .find(|f| f.id == *callee)
                    .is_some_and(|f| is_unsigned_nondet(&f.name)),
                Operation::Cast {
                    kind: CastKind::ZExt,
                    ..
                } => true,
                // An unconditionally non-negative arithmetic result — `x % k`
                // (unsigned), `x & mask`, `x >> k` — is a sound witness that the
                // callee parameter it feeds is non-negative, regardless of `x`'s
                // sign. See [`def_result_nonneg`].
                Operation::BinaryOp { .. } => def_result_nonneg(inst, module),
                _ => false,
            };
        }
    }
    false
}

/// Given a signed integer comparison of `arg` against the constant `0` used as a
/// `CondBr` condition, return the successor edge target on which `arg >= 0` is
/// *guaranteed* (or `None` for a non-signed / non-order predicate). `arg_is_lhs`
/// says whether `arg` is the comparison's left operand (else it is the right, and
/// the predicate is swapped to the equivalent `arg <p> 0` form).
fn signed_cmp_nonneg_edge(
    pred: BinaryOp,
    arg_is_lhs: bool,
    then_target: BlockId,
    else_target: BlockId,
) -> Option<BlockId> {
    use BinaryOp::{ICmpSge, ICmpSgt, ICmpSle, ICmpSlt};
    // Normalize to `arg <p> 0`: swap the predicate when `arg` is the right operand
    // (`k <pred> arg` ⟺ `arg <swap(pred)> k`).
    let p = if arg_is_lhs {
        pred
    } else {
        match pred {
            ICmpSge => ICmpSle,
            ICmpSle => ICmpSge,
            ICmpSgt => ICmpSlt,
            ICmpSlt => ICmpSgt,
            other => other,
        }
    };
    match p {
        // `arg >= 0` / `arg > 0`: the THEN edge is entered only when `arg >= 0`.
        ICmpSge | ICmpSgt => Some(then_target),
        // `arg < 0` / `arg <= 0`: the ELSE edge is `!(arg<0)` ⇒ `arg>=0`
        // (resp. `!(arg<=0)` ⇒ `arg>0`).
        ICmpSlt | ICmpSle => Some(else_target),
        _ => None,
    }
}

/// Is call argument `arg` (fed at a call in block `call_block` of `caller`)
/// provably **non-negative** because a *dominating signed guard* `arg >= 0` gates
/// the call — e.g. an early `if (arg < 0) return;`?
///
/// Sound sufficient condition: some block `g` in `caller` ends in a
/// `CondBr` whose condition is a signed integer comparison of the *exact* value
/// `arg` against the constant `0`, and the successor edge that is taken **only
/// when `arg >= 0`** ([`signed_cmp_nonneg_edge`]) leads to a block `t` such that
/// (a) `g` is `t`'s **sole predecessor** — so control reaches `t` only by taking
/// that `arg >= 0` edge — and (b) `t` **dominates** `call_block` — so every path
/// to the call passes through `t`. Together these establish `arg >= 0` on entry to
/// the call. Because `arg` is matched by identity in both the guard and the call,
/// in the mem2reg-promoted SSA the ranking analysis consumes it is the *same*
/// value at both sites (never reassigned between them), so the fact is preserved.
///
/// Fails closed (returns `false`) on anything it cannot match — including
/// un-promoted IR where the guard and the call load distinct SSA temporaries, so
/// only recall is ever lost, never soundness.
fn arg_is_guarded_nonneg(
    arg: ValueId,
    caller: &AirFunction,
    call_block: BlockId,
    module: &AirModule,
) -> bool {
    // dst ↦ (op, operands) for resolving each `CondBr` condition to its comparison.
    let mut def_of: BTreeMap<ValueId, (&Operation, &[ValueId])> = BTreeMap::new();
    for block in &caller.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                def_of.insert(dst, (&inst.op, inst.operands.as_slice()));
            }
        }
    }
    let is_zero = |v: ValueId| {
        matches!(
            module.constants.get(&v),
            Some(Constant::Int { value: 0, .. } | Constant::ZeroInit)
        )
    };
    let cfg = Cfg::build(caller);
    let idom = compute_dominators(&cfg);
    for block in &caller.blocks {
        let Some(term) = block.instructions.last() else {
            continue;
        };
        let Operation::CondBr {
            then_target,
            else_target,
        } = &term.op
        else {
            continue;
        };
        let Some(&cond) = term.operands.first() else {
            continue;
        };
        let Some((Operation::BinaryOp { kind }, ops)) = def_of.get(&cond).copied() else {
            continue;
        };
        if !matches!(
            kind,
            BinaryOp::ICmpSlt | BinaryOp::ICmpSle | BinaryOp::ICmpSgt | BinaryOp::ICmpSge
        ) {
            continue;
        }
        let (Some(&lo), Some(&ro)) = (ops.first(), ops.get(1)) else {
            continue;
        };
        // Exactly one operand is `arg`, the other the constant `0`.
        let arg_is_lhs = lo == arg && is_zero(ro);
        let arg_is_rhs = ro == arg && is_zero(lo);
        if !(arg_is_lhs || arg_is_rhs) {
            continue;
        }
        let Some(nonneg_target) =
            signed_cmp_nonneg_edge(*kind, arg_is_lhs, *then_target, *else_target)
        else {
            continue;
        };
        // Entering `nonneg_target` must imply the `arg >= 0` edge was taken (sole
        // predecessor = this guard), and every path to the call must pass through
        // it (dominance). Then `arg >= 0` holds at the call.
        let sole_pred = cfg
            .predecessors
            .get(&nonneg_target)
            .is_some_and(|preds| preds.len() == 1 && preds.contains(&block.id));
        if sole_pred && dominates(nonneg_target, call_block, &idom) {
            return true;
        }
    }
    false
}

/// Given a signed integer comparison `v <pred> c` (with `v` the left operand when
/// `v_is_lhs`, else the right) against the *constant* `c`, return the successor edge
/// (`then`/`else`) on which `v >= 0` is guaranteed, or `None`. Generalizes
/// [`signed_cmp_nonneg_edge`] (which fixes `c = 0`) to any constant, recognizing the
/// canonical `v >= 0` forms `sge v, 0`, `sgt v, -1`, `slt v, 0`, `sle v, -1`
/// (and rhs duals) that `-O0`+instcombine emits for `if (v < 0) …` / `if (v >= 0)`.
fn signed_cmp_const_nonneg_edge(
    pred: BinaryOp,
    v_is_lhs: bool,
    c: i128,
    then_target: BlockId,
    else_target: BlockId,
) -> Option<BlockId> {
    use BinaryOp::{ICmpSge, ICmpSgt, ICmpSle, ICmpSlt};
    // Normalize to `v <p> c` (swap the predicate when `v` is the right operand).
    let p = if v_is_lhs {
        pred
    } else {
        match pred {
            ICmpSge => ICmpSle,
            ICmpSle => ICmpSge,
            ICmpSgt => ICmpSlt,
            ICmpSlt => ICmpSgt,
            other => other,
        }
    };
    match p {
        // `v >= c` ⇒ `v >= 0` when `c >= 0`; the THEN edge is that branch.
        ICmpSge if c >= 0 => Some(then_target),
        // `v > c` ⇒ `v >= c + 1 >= 0` when `c >= -1`; THEN edge.
        ICmpSgt if c >= -1 => Some(then_target),
        // `v < c` false ⇒ `v >= c >= 0` when `c >= 0`; the ELSE edge.
        ICmpSlt if c >= 0 => Some(else_target),
        // `v <= c` false ⇒ `v >= c + 1 >= 0` when `c >= -1`; ELSE edge.
        ICmpSle if c >= -1 => Some(else_target),
        _ => None,
    }
}

/// Does a *dominating* signed guard force `v >= 0` on every path to `header`? Sound
/// sufficient condition (mirrors [`arg_is_guarded_nonneg`] but for a loop-entry use
/// point and any constant bound): some block `g` ends in a `CondBr` on a signed
/// comparison of the *exact* value `v` against a constant, whose `v >= 0` edge
/// ([`signed_cmp_const_nonneg_edge`]) leads to a block `t` that (a) has `g` as its
/// **sole predecessor** and (b) **dominates** `header`. In the mem2reg-promoted SSA
/// the ranking analysis consumes, `v` is the same value at the guard and at the
/// phi's preheader incoming (never reassigned between), so the fact holds on entry.
fn value_dominating_nonneg(
    v: ValueId,
    func: &AirFunction,
    header: BlockId,
    module: &AirModule,
) -> bool {
    let mut def_of: BTreeMap<ValueId, (&Operation, &[ValueId])> = BTreeMap::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                def_of.insert(dst, (&inst.op, inst.operands.as_slice()));
            }
        }
    }
    let const_int = |x: ValueId| match module.constants.get(&x) {
        Some(Constant::Int { value, .. }) => Some(i128::from(*value)),
        Some(Constant::ZeroInit) => Some(0),
        _ => None,
    };
    let cfg = Cfg::build(func);
    let idom = compute_dominators(&cfg);
    for block in &func.blocks {
        let Some(term) = block.instructions.last() else {
            continue;
        };
        let Operation::CondBr {
            then_target,
            else_target,
        } = &term.op
        else {
            continue;
        };
        let Some(&cond) = term.operands.first() else {
            continue;
        };
        let Some((Operation::BinaryOp { kind }, ops)) = def_of.get(&cond).copied() else {
            continue;
        };
        if !matches!(
            kind,
            BinaryOp::ICmpSlt | BinaryOp::ICmpSle | BinaryOp::ICmpSgt | BinaryOp::ICmpSge
        ) {
            continue;
        }
        let (Some(&lo), Some(&ro)) = (ops.first(), ops.get(1)) else {
            continue;
        };
        // Exactly one operand is `v`, the other a constant.
        let (v_is_lhs, c) = if lo == v {
            let Some(c) = const_int(ro) else { continue };
            (true, c)
        } else if ro == v {
            let Some(c) = const_int(lo) else { continue };
            (false, c)
        } else {
            continue;
        };
        let Some(nonneg_target) =
            signed_cmp_const_nonneg_edge(*kind, v_is_lhs, c, *then_target, *else_target)
        else {
            continue;
        };
        let sole_pred = cfg
            .predecessors
            .get(&nonneg_target)
            .is_some_and(|preds| preds.len() == 1 && preds.contains(&block.id));
        if sole_pred && dominates(nonneg_target, header, &idom) {
            return true;
        }
    }
    false
}

/// Is value `v` provably a **non-negative** integer on entry to a loop whose header
/// is `header` — either an unconditionally non-negative *source*
/// ([`arg_is_nonneg`]: constant, unsigned nondet, `zext`, or a non-negative
/// arithmetic result) or gated by a *dominating signed guard* forcing `v >= 0` that
/// reaches the header ([`value_dominating_nonneg`], e.g. an early `if (v < 0)
/// return;` / `if (!(v >= 0))`)? Used to establish the **base case** of a header
/// phi's `phi >= 0` loop invariant in [`inductive_nonneg_header_phis`]. Fails closed
/// (only recall is affected).
fn value_is_entry_nonneg(
    v: ValueId,
    func: &AirFunction,
    header: BlockId,
    module: &AirModule,
) -> bool {
    arg_is_nonneg(v, func, module) || value_dominating_nonneg(v, func, header, module)
}

/// The header phis `p` for which `p >= 0` is a **sound inductive loop invariant**:
///
/// - **base case** — every *preheader* (non-latch) incoming value of `p` is provably
///   non-negative ([`value_is_entry_nonneg`]); and
/// - **inductive step** — every split-expanded, type-bounded continuing branch
///   *preserves* it: `region ∧ p ≥ 0 ⇒ next(p) ≥ 0` (a Farkas/Z3 check, mirroring
///   [`assemble_branches`]'s split expansion and infeasible-cross-product pruning so
///   the proof uses exactly the regions the ranking will).
///
/// Adding a proven `p ≥ 0` to the region is sound — it is a true fact about every
/// continuing state, so the branch regions stay a *superset* of the real transition
/// relation (a ranking of the superset still soundly ranks the real loop). Its value
/// is that the `!=`/`==`-else split's **negative** half-space (`p ≤ rhs − 1`) then
/// becomes infeasible and is pruned, recovering a `while (x != 0) x--` (entry
/// `x ≥ 0`) style loop whose lower bound the non-convex `!=` guard alone cannot
/// supply. A phi whose `next` is a havoc (non-affine) on any feasible branch, or
/// whose `p ≥ 0` is not preserved (e.g. `while (0 <= j && ...) j--`, where `j = 0`
/// steps to `-1`), fails closed and is not returned — so a genuinely oscillating /
/// diverging loop is never spuriously bounded.
// NOTE: the arguments are the already-built pieces of the loop model
// (function/module, the loop's header + latch set + header phis, and the raw
// branches + base bounds + universe) — passing the assembled state avoids
// recomputing it, and bundling it into a struct would only obscure the check.
#[allow(clippy::too_many_arguments)]
fn inductive_nonneg_header_phis(
    func: &AirFunction,
    module: &AirModule,
    header: BlockId,
    latch_set: &BTreeSet<BlockId>,
    header_phis: &BTreeSet<ValueId>,
    raws: &[RawBranch],
    base_bounds: &[Constraint],
    universe: &BTreeSet<ValueId>,
) -> BTreeSet<ValueId> {
    let mut out = BTreeSet::new();
    let Some(header_block) = func.blocks.iter().find(|b| b.id == header) else {
        return out;
    };
    for &p in header_phis {
        // --- Base case: every preheader incoming value is provably non-negative. ---
        let mut has_preheader = false;
        let mut base_ok = true;
        for inst in &header_block.instructions {
            let Operation::Phi { incoming } = &inst.op else {
                continue;
            };
            if inst.dst != Some(p) {
                continue;
            }
            for (pred, v) in incoming {
                if latch_set.contains(pred) {
                    continue; // a back-edge value is the inductive step, not the base
                }
                has_preheader = true;
                if !value_is_entry_nonneg(*v, func, header, module) {
                    base_ok = false;
                }
            }
        }
        if !has_preheader || !base_ok {
            continue;
        }
        // --- Inductive step: `p >= 0` preserved by every split-expanded branch. ---
        let hyp = Constraint {
            coeffs: BTreeMap::from([(p, 1)]),
            constant: 0,
        };
        let mut preserved = true;
        'branches: for raw in raws {
            // Mirror `assemble_branches`: over the cap the splits are dropped (a
            // weaker/larger region), so the induction proof must use that same larger
            // region to stay valid for the branches actually assembled.
            let choices = if raw.splits.len() <= MAX_NE_SPLITS {
                split_half_space_choices(&raw.splits)
            } else {
                vec![Vec::new()]
            };
            for extra in choices {
                let mut region = raw.guards.clone();
                region.extend(base_bounds.iter().cloned());
                region.extend(extra);
                region.push(hyp.clone());
                // An infeasible sub-branch models no reachable state ⇒ vacuously
                // preserved (this is where the negative half of `p`'s own `!=`-split
                // drops out under `p >= 0`).
                if region_is_infeasible(&region, universe) {
                    continue;
                }
                // `next(p)` must be a concrete affine on this feasible branch — a
                // havoc `next` could take any in-range value, including a negative
                // one, so it does not preserve `p >= 0`.
                let Some(Some(a)) = raw.raw_next.get(&p) else {
                    preserved = false;
                    break 'branches;
                };
                // region ⇒ a >= 0, i.e. `region ∧ (a <= -1)` is unsatisfiable.
                // `a <= -1` ⟺ `-a - 1 >= 0`.
                let Some(neg) = a.scale(-1).and_then(|na| na.add(&Affine::constant(-1))) else {
                    preserved = false;
                    break 'branches;
                };
                let mut check = region;
                check.push(Constraint {
                    coeffs: neg.terms,
                    constant: neg.constant,
                });
                if !region_is_infeasible(&check, universe) {
                    preserved = false;
                    break 'branches;
                }
            }
        }
        if preserved {
            out.insert(p);
        }
    }
    out
}

/// Parameter **positions** of a recursion SCC that are provably fed only
/// non-negative unsigned values at every call site **from outside the SCC** (the
/// base case of the recursion). Used to give an otherwise sign-`Unknown`
/// recursion parameter the sound unsigned range `[0, 2^w−1]`, which the existing
/// `!=`-split + overflow-check machinery needs to rank a `-O0`/unsigned counter
/// such as `id(unsigned x){ if(x==0) return; return id(x-1); }` (its `x` carries
/// no local signedness hint, so `infer_signs` leaves it `Unknown`).
///
/// # Soundness
///
/// A position `i` is returned only when **every** external (non-SCC) call site to
/// **any** SCC member passes an argument at position `i` that is provably
/// non-negative — either from an unsigned *source* ([`arg_is_nonneg`]) or gated by
/// a *dominating signed guard* `arg >= 0` at the call ([`arg_is_guarded_nonneg`]) —
/// establishing the invariant "position `i` is a non-negative value" at the base
/// (entry) of the recursion. That the invariant is *preserved* across recursive
/// frames is a tautology on the register's unsigned interpretation: position `i`
/// is a `w`-bit register always in `[0, 2^w−1]`; the caller only marks such
/// positions `Unsigned` (never `Signed`), and the ranking model's guards on them
/// are sign-agnostic (`==`/`!=`; a signed comparison would have made `infer_signs`
/// return `Signed`, which the caller preserves). The affine transitions are
/// overflow-checked against `[0, 2^w−1]` and demoted to a free in-range havoc on
/// any wrap, so the model over-approximates the unsigned machine semantics — a
/// ranking of it is a sound termination proof. Crucially the overflow check is
/// self-protecting: a decrement that could carry the value below `0` (the only way
/// a genuinely-signed entry's `sub nsw` could reach signed-overflow UB) fails the
/// `[0, 2^w−1]`-stability test and demotes to havoc ⇒ that half abstains, so a
/// recursion that only terminates for negative inputs is never ranked. With **no**
/// external call site the base case is unestablished, so the empty set is returned
/// (fail-closed).
fn scc_positions_unsigned_by_entry(
    module: &AirModule,
    scc: &BTreeSet<FunctionId>,
) -> BTreeSet<u32> {
    let mut sites: Vec<(&AirFunction, BlockId, &Vec<ValueId>)> = Vec::new();
    for caller in &module.functions {
        if scc.contains(&caller.id) {
            continue; // an intra-SCC (recursive) call is the inductive step, not a base entry
        }
        for block in &caller.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if scc.contains(callee) {
                        sites.push((caller, block.id, &inst.operands));
                    }
                }
            }
        }
    }
    if sites.is_empty() {
        return BTreeSet::new();
    }
    let max_len = sites.iter().map(|(_, _, a)| a.len()).max().unwrap_or(0);
    let mut out = BTreeSet::new();
    for i in 0..max_len {
        let all_nonneg = sites.iter().all(|(caller, call_block, args)| {
            args.get(i).is_some_and(|&a| {
                arg_is_nonneg(a, caller, module)
                    || arg_is_guarded_nonneg(a, caller, *call_block, module)
            })
        });
        if all_nonneg {
            // INVARIANT: LLVM caps a call's positional argument count well below 2^32.
            #[allow(clippy::cast_possible_truncation)]
            out.insert(i as u32);
        }
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

/// Is the conjunction of `region` constraints (`Σ aᵢ·zᵢ + b ≥ 0`) provably
/// **unsatisfiable** (an empty region)? Returns `true` only on a definitive Z3
/// `Unsat`; a `Sat` or `Unknown` (or any un-encodable constraint) returns `false`
/// so the caller keeps the branch — dropping a branch is only sound when its
/// region is *certainly* empty.
fn region_is_infeasible(region: &[Constraint], universe: &BTreeSet<ValueId>) -> bool {
    let index: BTreeMap<ValueId, usize> = universe.iter().copied().zip(0..).collect();
    let vars: BTreeMap<usize, z3::ast::Int> = (0..universe.len())
        .map(|i| (i, z3::ast::Int::new_const(sym_name(i))))
        .collect();
    let solver = new_solver();
    for c in region {
        let Some(b) = constraint_to_z3(c, &index, &vars) else {
            return false; // un-encodable ⇒ cannot prove empty ⇒ keep the branch.
        };
        solver.assert(&b);
    }
    matches!(solver.check(), z3::SatResult::Unsat)
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
    /// Disjunctive `!=` / `==`-else half-space splits collected along this path
    /// (each entry: the two convex half-spaces whose union is the guard region).
    /// [`assemble_branches`] forks the branch into the cartesian product of
    /// half-space choices, requiring the ranking to hold on every one.
    splits: Vec<[Constraint; 2]>,
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
    let all: Vec<usize> = (0..model.branches.len()).collect();
    // Fast path: a single lexicographic tuple that ranks every branch jointly (the
    // classic Cook/BMS synthesis). Covers all previously-ranked programs at no extra
    // Z3 cost, so it can never regress a `true`.
    if greedy_lex_subset(model, &all) {
        return true;
    }
    // Disjunctive fallback (only when the joint tuple fails — so zero cost on the
    // common case): decompose the branch "can-immediately-follow" graph into SCCs
    // and rank each *cyclic* SCC independently. See [`disjunctive_scc_rank`].
    disjunctive_scc_rank(model, &all)
}

/// Greedy lexicographic synthesis restricted to a subset of branch indices. Each
/// round finds one `f` that is bounded (`f ≥ 0`) and non-increasing on every
/// remaining branch in the subset and strictly decreasing on one of them; that
/// branch is removed. All removed within [`MAX_LEX_ROUNDS`] ⇒ the subset jointly
/// admits a lexicographic ranking. An empty subset is vacuously ranked (no cyclic
/// branch to bound). Sound because [`synthesize_round`] over a subset is exactly
/// the joint-ranking soundness condition applied to those branches only.
fn greedy_lex_subset(model: &MultiPathModel, indices: &[usize]) -> bool {
    let mut remaining: Vec<usize> = indices.to_vec();
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

/// Maximum branch count for the disjunctive SCC fallback. The fallback is
/// `O(n²)` Z3 feasibility checks; above this cap it abstains (recall-only — a
/// larger model simply falls back to `unknown`, never to a wrong verdict).
const MAX_DISJUNCTIVE_BRANCHES: usize = 16;

/// Disjunctive termination via a **strongly-connected-component decomposition** of
/// the branch *can-immediately-follow* graph (a Podelski–Rybalchenko disjunctive
/// / transition-invariant argument that a single lexicographic tuple cannot
/// express).
///
/// Build a directed graph `E` over the branches where `i → j` iff branch `j` can
/// fire immediately after branch `i` ([`branch_can_follow`], a sound
/// over-approximation — an edge is present unless *provably* infeasible). Any
/// infinite execution is an infinite walk in `E`, whose tail is confined to a
/// single strongly-connected component (finitely many branches). So if **every
/// cyclic SCC** (a component that contains a cycle — size ≥ 2, or a singleton with
/// a self-edge) admits its own lexicographic ranking, no infinite execution can
/// exist and the loop/recursion terminates. A singleton SCC with no self-edge can
/// only occur finitely often, so it needs no ranking.
///
/// # Soundness
///
/// `E` never drops a real edge: [`branch_can_follow`] keeps `i → j` unless the
/// *superset* region of `j` is UNSAT on the *superset* post-state of `i`, so every
/// real transition sequence is a walk in `E`. Hence the SCCs are a superset of the
/// real recurrent branch sets, and ranking each SCC is at least as strong as
/// required — no wrong `true`. The classic ping-pong counterexample (two branches
/// each individually well-founded but jointly divergent) is rejected because its
/// cross edges are feasible ⇒ the two branches share one SCC ⇒ they must rank
/// *jointly*, which they cannot.
fn disjunctive_scc_rank(model: &MultiPathModel, indices: &[usize]) -> bool {
    let n = indices.len();
    if n == 0 || n > MAX_DISJUNCTIVE_BRANCHES {
        return false;
    }
    // Adjacency (positions 0..n map to `indices[pos]`), including self-edges.
    let mut adj = vec![vec![false; n]; n];
    for (a, row) in adj.iter_mut().enumerate() {
        for (b, cell) in row.iter_mut().enumerate() {
            *cell = branch_can_follow(model, indices[a], indices[b]);
        }
    }
    // Reachability over paths of length ≥ 1 (Floyd–Warshall transitive closure).
    let mut reach = adj;
    for k in 0..n {
        for i in 0..n {
            if reach[i][k] {
                for j in 0..n {
                    if reach[k][j] {
                        reach[i][j] = true;
                    }
                }
            }
        }
    }
    // Rank each cyclic SCC. A branch `i` is on a cycle iff `reach[i][i]`; its SCC
    // is `{ j : reach[i][j] ∧ reach[j][i] }`. Non-cyclic singletons need no rank.
    let mut done = vec![false; n];
    for i in 0..n {
        if done[i] {
            continue;
        }
        if !reach[i][i] {
            done[i] = true; // non-recurrent singleton — occurs finitely often.
            continue;
        }
        let mut scc: Vec<usize> = Vec::new();
        for j in 0..n {
            if reach[i][j] && reach[j][i] {
                done[j] = true;
                scc.push(indices[j]);
            }
        }
        if !greedy_lex_subset(model, &scc) {
            return false;
        }
    }
    true
}

/// Can branch `j` fire *immediately after* branch `i`? A sound
/// **over-approximation** for [`disjunctive_scc_rank`]: returns `true` unless the
/// follow-on is provably infeasible (Z3 `Unsat`). Any `Sat`/`Unknown`/encoding
/// failure keeps the edge — a kept edge only enlarges an SCC, making the
/// disjunctive proof strictly harder (fail-closed), never spuriously easier.
///
/// Encoding: pre-state variables satisfy `region_i`; the post-state applies
/// branch `i`'s transition (each ranking-state phi `m` takes `next_i(m)` over the
/// pre-state; loop-invariant template symbols are unchanged; every other leaf is a
/// fresh unconstrained value — a sound loosening). The edge is infeasible iff
/// `region_j` is UNSAT on that post-state.
fn branch_can_follow(model: &MultiPathModel, i: usize, j: usize) -> bool {
    let bi = &model.branches[i];
    let bj = &model.branches[j];
    let index: BTreeMap<ValueId, usize> = model.universe.iter().copied().zip(0..).collect();
    let n = model.universe.len();
    let pre: BTreeMap<usize, z3::ast::Int> = (0..n)
        .map(|k| (k, z3::ast::Int::new_const(format!("pre_{k}"))))
        .collect();

    let solver = new_solver();
    // region_i(pre) ≥ 0.
    for c in &bi.region {
        let Some(b) = constraint_to_z3(c, &index, &pre) else {
            return true; // encoding failure ⇒ keep the edge (fail-closed).
        };
        solver.assert(&b);
    }
    // Post-state: phi ↦ next_i(pre); invariant template symbol ↦ itself; else fresh.
    let mut post: BTreeMap<usize, z3::ast::Int> = BTreeMap::new();
    for (m, &idx) in &index {
        let expr = if let Some(nxt) = bi.next.get(m) {
            match affine_to_z3(nxt, &index, &pre) {
                Some(e) => e,
                None => return true, // encoding failure ⇒ keep the edge.
            }
        } else if model.template.contains(m) {
            // Loop-invariant template symbol: unchanged across the step.
            pre[&idx].clone()
        } else {
            // Havoc leaf (re-read each iteration): a fresh unconstrained value.
            z3::ast::Int::new_const(format!("post_{idx}"))
        };
        post.insert(idx, expr);
    }
    // region_j(post) ≥ 0.
    for c in &bj.region {
        let Some(b) = constraint_to_z3(c, &index, &post) else {
            return true;
        };
        solver.assert(&b);
    }
    // Infeasible only on a definitive UNSAT; Sat/Unknown keep the edge.
    !matches!(solver.check(), z3::SatResult::Unsat)
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

    let mut signs = infer_signs(func, module);

    // Interprocedural non-negativity: a self-recursive parameter with no local
    // signedness hint (`Unknown`) — e.g. an unsigned `-O0` counter such as
    // `id(unsigned x){ if(x==0) return; return id(x-1); }` — is given the sound
    // unsigned range `[0, 2^w−1]` when every external call site feeds it a
    // provably-non-negative unsigned value. This is exactly the fact the
    // `!=`-split needs to prove `x − 1` never underflows so `f = x` ranks. Only
    // `Unknown` positions are lifted; a `Signed` inference is preserved (never
    // overridden) to keep signed guards interpreted consistently.
    let self_scc: BTreeSet<FunctionId> = BTreeSet::from([func.id]);
    let unsigned_positions = scc_positions_unsigned_by_entry(module, &self_scc);
    for p in &func.params {
        if unsigned_positions.contains(&p.index) && !matches!(signs.get(&p.id), Some(Sign::Signed))
        {
            signs.insert(p.id, Sign::Unsigned);
        }
    }

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
            let mut splits: Vec<[Constraint; 2]> = Vec::new();
            // Necessary stay-conditions of the branches entering the call block.
            for pair in path.windows(2) {
                if let Some(block) = func.blocks.iter().find(|b| b.id == pair[0]) {
                    add_path_guards(
                        &mut guards,
                        &mut splits,
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
            // Drop an infeasible entry→call path (a short-circuit `&&`/`||` guard's
            // exit shortcut): it models no reachable recursive step, and skipping it
            // keeps the feasible-path count small.
            if path_guard_is_unsat(&guards) {
                continue;
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
            raws.push(RawBranch {
                guards,
                splits,
                raw_next,
            });
            if raws.len() > MAX_PATHS {
                return None; // too many *feasible* entry→call paths ⇒ abstain.
            }
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

    // Give every sign-`Unknown`, `nsw`-free, Eq/Ne-only leaf the sound `[0, 2^w−1]`
    // bit-pattern bound. This is what lets a recursion whose only stopping guard is
    // a `!=`/`==` on a defined-wraparound counter — e.g. `int id(int x){ if(x==0)
    // return 0; return id((unsigned)x-1)+1; }`, which terminates for *every* input
    // via the well-defined unsigned decrement — rank: the `!=` split's negative half
    // becomes infeasible and `x − 1` stays in range. A signed `sub nsw x, 1` counter
    // stays `Unknown` (ineligible) ⇒ abstains, as it must.
    let ineligible = bitpattern_ineligible_leaves(func, &defs, &header_phis, module);
    promote_bitpattern_unsigned(&mut signs, &universe, &ineligible);

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

    // --- Pass 2: expand disjunctive splits, classify each argument transition
    // (affine-stable vs havoc), and assemble the final per-branch regions.
    let branches = assemble_branches(raws, &base_bounds, &bound_of, &mut universe);

    let template: BTreeSet<ValueId> = param_ids.iter().chain(invariants.iter()).copied().collect();
    Some(MultiPathModel {
        template,
        universe,
        branches,
    })
}

// ---------------------------------------------------------------------------
// Mutual recursion ranking (an SCC of ≥2 mutually-recursive functions)
// ---------------------------------------------------------------------------

/// Does an SCC of **mutually-recursive** functions (`scc`, a set of ≥2 function
/// ids forming a single call-graph strongly-connected component) admit a linear
/// ranking function that bounds the recursion depth over the whole cycle (⇒ the
/// mutual recursion terminates)?
///
/// The whole SCC is modelled as one [`MultiPathModel`] over a **positionally
/// unified** ranking state: because every member has an *identical* parameter
/// signature (enforced below), parameter *position* `p` is identified with a
/// single ranking symbol shared across all members. Each model branch is one
/// **entry → intra-SCC-call-site path** of some member `fi` calling some member
/// `fj`: its guard is the conjunction of the path's branch stay-conditions (a
/// necessary condition on any frame reaching that call — an over-approximation),
/// and its `next` maps each position symbol to the affine form of the
/// corresponding call argument passed to `fj` (or a type-bounded havoc symbol
/// when that argument is not exactly affine). A ranking `f` that is `≥ 0` on
/// every call's guard and strictly decreases by `≥ 1` from a caller frame to the
/// callee frame it spawns bounds every root-to-leaf chain of intra-SCC calls;
/// with finitely many call sites the whole mutual-recursion call tree is finite
/// ⇒ it terminates.
///
/// # Soundness
///
/// Choosing a *single* formula over parameter positions is one valid (if
/// restrictive) ranking assignment for the SCC's configuration graph — it can
/// only make the search less complete, never unsound. Every branch's transition
/// relation over-approximates the real caller→callee step (guards are only
/// necessary conditions; each affine `next` is overflow-checked to equal the
/// machine value, else demoted to a free in-range havoc; all other leaves are
/// universally quantified), so a lexicographic ranking of the modelled
/// (superset) relation ranks the real one. `greedy_lex_rank`'s `Sat`-only Farkas
/// synthesis therefore never yields a wrong `true`. Abstains (`false`) on any
/// member that is a declaration, has a loop (entry→call paths not enumerable),
/// has a non-integer or mismatched parameter signature, or whose transitions are
/// not affine.
#[must_use]
pub fn mutual_recursion_is_ranked(module: &AirModule, scc: &BTreeSet<FunctionId>) -> bool {
    match build_mutual_recursion_model(module, scc) {
        Some(model) => greedy_lex_rank(&model),
        None => false,
    }
}

/// Remap an affine form's leaf symbols through `map` (identity for absent keys),
/// merging coefficients that collide onto the same target symbol.
fn remap_affine(a: &Affine, map: &BTreeMap<ValueId, ValueId>) -> Affine {
    let mut terms: BTreeMap<ValueId, i128> = BTreeMap::new();
    for (&s, &c) in &a.terms {
        let key = map.get(&s).copied().unwrap_or(s);
        let e = terms.entry(key).or_insert(0);
        *e = e.saturating_add(c);
    }
    terms.retain(|_, c| *c != 0);
    Affine {
        terms,
        constant: a.constant,
    }
}

/// Remap a constraint's leaf symbols through `map` (see [`remap_affine`]).
fn remap_constraint(c: &Constraint, map: &BTreeMap<ValueId, ValueId>) -> Constraint {
    let mut coeffs: BTreeMap<ValueId, i128> = BTreeMap::new();
    for (&s, &k) in &c.coeffs {
        let key = map.get(&s).copied().unwrap_or(s);
        let e = coeffs.entry(key).or_insert(0);
        *e = e.saturating_add(k);
    }
    coeffs.retain(|_, v| *v != 0);
    Constraint {
        coeffs,
        constant: c.constant,
    }
}

/// Order a function's parameters by their positional `index`.
fn ordered_params(f: &AirFunction) -> Vec<&AirParam> {
    let mut v: Vec<&AirParam> = f.params.iter().collect();
    v.sort_by_key(|p| p.index);
    v
}

/// Build the mutual-recursion [`MultiPathModel`] for the SCC `scc`, or `None`
/// (⇒ abstain) when its shape is unsupported. See [`mutual_recursion_is_ranked`].
// NOTE: one cohesive extraction pipeline (validate members → unify param
// positions → collect intra-SCC call paths → per-path guards/transitions,
// remapped to the shared vocabulary → classify overflow → assemble), mirroring
// `build_recursion_model`; splitting it would only scatter the shared state.
#[allow(clippy::too_many_lines)]
fn build_mutual_recursion_model(
    module: &AirModule,
    scc: &BTreeSet<FunctionId>,
) -> Option<MultiPathModel> {
    if scc.len() < 2 {
        return None;
    }
    // Collect member functions (deterministic canonical order = by id).
    let mut members: Vec<&AirFunction> = module
        .functions
        .iter()
        .filter(|f| scc.contains(&f.id))
        .collect();
    if members.len() != scc.len() {
        return None; // a member is missing from the module (should not happen)
    }
    members.sort_by_key(|f| f.id);
    for f in &members {
        if f.is_declaration {
            return None;
        }
    }
    // Loop-free bodies: entry→call-site path enumeration requires acyclic CFGs.
    let cfgs: Vec<Cfg> = members.iter().map(|f| Cfg::build(f)).collect();
    if cfgs.iter().any(cfg_has_loops) {
        return None;
    }

    // Require identical parameter signatures across all members — same `index` set
    // and the same `TypeId` at each index — so parameter *position* is a
    // well-defined, type-consistent ranking symbol across the SCC. Abstain else.
    let canonical = members[0];
    let canon_params = ordered_params(canonical);
    for f in &members {
        let ps = ordered_params(f);
        if ps.len() != canon_params.len() {
            return None;
        }
        if canon_params
            .iter()
            .zip(ps.iter())
            .any(|(a, b)| a.index != b.index || a.param_type != b.param_type)
        {
            return None;
        }
    }

    // Integer positions carry a ranking symbol = the canonical member's param id.
    let mut param_ids: BTreeSet<ValueId> = BTreeSet::new();
    let mut param_type_of: BTreeMap<ValueId, TypeId> = BTreeMap::new();
    let mut shared_of_index: BTreeMap<u32, ValueId> = BTreeMap::new();
    for p in &canon_params {
        if let Some(ty) = p.param_type {
            if int_width(module, ty).is_some() {
                param_ids.insert(p.id);
                param_type_of.insert(p.id, ty);
                shared_of_index.insert(p.index, p.id);
            }
        }
    }
    if param_ids.is_empty() {
        return None;
    }

    // Per-member remap: this member's param at index `i` ↦ the shared symbol for
    // position `i` (integer positions only).
    let member_remap: Vec<BTreeMap<ValueId, ValueId>> = members
        .iter()
        .map(|f| {
            ordered_params(f)
                .iter()
                .filter_map(|p| shared_of_index.get(&p.index).map(|&s| (p.id, s)))
                .collect()
        })
        .collect();

    // Merged signedness over the shared symbols: a position keeps a concrete sign
    // only if every member that determines it agrees (else stays unbounded/free).
    let mut signs: BTreeMap<ValueId, Sign> = BTreeMap::new();
    let mut sign_conflict: BTreeSet<ValueId> = BTreeSet::new();
    for (i, f) in members.iter().enumerate() {
        for (vid, sign) in infer_signs(f, module) {
            if matches!(sign, Sign::Unknown) {
                continue;
            }
            if let Some(&shared) = member_remap[i].get(&vid) {
                match signs.get(&shared) {
                    None => {
                        signs.insert(shared, sign);
                    }
                    Some(&prev) if prev != sign => {
                        sign_conflict.insert(shared);
                    }
                    Some(_) => {}
                }
            }
        }
    }
    for c in &sign_conflict {
        signs.remove(c);
    }

    // Interprocedural non-negativity over the shared positional symbols: a
    // position with no local signedness hint (and no conflicting hint above) is
    // given the sound unsigned range `[0, 2^w−1]` when every call site entering
    // the SCC from outside feeds it a provably-non-negative unsigned value —
    // unlocking mutually-recursive unsigned counters (`id`/`id2`). A `Signed`
    // inference is preserved, never overridden.
    let unsigned_positions = scc_positions_unsigned_by_entry(module, scc);
    for p in &canon_params {
        if let Some(&shared) = shared_of_index.get(&p.index) {
            if unsigned_positions.contains(&p.index)
                && !matches!(signs.get(&shared), Some(Sign::Signed))
            {
                signs.insert(shared, Sign::Unsigned);
            }
        }
    }

    // --- Pass 1: one raw transition per (member × call site × entry→site path),
    // remapped into the shared positional vocabulary. ---------------------------
    let mut raws: Vec<RawBranch> = Vec::new();
    let mut universe: BTreeSet<ValueId> = param_ids.clone();
    for (mi, f) in members.iter().enumerate() {
        let cfg = &cfgs[mi];
        let remap = &member_remap[mi];
        let all_blocks: BTreeSet<BlockId> = f.blocks.iter().map(|b| b.id).collect();
        let defs = index_defs(f, &all_blocks);
        let block_of = index_block_of(f);
        // This member's own integer params are the affine leaves for resolution.
        let header_phis: BTreeSet<ValueId> = remap.keys().copied().collect();
        let entry = f.entry_block.unwrap_or(f.blocks.first()?.id);

        // Intra-SCC recursive call sites: a `CallDirect` to any SCC member.
        let mut sites: Vec<(BlockId, Vec<ValueId>)> = Vec::new();
        for block in &f.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if scc.contains(callee) {
                        sites.push((block.id, inst.operands.clone()));
                    }
                }
            }
        }

        for (cb, args) in &sites {
            let paths = enumerate_body_paths(cfg, &all_blocks, entry, *cb)?;
            for path in &paths {
                let path_pred = path_pred_map(path);
                let mut guards_local: Vec<Constraint> = Vec::new();
                let mut splits_local: Vec<[Constraint; 2]> = Vec::new();
                let mut local_universe: BTreeSet<ValueId> = BTreeSet::new();
                for pair in path.windows(2) {
                    if let Some(block) = f.blocks.iter().find(|b| b.id == pair[0]) {
                        add_path_guards(
                            &mut guards_local,
                            &mut splits_local,
                            &mut local_universe,
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
                // Drop an infeasible entry→call path (a short-circuit `&&`/`||`
                // guard's exit shortcut) before it contributes a vacuous branch.
                if path_guard_is_unsat(&guards_local) {
                    continue;
                }
                // Transition: shared symbol at position `p` ↦ affine of the call's
                // argument operand `p` (the callee's param `p`), remapped.
                let mut raw_next: BTreeMap<ValueId, Option<Affine>> = BTreeMap::new();
                for p in &canon_params {
                    let Some(&shared) = shared_of_index.get(&p.index) else {
                        continue;
                    };
                    let a = args.get(p.index as usize).and_then(|&arg| {
                        resolve_affine_path(
                            arg,
                            &defs,
                            &block_of,
                            &header_phis,
                            module,
                            &path_pred,
                            0,
                        )
                    });
                    let a = a.map(|aff| remap_affine(&aff, remap));
                    if let Some(ref aff) = a {
                        universe.extend(aff.terms.keys().copied());
                    }
                    raw_next.insert(shared, a);
                }
                let guards: Vec<Constraint> = guards_local
                    .iter()
                    .map(|c| remap_constraint(c, remap))
                    .collect();
                for c in &guards {
                    universe.extend(c.coeffs.keys().copied());
                }
                // Remap each `!=` split's two half-spaces into the shared vocabulary.
                let splits: Vec<[Constraint; 2]> = splits_local
                    .iter()
                    .map(|[a, b]| [remap_constraint(a, remap), remap_constraint(b, remap)])
                    .collect();
                for [a, b] in &splits {
                    universe.extend(a.coeffs.keys().copied());
                    universe.extend(b.coeffs.keys().copied());
                }
                raws.push(RawBranch {
                    guards,
                    splits,
                    raw_next,
                });
                if raws.len() > MAX_PATHS {
                    return None; // too many *feasible* entry→call paths ⇒ abstain.
                }
            }
        }
    }
    if raws.is_empty() {
        return None;
    }

    // Type bounds for the shared ranking symbols only (true facts about the integer
    // value at each position). Other leaves stay free (universally quantified).
    let mut base_bounds: Vec<Constraint> = Vec::new();
    let mut bound_of: BTreeMap<ValueId, (i128, i128)> = BTreeMap::new();
    for (&sym, &ty) in &param_type_of {
        if let Some((lo, hi)) = bounds_from_type(sym, ty, &signs, module) {
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

    // --- Pass 2: expand disjunctive splits, classify each argument transition
    // (affine-stable vs havoc), and assemble the final per-branch regions.
    let branches = assemble_branches(raws, &base_bounds, &bound_of, &mut universe);

    Some(MultiPathModel {
        template: param_ids,
        universe,
        branches,
    })
}

/// Build the path-sensitive [`MultiPathModel`] for a **single-latch** natural loop,
/// or `None` (⇒ abstain / fall back) when it is not in the supported shape.
fn build_multipath_model(
    func: &AirFunction,
    module: &AirModule,
    cfg: &Cfg,
    li: &LoopInfo,
) -> Option<MultiPathModel> {
    build_multipath_model_multi(
        func,
        module,
        cfg,
        li.header,
        &[li.latch],
        &li.body,
        &li.idom,
    )
}

/// Build the path-sensitive [`MultiPathModel`] for a natural loop with header
/// `header` and one *or more* back-edge latches (all `latch → header`), or `None`
/// (⇒ abstain / fall back) when it is not in the supported path-sensitive shape.
///
/// # Multi-latch soundness
///
/// A loop with several back-edges into one header (e.g. a `continue`, or the two
/// exits of a short-circuit `while (a && b)`) returns to the header via *exactly
/// one* back-edge each iteration. We enumerate every `header → latchᵢ` path across
/// **all** latches and record one [`Branch`] per (latch × path). A single
/// lexicographic ranking function found by [`greedy_lex_rank`] that is bounded and
/// strictly lex-decreasing on *every* branch therefore strictly decreases across
/// every possible iteration transition — so the loop runs finitely many times.
/// (This is why per-latch *independent* ranking is unsound but one *common*
/// ranking over the union is sound.)
// NOTE: this is one cohesive extraction pipeline (reject-nested → collect phis →
// enumerate paths across all latches → per-path guards/transitions → classify
// overflow → assemble); splitting it would only scatter the shared
// `defs`/`universe`/`bound_of` state.
#[allow(clippy::too_many_lines)]
fn build_multipath_model_multi(
    func: &AirFunction,
    module: &AirModule,
    cfg: &Cfg,
    header: BlockId,
    latches: &[BlockId],
    body: &BTreeSet<BlockId>,
    idom: &BTreeMap<BlockId, BlockId>,
) -> Option<MultiPathModel> {
    if latches.is_empty() {
        return None;
    }
    let latch_set: BTreeSet<BlockId> = latches.iter().copied().collect();
    let defs = index_defs(func, body);
    let block_of = index_block_of(func);
    let mut signs = infer_signs(func, module);

    // Reject a **nested inner loop**: the only dominance back-edges inside the body
    // may be this loop's own `latchᵢ → header` edges. Any other in-body back-edge is
    // an inner loop whose many iterations path enumeration would miss (unsound) —
    // bail (for single-latch, the caller falls back to the havoc model). (The
    // function is reducible — the extractor rejected irreducible CFGs — so every
    // in-body cycle has a dominance back-edge and is caught here.)
    for (&u, succs) in &cfg.successors {
        if !body.contains(&u) {
            continue;
        }
        for &v in succs {
            if body.contains(&v)
                && dominates(v, u, idom)
                && !(v == header && latch_set.contains(&u))
            {
                return None;
            }
        }
    }

    // Header phis (dsts). Each is resolved per latch to its back-edge incoming value.
    let header_block = func.blocks.iter().find(|b| b.id == header)?;
    let mut header_phis: BTreeSet<ValueId> = BTreeSet::new();
    for inst in &header_block.instructions {
        if let Operation::Phi { .. } = &inst.op {
            if let Some(dst) = inst.dst {
                header_phis.insert(dst);
            }
        }
    }
    if header_phis.is_empty() {
        return None;
    }

    // --- Pass 1: per-(latch × path) guards + raw affine transitions. -----------
    let mut raws: Vec<RawBranch> = Vec::new();
    let mut universe: BTreeSet<ValueId> = header_phis.clone();
    for &latch in latches {
        // This latch's back-edge incoming value for each header phi.
        let mut phi_next: BTreeMap<ValueId, ValueId> = BTreeMap::new();
        for inst in &header_block.instructions {
            if let Operation::Phi { incoming } = &inst.op {
                let Some(dst) = inst.dst else { continue };
                if let Some((_, v)) = incoming.iter().find(|(pred, _)| *pred == latch) {
                    phi_next.insert(dst, *v);
                }
            }
        }
        let latch_block = func.blocks.iter().find(|b| b.id == latch)?;
        // Enumerate every header→latch path of the (acyclic) body.
        let paths = enumerate_body_paths(cfg, body, header, latch)?;
        for path in &paths {
            let path_pred = path_pred_map(path);
            let mut guards: Vec<Constraint> = Vec::new();
            let mut splits: Vec<[Constraint; 2]> = Vec::new();
            // Necessary stay-conditions along the path's internal/exit branches …
            for pair in path.windows(2) {
                if let Some(block) = func.blocks.iter().find(|b| b.id == pair[0]) {
                    add_path_guards(
                        &mut guards,
                        &mut splits,
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
                &mut splits,
                &mut universe,
                latch_block,
                header,
                &defs,
                &block_of,
                &header_phis,
                module,
                &path_pred,
            );

            // Drop a path whose guard is unconditionally false (a short-circuit
            // `&&`/`||` exit shortcut). It models no reachable transition, and
            // skipping it here keeps the feasible-path count under `MAX_PATHS`.
            if path_guard_is_unsat(&guards) {
                continue;
            }

            let mut raw_next: BTreeMap<ValueId, Option<Affine>> = BTreeMap::new();
            for &phi in &header_phis {
                // A phi lacking a back-edge value for this latch cannot be modelled;
                // over-approximate it as an (unresolved) havoc below.
                let a = phi_next.get(&phi).and_then(|&nv| {
                    resolve_affine_path(nv, &defs, &block_of, &header_phis, module, &path_pred, 0)
                });
                if let Some(ref aff) = a {
                    universe.extend(aff.terms.keys().copied());
                }
                raw_next.insert(phi, a);
            }
            raws.push(RawBranch {
                guards,
                splits,
                raw_next,
            });
            if raws.len() > MAX_PATHS {
                return None;
            }
        }
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

    // Function parameters carry their declared type on [`AirParam`], not on any
    // instruction [`Def`], so [`type_bounds`] (which reads `defs`) misses them. Index
    // each param's `TypeId` so a param used as a loop bound (`while (x > z)` with `z`
    // a parameter) or a param-initialized counter still gets its type range in the
    // *path-sensitive* model — without it a loop whose guard/counter operands are
    // spilled-then-promoted parameters (the `-O0` recursion/argument reservoir) has
    // no bound on those leaves, the overflow-freedom check cannot bound the counter's
    // `next`, and the (otherwise trivially ranked) loop abstains. This mirrors the
    // fallback in [`build_loop_model`] (the havoc model already had it).
    let param_types: BTreeMap<ValueId, TypeId> = func
        .params
        .iter()
        .filter_map(|p| p.param_type.map(|t| (p.id, t)))
        .collect();

    // Give every sign-`Unknown`, `nsw`-free, Eq/Ne-only leaf the sound `[0, 2^w−1]`
    // bit-pattern bound so a defined-wraparound counter guarded only by `!=`/`==`
    // (`while (x != 0) x--` on an unsigned/`-fwrapv` `x`) ranks: the `!=` split's
    // negative half is pruned and `x − 1` stays in range. A signed `nsw` counter or
    // one feeding an ordered comparison stays `Unknown` ⇒ abstains, as it must.
    let ineligible = bitpattern_ineligible_leaves(func, &defs, &header_phis, module);
    promote_bitpattern_unsigned(&mut signs, &universe, &ineligible);

    // Type bounds for every integer leaf with a known signedness/width — true facts
    // about the real state, shared by every branch region.
    let mut base_bounds: Vec<Constraint> = Vec::new();
    let mut bound_of: BTreeMap<ValueId, (i128, i128)> = BTreeMap::new();
    for &sym in &universe {
        if let Some((lo, hi)) = type_bounds(sym, &defs, &signs, module).or_else(|| {
            param_types
                .get(&sym)
                .and_then(|&t| bounds_from_type(sym, t, &signs, module))
        }) {
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

    // Strengthen the region with sound inductive `phi >= 0` invariants derived from
    // an entry-guarded / non-negative-source preheader value. Each proven bound lets
    // the `!=`/`==`-else split's negative half-space be pruned as infeasible in
    // `assemble_branches`, recovering entry-guarded decrement loops (`while (x != 0)
    // x--`, entry `x >= 0`) whose lower bound the non-convex guard alone drops. This
    // must run *before* `assemble_branches` consumes `raws`.
    let nonneg_phis = inductive_nonneg_header_phis(
        func,
        module,
        header,
        &latch_set,
        &header_phis,
        &raws,
        &base_bounds,
        &universe,
    );
    for &p in &nonneg_phis {
        base_bounds.push(Constraint {
            coeffs: BTreeMap::from([(p, 1)]),
            constant: 0,
        });
        match bound_of.get_mut(&p) {
            // A sign-based type bound already exists ⇒ tighten its lower bound to `0`
            // so the overflow-freedom check and any havoc box use `[0, hi]`.
            Some(b) => b.0 = b.0.max(0),
            // No type bound (sign `Unknown`, the common case for a `==`/`!=`-only
            // counter). A proven-non-negative `w`-bit two's-complement value has a
            // clear sign bit, so it lies in `[0, 2^(w-1) - 1]`. Recording that sound
            // box lets `assemble_branches` classify the phi's affine `next` (e.g.
            // `x - 1`) as overflow-stable instead of demoting it to a havoc symbol
            // (which would erase the very decrease the ranking needs).
            None => {
                if let Some(w) = defs
                    .get(&p)
                    .and_then(|d| d.result_type)
                    .and_then(|t| int_width(module, t))
                    .map(i128::from)
                    .filter(|w| (1..=64).contains(w))
                {
                    let hi = (1i128 << (w - 1)) - 1;
                    bound_of.insert(p, (0, hi));
                    base_bounds.push(Constraint {
                        coeffs: BTreeMap::from([(p, -1)]),
                        constant: hi,
                    });
                }
            }
        }
    }

    // --- Pass 2: expand disjunctive splits, classify each transition
    // (affine-stable vs havoc), and assemble the final per-branch regions.
    let branches = assemble_branches(raws, &base_bounds, &bound_of, &mut universe);

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

/// A deterministic fresh **havoc** symbol for header phi `phi` on raw branch `bi`,
/// half-space-choice `si` (used when the phi's `next` on that path is non-affine or
/// may overflow). `si` keeps the two disjunctive sub-branches of one raw branch
/// from sharing a havoc symbol (each sub-branch havocs independently).
fn havoc_id(phi: ValueId, bi: usize, si: usize) -> ValueId {
    let mut bytes = format!("{phi:?}").into_bytes();
    bytes.extend_from_slice(&bi.to_le_bytes());
    bytes.extend_from_slice(&si.to_le_bytes());
    ValueId(make_id("ranking_havoc", &bytes))
}

/// Maximum disjunctive `!=` / `==`-else splits expanded per raw branch. Each split
/// forks the branch into two half-space branches (both must rank), so the branch
/// count grows by `2^(#splits)`; the cap bounds the Farkas work. Above it, the
/// splits are dropped (a sound weakening — the guard is simply not used).
const MAX_NE_SPLITS: usize = 3;

/// Expand a raw branch's disjunctive splits into the cartesian product of
/// half-space choices: one `Vec<Constraint>` of extra region constraints per
/// combination (`[Vec::new()]` when there are no splits). Deterministic order.
fn split_half_space_choices(splits: &[[Constraint; 2]]) -> Vec<Vec<Constraint>> {
    let mut out: Vec<Vec<Constraint>> = vec![Vec::new()];
    for pair in splits {
        let mut next: Vec<Vec<Constraint>> = Vec::with_capacity(out.len() * 2);
        for base in &out {
            for half in pair {
                let mut v = base.clone();
                v.push(half.clone());
                next.push(v);
            }
        }
        out = next;
    }
    out
}

/// Assemble the final [`Branch`] list from the raw per-path transitions, shared by
/// all three [`MultiPathModel`] builders (single/multi-latch loops, self-recursion,
/// mutual recursion).
///
/// For each raw branch it (1) expands the disjunctive `!=` / `==`-else splits into
/// the cartesian product of convex half-space choices — **both/every** resulting
/// sub-branch becomes a branch the ranking must cover, which is sound because their
/// union is a superset of the real continuing states on the path — then (2)
/// classifies every phi/param transition as affine-**stable** (its affine `next`
/// provably never wraps its type range on that sub-branch's region ⇒ affine model
/// == machine update) or **havoc** (a fresh type-bounded free symbol). Running the
/// overflow classification *per half-space* is the point: a half-space bound (e.g.
/// `n ≥ 1` from `n ≠ 0`) can prove a `next` (`n − 1`) overflow-free that is unstable
/// on the un-split region, recovering an otherwise-dropped ranking.
fn assemble_branches(
    raws: Vec<RawBranch>,
    base_bounds: &[Constraint],
    bound_of: &BTreeMap<ValueId, (i128, i128)>,
    universe: &mut BTreeSet<ValueId>,
) -> Vec<Branch> {
    let mut fresh_syms: BTreeSet<ValueId> = BTreeSet::new();
    let mut branches: Vec<Branch> = Vec::with_capacity(raws.len());
    for (bi, raw) in raws.into_iter().enumerate() {
        // Over the cap, drop the splits (sound weakening) rather than blow up.
        let choices = if raw.splits.len() <= MAX_NE_SPLITS {
            split_half_space_choices(&raw.splits)
        } else {
            vec![Vec::new()]
        };
        for (si, extra) in choices.into_iter().enumerate() {
            let mut check_region = raw.guards.clone();
            check_region.extend(base_bounds.iter().cloned());
            check_region.extend(extra);

            // A disjunctive `!=`-split can produce cross combinations whose region is
            // empty (e.g. `x ≥ 1` from one guard's half AND `x ≤ 0` from another's).
            // Such a sub-branch models no reachable continuing state, so dropping it
            // is sound — the remaining feasible sub-branches still union-cover every
            // real transition — and it keeps the branch count (hence the greedy
            // lexicographic round count) proportional to the *feasible* paths, which
            // is what lets a two-`!=`-guard recursion (e.g. `EvenOdd`: `n != 0` and
            // `n != 1`) rank within [`MAX_LEX_ROUNDS`]. Only a *definitive* `Unsat`
            // skips; an `Unknown` keeps the branch (fail-safe). Only split branches
            // can be infeasible cross-products, so the (no-split) common path — one
            // choice over the always-satisfiable base region — skips the SMT call.
            if !raw.splits.is_empty() && region_is_infeasible(&check_region, universe) {
                continue;
            }

            let mut next: BTreeMap<ValueId, Affine> = BTreeMap::new();
            let mut fresh_bounds: Vec<Constraint> = Vec::new();
            for (&sym, cand) in &raw.raw_next {
                let stable = match (cand, bound_of.get(&sym)) {
                    (Some(a), Some(&(lo, hi)))
                        if next_never_overflows(a, lo, hi, &check_region, universe) =>
                    {
                        Some(a.clone())
                    }
                    _ => None,
                };
                if let Some(a) = stable {
                    next.insert(sym, a);
                } else {
                    let fresh = havoc_id(sym, bi, si);
                    fresh_syms.insert(fresh);
                    if let Some(&(lo, hi)) = bound_of.get(&sym) {
                        fresh_bounds.push(Constraint {
                            coeffs: BTreeMap::from([(fresh, 1)]),
                            constant: -lo,
                        });
                        fresh_bounds.push(Constraint {
                            coeffs: BTreeMap::from([(fresh, -1)]),
                            constant: hi,
                        });
                    }
                    next.insert(sym, Affine::symbol(fresh));
                }
            }
            check_region.extend(fresh_bounds);
            branches.push(Branch {
                region: check_region,
                next,
            });
        }
    }
    universe.extend(fresh_syms);
    branches
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
/// Returns `None` if the *structural* path count would exceed [`MAX_ENUM_PATHS`]
/// (⇒ abstain). The caller then prunes infeasible paths ([`path_guard_is_unsat`])
/// and enforces the tighter [`MAX_PATHS`] cap on the surviving feasible paths.
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

/// DFS helper for [`enumerate_body_paths`]; returns `false` once the
/// [`MAX_ENUM_PATHS`] cap is exceeded (deterministic: successors iterate in sorted
/// `BTreeSet` order).
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
        return out.len() <= MAX_ENUM_PATHS;
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

/// Does `guards` contain an **unconditional contradiction** — a constraint
/// `Σ aᵢ·zᵢ + b ≥ 0` with *no* non-zero variable term and a negative constant
/// (`b < 0`), i.e. `b ≥ 0` is false for every state? Such a guard proves the whole
/// path is **infeasible**: no reachable state takes it.
///
/// [`resolve_bool_guard`] emits exactly this marker (`{}, −1`) when a *constant*
/// boolean condition contradicts the polarity the path takes — the archetype being
/// the **exit shortcut of a short-circuit `&&`/`||`**: `while (a && b && c)` lowers
/// to a chain of blocks whose early-exit edges re-enter the merge with the `i1` phi
/// pinned to `false`, so every path that leaves via a shortcut carries this
/// contradiction.
///
/// Skipping such a path is sound (it models no real transition, so dropping it
/// cannot hide a real non-terminating transition — it only removes a vacuously
/// rankable branch). Its purpose is budget: these phantom shortcut paths otherwise
/// multiply the feasible-path count past [`MAX_PATHS`], making an otherwise
/// path-enumerable loop/recursion abstain. Pruning them up front lets a loop
/// guarded by a multi-conjunct `&& … != …` (whose real body still has few paths)
/// fit under the cap so its feasible `!=`-split paths are modelled.
fn path_guard_is_unsat(guards: &[Constraint]) -> bool {
    guards
        .iter()
        .any(|c| c.constant < 0 && c.coeffs.values().all(|&v| v == 0))
}

/// Add the necessary stay-condition imposed by taking the `block → next_block`
/// edge (a `CondBr` in the taken polarity), path-sensitively resolved. Non-affine
/// conditions and non-`CondBr` terminators contribute nothing (a sound weakening).
#[allow(clippy::too_many_arguments)]
fn add_path_guards(
    region: &mut Vec<Constraint>,
    splits: &mut Vec<[Constraint; 2]>,
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
        splits,
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
    splits: &mut Vec<[Constraint; 2]>,
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
    let recurse = |region: &mut Vec<Constraint>,
                   splits: &mut Vec<[Constraint; 2]>,
                   universe: &mut BTreeSet<ValueId>,
                   v: ValueId,
                   want: bool| {
        resolve_bool_guard(
            region,
            splits,
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
            // A `!=` / `==`-else guard is non-convex: record its two convex
            // half-spaces for disjunctive expansion in `assemble_branches`.
            if let Some(pair) = ne_split(*kind, &lhs, &rhs, want_true) {
                for c in &pair {
                    universe.extend(c.coeffs.keys().copied());
                }
                splits.push(pair);
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
                recurse(region, splits, universe, *val, want_true);
            }
        }
        // Logical `and` taken true ⇒ both conjuncts true; logical `or` taken false
        // ⇒ both disjuncts false. The opposite directions are non-convex (drop).
        Operation::BinaryOp {
            kind: BinaryOp::And,
        } if want_true => {
            for &op in def.operands.iter().take(2) {
                recurse(region, splits, universe, op, true);
            }
        }
        Operation::BinaryOp { kind: BinaryOp::Or } if !want_true => {
            for &op in def.operands.iter().take(2) {
                recurse(region, splits, universe, op, false);
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
                        recurse(region, splits, universe, v, want_true ^ bit);
                    }
                }
                (_, Some(bit)) => {
                    if let Some(v) = a {
                        recurse(region, splits, universe, v, want_true ^ bit);
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
                recurse(region, splits, universe, src, want_true);
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
        /// The bound is a **function parameter** (`i32`): its `TypeId` lives on
        /// [`AirParam`], not on any instruction `Def`, so the loop model must read
        /// the param table to bound it (regression guard for that fix).
        Param,
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
            // A parameter bound is added to `func.params` below (no Def record).
            Bound::Param => (vid("param_bound"), vec![]),
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

        // When the bound is a parameter, declare it (typed `i32`) so the loop model
        // can recover its type range from the param table.
        let params = if matches!(bound, Bound::Param) {
            vec![AirParam {
                id: bound_v,
                name: None,
                index: 0,
                param_type: Some(i32t),
            }]
        } else {
            Vec::new()
        };

        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params,
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

    /// `n = nondet_int(); if (!(n >= 0)) return; x = n; while (x != c) x += step;`
    ///
    /// The header phi `x`'s preheader value is a **signed** nondet gated by a
    /// dominating `n >= 0` guard (emitted as `icmp sgt n, -1` when `neg_one_form`,
    /// else `icmp sge n, 0`), and `x` carries **no** signedness hint (used only in
    /// `!=` / `+`). This isolates the inductive `x >= 0` invariant: without it the
    /// non-convex `!=` guard supplies no lower bound and `x` has no type range, so
    /// the loop abstains; with it the `!=`-split's negative half-space is pruned and
    /// (for a decrement toward `0`) `f = x` ranks.
    fn guarded_ne_loop(step: i64, c: i64, guarded: bool, neg_one_form: bool) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });
        let mut constants = BTreeMap::new();

        let b0 = bid("entry");
        let pre = bid("pre");
        let h = bid("header");
        let l = bid("latch");
        let e = bid("exit");
        let early = bid("early");

        let n = vid("n"); // nondet preheader value
        let x = vid("x"); // header phi
        let xn = vid("xn"); // latch: x + step
        let gcmp = vid("gcmp"); // guard compare
        let gk = vid("gk"); // guard constant (0 or -1)
        let cval = vid("c"); // loop guard compare
        let cbound = vid("cbound");
        let step_v = vid("step");
        constants.insert(cbound, Constant::Int { value: c, bits: 32 });
        constants.insert(
            step_v,
            Constant::Int {
                value: step,
                bits: 32,
            },
        );
        constants.insert(
            gk,
            Constant::Int {
                value: if neg_one_form { -1 } else { 0 },
                bits: 32,
            },
        );

        // entry: n = nondet ; [if guarded: gcmp = n >(=) k ; condbr -> pre/early]
        let mut entry = AirBlock::new(b0);
        entry.instructions.push(vinst(
            "n_call",
            Operation::CallDirect {
                callee: FunctionId(make_id("func", b"__VERIFIER_nondet_int")),
            },
            n,
            vec![],
            i32t,
        ));
        if guarded {
            entry.instructions.push(vinst(
                "gcmp",
                Operation::BinaryOp {
                    kind: if neg_one_form {
                        BinaryOp::ICmpSgt
                    } else {
                        BinaryOp::ICmpSge
                    },
                },
                gcmp,
                vec![n, gk],
                i1t,
            ));
            entry.instructions.push(term(
                "g_condbr",
                Operation::CondBr {
                    then_target: pre,
                    else_target: early,
                },
                vec![gcmp],
            ));
        } else {
            entry
                .instructions
                .push(term("br_pre", Operation::Br { target: pre }, vec![]));
        }

        // pre: br header  (preheader — sole pred of header from the nonneg edge)
        let mut preb = AirBlock::new(pre);
        preb.instructions
            .push(term("br_header", Operation::Br { target: h }, vec![]));

        // header: phi x=[pre:n, latch:xn] ; c = icmp ne(x, cbound) ; condbr -> latch/exit
        let mut header = AirBlock::new(h);
        header.instructions.push(vinst(
            "phi_x",
            Operation::Phi {
                incoming: vec![(pre, n), (l, xn)],
            },
            x,
            vec![],
            i32t,
        ));
        header.instructions.push(vinst(
            "cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            cval,
            vec![x, cbound],
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

        // latch: xn = add nsw(x, step) ; br header. The counter derives from a signed
        // `nondet_int`, so the update is `add nsw` — a signed overflow is UB, which is
        // exactly why an *unguarded* `x` may not be read as an unsigned bit pattern
        // (it keeps no bound ⇒ the `!=` split abstains). The defined-wraparound
        // (plain-`add`) counterpart is built via [`strip_add_nsw`].
        let mut latch = AirBlock::new(l);
        latch.instructions.push(
            vinst(
                "add",
                Operation::BinaryOp {
                    kind: BinaryOp::Add,
                },
                xn,
                vec![x, step_v],
                i32t,
            )
            .with_no_signed_wrap(),
        );
        latch
            .instructions
            .push(term("br_latch", Operation::Br { target: h }, vec![]));

        let mut exit = AirBlock::new(e);
        exit.instructions.push(term("ret", Operation::Ret, vec![x]));
        let mut earlyb = AirBlock::new(early);
        earlyb
            .instructions
            .push(term("ret_early", Operation::Ret, vec![n]));

        let blocks = if guarded {
            vec![entry, preb, header, latch, exit, earlyb]
        } else {
            vec![entry, preb, header, latch, exit]
        };
        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks,
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

    #[test]
    fn guarded_nonneg_ne_decrement_is_ranked() {
        // if (n >= 0) { x = n; while (x != 0) x--; }  → f = x ranks.
        assert!(ranked(&guarded_ne_loop(-1, 0, true, false)));
        // `n > -1` (the instcombine `>= 0` form) must work identically.
        assert!(ranked(&guarded_ne_loop(-1, 0, true, true)));
    }

    #[test]
    fn unguarded_ne_decrement_abstains() {
        // NO entry guard and a *signed* `add nsw` decrement: `x` may start negative, so
        // `while (x != 0) x--` is UB / diverges for `x < 0` ⇒ the negative half-space
        // is feasible and cannot be ranked ⇒ abstain (no wrong `true`).
        assert!(!ranked(&guarded_ne_loop(-1, 0, false, false)));
    }

    #[test]
    fn unguarded_ne_decrement_plain_add_is_ranked() {
        // Same loop but with a *defined-wraparound* (plain `add`) decrement — an
        // unsigned / `-fwrapv` `while (x != 0) x--`. Even with NO entry guard, the
        // Eq/Ne-only, `nsw`-free `x` earns the `[0, 2^w−1]` bit-pattern bound: the
        // `!=` split's `x ≤ -1` half is pruned as infeasible and `x ≥ 1` ranks with
        // `f = x`. Terminates for every input ⇒ TRUE.
        assert!(ranked(&strip_add_nsw(guarded_ne_loop(-1, 0, false, false))));
    }

    #[test]
    fn guarded_nonneg_ne_increment_abstains() {
        // Entry `n >= 0` but `while (x != 0) x++`: non-terminating for `x > 0` (never
        // returns to `0`). The `x >= 1` half-space diverges (`x++` unbounded above),
        // so it cannot be ranked ⇒ abstain.
        assert!(!ranked(&guarded_ne_loop(1, 0, true, false)));
        // `while (x != 10) x++` with only `x >= 0` known: diverges for `x > 10` ⇒
        // the `x >= 11` half cannot be ranked ⇒ abstain.
        assert!(!ranked(&guarded_ne_loop(1, 10, true, false)));
    }

    #[test]
    fn even_start_ne_odd_target_abstains() {
        // `while (x != 5) x += 2` from an entry-nonneg start: the `x >= 6` half-space
        // diverges (`x += 2` unbounded above) ⇒ abstain (mandatory negative test).
        assert!(!ranked(&guarded_ne_loop(2, 5, true, false)));
    }

    #[test]
    fn signed_cmp_const_nonneg_edge_forms() {
        use BinaryOp::{ICmpSge, ICmpSgt, ICmpSle, ICmpSlt};
        let (t, e) = (bid("t"), bid("e"));
        // `v >= 0` / `v > -1` ⇒ THEN edge.
        assert_eq!(
            signed_cmp_const_nonneg_edge(ICmpSge, true, 0, t, e),
            Some(t)
        );
        assert_eq!(
            signed_cmp_const_nonneg_edge(ICmpSgt, true, -1, t, e),
            Some(t)
        );
        // `v < 0` / `v <= -1` ⇒ ELSE edge.
        assert_eq!(
            signed_cmp_const_nonneg_edge(ICmpSlt, true, 0, t, e),
            Some(e)
        );
        assert_eq!(
            signed_cmp_const_nonneg_edge(ICmpSle, true, -1, t, e),
            Some(e)
        );
        // Insufficient constants ⇒ no non-negativity edge.
        assert_eq!(signed_cmp_const_nonneg_edge(ICmpSge, true, -1, t, e), None);
        assert_eq!(signed_cmp_const_nonneg_edge(ICmpSgt, true, -2, t, e), None);
        // rhs form `k <p> v` swaps the predicate: `-1 < v` ⟺ `v > -1` ⇒ THEN.
        assert_eq!(
            signed_cmp_const_nonneg_edge(ICmpSlt, false, -1, t, e),
            Some(t)
        );
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
    fn count_up_strict_param_bound_is_ranked() {
        // for (i = 0; i < y; i++) where `y` is a FUNCTION PARAMETER → f = y - i.
        // Without the param-table type bound the `i + 1` overflow check cannot
        // bound `y ≤ INT_MAX`, judges the counter unstable, and abstains; with it,
        // this ranks like the `Bound::Nondet` sibling above.
        let m = counter_loop(BinaryOp::ICmpSlt, Bound::Param, 1);
        assert!(ranked(&m));
    }

    #[test]
    fn count_down_to_const_signed_is_ranked() {
        // while (i > 5) i--  (signed)  →  f = i - 5.
        let m = counter_loop(BinaryOp::ICmpSgt, Bound::Const(5), -1);
        assert!(ranked(&m));
    }

    /// Build `f(int x, int y, int z) { while (x > z && y > z) { x += step; y += step; } }`
    /// as a module. Both counters `x`,`y` are header phis whose preheader value is a
    /// **function parameter**, and the invariant lower bound `z` is a parameter too.
    /// The short-circuit `&&` splits the guard across two blocks (header + check2), so
    /// this is the path-sensitive multipath model's shape, NOT the single-guard havoc
    /// model. `z`'s type range lives only on `AirParam`, so without the param-table
    /// fallback in [`build_multipath_model_multi`] the overflow-freedom check cannot
    /// bound the decrement and the loop abstains.
    #[allow(clippy::too_many_lines)]
    fn compound_param_loop(step: i64) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });
        let mut constants = BTreeMap::new();
        let step_v = vid("step");
        constants.insert(
            step_v,
            Constant::Int {
                value: step,
                bits: 32,
            },
        );

        let b0 = bid("entry");
        let h = bid("header");
        let ch2 = bid("check2");
        let body = bid("body");
        let e = bid("exit");

        // Parameters (leaves with a type on `AirParam`, no `Def`).
        let px = vid("x");
        let py = vid("y");
        let pz = vid("z");
        // Header phis + latch increments + guard results.
        let xphi = vid("xphi");
        let yphi = vid("yphi");
        let xn = vid("xn");
        let yn = vid("yn");
        let c1 = vid("c1");
        let c2 = vid("c2");

        let mut entry = AirBlock::new(b0);
        entry
            .instructions
            .push(term("br_entry", Operation::Br { target: h }, vec![]));

        // header: phi x,y ; c1 = x sgt z ; condbr c1 -> check2/exit
        let mut header = AirBlock::new(h);
        header.instructions.push(vinst(
            "phi_x",
            Operation::Phi {
                incoming: vec![(b0, px), (body, xn)],
            },
            xphi,
            vec![],
            i32t,
        ));
        header.instructions.push(vinst(
            "phi_y",
            Operation::Phi {
                incoming: vec![(b0, py), (body, yn)],
            },
            yphi,
            vec![],
            i32t,
        ));
        header.instructions.push(vinst(
            "cmp1",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            c1,
            vec![xphi, pz],
            i1t,
        ));
        header.instructions.push(term(
            "condbr1",
            Operation::CondBr {
                then_target: ch2,
                else_target: e,
            },
            vec![c1],
        ));

        // check2: c2 = y sgt z ; condbr c2 -> body/exit
        let mut check2 = AirBlock::new(ch2);
        check2.instructions.push(vinst(
            "cmp2",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            c2,
            vec![yphi, pz],
            i1t,
        ));
        check2.instructions.push(term(
            "condbr2",
            Operation::CondBr {
                then_target: body,
                else_target: e,
            },
            vec![c2],
        ));

        // body: xn = x + step ; yn = y + step ; br header
        let mut bodyb = AirBlock::new(body);
        bodyb.instructions.push(vinst(
            "add_x",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            xn,
            vec![xphi, step_v],
            i32t,
        ));
        bodyb.instructions.push(vinst(
            "add_y",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            yn,
            vec![yphi, step_v],
            i32t,
        ));
        bodyb
            .instructions
            .push(term("br_body", Operation::Br { target: h }, vec![]));

        let mut exit = AirBlock::new(e);
        exit.instructions
            .push(term("ret", Operation::Ret, vec![xphi]));

        let params = vec![
            AirParam {
                id: px,
                name: None,
                index: 0,
                param_type: Some(i32t),
            },
            AirParam {
                id: py,
                name: None,
                index: 1,
                param_type: Some(i32t),
            },
            AirParam {
                id: pz,
                name: None,
                index: 2,
                param_type: Some(i32t),
            },
        ];

        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params,
            blocks: vec![entry, header, check2, bodyb, exit],
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

    #[test]
    fn compound_guard_param_bound_decreasing_is_ranked() {
        // while (x > z && y > z) { x--; y--; } with x,y,z FUNCTION PARAMETERS.
        // The short-circuit `&&` puts this on the path-sensitive multipath model,
        // which (before the param-table fallback) had no type range for the `z`
        // parameter and abstained despite the trivial rank `f = x - z`.
        let m = compound_param_loop(-1);
        assert!(
            ranked(&m),
            "param-bounded decreasing compound loop must rank"
        );
    }

    #[test]
    fn compound_guard_param_bound_increasing_not_ranked() {
        // while (x > z && y > z) { x++; y++; } — both counters move AWAY from the
        // lower bound `z`; the loop can run forever, so it must NOT rank (soundness:
        // the param bound must not manufacture a wrong `true`).
        let m = compound_param_loop(1);
        assert!(
            !ranked(&m),
            "increasing compound loop is non-terminating and must abstain"
        );
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

    /// Build `while (x != bound) x += step;` with `x` typed `i32`, plus a dead
    /// `x / one` op that hints `x`'s signedness (`unsigned` ⇒ `udiv`, else `sdiv`)
    /// so the loop var carries a concrete type range. Exercises the disjunctive
    /// `!=` half-space split.
    #[allow(clippy::too_many_lines)]
    fn ne_counter_loop(bound: i64, step: i64, unsigned: bool) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let b0 = bid("entry");
        let h = bid("header");
        let l = bid("latch");
        let e = bid("exit");

        let x = vid("x");
        let xn = vid("xn");
        let cval = vid("c");
        let dead = vid("dead");
        let x_init = vid("x_init");
        let step_v = vid("step");
        let bound_v = vid("bound_const");
        let one = vid("one");
        let mut constants = BTreeMap::new();
        constants.insert(x_init, Constant::Int { value: 0, bits: 32 });
        constants.insert(
            step_v,
            Constant::Int {
                value: step,
                bits: 32,
            },
        );
        constants.insert(
            bound_v,
            Constant::Int {
                value: bound,
                bits: 32,
            },
        );
        constants.insert(one, Constant::Int { value: 1, bits: 32 });

        let mut entry = AirBlock::new(b0);
        entry
            .instructions
            .push(term("br_entry", Operation::Br { target: h }, vec![]));

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
        // Signedness hint: `udiv`/`sdiv` marks `x` unsigned/signed for `infer_signs`.
        header.instructions.push(vinst(
            "hint",
            Operation::BinaryOp {
                kind: if unsigned {
                    BinaryOp::UDiv
                } else {
                    BinaryOp::SDiv
                },
            },
            dead,
            vec![x, one],
            i32t,
        ));
        header.instructions.push(vinst(
            "cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
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
        module_of(func, constants)
    }

    #[test]
    fn ne_zero_unsigned_countdown_ranked() {
        // while (x != 0) x -= 1;  with `x` unsigned. The `!=` split forks into the
        // half-space `x ≥ 1` (where `x − 1` never underflows ⇒ `f = x` ranks) and
        // `x ≤ -1` (infeasible under `x ≥ 0` ⇒ vacuously ranked). Both rank ⇒ TRUE.
        // Was abstained before the split: on the un-split region `x ≥ 0` the `x − 1`
        // step underflows at `x = 0` ⇒ havoc ⇒ unranked.
        let m = ne_counter_loop(0, -1, true);
        assert!(ranked(&m));
    }

    #[test]
    fn ne_five_step_two_diverging_half_abstains() {
        // while (x != 5) x += 2;  with `x` signed. The task's negative: from an even
        // start the loop never hits 5. The `!=` split's `x ≥ 6` half diverges
        // (`x += 2` grows, unbounded / overflows near INT_MAX) ⇒ that half cannot be
        // ranked ⇒ abstain. (The `x ≤ 4` half ranks, but BOTH are required.)
        let m = ne_counter_loop(5, 2, false);
        assert!(!ranked(&m));
    }

    #[test]
    fn ne_zero_signed_countdown_from_zero_ranked() {
        // while (x != 0) x -= 1;  with `x` *signed* and a constant-`0` preheader init
        // (`ne_counter_loop` seeds `x_init = 0`). Since `x` starts non-negative, the
        // inductive `x >= 0` invariant is established, the `!=`-split's `x ≤ -1` half
        // is pruned as infeasible, and the `x ≥ 1` half ranks with `f = x`. This is
        // sound and complete for THIS program: from `x = 0` the loop never enters, so
        // it terminates. (A genuinely-unbounded signed init — where `x < 0` really is
        // reachable — is the abstain case covered by `unguarded_ne_decrement_abstains`.)
        let m = ne_counter_loop(0, -1, false);
        assert!(ranked(&m));
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

    #[test]
    fn path_guard_is_unsat_detects_contradiction_only() {
        let x = vid("pgu_x");
        // `−1 ≥ 0` (the short-circuit marker) ⇒ unsatisfiable.
        assert!(path_guard_is_unsat(&[Constraint {
            coeffs: BTreeMap::new(),
            constant: -1,
        }]));
        // All-zero coefficients + negative constant ⇒ still unconditionally false.
        assert!(path_guard_is_unsat(&[Constraint {
            coeffs: BTreeMap::from([(x, 0)]),
            constant: -3,
        }]));
        // A real half-space (`x ≥ 1`) is satisfiable ⇒ not pruned.
        assert!(!path_guard_is_unsat(&[Constraint {
            coeffs: BTreeMap::from([(x, 1)]),
            constant: -1,
        }]));
        // `0 ≥ 0` (empty, non-negative constant) is trivially TRUE ⇒ not pruned.
        assert!(!path_guard_is_unsat(&[Constraint {
            coeffs: BTreeMap::new(),
            constant: 0,
        }]));
    }

    /// Build the LeikeHeizmann-TACAS2014-Ex9 loop
    /// `while (x > 0 && y > 0 && x != y) { if (x < y) x--; else if (y < x) y--; }`.
    /// Its **three**-conjunct short-circuit `&&` guard lowers to a merge block whose
    /// `i1` phi is `false` on the two early-exit shortcuts, so the header→latch path
    /// set is 3 (entry sub-paths) × 3 (body branches) = **9 structural paths** — over
    /// [`MAX_PATHS`]. Six carry the short-circuit `false` marker (infeasible) and one
    /// is the `x == y` stutter (excluded by the `!=` split); only two are feasible
    /// (`x--` under `x < y`, `y--` under `y < x`), jointly ranked by `f = x + y`.
    ///
    /// When `terminating` is false the `x < y` branch does `y = y + 1` instead — a
    /// genuinely non-terminating loop (`y` diverges while `x != y` stays true), the
    /// negative control that the infeasible-path pruning did not drop a *real* path.
    #[allow(clippy::too_many_lines)]
    fn leike_two_var(terminating: bool) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let zero = vid("l_zero");
        let one = vid("l_one");
        let false_c = vid("l_false");
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });
        constants.insert(false_c, Constant::Int { value: 0, bits: 1 });

        let entry = bid("l_entry");
        let h = bid("l_h");
        let h2 = bid("l_h2");
        let h3 = bid("l_h3");
        let g = bid("l_g");
        let body = bid("l_body");
        let pmid = bid("l_pmid");
        let pa = bid("l_pa");
        let pb = bid("l_pb");
        let pnone = bid("l_pnone");
        let latch = bid("l_latch");
        let exit = bid("l_exit");

        let (x, y, x0, y0, xn, yn) = (
            vid("l_x"),
            vid("l_y"),
            vid("l_x0"),
            vid("l_y0"),
            vid("l_xn"),
            vid("l_yn"),
        );
        let (cx, cy, cn, gp, cxy, cyx) = (
            vid("l_cx"),
            vid("l_cy"),
            vid("l_cn"),
            vid("l_gp"),
            vid("l_cxy"),
            vid("l_cyx"),
        );
        // pa's update: terminating ⇒ x-1 (y held); else ⇒ y+1 (x held).
        let pa_upd = vid("l_pa_upd");
        let pb_dec = vid("l_pb_dec");

        // entry: x0 = nondet; y0 = nondet; br h
        let mut eb = AirBlock::new(entry);
        eb.instructions
            .push(ndcall("l_ndx", "__VERIFIER_nondet_int", x0, i32t));
        eb.instructions
            .push(ndcall("l_ndy", "__VERIFIER_nondet_int", y0, i32t));
        eb.instructions
            .push(term("l_bre", Operation::Br { target: h }, vec![]));

        // h: phi x, phi y; cx = x > 0; condbr cx -> h2 else g
        let mut hb = AirBlock::new(h);
        hb.instructions.push(vinst(
            "l_phix",
            Operation::Phi {
                incoming: vec![(entry, x0), (latch, xn)],
            },
            x,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "l_phiy",
            Operation::Phi {
                incoming: vec![(entry, y0), (latch, yn)],
            },
            y,
            vec![],
            i32t,
        ));
        hb.instructions.push(vinst(
            "l_cmpx",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            cx,
            vec![x, zero],
            i1t,
        ));
        hb.instructions.push(term(
            "l_cbh",
            Operation::CondBr {
                then_target: h2,
                else_target: g,
            },
            vec![cx],
        ));

        // h2: cy = y > 0; condbr cy -> h3 else g
        let mut h2b = AirBlock::new(h2);
        h2b.instructions.push(vinst(
            "l_cmpy",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            cy,
            vec![y, zero],
            i1t,
        ));
        h2b.instructions.push(term(
            "l_cbh2",
            Operation::CondBr {
                then_target: h3,
                else_target: g,
            },
            vec![cy],
        ));

        // h3: cn = x != y; br g
        let mut h3b = AirBlock::new(h3);
        h3b.instructions.push(vinst(
            "l_cmpn",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            cn,
            vec![x, y],
            i1t,
        ));
        h3b.instructions
            .push(term("l_brh3", Operation::Br { target: g }, vec![]));

        // g: gp = phi [false,h],[false,h2],[cn,h3]; condbr gp -> body else exit
        let mut gb = AirBlock::new(g);
        gb.instructions.push(vinst(
            "l_phig",
            Operation::Phi {
                incoming: vec![(h, false_c), (h2, false_c), (h3, cn)],
            },
            gp,
            vec![],
            i1t,
        ));
        gb.instructions.push(term(
            "l_cbg",
            Operation::CondBr {
                then_target: body,
                else_target: exit,
            },
            vec![gp],
        ));

        // body: cxy = x < y; condbr cxy -> pa else pmid
        let mut bb = AirBlock::new(body);
        bb.instructions.push(vinst(
            "l_cxy",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            cxy,
            vec![x, y],
            i1t,
        ));
        bb.instructions.push(term(
            "l_cbb",
            Operation::CondBr {
                then_target: pa,
                else_target: pmid,
            },
            vec![cxy],
        ));

        // pmid: cyx = y < x; condbr cyx -> pb else pnone
        let mut pmb = AirBlock::new(pmid);
        pmb.instructions.push(vinst(
            "l_cyx",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            cyx,
            vec![y, x],
            i1t,
        ));
        pmb.instructions.push(term(
            "l_cbm",
            Operation::CondBr {
                then_target: pb,
                else_target: pnone,
            },
            vec![cyx],
        ));

        // pa: terminating ⇒ x-1; else ⇒ y+1. br latch
        let mut pab = AirBlock::new(pa);
        pab.instructions.push(vinst(
            "l_paupd",
            Operation::BinaryOp {
                kind: if terminating {
                    BinaryOp::Sub
                } else {
                    BinaryOp::Add
                },
            },
            pa_upd,
            if terminating {
                vec![x, one]
            } else {
                vec![y, one]
            },
            i32t,
        ));
        pab.instructions
            .push(term("l_brpa", Operation::Br { target: latch }, vec![]));

        // pb: y - 1; br latch
        let mut pbb = AirBlock::new(pb);
        pbb.instructions.push(vinst(
            "l_pbdec",
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            pb_dec,
            vec![y, one],
            i32t,
        ));
        pbb.instructions
            .push(term("l_brpb", Operation::Br { target: latch }, vec![]));

        // pnone: (x == y stutter, infeasible under x != y) br latch
        let mut pnb = AirBlock::new(pnone);
        pnb.instructions
            .push(term("l_brpn", Operation::Br { target: latch }, vec![]));

        // latch: xn = phi; yn = phi; br h
        let (pa_x, pa_y) = if terminating {
            (pa_upd, y)
        } else {
            (x, pa_upd)
        };
        let mut latb = AirBlock::new(latch);
        latb.instructions.push(vinst(
            "l_phixn",
            Operation::Phi {
                incoming: vec![(pa, pa_x), (pb, x), (pnone, x)],
            },
            xn,
            vec![],
            i32t,
        ));
        latb.instructions.push(vinst(
            "l_phiyn",
            Operation::Phi {
                incoming: vec![(pa, pa_y), (pb, pb_dec), (pnone, y)],
            },
            yn,
            vec![],
            i32t,
        ));
        latb.instructions
            .push(term("l_brlat", Operation::Br { target: h }, vec![]));

        let mut exb = AirBlock::new(exit);
        exb.instructions.push(term("l_ret", Operation::Ret, vec![]));

        let func = main_func(
            vec![eb, hb, h2b, h3b, gb, bb, pmb, pab, pbb, pnb, latb, exb],
            entry,
        );
        module_of(func, constants)
    }

    #[test]
    fn leike_two_var_nine_paths_ranked_after_pruning() {
        // 9 structural header→latch paths (> MAX_PATHS); the six short-circuit
        // shortcuts are pruned as infeasible, leaving the two feasible `x--`/`y--`
        // branches, jointly ranked by `f = x + y`. Without infeasible-path pruning
        // the raw path count exceeds the cap and the loop abstains.
        assert!(ranked(&leike_two_var(true)));
    }

    #[test]
    fn leike_two_var_nonterminating_abstains() {
        // The `x < y` branch now does `y = y + 1` (diverges) — no ranking function
        // exists. Pruning must NOT have dropped this real feasible path ⇒ abstain.
        assert!(!ranked(&leike_two_var(false)));
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
        // Signed `int` counter update `n + step` ⇒ `add nsw` (a signed overflow is
        // UB). The `nsw` flag keeps `n` ineligible for the unsigned bit-pattern
        // bound, so a `!=`-guarded signed decrement stays *unranked* (its negative
        // half cannot be pruned) — the soundness invariant this recursion model
        // preserves. The unsigned/defined-wraparound counterpart is built without
        // `nsw` (see `self_rec_plain_add`).
        rb.instructions.push(
            vinst(
                "add",
                Operation::BinaryOp {
                    kind: BinaryOp::Add,
                },
                na,
                vec![n, step_v],
                i32t,
            )
            .with_no_signed_wrap(),
        );
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

    /// Like [`self_rec`] but the counter update is a **plain** (non-`nsw`) `add` —
    /// a *defined-wraparound* (unsigned / `-fwrapv`) decrement. Guarded only by
    /// `!=`/`==`, such a counter reaches its bound for every input, so the
    /// disjunctive `!=` split ranks it once the leaf carries the `[0, 2^w−1]`
    /// bit-pattern bound. Models `int id(int x){ if(x==0) return 0;
    /// return id((unsigned)x-1)+1; }`.
    fn self_rec_plain_add(cmp: BinaryOp, bound: i64, step: i64) -> AirModule {
        strip_add_nsw(self_rec(cmp, bound, step))
    }

    /// Drop the `nsw` flag from every `add` in the module, turning a signed counter
    /// update into a *defined-wraparound* (unsigned / `-fwrapv`) one — the LLVM shape
    /// of `(unsigned)x ± k`. Used to build the plain-`add` termination fixtures.
    fn strip_add_nsw(mut m: AirModule) -> AirModule {
        for func in &mut m.functions {
            for block in &mut func.blocks {
                for inst in &mut block.instructions {
                    if matches!(
                        inst.op,
                        Operation::BinaryOp {
                            kind: BinaryOp::Add
                        }
                    ) {
                        inst.extensions.remove("llvm.nsw");
                    }
                }
            }
        }
        m
    }

    fn rec_ranked(m: &AirModule) -> bool {
        let func = &m.functions[0];
        let cfg = Cfg::build(func);
        recursion_is_ranked(func, m, &cfg)
    }

    /// `self_rec` (recursive `f`) plus an external `main` that calls `f(entry)`,
    /// where `entry` is either the result of the nondet source `nondet_name`
    /// (a declaration) or, when `nondet_name` is empty, the constant `arg_const`.
    /// This exercises the interprocedural non-negativity path: `f`'s parameter has
    /// no local signedness hint, so it is ranked only when the external call site
    /// proves it unsigned.
    fn self_rec_called(
        cmp: BinaryOp,
        bound: i64,
        step: i64,
        nondet_name: &str,
        arg_const: i64,
    ) -> AirModule {
        let mut m = self_rec(cmp, bound, step);
        let i32t = tid("i32");
        let f_id = FunctionId(make_id("func", b"f"));

        let inp = vid("main_inp");
        let mut main_entry = AirBlock::new(bid("main_entry"));
        let arg = if nondet_name.is_empty() {
            let cst = vid("main_arg_const");
            m.constants.insert(
                cst,
                Constant::Int {
                    value: arg_const,
                    bits: 32,
                },
            );
            cst
        } else {
            let nondet_id = FunctionId(make_id("func", nondet_name.as_bytes()));
            m.functions.push(AirFunction {
                id: nondet_id,
                name: nondet_name.to_string(),
                params: Vec::new(),
                blocks: Vec::new(),
                entry_block: None,
                is_declaration: true,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            });
            main_entry.instructions.push(vinst(
                "main_call_nondet",
                Operation::CallDirect { callee: nondet_id },
                inp,
                vec![],
                i32t,
            ));
            inp
        };
        main_entry.instructions.push(term(
            "main_call_f",
            Operation::CallDirect { callee: f_id },
            vec![arg],
        ));
        main_entry
            .instructions
            .push(term("main_ret", Operation::Ret, vec![]));

        m.functions.push(AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![main_entry],
            entry_block: Some(bid("main_entry")),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        m
    }

    /// Like [`self_rec_called`], but the entry argument is `f(inp <op> k)` where
    /// `inp` is a **signed** `__VERIFIER_nondet_int()` and `op`/`k` form an
    /// arithmetic expression. This isolates [`def_result_nonneg`] (via
    /// [`arg_is_nonneg`]): the signed source alone leaves the position unbounded (⇒
    /// abstain), so a `true` here comes *solely* from the non-negative-result
    /// reasoning about the entry expression.
    fn self_rec_called_expr(
        cmp: BinaryOp,
        bound: i64,
        step: i64,
        op: BinaryOp,
        k: i64,
    ) -> AirModule {
        let mut m = self_rec(cmp, bound, step);
        let i32t = tid("i32");
        let f_id = FunctionId(make_id("func", b"f"));

        let inp = vid("main_inp");
        let kc = vid("main_k");
        let arg = vid("main_arg");
        m.constants.insert(kc, Constant::Int { value: k, bits: 32 });

        let nondet_id = FunctionId(make_id("func", b"__VERIFIER_nondet_int"));
        m.functions.push(AirFunction {
            id: nondet_id,
            name: "__VERIFIER_nondet_int".to_string(),
            params: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });

        let mut main_entry = AirBlock::new(bid("main_entry"));
        main_entry.instructions.push(vinst(
            "main_call_nondet",
            Operation::CallDirect { callee: nondet_id },
            inp,
            vec![],
            i32t,
        ));
        main_entry.instructions.push(vinst(
            "main_arg_expr",
            Operation::BinaryOp { kind: op },
            arg,
            vec![inp, kc],
            i32t,
        ));
        main_entry.instructions.push(term(
            "main_call_f",
            Operation::CallDirect { callee: f_id },
            vec![arg],
        ));
        main_entry
            .instructions
            .push(term("main_ret", Operation::Ret, vec![]));

        m.functions.push(AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![main_entry],
            entry_block: Some(bid("main_entry")),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        m
    }

    #[test]
    fn urem_entry_ne_zero_recursion_is_ranked() {
        // f(n){ if (n != 0) f(n-1); } entered as `f(nondet_int() % 256)`. The nondet
        // is SIGNED, so only `def_result_nonneg` (an unsigned remainder is never
        // negative) proves the base non-negative and recovers the `!=`-split's
        // dropped lower bound ⇒ `f = n` ranks.
        assert!(rec_ranked(&self_rec_called_expr(
            BinaryOp::ICmpNe,
            0,
            -1,
            BinaryOp::URem,
            256,
        )));
    }

    #[test]
    fn and_mask_entry_ne_zero_recursion_is_ranked() {
        // Entered as `f(nondet_int() & 0xFF)`: the mask's sign bit is clear, so the
        // result is in `[0, 255]` ⇒ non-negative base ⇒ ranks.
        assert!(rec_ranked(&self_rec_called_expr(
            BinaryOp::ICmpNe,
            0,
            -1,
            BinaryOp::And,
            0xFF,
        )));
    }

    #[test]
    fn lshr_entry_ne_zero_recursion_is_ranked() {
        // Entered as `f(nondet_int() >> 1)` (logical): the top bit becomes `0` ⇒
        // non-negative base ⇒ ranks.
        assert!(rec_ranked(&self_rec_called_expr(
            BinaryOp::ICmpNe,
            0,
            -1,
            BinaryOp::LShr,
            1,
        )));
    }

    #[test]
    fn srem_entry_ne_zero_recursion_not_ranked() {
        // MANDATORY negative: a SIGNED remainder `nondet_int() % 256` (`srem`) can be
        // NEGATIVE (e.g. `-7 % 256 = -7`), so the base is not provably non-negative ⇒
        // the `!=`-split's negative half stays feasible ⇒ abstain (no wrong `true`).
        assert!(!rec_ranked(&self_rec_called_expr(
            BinaryOp::ICmpNe,
            0,
            -1,
            BinaryOp::SRem,
            256,
        )));
    }

    #[test]
    fn and_negative_mask_entry_ne_zero_recursion_not_ranked() {
        // MANDATORY negative: `nondet_int() & -1` (all-ones mask, sign bit set) is
        // just the signed nondet ⇒ may be negative ⇒ abstain.
        assert!(!rec_ranked(&self_rec_called_expr(
            BinaryOp::ICmpNe,
            0,
            -1,
            BinaryOp::And,
            -1,
        )));
    }

    #[test]
    fn def_result_nonneg_classifies_ops() {
        let i32t = tid("i32");
        let mut m = self_rec(BinaryOp::ICmpNe, 0, -1); // reuse for its i32 type table
        m.types.insert(i32t, AirType::Integer { bits: 32 });
        let x = vid("dr_x");
        let k0 = vid("dr_k0");
        let k1 = vid("dr_k1");
        let mask = vid("dr_mask");
        let negmask = vid("dr_negmask");
        m.constants.insert(k0, Constant::Int { value: 0, bits: 32 });
        m.constants.insert(k1, Constant::Int { value: 1, bits: 32 });
        m.constants.insert(
            mask,
            Constant::Int {
                value: 0xFF,
                bits: 32,
            },
        );
        m.constants.insert(
            negmask,
            Constant::Int {
                value: -1,
                bits: 32,
            },
        );
        let mk = |op: BinaryOp, ops: Vec<ValueId>| {
            vinst(
                "dr_d",
                Operation::BinaryOp { kind: op },
                vid("dr_d"),
                ops,
                i32t,
            )
        };
        // Non-negative results.
        assert!(def_result_nonneg(&mk(BinaryOp::URem, vec![x, k0]), &m));
        assert!(def_result_nonneg(&mk(BinaryOp::And, vec![x, mask]), &m));
        assert!(def_result_nonneg(&mk(BinaryOp::And, vec![mask, x]), &m)); // commutative
        assert!(def_result_nonneg(&mk(BinaryOp::LShr, vec![x, k1]), &m));
        // Possibly-negative results (fail closed).
        assert!(!def_result_nonneg(&mk(BinaryOp::SRem, vec![x, k0]), &m));
        assert!(!def_result_nonneg(&mk(BinaryOp::AShr, vec![x, k1]), &m));
        assert!(!def_result_nonneg(&mk(BinaryOp::And, vec![x, negmask]), &m));
        assert!(!def_result_nonneg(&mk(BinaryOp::LShr, vec![x, k0]), &m)); // shift by 0
        assert!(!def_result_nonneg(&mk(BinaryOp::Add, vec![x, k1]), &m));
    }

    #[test]
    fn unsigned_nondet_entry_ne_zero_recursion_is_ranked() {
        // `unsigned f(x){ if (x != 0) f(x - 1); }` called as `f(nondet_uint())`.
        // The parameter carries no local signedness hint (only `!=` + `add`), so
        // `infer_signs` leaves it `Unknown`; the unsigned nondet source at the
        // external call site proves it non-negative ⇒ `[0, 2^32−1]` ⇒ `f = x`
        // ranks (the mem2reg/`-O0` `id`-family shape).
        assert!(rec_ranked(&self_rec_called(
            BinaryOp::ICmpNe,
            0,
            -1,
            "__VERIFIER_nondet_uint",
            0,
        )));
    }

    #[test]
    fn nonneg_const_entry_ne_zero_recursion_is_ranked() {
        // Same shape, entered with a non-negative constant `f(7)` — also a sound
        // witness that the parameter is non-negative at the base.
        assert!(rec_ranked(&self_rec_called(BinaryOp::ICmpNe, 0, -1, "", 7)));
    }

    #[test]
    fn signed_nondet_entry_ne_zero_recursion_not_ranked() {
        // MANDATORY negative: entered from a SIGNED nondet source, the parameter
        // may be negative, so `f(x-1)` can decrease past the type minimum — must
        // NOT be treated as unsigned, must abstain (no wrong `true`).
        assert!(!rec_ranked(&self_rec_called(
            BinaryOp::ICmpNe,
            0,
            -1,
            "__VERIFIER_nondet_int",
            0,
        )));
    }

    #[test]
    fn negative_const_entry_ne_zero_recursion_not_ranked() {
        // Entered with a NEGATIVE constant `f(-3)`: not provably non-negative ⇒
        // the position is not treated as unsigned ⇒ abstain.
        assert!(!rec_ranked(&self_rec_called(
            BinaryOp::ICmpNe,
            0,
            -1,
            "",
            -3
        )));
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
        // f(int n){ if (n != 0) f(n - 1); }  with a *signed* `sub nsw` decrement:
        // decrementing past INT_MIN is undefined behavior, so `n` may not be read as
        // an unsigned bit pattern ⇒ it keeps no bound, the `!=` split's negative half
        // cannot be pruned ⇒ abstain (sound: a negative signed `n` is UB / diverges).
        assert!(!rec_ranked(&self_rec(BinaryOp::ICmpNe, 0, -1)));
    }

    #[test]
    fn ne_guard_plain_add_recursion_is_ranked() {
        // f(int x){ if (x != 0) f((unsigned)x - 1); }  — a *defined-wraparound*
        // (non-`nsw`) decrement guarded only by `!=`. The Eq/Ne-only, `nsw`-free `x`
        // earns the `[0, 2^w−1]` bit-pattern bound, so the `!=` split gives `x ≥ 1`
        // (where `x − 1` never underflows ⇒ f = x ranks) and prunes `x ≤ −1` as
        // infeasible. Terminates for every input ⇒ TRUE. (Was abstained: no bound.)
        assert!(rec_ranked(&self_rec_plain_add(BinaryOp::ICmpNe, 0, -1)));
    }

    #[test]
    fn ne_guard_plain_add_increasing_recursion_not_ranked() {
        // f(int x){ if (x != 5) f((unsigned)x + 2); }  — even a defined-wraparound
        // counter that *increases* away from the `!=` target does not rank: the
        // split's `x > 5` half is unbounded above (and `x + 2` overflows the bound),
        // so it cannot be ranked, and both halves must ⇒ abstain (the negative test).
        assert!(!rec_ranked(&self_rec_plain_add(BinaryOp::ICmpNe, 5, 2)));
    }

    /// `unsigned f(n){ t = n / 1; if (n != 0) f(n - 1); }` — the `udiv` hints `n`
    /// unsigned so it carries the range `[0, 2^32-1]`. Exercises the disjunctive
    /// `!=` split on the recursion model.
    fn self_rec_unsigned_ne_zero() -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let zero = vid("zero");
        let one = vid("one");
        let negone = vid("negone");
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });
        constants.insert(
            negone,
            Constant::Int {
                value: -1,
                bits: 32,
            },
        );

        let f_id = FunctionId(make_id("func", b"f"));
        let n = vid("n");
        let dead = vid("dead");
        let c = vid("c");
        let na = vid("na");
        let entry = bid("f_entry");
        let rec = bid("f_rec");
        let base = bid("f_base");

        let mut eb = AirBlock::new(entry);
        // Unsigned hint: `n u/ 1` marks `n` unsigned for `infer_signs`.
        eb.instructions.push(vinst(
            "hint",
            Operation::BinaryOp {
                kind: BinaryOp::UDiv,
            },
            dead,
            vec![n, one],
            i32t,
        ));
        eb.instructions.push(vinst(
            "cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            c,
            vec![n, zero],
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
            vec![n, negone],
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

    #[test]
    fn ne_zero_unsigned_recursion_is_ranked() {
        // f(unsigned n){ if (n != 0) f(n - 1); }  — the `!=` split gives `n ≥ 1`
        // (where `n - 1` never underflows ⇒ f = n ranks) and `n ≤ -1` (infeasible
        // under `n ≥ 0` ⇒ vacuous). Both rank ⇒ TRUE (was abstained: on `n ≥ 0` the
        // `n - 1` step underflows at n = 0 ⇒ havoc ⇒ unranked).
        assert!(rec_ranked(&self_rec_unsigned_ne_zero()));
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

    // --- mutual recursion ranking -------------------------------------------

    /// Build one member `void <name>(int n) { if (n CMP bound) <callee>(n+step); }`
    /// of a mutually-recursive pair. `fid`/`callee_id` identify this member and the
    /// function it recurses into; `n` is this member's parameter symbol.
    fn mutual_member(
        name: &str,
        fid: FunctionId,
        callee_id: FunctionId,
        n: ValueId,
        cmp: BinaryOp,
        bound_v: ValueId,
        step_v: ValueId,
        i32t: TypeId,
        i1t: TypeId,
    ) -> AirFunction {
        let c = vid(&format!("{name}_c"));
        let na = vid(&format!("{name}_na"));
        let entry = bid(&format!("{name}_entry"));
        let rec = bid(&format!("{name}_rec"));
        let base = bid(&format!("{name}_base"));

        let mut eb = AirBlock::new(entry);
        eb.instructions.push(vinst(
            &format!("{name}_cmp"),
            Operation::BinaryOp { kind: cmp },
            c,
            vec![n, bound_v],
            i1t,
        ));
        eb.instructions.push(term(
            &format!("{name}_condbr"),
            Operation::CondBr {
                then_target: rec,
                else_target: base,
            },
            vec![c],
        ));

        let mut rb = AirBlock::new(rec);
        rb.instructions.push(vinst(
            &format!("{name}_add"),
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            na,
            vec![n, step_v],
            i32t,
        ));
        rb.instructions.push(term(
            &format!("{name}_call"),
            Operation::CallDirect { callee: callee_id },
            vec![na],
        ));
        rb.instructions.push(term(
            &format!("{name}_br"),
            Operation::Br { target: base },
            vec![],
        ));

        let mut bb = AirBlock::new(base);
        bb.instructions
            .push(term(&format!("{name}_ret"), Operation::Ret, vec![]));

        AirFunction {
            id: fid,
            name: name.to_string(),
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
        }
    }

    /// Build a module with a mutually-recursive pair `f`/`g`, each
    /// `void h(int n){ if (n CMP bound) other(n+step); }`, plus the shared
    /// `{f, g}` SCC set. Both members share one signature so positions unify.
    fn mutual_pair(cmp: BinaryOp, bound: i64, step: i64) -> (AirModule, BTreeSet<FunctionId>) {
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
        let g_id = FunctionId(make_id("func", b"g"));
        let f = mutual_member("f", f_id, g_id, vid("f_n"), cmp, bound_v, step_v, i32t, i1t);
        let g = mutual_member("g", g_id, f_id, vid("g_n"), cmp, bound_v, step_v, i32t, i1t);

        let mut m = module_of(f, constants);
        m.functions.push(g);
        let scc: BTreeSet<FunctionId> = [f_id, g_id].into_iter().collect();
        (m, scc)
    }

    #[test]
    fn mutual_count_down_is_ranked() {
        // f(n){ if (n>0) g(n-1); }  g(n){ if (n>0) f(n-1); }  — |n| decreases on
        // every cross-call under the shared guard n>0 ⇒ the SCC is ranked by f=n.
        let (m, scc) = mutual_pair(BinaryOp::ICmpSgt, 0, -1);
        assert!(mutual_recursion_is_ranked(&m, &scc));
    }

    #[test]
    fn mutual_count_down_to_const_is_ranked() {
        // f(n){ if (n>3) g(n-1); }  g(n){ if (n>3) f(n-1); }  → f = n - 3.
        let (m, scc) = mutual_pair(BinaryOp::ICmpSgt, 3, -1);
        assert!(mutual_recursion_is_ranked(&m, &scc));
    }

    #[test]
    fn mutual_count_up_not_ranked() {
        // f(n){ if (n>0) g(n+1); }  g(n){ if (n>0) f(n+1); }  — recurses forever for
        // n > 0; no ranking function ⇒ abstain (never a wrong `true`).
        let (m, scc) = mutual_pair(BinaryOp::ICmpSgt, 0, 1);
        assert!(!mutual_recursion_is_ranked(&m, &scc));
    }

    #[test]
    fn mutual_ne_guard_not_ranked() {
        // A `!=` guard is not convex; with only a type bound the argument can walk
        // past INT_MIN ⇒ abstain (this diverges for a negative signed `n`).
        let (m, scc) = mutual_pair(BinaryOp::ICmpNe, 0, -1);
        assert!(!mutual_recursion_is_ranked(&m, &scc));
    }

    #[test]
    fn mutual_singleton_scc_abstains() {
        // A size-1 SCC is not mutual recursion — the dedicated `recursion_is_ranked`
        // path handles it; the mutual builder abstains.
        let (m, _) = mutual_pair(BinaryOp::ICmpSgt, 0, -1);
        let solo: BTreeSet<FunctionId> = [m.functions[0].id].into_iter().collect();
        assert!(!mutual_recursion_is_ranked(&m, &solo));
    }

    // --- disjunctive `!=`-split: infeasible-branch skip + guarded non-negativity

    #[test]
    fn region_is_infeasible_detects_empty_and_keeps_nonempty() {
        let x = vid("ri_x");
        let universe: BTreeSet<ValueId> = [x].into_iter().collect();
        // `x >= 1` ∧ `x <= 0` ⇒ empty.
        let empty = vec![
            Constraint {
                coeffs: BTreeMap::from([(x, 1)]),
                constant: -1,
            },
            Constraint {
                coeffs: BTreeMap::from([(x, -1)]),
                constant: 0,
            },
        ];
        assert!(region_is_infeasible(&empty, &universe));
        // `x >= 1` ∧ `x <= 5` ⇒ satisfiable ⇒ NOT skipped.
        let nonempty = vec![
            Constraint {
                coeffs: BTreeMap::from([(x, 1)]),
                constant: -1,
            },
            Constraint {
                coeffs: BTreeMap::from([(x, -1)]),
                constant: 5,
            },
        ];
        assert!(!region_is_infeasible(&nonempty, &universe));
    }

    /// Build a caller `void caller() { int x = nondet_int(); [if (x < 0) return;]
    /// callee(x); }`. When `guarded`, an early `x < 0` return dominates the call so
    /// `x >= 0` holds at it; otherwise the call is unguarded.
    fn guard_caller(guarded: bool) -> (AirModule, ValueId, BlockId) {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let zero = vid("gc_zero");
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });

        let callee_id = FunctionId(make_id("func", b"callee"));
        let x = vid("gc_x");
        let cond = vid("gc_cond");
        let entry = bid("gc_entry");
        let ret_blk = bid("gc_ret");
        let call_blk = bid("gc_call");

        let mut eb = AirBlock::new(entry);
        eb.instructions.push(vinst(
            "gc_nd",
            Operation::CallDirect {
                callee: FunctionId(make_id("func", b"__VERIFIER_nondet_int")),
            },
            x,
            vec![],
            i32t,
        ));
        let call_block_id = if guarded {
            eb.instructions.push(vinst(
                "gc_cmp",
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpSlt,
                },
                cond,
                vec![x, zero],
                i1t,
            ));
            eb.instructions.push(term(
                "gc_condbr",
                Operation::CondBr {
                    then_target: ret_blk,
                    else_target: call_blk,
                },
                vec![cond],
            ));
            call_blk
        } else {
            eb.instructions
                .push(term("gc_br", Operation::Br { target: call_blk }, vec![]));
            call_blk
        };

        let mut cb = AirBlock::new(call_blk);
        cb.instructions.push(term(
            "gc_call",
            Operation::CallDirect { callee: callee_id },
            vec![x],
        ));
        cb.instructions
            .push(term("gc_call_ret", Operation::Ret, vec![]));

        let mut blocks = vec![eb, cb];
        if guarded {
            let mut rb = AirBlock::new(ret_blk);
            rb.instructions
                .push(term("gc_ret_i", Operation::Ret, vec![]));
            blocks.insert(1, rb);
        }

        let caller = AirFunction {
            id: FunctionId(make_id("func", b"caller")),
            name: "caller".to_string(),
            params: Vec::new(),
            blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        (module_of(caller, constants), x, call_block_id)
    }

    #[test]
    fn arg_is_guarded_nonneg_recognizes_early_return_guard() {
        let (m, x, call_blk) = guard_caller(true);
        let caller = &m.functions[0];
        assert!(arg_is_guarded_nonneg(x, caller, call_blk, &m));
    }

    #[test]
    fn arg_is_guarded_nonneg_rejects_unguarded_call() {
        let (m, x, call_blk) = guard_caller(false);
        let caller = &m.functions[0];
        assert!(!arg_is_guarded_nonneg(x, caller, call_blk, &m));
    }

    /// Build an `EvenOdd`-shaped SCC: two mutually-recursive members, each
    /// `int h(int n){ if (n==0) return 0; if (n==1) return 1; return other(n-1); }`
    /// (two `==` guards ⇒ the `!=`-split forks each recursive path into four
    /// sub-branches, most infeasible), plus a `main` that reaches the SCC via
    /// `int x = nondet_int(); [if (x < 0) return 0;] f(x);`. Returns the module and
    /// the `{f, g}` SCC.
    fn evenodd_scc(guarded: bool) -> (AirModule, BTreeSet<FunctionId>) {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();
        let zero = vid("eo_zero");
        let one = vid("eo_one");
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });
        constants.insert(one, Constant::Int { value: 1, bits: 32 });

        let f_id = FunctionId(make_id("func", b"eo_f"));
        let g_id = FunctionId(make_id("func", b"eo_g"));

        let member = |name: &str, fid: FunctionId, callee: FunctionId, n: ValueId| -> AirFunction {
            let c0 = vid(&format!("{name}_c0"));
            let c1 = vid(&format!("{name}_c1"));
            let na = vid(&format!("{name}_na"));
            let entry = bid(&format!("{name}_entry"));
            let t1 = bid(&format!("{name}_t1"));
            let rec = bid(&format!("{name}_rec"));
            let ret0 = bid(&format!("{name}_ret0"));
            let ret1 = bid(&format!("{name}_ret1"));

            let mut eb = AirBlock::new(entry);
            eb.instructions.push(vinst(
                &format!("{name}_cmp0"),
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpEq,
                },
                c0,
                vec![n, zero],
                i1t,
            ));
            eb.instructions.push(term(
                &format!("{name}_br0"),
                Operation::CondBr {
                    then_target: ret0,
                    else_target: t1,
                },
                vec![c0],
            ));

            let mut t1b = AirBlock::new(t1);
            t1b.instructions.push(vinst(
                &format!("{name}_cmp1"),
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpEq,
                },
                c1,
                vec![n, one],
                i1t,
            ));
            t1b.instructions.push(term(
                &format!("{name}_br1"),
                Operation::CondBr {
                    then_target: ret1,
                    else_target: rec,
                },
                vec![c1],
            ));

            let mut rb = AirBlock::new(rec);
            rb.instructions.push(vinst(
                &format!("{name}_sub"),
                Operation::BinaryOp {
                    kind: BinaryOp::Sub,
                },
                na,
                vec![n, one],
                i32t,
            ));
            rb.instructions.push(term(
                &format!("{name}_call"),
                Operation::CallDirect { callee },
                vec![na],
            ));
            rb.instructions
                .push(term(&format!("{name}_rret"), Operation::Ret, vec![]));

            let mut r0 = AirBlock::new(ret0);
            r0.instructions
                .push(term(&format!("{name}_r0"), Operation::Ret, vec![]));
            let mut r1 = AirBlock::new(ret1);
            r1.instructions
                .push(term(&format!("{name}_r1"), Operation::Ret, vec![]));

            AirFunction {
                id: fid,
                name: name.to_string(),
                params: vec![AirParam {
                    id: n,
                    name: None,
                    index: 0,
                    param_type: Some(i32t),
                }],
                blocks: vec![eb, t1b, rb, r0, r1],
                entry_block: Some(entry),
                is_declaration: false,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            }
        };

        let f = member("eo_f", f_id, g_id, vid("eo_f_n"));
        let g = member("eo_g", g_id, f_id, vid("eo_g_n"));

        // main: x = nondet_int(); [if (x < 0) return;] f(x);
        let x = vid("eo_main_x");
        let cond = vid("eo_main_cond");
        let entry = bid("eo_main_entry");
        let ret_blk = bid("eo_main_ret");
        let call_blk = bid("eo_main_call");
        let mut eb = AirBlock::new(entry);
        eb.instructions.push(vinst(
            "eo_main_nd",
            Operation::CallDirect {
                callee: FunctionId(make_id("func", b"__VERIFIER_nondet_int")),
            },
            x,
            vec![],
            i32t,
        ));
        let mut main_blocks;
        if guarded {
            eb.instructions.push(vinst(
                "eo_main_cmp",
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpSlt,
                },
                cond,
                vec![x, zero],
                i1t,
            ));
            eb.instructions.push(term(
                "eo_main_br",
                Operation::CondBr {
                    then_target: ret_blk,
                    else_target: call_blk,
                },
                vec![cond],
            ));
            let mut rb = AirBlock::new(ret_blk);
            rb.instructions
                .push(term("eo_main_ret_i", Operation::Ret, vec![]));
            main_blocks = vec![eb, rb];
        } else {
            eb.instructions.push(term(
                "eo_main_br",
                Operation::Br { target: call_blk },
                vec![],
            ));
            main_blocks = vec![eb];
        }
        let mut cb = AirBlock::new(call_blk);
        cb.instructions.push(term(
            "eo_main_call",
            Operation::CallDirect { callee: f_id },
            vec![x],
        ));
        cb.instructions
            .push(term("eo_main_cret", Operation::Ret, vec![]));
        main_blocks.push(cb);

        let main = AirFunction {
            id: FunctionId(make_id("func", b"eo_main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: main_blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut m = module_of(f, constants);
        m.functions.push(g);
        m.functions.push(main);
        let scc: BTreeSet<FunctionId> = [f_id, g_id].into_iter().collect();
        (m, scc)
    }

    #[test]
    fn evenodd_two_eq_guards_ranked_under_nonneg_entry() {
        // With `main`'s `if (x < 0) return;` guard, the SCC parameter is provably
        // `>= 0` at entry, so the two `==`-else `!=`-splits' negative halves are
        // infeasible (skipped) and the surviving `n >= 2` transition `n -> n - 1`
        // ranks by `f = n`. Exercises BOTH the guarded-non-negativity precondition
        // and the infeasible-branch skip (8 raw sub-branches ⇒ 2 feasible).
        let (m, scc) = evenodd_scc(true);
        assert!(mutual_recursion_is_ranked(&m, &scc));
    }

    #[test]
    fn evenodd_two_eq_guards_abstains_without_guard() {
        // Without the entry guard the parameter is a plain signed `int`: the
        // `!=`-split's `n <= -1` half is feasible and `n - 1` diverges toward
        // `INT_MIN` ⇒ that half cannot be ranked ⇒ abstain (sound: `isOdd(-1)`
        // recurses forever). Negative control for the guarded case above.
        let (m, scc) = evenodd_scc(false);
        assert!(!mutual_recursion_is_ranked(&m, &scc));
    }

    // --- multi-latch loops (two back-edges into one header) -----------------

    /// Build `main` containing a single loop whose header has **two** back-edges
    /// (the `continue` shape):
    /// ```c
    /// int i = nondet();
    /// while (i > 0) {          // header: phi i ; icmp sgt(i,0) ; condbr body/exit
    ///   if (nondet()) i += step1;   // body → l1 (back-edge)
    ///   else          i += step2;   // body → l2 (back-edge)
    /// }
    /// ```
    /// Both `l1 → header` and `l2 → header` are dominance back-edges, so the header
    /// has two latches (the single-latch extractor rejects it). A common ranking
    /// function `f = i` proves termination iff *both* steps decrease `i`.
    fn multilatch_loop(step1: i64, step2: i64) -> AirModule {
        let i32t = tid("i32");
        let i1t = tid("i1");
        let mut constants = BTreeMap::new();

        let b0 = bid("ml_entry");
        let h = bid("ml_header");
        let body = bid("ml_body");
        let l1 = bid("ml_l1");
        let l2 = bid("ml_l2");
        let e = bid("ml_exit");

        let i = vid("ml_i"); // header phi
        let i1v = vid("ml_i1"); // l1: i + step1
        let i2v = vid("ml_i2"); // l2: i + step2
        let i_init = vid("ml_i_init"); // nondet init
        let cond = vid("ml_cond"); // header guard result
        let brc = vid("ml_brc"); // body branch nondet
        let zero = vid("ml_zero");
        let s1 = vid("ml_step1");
        let s2 = vid("ml_step2");
        constants.insert(zero, Constant::Int { value: 0, bits: 32 });
        constants.insert(
            s1,
            Constant::Int {
                value: step1,
                bits: 32,
            },
        );
        constants.insert(
            s2,
            Constant::Int {
                value: step2,
                bits: 32,
            },
        );

        let nd_int = FunctionId(make_id("func", b"__VERIFIER_nondet_int"));

        // entry: i_init = nondet(); br header
        let mut entry = AirBlock::new(b0);
        entry.instructions.push(vinst(
            "ml_init",
            Operation::CallDirect { callee: nd_int },
            i_init,
            vec![],
            i32t,
        ));
        entry
            .instructions
            .push(term("ml_br_entry", Operation::Br { target: h }, vec![]));

        // header: phi i ; cond = icmp sgt(i,0) ; condbr cond -> body/exit
        let mut header = AirBlock::new(h);
        header.instructions.push(vinst(
            "ml_phi",
            Operation::Phi {
                incoming: vec![(b0, i_init), (l1, i1v), (l2, i2v)],
            },
            i,
            vec![],
            i32t,
        ));
        header.instructions.push(vinst(
            "ml_cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            cond,
            vec![i, zero],
            i1t,
        ));
        header.instructions.push(term(
            "ml_condbr",
            Operation::CondBr {
                then_target: body,
                else_target: e,
            },
            vec![cond],
        ));

        // body: brc = nondet(); condbr brc -> l1/l2
        let mut bodyb = AirBlock::new(body);
        bodyb.instructions.push(vinst(
            "ml_brc",
            Operation::CallDirect { callee: nd_int },
            brc,
            vec![],
            i32t,
        ));
        bodyb.instructions.push(term(
            "ml_bodybr",
            Operation::CondBr {
                then_target: l1,
                else_target: l2,
            },
            vec![brc],
        ));

        // l1: i1 = i + step1 ; br header  (back-edge)
        let mut lb1 = AirBlock::new(l1);
        lb1.instructions.push(vinst(
            "ml_add1",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            i1v,
            vec![i, s1],
            i32t,
        ));
        lb1.instructions
            .push(term("ml_br1", Operation::Br { target: h }, vec![]));

        // l2: i2 = i + step2 ; br header  (back-edge)
        let mut lb2 = AirBlock::new(l2);
        lb2.instructions.push(vinst(
            "ml_add2",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            i2v,
            vec![i, s2],
            i32t,
        ));
        lb2.instructions
            .push(term("ml_br2", Operation::Br { target: h }, vec![]));

        // exit: ret i
        let mut exit = AirBlock::new(e);
        exit.instructions
            .push(term("ml_ret", Operation::Ret, vec![i]));

        let func = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![entry, header, bodyb, lb1, lb2, exit],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        module_of(func, constants)
    }

    #[test]
    fn multilatch_both_decrease_is_ranked() {
        // while (i>0) { if (*) i-=1; else i-=2; }  — both back-edges decrease i,
        // so f = i ranks the whole loop (single common ranking over two latches).
        assert!(ranked(&multilatch_loop(-1, -2)));
    }

    #[test]
    fn multilatch_one_increases_abstains() {
        // while (i>0) { if (*) i-=1; else i+=1; }  — the `i+=1` latch can be taken
        // forever, so no common ranking exists ⇒ abstain (never a wrong `true`).
        assert!(!ranked(&multilatch_loop(-1, 1)));
    }

    // --- Disjunctive SCC ranking (branch can-follow decomposition) -----------

    /// `expr ≥ 0` constraint from a symbol coefficient list + constant.
    fn cons(terms: &[(ValueId, i128)], k: i128) -> Constraint {
        Constraint {
            coeffs: terms.iter().copied().collect(),
            constant: k,
        }
    }
    /// Affine form `Σ cᵢ·sᵢ + k`.
    fn aff(terms: &[(ValueId, i128)], k: i128) -> Affine {
        Affine {
            terms: terms.iter().copied().collect(),
            constant: k,
        }
    }

    #[test]
    fn disjunctive_two_directional_counter_is_ranked() {
        // Two mutually-exclusive convex branches whose union is a `!=` guard:
        //   A: x ≥ 1 ⇒ x := x-1     B: x ≤ -1 ⇒ x := x+1
        // (the shape of `addition(m,n)`: `n>0 ⇒ n-1`, `n<0 ⇒ n+1`). No single
        // linear `f` decreases on both, so the joint greedy tuple fails; but the
        // cross edges are infeasible (A stays ≥ 0, B stays ≤ 0), so each singleton
        // SCC ranks on its own (`f = x` / `f = -x`) ⇒ disjunctively terminating.
        let x = vid("x");
        let a = Branch {
            region: vec![cons(&[(x, 1)], -1)],               // x - 1 ≥ 0
            next: BTreeMap::from([(x, aff(&[(x, 1)], -1))]), // x - 1
        };
        let b = Branch {
            region: vec![cons(&[(x, -1)], -1)],             // -x - 1 ≥ 0
            next: BTreeMap::from([(x, aff(&[(x, 1)], 1))]), // x + 1
        };
        let model = MultiPathModel {
            template: BTreeSet::from([x]),
            universe: BTreeSet::from([x]),
            branches: vec![a, b],
        };
        // The joint tuple genuinely fails, and the disjunctive fallback succeeds.
        assert!(!greedy_lex_subset(&model, &[0, 1]));
        assert!(greedy_lex_rank(&model));
    }

    #[test]
    fn disjunctive_pingpong_havoc_abstains() {
        // Ping-pong: A decreases x but RESETS y to an arbitrary in-range value; B
        // decreases y but RESETS x. Each branch is individually well-founded, yet
        // the loop can diverge by alternating (the classic transition-invariant
        // counterexample). The cross edges are feasible (the reset value can
        // re-enable the other guard), so A and B share one SCC and must rank
        // jointly — which they cannot ⇒ abstain. MANDATORY soundness test.
        let x = vid("x");
        let y = vid("y");
        let ha = vid("ha"); // fresh reset value for y in branch A
        let hb = vid("hb"); // fresh reset value for x in branch B
        let bound = |s: ValueId| vec![cons(&[(s, 1)], 0), cons(&[(s, -1)], 1_000_000)];
        let mut a_region = vec![cons(&[(x, 1)], -1)]; // x ≥ 1
        a_region.extend(bound(ha));
        let mut b_region = vec![cons(&[(y, 1)], -1)]; // y ≥ 1
        b_region.extend(bound(hb));
        let a = Branch {
            region: a_region,
            next: BTreeMap::from([(x, aff(&[(x, 1)], -1)), (y, aff(&[(ha, 1)], 0))]),
        };
        let b = Branch {
            region: b_region,
            next: BTreeMap::from([(y, aff(&[(y, 1)], -1)), (x, aff(&[(hb, 1)], 0))]),
        };
        let model = MultiPathModel {
            template: BTreeSet::from([x, y]),
            universe: BTreeSet::from([x, y, ha, hb]),
            branches: vec![a, b],
        };
        assert!(branch_can_follow(&model, 0, 1)); // A → B is feasible (reset).
        assert!(branch_can_follow(&model, 1, 0)); // B → A is feasible (reset).
        assert!(!greedy_lex_rank(&model)); // one SCC, not jointly rankable ⇒ abstain.
    }

    #[test]
    fn disjunctive_diverging_half_abstains() {
        // `while (x != 5) x += 2` (even start): the split gives A: x ≥ 6 ⇒ x+2 and
        // B: x ≤ 4 ⇒ x+2. Branch A self-loops (x stays ≥ 6) but has no bounded
        // decreasing rank (x grows unboundedly) ⇒ its SCC cannot rank ⇒ abstain.
        let x = vid("x");
        let a = Branch {
            region: vec![cons(&[(x, 1)], -6)],              // x - 6 ≥ 0
            next: BTreeMap::from([(x, aff(&[(x, 1)], 2))]), // x + 2
        };
        let b = Branch {
            region: vec![cons(&[(x, -1)], 4)], // -x + 4 ≥ 0  (x ≤ 4)
            next: BTreeMap::from([(x, aff(&[(x, 1)], 2))]), // x + 2
        };
        let model = MultiPathModel {
            template: BTreeSet::from([x]),
            universe: BTreeSet::from([x]),
            branches: vec![a, b],
        };
        assert!(branch_can_follow(&model, 0, 0)); // A → A (diverging self-loop).
        assert!(!branch_can_follow(&model, 0, 1)); // A (x≥6→x+2) can't reach B (x≤4).
        assert!(!greedy_lex_rank(&model));
    }
}
