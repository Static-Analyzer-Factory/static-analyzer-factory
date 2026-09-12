# Plan 213 — Movement 1: make SAF's own verdict trustworthy, then cut the delegation

**Status:** DESIGNED, not started. **Unblocked** — `plans/212` LANDED 2026-09-13 (score 151 → 110 → 137).
**Branch:** cut `movement1/sound-then-cut` off `movement0/scoreboard-2027`.
**Track:** TRUE-side. Movement 1 of 4. **Expected weighted effect: ~0.** It buys legitimacy and a trustworthy baseline, not points. Say so when reporting it.
**Design source:** `plans/211` §5.1(a), §5.1(c), §5.4. Memories: `saf-refine-eq-false-bug`, `saf-unsigned-refine-unsoundness`, `saf-prover-full-soundness-sweep`.

---

## 0. Goal

Discharge the §1 mandate of `plans/211`: **SAF must never delegate a verdict.** Today
`try_unreach_true` / `try_overflow_true` emit `true` only when an in-process CPAchecker
confirms, which makes those results computable by CPAchecker alone.

The gate cannot simply be deleted. It is currently **masking a real unsoundness in SAF's
own prover**, and cutting it today costs roughly **−160 weighted against a score of 137**
(the figure was computed against the pre-2027 scoreboard's 151; the sign and conclusion are
unchanged, only the denominator — see `saf-score-137-under-2027-rules`).
So this movement is 80% hardening and 20% deletion.

> ### FOLDED IN FROM MOVEMENT 0 — a second delegation, on the MEASUREMENT side
>
> This movement hunts delegation in the VERDICT. Movement 0's adversarial review found the
> same disease in the SCOREBOARD, and it inflates the number this movement will be judged
> against.
>
> **`scripts/validate_witness.sh` Stage 4 "confirms" a witness by having CBMC re-verify the
> PROGRAM. The witness is never passed to CBMC.** So a `CONFIRMED` from that stage means
> "CBMC agrees the bug is real", not "a validator accepted SAF's witness" — which is
> exactly the substitution this movement exists to eliminate, one layer out.
>
> Measured exposure on the only available sample (255 of 940 at-risk unreach-call rows):
> **−2 weighted**, losing `nla-digbench|unreach-call` and
> `recursified_loop-invariants|unreach-call`. The control flip rate was 33% (66/200). The
> memsafety half (5,405 CONFIRMED rows, 7 of whose 15 clusters are single-row) is
> **entirely unmeasured**, so −2 is a floor, not an estimate.
>
> **Add as slice 1-0, before any prover fix:** run the eval with `SAF_SKIP_CBMC=1` (or
> equivalent) over the full memsafety + unreach populations and publish the delta. Record
> WHICH validator confirmed each row in the per-task dump so this can never again be
> invisible. Until then, 137 is the correct 2027 re-score of the dump but is **not
> defensible as "SAF's honest score"**, and saying so is part of this movement's
> deliverable.
>
> **Also fold in the authoritative re-run.** 137 is a re-score of the 2026-09-11 run
> spliced with a fresh termination sweep, not a single end-to-end pass. It cannot see any
> cluster newly solved since then — notably by `d2ee9313` (the 19 GB → 244 MB OOM fix).
> This movement needs a clean baseline anyway after changing the abstract domain, so run
> the full 55,690-task eval ONCE here and let it serve both purposes. Budget ~24 h at
> `--jobs 8`; the exact reproduction command is in `plans/212` §2.

## 1. The measured exposure

`scripts/p211_sweep_soundness.py` over the COMPLETE expected-FALSE population:

| property | expected-FALSE tasks | wrong PROVE | clusters |
|---|---:|---:|---|
| unreach-call | 4,324 | **3** | `bitvector-regression` ×2, `loop-simple` ×1 |
| no-overflow | 3,733 | **4** | `signedintegeroverflow-regression` ×3, `openssl-simplified` ×1 |

Penalties pass the per-cluster dedup cap in FULL, so those 7 are worth about −160 weighted.

