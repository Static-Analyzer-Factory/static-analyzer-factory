//! Fixed-k Bounded Model Checking (BMC) base-case candidate enumeration for the
//! `unreach-call` property.
//!
//! # What this adds over the Z3 *path-guard* engine
//!
//! [`crate::property::enumerate_false_candidates`] asks Z3 whether a `reach_error`
//! site is reachable by extracting the branch **guards** along a CFG path and
//! treating every compared operand as a *fresh, unconstrained* variable
//! (`saf_analysis::guard::OperandInfo::Value`). That loses the arithmetic that
//! *defines* the operand: for
//!
//! ```c
//! int x = __VERIFIER_nondet_int();
//! int y = x * 3 + 7;
//! if (y == 100) reach_error();
//! ```
//!
//! the guard engine models `y` as free, sets `y = 100`, and stops — but the
//! replay driver pins the nondet **input** `x`, not `y`, so `x = 0 → y = 7 ≠ 100`
//! and the run never reaches the error (→ `unknown`). The input is a *preimage* of
//! the compared value, which a blind/CmpLog fuzzer also cannot invert.
//!
//! The BMC engine instead symbolically executes a bounded (loop-unwound to the
//! acyclic base case, `k = 1`) path from the error site's function entry to the
//! error block, encoding **every** SSA definition on the path as a machine-width
//! bitvector — arithmetic, casts, phis, selects, comparisons — so the guard
//! becomes a constraint over the actual `__VERIFIER_nondet_*` inputs. On SAT it
//! reads the model in nondet-call order to produce the concrete input vector.
//!
//! # Soundness
//!
//! The engine is UNSOUND on its own (bitvector encoding is only an approximation:
//! 64-bit-uniform arithmetic, identity casts, havoc for memory/unknown calls). It
//! emits only [`FalseCandidate`]s, which the caller feeds through the SAME native
//! concrete-replay gate as the other stages — a spurious or imprecise model can
//! only ever fail to reproduce → `unknown`. Never a verdict from a solver model
//! alone (confirmer contract R6).
//!
//! # Cost
//!
//! Fires only when a function actually has a nondet input flowing through at least
//! one *arithmetic* operation into a branch guard (the precise gate
//! [`nondet_flows_through_arith_to_guard`]) — the case the cheaper guard engine
//! cannot already crack. Path count, guard depth, sites, and the Z3 timeout are
//! all hard-capped so a gated task adds only a bounded solver budget.

use std::collections::{BTreeMap, BTreeSet};

use saf_analysis::z3_utils::reachability::block_paths_between;
use saf_core::air::{AirFunction, AirModule, BinaryOp, Constant, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};
use z3::ast::{BV, Bool};

use crate::property::{
    FalseCandidate, PropertyAnalysisConfig, is_scalar_integer_nondet, resolve_nondet_sequence,
};
use crate::property_kind::DataModel;
use crate::ssa_encode::{self, BV_WIDTH};

/// Max acyclic (k = 1 unwound) paths explored per error site.
const BMC_MAX_PATHS: usize = 8;

/// Per-solve Z3 timeout (ms). Kept small: BMC is a recall bonus, not a proof.
/// Base-case QF_BV solves are typically sub-50 ms; the cap only bounds
/// pathological inputs so a gated task cannot blow the task budget.
const BMC_Z3_TIMEOUT_MS: u32 = 1000;

/// Deterministic Z3 rlimit and seed (determinism: NFR-DET).
const BMC_Z3_RLIMIT: u32 = 4_000_000;
const BMC_Z3_SEED: u32 = 0;

/// Max error sites encoded per module (bounds cost on pathological inputs).
const BMC_MAX_SITES: usize = 3;

