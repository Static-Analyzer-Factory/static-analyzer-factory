//! Incremental Bounded Model Checking (BMC) for the `unreach-call` property.
//!
//! # What this adds over fixed-k path BMC ([`crate::bmc`]) and forward SE ([`crate::se_interp`])
//!
//! [`crate::bmc`] only unwinds loops to their acyclic base case (`k = 1`), so a
//! violation reachable only after a loop runs *several* iterations
//! (`for (i=0;i<20;i++) s+=x; if (s==60) reach_error();`) is proposed with the
//! accumulator un-grown and the guard UNSAT. [`crate::se_interp`] does unwind
//! loops, but by *forking* on every branch under a per-block visit cap of 8 and a
//! bounded state/solver-call budget — so a loop that needs, say, 20 iterations to
//! reach the error is beyond its cap, and even shallower loops burn the fork
//! budget on the exponential branch tree.
//!
//! This engine instead does classic CBMC/ESBMC-style **incremental** BMC: it keeps
//! ONE persistent Z3 context and unwinds the loop step by step, encoding each new
//! iteration's transition relation *permanently* into that context and, at every
//! depth `k`, asking a single **`check-sat-assuming(a_k)`** query — gated by a fresh
//! activation literal `a_k` — whether the `reach_error` site becomes reachable when
//! the loop exits after exactly `k` iterations. Because the transition relation and
//! Z3's learned clauses are reused across depths (only the per-depth exit check is
//! transient, deactivated by never re-asserting `a_k`), the engine reaches an order
//! of magnitude deeper unwinding per solver budget than either re-solving whole
//! paths from scratch (fixed-k) or forking every branch (SE).
//!
//! # Soundness
//!
//! Like the sibling engines, this is UNSOUND on its own (64-bit-uniform bitvector
//! arithmetic, identity casts, havoc for un-modelled memory / calls, a single
//! committed body path per iteration). It emits only [`FalseCandidate`]s, fed
//! through the SAME native concrete-replay gate as every other stage — a spurious
//! or imprecise model can only ever fail to reproduce → `unknown`. Never a verdict
//! from a solver model alone (confirmer contract R6). The nondet inputs are read in
//! execution order (prefix, then each unwound body iteration, then the exit path),
//! so the replay driver pins them per call in the order the program consumes them.
//!
//! # Cost
//!
//! Fires only for a `reach_error`-containing function that has a scalar-int nondet
//! input AND a CFG cycle (the loop-carried class the acyclic base case cannot
//! crack). Unwind depth, exit-path count, per-check Z3 timeout, and the total
//! solver-call budget are all hard-capped, and the unwinding stops early the moment
//! the permanent loop condition becomes infeasible (a bounded loop cannot iterate
//! further) or a check times out — so a gated task adds only a bounded budget.

use std::collections::{BTreeMap, BTreeSet};

use saf_analysis::z3_utils::reachability::block_paths_between;
use saf_core::air::{AirBlock, AirFunction, AirModule, Constant, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};
use z3::SatResult;
use z3::ast::{BV, Bool};

use crate::property::{
    FalseCandidate, NondetCall, PropertyAnalysisConfig, is_scalar_integer_nondet,
};
use crate::property_kind::DataModel;
use crate::ssa_encode::{self, BV_WIDTH};

/// Max loop-unwind depth (iterations) explored in the persistent context.
const INC_MAX_K: u32 = 48;

/// Max `reach_error`-containing functions unwound per module.
const INC_MAX_SITES: usize = 2;

/// Max distinct simple exit paths (header → error block) checked per depth.
const INC_MAX_EXIT_PATHS: usize = 3;

/// Max simple paths inspected when picking the single per-iteration body path.
const INC_MAX_BODY_PATHS: usize = 8;

/// Max blocks in one state's witness trace (bounds clone / lowering cost).
const INC_MAX_TRACE: usize = 600;

/// Total `check-sat` / `check-sat-assuming` queries per function.
const INC_MAX_SOLVER_CALLS: usize = 240;

/// Per-query Z3 timeout (ms). Kept small: each incremental check is a QF_BV solve
/// that reuses the previous depth's learned clauses. The cap only bounds
/// pathological inputs so a gated task cannot blow the task budget.
const INC_Z3_TIMEOUT_MS: u32 = 300;

