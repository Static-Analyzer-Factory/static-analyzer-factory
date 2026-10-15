//! `no-overflow` TRUE for programs that spawn threads (plan 222 §2z).
//!
//! The exhaustive engine and the interval sentinel are sequential: under
//! interleavings a load of a shared global may return a value another thread
//! stored at any time. This prover is sound for every interleaving because it
//! never relies on the order of events:
//!
//! 1. **Execution bounds.** Every function gets a bound on how often it runs in
//!    the whole program run, summed over all threads: `main` runs once, a call
//!    site or `pthread_create` site outside any CFG cycle adds the caller's
//!    count once, one inside a cycle makes the callee unbounded, recursion is
//!    unbounded. A function whose address is used anywhere but as the start
//!    routine of `pthread_create` fails the proof (it could run from anywhere).
//! 2. **Global ranges.** A scalar integer global whose address is used only as
//!    the pointer of plain loads and stores of its own width ("tracked") holds,
//!    at any time, its initial value, a value some store wrote, or such a value
//!    plus increments: a store of `load(g) + c` (an "increment") adds `c` to SOME
//!    value `g` held, so along any chain of increments each execution counts
//!    once and every value `g` ever holds lies in
//!    `[lo − Σ neg, hi + Σ pos]` over the bounded increment executions, where
//!    `[lo, hi]` covers the initial value and every non-increment store.
//!    Interleavings only choose WHICH earlier value an increment read, so the
//!    bound holds for all of them (lost updates only shrink it). Any other
//!    global is `⊤` when loaded.
//! 3. **Value ranges.** Per function, an interval fixpoint over the SSA values
//!    (phis joined, values that keep changing widened to their type's range),
//!    with tracked loads at their global's range and every other load, call
//!    result and parameter at `⊤`.
//! 4. **Obligations.** Every signed (`nsw`) add/sub/mul, `sdiv`/`srem`, signed
//!    `shl` and `abs` call in a function that may run must stay in range.
//!
//! Atomic read-modify-writes (`Instruction::is_atomic_pair`) make their target
//! untracked. Anything not understood fails the proof (`Abstain`).

// NOTE: interval arithmetic reads best with its textbook names (`a`, `b` the
// operands, `f` the function, `i` the instruction, `r` the range map).
#![allow(clippy::many_single_char_names)]

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{
    AirFunction, AirModule, AirType, BinaryOp, CastKind, Constant, Instruction, Operation,
};
use saf_core::ids::{BlockId, FunctionId, ValueId};

/// Result of [`prove_no_overflow_concurrent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcOverflowProof {
    /// No signed overflow in any interleaving.
    Proven,
    /// Not proven (the reason names the first obligation or feature that failed).
    Abstain(String),
}

/// "Runs without bound" in an execution count.
const UNBOUNDED: u64 = u64::MAX;
/// Counts beyond this are treated as unbounded (keeps the range sums small).
const COUNT_CAP: u64 = 1 << 20;
/// Interval iterations per function before a still-changing value is widened.
const WIDEN_AFTER: u32 = 3;
/// Engine budget per pure-leaf function, seconds.
const LEAF_BUDGET_S: u64 = 10;
/// Outer iterations over the global ranges before widening them.
const GLOBAL_ROUNDS: u32 = 12;

/// A signed interval of `bits`-wide values (`lo <= hi`), or the empty set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Iv {
    lo: i128,
    hi: i128,
}

fn smin(bits: u32) -> i128 {
    -(1i128 << (bits - 1))
}
fn smax(bits: u32) -> i128 {
    (1i128 << (bits - 1)) - 1
}
fn top(bits: u32) -> Iv {
    Iv {
        lo: smin(bits),
        hi: smax(bits),
    }
}
fn point(v: i128) -> Iv {
    Iv { lo: v, hi: v }
}
fn join(a: Option<Iv>, b: Iv) -> Iv {
    match a {
        None => b,
        Some(a) => Iv {
            lo: a.lo.min(b.lo),
            hi: a.hi.max(b.hi),
        },
    }
}
/// `v` as the signed value of its low `bits` bits.
fn wrap(v: i128, bits: u32) -> i128 {
    if bits >= 128 {
        return v;
    }
    let m = (1i128 << bits) - 1;
    let x = v & m;
    if x >> (bits - 1) & 1 == 1 {
        x - (1i128 << bits)
    } else {
        x
    }
}
fn fits(iv: Iv, bits: u32) -> bool {
    iv.lo >= smin(bits) && iv.hi <= smax(bits)
}

/// Prove that no signed overflow happens in any interleaving (see the module
/// documentation). `signed_shl_locs`: source locations of the shifts clang
/// treats as signed (`None`: every `shl` is signed).
#[must_use]
pub fn prove_no_overflow_concurrent(
    module: &AirModule,
    signed_shl_locs: Option<&BTreeSet<(u32, u32)>>,
) -> ConcOverflowProof {
    match Prover::new(module, signed_shl_locs).prove() {
        Ok(()) => ConcOverflowProof::Proven,
        Err(e) => ConcOverflowProof::Abstain(e),
    }
}

type Res<T> = Result<T, String>;

struct Prover<'a> {
    module: &'a AirModule,
    signed_shl_locs: Option<&'a BTreeSet<(u32, u32)>>,
    /// Defined functions by id.
    funcs: BTreeMap<FunctionId, &'a AirFunction>,
    /// Execution bound per defined function (0: never runs).
    count: BTreeMap<FunctionId, u64>,
    /// How often a block may run per invocation of its function, for blocks
    /// on a CFG cycle (absent: once; see [`block_multiplicity`]).
    mult: BTreeMap<FunctionId, BTreeMap<BlockId, u64>>,
    /// Tracked globals: id → bit width.
    tracked: BTreeMap<ValueId, u32>,
    /// Current range of each tracked global.
    grange: BTreeMap<ValueId, Iv>,
    /// Per function, the tracked globals only it stores to, when it runs at
    /// most once: inside it, two reads with no store of its own between them
    /// read the same value (nobody else writes), see [`load_aliases`].
    single_writer: BTreeMap<FunctionId, BTreeSet<ValueId>>,
}

impl<'a> Prover<'a> {
    fn new(module: &'a AirModule, signed_shl_locs: Option<&'a BTreeSet<(u32, u32)>>) -> Self {
        let funcs: BTreeMap<FunctionId, &AirFunction> = module
            .functions
            .iter()
            .filter(|f| !f.is_declaration && !f.blocks.is_empty())
            .map(|f| (f.id, f))
            .collect();
        let mut mult = BTreeMap::new();
        for f in funcs.values() {
            mult.insert(f.id, block_multiplicity(module, f));
        }
        Self {
            module,
            signed_shl_locs,
            funcs,
            count: BTreeMap::new(),
            mult,
            tracked: BTreeMap::new(),
            grange: BTreeMap::new(),
            single_writer: BTreeMap::new(),
        }
    }

    fn prove(&mut self) -> Res<()> {
        self.check_function_addresses()?;
        self.compute_counts()?;
        self.compute_tracked();
        self.compute_single_writer();
        // Global ranges to a fixpoint (each round re-evaluates every function
        // against the current global ranges).
        for g in self.tracked.keys() {
            let init = self.global_init(*g);
            self.grange.insert(*g, init);
        }
        let mut rounds = 0;
        loop {
            rounds += 1;
            let ranges = self.all_function_ranges(false)?;
            let next = self.next_global_ranges(&ranges);
            let mut changed = false;
            for (g, r) in next {
                let bits = self.tracked[&g];
                let old = self.grange[&g];
                let mut new = join(Some(old), r);
                if rounds > GLOBAL_ROUNDS && new != old {
                    new = top(bits);
                }
                if new != old {
                    changed = true;
                    self.grange.insert(g, new);
                }
            }
            if !changed {
                break;
            }
            if rounds > GLOBAL_ROUNDS + 2 {
                return Err("global-ranges-diverge".to_string());
            }
        }
        // Final pass: the obligations.
        self.all_function_ranges(true)?;
        Ok(())
    }

