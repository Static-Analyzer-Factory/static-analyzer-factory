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
//!    Copies generalise increments: a store of `load(h) + c` to `g` adds `c`
//!    to some value `h` held, so `g` is bounded over the globals it copies
//!    from. Past the type's range a wrapping (non-`nsw`) increment makes the
//!    global `⊤`.
//! 3. **Value ranges.** Per function, an interval fixpoint over the SSA values
//!    (phis joined, values that keep changing widened to their type's range),
//!    with tracked loads at their global's range and every other load, call
//!    result and parameter at `⊤`. Values are narrowed by the facts in force:
//!    dominating branch conditions, `assume` calls, and disjunctions (several
//!    predecessors or an `i1` phi, never a loop header's). Two loads read the
//!    same value when nothing else can store between them: the global has a
//!    single writer that runs once, or both lie in one atomic section
//!    (`__VERIFIER_atomic_begin` … `_end`). A global `main` stores to only
//!    before it can have spawned a thread is constant once threads run; loads
//!    in thread-only code see its value at the spawn sites.
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
/// The intersection of two ranges of one value (`a` when they are disjoint,
/// which only unreachable code can make them).
fn meet(a: Iv, b: Iv) -> Iv {
    let m = Iv {
        lo: a.lo.max(b.lo),
        hi: a.hi.min(b.hi),
    };
    if m.lo <= m.hi { m } else { a }
}
/// Every SSA value of `f` and the instruction defining it.
fn defs_of(f: &AirFunction) -> BTreeMap<ValueId, &Instruction> {
    f.blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter_map(|i| i.dst.map(|d| (d, i)))
        .collect()
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
    /// read the same value (nobody else writes), see [`alias_dataflow`].
    single_writer: BTreeMap<FunctionId, BTreeSet<ValueId>>,
    /// Functions that return only when their one argument is non-zero
    /// (`assume_abort_if_not`, see [`is_assume_fn`]), and `__VERIFIER_assume`.
    assume_fns: BTreeSet<FunctionId>,
    /// Tracked globals stored to only by `main` before it can have started a
    /// thread ([`Self::pre_spawn_stores`]): constant once any thread runs.
    frozen_globals: BTreeSet<ValueId>,
    /// Functions that run only inside threads (reached from `main` by no
    /// chain of direct calls), so only after the first spawn.
    post_only: BTreeSet<FunctionId>,
    /// The range of each frozen global once threads run: its value at the
    /// spawn sites ([`Self::frozen_ranges`]). Used by loads in `post_only`.
    frozen: BTreeMap<ValueId, Iv>,
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
            assume_fns: module
                .functions
                .iter()
                .filter(|f| {
                    if f.is_declaration || f.blocks.is_empty() {
                        f.name == "__VERIFIER_assume"
                    } else {
                        is_assume_fn(module, f)
                    }
                })
                .map(|f| f.id)
                .collect(),
            frozen_globals: BTreeSet::new(),
            post_only: BTreeSet::new(),
            frozen: BTreeMap::new(),
        }
    }

    fn prove(&mut self) -> Res<()> {
        self.check_function_addresses()?;
        self.compute_counts()?;
        self.compute_tracked();
        self.compute_single_writer();
        self.compute_post_only();
        let ranges = self.global_fixpoint()?;
        // Second round with the frozen globals' values at the spawn sites,
        // read off the first round's (sound) ranges.
        if !self.frozen_globals.is_empty() && !self.post_only.is_empty() {
            self.frozen = self.frozen_ranges(&ranges);
            if !self.frozen.is_empty() {
                self.global_fixpoint()?;
            }
        }
        // Final pass: the obligations.
        self.all_function_ranges(true)?;
        Ok(())
    }

    /// Global ranges to a fixpoint (each round re-evaluates every function
    /// against the current global ranges); the value ranges of the last round.
    fn global_fixpoint(&mut self) -> Res<BTreeMap<ValueId, Iv>> {
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
                return Ok(ranges);
            }
            if rounds > GLOBAL_ROUNDS + 2 {
                return Err("global-ranges-diverge".to_string());
            }
        }
    }

    /// See [`Self::post_only`].
    fn compute_post_only(&mut self) {
        let Some(main) = self.funcs.values().find(|f| f.name == "main").map(|f| f.id) else {
            return;
        };
        let mut seen: BTreeSet<FunctionId> = BTreeSet::from([main]);
        let mut stack = vec![main];
        while let Some(f) = stack.pop() {
            for i in self.funcs[&f]
                .blocks
                .iter()
                .flat_map(|b| b.instructions.iter())
            {
                if let Operation::CallDirect { callee } = i.op {
                    if self.funcs.contains_key(&callee) && seen.insert(callee) {
                        stack.push(callee);
                    }
                }
            }
        }
        self.post_only = self
            .funcs
            .keys()
            .filter(|f| !seen.contains(f))
            .copied()
            .collect();
    }

    /// Per frozen global, the join over every `pthread_create` site of its
    /// value there: in `main`, the SSA value it is known to hold, narrowed by
    /// the facts in force at the site; elsewhere its global range. After the
    /// first spawn the global never changes, so every thread reads its value
    /// at that first site.
    fn frozen_ranges(&self, ranges: &BTreeMap<ValueId, Iv>) -> BTreeMap<ValueId, Iv> {
        let Some(main) = self.funcs.values().find(|f| f.name == "main") else {
            return BTreeMap::new();
        };
        let is_spawn = |i: &Instruction| self.callee_name(i) == Some("pthread_create");
        let outside = self.funcs.values().any(|f| {
            f.id != main.id
                && f.blocks
                    .iter()
                    .flat_map(|b| b.instructions.iter())
                    .any(&is_spawn)
        });
        let sites: Positions = main
            .blocks
            .iter()
            .flat_map(|b| {
                b.instructions
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| is_spawn(i))
                    .map(move |(k, _)| (b.id, k))
            })
            .collect();
        let (alias, states) = self.aliases(main, &sites);
        let defs = defs_of(main);
        let (guards, facts) = self.instruction_facts(main, &defs);
        let alts = edge_alternatives(main, &guards);
        let cfg = CfgFacts::of(main);
        let mut out = BTreeMap::new();
        for g in &self.frozen_globals {
            let bits = self.tracked[g];
            let mut acc: Option<Iv> = outside.then(|| self.grange[g]);
            for (b, k) in &sites {
                let at = states
                    .get(&(*b, *k))
                    .and_then(|st| st.get(g))
                    .and_then(|v| {
                        let x = self.operand_range(main, ranges, *v, bits)?;
                        let ctx = Ctx {
                            defs: &defs,
                            guards: &facts[b][*k],
                            alias: &alias,
                            blocks: &guards,
                            alts: alts.get(b).map_or(&[], Vec::as_slice),
                            cfg: &cfg,
                        };
                        Some(self.refine(main, ranges, &ctx, *v, x))
                    });
                acc = Some(join(acc, at.unwrap_or(self.grange[g])));
            }
            if let Some(r) = acc {
                out.insert(*g, r);
            }
        }
        out
    }

    /// [`alias_dataflow`] for `f`: its single-writer globals always, the other
    /// tracked globals inside atomic sections.
    #[allow(clippy::type_complexity)]
    fn aliases(
        &self,
        f: &AirFunction,
        want: &Positions,
    ) -> (
        BTreeMap<ValueId, ValueId>,
        BTreeMap<(BlockId, usize), BTreeMap<ValueId, ValueId>>,
    ) {
        let none = BTreeSet::new();
        let sw = self.single_writer.get(&f.id).unwrap_or(&none);
        let shared: BTreeSet<ValueId> = self
            .tracked
            .keys()
            .filter(|g| !sw.contains(g))
            .copied()
            .collect();
        let inert = |i: &Instruction| self.inert_call(i);
        let module = self.module;
        let callee = move |i: &Instruction| -> Option<(&'a str, bool)> {
            let Operation::CallDirect { callee } = i.op else {
                return None;
            };
            let c = module.function(callee)?;
            Some((c.name.as_str(), c.is_declaration || c.blocks.is_empty()))
        };
        let scope = AliasScope {
            sw,
            shared: &shared,
            inert: &inert,
            callee: &callee,
        };
        alias_dataflow(f, &scope, want)
    }

    /// A call that stores to no global: a nondet source, `__VERIFIER_assume`,
    /// a process exit, or an `assume` helper ([`Self::assume_fns`]) whose only
    /// calls are such.
    fn inert_call(&self, i: &Instruction) -> bool {
        let Operation::CallDirect { callee } = i.op else {
            return false;
        };
        let Some(c) = self.module.function(callee) else {
            return false;
        };
        let inert_extern = |c: &AirFunction| {
            (c.is_declaration || c.blocks.is_empty())
                && (c.name.starts_with("__VERIFIER_nondet_")
                    || matches!(
                        c.name.as_str(),
                        "__VERIFIER_assume"
                            | "abort"
                            | "exit"
                            | "_exit"
                            | "__assert_fail"
                            | "reach_error"
                    ))
        };
        if inert_extern(c) {
            return true;
        }
        self.assume_fns.contains(&callee)
            && c.blocks
                .iter()
                .flat_map(|b| b.instructions.iter())
                .all(|j| match j.op {
                    Operation::Store | Operation::CallIndirect { .. } => false,
                    Operation::CallDirect { callee } => {
                        self.module.function(callee).is_some_and(inert_extern)
                    }
                    _ => true,
                })
    }

    /// The guards in force at each instruction: the block's edge guards, plus
    /// the facts of the `assume` calls before it in the block (expanded through
    /// `i1` phis like edge guards).
    fn instruction_facts(
        &self,
        f: &AirFunction,
        defs: &BTreeMap<ValueId, &Instruction>,
    ) -> (Guards, InstrFacts) {
        let guards = edge_guards(self.module, f, &self.assume_fns);
        let facts_at = |b: &saf_core::air::AirBlock| -> Vec<Vec<(ValueId, bool)>> {
            let mut local: Vec<(ValueId, bool)> = guards.get(&b.id).cloned().unwrap_or_default();
            let mut per = Vec::with_capacity(b.instructions.len());
            for i in &b.instructions {
                per.push(local.clone());
                if let Some(fct) = assume_fact(defs, &self.assume_fns, i) {
                    for e in expand_fact(self.module, defs, &guards, fct) {
                        if !local.contains(&e) {
                            local.push(e);
                        }
                    }
                }
            }
            per
        };
        let facts = f.blocks.iter().map(|b| (b.id, facts_at(b))).collect();
        (guards, facts)
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
    ///
    /// A store in `main` before it can have started any thread
    /// ([`Self::pre_spawn_stores`]) happens before every thread runs, so it
    /// does not make `main` a second writer of a thread's global.
    fn compute_single_writer(&mut self) {
        let pre = self.pre_spawn_stores();
        let main = self.funcs.values().find(|f| f.name == "main").map(|f| f.id);
        let mut writers: BTreeMap<ValueId, BTreeSet<FunctionId>> = BTreeMap::new();
        let mut pre_only: BTreeSet<ValueId> = BTreeSet::new();
        for f in self.funcs.values() {
            for b in &f.blocks {
                for (k, i) in b.instructions.iter().enumerate() {
                    if !matches!(i.op, Operation::Store) {
                        continue;
                    }
                    let Some(g) = i.operands.get(1).filter(|g| self.tracked.contains_key(g)) else {
                        continue;
                    };
                    if Some(f.id) == main && pre.contains(&(b.id, k)) {
                        pre_only.insert(*g);
                    } else {
                        writers.entry(*g).or_default().insert(f.id);
                    }
                }
            }
        }
        // Written only before the first spawn: `main` is its single writer,
        // and the global is frozen once threads run.
        if let Some(m) = main {
            for g in pre_only {
                if let std::collections::btree_map::Entry::Vacant(e) = writers.entry(g) {
                    e.insert(BTreeSet::from([m]));
                    self.frozen_globals.insert(g);
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

    /// Stores in `main` (run once, called from nowhere) at positions no thread
    /// creation can precede: not after a call that may start a thread in its
    /// block, nor in a block reachable from one.
    fn pre_spawn_stores(&self) -> BTreeSet<(BlockId, usize)> {
        let Some(main) = self.funcs.values().find(|f| f.name == "main") else {
            return BTreeSet::new();
        };
        if self.count.get(&main.id).copied() != Some(1) {
            return BTreeSet::new();
        }
        let called = self.funcs.values().any(|f| {
            f.blocks
                .iter()
                .flat_map(|b| b.instructions.iter())
                .any(|i| matches!(i.op, Operation::CallDirect { callee } if callee == main.id))
        });
        if called {
            return BTreeSet::new();
        }
        // Functions that may start a thread, transitively.
        let direct_spawn = |i: &Instruction| {
            matches!(self.callee_name(i), Some("pthread_create" | "thrd_create"))
                || matches!(i.op, Operation::CallIndirect { .. })
        };
        let mut spawns: BTreeSet<FunctionId> = BTreeSet::new();
        loop {
            let before = spawns.len();
            for f in self.funcs.values() {
                let any = f.blocks.iter().flat_map(|b| b.instructions.iter()).any(|i| {
                    direct_spawn(i)
                        || matches!(i.op, Operation::CallDirect { callee } if spawns.contains(&callee))
                });
                if any {
                    spawns.insert(f.id);
                }
            }
            if spawns.len() == before {
                break;
            }
        }
        let may_spawn = |i: &Instruction| {
            direct_spawn(i)
                || matches!(i.op, Operation::CallDirect { callee } if spawns.contains(&callee))
        };
        // Blocks reachable from a spawning call.
        let succ: BTreeMap<BlockId, Vec<BlockId>> = main
            .blocks
            .iter()
            .map(|b| {
                (
                    b.id,
                    b.instructions.last().map(successors).unwrap_or_default(),
                )
            })
            .collect();
        let mut after: BTreeSet<BlockId> = BTreeSet::new();
        let mut stack: Vec<BlockId> = main
            .blocks
            .iter()
            .filter(|b| b.instructions.iter().any(&may_spawn))
            .flat_map(|b| succ[&b.id].iter().copied())
            .collect();
        while let Some(b) = stack.pop() {
            if after.insert(b) {
                stack.extend(succ.get(&b).into_iter().flatten().copied());
            }
        }
        let mut out = BTreeSet::new();
        for b in &main.blocks {
            if after.contains(&b.id) {
                continue;
            }
            for (k, i) in b.instructions.iter().enumerate() {
                if may_spawn(i) {
                    break;
                }
                if matches!(i.op, Operation::Store) {
                    out.insert((b.id, k));
                }
            }
        }
        out
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
    ///
    /// Increments generalise to copies between tracked globals of one width: a
    /// store of `load(h) + c` to `g` (`c` may be 0: `tmp = n; n2 = tmp + 1`)
    /// writes SOME value `h` held plus `c`. Tracing any value back through the
    /// copies that produced it reaches an initial value or a non-copy store,
    /// over distinct, strictly earlier store executions, and only through the
    /// globals `g` copies from, directly or not (its "sources", `g` included).
    /// So `g` lies in `[lo − Σ neg, hi + Σ pos]` with `[lo, hi]` over the inits
    /// and non-copy stores of its sources and the sums over the copy
    /// executions into them.
    fn next_global_ranges(&self, ranges: &BTreeMap<ValueId, Iv>) -> BTreeMap<ValueId, Iv> {
        let mut base: Vec<(ValueId, Iv)> = Vec::new();
        // (target, source, executions × |c| (`None`: unbounded), c ≥ 0, wraps).
        let mut copies: Vec<(ValueId, ValueId, Option<u128>, bool, bool)> = Vec::new();
        for g in self.tracked.keys() {
            base.push((*g, self.global_init(*g)));
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
                    let copy = copy_of(self.module, &defs, v)
                        .filter(|(h, _)| self.tracked.get(h) == Some(&bits));
                    if let Some((h, c)) = copy {
                        let c = wrap(c, bits);
                        // Amounts are not counts: they saturate only at
                        // "unbounded", never at `COUNT_CAP`.
                        let amount = if k == UNBOUNDED {
                            None
                        } else {
                            u128::from(k).checked_mul(c.unsigned_abs())
                        };
                        let wraps = c != 0 && !defs.get(&v).is_some_and(|d| d.has_no_signed_wrap());
                        copies.push((g, h, amount, c >= 0, wraps));
                        continue;
                    }
                    let r = self.operand_range(f, ranges, v, bits).unwrap_or(top(bits));
                    base.push((g, r));
                }
            }
        }
        let mut from: BTreeMap<ValueId, BTreeSet<ValueId>> = BTreeMap::new();
        for (g, h, ..) in &copies {
            from.entry(*g).or_default().insert(*h);
        }
        let mut out = BTreeMap::new();
        for (g, bits) in &self.tracked {
            let mut src: BTreeSet<ValueId> = BTreeSet::from([*g]);
            let mut stack = vec![*g];
            while let Some(x) = stack.pop() {
                for h in from.get(&x).into_iter().flatten() {
                    if src.insert(*h) {
                        stack.push(*h);
                    }
                }
            }
            let mut b: Option<Iv> = None;
            for (x, r) in &base {
                if src.contains(x) {
                    b = Some(join(b, *r));
                }
            }
            let b = b.unwrap_or(top(*bits));
            let (mut pos, mut neg): (Option<u128>, Option<u128>) = (Some(0), Some(0));
            let mut wraps = false;
            for (x, _, amount, up, w) in &copies {
                if !src.contains(x) {
                    continue;
                }
                let slot = if *up { &mut pos } else { &mut neg };
                *slot = slot.zip(*amount).and_then(|(p, q)| p.checked_add(q));
                wraps |= *w;
            }
            let total = |m: Option<u128>| m.and_then(|x| i128::try_from(x).ok());
            let hi = total(pos).and_then(|p| b.hi.checked_add(p));
            let lo = total(neg).and_then(|n| b.lo.checked_sub(n));
            let iv = match (lo, hi) {
                (Some(lo), Some(hi)) if fits(Iv { lo, hi }, *bits) => Iv { lo, hi },
                // Out of range: a wrapping increment can reach any value; an
                // `nsw` one cannot pass the bound without overflowing, which its
                // own obligation reports.
                _ if wraps => top(*bits),
                _ => Iv {
                    lo: lo.map_or(smin(*bits), |x| x.max(smin(*bits))),
                    hi: hi.map_or(smax(*bits), |x| x.min(smax(*bits))),
                },
            };
            out.insert(*g, iv);
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
        let alias = self.aliases(f, &BTreeSet::new()).0;
        let (guards, facts) = self.instruction_facts(f, &defs);
        let alts = edge_alternatives(f, &guards);
        let cfg = CfgFacts::of(f);
        for _ in 0..64 {
            let mut changed = false;
            for b in &f.blocks {
                for (k, i) in b.instructions.iter().enumerate() {
                    let g = Ctx {
                        defs: &defs,
                        guards: &facts[&b.id][k],
                        alias: &alias,
                        blocks: &guards,
                        alts: alts.get(&b.id).map_or(&[], Vec::as_slice),
                        cfg: &cfg,
                    };
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
                        for (k, i) in b.instructions.iter().enumerate() {
                            let g = Ctx {
                                defs: &defs,
                                guards: &facts[&b.id][k],
                                alias: &alias,
                                blocks: &guards,
                                alts: alts.get(&b.id).map_or(&[], Vec::as_slice),
                                cfg: &cfg,
                            };
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
        let fail = |what: &str| -> Res<Option<Iv>> {
            let line = i
                .span
                .as_ref()
                .and_then(|sp| crate::witness_lower::span_to_location(self.module, sp))
                .map_or(String::new(), |(_, l, _)| format!("@L{l}"));
            Err(format!("{what}-in-{}{line}", f.name))
        };
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
                    Some(gr) if !i.is_atomic_pair() => {
                        let fz = g
                            .filter(|_| self.post_only.contains(&f.id))
                            .and_then(|g| self.frozen.get(&g));
                        Some(fz.map_or(gr, |z| meet(gr, *z)))
                    }
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
        let mut x = self.refine_in(f, r, g, g.guards, v, x, 0);
        for alts in g.alts {
            let mut acc: Option<Iv> = None;
            for alt in alts {
                acc = Some(join(acc, self.refine_in(f, r, g, alt, v, x, 1)));
            }
            if let Some(a) = acc {
                x = meet(x, a);
            }
        }
        x
    }

    /// [`Self::refine`] under the facts `facts`. A fact that an `i1` phi has
    /// some value with several incoming edges still possible (`a || b` true)
    /// is a disjunction: the path came from one of them, so `v` lies in the
    /// join of its refinements under each edge's own facts (that block's
    /// guards and the incoming value, as in [`expand_fact`]).
    #[allow(clippy::too_many_arguments)]
    fn refine_in(
        &self,
        f: &AirFunction,
        r: &BTreeMap<ValueId, Iv>,
        g: &Ctx<'_>,
        facts: &[(ValueId, bool)],
        v: ValueId,
        x: Iv,
        depth: u32,
    ) -> Iv {
        let konst = |w: ValueId| match self.module.constants.get(&w) {
            Some(Constant::Int { value, .. }) => Some(*value != 0),
            _ => None,
        };
        let mut x = x;
        for &(c, pol) in facts {
            let Some(ci) = g.defs.get(&c) else { continue };
            if let Operation::Phi { incoming } = &ci.op {
                if depth >= 3 || g.cfg.header_phis.contains(&c) {
                    continue;
                }
                let Some(dst_block) = g.cfg.phis.get(&c).copied() else {
                    continue;
                };
                let live: Vec<&(BlockId, ValueId)> = incoming
                    .iter()
                    .filter(|(_, w)| konst(*w) != Some(!pol))
                    .collect();
                let mut acc: Option<Iv> = None;
                for (p, w) in live {
                    let mut alt: Vec<(ValueId, bool)> =
                        g.blocks.get(p).cloned().unwrap_or_default();
                    if konst(*w).is_none() {
                        alt.push((*w, pol));
                    }
                    if let Some(e) = g.cfg.edges.get(&(*p, dst_block)) {
                        alt.push(*e);
                    }
                    acc = Some(join(acc, self.refine_in(f, r, g, &alt, v, x, depth + 1)));
                }
                if let Some(a) = acc {
                    x = meet(x, a);
                }
                continue;
            }
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
    /// Every block's edge guards (for the disjunctive refinement in
    /// [`Prover::refine`]).
    blocks: &'b Guards,
    /// The disjunctions in force in this block ([`edge_alternatives`]).
    alts: &'b [Alternatives],
    cfg: &'b CfgFacts,
    /// Loads that read the same value as an earlier value ([`alias_dataflow`]).
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

/// Per block, the conditions in force on entry ([`edge_guards`]).
type Guards = BTreeMap<BlockId, Vec<(ValueId, bool)>>;

/// A disjunction of conjunctions of conditions ([`edge_alternatives`]).
type Alternatives = Vec<Vec<(ValueId, bool)>>;

/// Per block, per instruction, the facts in force just before it
/// ([`Prover::instruction_facts`]).
type InstrFacts = BTreeMap<BlockId, Vec<Vec<(ValueId, bool)>>>;

/// Positions in a function: block and instruction index.
type Positions = BTreeSet<(BlockId, usize)>;

/// Which globals [`alias_dataflow`] follows in a function, and how.
struct AliasScope<'b, 'm> {
    /// Single-writer globals of the function (see [`Prover::single_writer`]):
    /// no other function stores to them and the function runs at most once,
    /// so between two of its own accesses nothing changes them.
    sw: &'b BTreeSet<ValueId>,
    /// Other tracked globals: followed only inside an atomic section, where no
    /// other thread runs (`__VERIFIER_atomic_begin()` … `_end()`, or a whole
    /// `__VERIFIER_atomic_*` function); forgotten at the section's ends and at
    /// any call that might store (`inert` says which keep them).
    shared: &'b BTreeSet<ValueId>,
    inert: &'b dyn Fn(&Instruction) -> bool,
    /// A direct call's callee name, and whether it is only declared.
    callee: &'b dyn Fn(&Instruction) -> Option<(&'m str, bool)>,
}

/// Every load of a followed global (see [`AliasScope`]) that must read the
/// value of an earlier load or store in `f`, mapped to that value; and the
/// value each followed global is known to hold just before each instruction of
/// `want`. A forward dataflow carries, per global, the SSA value it is known
/// to hold (a store sets it, a load with nothing known sets it to the load
/// itself), and whether every path is inside an atomic section; a join keeps
/// a value only when every predecessor agrees. No fixpoint within the round
/// limit: no aliases.
// NOTE: one dataflow — the transfer, the join and the fixpoint read best
// together.
#[allow(clippy::type_complexity, clippy::too_many_lines)]
fn alias_dataflow(
    f: &AirFunction,
    scope: &AliasScope<'_, '_>,
    want: &Positions,
) -> (
    BTreeMap<ValueId, ValueId>,
    BTreeMap<(BlockId, usize), BTreeMap<ValueId, ValueId>>,
) {
    type St = (BTreeMap<ValueId, Option<ValueId>>, bool);
    let Some(gr) = Graph::of(f) else {
        return (BTreeMap::new(), BTreeMap::new());
    };
    let (sw, shared) = (scope.sw, scope.shared);
    let kill_shared = |st: &mut St| {
        for g in shared {
            st.0.insert(*g, None);
        }
    };
    let entry: St = (
        sw.iter().chain(shared.iter()).map(|g| (*g, None)).collect(),
        f.name.starts_with("__VERIFIER_atomic_"),
    );
    let mut out: BTreeMap<BlockId, St> = BTreeMap::new();
    let pass = |out: &BTreeMap<BlockId, St>,
                alias: &mut BTreeMap<ValueId, ValueId>,
                at: &mut BTreeMap<(BlockId, usize), BTreeMap<ValueId, ValueId>>| {
        let mut new_out: BTreeMap<BlockId, St> = BTreeMap::new();
        for b in &f.blocks {
            let mut cur: St = if b.id == gr.entry {
                entry.clone()
            } else {
                let mut acc: Option<St> = None;
                for p in &gr.preds[&b.id] {
                    let Some(po) = new_out.get(p).or_else(|| out.get(p)) else {
                        continue;
                    };
                    acc = Some(match acc {
                        None => po.clone(),
                        Some(a) => (
                            a.0.iter()
                                .map(|(g, x)| (*g, if po.0.get(g) == Some(x) { *x } else { None }))
                                .collect(),
                            a.1 && po.1,
                        ),
                    });
                }
                let Some(mut acc) = acc else { continue };
                if !acc.1 {
                    kill_shared(&mut acc);
                }
                acc
            };
            for (k, i) in b.instructions.iter().enumerate() {
                if want.contains(&(b.id, k)) {
                    at.insert(
                        (b.id, k),
                        cur.0
                            .iter()
                            .filter_map(|(g, x)| x.map(|x| (*g, x)))
                            .collect(),
                    );
                }
                let followed =
                    |g: &ValueId, cur: &St| sw.contains(g) || (cur.1 && shared.contains(g));
                match i.op {
                    Operation::Store => {
                        if let (Some(v), Some(g)) = (i.operands.first(), i.operands.get(1)) {
                            if followed(g, &cur) {
                                let v = alias.get(v).copied().unwrap_or(*v);
                                cur.0.insert(*g, Some(v));
                            } else if shared.contains(g) {
                                cur.0.insert(*g, None);
                            }
                        }
                    }
                    Operation::Load => {
                        if let (Some(g), Some(d)) = (i.operands.first(), i.dst) {
                            if followed(g, &cur) {
                                match cur.0.get(g).copied().flatten() {
                                    Some(x) => {
                                        alias.insert(d, x);
                                    }
                                    None => {
                                        cur.0.insert(*g, Some(d));
                                    }
                                }
                            }
                        }
                    }
                    Operation::CallDirect { .. } | Operation::CallIndirect { .. } => {
                        match (scope.callee)(i) {
                            Some(("__VERIFIER_atomic_begin", true)) => {
                                kill_shared(&mut cur);
                                cur.1 = true;
                            }
                            Some(("__VERIFIER_atomic_end", true)) => {
                                kill_shared(&mut cur);
                                cur.1 = false;
                            }
                            _ if (scope.inert)(i) => {}
                            _ => kill_shared(&mut cur),
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
        let next = pass(&out, &mut scratch, &mut BTreeMap::new());
        if next == out {
            // Stable: the aliases of one more pass over the fixpoint.
            let (mut alias, mut at) = (BTreeMap::new(), BTreeMap::new());
            let _ = pass(&out, &mut alias, &mut at);
            return (alias, at);
        }
        out = next;
    }
    (BTreeMap::new(), BTreeMap::new())
}

/// The fact an `assume(c)` call establishes once it returns: the `i1`
/// comparison its argument extends (`assume_abort_if_not(x < n)` passes
/// `zext (icmp slt x, n)`).
fn assume_fact(
    defs: &BTreeMap<ValueId, &Instruction>,
    assume_fns: &BTreeSet<FunctionId>,
    i: &Instruction,
) -> Option<(ValueId, bool)> {
    let Operation::CallDirect { callee } = i.op else {
        return None;
    };
    if !assume_fns.contains(&callee) {
        return None;
    }
    let a = *i.operands.first()?;
    match defs.get(&a).map(|d| (&d.op, d)) {
        Some((
            Operation::Cast {
                kind: CastKind::ZExt | CastKind::SExt,
                ..
            },
            d,
        )) => d.operands.first().map(|c| (*c, true)),
        Some((Operation::BinaryOp { kind }, _)) if int_compare(*kind).is_some() => Some((a, true)),
        _ => None,
    }
}

/// A fact plus what it implies through `i1` phis (`while (a && b)`: `phi
/// [false, %cond], [%cmp2, %rhs]`): the phi has the fact's polarity only along
/// the incoming edges whose value is not the constant opposite, so when exactly
/// one is left the path came from that block — its value held, and so did that
/// block's own guards.
fn expand_fact(
    module: &AirModule,
    defs: &BTreeMap<ValueId, &Instruction>,
    guards: &BTreeMap<BlockId, Vec<(ValueId, bool)>>,
    fact: (ValueId, bool),
) -> Vec<(ValueId, bool)> {
    let konst = |v: ValueId| match module.constants.get(&v) {
        Some(Constant::Int { value, .. }) => Some(*value != 0),
        _ => None,
    };
    let mut out = vec![fact];
    let mut k = 0;
    while k < out.len() && out.len() < 64 {
        let (c, pol) = out[k];
        k += 1;
        let Some(Operation::Phi { incoming }) = defs.get(&c).map(|i| &i.op) else {
            continue;
        };
        let live: Vec<&(BlockId, ValueId)> = incoming
            .iter()
            .filter(|(_, v)| konst(*v) != Some(!pol))
            .collect();
        let [(p, v)] = live[..] else { continue };
        let mut add = Vec::new();
        if konst(*v).is_none() {
            add.push((*v, pol));
        }
        if let Some(pg) = guards.get(p) {
            add.extend(pg.iter().copied());
        }
        for e in add {
            if !out.contains(&e) {
                out.push(e);
            }
        }
    }
    out
}

/// A function that returns only when its single argument is non-zero:
/// exactly one conditional branch, on `arg != 0` / `arg == 0` (possibly through
/// a cast), whose zero-side successor reaches no `ret` (`if (!c) abort();`).
fn is_assume_fn(module: &AirModule, f: &AirFunction) -> bool {
    let [p] = &f.params[..] else { return false };
    // No other way out before the check: no `switch`, no second branch.
    let plain = f.blocks.iter().all(|b| {
        b.instructions.last().is_some_and(|t| {
            matches!(
                t.op,
                Operation::Br { .. }
                    | Operation::CondBr { .. }
                    | Operation::Ret
                    | Operation::Unreachable
            )
        })
    });
    if !plain {
        return false;
    }
    let branches: Vec<&Instruction> = f
        .blocks
        .iter()
        .filter_map(|b| b.instructions.last())
        .filter(|t| matches!(t.op, Operation::CondBr { .. }))
        .collect();
    let [t] = branches[..] else { return false };
    let Operation::CondBr {
        then_target,
        else_target,
    } = t.op
    else {
        return false;
    };
    let defs: BTreeMap<ValueId, &Instruction> = f
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter_map(|i| i.dst.map(|d| (d, i)))
        .collect();
    let Some(c) = t.operands.first().and_then(|c| defs.get(c)) else {
        return false;
    };
    let Operation::BinaryOp { kind } = c.op else {
        return false;
    };
    let [x, y] = c.operands[..] else { return false };
    let is_zero = |v: ValueId| {
        matches!(
            module.constants.get(&v),
            Some(Constant::Int { value: 0, .. })
        )
    };
    let is_param = |v: ValueId| {
        v == p.id
            || defs.get(&v).is_some_and(|d| {
                matches!(
                    d.op,
                    Operation::Cast {
                        kind: CastKind::ZExt | CastKind::SExt | CastKind::Trunc,
                        ..
                    }
                ) && d.operands.first() == Some(&p.id)
            })
    };
    let ok_operands = (is_param(x) && is_zero(y)) || (is_param(y) && is_zero(x));
    if !ok_operands {
        return false;
    }
    let zero_side = match kind {
        BinaryOp::ICmpNe => else_target,
        BinaryOp::ICmpEq => then_target,
        _ => return false,
    };
    // No `ret` reachable from the zero side.
    let succ: BTreeMap<BlockId, Vec<BlockId>> = f
        .blocks
        .iter()
        .map(|b| {
            (
                b.id,
                b.instructions.last().map(successors).unwrap_or_default(),
            )
        })
        .collect();
    let mut seen = BTreeSet::new();
    let mut stack = vec![zero_side];
    while let Some(b) = stack.pop() {
        if !seen.insert(b) {
            continue;
        }
        let Some(blk) = f.blocks.iter().find(|x| x.id == b) else {
            continue;
        };
        if blk
            .instructions
            .iter()
            .any(|i| matches!(i.op, Operation::Ret))
        {
            return false;
        }
        stack.extend(succ.get(&b).into_iter().flatten().copied());
    }
    true
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

fn edge_guards(
    module: &AirModule,
    f: &AirFunction,
    assume_fns: &BTreeSet<FunctionId>,
) -> BTreeMap<BlockId, Vec<(ValueId, bool)>> {
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
    // An `assume(c)` call in a strict dominator: every path into the block
    // passed it, and it returned only with `c` true.
    let fdefs: BTreeMap<ValueId, &Instruction> = f
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter_map(|i| i.dst.map(|d| (d, i)))
        .collect();
    for b in ids {
        let mut facts = Vec::new();
        for d in &dom[b] {
            if d == b {
                continue;
            }
            if let Some(blk) = f.blocks.iter().find(|x| x.id == *d) {
                facts.extend(
                    blk.instructions
                        .iter()
                        .filter_map(|i| assume_fact(&fdefs, assume_fns, i)),
                );
            }
        }
        if let Some(gs) = out.get_mut(b) {
            for e in facts {
                if !gs.contains(&e) {
                    gs.push(e);
                }
            }
        }
    }
    // A branch on an `i1` phi (`while (a && b)`): see [`expand_fact`]. Two
    // rounds cover a `&&` of three.
    for _ in 0..2 {
        let snapshot = out.clone();
        for gs in out.values_mut() {
            let extra: Vec<(ValueId, bool)> = gs
                .iter()
                .flat_map(|&fct| expand_fact(module, &fdefs, &snapshot, fct))
                .collect();
            for e in extra {
                if !gs.contains(&e) {
                    gs.push(e);
                }
            }
        }
    }
    out
}

/// Per block, the disjunctions in force on entry: for every dominator `d`
/// (itself included) that several forward edges enter, one alternative per
/// predecessor `p`: `p`'s guards and the condition of the edge `p → d`. The
/// path entered `d` along one of them. Not for loop headers (a back edge's
/// facts may be about an earlier iteration's values), and not when some
/// alternative knows nothing (the disjunction would bound nothing).
fn edge_alternatives(f: &AirFunction, guards: &Guards) -> BTreeMap<BlockId, Vec<Alternatives>> {
    let Some(g) = Graph::of(f) else {
        return BTreeMap::new();
    };
    let mut at: BTreeMap<BlockId, Alternatives> = BTreeMap::new();
    for d in &g.ids {
        let ps = &g.preds[d];
        if ps.len() < 2 || ps.iter().any(|p| g.dom[p].contains(d)) {
            continue;
        }
        let mut alts: Alternatives = Vec::new();
        for p in ps {
            let mut alt = guards.get(p).cloned().unwrap_or_default();
            if let Some(t) = g.term.get(p) {
                if let (
                    Operation::CondBr {
                        then_target,
                        else_target,
                    },
                    Some(&c),
                ) = (&t.op, t.operands.first())
                {
                    if then_target != else_target {
                        alt.push((c, then_target == d));
                    }
                }
            }
            alts.push(alt);
        }
        if alts.iter().all(|a| !a.is_empty()) {
            at.insert(*d, alts);
        }
    }
    g.ids
        .iter()
        .map(|b| {
            let v: Vec<Alternatives> = g.dom[b].iter().filter_map(|d| at.get(d).cloned()).collect();
            (*b, v)
        })
        .collect()
}

/// Per-function CFG facts for [`Prover::refine_in`].
#[derive(Default)]
struct CfgFacts {
    /// The condition a conditional edge `p → s` establishes.
    edges: BTreeMap<(BlockId, BlockId), (ValueId, bool)>,
    /// Phis of loop headers: an incoming back edge's facts may be about an
    /// earlier iteration's values, so they are never split into alternatives.
    header_phis: BTreeSet<ValueId>,
    /// The block of each phi.
    phis: BTreeMap<ValueId, BlockId>,
}

impl CfgFacts {
    fn of(f: &AirFunction) -> Self {
        let Some(g) = Graph::of(f) else {
            return Self::default();
        };
        let mut out = Self::default();
        for (p, t) in &g.term {
            if let (
                Operation::CondBr {
                    then_target,
                    else_target,
                },
                Some(&c),
            ) = (&t.op, t.operands.first())
            {
                if then_target != else_target {
                    out.edges.insert((*p, *then_target), (c, true));
                    out.edges.insert((*p, *else_target), (c, false));
                }
            }
        }
        for b in &f.blocks {
            let header = g.preds[&b.id].iter().any(|p| g.dom[p].contains(&b.id));
            for i in &b.instructions {
                if let (Operation::Phi { .. }, Some(d)) = (&i.op, i.dst) {
                    out.phis.insert(d, b.id);
                    if header {
                        out.header_phis.insert(d);
                    }
                }
            }
        }
        out
    }
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

/// A stored value that is a copy of a global plus a constant: `load(h)`
/// (`c` = 0), `load(h) + c`, `c + load(h)` or `load(h) - c` (the caller checks
/// that `h` is tracked).
fn copy_of(
    module: &AirModule,
    defs: &BTreeMap<ValueId, &Instruction>,
    v: ValueId,
) -> Option<(ValueId, i128)> {
    let load_of = |w: ValueId| {
        defs.get(&w).and_then(|d| {
            (matches!(d.op, Operation::Load) && !d.is_atomic_pair())
                .then(|| d.operands.first().copied())
                .flatten()
        })
    };
    if let Some(h) = load_of(v) {
        return Some((h, 0));
    }
    let i = defs.get(&v)?;
    let Operation::BinaryOp { kind } = i.op else {
        return None;
    };
    let [x, y] = i.operands[..] else { return None };
    let konst = |w: ValueId| match module.constants.get(&w) {
        Some(Constant::Int { value, .. }) => Some(i128::from(*value)),
        _ => None,
    };
    match kind {
        BinaryOp::Add => match (load_of(x), load_of(y)) {
            (Some(h), _) if konst(y).is_some() => Some((h, konst(y)?)),
            (_, Some(h)) if konst(x).is_some() => Some((h, konst(x)?)),
            _ => None,
        },
        BinaryOp::Sub => Some((load_of(x)?, -konst(y)?)),
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