/// Deterministic Z3 rlimit and seed (determinism: NFR-DET).
const INC_Z3_RLIMIT: u32 = 4_000_000;
const INC_Z3_SEED: u32 = 0;

const ERROR_NAMES: &[&str] = &["reach_error", "__VERIFIER_error"];

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Enumerate incremental-BMC FALSE *candidates* for `unreach-call`.
///
/// For each `reach_error` / `__VERIFIER_error`-containing function that has a
/// scalar-int nondet input and a CFG cycle, unwinds the loop in one persistent Z3
/// context up to [`INC_MAX_K`] and, at the first depth whose exit-to-error path is
/// SAT, emits a [`FalseCandidate`] whose `nondet_sequence` is the concrete input
/// vector read from the model in execution order. Candidates MUST be confirmed by
/// native replay before any verdict (see module docs).
#[must_use]
pub fn enumerate_incremental_candidates(
    module: &AirModule,
    config: &PropertyAnalysisConfig,
    data_model: DataModel,
) -> Vec<FalseCandidate> {
    let mut out = Vec::new();
    let mut sites_done = 0usize;

    for (func_id, error_block, error_inst) in error_call_sites(module) {
        if sites_done >= INC_MAX_SITES {
            break;
        }
        let Some(func) = module.function(func_id) else {
            continue;
        };
        if func.is_declaration || !incremental_gate(func, module) {
            continue;
        }
        let Some(info) = find_target_loop(func, module, error_block) else {
            continue;
        };
        sites_done += 1;
        if let Some(cand) =
            run_incremental(func, module, data_model, &info, error_block, error_inst)
        {
            out.push(cand);
        }
    }

    let _ = config; // config timeouts are advisory; the engine uses its own caps.
    out
}

/// The `reach_error` / `__VERIFIER_error` call sites as `(func, block, inst)`.
fn error_call_sites(module: &AirModule) -> Vec<(FunctionId, BlockId, InstId)> {
    let mut sites = Vec::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if module
                        .function(*callee)
                        .is_some_and(|t| ERROR_NAMES.contains(&t.name.as_str()))
                    {
                        sites.push((func.id, block.id, inst.id));
                    }
                }
            }
        }
    }
    sites
}

/// Gate: a scalar-int nondet input **and** a CFG cycle — the loop-carried class the
/// acyclic BMC base case cannot handle. Acyclic functions are left to the cheaper
/// guard / fixed-k stages, so skipping them keeps per-task cost bounded.
fn incremental_gate(func: &AirFunction, module: &AirModule) -> bool {
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
    has_nondet && !back_edges(func).is_empty()
}

// ---------------------------------------------------------------------------
// Loop discovery
// ---------------------------------------------------------------------------

/// A canonical natural loop chosen for unwinding toward the error block.
struct LoopInfo {
    /// Loop header (the back edge's target; where the loop-carried phis live).
    header: BlockId,
    /// The header successor that stays inside the loop (the "continue" edge).
    continue_succ: BlockId,
    /// The latch block (source of the back edge to `header`).
    latch: BlockId,
    /// A simple path `entry → … → header` (header last); encoded once as prefix.
    prefix_path: Vec<BlockId>,
    /// A simple path `continue_succ → … → latch` (both inclusive) staying inside
    /// the loop; one iteration of the transition relation.
    body_path: Vec<BlockId>,
    /// Simple paths `header → … → error_block` (header first); the exit checks.
    exit_paths: Vec<Vec<BlockId>>,
}