/// Max def-chain recursion depth for the gate / encoder (bounds cost).
const MAX_DEPTH: usize = 128;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Enumerate BMC (fixed-k base case) FALSE *candidates* for `unreach-call`.
///
/// For each `reach_error` / `__VERIFIER_error` site whose function has a nondet
/// input flowing through arithmetic into a guard (the gate the cheap guard engine
/// cannot handle), symbolically executes bounded acyclic paths to the error block
/// and, on the first SAT path, emits a [`FalseCandidate`] whose `nondet_sequence`
/// is the concrete input vector read from the model. Candidates MUST be confirmed
/// by native replay before any verdict (see module docs).
#[must_use]
pub fn enumerate_bmc_candidates(
    module: &AirModule,
    config: &PropertyAnalysisConfig,
    data_model: DataModel,
) -> Vec<FalseCandidate> {
    let mut candidates = Vec::new();
    let mut sites_done = 0usize;

    for (func_id, error_block, error_inst) in error_call_sites(module) {
        if sites_done >= BMC_MAX_SITES {
            break;
        }
        let Some(func) = module.function(func_id) else {
            continue;
        };
        if func.is_declaration {
            continue;
        }
        // Gate: only pay the solver cost when arithmetic sits between a nondet
        // input and a branch guard — the class the guard engine misses.
        if !nondet_flows_through_arith_to_guard(func, module) {
            continue;
        }
        let Some(entry) = func
            .entry_block
            .or_else(|| func.blocks.first().map(|b| b.id))
        else {
            continue;
        };
        sites_done += 1;

        let paths = block_paths_between(entry, error_block, func_id, module, BMC_MAX_PATHS);
        for path in &paths {
            if let Some(assignments) = solve_path(func, module, path, data_model) {
                let nondet_sequence = resolve_nondet_sequence(module, func_id, path, &assignments);
                candidates.push(FalseCandidate {
                    reach_error_inst: error_inst,
                    block_path: path.clone(),
                    assignments,
                    nondet_sequence,
                });
                // One candidate per site: the first feasible path is enough to
                // seed a replay; further paths only add cost.
                break;
            }
        }
    }

    // Incremental unwinding for cyclic error-functions: the acyclic base case
    // above only reaches `k = 1`, so a violation gated behind several loop
    // iterations (`for(i=0;i<20;i++) s+=x; if(s==60) reach_error();`) is missed.
    // The incremental engine grows the unwinding bound in ONE persistent Z3
    // context, gating each depth's reach_error check with a `check-sat-assuming`
    // activation literal, so it reaches an order of magnitude deeper per solver
    // budget. Its candidates go through the SAME native-replay gate.
    candidates.extend(crate::bmc_incremental::enumerate_incremental_candidates(
        module, config, data_model,
    ));

    candidates
}

/// The `reach_error` / `__VERIFIER_error` call sites as `(func, block, inst)`.
fn error_call_sites(module: &AirModule) -> Vec<(FunctionId, BlockId, InstId)> {
    const NAMES: &[&str] = &["reach_error", "__VERIFIER_error"];
    let mut sites = Vec::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if NAMES.contains(&target.name.as_str()) {
                            sites.push((func.id, block.id, inst.id));
                        }
                    }
                }
            }
        }
    }
    sites
}

// ---------------------------------------------------------------------------
// Gate: nondet → arithmetic → guard
// ---------------------------------------------------------------------------

/// Is `kind` an integer *arithmetic/bitwise* op (not a comparison, not float)?
fn is_arith_binop(kind: BinaryOp) -> bool {
    matches!(
        kind,
        BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::UDiv
            | BinaryOp::SDiv
            | BinaryOp::URem
            | BinaryOp::SRem
            | BinaryOp::And
            | BinaryOp::Or
            | BinaryOp::Xor
            | BinaryOp::Shl
            | BinaryOp::LShr
            | BinaryOp::AShr
    )
}

/// Index `dst ValueId → defining Instruction` and the set of scalar-int nondet
/// call result values, for `func`.
fn build_indexes<'a>(
    func: &'a AirFunction,
    module: &AirModule,
) -> (
    BTreeMap<ValueId, &'a saf_core::air::Instruction>,
    BTreeSet<ValueId>,
) {
    let mut def_map = BTreeMap::new();
    let mut nondet_dsts = BTreeSet::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                def_map.insert(dst, inst);
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(name) = module.function(*callee).map(|f| f.name.as_str()) {
                        if is_scalar_integer_nondet(name) {
                            nondet_dsts.insert(dst);
                        }
                    }
                }
            }
        }
    }
    (def_map, nondet_dsts)
}

