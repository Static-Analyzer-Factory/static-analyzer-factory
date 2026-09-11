//! Feature-driven lever routing for the `unreach-call` FALSE portfolio
//! (lever `portfolio-select`).
//!
//! The `unreach-call` strategy in `saf-cli` runs a *tail* of solver levers after
//! the cheap must-reach / native-replay stages: the fixed-`k` + incremental BMC
//! engine, the forward symbolic-execution engine, the blind byte-stream fuzzer,
//! and the bit-precise CBMC oracle. Historically these ran in ONE hard-coded order
//! on every task. This module extracts a handful of **cheap, deterministic AIR
//! Booleans** once per task and turns them into an ordered [`Lever`] plan, so each
//! task is routed to a *ranked, pruned* sequence instead of the static one.
//!
//! # Two effects, both sound
//!
//! 1. **Pruning (early abstention).** A lever whose own internal gate cannot
//!    possibly fire on this task is dropped from the plan. Every applicability
//!    predicate here is a *superset* of the corresponding lever's internal gate
//!    (see [`Lever::applicable`]), so a dropped lever would have enumerated **zero
//!    candidates** anyway — the verdict is provably unchanged and only the wasted
//!    scan/solve latency is saved. Lower per-task latency is what keeps a heavy
//!    task under the eval CPU limit (a bloated per-task budget is what has
//!    regressed earlier passes by timing *other* tasks out).
//!
//! 2. **Ranking (reordering).** When a cheap feature says a task sits squarely in
//!    one lever's sweet spot, that lever is promoted to the front. The current
//!    promotion: a **non-linear guard over nondet inputs** (`x*y`, `x/y`, `x%y`,
//!    `x<<y` feeding a comparison) is exactly the class the linear/bit-vector Z3
//!    engines stall on but the blind fuzzer cracks with its dictionary, so the
//!    fuzzer runs first. Reordering never drops a lever (all applicable levers
//!    still run, just in a different order), so — like pruning — it can only ever
//!    change *latency*, never a verdict: soundness is preserved by construction.
//!
//! The default plan (no discriminating feature) is byte-identical to the historical
//! order `[Bmc, Se, Fuzz, Cbmc]`, so the common case is a zero-behaviour-change
//! refactor. The feature vector is also the substrate a later arm can learn a
//! ranking over from solved-by labels.

use saf_core::air::{AirFunction, AirModule, BinaryOp, Instruction, Operation};
use saf_core::ids::ValueId;
use std::collections::BTreeSet;

/// The solver levers that make up the `unreach-call` FALSE portfolio *tail* (the
/// stages that run after must-reach and the intraprocedural / interprocedural
/// native-replay confirmers, which are always attempted first and are not routed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lever {
    /// Fixed-`k` + incremental bounded model checking (`crate::bmc`).
    Bmc,
    /// Forward symbolic execution (`crate::se_interp`).
    Se,
    /// Blind byte-stream greybox fuzzer (`crate::fuzz`).
    Fuzz,
    /// Bit-precise CBMC oracle (`crate::cbmc`).
    Cbmc,
}

impl Lever {
    /// Whether this lever *could* fire on a task with the given features.
    ///
    /// Each arm is a **superset** of the lever's own internal gate, so
    /// `!applicable` implies the lever's gate is also unsatisfiable and it would
    /// enumerate zero candidates. Pruning a non-applicable lever is therefore
    /// verdict-preserving.
    ///
    /// - `Bmc` / `Fuzz` / `Cbmc`: all need a fuzzable nondet input to have anything
    ///   to pin/drive (BMC's per-site gate, the fuzzer's gate and
    ///   [`crate::cbmc::cbmc_precheck`] all require it).
    /// - `Se`: additionally needs a reachable loop — it only fires on a
    ///   `reach_error` function with a CFG cycle, so it is `false` on a loop-free
    ///   program.
    ///
    /// `Cbmc` is deliberately NOT gated on a loop: its SAT backend also cracks the
    /// acyclic wide-conjunction *constraint-problem* class (`xcsp`), where every
    /// cheaper lever abstains. It runs last in the plan, so the extra reach costs
    /// latency only on tasks nothing else solved.
    #[must_use]
    fn applicable(self, f: UnreachFeatures) -> bool {
        match self {
            // The blind fuzzer now also drives float/double nondet, so it applies
            // whenever ANY fuzzable nondet is present. BMC/SE/CBMC are integer
            // (bitvector) solvers with no float model, so they stay gated on an
            // integer nondet input — a float-only program yields no candidates from
            // them, so pruning them is verdict-preserving.
            Lever::Fuzz => f.fuzzable_nondet || f.float_nondet,
            Lever::Bmc | Lever::Cbmc => f.fuzzable_nondet,
            Lever::Se => f.fuzzable_nondet && f.has_loop,
        }
    }
}