/// The successor blocks named by `block`'s terminator (deduplicated, ordered).
fn successors(block: &AirBlock) -> Vec<BlockId> {
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

/// DFS colouring for the back-edge finder.
#[derive(Clone, Copy, PartialEq)]
enum Color {
    White,
    Grey,
    Black,
}

/// The `(latch, header)` back edges of `func` (edges to an on-stack ancestor in a
/// DFS from entry), in deterministic sorted order.
fn back_edges(func: &AirFunction) -> Vec<(BlockId, BlockId)> {
    let Some(entry) = func
        .entry_block
        .or_else(|| func.blocks.first().map(|b| b.id))
    else {
        return Vec::new();
    };
    let succ: BTreeMap<BlockId, Vec<BlockId>> =
        func.blocks.iter().map(|b| (b.id, successors(b))).collect();
    let mut color: BTreeMap<BlockId, Color> =
        func.blocks.iter().map(|b| (b.id, Color::White)).collect();
    let mut edges: BTreeSet<(BlockId, BlockId)> = BTreeSet::new();
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
                Color::Grey => {
                    edges.insert((bid, next)); // back edge: latch=bid, header=next
                }
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
    edges.into_iter().collect()
}

/// The natural loop of back edge `latch → header`: `{header}` plus every block that
/// can reach `latch` without passing through `header`.
fn natural_loop(
    header: BlockId,
    latch: BlockId,
    preds: &BTreeMap<BlockId, Vec<BlockId>>,
) -> BTreeSet<BlockId> {
    let mut loop_blocks: BTreeSet<BlockId> = BTreeSet::new();
    loop_blocks.insert(header);
    if latch != header {
        loop_blocks.insert(latch);
        let mut stack = vec![latch];
        while let Some(n) = stack.pop() {
            for &p in preds.get(&n).map_or(&[][..], Vec::as_slice) {
                if loop_blocks.insert(p) {
                    stack.push(p);
                }
            }
        }
    }
    loop_blocks
}

/// Choose one canonical natural loop whose header can reach the error block, and
/// materialize the prefix / body / exit paths needed to unwind it.
fn find_target_loop(
    func: &AirFunction,
    module: &AirModule,
    error_block: BlockId,
) -> Option<LoopInfo> {
    let entry = func
        .entry_block
        .or_else(|| func.blocks.first().map(|b| b.id))?;
    let block_by_id: BTreeMap<BlockId, &AirBlock> = func.blocks.iter().map(|b| (b.id, b)).collect();
    let mut preds: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
    for b in &func.blocks {
        for s in successors(b) {
            preds.entry(s).or_default().push(b.id);
        }
    }

    for (latch, header) in back_edges(func) {
        let loop_blocks = natural_loop(header, latch, &preds);
        let Some(header_block) = block_by_id.get(&header) else {
            continue;
        };
        // The header successor(s) that remain inside the loop = the "continue" edge.
        let Some(continue_succ) = successors(header_block)
            .into_iter()
            .find(|s| loop_blocks.contains(s))
        else {
            continue;
        };
        // Prefix: entry → header (header may equal entry → a length-1 path).
        let prefix_path = block_paths_between(entry, header, func.id, module, 1)
            .into_iter()
            .next()
            .or_else(|| (entry == header).then(|| vec![header]))?;
        // Body: one simple iteration continue_succ → latch, staying in the loop.
        let body_path = if continue_succ == latch {
            vec![latch]
        } else {
            block_paths_between(continue_succ, latch, func.id, module, INC_MAX_BODY_PATHS)
                .into_iter()
                .find(|p| p.iter().all(|b| loop_blocks.contains(b)))?
        };
        // Exit: header → error block (simple; excludes re-entering the loop header).
        let exit_paths =
            block_paths_between(header, error_block, func.id, module, INC_MAX_EXIT_PATHS);
        if exit_paths.is_empty() {
            continue;
        }
        return Some(LoopInfo {
            header,
            continue_succ,
            latch,
            prefix_path,
            body_path,
            exit_paths,
        });
    }
    None
}

// ---------------------------------------------------------------------------
// Incremental unrolling driver
// ---------------------------------------------------------------------------

/// Unwind `info`'s loop in one persistent Z3 context, checking at each depth
/// whether the error becomes reachable. Returns the first replay candidate found.
fn run_incremental(
    func: &AirFunction,
    module: &AirModule,
    data_model: DataModel,
    info: &LoopInfo,
    error_block: BlockId,
    error_inst: InstId,
) -> Option<FalseCandidate> {
    let block_by_id: BTreeMap<BlockId, &AirBlock> = func.blocks.iter().map(|b| (b.id, b)).collect();
    let solver = new_solver();
    let ptr_base = compute_ptr_base(func);
    let mut enc = Enc::new(module, data_model, &solver, &ptr_base);

    // ---- Prefix: entry → header (all blocks strictly before the header). ----
    let prefix = &info.prefix_path;
    for i in 0..prefix.len().saturating_sub(1) {
        let block = *block_by_id.get(&prefix[i])?;
        let prev = if i == 0 { None } else { Some(prefix[i - 1]) };
        enc.encode_block_body(block, prev);
        if let Some(term) = block.terminator() {
            enc.force_branch(term, prefix[i + 1]);
        }
    }
    // Predecessor of the header on entry into iteration 0.
    let mut prev = (prefix.len() >= 2).then(|| prefix[prefix.len() - 2]);

    let header_block = *block_by_id.get(&info.header)?;
    let mut solver_calls = 0usize;

    for k in 0..=INC_MAX_K {
        // Encode the header's phis + guard for iteration k (permanent).
        enc.encode_block_body(header_block, prev);

        // ---- Exit check at depth k (transient, gated by a fresh literal). ----
        for exit_path in &info.exit_paths {
            if solver_calls >= INC_MAX_SOLVER_CALLS {
                return None;
            }
            let act = Bool::fresh_const("bmc_inc_act");
            let saved = enc.snapshot();
            enc.begin_buffer();
            enc.encode_exit(header_block, exit_path, error_block, &block_by_id);
            let buf = enc.end_buffer();
            let exit_nondet: Vec<(BV, String)> = enc.nondet[saved.nondet_len..].to_vec();

            // a_k ⇒ (all exit-path constraints hold).
            solver.assert(act.implies(Bool::and(&buf)));
            solver_calls += 1;
            match solver.check_assumptions(&[act.clone()]) {
                SatResult::Sat => {
                    if let Some(model) = solver.get_model() {
                        let mut seq = Vec::new();
                        for (bv, name) in enc.nondet[..saved.nondet_len]
                            .iter()
                            .chain(exit_nondet.iter())
                        {
                            let value = model.eval(bv, true).and_then(|b| b.as_i64()).unwrap_or(0);
                            seq.push(NondetCall {
                                func_name: name.clone(),
                                value,
                            });
                        }
                        return Some(FalseCandidate {
                            reach_error_inst: error_inst,
                            block_path: build_trace(info, k, exit_path),
                            assignments: BTreeMap::new(),
                            nondet_sequence: seq,
                        });
                    }
                    enc.restore(saved);
                }
                SatResult::Unsat => enc.restore(saved),
                // Timeout / resource-out: deeper unwindings only grow the formula,
                // so stop rather than burn the budget on strictly harder checks.
                SatResult::Unknown => {
                    enc.restore(saved);
                    return None;
                }
            }
        }

        if k == INC_MAX_K || solver_calls >= INC_MAX_SOLVER_CALLS {
            break;
        }

        // ---- Advance: commit "loop continued at iteration k" (permanent). ----
        if let Some(term) = header_block.terminator() {
            enc.force_branch(term, info.continue_succ);
        }
        let body = &info.body_path;
        for j in 0..body.len() {
            let block = *block_by_id.get(&body[j])?;
            let bprev = if j == 0 {
                Some(info.header)
            } else {
                Some(body[j - 1])
            };
            enc.encode_block_body(block, bprev);
            let next = if j + 1 < body.len() {
                body[j + 1]
            } else {
                info.header
            };
            if let Some(term) = block.terminator() {
                enc.force_branch(term, next);
            }
        }
        prev = Some(info.latch);

        // If the permanent loop condition is now infeasible (a bounded loop cannot
        // iterate further), no deeper depth can reach the error → stop.
        solver_calls += 1;
        if matches!(solver.check(), SatResult::Unsat) {
            break;
        }
    }
    None
}

/// Build a (bounded) witness block trace: prefix, then the body repeated `k` times
/// (each closing back to the header), then the exit path.
fn build_trace(info: &LoopInfo, k: u32, exit_path: &[BlockId]) -> Vec<BlockId> {
    let mut trace = info.prefix_path.clone();
    for _ in 0..k {
        if trace.len() >= INC_MAX_TRACE {
            break;
        }
        trace.extend(info.body_path.iter().copied());
        trace.push(info.header);
    }
    for &b in exit_path.iter().skip(1) {
        if trace.len() >= INC_MAX_TRACE {
            break;
        }
        trace.push(b);
    }
    trace
}

/// Fresh deterministic Z3 solver reused across all depths of one function.
fn new_solver() -> z3::Solver {
    let solver = z3::Solver::new();
    let mut params = z3::Params::new();
    params.set_u32("timeout", INC_Z3_TIMEOUT_MS);
    params.set_u32("rlimit", INC_Z3_RLIMIT);
    params.set_u32("random_seed", INC_Z3_SEED);
    solver.set_params(&params);
    solver
}

// ---------------------------------------------------------------------------
// Single-environment SSA → bitvector encoder (persistent context)
// ---------------------------------------------------------------------------

/// A snapshot of the mutable encoder state, taken before a transient exit check so
/// it can be rolled back regardless of the check's outcome.
struct Snapshot {
    env: BTreeMap<ValueId, BV>,
    memory: BTreeMap<ValueId, BV>,
    nondet_len: usize,
}

/// Forward SSA→bitvector encoder over one deterministically-unwound path, asserting
/// permanently into a shared persistent solver (or, in "buffer" mode, collecting
/// constraints for an activation-literal-gated transient check).
struct Enc<'a> {
    module: &'a AirModule,
    data_model: DataModel,
    solver: &'a z3::Solver,
    ptr_base: &'a BTreeMap<ValueId, ValueId>,
    /// Current SSA environment: value → its bitvector. Loop-carried values are
    /// overwritten each iteration; loop-invariant values persist.
    env: BTreeMap<ValueId, BV>,
    /// Per-object scalar memory cell: alloca base → current content bitvector.
    memory: BTreeMap<ValueId, BV>,
    /// Scalar-int nondet reads in execution order (bitvector + name).
    nondet: Vec<(BV, String)>,
    /// When `Some`, constraints are buffered (gated transient check) instead of
    /// asserted into the persistent solver.
    buffer: Option<Vec<Bool>>,
    /// Fresh-symbol counter (deterministic).
    counter: usize,
}