/// True iff some branch-guard comparison operand in `func` transitively depends
/// on a scalar-int nondet result **through at least one arithmetic operation** —
/// exactly the shape the leaf-variable guard engine cannot invert.
fn nondet_flows_through_arith_to_guard(func: &AirFunction, module: &AirModule) -> bool {
    let (def_map, nondet_dsts) = build_indexes(func, module);
    if nondet_dsts.is_empty() {
        return false;
    }

    // Condition values consumed by conditional terminators.
    let mut guard_conds = BTreeSet::new();
    for block in &func.blocks {
        if let Some(term) = block.terminator() {
            match &term.op {
                Operation::CondBr { .. } | Operation::Switch { .. } => {
                    if let Some(&c) = term.operands.first() {
                        guard_conds.insert(c);
                    }
                }
                _ => {}
            }
        }
    }
    if guard_conds.is_empty() {
        return false;
    }

    let mut memo_plain: BTreeMap<ValueId, bool> = BTreeMap::new();
    let mut memo_arith: BTreeMap<ValueId, bool> = BTreeMap::new();
    for cond in &guard_conds {
        if arith_between(
            *cond,
            &def_map,
            &nondet_dsts,
            &mut memo_plain,
            &mut memo_arith,
            0,
        ) {
            return true;
        }
    }
    false
}

/// Does `vid` (plainly) depend on any scalar-int nondet result?
fn plain_dep(
    vid: ValueId,
    def_map: &BTreeMap<ValueId, &saf_core::air::Instruction>,
    nondet_dsts: &BTreeSet<ValueId>,
    memo: &mut BTreeMap<ValueId, bool>,
    depth: usize,
) -> bool {
    if nondet_dsts.contains(&vid) {
        return true;
    }
    if let Some(&v) = memo.get(&vid) {
        return v;
    }
    if depth >= MAX_DEPTH {
        return false;
    }
    let result = def_map.get(&vid).is_some_and(|inst| {
        inst.operands
            .iter()
            .any(|&o| plain_dep(o, def_map, nondet_dsts, memo, depth + 1))
    });
    memo.insert(vid, result);
    result
}

/// Does the def-DAG rooted at `vid` reach a scalar-int nondet result along a path
/// that passes through at least one arithmetic op?
fn arith_between(
    vid: ValueId,
    def_map: &BTreeMap<ValueId, &saf_core::air::Instruction>,
    nondet_dsts: &BTreeSet<ValueId>,
    memo_plain: &mut BTreeMap<ValueId, bool>,
    memo_arith: &mut BTreeMap<ValueId, bool>,
    depth: usize,
) -> bool {
    if let Some(&v) = memo_arith.get(&vid) {
        return v;
    }
    if depth >= MAX_DEPTH {
        return false;
    }
    let Some(inst) = def_map.get(&vid) else {
        return false;
    };
    let result = if let Operation::BinaryOp { kind } = &inst.op {
        if is_arith_binop(*kind) {
            // This node is arithmetic: a nondet-dependent operand satisfies the
            // "through ≥1 arithmetic op" requirement immediately.
            inst.operands
                .iter()
                .any(|&o| plain_dep(o, def_map, nondet_dsts, memo_plain, depth + 1))
                || inst.operands.iter().any(|&o| {
                    arith_between(o, def_map, nondet_dsts, memo_plain, memo_arith, depth + 1)
                })
        } else {
            // Comparison: recurse (the arithmetic must be deeper).
            inst.operands
                .iter()
                .any(|&o| arith_between(o, def_map, nondet_dsts, memo_plain, memo_arith, depth + 1))
        }
    } else {
        // Cast/Copy/Phi/Select/… — pass through, arithmetic must be deeper.
        inst.operands
            .iter()
            .any(|&o| arith_between(o, def_map, nondet_dsts, memo_plain, memo_arith, depth + 1))
    };
    memo_arith.insert(vid, result);
    result
}