/// Cheap, deterministic AIR Booleans describing an `unreach-call` task, used to
/// route it to a [`Lever`] plan. Extracted once per task in [`UnreachFeatures::extract`].
// Four independent, orthogonal routing predicates — a flat struct is the clearest
// representation; a state machine / enum would obscure them.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnreachFeatures {
    /// The program references a `__VERIFIER_nondet_*` input the byte-stream shim
    /// can drive (see [`crate::fuzz::references_scalar_nondet`]). Without it there
    /// is a single deterministic path the earlier must-reach / replay stages
    /// already cover, so every solver lever is a no-op.
    pub fuzzable_nondet: bool,
    /// The program references a float/double `__VERIFIER_nondet_*` the byte-stream
    /// shim now drives ([`crate::fuzz::references_float_nondet`]). Enables the blind
    /// fuzzer on the `floats-*` / `nla-digbench`(double) class the integer solvers
    /// cannot touch, WITHOUT enabling those (float-blind) integer solvers.
    pub float_nondet: bool,
    /// Some defined function contains a CFG back-edge (a loop). Uses the
    /// whole-module [`crate::fast_paths::module_has_any_loop`] (NOT the
    /// reachability-scoped check) so that pruning the loop-only lever (SE) is sound
    /// even when a loop is reachable only via an unresolved indirect call.
    pub has_loop: bool,
    /// A nondet-derived value flows through a *non-linear* integer operation
    /// (`Mul` of two nondet operands, or a variable divisor/remainder/shift) into
    /// a comparison guard — the Z3-hard class the fuzzer is better at.
    pub nonlinear_nondet_guard: bool,
}

impl UnreachFeatures {
    /// Extract the routing features from a module. All three probes are linear in
    /// the program size and allocation-light, so this is cheap enough to run on
    /// every task before the (far more expensive) solver tail.
    #[must_use]
    pub fn extract(module: &AirModule) -> Self {
        Self {
            fuzzable_nondet: crate::fuzz::references_scalar_nondet(module),
            float_nondet: crate::fuzz::references_float_nondet(module),
            has_loop: crate::fast_paths::module_has_any_loop(module),
            nonlinear_nondet_guard: has_nonlinear_nondet_guard(module),
        }
    }
}

/// Build the ordered, pruned lever plan for an `unreach-call` task.
///
/// The base order is the historical `[Bmc, Se, Fuzz, Cbmc]`; a non-linear nondet
/// guard promotes `Fuzz` to the front. Non-applicable levers are then dropped. The
/// result is deterministic (a pure function of `f`).
#[must_use]
pub fn plan_unreach(f: UnreachFeatures) -> Vec<Lever> {
    // Base ranking. Promote the fuzzer ahead of the Z3 engines only for the
    // non-linear-guard class they stall on; otherwise keep the cheap->expensive
    // default so the common case is unchanged.
    let base: [Lever; 4] = if f.nonlinear_nondet_guard {
        [Lever::Fuzz, Lever::Bmc, Lever::Se, Lever::Cbmc]
    } else {
        [Lever::Bmc, Lever::Se, Lever::Fuzz, Lever::Cbmc]
    };
    base.into_iter().filter(|l| l.applicable(f)).collect()
}

// ---------------------------------------------------------------------------
// Non-linear-nondet-guard detection
// ---------------------------------------------------------------------------