impl<'a> Enc<'a> {
    fn new(
        module: &'a AirModule,
        data_model: DataModel,
        solver: &'a z3::Solver,
        ptr_base: &'a BTreeMap<ValueId, ValueId>,
    ) -> Self {
        Self {
            module,
            data_model,
            solver,
            ptr_base,
            env: BTreeMap::new(),
            memory: BTreeMap::new(),
            nondet: Vec::new(),
            buffer: None,
            counter: 0,
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            env: self.env.clone(),
            memory: self.memory.clone(),
            nondet_len: self.nondet.len(),
        }
    }

    fn restore(&mut self, s: Snapshot) {
        self.env = s.env;
        self.memory = s.memory;
        self.nondet.truncate(s.nondet_len);
    }

    fn begin_buffer(&mut self) {
        self.buffer = Some(Vec::new());
    }

    fn end_buffer(&mut self) -> Vec<Bool> {
        self.buffer.take().unwrap_or_default()
    }

    /// Assert `c` — permanently into the solver, or into the transient buffer.
    fn assert_c(&mut self, c: Bool) {
        match &mut self.buffer {
            Some(buf) => buf.push(c),
            None => self.solver.assert(&c),
        }
    }

    fn fresh(&mut self, tag: &str) -> BV {
        let name = format!("{tag}{}", self.counter);
        self.counter += 1;
        BV::new_const(name, BV_WIDTH)
    }