// ---------------------------------------------------------------------------
// Path encoder + solver
// ---------------------------------------------------------------------------

/// Symbolically execute the acyclic block `path` (entry → error block) of `func`,
/// forcing each branch to follow the path and encoding every SSA definition as a
/// 64-bit bitvector. Returns `Some(model)` mapping each scalar-int nondet result
/// `ValueId` to its concrete in-range value when the path is SAT; `None`
/// otherwise (UNSAT / unknown / no model).
fn solve_path(
    func: &AirFunction,
    module: &AirModule,
    path: &[BlockId],
    data_model: DataModel,
) -> Option<BTreeMap<ValueId, i64>> {
    let solver = new_solver();
    let mut enc = Encoder::new(module, data_model, &solver);

    for (i, &block_id) in path.iter().enumerate() {
        let block = func.blocks.iter().find(|b| b.id == block_id)?;
        let prev = if i == 0 { None } else { Some(path[i - 1]) };
        for inst in &block.instructions {
            if inst.is_terminator() {
                if i + 1 < path.len() {
                    enc.force_branch(inst, path[i + 1]);
                }
                break;
            }
            enc.define(inst, prev);
        }
    }

    match solver.check() {
        z3::SatResult::Sat => {
            let model = solver.get_model()?;
            let mut assignments = BTreeMap::new();
            for (vid, bv) in &enc.nondet_bvs {
                if let Some(v) = model.eval(bv, true).and_then(|b| b.as_i64()) {
                    assignments.insert(*vid, v);
                }
            }
            Some(assignments)
        }
        z3::SatResult::Unsat | z3::SatResult::Unknown => None,
    }
}

/// Fresh deterministic Z3 solver for a single path query.
fn new_solver() -> z3::Solver {
    let solver = z3::Solver::new();
    let mut params = z3::Params::new();
    params.set_u32("timeout", BMC_Z3_TIMEOUT_MS);
    params.set_u32("rlimit", BMC_Z3_RLIMIT);
    params.set_u32("random_seed", BMC_Z3_SEED);
    solver.set_params(&params);
    solver
}

/// SSA→bitvector encoder over one concrete acyclic path.
struct Encoder<'a> {
    module: &'a AirModule,
    data_model: DataModel,
    solver: &'a z3::Solver,
    /// `ValueId` → its 64-bit bitvector encoding.
    cache: BTreeMap<ValueId, BV>,
    /// Scalar-int nondet result `ValueId` → its (range-constrained) bitvector, so
    /// the model can be read back in nondet order.
    nondet_bvs: BTreeMap<ValueId, BV>,
    /// Counter for fresh havoc/nondet symbol names (deterministic).
    counter: usize,
}

impl<'a> Encoder<'a> {
    fn new(module: &'a AirModule, data_model: DataModel, solver: &'a z3::Solver) -> Self {
        Self {
            module,
            data_model,
            solver,
            cache: BTreeMap::new(),
            nondet_bvs: BTreeMap::new(),
            counter: 0,
        }
    }

    fn fresh(&mut self, tag: &str) -> BV {
        let name = format!("{tag}{}", self.counter);
        self.counter += 1;
        BV::new_const(name, BV_WIDTH)
    }

    fn zero() -> BV {
        BV::from_i64(0, BV_WIDTH)
    }

    /// Encode operand `vid` to a 64-bit bitvector, memoized.
    fn enc(&mut self, vid: ValueId) -> BV {
        if let Some(bv) = self.cache.get(&vid) {
            return bv.clone();
        }
        // Constant?
        if let Some(c) = self.module.constants.get(&vid) {
            let bv = match c {
                Constant::Int { value, .. } => Some(BV::from_i64(*value, BV_WIDTH)),
                Constant::Null | Constant::ZeroInit => Some(Self::zero()),
                _ => None,
            };
            if let Some(bv) = bv {
                self.cache.insert(vid, bv.clone());
                return bv;
            }
        }
        // Unwalked def (param / global / load / unknown) → havoc.
        let bv = self.fresh("h");
        self.cache.insert(vid, bv.clone());
        bv
    }