**This is not new information, and that matters.** `plans/210` §1 P2 found exactly the
same three unreach offenders — `bitvector-regression/signextension-1`,
`bitvector-regression/implicitunsignedconversion-1`, `loop-simple/deep-nested` — in
September 2026 and concluded "the in-process CPAchecker confirmation gate is MANDATORY."
That was a reasonable call then. Under the no-delegation mandate it is no longer
available, so **this movement fixes what plan 210 chose to mask.**

The 7 reduce to two classes:
1. **Unsound ⊥ from the interval domain** (4 tasks, 3 clusters) — §2a/§2b below.
2. **IR-invisible C semantics** (3 tasks, 1 cluster): clang constant-folds
   `(2147483647 + 1) - 23`, so no arithmetic instruction survives. **Already guarded in
   production** by `overflow_source_constant_folds` (clang `-Winteger-overflow`,
   fail-closed); only the standalone CLI lacks it. **KEEP that gate** — it is SAF's own
   clang invocation, not delegation.

## 2. The three fixes — THEY MUST LAND TOGETHER

(a) increases the number of ⊥ blocks, which increases exposure to (b). Landing (a) alone
would *raise* the wrong-TRUE rate.

### 2a. `Interval::refine_eq_false` — the recall bug

`crates/saf-analysis/src/absint/interval.rs` ~859:
```rust
if other.is_singleton() {
    if self.lo == other.lo && self.lo < self.hi { ... }   // 0 < 0 is false
    if self.hi == other.hi && self.lo < self.hi { ... }   // 0 < 0 is false
}
self.clone()                                             // should be ⊥
```
A singleton refined against an **equal** singleton — "x is known to be c, and this edge
asserts x != c" — falls through to `self.clone()` instead of `make_bottom`.
`refine_branch_condition` (`transfer.rs:1500-1507`) routes `(ICmpEq,false)` and
`(ICmpNe,true)` here, and `(ICmpEq,false)` is exactly what clang emits for the canonical
SV-COMP `if (!(cond)) { ERROR: reach_error(); }`.

**RED tests** (all four, as a table test):

| program (`int x = 0;`) | today | expected |
|---|---|---|
| `if (!(x == 0)) reach_error();` | ABSTAIN:error-reachable | **PROVE** |
| `if (x != 0) reach_error();` | ABSTAIN:error-reachable | **PROVE** |
| `if (x > 0) reach_error();` | PROVE | PROVE |
| `if (x == 1) reach_error();` | PROVE | PROVE |

This is ~3 lines of production change. **It is the single highest-recall change in the
whole plan-211 sequence**, and it is why `plans/211` §3 is marked SUPERSEDED.

### 2b. Unsigned comparison refinement — the soundness bug

`crates/saf-analysis/src/absint/transfer.rs:1510-1520`: all four unsigned arms delegate to
the **signed** refiners with no guard:
```rust
(BinaryOp::ICmpUlt, true) | (BinaryOp::ICmpUge, false) => (Some(lhs.refine_slt_true(&rhs)), None),
```
The evaluation counterpart `Interval::icmp_ult` (`interval.rs` ~734) has exactly the
guard the refinement side is missing — it delegates to `icmp_slt` only
`if self.lo >= 0 && other.lo >= 0`, else returns ⊤ `[0,1]`.

`Interval` is `{lo: i128, hi: i128, bits: u8, bottom: bool}` — **no signedness tag** — so
an unsigned value ≥ 2^31 and a negative signed value are the same domain element. When the
mis-signed refiner produces ⊥, `refine_branch_condition` returns `AbstractState::bottom()`,
declaring a REACHABLE branch infeasible, and `prove_unreachable` then sees the error block
⊥.

**Fix:** mirror the evaluation guard — refine only when both operands are provably
non-negative, otherwise no refinement.

**RED tests:** `bitvector-regression/implicitunsignedconversion-1` (`unsigned 1 < (int)-1`
is TRUE in C), `bitvector-regression/signextension-1` (sign- vs zero-extension of narrow
types), `loop-simple/deep-nested` (`uint32_max = 0xffffffff` sign-extending to −1). All
three must abstain.