    /// Encode operand `vid` to a bitvector, memoized in `env`.
    fn enc(&mut self, vid: ValueId) -> BV {
        if let Some(bv) = self.env.get(&vid) {
            return bv.clone();
        }
        if let Some(c) = self.module.constants.get(&vid) {
            if let Some(bv) = const_bv(c) {
                self.env.insert(vid, bv.clone());
                return bv;
            }
        }
        // Unknown leaf (param / global / undefined) → a stable fresh havoc symbol.
        let bv = self.fresh("h");
        self.env.insert(vid, bv.clone());
        bv
    }

    fn enc_bool(&mut self, vid: ValueId) -> Bool {
        let v = self.enc(vid);
        ssa_encode::truthy(&v)
    }

    /// Encode all non-terminator instructions of `block`. Phis are resolved from a
    /// snapshot of `env` (parallel-copy semantics) before any of them commit.
    fn encode_block_body(&mut self, block: &AirBlock, prev: Option<BlockId>) {
        let mut phi_writes: Vec<(ValueId, BV)> = Vec::new();
        for inst in &block.instructions {
            if let Operation::Phi { incoming } = &inst.op {
                if let Some(dst) = inst.dst {
                    let bv = self.encode_phi(incoming, prev);
                    phi_writes.push((dst, bv));
                }
            }
        }
        for (dst, bv) in phi_writes {
            self.env.insert(dst, bv);
        }
        for inst in &block.instructions {
            if inst.is_terminator() {
                break;
            }
            if matches!(inst.op, Operation::Phi { .. }) {
                continue;
            }
            self.define(inst);
        }
    }