/// True iff some defined function contains a non-linear integer operation over
/// nondet-derived values whose result feeds a comparison guard.
///
/// "Non-linear" is deliberately narrow to keep the promotion rare (a broad
/// promotion would reorder the common case and add fuzzer latency to tasks the Z3
/// engines solve quickly):
/// - `Mul` with **both** operands nondet-tainted (`x*y`; a `nondet*const` stays
///   linear and is *not* matched),
/// - `UDiv`/`SDiv`/`URem`/`SRem` with a nondet-tainted **divisor**,
/// - `Shl`/`LShr`/`AShr` with a nondet-tainted **shift amount**.
///
/// The result must transitively reach a comparison operand, tying the non-linear
/// value to an actual branch guard rather than an incidental computation.
#[must_use]
fn has_nonlinear_nondet_guard(module: &AirModule) -> bool {
    module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .any(|f| function_has_nonlinear_nondet_guard(f, module))
}

fn function_has_nonlinear_nondet_guard(func: &AirFunction, module: &AirModule) -> bool {
    let tainted = nondet_taint(func, module);
    if tainted.is_empty() {
        return false;
    }
    let feeds_cmp = feeds_comparison(func);
    if feeds_cmp.is_empty() {
        return false;
    }
    for block in &func.blocks {
        for inst in &block.instructions {
            let Operation::BinaryOp { kind } = &inst.op else {
                continue;
            };
            let Some(dst) = inst.dst else { continue };
            if !feeds_cmp.contains(&dst) {
                continue;
            }
            let is_tainted = |i: usize| inst.operands.get(i).is_some_and(|v| tainted.contains(v));
            let nonlinear = match kind {
                // nondet * nondet — a genuine quadratic, not a scaled linear term.
                BinaryOp::Mul => is_tainted(0) && is_tainted(1),
                // variable divisor / remainder / shift amount (operand[1]).
                BinaryOp::UDiv
                | BinaryOp::SDiv
                | BinaryOp::URem
                | BinaryOp::SRem
                | BinaryOp::Shl
                | BinaryOp::LShr
                | BinaryOp::AShr => is_tainted(1),
                _ => false,
            };
            if nonlinear {
                return true;
            }
        }
    }
    false
}

/// True iff `kind` is a floating-point arithmetic or comparison op — a value the
/// integer bitvector BMC encoder cannot model (it substitutes a fresh havoc), so a
/// nondet→guard flow through it yields only spurious models that never replay.
fn is_float_op(kind: BinaryOp) -> bool {
    matches!(
        kind,
        BinaryOp::FAdd
            | BinaryOp::FSub
            | BinaryOp::FMul
            | BinaryOp::FDiv
            | BinaryOp::FRem
            | BinaryOp::FCmpOeq
            | BinaryOp::FCmpOne
            | BinaryOp::FCmpOgt
            | BinaryOp::FCmpOge
            | BinaryOp::FCmpOlt
            | BinaryOp::FCmpOle
    )
}