    /// A function's address may only be the start routine of `pthread_create`;
    /// any other use (a store, another call, an indirect call anywhere) lets it
    /// run from places the counts do not see.
    fn check_function_addresses(&self) -> Res<()> {
        let fn_values: BTreeMap<ValueId, FunctionId> = self
            .module
            .functions
            .iter()
            .map(|f| (ValueId::new(f.id.raw()), f.id))
            .collect();
        let refers_to_fn = |v: &ValueId| {
            fn_values.contains_key(v)
                || matches!(self.module.constants.get(v), Some(Constant::GlobalRef(t)) if fn_values.contains_key(t))
        };
        for g in &self.module.globals {
            if let Some(init) = &g.init {
                if constant_mentions(init, &|v| refers_to_fn(v)) {
                    return Err(format!("fn-address-in-global-{}", g.name));
                }
            }
        }
        for f in self.funcs.values() {
            for b in &f.blocks {
                for i in &b.instructions {
                    if matches!(i.op, Operation::CallIndirect { .. }) {
                        return Err("call-indirect".to_string());
                    }
                    let is_spawn = self.callee_name(i) == Some("pthread_create");
                    for (k, v) in i.operands.iter().enumerate() {
                        if refers_to_fn(v) && !(is_spawn && k == 2) {
                            return Err(format!("fn-address-escapes-in-{}", f.name));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn callee_name(&self, i: &Instruction) -> Option<&'a str> {
        match &i.op {
            Operation::CallDirect { callee } => {
                self.module.function(*callee).map(|f| f.name.as_str())
            }
            _ => None,
        }
    }

    /// The thread function a `pthread_create` site starts.
    fn spawn_target(&self, i: &Instruction) -> Option<FunctionId> {
        let v = *i.operands.get(2)?;
        let v = match self.module.constants.get(&v) {
            Some(Constant::GlobalRef(t)) => *t,
            _ => v,
        };
        self.module
            .functions
            .iter()
            .find(|f| ValueId::new(f.id.raw()) == v)
            .map(|f| f.id)
    }

    /// Execution bounds (see the module documentation).
    fn compute_counts(&mut self) -> Res<()> {
        let main = self
            .funcs
            .values()
            .find(|f| f.name == "main")
            .map(|f| f.id)
            .ok_or("no-main")?;
        // Edges caller → (callee, executions per caller invocation).
        let mut edges: Vec<(FunctionId, FunctionId, u64)> = Vec::new();
        for f in self.funcs.values() {
            let mult = &self.mult[&f.id];
            for b in &f.blocks {
                let times = mult.get(&b.id).copied().unwrap_or(1);
                for i in &b.instructions {
                    let target = match &i.op {
                        Operation::CallDirect { callee } if self.funcs.contains_key(callee) => {
                            Some(*callee)
                        }
                        Operation::CallDirect { .. }
                            if self.callee_name(i) == Some("pthread_create") =>
                        {
                            Some(self.spawn_target(i).ok_or("spawn-target-unknown")?)
                        }
                        _ => None,
                    };
                    if let Some(t) = target {
                        if !self.funcs.contains_key(&t) {
                            return Err("spawn-target-undefined".to_string());
                        }
                        edges.push((f.id, t, times));
                    }
                }
            }
        }
        let mut count: BTreeMap<FunctionId, u64> = self.funcs.keys().map(|k| (*k, 0)).collect();
        // Relaxation: a function's count is 1 (main) plus its call sites'. More
        // rounds than functions without settling means a cycle: unbounded.
        let n = self.funcs.len() + 1;
        let mut settled = false;
        for _ in 0..=n {
            let mut next: BTreeMap<FunctionId, u64> = self.funcs.keys().map(|k| (*k, 0)).collect();
            next.insert(main, 1);
            for &(from, to, times) in &edges {
                let c = count[&from];
                if c == 0 {
                    continue;
                }
                let add = sat_mul(c, u128::from(times));
                let e = next.entry(to).or_insert(0);
                *e = sat_add(*e, add);
            }
            if next == count {
                settled = true;
                break;
            }
            count = next;
        }
        if !settled {
            // A recursive cycle keeps growing: mark everything still growing
            // (reachable from the cycle) unbounded by one more relaxation with
            // the growing ones forced to `UNBOUNDED`.
            for c in count.values_mut() {
                if *c > 0 {
                    *c = UNBOUNDED;
                }
            }
        }
        self.count = count;
        Ok(())
    }

    /// Scalar integer globals accessed only as the pointer of plain loads and
    /// stores of their own width.
    fn compute_tracked(&mut self) {
        let collapsed = self.module.collapsed_globals.clone().unwrap_or_default();
        let mut cand: BTreeMap<ValueId, u32> = BTreeMap::new();
        for g in &self.module.globals {
            if collapsed.contains(&g.id) {
                continue;
            }
            if let Some(AirType::Integer { bits }) =
                g.value_type.and_then(|t| self.module.get_type(t))
            {
                let bits = u32::from(*bits);
                if (1..=64).contains(&bits) {
                    cand.insert(g.id, bits);
                }
            }
        }
        let mut bad: BTreeSet<ValueId> = BTreeSet::new();
        // Any mention inside another constant (`GlobalRef`, `GlobalElem`, an
        // aggregate initialiser) is an escape.
        for c in self.module.constants.values() {
            mark_mentions(c, &cand, &mut bad);
        }
        for g in &self.module.globals {
            if let Some(init) = &g.init {
                mark_mentions(init, &cand, &mut bad);
            }
        }
        for f in self.funcs.values() {
            for b in &f.blocks {
                for i in &b.instructions {
                    for (k, v) in i.operands.iter().enumerate() {
                        let Some(&bits) = cand.get(v) else { continue };
                        let ok = match i.op {
                            Operation::Load => {
                                k == 0 && !i.is_atomic_pair() && self.int_bits(i) == Some(bits)
                            }
                            Operation::Store => {
                                k == 1
                                    && !i.is_atomic_pair()
                                    && i.operands.first().and_then(|x| self.value_bits(f, *x))
                                        == Some(bits)
                            }
                            _ => false,
                        };
                        if !ok {
                            bad.insert(*v);
                        }
                    }
                }
            }
        }
        self.tracked = cand.into_iter().filter(|(g, _)| !bad.contains(g)).collect();
    }

    /// See [`Self::single_writer`]: every store to `g` is in one function
    /// whose execution count is at most 1.
    fn compute_single_writer(&mut self) {
        let mut writers: BTreeMap<ValueId, BTreeSet<FunctionId>> = BTreeMap::new();
        for f in self.funcs.values() {
            for i in f.blocks.iter().flat_map(|b| b.instructions.iter()) {
                if matches!(i.op, Operation::Store) {
                    if let Some(g) = i.operands.get(1).filter(|g| self.tracked.contains_key(g)) {
                        writers.entry(*g).or_default().insert(f.id);
                    }
                }
            }
        }
        for (g, fs) in writers {
            if fs.len() != 1 {
                continue;
            }
            let f = *fs.iter().next().expect("one writer");
            if self.count.get(&f).copied() == Some(1) {
                self.single_writer.entry(f).or_default().insert(g);
            }
        }
    }

    fn global_init(&self, g: ValueId) -> Iv {
        let bits = self.tracked[&g];
        let glob = self.module.globals.iter().find(|x| x.id == g);
        match glob.and_then(|x| x.init.as_ref()) {
            Some(Constant::Int { value, .. }) => point(wrap(i128::from(*value), bits)),
            // No initialiser (a tentative definition) or zero-initialised.
            None | Some(Constant::ZeroInit | Constant::Null) => point(0),
            _ => top(bits),
        }
    }

    fn int_bits(&self, i: &Instruction) -> Option<u32> {
        match i.result_type.and_then(|t| self.module.get_type(t)) {
            Some(AirType::Integer { bits }) => Some(u32::from(*bits)),
            _ => None,
        }
    }

    /// Integer width of a value: a constant's, a parameter's, or its defining
    /// instruction's result type.
    fn value_bits(&self, f: &AirFunction, v: ValueId) -> Option<u32> {
        if let Some(Constant::Int { bits, .. }) = self.module.constants.get(&v) {
            return Some(u32::from(*bits));
        }
        if let Some(p) = f.params.iter().find(|p| p.id == v) {
            return match p.param_type.and_then(|t| self.module.get_type(t)) {
                Some(AirType::Integer { bits }) => Some(u32::from(*bits)),
                _ => None,
            };
        }
        f.blocks
            .iter()
            .flat_map(|b| b.instructions.iter())
            .find(|i| i.dst == Some(v))
            .and_then(|i| self.int_bits(i))
    }

    /// Ranges of every SSA value of every function that may run; with
    /// `check`, every obligation too.
    fn all_function_ranges(&self, check: bool) -> Res<BTreeMap<ValueId, Iv>> {
        let mut out = BTreeMap::new();
        for f in self.funcs.values() {
            if self.count.get(&f.id).copied().unwrap_or(0) == 0 {
                continue;
            }
            let r = match self.function_ranges(f, check) {
                Ok(r) => r,
                // A pure leaf the engine proves overflow-free for ANY arguments
                // discharges its own obligations (`plus(a, b)` guarded by
                // `assume(b <= 0 || a <= INT_MAX - b)`: relational, beyond
                // intervals). Its ranges are not needed: a call result is `⊤`.
                Err(e) if check && self.pure_leaf(f.id, 0) && self.engine_proves(f.id) => {
                    let _ = e;
                    continue;
                }
                Err(e) => return Err(e),
            };
            out.extend(r);
        }
        Ok(out)
    }

    /// A function whose memory accesses all go to its own `alloca`s and whose
    /// calls go only to other pure leaves or to the inert externs (`abort`,
    /// nondet sources, the assertion helpers): nothing it reads can be written
    /// by a caller or another thread, so its behaviour depends on its arguments
    /// alone.
    fn pure_leaf(&self, f: FunctionId, depth: u32) -> bool {
        if depth > 8 {
            return false;
        }
        let Some(func) = self.funcs.get(&f) else {
            return false;
        };
        let allocas: BTreeSet<ValueId> = func
            .blocks
            .iter()
            .flat_map(|b| b.instructions.iter())
            .filter(|i| matches!(i.op, Operation::Alloca { .. }))
            .filter_map(|i| i.dst)
            .collect();
        for i in func.blocks.iter().flat_map(|b| b.instructions.iter()) {
            match &i.op {
                Operation::Load => {
                    if !i.operands.first().is_some_and(|p| allocas.contains(p)) {
                        return false;
                    }
                }
                Operation::Store => {
                    if !i.operands.get(1).is_some_and(|p| allocas.contains(p)) {
                        return false;
                    }
                }
                Operation::Memcpy
                | Operation::Memset
                | Operation::Gep { .. }
                | Operation::HeapAlloc { .. }
                | Operation::Global { .. }
                | Operation::CallIndirect { .. } => return false,
                Operation::CallDirect { callee } => {
                    if self.funcs.contains_key(callee) {
                        if *callee == f || !self.pure_leaf(*callee, depth + 1) {
                            return false;
                        }
                    } else {
                        let name = self
                            .module
                            .function(*callee)
                            .map_or("", |c| c.name.as_str());
                        let inert = name.starts_with("__VERIFIER_nondet_")
                            || matches!(
                                name,
                                "abort"
                                    | "exit"
                                    | "__assert_fail"
                                    | "reach_error"
                                    | "__VERIFIER_error"
                            )
                            || name.starts_with("llvm.dbg.")
                            || name.starts_with("llvm.lifetime.");
                        if !inert {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        true
    }

    /// The exact-semantics engine proves `f` overflow-free from fresh
    /// arguments ([`crate::unreach_prove::prove_function_no_overflow`]).
    fn engine_proves(&self, f: FunctionId) -> bool {
        matches!(
            crate::unreach_prove::prove_function_no_overflow(
                self.module,
                f,
                std::time::Duration::from_secs(LEAF_BUDGET_S),
                self.signed_shl_locs,
            ),
            crate::unreach_prove::ExhaustiveProof::Proven
        )
    }

    /// Next global ranges from the stores (see the module documentation).
    fn next_global_ranges(&self, ranges: &BTreeMap<ValueId, Iv>) -> BTreeMap<ValueId, Iv> {
        let mut base: BTreeMap<ValueId, Iv> = BTreeMap::new();
        // Total increment amounts (`None`: unbounded). Amounts are not counts:
        // they saturate only at "unbounded", never at `COUNT_CAP`.
        let mut pos: BTreeMap<ValueId, Option<u128>> = BTreeMap::new();
        let mut neg: BTreeMap<ValueId, Option<u128>> = BTreeMap::new();
        for g in self.tracked.keys() {
            base.insert(*g, self.global_init(*g));
        }
        for f in self.funcs.values() {
            let fc = self.count.get(&f.id).copied().unwrap_or(0);
            if fc == 0 {
                continue;
            }
            let defs: BTreeMap<ValueId, &Instruction> = f
                .blocks
                .iter()
                .flat_map(|b| b.instructions.iter())
                .filter_map(|i| i.dst.map(|d| (d, i)))
                .collect();
            for b in &f.blocks {
                let k = sat_mul(
                    fc,
                    u128::from(self.mult[&f.id].get(&b.id).copied().unwrap_or(1)),
                );
                for i in &b.instructions {
                    if !matches!(i.op, Operation::Store) {
                        continue;
                    }
                    let (Some(&v), Some(&g)) = (i.operands.first(), i.operands.get(1)) else {
                        continue;
                    };
                    let Some(&bits) = self.tracked.get(&g) else {
                        continue;
                    };
                    if let Some(c) = increment_of(self.module, &defs, v, g) {
                        let amount = if k == UNBOUNDED {
                            None
                        } else {
                            u128::from(k).checked_mul(c.unsigned_abs())
                        };
                        let slot = if c >= 0 { &mut pos } else { &mut neg };
                        let e = slot.entry(g).or_insert(Some(0));
                        *e = e.zip(amount).and_then(|(x, y)| x.checked_add(y));
                        continue;
                    }
                    let r = self.operand_range(f, ranges, v, bits).unwrap_or(top(bits));
                    let e = base.get(&g).copied();
                    base.insert(g, join(e, r));
                }
            }
        }
        let mut out = BTreeMap::new();
        for (g, bits) in &self.tracked {
            let b = base[g];
            let p = pos
                .get(g)
                .copied()
                .unwrap_or(Some(0))
                .and_then(|x| i128::try_from(x).ok());
            let n = neg
                .get(g)
                .copied()
                .unwrap_or(Some(0))
                .and_then(|x| i128::try_from(x).ok());
            let hi = p
                .and_then(|p| b.hi.checked_add(p))
                .map_or(smax(*bits), |x| x.min(smax(*bits)));
            let lo = n
                .and_then(|n| b.lo.checked_sub(n))
                .map_or(smin(*bits), |x| x.max(smin(*bits)));
            out.insert(*g, Iv { lo, hi });
        }
        out
    }

    /// Range of an operand in `f`: a constant, a computed SSA value, or `⊤`.
    fn operand_range(
        &self,
        f: &AirFunction,
        ranges: &BTreeMap<ValueId, Iv>,
        v: ValueId,
        bits: u32,
    ) -> Option<Iv> {
        if let Some(c) = self.module.constants.get(&v) {
            return match c {
                Constant::Int { value, .. } => Some(point(wrap(i128::from(*value), bits))),
                Constant::Null | Constant::ZeroInit => Some(point(0)),
                _ => Some(top(bits)),
            };
        }
        if f.params.iter().any(|p| p.id == v) {
            return Some(top(bits));
        }
        ranges.get(&v).copied()
    }

    /// The interval fixpoint of one function (see the module documentation);
    /// with `check`, a last pass over the stable ranges discharges the
    /// obligations.
    fn function_ranges(&self, f: &AirFunction, check: bool) -> Res<BTreeMap<ValueId, Iv>> {
        let mut r: BTreeMap<ValueId, Iv> = BTreeMap::new();
        let mut bumps: BTreeMap<ValueId, u32> = BTreeMap::new();
        let defs: BTreeMap<ValueId, &Instruction> = f
            .blocks
            .iter()
            .flat_map(|b| b.instructions.iter())
            .filter_map(|i| i.dst.map(|d| (d, i)))
            .collect();
        let guards = edge_guards(self.module, f);
        let alias = self
            .single_writer
            .get(&f.id)
            .map(|gs| load_aliases(f, gs))
            .unwrap_or_default();
        let none: Vec<(ValueId, bool)> = Vec::new();
        for _ in 0..64 {
            let mut changed = false;
            for b in &f.blocks {
                let g = Ctx {
                    defs: &defs,
                    guards: guards.get(&b.id).unwrap_or(&none),
                    alias: &alias,
                };
                for i in &b.instructions {
                    let Some(d) = i.dst else { continue };
                    let Some(bits) = self.int_bits(i) else {
                        continue;
                    };
                    let Some(new) = self.transfer(f, &r, &g, i, bits, false)? else {
                        continue;
                    };
                    let old = r.get(&d).copied();
                    let mut next = join(old, new);
                    if Some(next) != old {
                        let n = bumps.entry(d).or_insert(0);
                        *n += 1;
                        if *n > WIDEN_AFTER {
                            next = top(bits);
                        }
                        if Some(next) != old {
                            r.insert(d, next);
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                if check {
                    for b in &f.blocks {
                        let g = Ctx {
                            defs: &defs,
                            guards: guards.get(&b.id).unwrap_or(&none),
                            alias: &alias,
                        };
                        for i in &b.instructions {
                            if let Some(bits) = self.int_bits(i) {
                                self.transfer(f, &r, &g, i, bits, true)?;
                            }
                        }
                    }
                }
                return Ok(r);
            }
        }
        Err(format!("ranges-unstable-in-{}", f.name))
    }

    /// The range of `i`'s result (`None`: not yet known, an operand is still
    /// empty). With `check`, an operation that may overflow fails.
    // NOTE: the operation table is one cohesive match.
    #[allow(clippy::too_many_lines)]
    fn transfer(
        &self,
        f: &AirFunction,
        r: &BTreeMap<ValueId, Iv>,
        g: &Ctx<'_>,
        i: &Instruction,
        bits: u32,
        check: bool,
    ) -> Res<Option<Iv>> {
        // While checking, an operand with no range yet (only on code the
        // fixpoint never reached) counts as `⊤`.
        let opnd = |k: usize, w: u32| -> Option<Iv> {
            let v = g.canon(*i.operands.get(k)?);
            let x = self.operand_range(f, r, v, w);
            let x = if check { Some(x.unwrap_or(top(w))) } else { x };
            x.map(|x| self.refine(f, r, g, v, x))
        };
        let fail = |what: &str| -> Res<Option<Iv>> { Err(format!("{what}-in-{}", f.name)) };
        Ok(match &i.op {
            Operation::BinaryOp { kind } => {
                // Comparisons: a boolean.
                let k = *kind;
                if is_compare(k) {
                    return Ok(Some(Iv { lo: -1, hi: 0 }.max_bool(bits)));
                }
                let (Some(a), Some(b)) = (opnd(0, bits), opnd(1, bits)) else {
                    return Ok(None);
                };
                match k {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                        let exact = match k {
                            BinaryOp::Add => Iv {
                                lo: a.lo + b.lo,
                                hi: a.hi + b.hi,
                            },
                            BinaryOp::Sub => Iv {
                                lo: a.lo - b.hi,
                                hi: a.hi - b.lo,
                            },
                            _ => {
                                let ps = [a.lo * b.lo, a.lo * b.hi, a.hi * b.lo, a.hi * b.hi];
                                Iv {
                                    lo: *ps.iter().min().unwrap_or(&0),
                                    hi: *ps.iter().max().unwrap_or(&0),
                                }
                            }
                        };
                        if fits(exact, bits) {
                            Some(exact)
                        } else if i.has_no_signed_wrap() {
                            if check {
                                return fail("overflow");
                            }
                            Some(top(bits))
                        } else {
                            Some(top(bits))
                        }
                    }
                    BinaryOp::SDiv | BinaryOp::SRem => {
                        let min = smin(bits);
                        if check && a.lo <= min && b.lo <= -1 && b.hi >= -1 {
                            return fail("sdiv-int-min");
                        }
                        let m = a.lo.unsigned_abs().max(a.hi.unsigned_abs());
                        let m = i128::try_from(m).unwrap_or(i128::MAX).min(smax(bits));
                        if k == BinaryOp::SDiv {
                            Some(Iv { lo: -m, hi: m })
                        } else {
                            let mb = b.lo.unsigned_abs().max(b.hi.unsigned_abs());
                            let mb = i128::try_from(mb).unwrap_or(i128::MAX).saturating_sub(1);
                            let m = m.min(mb.max(0));
                            Some(Iv { lo: -m, hi: m })
                        }
                    }
                    BinaryOp::UDiv | BinaryOp::URem => {
                        if a.lo >= 0 && b.lo >= 0 {
                            if k == BinaryOp::UDiv {
                                Some(Iv { lo: 0, hi: a.hi })
                            } else {
                                Some(Iv {
                                    lo: 0,
                                    hi: a.hi.min((b.hi - 1).max(0)),
                                })
                            }
                        } else {
                            Some(top(bits))
                        }
                    }
                    BinaryOp::And => {
                        if a.lo >= 0 || b.lo >= 0 {
                            let hi = match (a.lo >= 0, b.lo >= 0) {
                                (true, true) => a.hi.min(b.hi),
                                (true, false) => a.hi,
                                _ => b.hi,
                            };
                            Some(Iv { lo: 0, hi })
                        } else {
                            Some(top(bits))
                        }
                    }
                    BinaryOp::Or | BinaryOp::Xor => {
                        if a.lo >= 0 && b.lo >= 0 {
                            let m = a.hi.max(b.hi);
                            let p = next_pow2_minus1(m);
                            let iv = Iv { lo: 0, hi: p };
                            if fits(iv, bits) {
                                Some(iv)
                            } else {
                                Some(top(bits))
                            }
                        } else {
                            Some(top(bits))
                        }
                    }
                    BinaryOp::Shl => {
                        let signed = self.shl_is_signed(i);
                        if b.lo < 0 || b.hi >= i128::from(bits) {
                            if check && signed {
                                return fail("shl-amount");
                            }
                            return Ok(Some(top(bits)));
                        }
                        // `0 <= b <= b.hi < bits <= 64`: `a * 2^b` stays in i128; it
                        // is monotone in `a` and in `b` for a fixed sign of `a`,
                        // so the extremes are at the corners.
                        let corners = [
                            a.lo * (1i128 << b.lo),
                            a.lo * (1i128 << b.hi),
                            a.hi * (1i128 << b.lo),
                            a.hi * (1i128 << b.hi),
                        ];
                        let exact = Iv {
                            lo: corners.iter().copied().min().unwrap_or(0),
                            hi: corners.iter().copied().max().unwrap_or(0),
                        };
                        if fits(exact, bits) {
                            Some(exact)
                        } else {
                            if check && signed {
                                return fail("shl-overflow");
                            }
                            Some(top(bits))
                        }
                    }
                    BinaryOp::AShr => Some(Iv {
                        lo: a.lo.min(0),
                        hi: a.hi.max(0),
                    }),
                    BinaryOp::LShr => {
                        if a.lo >= 0 {
                            Some(Iv { lo: 0, hi: a.hi })
                        } else {
                            Some(top(bits))
                        }
                    }
                    _ => Some(top(bits)),
                }
            }
            Operation::Cast { kind, .. } => {
                let src_bits = i.operands.first().and_then(|v| self.value_bits(f, *v));
                match (kind, src_bits) {
                    (CastKind::ZExt, Some(sb)) => {
                        let Some(a) = opnd(0, sb) else {
                            return Ok(None);
                        };
                        if a.lo >= 0 {
                            Some(a)
                        } else {
                            Some(Iv {
                                lo: 0,
                                hi: (1i128 << sb) - 1,
                            })
                        }
                    }
                    (CastKind::SExt, Some(sb)) => opnd(0, sb),
                    (CastKind::Trunc, Some(sb)) => {
                        let Some(a) = opnd(0, sb) else {
                            return Ok(None);
                        };
                        if fits(a, bits) {
                            Some(a)
                        } else {
                            Some(top(bits))
                        }
                    }
                    _ => Some(top(bits)),
                }
            }
            Operation::Phi { incoming } => {
                let mut acc: Option<Iv> = None;
                for (_, v) in incoming {
                    if let Some(x) = self.operand_range(f, r, *v, bits) {
                        acc = Some(join(acc, x));
                    }
                }
                acc
            }
            Operation::Select => {
                let (Some(a), Some(b)) = (opnd(1, bits), opnd(2, bits)) else {
                    return Ok(None);
                };
                Some(join(Some(a), b))
            }
            Operation::Copy | Operation::Freeze => opnd(0, bits),
            Operation::Load => {
                let g = i.operands.first().copied();
                match g.and_then(|g| self.grange.get(&g).copied()) {
                    Some(gr) if !i.is_atomic_pair() => Some(gr),
                    _ => Some(top(bits)),
                }
            }
            Operation::CallDirect { .. } => {
                let name = self.callee_name(i).unwrap_or("");
                if matches!(name, "abs" | "labs" | "llabs") || name.starts_with("llvm.abs.") {
                    let ab = i
                        .operands
                        .first()
                        .and_then(|v| self.value_bits(f, *v))
                        .unwrap_or(bits);
                    let Some(a) = opnd(0, ab) else {
                        return Ok(None);
                    };
                    if check && a.lo <= smin(ab) {
                        return fail("abs-int-min");
                    }
                    let m = a.lo.unsigned_abs().max(a.hi.unsigned_abs());
                    let m = i128::try_from(m).unwrap_or(i128::MAX).min(smax(bits));
                    Some(Iv { lo: 0, hi: m })
                } else {
                    Some(top(bits))
                }
            }
            _ => Some(top(bits)),
        })
    }

    /// `x`, the range of `v`, narrowed by the comparisons the edges into this
    /// block established (an SSA value is the thread's own: what the guard
    /// read is what the use reads). Through a sign extension always, through a
    /// zero extension when `v` is non-negative.
    fn refine(
        &self,
        f: &AirFunction,
        r: &BTreeMap<ValueId, Iv>,
        g: &Ctx<'_>,
        v: ValueId,
        x: Iv,
    ) -> Iv {
        let mut x = x;
        for &(c, pol) in g.guards {
            let Some(ci) = g.defs.get(&c) else { continue };
            let Operation::BinaryOp { kind } = ci.op else {
                continue;
            };
            let Some(k) = int_compare(kind) else { continue };
            let [p, q] = ci.operands[..] else { continue };
            let k = if pol { k } else { k.negate() };
            for (lhs, rhs, kk) in [(p, q, k), (q, p, k.swap())] {
                let src = |d: &Instruction| d.operands.first().map(|o| g.canon(*o));
                let same = g.canon(lhs) == v
                    || g.defs.get(&lhs).is_some_and(|d| match d.op {
                        Operation::Cast {
                            kind: CastKind::SExt,
                            ..
                        } => src(d) == Some(v),
                        Operation::Cast {
                            kind: CastKind::ZExt,
                            ..
                        } => src(d) == Some(v) && x.lo >= 0,
                        _ => false,
                    });
                if !same {
                    continue;
                }
                let Some(w) = self.value_bits(f, lhs) else {
                    continue;
                };
                let Some(o) = self.operand_range(f, r, g.canon(rhs), w) else {
                    continue;
                };
                let y = kk.narrow(x, o);
                if y.lo <= y.hi {
                    x = y;
                }
            }
        }
        x
    }

    /// Is this `shl` signed (checked)? Unknown location or no table: yes.
    fn shl_is_signed(&self, i: &Instruction) -> bool {
        let (Some(set), Some(span)) = (self.signed_shl_locs, i.span.as_ref()) else {
            return true;
        };
        crate::witness_lower::span_to_location(self.module, span)
            .is_none_or(|(_, line, col)| set.contains(&(line, col)))
    }
}

impl Iv {
    /// A comparison's result in `bits` (an `i1` is `[-1, 0]` signed; wider
    /// results are `[0, 1]`).
    fn max_bool(self, bits: u32) -> Iv {
        if bits == 1 { self } else { Iv { lo: 0, hi: 1 } }
    }
}

/// Per-block context of the transfer function: the function's definitions and
/// the guards the edges into the block established.
struct Ctx<'b> {
    defs: &'b BTreeMap<ValueId, &'b Instruction>,
    guards: &'b [(ValueId, bool)],
    /// Loads that read the same value as an earlier value ([`load_aliases`]).
    alias: &'b BTreeMap<ValueId, ValueId>,
}

impl Ctx<'_> {
    /// The value `v` is known to equal (itself when none).
    fn canon(&self, v: ValueId) -> ValueId {
        self.alias.get(&v).copied().unwrap_or(v)
    }
}

/// An integer comparison, as "left `Cmp` right".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cmp {
    Eq,
    Ne,
    Slt,
    Sle,
    Sgt,
    Sge,
    Ult,
    Ule,
    Ugt,
    Uge,
}

fn int_compare(k: BinaryOp) -> Option<Cmp> {
    Some(match k {
        BinaryOp::ICmpEq => Cmp::Eq,
        BinaryOp::ICmpNe => Cmp::Ne,
        BinaryOp::ICmpSlt => Cmp::Slt,
        BinaryOp::ICmpSle => Cmp::Sle,
        BinaryOp::ICmpSgt => Cmp::Sgt,
        BinaryOp::ICmpSge => Cmp::Sge,
        BinaryOp::ICmpUlt => Cmp::Ult,
        BinaryOp::ICmpUle => Cmp::Ule,
        BinaryOp::ICmpUgt => Cmp::Ugt,
        BinaryOp::ICmpUge => Cmp::Uge,
        _ => return None,
    })
}

impl Cmp {
    /// `!(a k b)` as `a k' b`.
    fn negate(self) -> Cmp {
        match self {
            Cmp::Eq => Cmp::Ne,
            Cmp::Ne => Cmp::Eq,
            Cmp::Slt => Cmp::Sge,
            Cmp::Sle => Cmp::Sgt,
            Cmp::Sgt => Cmp::Sle,
            Cmp::Sge => Cmp::Slt,
            Cmp::Ult => Cmp::Uge,
            Cmp::Ule => Cmp::Ugt,
            Cmp::Ugt => Cmp::Ule,
            Cmp::Uge => Cmp::Ult,
        }
    }
    /// `a k b` as `b k' a`.
    fn swap(self) -> Cmp {
        match self {
            Cmp::Eq | Cmp::Ne => self,
            Cmp::Slt => Cmp::Sgt,
            Cmp::Sle => Cmp::Sge,
            Cmp::Sgt => Cmp::Slt,
            Cmp::Sge => Cmp::Sle,
            Cmp::Ult => Cmp::Ugt,
            Cmp::Ule => Cmp::Uge,
            Cmp::Ugt => Cmp::Ult,
            Cmp::Uge => Cmp::Ule,
        }
    }
    /// The range of `a` given `a k b` with `b` in `o` (signed ranges; an
    /// unsigned comparison narrows only where both sides are non-negative,
    /// where it agrees with the signed one — or, for `<u`/`<=u` against a
    /// non-negative bound, where it also proves `a` non-negative).
    fn narrow(self, x: Iv, o: Iv) -> Iv {
        match self {
            Cmp::Eq => Iv {
                lo: x.lo.max(o.lo),
                hi: x.hi.min(o.hi),
            },
            Cmp::Ne => {
                if o.lo == o.hi && x.lo == o.lo {
                    Iv {
                        lo: x.lo + 1,
                        hi: x.hi,
                    }
                } else if o.lo == o.hi && x.hi == o.lo {
                    Iv {
                        lo: x.lo,
                        hi: x.hi - 1,
                    }
                } else {
                    x
                }
            }
            Cmp::Slt => Iv {
                lo: x.lo,
                hi: x.hi.min(o.hi - 1),
            },
            Cmp::Sle => Iv {
                lo: x.lo,
                hi: x.hi.min(o.hi),
            },
            Cmp::Sgt => Iv {
                lo: x.lo.max(o.lo + 1),
                hi: x.hi,
            },
            Cmp::Sge => Iv {
                lo: x.lo.max(o.lo),
                hi: x.hi,
            },
            Cmp::Ult if o.lo >= 0 => Iv {
                lo: x.lo.max(0),
                hi: x.hi.min(o.hi - 1),
            },
            Cmp::Ule if o.lo >= 0 => Iv {
                lo: x.lo.max(0),
                hi: x.hi.min(o.hi),
            },
            Cmp::Ugt if o.lo >= 0 && x.lo >= 0 => Iv {
                lo: x.lo.max(o.lo + 1),
                hi: x.hi,
            },
            Cmp::Uge if o.lo >= 0 && x.lo >= 0 => Iv {
                lo: x.lo.max(o.lo),
                hi: x.hi,
            },
            _ => x,
        }
    }
}

/// For the single-writer globals `gs` of `f` (see [`Prover::single_writer`]):
/// every load of one that must read the value of an earlier load or store in
/// `f`, mapped to that value. A forward dataflow carries, per global, the SSA
/// value it is known to hold (a store sets it, a load with nothing known sets
/// it to the load itself); a join keeps it only when every predecessor agrees.
/// Sound because no other function stores to `g` and `f` runs at most once:
/// between two of `f`'s own accesses nothing changes `g`. No fixpoint within
/// the round limit: no aliases.
fn load_aliases(f: &AirFunction, gs: &BTreeSet<ValueId>) -> BTreeMap<ValueId, ValueId> {
    type St = BTreeMap<ValueId, Option<ValueId>>;
    let Some(gr) = Graph::of(f) else {
        return BTreeMap::new();
    };
    let mut out: BTreeMap<BlockId, St> = BTreeMap::new();
    let pass = |out: &BTreeMap<BlockId, St>, alias: &mut BTreeMap<ValueId, ValueId>| {
        let mut new_out: BTreeMap<BlockId, St> = BTreeMap::new();
        for b in &f.blocks {
            let mut cur: St = if b.id == gr.entry {
                gs.iter().map(|g| (*g, None)).collect()
            } else {
                let mut acc: Option<St> = None;
                for p in &gr.preds[&b.id] {
                    let Some(po) = new_out.get(p).or_else(|| out.get(p)) else {
                        continue;
                    };
                    acc = Some(match acc {
                        None => po.clone(),
                        Some(a) => a
                            .iter()
                            .map(|(g, x)| (*g, if po.get(g) == Some(x) { *x } else { None }))
                            .collect(),
                    });
                }
                let Some(acc) = acc else { continue };
                acc
            };
            for i in &b.instructions {
                match i.op {
                    Operation::Store => {
                        if let (Some(v), Some(g)) = (i.operands.first(), i.operands.get(1)) {
                            if gs.contains(g) {
                                let v = alias.get(v).copied().unwrap_or(*v);
                                cur.insert(*g, Some(v));
                            }
                        }
                    }
                    Operation::Load => {
                        if let (Some(g), Some(d)) = (i.operands.first(), i.dst) {
                            if gs.contains(g) {
                                match cur.get(g).copied().flatten() {
                                    Some(x) => {
                                        alias.insert(d, x);
                                    }
                                    None => {
                                        cur.insert(*g, Some(d));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            new_out.insert(b.id, cur);
        }
        new_out
    };
    for _ in 0..64 {
        let mut scratch = BTreeMap::new();
        let next = pass(&out, &mut scratch);
        if next == out {
            // Stable: the aliases of one more pass over the fixpoint.
            let mut alias = BTreeMap::new();
            let _ = pass(&out, &mut alias);
            return alias;
        }
        out = next;
    }
    BTreeMap::new()
}

/// Per block, the conditions the edges into it established: `(c, pol)` when a
/// strict dominator `D` branches on `c` and its `pol` successor `S` is the
/// block itself or dominates it while `D` is `S`'s only predecessor (so every
/// path into the block took that edge).
/// A function's CFG: blocks, predecessors, terminators and dominators.
struct Graph<'f> {
    ids: Vec<BlockId>,
    entry: BlockId,
    preds: BTreeMap<BlockId, BTreeSet<BlockId>>,
    succs: BTreeMap<BlockId, Vec<BlockId>>,
    term: BTreeMap<BlockId, &'f Instruction>,
    /// Every block's dominators (itself included).
    dom: BTreeMap<BlockId, BTreeSet<BlockId>>,
}

impl<'f> Graph<'f> {
    fn of(f: &'f AirFunction) -> Option<Self> {
        let ids: Vec<BlockId> = f.blocks.iter().map(|b| b.id).collect();
        let entry = f.entry_block.or_else(|| ids.first().copied())?;
        let mut preds: BTreeMap<BlockId, BTreeSet<BlockId>> =
            ids.iter().map(|b| (*b, BTreeSet::new())).collect();
        let mut succs: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
        let mut term: BTreeMap<BlockId, &Instruction> = BTreeMap::new();
        for b in &f.blocks {
            let ss = b.instructions.last().map(successors).unwrap_or_default();
            if let Some(t) = b.instructions.last() {
                term.insert(b.id, t);
            }
            for x in &ss {
                preds.entry(*x).or_default().insert(b.id);
            }
            succs.insert(b.id, ss);
        }
        let all: BTreeSet<BlockId> = ids.iter().copied().collect();
        let mut dom: BTreeMap<BlockId, BTreeSet<BlockId>> =
            ids.iter().map(|b| (*b, all.clone())).collect();
        dom.insert(entry, BTreeSet::from([entry]));
        let mut changed = true;
        while changed {
            changed = false;
            for b in &ids {
                if *b == entry {
                    continue;
                }
                let mut new: Option<BTreeSet<BlockId>> = None;
                for p in &preds[b] {
                    let d = &dom[p];
                    new = Some(match new {
                        None => d.clone(),
                        Some(n) => n.intersection(d).copied().collect(),
                    });
                }
                let mut new = new.unwrap_or_else(|| all.clone());
                new.insert(*b);
                if new != dom[b] {
                    dom.insert(*b, new);
                    changed = true;
                }
            }
        }
        Some(Self {
            ids,
            entry,
            preds,
            succs,
            term,
            dom,
        })
    }
}

/// Per block on a CFG cycle, how often it may run per invocation of `f`: the
/// product, over the natural loops containing it, of each loop's trip bound
/// (`+ 1` for the loop's own header); `UNBOUNDED` when a containing loop has
/// no bound or the cycle is irreducible.
fn block_multiplicity(module: &AirModule, f: &AirFunction) -> BTreeMap<BlockId, u64> {
    let mut out = BTreeMap::new();
    let cyclic = cyclic_blocks(f);
    if cyclic.is_empty() {
        return out;
    }
    let Some(g) = Graph::of(f) else {
        return cyclic.into_iter().map(|b| (b, UNBOUNDED)).collect();
    };
    let _ = g.entry;
    // Natural loops: header → (latches, body).
    let mut latches: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
    for b in &g.ids {
        for h in &g.succs[b] {
            if g.dom[b].contains(h) {
                latches.entry(*h).or_default().push(*b);
            }
        }
    }
    let mut loops: Vec<(BlockId, BTreeSet<BlockId>, Option<u64>)> = Vec::new();
    for (h, ls) in &latches {
        let mut body: BTreeSet<BlockId> = BTreeSet::from([*h]);
        let mut stack: Vec<BlockId> = ls.clone();
        while let Some(x) = stack.pop() {
            if body.insert(x) {
                stack.extend(g.preds[&x].iter().copied());
            }
        }
        let bound = if ls.len() == 1 {
            trip_bound(module, f, &g, *h, ls[0], &body)
        } else {
            None
        };
        loops.push((*h, body, bound));
    }
    for b in cyclic {
        let mut m: u64 = 1;
        let mut inside = false;
        for (h, body, bound) in &loops {
            if !body.contains(&b) {
                continue;
            }
            inside = true;
            let t = match bound {
                Some(t) if b == *h => t.saturating_add(1),
                Some(t) => *t,
                None => UNBOUNDED,
            };
            m = sat_mul(m, u128::from(t));
        }
        out.insert(b, if inside { m } else { UNBOUNDED });
    }
    out
}

/// Trip bound of the natural loop with header `h` and single latch `latch`:
/// the header branches on `p K N` (`N` a constant) with exactly one successor
/// in the loop, `p` is a header phi whose latch value is `p + s` (`s > 0` a
/// constant) and whose other incoming values are constants, and `N − 1 + s`
/// fits the type (no wrap-around can keep the loop going). The body then runs
/// at most `⌈(N − c0) / s⌉` times (`K = <`) or `⌊(N − c0) / s⌋ + 1` (`K = <=`)
/// per entry, `c0` the least initial value.
// NOTE: one recognizer for one loop shape (header compare, phi, step); split,
// its conditions would be harder to check against each other.
#[allow(clippy::too_many_lines)]
fn trip_bound(
    module: &AirModule,
    f: &AirFunction,
    g: &Graph<'_>,
    h: BlockId,
    latch: BlockId,
    body: &BTreeSet<BlockId>,
) -> Option<u64> {
    let t = g.term.get(&h)?;
    let Operation::CondBr {
        then_target,
        else_target,
    } = t.op
    else {
        return None;
    };
    let stay_then = match (body.contains(&then_target), body.contains(&else_target)) {
        (true, false) => true,
        (false, true) => false,
        _ => return None,
    };
    let c = *t.operands.first()?;
    let insts: BTreeMap<ValueId, &Instruction> = f
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter_map(|i| i.dst.map(|d| (d, i)))
        .collect();
    let ci = insts.get(&c)?;
    let Operation::BinaryOp { kind } = ci.op else {
        return None;
    };
    let k = int_compare(kind)?;
    let k = if stay_then { k } else { k.negate() };
    let [a, b] = ci.operands[..] else { return None };
    let konst = |v: ValueId| match module.constants.get(&v) {
        Some(Constant::Int { value, .. }) => Some(i128::from(*value)),
        _ => None,
    };
    let header = f.blocks.iter().find(|x| x.id == h)?;
    let is_header_phi = |v: ValueId| {
        header
            .instructions
            .iter()
            .any(|i| i.dst == Some(v) && matches!(i.op, Operation::Phi { .. }))
    };
    let (p, n, k) = if is_header_phi(a) {
        (a, konst(b)?, k)
    } else if is_header_phi(b) {
        (b, konst(a)?, k.swap())
    } else {
        return None;
    };
    let phi = insts.get(&p)?;
    let Operation::Phi { incoming } = &phi.op else {
        return None;
    };
    let bits = match phi.result_type.and_then(|t| module.get_type(t)) {
        Some(AirType::Integer { bits }) => u32::from(*bits),
        _ => return None,
    };
    let mut step: Option<i128> = None;
    let mut c0: Option<i128> = None;
    for (from, v) in incoming {
        if *from == latch {
            let inc = insts.get(v)?;
            let Operation::BinaryOp {
                kind: BinaryOp::Add,
            } = inc.op
            else {
                return None;
            };
            let [x, y] = inc.operands[..] else {
                return None;
            };
            let s = if x == p {
                konst(y)?
            } else if y == p {
                konst(x)?
            } else {
                return None;
            };
            if s <= 0 {
                return None;
            }
            step = Some(s);
        } else {
            let v0 = wrap(konst(*v)?, bits);
            c0 = Some(c0.map_or(v0, |m: i128| m.min(v0)));
        }
    }
    let (s, c0) = (step?, c0?);
    let n = wrap(n, bits);
    let (unsigned, inclusive) = match k {
        Cmp::Slt => (false, false),
        Cmp::Sle => (false, true),
        Cmp::Ult => (true, false),
        Cmp::Ule => (true, true),
        _ => return None,
    };
    if unsigned && (c0 < 0 || n < 0) {
        return None;
    }
    let last = if inclusive { n } else { n - 1 };
    if last + s > smax(bits) {
        return None;
    }
    let trips = if last < c0 { 0 } else { (last - c0) / s + 1 };
    u64::try_from(trips).ok()
}

fn edge_guards(module: &AirModule, f: &AirFunction) -> BTreeMap<BlockId, Vec<(ValueId, bool)>> {
    let Some(g) = Graph::of(f) else {
        return BTreeMap::new();
    };
    let (ids, preds, dom, term) = (&g.ids, &g.preds, &g.dom, &g.term);
    let mut out: BTreeMap<BlockId, Vec<(ValueId, bool)>> = BTreeMap::new();
    for b in ids {
        let mut gs = Vec::new();
        for d in &dom[b] {
            if d == b {
                continue;
            }
            let Some(t) = term.get(d) else { continue };
            let Operation::CondBr {
                then_target,
                else_target,
            } = t.op
            else {
                continue;
            };
            let Some(&c) = t.operands.first() else {
                continue;
            };
            if then_target == else_target {
                continue;
            }
            for (s, pol) in [(then_target, true), (else_target, false)] {
                let single = preds.get(&s).is_some_and(|p| p.len() == 1 && p.contains(d));
                if single && (s == *b || dom[b].contains(&s)) {
                    gs.push((c, pol));
                }
            }
        }
        out.insert(*b, gs);
    }
    // A branch on an `i1` phi (`while (a && b)`: `phi [false, %cond], [%cmp2,
    // %rhs]`): the phi has the branch's polarity only along the incoming edges
    // whose value is not the constant opposite, so when exactly one is left the
    // path came from that block — its value held, and so did that block's own
    // guards. Two rounds cover a `&&` of three.
    let defs: BTreeMap<ValueId, &Instruction> = f
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter_map(|i| i.dst.map(|d| (d, i)))
        .collect();
    let konst = |v: ValueId| match module.constants.get(&v) {
        Some(Constant::Int { value, .. }) => Some(*value != 0),
        _ => None,
    };
    for _ in 0..2 {
        let snapshot = out.clone();
        for (b, gs) in &mut out {
            let mut extra: Vec<(ValueId, bool)> = Vec::new();
            for &(c, pol) in gs.iter() {
                let Some(Operation::Phi { incoming }) = defs.get(&c).map(|i| &i.op) else {
                    continue;
                };
                let live: Vec<&(BlockId, ValueId)> = incoming
                    .iter()
                    .filter(|(_, v)| konst(*v) != Some(!pol))
                    .collect();
                let [(p, v)] = live[..] else { continue };
                if konst(*v).is_none() {
                    extra.push((*v, pol));
                }
                if let Some(pg) = snapshot.get(p) {
                    extra.extend(pg.iter().copied());
                }
            }
            let _ = b;
            for e in extra {
                if !gs.contains(&e) {
                    gs.push(e);
                }
            }
        }
    }
    out
}

fn successors(t: &Instruction) -> Vec<BlockId> {
    match &t.op {
        Operation::Br { target } => vec![*target],
        Operation::CondBr {
            then_target,
            else_target,
        } => vec![*then_target, *else_target],
        Operation::Switch { default, cases } => {
            let mut v = vec![*default];
            v.extend(cases.iter().map(|(_, t)| *t));
            v
        }
        _ => Vec::new(),
    }
}

fn is_compare(k: BinaryOp) -> bool {
    matches!(
        k,
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
            | BinaryOp::FCmpOeq
            | BinaryOp::FCmpOne
            | BinaryOp::FCmpOgt
            | BinaryOp::FCmpOge
            | BinaryOp::FCmpOlt
            | BinaryOp::FCmpOle
            | BinaryOp::FCmpUeq
            | BinaryOp::FCmpUne
            | BinaryOp::FCmpUgt
            | BinaryOp::FCmpUge
            | BinaryOp::FCmpUlt
            | BinaryOp::FCmpUle
            | BinaryOp::FCmpOrd
            | BinaryOp::FCmpUno
            | BinaryOp::FCmpTrue
            | BinaryOp::FCmpFalse
    )
}

/// `2^k − 1` for the least `k` with `2^k − 1 >= m` (`m >= 0`).
fn next_pow2_minus1(m: i128) -> i128 {
    let mut p: i128 = 0;
    while p < m {
        p = p * 2 + 1;
    }
    p
}

fn sat_add(a: u64, b: u64) -> u64 {
    let s = a.saturating_add(b);
    if s >= COUNT_CAP { UNBOUNDED } else { s }
}

fn sat_mul(a: u64, b: u128) -> u64 {
    if a == UNBOUNDED || b >= u128::from(UNBOUNDED) {
        return UNBOUNDED;
    }
    let p = u128::from(a).saturating_mul(b);
    if p >= u128::from(COUNT_CAP) {
        UNBOUNDED
    } else {
        // INVARIANT: below `COUNT_CAP`, which fits in u64.
        #[allow(clippy::cast_possible_truncation)]
        let p = p as u64;
        p
    }
}

/// `v` is `load(g) + c` or `load(g) − c` (`c` a constant): the increment `c`.
fn increment_of(
    module: &AirModule,
    defs: &BTreeMap<ValueId, &Instruction>,
    v: ValueId,
    g: ValueId,
) -> Option<i128> {
    let i = defs.get(&v)?;
    let Operation::BinaryOp { kind } = i.op else {
        return None;
    };
    let [x, y] = i.operands[..] else { return None };
    let is_load_of_g = |w: ValueId| {
        defs.get(&w).is_some_and(|d| {
            matches!(d.op, Operation::Load) && !d.is_atomic_pair() && d.operands.first() == Some(&g)
        })
    };
    let konst = |w: ValueId| match module.constants.get(&w) {
        Some(Constant::Int { value, .. }) => Some(i128::from(*value)),
        _ => None,
    };
    match kind {
        BinaryOp::Add if is_load_of_g(x) => konst(y),
        BinaryOp::Add if is_load_of_g(y) => konst(x),
        BinaryOp::Sub if is_load_of_g(x) => konst(y).map(|c| -c),
        _ => None,
    }
}

/// Blocks of `f` that lie on a CFG cycle.
fn cyclic_blocks(f: &AirFunction) -> BTreeSet<BlockId> {
    let mut succ: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
    for b in &f.blocks {
        let mut s = Vec::new();
        if let Some(t) = b.instructions.last() {
            match &t.op {
                Operation::Br { target } => s.push(*target),
                Operation::CondBr {
                    then_target,
                    else_target,
                } => {
                    s.push(*then_target);
                    s.push(*else_target);
                }
                Operation::Switch { default, cases } => {
                    s.push(*default);
                    s.extend(cases.iter().map(|(_, t)| *t));
                }
                _ => {}
            }
        }
        succ.insert(b.id, s);
    }
    let mut out = BTreeSet::new();
    for b in &f.blocks {
        let mut seen: BTreeSet<BlockId> = BTreeSet::new();
        let mut stack: Vec<BlockId> = succ[&b.id].clone();
        while let Some(x) = stack.pop() {
            if x == b.id {
                out.insert(b.id);
                break;
            }
            if seen.insert(x) {
                if let Some(s) = succ.get(&x) {
                    stack.extend(s.iter().copied());
                }
            }
        }
    }
    out
}

/// Does constant `c` mention a value `p` accepts?
fn constant_mentions(c: &Constant, p: &dyn Fn(&ValueId) -> bool) -> bool {
    match c {
        Constant::GlobalRef(v) => p(v),
        Constant::GlobalElem { base, .. } => p(base),
        Constant::Aggregate { elements, .. } => elements.iter().any(|e| constant_mentions(e, p)),
        _ => false,
    }
}

fn mark_mentions(c: &Constant, cand: &BTreeMap<ValueId, u32>, bad: &mut BTreeSet<ValueId>) {
    match c {
        Constant::GlobalRef(v) | Constant::GlobalElem { base: v, .. } => {
            if cand.contains_key(v) {
                bad.insert(*v);
            }
        }
        Constant::Aggregate { elements, .. } => {
            for e in elements {
                mark_mentions(e, cand, bad);
            }
        }
        _ => {}
    }
}
