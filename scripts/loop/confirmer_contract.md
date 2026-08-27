# SAF confirmer contract — the fail-closed rules every FALSE-emitting lever MUST obey

**Status:** normative (plan 205 Movement 1d). Derived from `plans/205-loop-generalization/svcomp-compliance.md`
(evidence-backed, primary-sourced). Any loop lever that can emit `false(<property>)` — a fuzzer, a
symbolic/BMC input oracle, a memory/overflow/race confirmer, a harness synthesizer — is bound by this
contract. Breaking a rule turns a 0 (abstain) into a −16 (wrong FALSE) or −32 (wrong TRUE); the whole point
of SAF is that **abstaining always beats guessing**.

This file is part of the immutable loop harness (a worker may READ it but never edit it).

---

## 0. Fuzzing and native execution ARE allowed — this is not the risk

SV-COMP is method-agnostic. Nothing in the rules forbids any technique; they define only the input (a C
program + a specification) and the required output (a verdict + a validator-confirmed witness). **Greybox /
coverage-guided fuzzing and native execution are directly precedented and competitive** — VeriAbs/VeriFuzz
run AFL greybox fuzzing inside ReachSafety and took 1st/2nd. SAF's exact model — compile → harness the
`__VERIFIER_nondet_*` inputs → run natively under a sanitizer → confirm on the property's real violation
event → serialize the reproducing run into a validator-confirmed witness — is *architecturally identical* to
SV-COMP's own execution-based validators (`cpa-witness2test`). So **build fuzzers and native-replay oracles
freely.**

The compliance risk is NOT the technique. It is the **accountability contract**: confirm on the *right*
event, under the *right* machine model, with the *right* witness format. The seven rules below are that
contract. Each is fail-closed — when it triggers, it costs only recall (an abstain), never a wrong verdict.

The one behavioral prohibition: **no fingerprinting.** Never key a confirmer on the program name, path, hash,
task id, function name, or benchmark category. (Already a redline; it also scores 0 under the loop's
cluster-dedup and is caught by the holdout.)

---

## R1 — Match the runtime event to the property (the dominant risk). The benchmarks are NOT UB-free.

