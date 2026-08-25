//! KLEE-style bounded forward symbolic interpreter for the `unreach-call`
//! property.
//!
//! # What this adds over the fixed-k path BMC engine ([`crate::bmc`])
//!
//! [`crate::bmc`] enumerates a handful of *acyclic* (`k = 1` unwound) CFG paths
//! between a function's entry and a `reach_error` block, encodes each whole path,
//! and solves it. That misses two large classes of `unreach-call` violations:
//!
//! 1. **Loop-carried guards.** The error is reachable only after a loop runs a few
//!    iterations (`for (i=0;i<4;i++) s+=x; if (s==40) reach_error();`). BMC's `k=1`
//!    acyclic unwinding never re-enters the loop body, so the accumulator never
//!    grows and the guard is UNSAT under its model.
//! 2. **Branchy control flow.** When many conditionals precede the error, the
//!    feasible path is one of exponentially many; BMC's fixed path cap enumerates
//!    whole paths blindly and can exhaust the cap before finding a feasible one.
//!
//! This engine instead performs *forward* symbolic execution KLEE-style: it keeps
//! a worklist of symbolic states, and at every conditional branch it asks Z3
//! whether each side is feasible under the accumulated path condition, **forking**
//! into the feasible successors and **pruning** the infeasible ones. Loops are
//! unwound lazily by re-executing blocks up to a per-block visit cap, so a small
//! bounded trip count is explored naturally. When a state reaches `reach_error`
//! its full path condition is solved for a model, yielding the concrete
//! `__VERIFIER_nondet_*` input vector — in execution order, so per-iteration nondet
//! reads inside an unwound loop each get their own value.
//!
//! Two KLEE solver optimizations keep the fork search inside a bounded budget:
//!
//! * **Constraint independence.** A branch feasibility query only needs the path
//!   constraints that share variables (transitively) with the branch guard. We
//!   track, per SSA value, the set of fresh symbolic variables it depends on, and
//!   send Z3 only the connected component (plus variable-free constraints, which
//!   are always relevant). Dropping unrelated constraints is sound *and* complete:
//!   disjoint-variable constraint sets are jointly satisfiable, so a component's
//!   (un)satisfiability decides the whole set's.
//! * **Counterexample / query cache.** Feasibility results are memoized on the set
//!   of constraint identities sent to Z3, so the repeated sub-queries a loop
//!   generates are solved once.
//!
//! # Soundness
//!
//! Like BMC, the engine is UNSOUND on its own (64-bit-uniform bitvector arithmetic,
//! identity casts, havoc for un-modelled memory / calls). It emits only
//! [`FalseCandidate`]s, fed through the SAME native concrete-replay gate as every
//! other stage — a spurious or imprecise model can only ever fail to reproduce →
//! `unknown`. Never a verdict from a solver model alone (confirmer contract R6).
//!
//! # Cost
//!
//! Fires only when a `reach_error`-containing function has a scalar-int nondet
//! input **and a CFG cycle** — precisely the loop-carried class BMC's acyclic base
//! case cannot crack. Acyclic functions are left to the cheaper guard / BMC stages.
//! Step count, fork count, per-block visit count, per-state constraint count, total
//! Z3 calls, and the per-query timeout are all hard-capped, so a gated task adds
//! only a bounded solver budget.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirBlock, AirFunction, AirModule, Constant, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};
use z3::SatResult;
use z3::ast::{BV, Bool};

use crate::property::{
    FalseCandidate, NondetCall, PropertyAnalysisConfig, is_scalar_integer_nondet,
};
use crate::property_kind::DataModel;
use crate::ssa_encode::{self, BV_WIDTH};

/// Max `reach_error`-containing functions symbolically executed per module.
const SE_MAX_FUNCS: usize = 2;

/// Loop-unwind bound: max times one block is (re-)entered on a single state's
/// path. A loop that needs more than this many iterations to reach the error is
/// left `unknown`.
const SE_BLOCK_VISIT_CAP: usize = 8;

/// Max total instructions executed across all states for one function.
const SE_MAX_STEPS: usize = 40_000;

/// Max states dequeued from the worklist for one function.
const SE_MAX_STATES: usize = 6_000;

/// Max Z3 feasibility / model queries for one function.
const SE_MAX_SOLVER_CALLS: usize = 4_000;

/// Max blocks in a single state's trace (bounds witness / clone cost).
const SE_MAX_TRACE: usize = 600;

/// Max path constraints on a single state (kills pathological growth).
const SE_MAX_CON: usize = 4_000;

/// Per-query Z3 timeout (ms). Feasibility checks are small QF_BV solves.
const SE_Z3_TIMEOUT_MS: u32 = 800;

/// Deterministic Z3 rlimit and seed (NFR-DET).
const SE_Z3_RLIMIT: u32 = 3_000_000;
const SE_Z3_SEED: u32 = 0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Enumerate KLEE-style forward-SE FALSE *candidates* for `unreach-call`.
///
/// For each `reach_error` / `__VERIFIER_error`-containing function that has a
/// scalar-int nondet input and a CFG cycle (the loop-carried class BMC misses),
/// forward-symbolically executes from the function entry, forking on Z3-feasible
/// branches and unwinding loops to a bounded visit cap, and on the first state that
/// reaches an error call emits a [`FalseCandidate`] whose `nondet_sequence` is the
/// concrete input vector read from the model. Candidates MUST be confirmed by
/// native replay before any verdict (see module docs).
#[must_use]
pub fn enumerate_se_candidates(
    module: &AirModule,
    config: &PropertyAnalysisConfig,
    data_model: DataModel,
) -> Vec<FalseCandidate> {
    let mut candidates = Vec::new();
    let mut funcs_done: BTreeSet<FunctionId> = BTreeSet::new();

    for func in &module.functions {
        if funcs_done.len() >= SE_MAX_FUNCS {
            break;
        }
        if func.is_declaration || !function_has_error_call(func, module) {
            continue;
        }
        if !gate(func, module) {
            continue;
        }
        let Some(entry) = func
            .entry_block
            .or_else(|| func.blocks.first().map(|b| b.id))
        else {
            continue;
        };
        funcs_done.insert(func.id);

        let mut interp = Interp::new(module, func, data_model);
        if let Some(cand) = interp.run(entry) {
            candidates.push(cand);
        }
    }

    let _ = config; // config timeouts are advisory; SE uses its own hard caps.
    candidates
}

// ---------------------------------------------------------------------------
// Driller-style concolic flip-seed generation (lever `fuzz-concolic-z3`)
// ---------------------------------------------------------------------------

/// Max flip *seeds* returned by one concolic invocation (bounds the extra fuzz
/// execs the caller queues).
const CONC_MAX_FLIPS: usize = 24;

/// Max conditional/switch branch points examined on the concrete path (bounds the
/// Z3 direction + flip solves).
const CONC_MAX_BRANCHES: usize = 512;

/// Per-block visit cap for the concrete path. Set much higher than the forking
/// engine's [`SE_BLOCK_VISIT_CAP`]: concolic follows ONE concrete path (no fork
/// explosion), so it can afford to unwind a constant-bounded loop far deeper — which
/// is precisely its edge, cracking a computed guard *after* a loop whose trip count
/// exceeds the forking engine's cap (`for(i=0;i<20;i++) s+=x; if(s==C) reach_error();`).
/// Still bounded, so an input-independent loop cannot spin forever.
const CONC_BLOCK_VISIT_CAP: usize = 128;