    /// Encode a value operand as a Z3 boolean (`value != 0`).
    fn enc_bool(&mut self, vid: ValueId) -> Bool {
        let v = self.enc(vid);
        ssa_encode::truthy(&v)
    }

    /// Assign the SSA definition produced by `inst` (a non-terminator).
    fn define(&mut self, inst: &saf_core::air::Instruction, prev: Option<BlockId>) {
        // Side-effecting call with no dst: honour __VERIFIER_assume as a hard
        // path filter (confirmer contract R4).
        let Some(dst) = inst.dst else {
            if let Operation::CallDirect { callee } = &inst.op {
                if self.module.function(*callee).map(|f| f.name.as_str())
                    == Some("__VERIFIER_assume")
                {
                    if let Some(&arg) = inst.operands.first() {
                        let c = self.enc_bool(arg);
                        self.solver.assert(&c);
                    }
                }
            }
            return;
        };

        let bv = self.encode_op(inst, prev);
        self.cache.insert(dst, bv);
    }

    fn encode_op(&mut self, inst: &saf_core::air::Instruction, prev: Option<BlockId>) -> BV {
        match &inst.op {
            Operation::BinaryOp { kind } => self.encode_binop(*kind, &inst.operands),
            // Casts are identity in the 64-bit-uniform model: the scalar-int
            // nondet inputs are range-constrained to their declared width, so a
            // narrowing/widening cast of an in-range value is the identity. An
            // out-of-range value only makes the model fail to replay (sound).
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
            Operation::Phi { incoming } => self.encode_phi(incoming, prev),
            Operation::CallDirect { callee } => {
                let name = self.module.function(*callee).map(|f| f.name.to_string());
                if let Some(name) = name {
                    if is_scalar_integer_nondet(&name) {
                        return self.fresh_nondet(inst.dst, &name);
                    }
                }
                self.fresh("h")
            }
            _ => self.fresh("h"),
        }
    }

    /// A fresh nondet input bitvector constrained to the declared type's range
    /// (confirmer contract R5), recorded for model read-back.
    fn fresh_nondet(&mut self, dst: Option<ValueId>, name: &str) -> BV {
        let bv = self.fresh("n");
        if let Some((lo, hi)) = ssa_encode::nondet_range(name, self.data_model) {
            self.solver.assert(bv.bvsge(BV::from_i64(lo, BV_WIDTH)));
            self.solver.assert(bv.bvsle(BV::from_i64(hi, BV_WIDTH)));
        }
        if let Some(dst) = dst {
            self.nondet_bvs.insert(dst, bv.clone());
        }
        bv
    }

    fn encode_phi(&mut self, incoming: &[(BlockId, ValueId)], prev: Option<BlockId>) -> BV {
        if let Some(prev) = prev {
            if let Some((_, v)) = incoming.iter().find(|(b, _)| *b == prev) {
                return self.enc(*v);
            }
        }
        self.fresh("h")
    }

    fn encode_binop(&mut self, kind: BinaryOp, operands: &[ValueId]) -> BV {
        if operands.len() < 2 {
            return self.fresh("h");
        }
        let a = self.enc(operands[0]);
        let b = self.enc(operands[1]);
        let mut guards = Vec::new();
        let result = ssa_encode::encode_binop(kind, &a, &b, &mut guards);
        for g in &guards {
            self.solver.assert(g);
        }
        result.unwrap_or_else(|| self.fresh("h"))
    }