The sv-benchmarks set contains undefined behavior that is *not* the property under test: signed overflow
inside an `unreach-call` task whose expected verdict is TRUE (issue #307), division-by-zero (#504). A naive
"any sanitizer trap → FALSE" pipeline confirms off the **wrong** event → wrong FALSE (−16).

**Rule:** Confirm a FALSE ONLY on the property's exact violation event:

| property | the ONLY event that confirms a FALSE |
|---|---|
| `unreach-call` | reaching the `reach_error()` / `__assert_fail` call site (`CHECK LTL(G ! call(reach_error()))`) |
| `no-overflow` | an **in-scope signed-integer *operation*** overflow (see R2) |
| `valid-memsafety` (`valid-deref`/`valid-free`/`valid-memtrack`) | the matching ASan memory error |
| `no-data-race` | a TSan data race, under a forced schedule (see R7) |
| `termination` | non-termination evidence per the current (still-maturing) format — re-verify validator support first |

On ANY trap that is not the property under test → **abstain (UNKNOWN = 0)**. Never run a catch-all
`sanitizer → FALSE`.

## R2 — Narrow the overflow oracle to signed-integer operations (exclude conversions)

The `no-overflow` property is defined strictly on signed-integer *operations* whose result is out of range and
**explicitly excludes conversions** ("conversions to signed-integer types do not violate this property"). But
`-fsanitize=undefined` also traps on conversion truncation, shift-out-of-bounds, pointer overflow, etc. — a
trap on any of those in a TRUE task is a wrong FALSE (−16).

**Rule:** Arm UBSan narrowly — `-fsanitize=signed-integer-overflow` ONLY — and confirm ONLY on a
signed-integer-*operation* overflow. Ignore conversion / shift / pointer traps for this property.

## R3 — Compile and run under the task's DECLARED data model (ILP32 vs LP64)

Each category mandates ILP32 (32-bit) or LP64 (64-bit); analysis must use *that* model. SAF's native replay on
x86-64 defaults to LP64. Running an **ILP32** task under LP64 uses the wrong integer/pointer widths → a
computation fails to overflow/wrap as 32-bit semantics require, or reaches `reach_error()` on a path 32-bit
semantics forbid → an out-of-model FALSE the validator refuses (0) or, on a genuinely-correct program, a false
alarm (−16). This is the same "64-bit gap" SAF already recorded for R6 overflow witnesses.

**Rule:** Read the category's data model from the benchmark definition (do not hardcode); compile/run with
`-m32` for ILP32 tasks. If the required model cannot be produced (missing multilib) → **abstain**.

## R4 — Honor `__VERIFIER_assume` as a hard path filter

`__VERIFIER_assume(cond)` blocks paths (`if (!cond) { LOOP: goto LOOP; }`); it does not return a value.
Reaching the error on a path an `assume` would have killed is a spurious FALSE.

**Rule:** Treat every `__VERIFIER_assume` as a hard path filter. Discard any concrete input assignment it would
block *before* confirming a FALSE. (A symbolic/BMC oracle must add the assume as a path constraint; a fuzzer
must reject inputs the assume kills.)

## R5 — Keep nondet inputs in-model (right type / width / signedness)

`__VERIFIER_nondet_X()` returns an arbitrary value of the indicated type, no side effects. Inputs MUST be
within the declared C type range for that type/width/signedness. Undocumented nondet functions exist in the
set (`__VERIFIER_nondet_longlong`/`_charp`/`_u8`/`_u16`, issue #1304).

**Rule:** Emit only in-range values for the declared C type; get width/signedness right for every nondet
function. If a function's width/signedness is ambiguous → **abstain**, do not guess.

## R6 — Reproduce deterministically before emitting

Relying on the literal uninitialized-read nondet template at runtime yields garbage that differs across runs;
such a witness will not re-confirm (→ 0).

**Rule:** Provide real nondet implementations that inject the witness's exact concrete values, and require the
*same* inputs to re-trigger the violation deterministically *before* emitting the FALSE. Determinism is also a
SAF-wide invariant (BTreeMap/BTreeSet, stubbed `rand`). Never emit a FALSE from a solver model alone — always
end in a native replay on the **original, unsliced** program that confirms the actual event (a slice may drop
a constraint the real program enforces; slice to search, confirm on the original).

## R7 — Concurrency needs a forced schedule AND a GraphML 1.0 witness

A dynamic race/assertion detector fires only if the conflicting accesses actually run concurrently; under the
default OS schedule short programs serialize → 0 recall and no reproducing schedule to witness. And a
concurrency violation witness in **YAML 2.0 scores 0** — concurrency/`no-data-race`/termination FALSE witnesses
are **GraphML 1.0 ONLY**.

**Rule:** A concurrency FALSE must carry an explicit, deterministic interleaving that drives the replay into
the violation, serialized as a **GraphML 1.0** witness (threadId sequence + createThread edges), targeted at
CPAchecker-4.0 / Dartagnan / ConcurrentWitness2Test. Additionally **abstain on relaxed-memory** (pthread-wmm /
C11-relaxed / any dropped `#pragma omp`) — native x86 replay is TSO/SC-only, and a fail-open OpenMP probe
produces a wrong verdict (plan-202 lesson: SAF's frontend drops `#pragma omp`, so a race gate must be
fail-closed on OpenMP).

---

## Witness format targeting (per property)

- **Sequential reachability FALSE** (`unreach-call`, `no-overflow`, `valid-memsafety`): **YAML 2.0** violation
  witness → validated by **CPAchecker** and **Witch3**. Do NOT target UAutomizer for the 2.0-violation path
  (it validates 1.0-violation and 2.0-correctness only).
- **Concurrency / `no-data-race` / termination FALSE**: **GraphML 1.0** only (R7).
- Every witness is syntax-checked by **WitnessLint** first — a syntactically invalid witness is never
  confirmed. Keep witnesses small and concrete: violation-witness validation has a **90 s CPU budget**.

## Packaging (submission-time, for the operator)

No compiler is guaranteed on the competition machine — **bundle SAF's own gcc/clang toolchain and 32-bit
multilib** (for R3) in the submission archive.

---

## Pre-emit checklist (a lever author ticks every box before returning `false(<prop>)`)

1. The confirming event is the property's exact violation event (R1) — not an incidental UBSan/ASan/div-by-zero trap.
2. For `no-overflow`: UBSan armed as `signed-integer-overflow` only; the trap is on a signed-integer operation, not a conversion/shift/pointer (R2).
3. Task compiled/run under its declared ILP32/LP64 model, or abstained if unavailable (R3).
4. Every `__VERIFIER_assume` on the path is satisfied by the concrete inputs (R4).
5. All nondet inputs are in-range for their declared type/width/signedness; abstained on any ambiguous width (R5).
6. The violation re-triggers deterministically from the witness's exact injected values on the original program (R6).
7. Concurrency: an explicit forced schedule reproduces the violation and the witness is GraphML 1.0; abstained on relaxed-memory/OpenMP (R7).
8. No fingerprinting: the confirmer keys on nothing task-specific (name/path/hash/id/function/category).

If any box cannot be ticked → **return UNKNOWN (0)**. A wrong FALSE is −16; a wrong TRUE is −32; an abstain is 0.

---

## SOUNDNESS SENTINEL — full-svcomp25 FP/wrong-TRUE regressions to FIX (added 2026-08-26)

A full svcomp25 run (48,380 tasks) of the loop-enhanced SAF scored a big CONFIRMED
gain (6,161 → 9,503) **but FAILED soundness: 6 false alarms + 1 wrong-TRUE.** These
are on tasks OUTSIDE the 1,000-task sample the per-arm gate uses, so the sample's
`FP=0` gate never saw them. `tests/benchmarks/svcomp-splits/soundness-sentinel.jsonl`
(the 7 tasks + a sample of each affected cluster) is now raw-evaluated EVERY arm; an
arm that RAISES its FP+wrong-TRUE is hard-reverted (`ALERT_SENTINEL_REGRESSION`).

**These are HIGH-PRIORITY sound-first fixes. Each must be PRECISE (a targeted abstain
on the unsound case), NOT a blunt gate that tanks recall. Root causes + specs:**

1. **`fuzz-pointer-sound` (unreach-call) — the blind fuzzer confirms a `reach_error`
   reached via a nondet value cast to a POINTER.** `aws-c-common/aws_string_new_from_array_harness`
   does `(void*)__VERIFIER_nondet_ulong()` — a scalar nondet becomes an arbitrary
   pointer that is dereferenced, so the fuzzer's arbitrary bytes fabricate an invalid
   pointer and hit `reach_error` on an infeasible path. The driver already NULLs
   `__VERIFIER_nondet_pointer`, but this bypasses it via an integer→pointer cast.
   FIX (precise): in `fuzz.rs`/`fuzz_confirm_false`, ABSTAIN when a DEF-USE taint shows
   a `__VERIFIER_nondet_*` result flowing (through casts) into an `Operation::Cast{kind:
   IntToPtr}` whose result is `Load`/`Store`-dereferenced. Do NOT blunt-gate on any
   IntToPtr. (harness-havoc is the eventual recall recovery via symbolic pointers.)

2. **`overflow-nonlinear-sound` (no-overflow) — the UBSan overflow confirmer fires on
   NONLINEAR (var×var) arithmetic under arbitrary nondet inputs.** `nla-digbench/hard2`,
   `termination-crafted-lit/…ESOP2008-easy2`, `termination-numeric/twisted`: a nonlinear
   recurrence overflows only for inputs the real program's (loop-invariant) precondition
   forbids. FIX (precise): abstain the overflow confirm when the overflowing operation
   depends on a nonlinear multiply (`Operation::Binary{kind: Mul}` with BOTH operands
   non-constant) on the reaching path. Not a blunt "any multiply" gate.

3. **`termination-recursion-sound` (termination) — a wrong-TRUE on recursive heap-alloc.**
   `termination-memory-linkedlists/ll_create_rec-alloca-1`: `program_structurally_terminates`
   returned TRUE though the recursion/loop can be unbounded (heap/alloca linked-list build).
   The `(A)` acyclicity/SCC check SHOULD have abstained — so this is a DETECTION MISS.
   FIX: reproduce SAF on this task, trace why the recursion/non-termination was not caught
   (promote_module interaction? loop-vs-recursion? ranking-function synthesized wrongly),
   and abstain. Verdict-only property; abstaining is always sound.

After these land FP=0 on the sentinel, resume the normal lever rotation.

## SENTINEL UPDATE 2026-08-26 — the 3 sentinel LEVERS are RETIRED (verified clean at HEAD)

Two independent lines of evidence now agree the FP/wrong-TRUE surface these three
levers targeted is NOT present at current HEAD, so they are PARKED in levers.tsv
(they were burning arms re-litigating phantom bugs and stalling the campaign):

- **termination-recursion-sound** — the wrong-TRUE task `ll_create_rec-alloca-1` was
  rewritten signed->unsigned (FALSE->TRUE) and RELABELED upstream; SAF now correctly
  emits `true`. Abstaining would LOSE a correct point. Arm-146 verified sound across
  all 12 FALSE recursion tasks + an 84-task FALSE sweep. Nothing to fix.
- **overflow-nonlinear-sound** — the nonlinear-abstain was REVERTED twice (arm-142
  gen_dw=-5, arm-145 gen_dw=-1): it costs correct overflow detections with NO soundness
  payoff at HEAD. Net-negative.
- **fuzz-pointer-sound** — banked (ACCUMULATE). The real SV-COMP YAML validator (Witch3/
  Symbiotic-Witch, run offline on cd-vm-14, 2026-08-26) shows ZERO unreach-call false
  alarms across the full 48,380-task run; the pointer-nondet class is correct-but-
  sometimes-unconfirmed (KLEE incompleteness), not a false alarm.

IMPORTANT — soundness protection is NOT removed. The sentinel GATE stays fully active:
`tests/benchmarks/svcomp-splits/soundness-sentinel.jsonl` is raw-evaluated EVERY arm and
any arm that raises FP+wrong-TRUE is hard-reverted (ALERT_SENTINEL_REGRESSION). Retiring
the *levers* only stops the redundant fix-attempts; the *gate* still guards regressions.
Future arms: do NOT re-investigate these three. Focus on the recall frontier
(input-synthesis/last-mile + confirmation-gap).

## SENTINEL UPDATE 2026-08-27 — RETIREMENT REVERSED (fresh at-scale run PROVES the bugs are LIVE)

The 2026-08-26 retirement was WRONG. A fresh full-svcomp25 run on HEAD (48,380 tasks,
completed 2026-08-27) FAILED soundness: FP=7 + wrong-TRUE=1. FIVE of the eight violations
are EXACTLY the three sentinel targets (all -16/-32):
  - overflow-nonlinear-sound: no-overflow FP on nla-digbench/hard2,
    termination-crafted-lit/ChawdharyCookGulwaniSagivYang-ESOP2008-easy2,
    termination-numeric/twisted.
  - fuzz-pointer-sound: unreach-call FP on aws-c-common/aws_string_new_from_array_harness
    (the coverage-guided fuzzer reached reach_error via (void*)__VERIFIER_nondet_ulong()).
  - termination-recursion-sound: termination WRONG-TRUE on
    termination-memory-linkedlists/ll_create_rec-alloca-1 (expected FALSE; SAF proves TRUE).
    [arm-146 "relabeled to TRUE, nothing to fix" was a MISREAD.]
Plus THREE MORE in the same root class, not yet sentinel-named: unreach-call FP on
floats-esbmc-regression/nearbyint2, floats-esbmc-regression/rint2 (float nondet) and
recursified_nla-digbench/recursified_geo1-u (nonlinear). ROOT CLASS = SAF native-replay /
fuzz confirmation can spuriously reach reach_error when driven by nondet values the real
program semantics/precondition forbid (pointer / nonlinear-arith / float).

The three levers are UN-PARKED. The fix must be PRECISE (a targeted abstain on the genuinely
unsound case, NOT a blunt gate — the naive overflow-nonlinear abstain reverted -5/-1). The
per-arm SAMPLED (1000-task) gate does NOT see these; only a full-reservoir run does — treat
the soundness-sentinel.jsonl gate as necessary-but-not-sufficient. Do NOT re-retire without a
full-reservoir FP=0 run confirming the fix.