**Then widen the audit.** "The interval domain has no signedness" is a *class*, not a
bug. Audit every signed/unsigned pair the same way — `icmp_*`, `div`/`rem`, shifts,
`zext`/`sext`, and the `Interval::make_top(64)` seeding at `fixpoint.rs:336-338` which
uses a 64-bit ⊤ for parameters of every width.

### 2c. Reachable universe + completeness gate

`prove_no_signed_overflow` (`checker.rs:1156-1160`) universally quantifies over **every
defined function in the module, including unreachable ones**, and has no notion of
frontend completeness. Its only structural guard is a per-instruction `indirect-call`
abstain.

Extract the gate that already exists **twice** — `race_true.rs:491-596` and
`termination.rs:114-209` — into a shared `crates/saf-svcomp/src/universe.rs`:

```rust
pub fn reachable_universe(m: &AirModule, pol: &ExternalPolicy) -> Result<Universe, String>;
```
It must: require a defined `main`; reject `llvm.global_ctors` / `global_dtors`; build the
call graph; set `reachable = main-tree ∪ ⋃ thread-entry trees`; and return `Err(reason)`
on any reachable indirect call, any reachable block whose terminator is `None` (the
frontend dropped an `indirectbr`/`callbr` and the CFG is a lie), or any reachable external
not on `pol`'s whitelist.

This is the `plans/211` §4 / `plans/201` "the AIR is lossy" defence, and it is a
**whitelist** — the only safe discipline. `ExternalPolicy` is per property; Movement 2
supplies the memsafety one.

**Open question to resolve here, not later:** `openssl-simplified/s3_srvr_1a.cil` is a
wrong PROVE whose mechanism is **hypothesised, not verified** — its arithmetic
(`s__verify_mode + 1/+2/+4`, from `__VERIFIER_nondet_int`) should abstain
`top-or-bottom-operand`, so PROVE implies the absint called those blocks unreachable and
`prove_no_signed_overflow_with_result` skipped them (`state_at_inst == None ⇒ continue`).
Confirm the mechanism before claiming the class is closed. Note the general principle:
**`None ⇒ skip` is only sound if the absint's ⊥ is trustworthy**, which §2b shows it is
not yet.

## 3. Slice D — cut the delegation

**Only after §4's gate passes at 0.**

Delete from `crates/saf-cli/src/commands.rs`: `cpachecker_confirms_no_overflow`,
`cpachecker_confirms_unreach`, and their call sites in `try_overflow_true` (~5324) and
`try_unreach_true` (~5437). **KEEP `overflow_source_constant_folds`** (~5204).

Be honest about what this does to the score: the 5 no-overflow clusters where SAF proves
but CPAchecker declines do **not** become points — they become RAW-only verdicts that
score 0, because `no-overflow` TRUE still needs a confirmed correctness witness
(`saf-validator-ceiling-true-side`: 0 of 5 confirmed). The gain here is legitimacy.

## 4. Merge gate

- [ ] `scripts/p211_sweep_soundness.py` returns **0 PROVEs** on the FULL expected-FALSE
      population for BOTH properties (4,324 + 3,733). Not a sample. This is the bar.
- [ ] Full 55,690-task authoritative run: `false_alarms == 0`, `wrong_true == 0`,
      weighted not below the post-Movement-0 baseline.
- [ ] `make lint` clean; workspace tests green.

## 5. Re-baseline before scoping Movement 3

After §2a the `error-reachable` histogram will move, possibly a lot — it is currently 70%
of unreach-call tasks and the bug fires on a three-line program. Re-run:
```
scripts/measure_prove_funnel.py --per-task lever1-pertask.jsonl --property unreach-call -n 150
scripts/measure_prove_funnel.py --per-task lever1-pertask.jsonl --property no-overflow  -n 150
scripts/p211_probe_unsolved.py  no-overflow 4 7
```
**The post-fix histogram is the only valid input for scoping `plans/215`.** Today's is not.

Also re-run `p211_probe_unsolved.py` restricted to the 8 verdict-only `unreach-call`
clusters (`array-cav19`, `array-crafted`, `array-lopstr16`, `array-multidimensional`,
`array-patterns`, `array-programs`, `floats-cbmc-regression`, `heap-data`): under the 2027
carve-out those need **no witness**, so any that PROVE after §2a are bankable immediately.