/// Concolically generate new nondet-input seeds that flip the branches a *stuck*
/// fuzz seed did NOT take — the Driller "selective symbolic execution on a
/// coverage plateau" mechanism (Stephens et al., NDSS 2016), implemented fresh.
///
/// `seq` is the concrete scalar-integer `__VERIFIER_nondet_*` value sequence a
/// coverage-plateaued fuzz input produced (in call order). This function
/// *concolically* re-executes that concrete path over the AIR of the
/// `reach_error`-containing function: it feeds each nondet read its concrete value
/// from `seq` (pinning it, so the taken direction of every branch is the one the
/// seed actually took), while symbolically accumulating the path condition over the
/// nondet inputs. At every conditional it Z3-solves the path prefix conjoined with
/// the *negation* of the taken guard (constraint-sliced to the connected component,
/// with an optimistic fallback that drops the prefix when the exact preimage is
/// UNSAT), reading a model back into a fresh nondet-value sequence that drives
/// execution down the un-taken side.
///
/// # Why this is sound
///
/// The returned sequences are only *fuzz seeds*: the caller lays them into the
/// byte-stream input and re-runs the ORIGINAL program, whose native replay remains
/// the sole FALSE arbiter (confirmer contract R6). An imprecise model — from the
/// 64-bit-uniform bitvector encoding, an optimistic (prefix-dropped) solve, or a
/// misaligned concrete value when nondet reads live in a callee — can only ever
/// yield a seed that fails to advance coverage, never a wrong verdict. Nondet reads
/// automatically stay in range (R5): `fresh_nondet` asserts the declared type's
/// bounds, which the flip solve carries along via constraint slicing.
///
/// Deterministic: single concrete path, path-order flip enumeration, fixed Z3 seed.
#[must_use]
pub fn enumerate_concolic_flip_seeds(
    module: &AirModule,
    seq: &[NondetCall],
    data_model: DataModel,
) -> Vec<Vec<NondetCall>> {
    let Some(func) = module.functions.iter().find(|f| {
        !f.is_declaration && function_has_error_call(f, module) && has_scalar_nondet(f, module)
    }) else {
        return Vec::new();
    };
    let Some(entry) = func
        .entry_block
        .or_else(|| func.blocks.first().map(|b| b.id))
    else {
        return Vec::new();
    };
    let mut interp = Interp::new(module, func, data_model);
    interp.concolic_run(entry, seq)
}

/// Does `func` contain a call to a scalar-integer `__VERIFIER_nondet_*`?
fn has_scalar_nondet(func: &AirFunction, module: &AirModule) -> bool {
    func.blocks
        .iter()
        .flat_map(|b| &b.instructions)
        .any(|inst| {
            matches!(&inst.op, Operation::CallDirect { callee }
                if module
                    .function(*callee)
                    .is_some_and(|f| is_scalar_integer_nondet(&f.name)))
        })
}

const ERROR_NAMES: &[&str] = &["reach_error", "__VERIFIER_error"];

/// Does `func` directly call `reach_error` / `__VERIFIER_error`?
fn function_has_error_call(func: &AirFunction, module: &AirModule) -> bool {
    func.blocks
        .iter()
        .flat_map(|b| &b.instructions)
        .any(|inst| {
            matches!(&inst.op, Operation::CallDirect { callee }
            if module
                .function(*callee)
                .is_some_and(|f| ERROR_NAMES.contains(&f.name.as_str())))
        })
}

/// Gate: run SE only when `func` has a scalar-int nondet input **and** a CFG cycle
/// — the loop-carried reachability BMC's acyclic base case cannot handle. Acyclic
/// functions are already covered by the cheaper guard / BMC stages, so skipping
/// them keeps per-task cost bounded.
fn gate(func: &AirFunction, module: &AirModule) -> bool {
    let has_nondet = func
        .blocks
        .iter()
        .flat_map(|b| &b.instructions)
        .any(|inst| {
            matches!(&inst.op, Operation::CallDirect { callee }
                if module
                    .function(*callee)
                    .is_some_and(|f| is_scalar_integer_nondet(&f.name)))
        });
    has_nondet && cfg_has_cycle(func)
}

/// DFS colouring for the cycle detector.
#[derive(Clone, Copy, PartialEq)]
enum Color {
    White,
    Grey,
    Black,
}

/// True iff the CFG of `func` has a cycle (a back edge in DFS from the entry).
fn cfg_has_cycle(func: &AirFunction) -> bool {
    let Some(entry) = func
        .entry_block
        .or_else(|| func.blocks.first().map(|b| b.id))
    else {
        return false;
    };
    let succ: BTreeMap<BlockId, Vec<BlockId>> = func
        .blocks
        .iter()
        .map(|b| (b.id, block_successors(b)))
        .collect();

    // Iterative DFS with a "on current stack" colouring; a grey successor is a
    // back edge ⇒ cycle.
    let mut color: BTreeMap<BlockId, Color> =
        func.blocks.iter().map(|b| (b.id, Color::White)).collect();
    // Stack of (block, next-successor-index).
    let mut stack: Vec<(BlockId, usize)> = vec![(entry, 0)];
    if let Some(c) = color.get_mut(&entry) {
        *c = Color::Grey;
    }
    while let Some(&mut (bid, ref mut idx)) = stack.last_mut() {
        let succs = succ.get(&bid).map_or(&[][..], Vec::as_slice);
        if *idx < succs.len() {
            let next = succs[*idx];
            *idx += 1;
            match color.get(&next).copied().unwrap_or(Color::Black) {
                Color::Grey => return true, // back edge
                Color::White => {
                    if let Some(c) = color.get_mut(&next) {
                        *c = Color::Grey;
                    }
                    stack.push((next, 0));
                }
                Color::Black => {}
            }
        } else {
            if let Some(c) = color.get_mut(&bid) {
                *c = Color::Black;
            }
            stack.pop();
        }
    }
    false
}