    /// Assign the SSA definition produced by `inst`, or handle its side effect
    /// (`__VERIFIER_assume` path filter, `store` into the scalar memory model).
    /// Phis are handled ahead of this in [`Self::encode_block_body`].
    fn define(&mut self, inst: &Instruction) {
        if let Operation::Store = &inst.op {
            self.exec_store(inst);
            return;
        }
        let Some(dst) = inst.dst else {
            if let Operation::CallDirect { callee } = &inst.op {
                if self.module.function(*callee).map(|f| f.name.as_str())
                    == Some("__VERIFIER_assume")
                {
                    if let Some(&arg) = inst.operands.first() {
                        let c = self.enc_bool(arg);
                        self.assert_c(c);
                    }
                }
            }
            return;
        };
        let bv = self.encode_op(inst);
        self.env.insert(dst, bv);
    }

    fn encode_op(&mut self, inst: &Instruction) -> BV {
        match &inst.op {
            Operation::BinaryOp { kind } => {
                if inst.operands.len() < 2 {
                    return self.fresh("h");
                }
                let a = self.enc(inst.operands[0]);
                let b = self.enc(inst.operands[1]);
                let mut guards = Vec::new();
                let result = ssa_encode::encode_binop(*kind, &a, &b, &mut guards);
                for g in guards {
                    self.assert_c(g);
                }
                result.unwrap_or_else(|| self.fresh("h"))
            }
            Operation::Cast { .. } | Operation::Copy | Operation::Freeze => {
                match inst.operands.first() {
                    Some(&o) => self.enc(o),
                    None => self.fresh("h"),
                }
            }
            Operation::Select if inst.operands.len() >= 3 => {
                let c = self.enc_bool(inst.operands[0]);
                let a = self.enc(inst.operands[1]);
                let b = self.enc(inst.operands[2]);
                c.ite(&a, &b)
            }
            Operation::Load => self.exec_load(inst),
            Operation::Alloca { .. } | Operation::Gep { .. } => self.fresh("p"),
            Operation::CallDirect { callee } => {
                let name = self.module.function(*callee).map(|f| f.name.to_string());
                if let Some(name) = name {
                    if is_scalar_integer_nondet(&name) {
                        return self.fresh_nondet(&name);
                    }
                }
                self.fresh("h")
            }
            _ => self.fresh("h"),
        }
    }

    fn encode_phi(&mut self, incoming: &[(BlockId, ValueId)], prev: Option<BlockId>) -> BV {
        if let Some(prev) = prev {
            if let Some((_, v)) = incoming.iter().find(|(bb, _)| *bb == prev) {
                return self.enc(*v);
            }
        }
        self.fresh("h")
    }

    /// A fresh nondet input bitvector constrained to the declared type's range
    /// (R5), recorded in execution order for model read-back.
    fn fresh_nondet(&mut self, name: &str) -> BV {
        let bv = self.fresh("n");
        if let Some((lo, hi)) = ssa_encode::nondet_range(name, self.data_model) {
            let lo_c = bv.bvsge(BV::from_i64(lo, BV_WIDTH));
            let hi_c = bv.bvsle(BV::from_i64(hi, BV_WIDTH));
            self.assert_c(lo_c);
            self.assert_c(hi_c);
        }
        self.nondet.push((bv.clone(), name.to_string()));
        bv
    }