    /// Constrain the path to leave `inst`'s block toward `next`.
    fn force_branch(&mut self, inst: &saf_core::air::Instruction, next: BlockId) {
        match &inst.op {
            Operation::CondBr {
                then_target,
                else_target,
            } => {
                let Some(&cond) = inst.operands.first() else {
                    return;
                };
                let is_then = next == *then_target;
                let is_else = next == *else_target;
                // Ambiguous (both edges to `next`, or neither) → no constraint.
                if is_then && !is_else {
                    let c = self.enc_bool(cond);
                    self.solver.assert(&c);
                } else if is_else && !is_then {
                    let c = self.enc_bool(cond);
                    self.solver.assert(c.not());
                }
            }
            Operation::Switch { default, cases } => {
                let Some(&disc_v) = inst.operands.first() else {
                    return;
                };
                let disc = self.enc(disc_v);
                let matching: Vec<i64> = cases
                    .iter()
                    .filter(|(_, t)| *t == next)
                    .map(|(v, _)| *v)
                    .collect();
                if !matching.is_empty() {
                    #[allow(deprecated)]
                    let ors: Vec<Bool> = matching
                        .iter()
                        .map(|v| disc._eq(BV::from_i64(*v, BV_WIDTH)))
                        .collect();
                    self.solver.assert(Bool::or(&ors));
                } else if *default == next {
                    for (v, _) in cases {
                        #[allow(deprecated)]
                        self.solver
                            .assert(disc._eq(BV::from_i64(*v, BV_WIDTH)).not());
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, CastKind, Instruction};
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

    /// Builds `main`:
    /// ```c
    /// int x = __VERIFIER_nondet_int();
    /// int y = x * 3 + 7;           // arithmetic between input and guard
    /// if (y == 100) reach_error(); // (unsat over int: 93 % 3 != 0)
    /// ```
    /// with a chosen `target` the guard compares against, so tests can pick a
    /// SAT (`target` reachable) or UNSAT constant.
    fn build_module(target: i64) -> (AirModule, FunctionId, BlockId, InstId) {
        let (main_id, nd_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, err_bb, exit_bb) = (bid(10), bid(11), bid(12));

        let x = vid(100);
        let three = vid(101);
        let mul = vid(102);
        let seven = vid(103);
        let add = vid(104);
        let tgt = vid(105);
        let cmp = vid(106);

        let insts = vec![
            Instruction::new(iid(1), Operation::CallDirect { callee: nd_id }).with_dst(x),
            binop(2, BinaryOp::Mul, x, three, mul),
            binop(3, BinaryOp::Add, mul, seven, add),
            binop(4, BinaryOp::ICmpEq, add, tgt, cmp),
            Instruction::new(
                iid(5),
                Operation::CondBr {
                    then_target: err_bb,
                    else_target: exit_bb,
                },
            )
            .with_operands(vec![cmp]),
        ];

        let err_call = Instruction::new(iid(6), Operation::CallDirect { callee: err_id });
        let err_block = blk(
            err_bb,
            vec![err_call, Instruction::new(iid(7), Operation::Ret)],
        );
        let exit_block = blk(exit_bb, vec![Instruction::new(iid(8), Operation::Ret)]);

        let main = func(
            main_id,
            "main",
            vec![blk(entry, insts), err_block, exit_block],
            entry,
        );

        let mut module = AirModule::new(ModuleId::new(1));
        module
            .constants
            .insert(three, Constant::Int { value: 3, bits: 32 });
        module
            .constants
            .insert(seven, Constant::Int { value: 7, bits: 32 });
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
        (module, main_id, err_bb, iid(6))
    }

    #[test]
    fn gate_fires_on_arith_between_nondet_and_guard() {
        let (module, main_id, _, _) = build_module(103);
        let func = module.function(main_id).unwrap();
        assert!(
            nondet_flows_through_arith_to_guard(func, &module),
            "x*3+7 feeding a guard must trip the arithmetic gate"
        );
    }

    #[test]
    fn gate_skips_direct_nondet_guard() {
        // `if (nondet() == C)` — no arithmetic between input and guard; the cheap
        // guard engine already handles it, so BMC must NOT fire.
        let (main_id, nd_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, err_bb, exit_bb) = (bid(10), bid(11), bid(12));
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
                    else_target: exit_bb,
                },
            )
            .with_operands(vec![cmp]),
        ];
        let main = func(
            main_id,
            "main",
            vec![
                blk(entry, insts),
                blk(err_bb, vec![Instruction::new(iid(4), Operation::Ret)]),
                blk(exit_bb, vec![Instruction::new(iid(5), Operation::Ret)]),
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
        let f = module.function(main_id).unwrap();
        assert!(
            !nondet_flows_through_arith_to_guard(f, &module),
            "a direct nondet==C guard must NOT trip the arithmetic gate"
        );
    }

    #[test]
    fn solves_arithmetic_guard_for_input_preimage() {
        // target 103 → x*3+7 == 103 → x == 32 (SAT).
        let (module, _, _, _) = build_module(103);
        let config = PropertyAnalysisConfig::default();
        let cands = enumerate_bmc_candidates(&module, &config, DataModel::LP64);
        assert_eq!(cands.len(), 1, "expected one BMC candidate");
        let seq = &cands[0].nondet_sequence;
        assert_eq!(seq.len(), 1, "one nondet input on the path");
        assert_eq!(seq[0].func_name, "__VERIFIER_nondet_int");
        assert_eq!(seq[0].value, 32, "x must be the preimage 32 (32*3+7=103)");
    }

    #[test]
    fn unsat_guard_yields_no_candidate() {
        // target 100 → x*3+7 == 100 → 3x == 93 → x == 31 (SAT actually: 31*3+7=100).
        // Use 101 which is 3x == 94 → not divisible → UNSAT over the integers.
        let (module, _, _, _) = build_module(101);
        let config = PropertyAnalysisConfig::default();
        let cands = enumerate_bmc_candidates(&module, &config, DataModel::LP64);
        assert!(
            cands.is_empty(),
            "x*3+7 == 101 has no integer solution → no candidate"
        );
    }

    #[test]
    fn candidate_is_deterministic() {
        let (module, _, _, _) = build_module(103);
        let config = PropertyAnalysisConfig::default();
        let a = enumerate_bmc_candidates(&module, &config, DataModel::LP64);
        let b = enumerate_bmc_candidates(&module, &config, DataModel::LP64);
        assert_eq!(a.len(), b.len());
        assert_eq!(a[0].nondet_sequence, b[0].nondet_sequence);
    }

    #[test]
    fn cast_is_transparent_in_encoding() {
        // Ensure a cast on the arithmetic result does not break the gate.
        let (main_id, nd_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, err_bb, exit_bb) = (bid(10), bid(11), bid(12));
        let x = vid(100);
        let two = vid(101);
        let mul = vid(102);
        let cast = vid(103);
        let c = vid(104);
        let cmp = vid(105);
        let insts = vec![
            Instruction::new(iid(1), Operation::CallDirect { callee: nd_id }).with_dst(x),
            binop(2, BinaryOp::Mul, x, two, mul),
            Instruction::new(
                iid(3),
                Operation::Cast {
                    kind: CastKind::SExt,
                    target_bits: Some(64),
                },
            )
            .with_operands(vec![mul])
            .with_dst(cast),
            binop(4, BinaryOp::ICmpEq, cast, c, cmp),
            Instruction::new(
                iid(5),
                Operation::CondBr {
                    then_target: err_bb,
                    else_target: exit_bb,
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
                        Instruction::new(iid(6), Operation::CallDirect { callee: err_id }),
                        Instruction::new(iid(7), Operation::Ret),
                    ],
                ),
                blk(exit_bb, vec![Instruction::new(iid(8), Operation::Ret)]),
            ],
            entry,
        );
        let mut module = AirModule::new(ModuleId::new(1));
        module
            .constants
            .insert(two, Constant::Int { value: 2, bits: 32 });
        module.constants.insert(
            c,
            Constant::Int {
                value: 84,
                bits: 32,
            },
        );
        module.functions.push(main);
        module.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));
        module.functions.push(decl(err_id, "reach_error"));
        let config = PropertyAnalysisConfig::default();
        let cands = enumerate_bmc_candidates(&module, &config, DataModel::LP64);
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0].nondet_sequence[0].value, 42, "x*2 == 84 → x == 42");
    }
}