/// The successor blocks named by `block`'s terminator.
fn block_successors(block: &AirBlock) -> Vec<BlockId> {
    match block.terminator().map(|t| &t.op) {
        Some(Operation::Br { target }) => vec![*target],
        Some(Operation::CondBr {
            then_target,
            else_target,
        }) => {
            if then_target == else_target {
                vec![*then_target]
            } else {
                vec![*then_target, *else_target]
            }
        }
        Some(Operation::Switch { default, cases }) => {
            let mut out: BTreeSet<BlockId> = BTreeSet::new();
            out.insert(*default);
            for (_, t) in cases {
                out.insert(*t);
            }
            out.into_iter().collect()
        }
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Symbolic state
// ---------------------------------------------------------------------------

/// One path constraint: a Z3 boolean plus the set of fresh symbolic variables it
/// references (for constraint-independence) and a unique id (for the query cache).
#[derive(Clone)]
struct Con {
    id: u32,
    b: Bool,
    vars: BTreeSet<u32>,
}

/// A symbolic execution state (a point in one explored path). Cloned at each fork.
#[derive(Clone)]
struct State {
    /// Current block.
    block: BlockId,
    /// Previously executed block (for phi resolution).
    prev: Option<BlockId>,
    /// SSA value → its bitvector encoding.
    values: BTreeMap<ValueId, BV>,
    /// SSA value → the fresh symbolic variables its encoding depends on.
    vref: BTreeMap<ValueId, BTreeSet<u32>>,
    /// Per-object scalar memory cell: alloca base pointer → current content BV.
    memory: BTreeMap<ValueId, BV>,
    /// Variable dependencies of each memory cell's content.
    mem_vref: BTreeMap<ValueId, BTreeSet<u32>>,
    /// Accumulated path condition.
    constraints: Vec<Con>,
    /// Scalar-int nondet calls executed on this path, in order (BV + name).
    nondet: Vec<(BV, String)>,
    /// Block trace (with loop repeats) for the witness.
    trace: Vec<BlockId>,
    /// Per-block visit count (loop-unwind bound).
    visits: BTreeMap<BlockId, usize>,
}

impl State {
    fn new(entry: BlockId) -> Self {
        Self {
            block: entry,
            prev: None,
            values: BTreeMap::new(),
            vref: BTreeMap::new(),
            memory: BTreeMap::new(),
            mem_vref: BTreeMap::new(),
            constraints: Vec::new(),
            nondet: Vec::new(),
            trace: Vec::new(),
            visits: BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Interpreter
// ---------------------------------------------------------------------------

struct Interp<'a> {
    module: &'a AirModule,
    data_model: DataModel,
    block_by_id: BTreeMap<BlockId, &'a AirBlock>,
    /// Pointer value → root alloca it aliases (through copy/cast/freeze).
    ptr_base: BTreeMap<ValueId, ValueId>,
    /// Fresh symbolic-variable counter (deterministic).
    var_counter: u32,
    /// Constraint id counter (deterministic).
    con_counter: u32,
    steps: usize,
    solver_calls: usize,
    /// Feasibility cache keyed on the sorted constraint-id set of a query.
    cache: BTreeMap<Vec<u32>, bool>,
}

impl<'a> Interp<'a> {
    fn new(module: &'a AirModule, func: &'a AirFunction, data_model: DataModel) -> Self {
        let block_by_id = func.blocks.iter().map(|b| (b.id, b)).collect();
        let ptr_base = compute_ptr_base(func);
        Self {
            module,
            data_model,
            block_by_id,
            ptr_base,
            var_counter: 0,
            con_counter: 0,
            steps: 0,
            solver_calls: 0,
            cache: BTreeMap::new(),
        }
    }

    /// Forward-execute from `entry`, returning the first replay candidate that
    /// reaches an error call (or `None` if none within budget).
    fn run(&mut self, entry: BlockId) -> Option<FalseCandidate> {
        let mut worklist: Vec<State> = vec![State::new(entry)];
        let mut states_done = 0usize;
        while let Some(state) = worklist.pop() {
            if states_done >= SE_MAX_STATES
                || self.steps >= SE_MAX_STEPS
                || self.solver_calls >= SE_MAX_SOLVER_CALLS
            {
                break;
            }
            states_done += 1;
            if let Some(cand) = self.exec_block(state, &mut worklist) {
                return Some(cand);
            }
        }
        None
    }

    /// Execute the current block of `state`: run its non-terminator instructions
    /// (returning a candidate if a `reach_error` call is reached and its path
    /// condition is SAT), then fork on the terminator into feasible successors.
    fn exec_block(
        &mut self,
        mut state: State,
        worklist: &mut Vec<State>,
    ) -> Option<FalseCandidate> {
        let cur = state.block;
        let count = state.visits.entry(cur).or_insert(0);
        *count += 1;
        if *count > SE_BLOCK_VISIT_CAP
            || state.trace.len() >= SE_MAX_TRACE
            || state.constraints.len() >= SE_MAX_CON
        {
            return None;
        }
        state.trace.push(cur);

        let block = *self.block_by_id.get(&cur)?;
        let prev = state.prev;
        for inst in &block.instructions {
            if inst.is_terminator() {
                break;
            }
            self.steps += 1;
            if self.steps >= SE_MAX_STEPS {
                return None;
            }
            if let Operation::CallDirect { callee } = &inst.op {
                if let Some(name) = self.module.function(*callee).map(|f| f.name.as_str()) {
                    if ERROR_NAMES.contains(&name) {
                        return self.try_solve(&state, inst.id);
                    }
                }
            }
            self.define(&mut state, inst, prev);
        }

        let term = block.terminator()?;
        self.step_terminator(state, cur, term, worklist);
        None
    }

    // -- SSA definition --------------------------------------------------

    /// Assign the SSA definition produced by `inst`, or handle its side effect
    /// (`__VERIFIER_assume` as a path filter, `store` into the memory model).
    fn define(&mut self, state: &mut State, inst: &Instruction, prev: Option<BlockId>) {
        if let Operation::Store = &inst.op {
            self.exec_store(state, inst);
            return;
        }
        let Some(dst) = inst.dst else {
            if let Operation::CallDirect { callee } = &inst.op {
                if self.module.function(*callee).map(|f| f.name.as_str())
                    == Some("__VERIFIER_assume")
                {
                    if let Some(&arg) = inst.operands.first() {
                        let con = self.cond_con(state, arg, false);
                        state.constraints.push(con);
                    }
                }
            }
            return;
        };
        let (bv, vars) = self.encode_op(state, inst, prev);
        state.values.insert(dst, bv);
        state.vref.insert(dst, vars);
    }

    /// Encode operand `vid` to a bitvector (memoized in `state.values`), recording
    /// its variable dependency set in `state.vref`.
    fn enc(&mut self, state: &mut State, vid: ValueId) -> BV {
        if let Some(bv) = state.values.get(&vid) {
            return bv.clone();
        }
        if let Some(c) = self.module.constants.get(&vid) {
            if let Some(bv) = const_bv(c) {
                state.values.insert(vid, bv.clone());
                state.vref.insert(vid, BTreeSet::new());
                return bv;
            }
        }
        // Unknown leaf (param / global / undefined) → fresh havoc symbol.
        let (bv, id) = self.fresh("h");
        state.values.insert(vid, bv.clone());
        state.vref.insert(vid, BTreeSet::from([id]));
        bv
    }

    fn vref_of(state: &State, vid: ValueId) -> BTreeSet<u32> {
        state.vref.get(&vid).cloned().unwrap_or_default()
    }

    fn encode_op(
        &mut self,
        state: &mut State,
        inst: &Instruction,
        prev: Option<BlockId>,
    ) -> (BV, BTreeSet<u32>) {
        match &inst.op {
            Operation::BinaryOp { kind } => {
                if inst.operands.len() < 2 {
                    return self.fresh_val("h");
                }
                let a = self.enc(state, inst.operands[0]);
                let b = self.enc(state, inst.operands[1]);
                let mut vars = Self::vref_of(state, inst.operands[0]);
                vars.extend(Self::vref_of(state, inst.operands[1]));
                let mut guards = Vec::new();
                let result = ssa_encode::encode_binop(*kind, &a, &b, &mut guards);
                for g in guards {
                    let con = self.mk_con(g, vars.clone());
                    state.constraints.push(con);
                }
                match result {
                    Some(bv) => (bv, vars),
                    None => self.fresh_val("h"),
                }
            }
            Operation::Cast { .. } | Operation::Copy | Operation::Freeze => {
                match inst.operands.first() {
                    Some(&o) => {
                        let bv = self.enc(state, o);
                        (bv, Self::vref_of(state, o))
                    }
                    None => self.fresh_val("h"),
                }
            }
            Operation::Select if inst.operands.len() >= 3 => {
                let c = self.enc(state, inst.operands[0]);
                let a = self.enc(state, inst.operands[1]);
                let b = self.enc(state, inst.operands[2]);
                let bv = ssa_encode::truthy(&c).ite(&a, &b);
                let mut vars = Self::vref_of(state, inst.operands[0]);
                vars.extend(Self::vref_of(state, inst.operands[1]));
                vars.extend(Self::vref_of(state, inst.operands[2]));
                (bv, vars)
            }
            Operation::Phi { incoming } => {
                if let Some(prev) = prev {
                    if let Some((_, v)) = incoming.iter().find(|(bb, _)| *bb == prev) {
                        let bv = self.enc(state, *v);
                        return (bv, Self::vref_of(state, *v));
                    }
                }
                self.fresh_val("h")
            }
            Operation::Load => self.exec_load(state, inst),
            // A pointer value: its identity is tracked structurally via `ptr_base`;
            // the bitvector itself is an opaque fresh symbol.
            Operation::Alloca { .. } | Operation::Gep { .. } => self.fresh_val("p"),
            Operation::CallDirect { callee } => {
                if let Some(name) = self.module.function(*callee).map(|f| f.name.to_string()) {
                    if is_scalar_integer_nondet(&name) {
                        return self.fresh_nondet(state, &name);
                    }
                }
                self.fresh_val("h")
            }
            _ => self.fresh_val("h"),
        }
    }

    /// Store `value` into the memory cell of its pointer operand's root alloca (if
    /// resolvable); otherwise a no-op (the load will havoc — sound).
    fn exec_store(&mut self, state: &mut State, inst: &Instruction) {
        if inst.operands.len() < 2 {
            return;
        }
        let (value, ptr) = (inst.operands[0], inst.operands[1]);
        let Some(&base) = self.ptr_base.get(&ptr) else {
            return;
        };
        let bv = self.enc(state, value);
        let vars = Self::vref_of(state, value);
        state.memory.insert(base, bv);
        state.mem_vref.insert(base, vars);
    }

    /// Load from the memory cell of the pointer operand's root alloca, or a fresh
    /// havoc symbol when the cell is unknown / the pointer is unresolvable.
    fn exec_load(&mut self, state: &mut State, inst: &Instruction) -> (BV, BTreeSet<u32>) {
        if let Some(&ptr) = inst.operands.first() {
            if let Some(&base) = self.ptr_base.get(&ptr) {
                if let Some(bv) = state.memory.get(&base) {
                    let vars = state.mem_vref.get(&base).cloned().unwrap_or_default();
                    return (bv.clone(), vars);
                }
            }
        }
        self.fresh_val("h")
    }

    /// A fresh nondet input bitvector constrained to the declared type's range
    /// (R5), recorded in execution order for model read-back.
    fn fresh_nondet(&mut self, state: &mut State, name: &str) -> (BV, BTreeSet<u32>) {
        let (bv, id) = self.fresh("n");
        let vars = BTreeSet::from([id]);
        if let Some((lo, hi)) = ssa_encode::nondet_range(name, self.data_model) {
            let lo_con = self.mk_con(bv.bvsge(BV::from_i64(lo, BV_WIDTH)), vars.clone());
            let hi_con = self.mk_con(bv.bvsle(BV::from_i64(hi, BV_WIDTH)), vars.clone());
            state.constraints.push(lo_con);
            state.constraints.push(hi_con);
        }
        state.nondet.push((bv.clone(), name.to_string()));
        (bv, vars)
    }

    fn fresh(&mut self, tag: &str) -> (BV, u32) {
        let id = self.var_counter;
        self.var_counter += 1;
        (BV::new_const(format!("{tag}{id}"), BV_WIDTH), id)
    }

    fn fresh_val(&mut self, tag: &str) -> (BV, BTreeSet<u32>) {
        let (bv, id) = self.fresh(tag);
        (bv, BTreeSet::from([id]))
    }

    fn mk_con(&mut self, b: Bool, vars: BTreeSet<u32>) -> Con {
        let id = self.con_counter;
        self.con_counter += 1;
        Con { id, b, vars }
    }

    /// A path constraint asserting (the negation of) the truthiness of `cond`.
    fn cond_con(&mut self, state: &mut State, cond: ValueId, negate: bool) -> Con {
        let bv = self.enc(state, cond);
        let mut b = ssa_encode::truthy(&bv);
        if negate {
            b = b.not();
        }
        let vars = Self::vref_of(state, cond);
        self.mk_con(b, vars)
    }

    // -- Terminator forking ---------------------------------------------

    fn step_terminator(
        &mut self,
        mut state: State,
        cur: BlockId,
        term: &Instruction,
        worklist: &mut Vec<State>,
    ) {
        match &term.op {
            Operation::Br { target } => {
                push_succ(state, cur, *target, worklist);
            }
            Operation::CondBr {
                then_target,
                else_target,
            } => {
                if then_target == else_target {
                    push_succ(state, cur, *then_target, worklist);
                    return;
                }
                let Some(&cond) = term.operands.first() else {
                    return;
                };
                let then_con = self.cond_con(&mut state, cond, false);
                if self.feasible(&state, &[then_con.clone()]) {
                    let mut s = state.clone();
                    s.constraints.push(then_con);
                    push_succ(s, cur, *then_target, worklist);
                }
                let else_con = self.cond_con(&mut state, cond, true);
                if self.feasible(&state, &[else_con.clone()]) {
                    let mut s = state.clone();
                    s.constraints.push(else_con);
                    push_succ(s, cur, *else_target, worklist);
                }
            }
            Operation::Switch { default, cases } => {
                self.step_switch(state, cur, term, *default, cases, worklist);
            }
            _ => {} // Ret / Unreachable / … — the path ends here.
        }
    }

    fn step_switch(
        &mut self,
        mut state: State,
        cur: BlockId,
        term: &Instruction,
        default: BlockId,
        cases: &[(i64, BlockId)],
        worklist: &mut Vec<State>,
    ) {
        let Some(&disc_v) = term.operands.first() else {
            return;
        };
        if cases.is_empty() {
            push_succ(state, cur, default, worklist);
            return;
        }
        let disc = self.enc(&mut state, disc_v);
        let dvars = Self::vref_of(&state, disc_v);

        let mut targets: BTreeSet<BlockId> = BTreeSet::new();
        targets.insert(default);
        for (_, t) in cases {
            targets.insert(*t);
        }

        for t in targets {
            let matching: Vec<i64> = cases
                .iter()
                .filter(|(_, tt)| *tt == t)
                .map(|(v, _)| *v)
                .collect();
            let extras: Vec<Con> = if matching.is_empty() {
                // Default target: discriminant differs from every case value.
                cases
                    .iter()
                    .map(|(v, _)| {
                        let b = ssa_encode::eq(&disc, &BV::from_i64(*v, BV_WIDTH)).not();
                        self.mk_con(b, dvars.clone())
                    })
                    .collect()
            } else {
                let ors: Vec<Bool> = matching
                    .iter()
                    .map(|v| ssa_encode::eq(&disc, &BV::from_i64(*v, BV_WIDTH)))
                    .collect();
                vec![self.mk_con(Bool::or(&ors), dvars.clone())]
            };
            if self.feasible(&state, &extras) {
                let mut s = state.clone();
                s.constraints.extend(extras);
                push_succ(s, cur, t, worklist);
            }
        }
    }

    // -- Solving ---------------------------------------------------------

    /// Feasibility of `state.constraints ∧ extras` under constraint independence:
    /// only the constraints sharing variables (transitively) with `extras`, plus
    /// all variable-free constraints, are sent to Z3. Memoized on the query's
    /// constraint-id set.
    fn feasible(&mut self, state: &State, extras: &[Con]) -> bool {
        self.solver_calls += 1;
        if self.solver_calls > SE_MAX_SOLVER_CALLS {
            return false; // budget exhausted → prune (sound: only costs recall)
        }

        // Seed the variable frontier from the extra constraints.
        let mut vs: BTreeSet<u32> = BTreeSet::new();
        for e in extras {
            vs.extend(e.vars.iter().copied());
        }
        // Variable-free path constraints are always relevant (they can be UNSAT on
        // their own, e.g. assume(0)).
        let mut chosen_ids: BTreeSet<u32> = BTreeSet::new();
        let mut chosen: Vec<&Bool> = Vec::new();
        for c in &state.constraints {
            if c.vars.is_empty() {
                chosen_ids.insert(c.id);
                chosen.push(&c.b);
            }
        }
        // Grow the connected component of `vs` to a fixpoint.
        loop {
            let mut grew = false;
            for c in &state.constraints {
                if chosen_ids.contains(&c.id) || c.vars.is_disjoint(&vs) {
                    continue;
                }
                chosen_ids.insert(c.id);
                chosen.push(&c.b);
                vs.extend(c.vars.iter().copied());
                grew = true;
            }
            if !grew {
                break;
            }
        }

        let mut key: Vec<u32> = chosen_ids.into_iter().collect();
        for e in extras {
            key.push(e.id);
        }
        key.sort_unstable();
        if let Some(&cached) = self.cache.get(&key) {
            return cached;
        }

        let solver = new_solver();
        for b in chosen {
            solver.assert(b);
        }
        for e in extras {
            solver.assert(&e.b);
        }
        let result = matches!(solver.check(), SatResult::Sat);
        self.cache.insert(key, result);
        result
    }

    /// Solve the full path condition at a reached `reach_error`; on SAT build a
    /// replay candidate reading each nondet input's model value in execution order.
    fn try_solve(&mut self, state: &State, err_inst: InstId) -> Option<FalseCandidate> {
        self.solver_calls += 1;
        let solver = new_solver();
        for c in &state.constraints {
            solver.assert(&c.b);
        }
        if !matches!(solver.check(), SatResult::Sat) {
            return None;
        }
        let model = solver.get_model()?;
        let mut nondet_sequence = Vec::new();
        for (bv, name) in &state.nondet {
            let value = model.eval(bv, true).and_then(|b| b.as_i64()).unwrap_or(0);
            nondet_sequence.push(NondetCall {
                func_name: name.clone(),
                value,
            });
        }
        Some(FalseCandidate {
            reach_error_inst: err_inst,
            block_path: state.trace.clone(),
            assignments: BTreeMap::new(),
            nondet_sequence,
        })
    }

    // -- Concolic flip-seed generation ----------------------------------

    /// Single concrete path (guided by `seq`) with per-branch flip solving. See
    /// [`enumerate_concolic_flip_seeds`] for the mechanism/soundness contract.
    // NOTE: the concrete-execute-then-fork-per-terminator loop is one cohesive unit
    // (instruction stepping, nondet pinning, branch/switch flip solving); splitting it
    // would scatter the shared state (pins, cursor, flips) across helpers.
    #[allow(clippy::too_many_lines)]
    fn concolic_run(&mut self, entry: BlockId, seq: &[NondetCall]) -> Vec<Vec<NondetCall>> {
        let mut state = State::new(entry);
        // Concrete-value pins (`nondet_k == seq[k]`), used ONLY to decide which
        // branch direction the seed took — never added to the path condition, so a
        // flip solve leaves the nondet inputs free.
        let mut pins: Vec<Con> = Vec::new();
        let mut cursor = 0usize;
        let mut flips: Vec<Vec<NondetCall>> = Vec::new();
        let mut seen: BTreeSet<Vec<(String, i64)>> = BTreeSet::new();
        let mut branches = 0usize;

        loop {
            if flips.len() >= CONC_MAX_FLIPS
                || branches >= CONC_MAX_BRANCHES
                || self.steps >= SE_MAX_STEPS
                || self.solver_calls >= SE_MAX_SOLVER_CALLS
            {
                break;
            }
            let cur = state.block;
            let count = state.visits.entry(cur).or_insert(0);
            *count += 1;
            if *count > CONC_BLOCK_VISIT_CAP
                || state.trace.len() >= SE_MAX_TRACE
                || state.constraints.len() >= SE_MAX_CON
            {
                break;
            }
            state.trace.push(cur);
            let Some(block) = self.block_by_id.get(&cur).copied() else {
                break;
            };
            let prev = state.prev;

            // Non-terminator instructions: intercept scalar nondet reads (to pin the
            // concrete value) and error calls (concrete seed already at the sink →
            // stop this path); everything else reuses the shared SSA encoder.
            let mut hit_error = false;
            for inst in &block.instructions {
                if inst.is_terminator() {
                    break;
                }
                self.steps += 1;
                if self.steps >= SE_MAX_STEPS {
                    return flips;
                }
                if let Operation::CallDirect { callee } = &inst.op {
                    let name = self.module.function(*callee).map(|f| f.name.as_str());
                    if let Some(name) = name {
                        if ERROR_NAMES.contains(&name) {
                            hit_error = true;
                            break;
                        }
                        if is_scalar_integer_nondet(name) {
                            if let Some(dst) = inst.dst {
                                let concrete = seq.get(cursor).map_or(0, |c| c.value);
                                cursor += 1;
                                let name = name.to_string();
                                let (bv, vars) =
                                    self.concolic_nondet(&mut state, &name, concrete, &mut pins);
                                state.values.insert(dst, bv);
                                state.vref.insert(dst, vars);
                            }
                            continue;
                        }
                    }
                }
                self.define(&mut state, inst, prev);
            }
            if hit_error {
                break;
            }

            let Some(term) = block.terminator() else {
                break;
            };
            match &term.op {
                Operation::Br { target } => {
                    state.prev = Some(cur);
                    state.block = *target;
                }
                Operation::CondBr {
                    then_target,
                    else_target,
                } => {
                    if then_target == else_target {
                        state.prev = Some(cur);
                        state.block = *then_target;
                        continue;
                    }
                    let Some(&cond) = term.operands.first() else {
                        break;
                    };
                    branches += 1;
                    let then_con = self.cond_con(&mut state, cond, false);
                    let else_con = self.cond_con(&mut state, cond, true);
                    let took_then = self.concolic_dir(&state, &pins, &then_con);
                    let (taken, flip) = if took_then {
                        (then_con, else_con)
                    } else {
                        (else_con, then_con)
                    };
                    // Flip goal: reach this branch (prefix) then diverge (`flip`).
                    if let Some(s) = self.solve_flip(&state, &flip) {
                        let key: Vec<(String, i64)> =
                            s.iter().map(|c| (c.func_name.clone(), c.value)).collect();
                        if seen.insert(key) {
                            flips.push(s);
                        }
                    }
                    state.constraints.push(taken);
                    state.prev = Some(cur);
                    state.block = if took_then {
                        *then_target
                    } else {
                        *else_target
                    };
                }
                Operation::Switch { default, cases } => {
                    branches += 1;
                    if let Some(next) = self.concolic_switch(
                        &mut state, term, *default, cases, &pins, &mut flips, &mut seen,
                    ) {
                        state.prev = Some(cur);
                        state.block = next;
                    } else {
                        break;
                    }
                }
                _ => break, // Ret / Unreachable — the concrete path ends here.
            }
        }
        flips
    }

    /// Concolic switch step: decide the concrete target under the pins, solve a flip
    /// seed for every OTHER reachable target, and return the taken target.
    // NOTE: the arguments (state, term, default, cases, pins, flips, seen) are the
    // full concolic context for one terminator; bundling them into a struct would
    // only obscure this single-use helper.
    #[allow(clippy::too_many_arguments)]
    fn concolic_switch(
        &mut self,
        state: &mut State,
        term: &Instruction,
        default: BlockId,
        cases: &[(i64, BlockId)],
        pins: &[Con],
        flips: &mut Vec<Vec<NondetCall>>,
        seen: &mut BTreeSet<Vec<(String, i64)>>,
    ) -> Option<BlockId> {
        let &disc_v = term.operands.first()?;
        let disc = self.enc(state, disc_v);
        let dvars = Self::vref_of(state, disc_v);

        let mut targets: BTreeSet<BlockId> = BTreeSet::new();
        targets.insert(default);
        for (_, t) in cases {
            targets.insert(*t);
        }
        // Build one match-condition per target (deterministic BlockId order).
        let cons: Vec<(BlockId, Con)> = targets
            .iter()
            .map(|&t| (t, self.switch_target_con(&disc, &dvars, cases, t)))
            .collect();

        // Concrete target = the first (ordered) whose condition holds under the pins.
        let taken = cons
            .iter()
            .find(|(_, con)| self.concolic_dir(state, pins, con))
            .map_or(default, |(t, _)| *t);

        for (t, con) in &cons {
            if *t == taken || flips.len() >= CONC_MAX_FLIPS {
                continue;
            }
            if let Some(s) = self.solve_flip(state, con) {
                let key: Vec<(String, i64)> =
                    s.iter().map(|c| (c.func_name.clone(), c.value)).collect();
                if seen.insert(key) {
                    flips.push(s);
                }
            }
        }
        // Commit the taken condition to the path and continue.
        if let Some((_, con)) = cons.into_iter().find(|(t, _)| *t == taken) {
            state.constraints.push(con);
        }
        Some(taken)
    }

    /// The Z3 condition for `switch` discriminant `disc` selecting `target`: a
    /// disjunction of `disc == case_value` for the case(s) routing to `target`, or —
    /// for the default target — the conjunction of `disc != v` over every case value.
    fn switch_target_con(
        &mut self,
        disc: &BV,
        dvars: &BTreeSet<u32>,
        cases: &[(i64, BlockId)],
        target: BlockId,
    ) -> Con {
        let matching: Vec<i64> = cases
            .iter()
            .filter(|(_, t)| *t == target)
            .map(|(v, _)| *v)
            .collect();
        let b = if matching.is_empty() {
            let nes: Vec<Bool> = cases
                .iter()
                .map(|(v, _)| ssa_encode::eq(disc, &BV::from_i64(*v, BV_WIDTH)).not())
                .collect();
            let refs: Vec<&Bool> = nes.iter().collect();
            Bool::and(&refs)
        } else {
            let ors: Vec<Bool> = matching
                .iter()
                .map(|v| ssa_encode::eq(disc, &BV::from_i64(*v, BV_WIDTH)))
                .collect();
            Bool::or(&ors)
        };
        self.mk_con(b, dvars.clone())
    }

    /// A fresh nondet input, additionally *pinned* to the concrete value the stuck
    /// seed produced (clamped into the declared range so the pin is consistent with
    /// the range constraints `fresh_nondet` already asserted). The pin decides branch
    /// direction only; it is never part of the path condition a flip solves.
    fn concolic_nondet(
        &mut self,
        state: &mut State,
        name: &str,
        concrete: i64,
        pins: &mut Vec<Con>,
    ) -> (BV, BTreeSet<u32>) {
        let (bv, vars) = self.fresh_nondet(state, name);
        let c = ssa_encode::nondet_range(name, self.data_model)
            .map_or(concrete, |(lo, hi)| concrete.clamp(lo, hi));
        let pin = self.mk_con(
            ssa_encode::eq(&bv, &BV::from_i64(c, BV_WIDTH)),
            vars.clone(),
        );
        pins.push(pin);
        (bv, vars)
    }

    /// Did the concrete seed take the `then` side (guard `then_con` holds under the
    /// pins)? A single constraint-sliced feasibility solve over `state.constraints ∪
    /// pins` (which pin every nondet to its concrete value). On budget exhaustion
    /// defaults to `false` — deterministic, and only affects which seeds we generate.
    fn concolic_dir(&mut self, state: &State, pins: &[Con], then_con: &Con) -> bool {
        self.solver_calls += 1;
        if self.solver_calls > SE_MAX_SOLVER_CALLS {
            return false;
        }
        let base: Vec<&Con> = state.constraints.iter().chain(pins.iter()).collect();
        let chosen = component(&base, &then_con.vars);
        let solver = new_solver();
        for b in chosen {
            solver.assert(b);
        }
        solver.assert(&then_con.b);
        matches!(solver.check(), SatResult::Sat)
    }

    /// Solve for a nondet-input sequence that reaches the current branch and takes the
    /// `flip` (un-taken) side. First tries the exact preimage — the constraint-sliced
    /// path prefix conjoined with `flip`; if that is UNSAT/unknown, falls back to the
    /// *optimistic* solve of `flip` alone (dropping the prefix), matching QSYM/Driller
    /// optimistic solving. Safe because the caller replay-gates every seed. Returns the
    /// model's value for each nondet read so far, or `None` when unsatisfiable / the
    /// flip does not depend on any input.
    fn solve_flip(&mut self, state: &State, flip: &Con) -> Option<Vec<NondetCall>> {
        self.solver_calls += 1;
        if self.solver_calls > SE_MAX_SOLVER_CALLS {
            return None;
        }
        let base: Vec<&Con> = state.constraints.iter().collect();
        let chosen = component(&base, &flip.vars);
        let solver = new_solver();
        for b in chosen {
            solver.assert(b);
        }
        solver.assert(&flip.b);
        let model = if matches!(solver.check(), SatResult::Sat) {
            solver.get_model()
        } else {
            // Optimistic fallback: satisfy only the flipped guard.
            self.solver_calls += 1;
            let s2 = new_solver();
            s2.assert(&flip.b);
            if matches!(s2.check(), SatResult::Sat) {
                s2.get_model()
            } else {
                None
            }
        }?;
        let mut out = Vec::with_capacity(state.nondet.len());
        for (bv, name) in &state.nondet {
            let value = model.eval(bv, true).and_then(|b| b.as_i64()).unwrap_or(0);
            out.push(NondetCall {
                func_name: name.clone(),
                value,
            });
        }
        if out.is_empty() {
            return None;
        }
        Some(out)
    }
}

/// Constraint-independence component: the connected set of constraints reachable from
/// the `seed` variable frontier (plus all variable-free constraints, which are always
/// relevant), as the Z3 booleans to assert. Shared by the concolic direction and flip
/// solves — sending Z3 only the relevant slice keeps each solve small.
fn component<'c>(cons: &[&'c Con], seed: &BTreeSet<u32>) -> Vec<&'c Bool> {
    let mut vs = seed.clone();
    let mut chosen_ids: BTreeSet<u32> = BTreeSet::new();
    let mut chosen: Vec<&Bool> = Vec::new();
    for c in cons {
        if c.vars.is_empty() && chosen_ids.insert(c.id) {
            chosen.push(&c.b);
        }
    }
    loop {
        let mut grew = false;
        for c in cons {
            if chosen_ids.contains(&c.id) || c.vars.is_disjoint(&vs) {
                continue;
            }
            chosen_ids.insert(c.id);
            chosen.push(&c.b);
            vs.extend(c.vars.iter().copied());
            grew = true;
        }
        if !grew {
            break;
        }
    }
    chosen
}

/// Push `state` onto the worklist advanced to `target`, recording `cur` as the
/// predecessor (for phi resolution).
fn push_succ(mut state: State, cur: BlockId, target: BlockId, worklist: &mut Vec<State>) {
    state.prev = Some(cur);
    state.block = target;
    worklist.push(state);
}

/// Constant → bitvector (integers / null / zero-init only; other constants have no
/// integer encoding and are left for a fresh havoc symbol).
fn const_bv(c: &Constant) -> Option<BV> {
    match c {
        Constant::Int { value, .. } => Some(BV::from_i64(*value, BV_WIDTH)),
        Constant::Null | Constant::ZeroInit => Some(BV::from_i64(0, BV_WIDTH)),
        _ => None,
    }
}

/// Fresh deterministic Z3 solver for one query.
fn new_solver() -> z3::Solver {
    let solver = z3::Solver::new();
    let mut params = z3::Params::new();
    params.set_u32("timeout", SE_Z3_TIMEOUT_MS);
    params.set_u32("rlimit", SE_Z3_RLIMIT);
    params.set_u32("random_seed", SE_Z3_SEED);
    solver.set_params(&params);
    solver
}

/// Map each pointer value that aliases an alloca (through `copy`/`cast`/`freeze`)
/// to that root alloca's `ValueId`, so scalar loads/stores hit the same memory
/// cell. A fixpoint over the (small) instruction set; only identity-preserving
/// forwarders are followed (offsetting geps are deliberately excluded — an
/// unresolved pointer just havocs, which stays sound).
fn compute_ptr_base(func: &AirFunction) -> BTreeMap<ValueId, ValueId> {
    let mut base: BTreeMap<ValueId, ValueId> = BTreeMap::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let (Operation::Alloca { .. }, Some(dst)) = (&inst.op, inst.dst) {
                base.insert(dst, dst);
            }
        }
    }
    // Propagate through identity forwarders to a fixpoint (bounded passes).
    for _ in 0..8 {
        let mut changed = false;
        for block in &func.blocks {
            for inst in &block.instructions {
                let Some(dst) = inst.dst else { continue };
                if base.contains_key(&dst) {
                    continue;
                }
                if matches!(
                    &inst.op,
                    Operation::Copy | Operation::Cast { .. } | Operation::Freeze
                ) {
                    if let Some(&src) = inst.operands.first() {
                        if let Some(&root) = base.get(&src) {
                            base.insert(dst, root);
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, BinaryOp};
    use saf_core::ids::ModuleId;

    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }
    fn bid(n: u128) -> BlockId {
        BlockId::new(n)
    }
    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }
    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }

    fn decl(id: FunctionId, name: &str) -> AirFunction {
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn func(id: FunctionId, name: &str, blocks: Vec<AirBlock>, entry: BlockId) -> AirFunction {
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn blk(id: BlockId, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id,
            label: None,
            instructions,
        }
    }

    fn binop(id: u128, kind: BinaryOp, a: ValueId, b: ValueId, dst: ValueId) -> Instruction {
        Instruction::new(iid(id), Operation::BinaryOp { kind })
            .with_operands(vec![a, b])
            .with_dst(dst)
    }

    /// Build `main` implementing:
    /// ```c
    /// int x = __VERIFIER_nondet_int();
    /// int s = 0;
    /// for (int i = 0; i < 4; i++) s += x;   // loop-carried accumulator
    /// if (s == target) reach_error();
    /// ```
    /// as mem2reg'd SSA (phi loop) — the loop-carried class the acyclic BMC base
    /// case cannot crack. `target == 40` is SAT (x = 10); a non-multiple-of-4 is
    /// UNSAT.
    fn build_loop_module(target: i64) -> AirModule {
        build_counted_loop_module(4, target)
    }

    /// As [`build_loop_module`] but with an explicit loop bound: `s = bound * x`
    /// after `for (i=0;i<bound;i++) s += x;`, then `if (s == target) reach_error()`.
    fn build_counted_loop_module(bound: i64, target: i64) -> AirModule {
        let (main_id, nd_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, header, body, exit, err_bb, ret_bb) =
            (bid(10), bid(11), bid(12), bid(13), bid(14), bid(15));

        let x = vid(100);
        let zero = vid(101);
        let four = vid(102);
        let one = vid(103);
        let tgt = vid(104);
        // phis
        let i_phi = vid(110);
        let s_phi = vid(111);
        // body
        let s_next = vid(112);
        let i_next = vid(113);
        // header cmp
        let icmp_lt = vid(114);
        // exit cmp
        let scmp_eq = vid(115);

        let entry_insts = vec![
            Instruction::new(iid(1), Operation::CallDirect { callee: nd_id }).with_dst(x),
            Instruction::new(iid(2), Operation::Br { target: header }),
        ];
        let header_insts = vec![
            Instruction::new(
                iid(3),
                Operation::Phi {
                    incoming: vec![(entry, zero), (body, i_next)],
                },
            )
            .with_dst(i_phi),
            Instruction::new(
                iid(4),
                Operation::Phi {
                    incoming: vec![(entry, zero), (body, s_next)],
                },
            )
            .with_dst(s_phi),
            binop(5, BinaryOp::ICmpSlt, i_phi, four, icmp_lt),
            Instruction::new(
                iid(6),
                Operation::CondBr {
                    then_target: body,
                    else_target: exit,
                },
            )
            .with_operands(vec![icmp_lt]),
        ];
        let body_insts = vec![
            binop(7, BinaryOp::Add, s_phi, x, s_next),
            binop(8, BinaryOp::Add, i_phi, one, i_next),
            Instruction::new(iid(9), Operation::Br { target: header }),
        ];
        let exit_insts = vec![
            binop(10, BinaryOp::ICmpEq, s_phi, tgt, scmp_eq),
            Instruction::new(
                iid(11),
                Operation::CondBr {
                    then_target: err_bb,
                    else_target: ret_bb,
                },
            )
            .with_operands(vec![scmp_eq]),
        ];
        let err_insts = vec![
            Instruction::new(iid(12), Operation::CallDirect { callee: err_id }),
            Instruction::new(iid(13), Operation::Ret),
        ];
        let ret_insts = vec![Instruction::new(iid(14), Operation::Ret)];

        let main = func(
            main_id,
            "main",
            vec![
                blk(entry, entry_insts),
                blk(header, header_insts),
                blk(body, body_insts),
                blk(exit, exit_insts),
                blk(err_bb, err_insts),
                blk(ret_bb, ret_insts),
            ],
            entry,
        );

        let mut module = AirModule::new(ModuleId::new(1));
        module
            .constants
            .insert(zero, Constant::Int { value: 0, bits: 32 });
        module.constants.insert(
            four,
            Constant::Int {
                value: bound,
                bits: 32,
            },
        );
        module
            .constants
            .insert(one, Constant::Int { value: 1, bits: 32 });
        module.constants.insert(
            tgt,
            Constant::Int {
                value: target,
                bits: 32,
            },
        );
        module.functions.push(main);
        module.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));
        module.functions.push(decl(err_id, "reach_error"));
        module
    }

    #[test]
    fn gate_requires_cycle_and_nondet() {
        let module = build_loop_module(40);
        let main = module.function_by_name("main").unwrap();
        assert!(gate(main, &module), "loop with nondet input must gate in");
    }

    #[test]
    fn gate_rejects_acyclic() {
        // A straight-line nondet==C guard (no loop) — BMC/guard already cover it.
        let (main_id, nd_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, err_bb, ret_bb) = (bid(10), bid(11), bid(12));
        let x = vid(100);
        let c = vid(101);
        let cmp = vid(102);
        let insts = vec![
            Instruction::new(iid(1), Operation::CallDirect { callee: nd_id }).with_dst(x),
            binop(2, BinaryOp::ICmpEq, x, c, cmp),
            Instruction::new(
                iid(3),
                Operation::CondBr {
                    then_target: err_bb,
                    else_target: ret_bb,
                },
            )
            .with_operands(vec![cmp]),
        ];
        let main = func(
            main_id,
            "main",
            vec![
                blk(entry, insts),
                blk(
                    err_bb,
                    vec![Instruction::new(
                        iid(4),
                        Operation::CallDirect { callee: err_id },
                    )],
                ),
                blk(ret_bb, vec![Instruction::new(iid(5), Operation::Ret)]),
            ],
            entry,
        );
        let mut module = AirModule::new(ModuleId::new(1));
        module
            .constants
            .insert(c, Constant::Int { value: 5, bits: 32 });
        module.functions.push(main);
        module.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));
        module.functions.push(decl(err_id, "reach_error"));
        let f = module.function_by_name("main").unwrap();
        assert!(!gate(f, &module), "acyclic function must NOT gate in");
    }

    #[test]
    fn solves_loop_carried_accumulator() {
        // s = 4*x after the loop; target 40 → x = 10 (SAT).
        let module = build_loop_module(40);
        let config = PropertyAnalysisConfig::default();
        let cands = enumerate_se_candidates(&module, &config, DataModel::LP64);
        assert_eq!(cands.len(), 1, "expected one SE candidate for s==40");
        let seq = &cands[0].nondet_sequence;
        assert_eq!(seq.len(), 1, "one nondet input read before the loop");
        assert_eq!(seq[0].func_name, "__VERIFIER_nondet_int");
        assert_eq!(seq[0].value, 10, "4*x == 40 → x == 10");
    }

    #[test]
    fn unsat_loop_guard_yields_no_candidate() {
        // s = 4*x can never equal 41 (not a multiple of 4) → UNSAT → no candidate.
        let module = build_loop_module(41);
        let config = PropertyAnalysisConfig::default();
        let cands = enumerate_se_candidates(&module, &config, DataModel::LP64);
        assert!(
            cands.is_empty(),
            "4*x == 41 has no integer solution → no candidate"
        );
    }

    #[test]
    fn candidate_is_deterministic() {
        let module = build_loop_module(40);
        let config = PropertyAnalysisConfig::default();
        let a = enumerate_se_candidates(&module, &config, DataModel::LP64);
        let b = enumerate_se_candidates(&module, &config, DataModel::LP64);
        assert_eq!(a.len(), b.len());
        assert_eq!(a[0].nondet_sequence, b[0].nondet_sequence);
        assert_eq!(a[0].block_path, b[0].block_path);
    }

    #[test]
    fn cycle_detector_flags_loop_and_clears_acyclic() {
        let looped = build_loop_module(40);
        assert!(cfg_has_cycle(looped.function_by_name("main").unwrap()));
    }

    /// Build `main` implementing `int x = nondet(); if (x == guard) reach_error();`
    /// (acyclic equality guard) — the concolic flip target.
    fn build_eq_guard_module(guard: i64) -> AirModule {
        let (main_id, nd_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, err_bb, ret_bb) = (bid(10), bid(11), bid(12));
        let x = vid(100);
        let c = vid(101);
        let cmp = vid(102);
        let insts = vec![
            Instruction::new(iid(1), Operation::CallDirect { callee: nd_id }).with_dst(x),
            binop(2, BinaryOp::ICmpEq, x, c, cmp),
            Instruction::new(
                iid(3),
                Operation::CondBr {
                    then_target: err_bb,
                    else_target: ret_bb,
                },
            )
            .with_operands(vec![cmp]),
        ];
        let main = func(
            main_id,
            "main",
            vec![
                blk(entry, insts),
                blk(
                    err_bb,
                    vec![
                        Instruction::new(iid(4), Operation::CallDirect { callee: err_id }),
                        Instruction::new(iid(5), Operation::Ret),
                    ],
                ),
                blk(ret_bb, vec![Instruction::new(iid(6), Operation::Ret)]),
            ],
            entry,
        );
        let mut module = AirModule::new(ModuleId::new(1));
        module.constants.insert(
            c,
            Constant::Int {
                value: guard,
                bits: 32,
            },
        );
        module.functions.push(main);
        module.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));
        module.functions.push(decl(err_id, "reach_error"));
        module
    }

    #[test]
    fn concolic_flips_equality_guard() {
        // A stuck seed drove x = 0 (the else side). Concolic must solve the flip:
        // x == 42 to take the reach_error branch.
        let module = build_eq_guard_module(42);
        let seq = vec![NondetCall {
            func_name: "__VERIFIER_nondet_int".to_string(),
            value: 0,
        }];
        let flips = enumerate_concolic_flip_seeds(&module, &seq, DataModel::LP64);
        assert!(
            flips.iter().any(|f| f.len() == 1
                && f[0].value == 42
                && f[0].func_name == "__VERIFIER_nondet_int"),
            "expected a flip seed x == 42, got {flips:?}"
        );
    }

    #[test]
    fn concolic_flip_is_deterministic() {
        let module = build_eq_guard_module(1337);
        let seq = vec![NondetCall {
            func_name: "__VERIFIER_nondet_int".to_string(),
            value: 0,
        }];
        let a = enumerate_concolic_flip_seeds(&module, &seq, DataModel::LP64);
        let b = enumerate_concolic_flip_seeds(&module, &seq, DataModel::LP64);
        assert_eq!(a, b);
        assert!(a.iter().any(|f| f.iter().any(|c| c.value == 1337)));
    }

    #[test]
    fn concolic_solves_deep_counted_loop_past_forking_cap() {
        // `for (i=0;i<20;i++) s+=x; if (s == 140) reach_error();` — s = 20*x, so
        // x = 7. The loop needs 20 iterations, well past the forking engine's
        // SE_BLOCK_VISIT_CAP (8), so `enumerate_se_candidates` cannot reach the guard;
        // the deeper concolic visit cap follows the whole concrete loop and flips it.
        assert!(20 > SE_BLOCK_VISIT_CAP, "test must exceed the forking cap");
        let module = build_counted_loop_module(20, 140);
        // A trivial (all-zero) plateau seed still drives the input-independent loop to
        // completion, so no fuzz-discovered depth is even required here.
        let seq = vec![NondetCall {
            func_name: "__VERIFIER_nondet_int".to_string(),
            value: 0,
        }];
        let flips = enumerate_concolic_flip_seeds(&module, &seq, DataModel::LP64);
        assert!(
            flips.iter().any(|f| f.iter().any(|c| c.value == 7)),
            "expected a flip seed x == 7 (20*x == 140), got {flips:?}"
        );
    }

    #[test]
    fn concolic_no_error_function_yields_no_flips() {
        // A module with nondet but no reach_error: nothing to flip toward.
        let (main_id, nd_id) = (fid(1), fid(2));
        let entry = bid(10);
        let x = vid(100);
        let main = func(
            main_id,
            "main",
            vec![blk(
                entry,
                vec![
                    Instruction::new(iid(1), Operation::CallDirect { callee: nd_id }).with_dst(x),
                    Instruction::new(iid(2), Operation::Ret),
                ],
            )],
            entry,
        );
        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(main);
        module.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));
        let flips = enumerate_concolic_flip_seeds(&module, &[], DataModel::LP64);
        assert!(flips.is_empty());
    }
}