    /// Store into the memory cell of the pointer operand's root alloca (if
    /// resolvable); otherwise a no-op (the load will havoc — sound).
    fn exec_store(&mut self, inst: &Instruction) {
        if inst.operands.len() < 2 {
            return;
        }
        let (value, ptr) = (inst.operands[0], inst.operands[1]);
        let Some(&base) = self.ptr_base.get(&ptr) else {
            return;
        };
        let bv = self.enc(value);
        self.memory.insert(base, bv);
    }

    /// Load from the memory cell of the pointer operand's root alloca, or a fresh
    /// havoc symbol when the cell is unknown / the pointer is unresolvable.
    fn exec_load(&mut self, inst: &Instruction) -> BV {
        if let Some(&ptr) = inst.operands.first() {
            if let Some(&base) = self.ptr_base.get(&ptr) {
                if let Some(bv) = self.memory.get(&base) {
                    return bv.clone();
                }
            }
        }
        self.fresh("h")
    }

    /// Constrain the current block to leave toward `next`.
    fn force_branch(&mut self, term: &Instruction, next: BlockId) {
        match &term.op {
            Operation::CondBr {
                then_target,
                else_target,
            } => {
                let Some(&cond) = term.operands.first() else {
                    return;
                };
                let is_then = next == *then_target;
                let is_else = next == *else_target;
                if is_then && !is_else {
                    let c = self.enc_bool(cond);
                    self.assert_c(c);
                } else if is_else && !is_then {
                    let c = self.enc_bool(cond);
                    self.assert_c(c.not());
                }
            }
            Operation::Switch { default, cases } => {
                let Some(&disc_v) = term.operands.first() else {
                    return;
                };
                let disc = self.enc(disc_v);
                let matching: Vec<i64> = cases
                    .iter()
                    .filter(|(_, t)| *t == next)
                    .map(|(v, _)| *v)
                    .collect();
                if !matching.is_empty() {
                    let ors: Vec<Bool> = matching
                        .iter()
                        .map(|v| ssa_encode::eq(&disc, &BV::from_i64(*v, BV_WIDTH)))
                        .collect();
                    self.assert_c(Bool::or(&ors));
                } else if *default == next {
                    for (v, _) in cases {
                        let c = ssa_encode::eq(&disc, &BV::from_i64(*v, BV_WIDTH)).not();
                        self.assert_c(c);
                    }
                }
            }
            _ => {}
        }
    }

    /// Encode an exit path `header → … → error_block` under the current buffer:
    /// force the header terminator toward the path's first successor, then encode
    /// every subsequent block and force its terminator toward the next.
    fn encode_exit(
        &mut self,
        header_block: &AirBlock,
        exit_path: &[BlockId],
        error_block: BlockId,
        block_by_id: &BTreeMap<BlockId, &AirBlock>,
    ) {
        if exit_path.len() < 2 {
            // Header itself is the error block: reaching it after k iterations is
            // the violation; the accumulated header state is the whole condition.
            return;
        }
        if let Some(term) = header_block.terminator() {
            self.force_branch(term, exit_path[1]);
        }
        for j in 1..exit_path.len() {
            let Some(&block) = block_by_id.get(&exit_path[j]) else {
                return;
            };
            let prev = Some(exit_path[j - 1]);
            self.encode_block_body(block, prev);
            if exit_path[j] == error_block {
                break;
            }
            if j + 1 < exit_path.len() {
                if let Some(term) = block.terminator() {
                    self.force_branch(term, exit_path[j + 1]);
                }
            }
        }
    }
}

/// Constant → bitvector (integers / null / zero-init only).
fn const_bv(c: &Constant) -> Option<BV> {
    match c {
        Constant::Int { value, .. } => Some(BV::from_i64(*value, BV_WIDTH)),
        Constant::Null | Constant::ZeroInit => Some(BV::from_i64(0, BV_WIDTH)),
        _ => None,
    }
}