/// True iff the nondet→guard dataflow inside `func` is **hostile** to the
/// linear-integer bitvector BMC engines ([`crate::bmc`] fixed-k and
/// [`crate::bmc_incremental`]): some nondet-tainted value feeds a comparison guard
/// through either
///
/// - a **non-linear integer** op (a symbolic×symbolic `Mul`, or a symbolic
///   divisor/remainder/shift-amount) — where QF_BV solving stalls (bit-blasted
///   multiplication is NP-hard; unwinding a loop over it compounds the blow-up); or
/// - a **floating-point** op — which the integer bitvector encoder does not model
///   (it havocs the value), so any model it proposes is spurious and fails replay.
///
/// The BMC engines abstain on this class so their (early, cost-bounded) Z3 queries
/// stay in the fast linear-integer fragment. This is the mandatory BMC cost-gate:
///
/// - **Sound** — it only *drops* candidates. A skipped candidate can turn a FALSE
///   into `unknown`, never a wrong verdict; the native-replay gate remains the sole
///   FALSE arbiter for every candidate that *is* emitted.
/// - **Recall-preserving in practice** — the blind fuzzer (which cracks exactly these
///   non-linear / float guards with its dictionary + `CmpLog`) is routed *ahead of*
///   the BMC engines for the same class ([`has_nonlinear_nondet_guard`] /
///   [`UnreachFeatures::float_nondet`]), so it gets first crack; abstaining here only
///   stops the BMC engines from *also* grinding on a task the fuzzer already owns.
#[must_use]
pub(crate) fn nondet_guard_is_bmc_hostile(func: &AirFunction, module: &AirModule) -> bool {
    let tainted = nondet_taint(func, module);
    if tainted.is_empty() {
        return false;
    }
    let feeds_cmp = feeds_comparison(func);
    if feeds_cmp.is_empty() {
        return false;
    }
    for block in &func.blocks {
        for inst in &block.instructions {
            let Operation::BinaryOp { kind } = &inst.op else {
                continue;
            };
            let Some(dst) = inst.dst else { continue };
            if !feeds_cmp.contains(&dst) {
                continue;
            }
            let is_tainted = |i: usize| inst.operands.get(i).is_some_and(|v| tainted.contains(v));
            let hostile = match kind {
                // nondet * nondet — a genuine quadratic (a `nondet*const` scaled
                // linear term stays fast, so is NOT hostile).
                BinaryOp::Mul => is_tainted(0) && is_tainted(1),
                // variable divisor / remainder / shift amount (operand[1]).
                BinaryOp::UDiv
                | BinaryOp::SDiv
                | BinaryOp::URem
                | BinaryOp::SRem
                | BinaryOp::Shl
                | BinaryOp::LShr
                | BinaryOp::AShr => is_tainted(1),
                // Any float op over a tainted operand — unmodelled by the BV encoder.
                k if is_float_op(*k) => is_tainted(0) || is_tainted(1),
                _ => false,
            };
            if hostile {
                return true;
            }
        }
    }
    false
}

/// Value-producing operations through which integer taint propagates (SSA
/// data-flow). Memory ops (`Load`/`Store`/`Gep`) are intentionally excluded — after
/// mem2reg most values are SSA registers, and skipping memory keeps the taint (and
/// hence the promotion) tight.
fn taint_propagates(op: &Operation) -> bool {
    matches!(
        op,
        Operation::BinaryOp { .. }
            | Operation::Cast { .. }
            | Operation::Phi { .. }
            | Operation::Select
            | Operation::Freeze
    )
}

/// The set of SSA values in `func` derived from a `__VERIFIER_nondet_*` call,
/// computed by a forward fixpoint over SSA data-flow. Callee names are resolved
/// through `module` (the reliable path for a direct call's target name).
fn nondet_taint(func: &AirFunction, module: &AirModule) -> BTreeSet<ValueId> {
    let mut tainted: BTreeSet<ValueId> = BTreeSet::new();

    // Seed: results of nondet calls.
    for block in &func.blocks {
        for inst in &block.instructions {
            if is_nondet_call(inst, module) {
                if let Some(dst) = inst.dst {
                    tainted.insert(dst);
                }
            }
        }
    }
    if tainted.is_empty() {
        return tainted;
    }

    // Forward fixpoint: a value-producing op with a tainted operand is tainted.
    // Terminates because each pass either adds >=1 value or stops.
    loop {
        let mut changed = false;
        for block in &func.blocks {
            for inst in &block.instructions {
                let Some(dst) = inst.dst else { continue };
                if tainted.contains(&dst) || !taint_propagates(&inst.op) {
                    continue;
                }
                if inst.operands.iter().any(|v| tainted.contains(v)) {
                    tainted.insert(dst);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    tainted
}

/// The set of SSA values in `func` that transitively feed a comparison / branch
/// guard, computed by a backward fixpoint from every comparison operand and
/// conditional-branch / switch condition.
fn feeds_comparison(func: &AirFunction) -> BTreeSet<ValueId> {
    let mut feeds: BTreeSet<ValueId> = BTreeSet::new();

    // Seed: operands of comparison ops and branch/switch conditions.
    for block in &func.blocks {
        for inst in &block.instructions {
            let is_seed = match &inst.op {
                Operation::BinaryOp { kind } => is_comparison(*kind),
                Operation::CondBr { .. } | Operation::Switch { .. } => true,
                _ => false,
            };
            if is_seed {
                feeds.extend(inst.operands.iter().copied());
            }
        }
    }
    if feeds.is_empty() {
        return feeds;
    }

    // Backward fixpoint: if a value that feeds a comparison is *defined* by an
    // instruction, that instruction's operands also feed the comparison.
    loop {
        let mut changed = false;
        for block in &func.blocks {
            for inst in &block.instructions {
                let Some(dst) = inst.dst else { continue };
                if !feeds.contains(&dst) {
                    continue;
                }
                for v in &inst.operands {
                    if feeds.insert(*v) {
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    feeds
}

/// True iff `inst` is a direct call to a `__VERIFIER_nondet_*` function, resolving
/// the callee name through `module`.
fn is_nondet_call(inst: &Instruction, module: &AirModule) -> bool {
    let Operation::CallDirect { callee } = &inst.op else {
        return false;
    };
    module
        .function(*callee)
        .is_some_and(|f| f.name.starts_with("__VERIFIER_nondet_"))
}

/// True for the integer/float comparison `BinaryOp` kinds.
fn is_comparison(kind: BinaryOp) -> bool {
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
            | BinaryOp::FCmpOeq
            | BinaryOp::FCmpOne
            | BinaryOp::FCmpOgt
            | BinaryOp::FCmpOge
            | BinaryOp::FCmpOlt
            | BinaryOp::FCmpOle
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, Constant};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId};
    use std::collections::BTreeMap;

    // ----- plan_unreach: pruning + ordering -------------------------------

    #[test]
    fn default_plan_is_the_historical_order() {
        let f = UnreachFeatures {
            fuzzable_nondet: true,
            float_nondet: false,
            has_loop: true,
            nonlinear_nondet_guard: false,
        };
        assert_eq!(
            plan_unreach(f),
            vec![Lever::Bmc, Lever::Se, Lever::Fuzz, Lever::Cbmc]
        );
    }

    #[test]
    fn loop_free_prunes_se_but_keeps_cbmc() {
        let f = UnreachFeatures {
            fuzzable_nondet: true,
            float_nondet: false,
            has_loop: false,
            nonlinear_nondet_guard: false,
        };
        // SE needs a CFG cycle -> dropped. CBMC is a whole-program bit-precise
        // solver whose value on a loop-free program is the constraint solve, not
        // unwinding, so it is RETAINED (last, after the cheaper levers abstain).
        assert_eq!(plan_unreach(f), vec![Lever::Bmc, Lever::Fuzz, Lever::Cbmc]);
    }

    #[test]
    fn no_nondet_prunes_every_solver_lever() {
        let f = UnreachFeatures {
            fuzzable_nondet: false,
            float_nondet: false,
            has_loop: true,
            nonlinear_nondet_guard: false,
        };
        assert!(plan_unreach(f).is_empty());
    }

    #[test]
    fn float_only_nondet_enables_only_the_fuzzer() {
        // A `floats-*` task: no integer nondet, but a float nondet the shim drives.
        // Only the (now float-aware) fuzzer applies; the integer solvers are pruned.
        let f = UnreachFeatures {
            fuzzable_nondet: false,
            float_nondet: true,
            has_loop: true,
            nonlinear_nondet_guard: false,
        };
        assert_eq!(plan_unreach(f), vec![Lever::Fuzz]);
    }

    #[test]
    fn nonlinear_guard_promotes_fuzz_to_front() {
        let f = UnreachFeatures {
            fuzzable_nondet: true,
            float_nondet: false,
            has_loop: true,
            nonlinear_nondet_guard: true,
        };
        assert_eq!(
            plan_unreach(f),
            vec![Lever::Fuzz, Lever::Bmc, Lever::Se, Lever::Cbmc]
        );
    }

    #[test]
    fn nonlinear_promotion_still_prunes_loop_free_se() {
        let f = UnreachFeatures {
            fuzzable_nondet: true,
            float_nondet: false,
            has_loop: false,
            nonlinear_nondet_guard: true,
        };
        // Fuzz promoted, then BMC; SE pruned (loop-free), CBMC retained last.
        assert_eq!(plan_unreach(f), vec![Lever::Fuzz, Lever::Bmc, Lever::Cbmc]);
    }

    #[test]
    fn plan_is_deterministic() {
        let f = UnreachFeatures {
            fuzzable_nondet: true,
            float_nondet: false,
            has_loop: true,
            nonlinear_nondet_guard: true,
        };
        assert_eq!(plan_unreach(f), plan_unreach(f));
    }

    // ----- non-linear-guard detection over AIR ----------------------------

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }
    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }
    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }
    fn bid(n: u128) -> BlockId {
        BlockId::new(n)
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

    fn func(id: FunctionId, name: &str, blocks: Vec<AirBlock>) -> AirFunction {
        let entry = blocks.first().map(|b| b.id);
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks,
            entry_block: entry,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn binop(id: u128, kind: BinaryOp, a: ValueId, b: ValueId, dst: ValueId) -> Instruction {
        Instruction::new(iid(id), Operation::BinaryOp { kind })
            .with_operands(vec![a, b])
            .with_dst(dst)
    }

    /// A one-block `main` with the given instructions plus a `__VERIFIER_nondet_int`
    /// declaration (id 2). Constant `ValueId`s are recorded in `module.constants`.
    fn module_with(insts: Vec<Instruction>, consts: &[ValueId]) -> AirModule {
        let block = AirBlock {
            id: bid(1),
            label: None,
            instructions: insts,
        };
        let main = func(fid(1), "main", vec![block]);
        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(main);
        module.functions.push(decl(fid(2), "__VERIFIER_nondet_int"));
        for c in consts {
            module
                .constants
                .insert(*c, Constant::Int { value: 7, bits: 32 });
        }
        module
    }

    fn nondet(id: u128, dst: ValueId) -> Instruction {
        Instruction::new(iid(id), Operation::CallDirect { callee: fid(2) }).with_dst(dst)
    }

    #[test]
    fn detects_nondet_times_nondet_into_guard() {
        // x = nondet(); y = nondet(); p = x*y; c = (p == 49);
        let (x, y, p, c, tgt) = (vid(1), vid(2), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                nondet(1, x),
                nondet(2, y),
                binop(3, BinaryOp::Mul, x, y, p),
                binop(4, BinaryOp::ICmpEq, p, tgt, c),
            ],
            &[tgt],
        );
        assert!(has_nonlinear_nondet_guard(&m));
    }

    #[test]
    fn nondet_times_constant_is_linear_not_promoted() {
        // x = nondet(); p = x*const; c = (p == 100);  -> const operand not tainted
        let (x, k, p, c, tgt) = (vid(1), vid(9), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                nondet(1, x),
                binop(3, BinaryOp::Mul, x, k, p),
                binop(4, BinaryOp::ICmpEq, p, tgt, c),
            ],
            &[k, tgt],
        );
        assert!(!has_nonlinear_nondet_guard(&m));
    }

    #[test]
    fn variable_divisor_into_guard_is_promoted() {
        // x = nondet(); y = nondet(); q = x / y; c = (q == 3);
        let (x, y, q, c, tgt) = (vid(1), vid(2), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                nondet(1, x),
                nondet(2, y),
                binop(3, BinaryOp::SDiv, x, y, q),
                binop(4, BinaryOp::ICmpEq, q, tgt, c),
            ],
            &[tgt],
        );
        assert!(has_nonlinear_nondet_guard(&m));
    }

    #[test]
    fn nonlinear_not_feeding_a_guard_is_not_promoted() {
        // x = nondet(); y = nondet(); p = x*y;  (p never compared)
        let (x, y, p) = (vid(1), vid(2), vid(3));
        let m = module_with(
            vec![nondet(1, x), nondet(2, y), binop(3, BinaryOp::Mul, x, y, p)],
            &[],
        );
        assert!(!has_nonlinear_nondet_guard(&m));
    }

    #[test]
    fn linear_guard_over_nondet_is_not_promoted() {
        // x = nondet(); s = x + const; c = (s == 10);  -> Add is linear
        let (x, k, s, c, tgt) = (vid(1), vid(9), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                nondet(1, x),
                binop(3, BinaryOp::Add, x, k, s),
                binop(4, BinaryOp::ICmpEq, s, tgt, c),
            ],
            &[k, tgt],
        );
        assert!(!has_nonlinear_nondet_guard(&m));
    }

    #[test]
    fn no_nondet_no_promotion() {
        let (a, b, p, c, tgt) = (vid(1), vid(2), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                binop(3, BinaryOp::Mul, a, b, p),
                binop(4, BinaryOp::ICmpEq, p, tgt, c),
            ],
            &[tgt],
        );
        assert!(!has_nonlinear_nondet_guard(&m));
    }

    // ----- nondet_guard_is_bmc_hostile: BMC cost-gate ---------------------

    #[test]
    fn bmc_hostile_on_symbolic_multiply() {
        // x = nondet(); y = nondet(); p = x*y; c = (p == 49);  -> Z3-stall class.
        let (x, y, p, c, tgt) = (vid(1), vid(2), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                nondet(1, x),
                nondet(2, y),
                binop(3, BinaryOp::Mul, x, y, p),
                binop(4, BinaryOp::ICmpEq, p, tgt, c),
            ],
            &[tgt],
        );
        assert!(nondet_guard_is_bmc_hostile(m.function(fid(1)).unwrap(), &m));
    }

    #[test]
    fn bmc_hostile_on_variable_shift() {
        // x = nondet(); y = nondet(); s = x << y; c = (s == 8);
        let (x, y, s, c, tgt) = (vid(1), vid(2), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                nondet(1, x),
                nondet(2, y),
                binop(3, BinaryOp::Shl, x, y, s),
                binop(4, BinaryOp::ICmpEq, s, tgt, c),
            ],
            &[tgt],
        );
        assert!(nondet_guard_is_bmc_hostile(m.function(fid(1)).unwrap(), &m));
    }

    #[test]
    fn bmc_hostile_on_float_guard() {
        // x = nondet(); p = x *. x (float); c = (p ==. tgt) — unmodelled by the BV
        // encoder, so BMC must abstain.
        let (x, p, c, tgt) = (vid(1), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                nondet(1, x),
                binop(3, BinaryOp::FMul, x, x, p),
                binop(4, BinaryOp::FCmpOeq, p, tgt, c),
            ],
            &[tgt],
        );
        assert!(nondet_guard_is_bmc_hostile(m.function(fid(1)).unwrap(), &m));
    }

    #[test]
    fn bmc_not_hostile_on_linear_integer_guard() {
        // x = nondet(); s = x*const + const feeding a guard — BMC's fast sweet spot,
        // so NOT hostile (it must still run).
        let (x, k, mul, add, c, tgt) = (vid(1), vid(9), vid(3), vid(6), vid(4), vid(5));
        let m = module_with(
            vec![
                nondet(1, x),
                binop(3, BinaryOp::Mul, x, k, mul), // nondet * const = linear
                binop(6, BinaryOp::Add, mul, k, add),
                binop(4, BinaryOp::ICmpEq, add, tgt, c),
            ],
            &[k, tgt],
        );
        assert!(!nondet_guard_is_bmc_hostile(
            m.function(fid(1)).unwrap(),
            &m
        ));
    }

    #[test]
    fn bmc_not_hostile_without_nondet() {
        let (a, b, p, c, tgt) = (vid(1), vid(2), vid(3), vid(4), vid(5));
        let m = module_with(
            vec![
                binop(3, BinaryOp::Mul, a, b, p),
                binop(4, BinaryOp::ICmpEq, p, tgt, c),
            ],
            &[tgt],
        );
        assert!(!nondet_guard_is_bmc_hostile(
            m.function(fid(1)).unwrap(),
            &m
        ));
    }
}