/// Map each pointer value that aliases an alloca (through `copy`/`cast`/`freeze`) to
/// that root alloca's `ValueId`, so scalar loads/stores hit the same memory cell.
/// Offsetting geps are excluded — an unresolved pointer just havocs (sound).
fn compute_ptr_base(func: &AirFunction) -> BTreeMap<ValueId, ValueId> {
    let mut base: BTreeMap<ValueId, ValueId> = BTreeMap::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let (Operation::Alloca { .. }, Some(dst)) = (&inst.op, inst.dst) {
                base.insert(dst, dst);
            }
        }
    }
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
    use saf_core::air::{BinaryOp, Instruction};
    use saf_core::ids::{InstId, ModuleId};

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

    /// Build `main` implementing, as mem2reg'd SSA (phi loop):
    /// ```c
    /// int x = __VERIFIER_nondet_int();
    /// int s = 0;
    /// for (int i = 0; i < bound; i++) s += x;   // loop-carried accumulator
    /// if (s == target) reach_error();
    /// ```
    /// `s == bound*x` after the loop, so a `target` that is a multiple of `bound`
    /// is SAT (`x = target/bound`); a non-multiple is UNSAT over the integers.
    /// A large `bound` (> the forward-SE visit cap of 8) is exactly the deep-loop
    /// class only incremental unwinding reaches.
    fn build_loop_module(bound: i64, target: i64) -> AirModule {
        let (main_id, nd_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, header, body, exit, err_bb, ret_bb) =
            (bid(10), bid(11), bid(12), bid(13), bid(14), bid(15));

        let x = vid(100);
        let zero = vid(101);
        let bnd = vid(102);
        let one = vid(103);
        let tgt = vid(104);
        let i_phi = vid(110);
        let s_phi = vid(111);
        let s_next = vid(112);
        let i_next = vid(113);
        let icmp_lt = vid(114);
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
            binop(5, BinaryOp::ICmpSlt, i_phi, bnd, icmp_lt),
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
            bnd,
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
        let module = build_loop_module(20, 60);
        let main = module.function_by_name("main").unwrap();
        assert!(
            incremental_gate(main, &module),
            "a loop with a nondet input must gate in"
        );
    }

    #[test]
    fn gate_rejects_acyclic() {
        // Straight-line `if (nondet()*3+7 == C)` — no loop; the fixed-k base case
        // handles it, so the incremental engine must NOT fire.
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
        assert!(
            !incremental_gate(f, &module),
            "an acyclic function must NOT gate in"
        );
    }

    #[test]
    fn solves_deep_loop_accumulator() {
        // 20 iterations (> the SE visit cap of 8, and > the fixed-k base of 1):
        // s == 20*x == 60 → x == 3. Only incremental unwinding reaches depth 20.
        let module = build_loop_module(20, 60);
        let config = PropertyAnalysisConfig::default();
        let cands = enumerate_incremental_candidates(&module, &config, DataModel::LP64);
        assert_eq!(
            cands.len(),
            1,
            "expected one incremental candidate for s==60"
        );
        let seq = &cands[0].nondet_sequence;
        assert_eq!(seq.len(), 1, "one nondet input read before the loop");
        assert_eq!(seq[0].func_name, "__VERIFIER_nondet_int");
        assert_eq!(seq[0].value, 3, "20*x == 60 → x == 3");
    }

    #[test]
    fn unsat_target_yields_no_candidate() {
        // s == 20*x can never equal 61 (not a multiple of 20) → no candidate.
        let module = build_loop_module(20, 61);
        let config = PropertyAnalysisConfig::default();
        let cands = enumerate_incremental_candidates(&module, &config, DataModel::LP64);
        assert!(
            cands.is_empty(),
            "20*x == 61 has no integer solution → no candidate"
        );
    }

    #[test]
    fn candidate_is_deterministic() {
        let module = build_loop_module(20, 60);
        let config = PropertyAnalysisConfig::default();
        let a = enumerate_incremental_candidates(&module, &config, DataModel::LP64);
        let b = enumerate_incremental_candidates(&module, &config, DataModel::LP64);
        assert_eq!(a.len(), b.len());
        assert_eq!(a[0].nondet_sequence, b[0].nondet_sequence);
    }
}
