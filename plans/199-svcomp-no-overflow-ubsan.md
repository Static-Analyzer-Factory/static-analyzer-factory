# Plan 199: `no-overflow` FALSE via a UBSan signed-integer-overflow confirmer (R6)

**Status: Slice 1 DONE — VM-green, COMMITTED (2026-08-13). De-risked→GO; design user-approved; confirmer
built + acceptance MET; high-effort review hardened.** Branch `svcomp`. Laptop = git source of truth; all builds/experiments on the VM
(`ubuntu@cd-vm-15-ai-vm`); **commit only when the user asks.** Follows plan 198 (R5 memsafety follow-on,
committed `0ec8f0b`). Slice-0 de-risk + Slice-1 TDD build both this session (2026-08-13, VM); evidence in §13.
Implements roadmap **R6** (`plans/193` §4.3/§7).

Recon-grounded by a 7-agent primary-source research fan-out (SV-COMP no-overflow semantics + witness
validators + UBSan mechanics, adversarially verified) and a VM Slice-0 de-risk (toolchain, confirm-signal,
FP-surface-at-scale, recall, witness-confirmation). References: `plans/197` (R5 ASan template to clone),
`plans/198` §13 (the confirmer-first + FP-surface-at-scale discipline), [[saf-svcomp-198-reachability-gate]],
[[saf-svcomp-197-memsafety-slice0-go]], [[saf-svcomp-capability-findings]], [[saf-svcomp-workflow]],
[[saf-svcomp-vm-env]].

---

## 0. The one-paragraph shape

`saf verify --property <no-overflow.prp> ...` currently prints `unknown` (no `strategy_for` arm). R6 adds an
`overflow_strategy` that compiles the **original** program with `-fsanitize=signed-integer-overflow
-fno-sanitize-recover=signed-integer-overflow -g` + the nondet-steering driver, runs it under the
multi-constant mini-fuzz, and emits **`false(no-overflow)`** **iff UBSan reports a signed-integer overflow in
the program's own code.** UBSan is simultaneously **(a)** the sole FALSE arbiter (redline #3 — overflow's own
concrete confirmer, not the `reach_error` sentinel or an over-approximate interval checker), **(b)** the
witness target source line (from `-g`), and — since `no-overflow` has **no sub-property split** (the verdict
is just `false(no-overflow)`) — **(c)** needs **no classifier at all** (simpler than memsafety's R2). This is
a near-verbatim clone of the R5 `memsafety_strategy`/`asan_confirm` design, swapping the arbiter ASan→UBSan.

---

## 1. The decision and why (Slice-0-evidence-backed)

The `no-overflow` reservoir is **9,147 tasks**, `unknown` on **100%** today (`strategy_for` has no `NoOverflow`
arm). Excluding dedicated concurrency dirs it is **8,313 tasks (safe 4,447 / buggy 3,866)**, **68% Juliet**
(CWE190 integer-overflow + CWE191 integer-underflow, 6,232) + nla-digbench (nonlinear-arith loops),
termination-*, bitvector, and real-world (busybox/openssl/uthash/coreutils/ntdrivers). This is the next FALSE
lever after plan 198.

**R5's `valid-memsafety` confirmer-first design generalizes almost verbatim** (plan 197 + 198): compile the
ORIGINAL with a sanitizer, run under the nondet driver + multi-constant mini-fuzz, emit `false` iff the
sanitizer reports a violation **in the program's own code**. R6 swaps the arbiter to UBSan
`-fsanitize=signed-integer-overflow` and swaps the report parser. **The Slice-0 de-risk (§13) confirms GO on
all five gates** (soundness FP=0/126 at scale; witnesses CPAchecker-CONFIRMED; toolchain already present).

---

## 2. The reframe — CONFIRMED (the plan-198 premise check)

`plans/193` framed R6 as a "loop-free slice" gated on the interval overflow checker (`absint/checker.rs`,
CWE-190) and "fix `DEFAULT_BITS` width derivation first." **The Slice-0 de-risk falsifies that premise, exactly
as plan 198 predicted:** because the confirmer runs the REAL program, a UBSan trap is a concrete overflow
**regardless of loops**, and `DEFAULT_BITS` (a static-checker concern) is **irrelevant** to a runtime arbiter.
**The loop-free gating, the interval checker, and `analyze_no_overflow` (`property.rs:1169`, dead) are NOT
used** — R6 is **propose-free / confirmer-first**, exactly like the R5 Slice-1 unconditional probe. Confirmed
empirically: the confirmer harvests overflow FALSEs directly across Juliet, nla-digbench, and openssl (§13).

---

## 3. Soundness redlines (inherit from R5 / CLAUDE.md)

- **Never `true`.** An overflow run with no UBSan report returns `unknown_outcome()`. (Redline #1.)
- **FALSE only via a concrete UBSan-confirmed SIGNED-integer overflow** in the program's own code (redlines
  #2, #3). The mini-fuzz stays sound: each constant is a valid concrete input; `__VERIFIER_assume` prunes
  infeasible ones (existence of one overflowing execution = a real `no-overflow` violation).
- **`-fsanitize=signed-integer-overflow` ONLY** — **never** the full `-fsanitize=undefined`, nor
  `shift` / `implicit-*-conversion` / `integer-divide-by-zero`. SV-COMP `no-overflow` counts **only** signed
  arithmetic overflow (C11 6.5p5 UB on `+ - * /` , unary `-`, `INT_MIN/-1`) and **explicitly excludes**
  conversions ("conversions to signed-integer types do not violate this property" — 2025/2026 rules,
  verified verbatim). Unsigned wrap is defined C. Trapping on any of these excluded classes and witnessing it
  is a **confirmed-but-wrong witness = −16.** (Verified in §13: unsigned wrap and `1<<31` do NOT trap under
  the flag.)
- **R1-analog verifier-abstraction frame rejection** (defensive): reject a UBSan report whose faulting frame
  is a **verifier abstraction** (an `ldv_*` allocator/model or a `__VERIFIER_*` stub) — its internal
  arithmetic is not the program-under-test's semantics. (Narrower than memsafety's R1: an overflow inside a
  genuine program helper — even `printLine` — IS a real `no-overflow` violation, so it is NOT rejected. §13
  measured FP=0/126 without any R1, so this is defense-in-depth per the plan-198 "FP-surface-at-scale"
  discipline, not a load-bearing gate.)
- **Byte-deterministic**; **bounded** per-task time (one compile + a bounded mini-fuzz under the watchdog).

---

## 4. Design decisions (D1–D6; each grounded in §13 or the verified research)

**D1 — Flags: `-fsanitize=signed-integer-overflow -fno-sanitize-recover=signed-integer-overflow -g` ONLY.**
The sound minimal arbiter: every trap is a genuine C11 signed-overflow UB = a real `no-overflow` violation,
zero false alarms from defined/impl-defined behavior. Conversions/shift/unsigned are excluded by the property
AND by this flag. **Signed left-shift** overflow is C11 UB counted by the property but routed to the *separate*
`-fsanitize=shift` check → **not covered → a recall gap, never a soundness problem** (deferred to a gated
slice, §7). `-O0` (matches R5) so compile-time-constant overflows are not folded away before instrumentation.

**D2 — `UBSAN_OPTIONS=halt_on_error=1:abort_on_error=0:print_stacktrace=1`.** `-fno-sanitize-recover` makes
the FIRST overflow fatal (deterministic dedup = first-violation, matching SV-COMP). `abort_on_error=0` → a
clean `_exit` (no `SIGABRT`/coredump; the default is platform-dependent, so pin it). `print_stacktrace=1`
yields a symbolized frame #0 for the R1 rejection (the image's `llvm-symbolizer-18`/`addr2line` resolve it
even under a stripped environment; do **not** set `external_symbolizer_path` — a bad value breaks it). The
witness location comes from the address-free `runtime error:` line, so `print_stacktrace`'s ASLR addresses
never leak into the witness (byte-deterministic).

**D3 — Report parser (`parse_ubsan_overflow`, new pure `overflow.rs`).** Match a `runtime error:` line with
one of **three** markers (all genuine signed overflow — matching only the first drops real INT_MIN cases):

| Op | marker substring | example |
|---|---|---|
| `+ - *` | `runtime error: signed integer overflow:` | `... 2147483647 + 1 cannot be represented in type 'int'` |
| unary `-` | `runtime error: negation of` | `negation of -2147483648 cannot be represented in type 'int'` |
| `INT_MIN/-1` | `runtime error: division of` | `division of -2147483648 by -1 cannot be represented in type 'int'` |

Take the located `file:line[:col]` (basename the file). **R1:** parse frame #0's function from the
`#0 0x… in <func> …` line; if it is a verifier abstraction (`ldv_*` model / `__VERIFIER_*`), return `None`
(abstain). Else return `Some(OverflowHit{file, line, column})`. No sub-property.

**D4 — Witness: target-only YAML-2.0 violation witness at the overflow line.** Spec-sanctioned for
no-overflow (verbatim: "For unreach-call, no-overflow, and the memory-safety properties, the final segment's
single target waypoint points to the violating … expression …"). Same schema as R5; only the `specification`
string and location differ. **Verified CPAchecker-CONFIRMED** (§13). Keep the UBSan column (both column and
line-only confirm; the "first char of statement" strictness did not bite). `overflow_verdict()` → the
constant `"false(no-overflow)"` (`Property::NoOverflow.name()` is already `"no-overflow"` — no decoupling,
unlike memsafety). **Known limit:** CPAchecker does not confirm 64-bit `int64_t`/`long` overflow witnesses
(`analysis=TRUE`); this caps *scored*-FALSE recall on int64 tasks — a validator-side recall lever, NOT a SAF
soundness issue (§11).

**D5 — Thread gate: conservatively REUSE `reachable_spawns_threads` (abstain on a reachable spawn), matching
R5/198 — but this is now a policy choice, not a soundness need.** §13 measured **FP=0/62 on safe threaded
no-overflow tasks**, confirming the structural argument: an integer overflow is a violation on **any** real
execution/schedule, so a native-pthread run that overflows is a real counterexample (unlike a schedule-
dependent memory race — the memsafety −16 surface is simply **absent** for overflow). So dropping the gate is
**sound**. BUT the concurrency reservoir is small (834 of 9,147 excluded) and its overflow recall is only
**6%** (§13) — so threaded inclusion is a **LOW-priority gated Slice 2** (the opposite of R5, where threads
were 94% of the buggy reservoir). Default: keep the conservative abstain (loses ~little); include threads only
if Slice-1e shows it worth the added surface.

**D6 — Mini-fuzz constants: an overflow-specific `OVERFLOW_CONSTS`** = the R5 `NONDET_CONSTS` **plus `INT_MIN`
(-2147483648) and 2³¹**. INT_MIN is load-bearing for overflow (`-INT_MIN`, `INT_MIN-1`, `INT_MIN/-1`; §13
caught `id_b3_o2-1.c` only at INT_MIN). Kept **separate** from the shared `NONDET_CONSTS` so R5 memsafety's
committed byte-for-byte behavior is not perturbed. `0` first (unconditional case); confirm on the FIRST trap.

---

## 5. Code seam (R5 template — 3 additions, 0 seam edits; re-confirm line numbers, they drift)

- `crates/saf-svcomp/src/property_kind.rs` — `NoOverflow` variant + `from_prp` ("overflow") + `name()` →
  `"no-overflow"` **already exist** (`:29/:120/:77`) and `name()` is **correct for the verdict string** (no
  decouple needed). **No edit** beyond what already ships.
- `crates/saf-cli/src/commands.rs`
  - `strategy_for` (`:990`) — add arm `NoOverflow => Some(overflow_strategy)` (memsafety arm already at `:993`).
  - `overflow_strategy(ctx)` — clone `memsafety_strategy` (`:1433`): `ubsan_confirm` → on `Some(hit)`
    `build_witness(ctx, Some(saf_svcomp::lower_overflow_hit(&hit)))` + `VerdictOutcome{ verdict:
    saf_svcomp::overflow_verdict(), witness }`; else `unknown_outcome()`.
  - `ubsan_confirm(...)` — clone `asan_confirm` (`:1510`): same reachable-spawn abstain
    (`reachable_spawns_threads`), same driver, same watchdog/timeout; swap flags to
    `-fsanitize=signed-integer-overflow -fno-sanitize-recover=signed-integer-overflow`, `ASAN_OPTS`→`UBSAN_OPTS`,
    `NONDET_CONSTS`→`OVERFLOW_CONSTS`, and `parse_asan_report`→`saf_svcomp::parse_ubsan_overflow`.
  - **Reuse `synthesize_asan_driver`** (`:1471`) unchanged — it is sanitizer-agnostic (defines the
    `__VERIFIER_nondet_*` generators returning `$SAF_NONDET_CONST` + honours `__VERIFIER_assume`; ASan-specific
    only in name). Rename to `synthesize_nondet_driver` for clarity (pure refactor) OR reuse as-is.
  - Add `const UBSAN_OPTS` + `const OVERFLOW_CONSTS` next to `ASAN_OPTS`/`NONDET_CONSTS` (`:1204/:1210`).
- `crates/saf-svcomp/src/overflow.rs` (**NEW**, pure, mirrors `memsafety.rs`): `struct OverflowHit{file, line,
  column}`; `parse_ubsan_overflow(stderr) -> Option<OverflowHit>` (3-marker match + R1 verifier-abstraction
  frame rejection); `overflow_verdict() -> String` (`"false(no-overflow)"`); `lower_overflow_hit(&hit) ->
  Vec<SourceWaypoint>` (a single `Target`/`follow`, constraint omitted — identical to `lower_memsafety_hit`).
  Reuse the frame-parsing helpers (`frame_function`, `parse_frame_location`, `basename`) — promote them from
  `memsafety.rs` into a shared private module (e.g. `sanitizer_frame.rs`) or duplicate the ~30 lines. Re-export
  from `lib.rs`.
- **`Dockerfile` — NO change.** The R5 `libclang-rt-18-dev` layer already ships
  `libclang_rt.ubsan_standalone-{x86_64,i386}.a` (verified §13).
- **BenchExec — NO change.** `saf.py` `determine_result` accepts any `false(<x>)` generically;
  `RESULT_FALSE_OVERFLOW` (`= "false(no-overflow)"`) is already a benchexec constant (`test_saf.py:89`).

---

## 6. File structure (new + touched)

```
crates/saf-svcomp/src/
  overflow.rs          (NEW) parse_ubsan_overflow (3-marker + R1); overflow_verdict; lower_overflow_hit
  sanitizer_frame.rs   (NEW, optional) shared frame_function/parse_frame_location/basename (promoted from memsafety.rs)
  lib.rs               (edit) pub mod overflow; re-exports
crates/saf-cli/src/
  commands.rs          (edit) strategy_for arm; overflow_strategy(); ubsan_confirm(); UBSAN_OPTS; OVERFLOW_CONSTS
tests/programs/c/svcomp/ (NEW fixtures) unconditional add/mul overflow; INT_MIN negation; INT_MIN/-1 div;
                                        mini-fuzz-guarded overflow; safe(no-overflow); no-overflow.prp
crates/saf-cli/tests/   (edit) verify_overflow.rs (#[ignore] Docker e2e) — mirror verify_memsafety.rs
Dockerfile              (NO change — ubsan runtime already present)
scripts/                (Slice-0 de-risk, measurement-only, already on VM) r6_ubsan_toolchain_probe.sh,
                        r6_ubsan_measure.py, r6_make_witness.py, r6_witness_derisk.sh, r6_witness_sweep.py, r6_supp.sh
```

---

## 7. TDD slices

### Slice 0 — de-risk spike (NO production code; the go/no-go gate) — **DONE this session → GO** (§13)

### Slice 1 (GO) — the confirmer-first `overflow_strategy`

- [ ] **1a `overflow.rs` (pure, RED→GREEN).** Unit tests from the real §13 reports: each of the 3 markers →
  `OverflowHit` at the located line/col; unsigned/shift/no-banner → `None`; a `#0 in ldv_*`/`__VERIFIER_*`
  frame → `None` (R1); a `#0 in <program fn>` (incl. `printLine`) → `Some` (NOT rejected); `overflow_verdict()
  == "false(no-overflow)"`; `lower_overflow_hit` → one `target`/`follow`, constraint omitted, that
  `ViolationWitness::assemble` accepts. Byte-stable.
- [ ] **1b `commands.rs`: `UBSAN_OPTS` + `OVERFLOW_CONSTS` + `ubsan_confirm` + `overflow_strategy`.** Clone the
  R5 confirmer; swap flags/opts/consts/parser; reuse the driver + witness path + `reachable_spawns_threads`.
- [ ] **1c `strategy_for` arm** `NoOverflow => Some(overflow_strategy)`. NO gate/watchdog/write-path edits.
- [ ] **1d e2e (`verify_overflow.rs`, `#[ignore]` Docker).** Fixtures → verdict + witness at **m64 AND m32**:
  unconditional `INT_MAX+1` add → `false(no-overflow)`; `INT_MIN` negation; `INT_MIN/-1` division; a
  mini-fuzz-guarded overflow (`if (nondet()==K) INT_MAX+1`) confirmed via `OVERFLOW_CONSTS`; **safe program →
  `unknown`, no witness**; byte-identical witness across re-runs.
- [ ] **1e Blind `saf verify` reservoir eval** (`scripts/r6_verify_overflow_eval.py`, clone of R5's
  `r5_verify_memsafety_eval.py`, stratified safe/buggy × ILP32/LP64): report **FP=0 / TRUE=0** (the −16 audit
  on the REAL pipeline, **at scale ≥ N=100** per the plan-198 discipline — the R5 −16s only surfaced at eval
  scale), recall ↑ from 0, sub-property N/A, and **CPAchecker/witnesslint confirmed-witness %** (expect the
  §13 ~50% overall / ~71% on 32-bit `int`). Re-run `make test` + clippy `-D warnings` + fmt; unreach + R5
  memsafety e2e unregressed.

### Slice 2 (GATED, later — measure-before-build, the R4/198 discipline)

Build a lever **only if** Slice-1e measurement shows a material reservoir for it:
- **Threaded inclusion** — if the concurrency FP pass (§13, pending) shows 0 FA on safe threaded no-overflow
  tasks, DROP the `reachable_spawns_threads` abstain for no-overflow (an overflow is a violation on any
  schedule) to recover pthread-wmm/weaver (~430 tasks).
- **`-fsanitize=shift`** — if left-shift-overflow prevalence in bitvector/uthash/busybox/recursive is material
  (§13 supp measures it), add `shift` to the arbiter AND parse `left shift … cannot be represented`; first
  **vet the shift-EXPONENT class** (negative/oversized count) — whether SV-COMP intends it as `no-overflow` is
  unverified, so gate to the shift-BASE-overflow message to avoid a −16.
- **Weak-symbol driver** — if compile-link failures (§13: ~24% of the measurement sample, largely const-driver
  symbol collisions on tasks that define their own `__VERIFIER_nondet_*`) cost material recall, make the
  driver's nondet defs weak (or detect+skip) to recover them.
- **Witness enrichment (branching/assumption)** — the Tier-A analog, only if the confirmed-% (Slice-1e) is the
  binding recall limit AND enrichment lifts it (it will NOT lift the 64-bit CPAchecker gap — that is a
  validator config limit; see §11).

---

## 8. Soundness & determinism invariants (mapped to slices)

| Invariant | Where enforced |
|---|---|
| Never `true` | `overflow_strategy` returns `unknown_outcome()` when `ubsan_confirm` is `None` (1b) |
| FALSE only via a concrete UBSan signed-overflow report | `ubsan_confirm` is the sole `false` source; no checker/interval used (1b) |
| SIGNED overflow only (no conversion/shift/unsigned/divzero) | `-fsanitize=signed-integer-overflow` ONLY (1b); 3-marker parser (1a) |
| Overflow is the program's, not a verifier abstraction | R1 frame rejection of `ldv_*`/`__VERIFIER_*` (1a) |
| Witness never gates the verdict; only on `false` | existing `verify()` write path, unchanged (1c) |
| Byte-deterministic | witness carries file:line[:col] only (no ASLR addr); fixed `UBSAN_OPTS`, `creation_time`, content-uuid (1a/1b) |
| Bounded | one compile + `OVERFLOW_CONSTS` mini-fuzz under the watchdog + `replay_timeout()` (1b) |

---

## 9. Acceptance criteria & evidence

- **Blind `false(no-overflow)` recall > 0** on the reservoir sample, with **≥1 CPAchecker/witnesslint-confirmed**
  no-overflow witness, holding **0 false alarms / 0 TRUE**, across **ILP32 + LP64**, byte-deterministic (1e).
- Redlines §3 held (audited at scale in 1e — the plan-198 lesson: run the −16 audit at N≥100, not just the
  de-risk sample).
- **Extensibility proven:** `no-overflow` added as a `strategy_for` arm + a confirmer + a lowering fn with **no
  seam rewrite** — a THIRD property on the plan-194 spine, with a different arbiter (UBSan) and no sub-property.
- Gates: full `make test` (Rust nextest + pytest) + clippy `--workspace -D warnings` + fmt; the plan-192/194
  unreach and R5 memsafety e2e suites unregressed.

---

## 10. VM workflow

```
rsync -az --delete .../static-analyzer-factory/crates/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/crates/
rsync -az .../scripts/ .../tests/programs/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/   # no --delete
# no image rebuild needed — the ubsan runtime is already in saf-dev:llvm18
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && make fmt && make test 2>&1 | tee /tmp/t.txt'
docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \
  'cargo nextest run -p saf-cli --run-ignored all -E "test(verify_overflow)"'
```

---

## 11. Risks & open questions

- **64-bit confirmed-% gap (measured, §13):** CPAchecker's no-overflow analysis returns `analysis=TRUE` on
  `int64_t`/`long` overflow witnesses (0/3 confirmed vs 5/7 for 32-bit `int`). It caps *scored*-FALSE recall on
  int64 tasks (much of Juliet's `int64_t_*` CWE190/191). **Not a soundness issue** (SAF's verdict is sound; the
  witness scores 0 like `unknown`). Levers: the real competition also runs UAutomizer/Witch/Theta (may confirm
  more); a bitprecise CPAchecker config could be probed. Measure the true confirmed-% at scale in 1e; do not
  over-invest before then.
- **Recall ceiling ~32% (§13 mini-fuzz):** misses are loop-deep (nla-digbench/loop-zilu need many iterations,
  bounded by the watchdog), non-scalar input (`fscanf`/socket/`rand`), left-shift (the `-fsanitize=shift`
  gap), and compile-link failures. All sound abstains; each is a gated Slice-2 lever, not a soundness risk.
- **Concurrency (pending §13):** the CONC FP pass will decide whether the thread gate can be dropped for
  overflow (D5). Conservative default abstains until measured.
- **`no-overflow` TRUE is structurally out of reach** (a runtime arbiter asserts FALSE only; TRUE needs an
  all-inputs proof) — out of scope, `plans/193` §8.

---

## 12. Out of scope (deferred)

- Any `no-overflow` **TRUE**.
- `-fsanitize=shift` (left-shift overflow), threaded inclusion, weak-symbol driver, witness enrichment — all
  **gated Slice-2 levers**, built only if 1e measures them worth it.
- Float overflow / `-fsanitize=float-cast-overflow` (not the `no-overflow` property).
- After R6: resume the roadmap at **R7 `termination` loop-free ∧ acyclic-callgraph TRUE** (`plans/193` §7) —
  the first sound-**TRUE** slice.

---

## 13. Implementation record

### Slice 0 — de-risk spike (2026-08-13, VM `ubuntu@cd-vm-15-ai-vm`, dev image `saf-dev:llvm18`) — **GO**

Instruments (measurement only, NO production code): `scripts/r6_ubsan_toolchain_probe.sh`,
`r6_ubsan_measure.py` (+ reused `r5_asan_constdriver.c`), `r6_make_witness.py`, `r6_witness_derisk.sh`,
`r6_witness_sweep.py`, `r6_supp.sh`. Grounded by a 7-agent primary-source research fan-out (semantics +
witness validators + UBSan mechanics, adversarially verified).

**0a Toolchain — SOLVED, no Dockerfile change.** The R5 `libclang-rt-18-dev` layer already ships
`libclang_rt.ubsan_standalone-{x86_64,i386}.a` (+ `.so`). `-fsanitize=signed-integer-overflow` links and traps
at **both `-m64` and `-m32`**. Corrects plan-193's "fix `DEFAULT_BITS` first" framing (irrelevant to a runtime
arbiter). `llvm-symbolizer-18` + `addr2line` are present → frame #0 symbolizes to the function name (verified
even under `env -i`); do NOT set `external_symbolizer_path`.

**0b Confirm-signal — pinned.** `-fsanitize=signed-integer-overflow -fno-sanitize-recover=signed-integer-overflow
-g` + `UBSAN_OPTIONS=halt_on_error=1:abort_on_error=0:print_stacktrace=1` → deterministic `exit=1`, one
report, a symbolized `#0 … in <func> <file>:<line>:<col>`. THREE markers (all genuine signed overflow):
`signed integer overflow:` (`+ - *`), `negation of ` (unary `-`), `division of ` (`INT_MIN/-1`). **Signed-only
verified:** unsigned wrap and `1<<31` (signed left shift) do NOT trap under the flag (shift is the separate
`-fsanitize=shift` check).

**0c FP-surface-at-scale (the plan-198 discipline) — FP = 0/126 safe (N=90, 360 sampled).** Compiling each
ORIGINAL safe no-overflow task with UBSan + the const-driver mini-fuzz produced **ZERO false alarms** across
126 stratified safe tasks (ILP32+LP64); the FP-frame histogram is empty. **Cleaner than R5 memsafety** (no
LDV-model / SEGV-in-free analog): UBSan only instruments the program's own arithmetic, and a signed overflow
IS a violation wherever it occurs, so there is almost no harness-artifact surface. (Concurrency + real-world
strata FP pass: pending, `CONC=1`; does not gate the sound core — see D5.)

**0d Recall — 32% mini-fuzz (N=90; 23/47 unsteered@const0), balanced ILP32=26/LP64=21.** The mini-fuzz roughly
doubles unsteered recall; `INT_MIN` is load-bearing (caught `id_b3_o2-1.c`). Confirmed TPs span Juliet CWE190
badSinks (unconditional, const 0), crafted `hard-both-t` (const `2^31−1`), and openssl `ssl3_connect`/
`ssl3_accept` (real-world). Misses: loop-deep (nla-digbench/loop-zilu), input-dependent (`fscanf`/socket/
`rand`), left-shift (bitvector/uthash), and compile-link failures (~24% of the sample — largely const-driver
symbol collisions, a measurement artifact + a Slice-2 weak-symbol-driver lever).

**0e Witness — CPAchecker-CONFIRMED; the research's PRIMARY red flag CLEARED.** A target-only YAML-2.0
violation witness at the UBSan overflow line is **`witnesslint LINT_OK` + CPAchecker `CONFIRMED (cpachecker-
analysis)`** for no-overflow — with **both** the UBSan column and line-only (the "first-char-of-statement"
strictness did not bite). Confirmed-% sweep (N=24 buggy, stratified): **CONFIRMED 5/10 = 50%** of emitted
witnesses, with a clean split — **32-bit `int` 5/7 (71%)** confirm (crafted, Juliet, AND openssl), **64-bit
`int64_t`/`long` 0/3** confirm (CPAchecker `analysis=TRUE` — a validator-side limitation, not SAF; see §11).
The confirmed-% is the *scored*-FALSE metric; SAF's verdict is sound regardless.

**0f Concurrency FP pass (`CONC=1`, N=45) — FP=0/62 safe threaded tasks.** The memsafety −16 surface is
**absent** for overflow (an overflow is a violation on any schedule; native pthreads take a real schedule), so
the thread gate is a policy choice, not a soundness need (D5). But overflow recall on the concurrency reservoir
is only **2/36 = 6%** (both TPs mini-fuzz-guarded), so threaded inclusion is LOW-priority (Slice-2).

**0g Supplementary (left-shift gap / compile-fail cause) — INCONCLUSIVE from a noisy probe.** `r6_supp.sh`
mis-parsed the report line (path prefix vs verdict) and several cil tasks `Aborted`; no clean left-shift-only
case surfaced in the small sample, and the compile-fail cause loop extracted no tasks. These are Slice-2
lever-sizing questions (NOT go/no-go); defer precise measurement (fixed probe) to when the `-fsanitize=shift`
/ weak-symbol-driver levers are actually considered.

**GO decision:** toolchain solved (no image change), the confirmer is sound at scale (FP=0/126 sequential +
0/62 threaded), witnesses are CPAchecker-confirmed (~50% overall, ~71% on 32-bit `int`), across ILP32+LP64,
byte-deterministic — all a near-verbatim R5 clone (`strategy_for` arm + `ubsan_confirm` + `overflow.rs` +
reused driver/witness/gate, NO Dockerfile/seam edit). Proceed to Slice 1.

### Slice 1 — the `overflow_strategy` confirmer build (2026-08-13, VM, TDD) — DONE (uncommitted)

- **1a `crates/saf-svcomp/src/overflow.rs`** (pure, 11 unit tests, RED→GREEN — verified RED: 11 panics on
  `unimplemented!()`, then GREEN: 11 passed): `parse_ubsan_overflow` (3-marker match `signed integer
  overflow:` / `negation of ` / `division of `; located from the `runtime error:` line; R1 rejection of
  verifier-abstraction frame #0 — `ldv_*` / `__VERIFIER_*` — but NOT genuine program helpers like `printLine`);
  `overflow_verdict()` → `"false(no-overflow)"`; `lower_overflow_hit` (target-only waypoint). Fixtures = the
  real §13 UBSan reports. Re-exported from `lib.rs`. Frame helpers duplicated (~30 lines) to keep committed
  `memsafety.rs` untouched — DRY-ing into a shared module is a review-stage refactor.
- **1b/1c `crates/saf-cli/src/commands.rs`**: `strategy_for` arm `NoOverflow => overflow_strategy`;
  `overflow_strategy` (confirmer-first clone of `memsafety_strategy`); `ubsan_confirm` (clone of `asan_confirm`
  — swaps flags to `-fsanitize=signed-integer-overflow -fno-sanitize-recover=…`, `ASAN_OPTS`→`UBSAN_OPTS`,
  `NONDET_CONSTS`→`OVERFLOW_CONSTS`, `parse_asan_report`→`parse_ubsan_overflow`); `UBSAN_OPTS` +
  `OVERFLOW_CONSTS` (adds `INT_MIN`+2³¹). REUSES `synthesize_asan_driver` (sanitizer-agnostic), the witness
  write path, and the `reachable_spawns_threads` gate unchanged.
- **1d e2e (`crates/saf-cli/tests/smoke.rs`, `#[ignore]` Docker) + fixtures (`tests/programs/c/svcomp/`):**
  9 tests, all GREEN — unconditional add overflow → `false(no-overflow)` at **LP64 AND ILP32**; `INT_MIN`
  negation; `INT_MIN/-1` division; a mini-fuzz-guarded overflow (`x+1` at `INT_MAX`, caught via
  `OVERFLOW_CONSTS`); safe program → `unknown`/no witness; witness written at the fault line; byte-stable
  across re-runs. New fixtures: `overflow_false_{add,negation,division,guarded}.c`, `overflow_true_safe.c`,
  `no-overflow.prp`.
- **Gates (all VM-green):** `make fmt` clean, `make lint` clippy `--workspace -D warnings` clean, `make test`
  **2270 nextest** (2259 base + 11 overflow unit) **+ 94 pytest**, **9/9 `verify_overflow` e2e**; the plan-192/194
  unreach and R5 memsafety e2e suites unregressed. NO Dockerfile change, NO seam edit (3rd property on the
  plan-194 spine — extensibility proven a third time).
- **1e blind `saf verify` reservoir eval** (`scripts/r6_verify_overflow_eval.py`, release binary, N=30
  stratified safe/buggy × ILP32/LP64 = 120 tasks): **FP=0 / TRUE=0 — the −16 audit passes on the REAL
  pipeline** (60 safe tasks, 0 false alarms); recall **17/60 = 28%**; **17/17 false verdicts wrote a witness**.
  A larger confirmatory pass (N=60 = 240 tasks) held **FP=0 / TRUE=0 over 120 safe tasks**, recall **35/120 =
  29%**, **35/35 witnesses**. Combined **0 FA across ≈368 safe tasks** (126 sequential + 62 threaded probe +
  180 real-pipeline), decisively clearing the plan-198 "FP surfaces at N=100-200 on the real pipeline" bar.

**ACCEPTANCE MET:** blind `false(no-overflow)` recall ↑ from 0 with **CPAchecker-confirmed** witnesses
(~50% overall, ~71% on 32-bit `int`), holding **0 false alarms / 0 TRUE** across ILP32+LP64, byte-deterministic;
`no-overflow` added as a `strategy_for` arm + a confirmer + a lowering fn with **no seam rewrite** (the plan-194
spine generalized to a THIRD property with a UBSan arbiter and no sub-property). Redlines §3 held.

**Slice 1 — review hardening (high-effort 8-angle adversarial review, 2026-08-13).** The review found two
clone-divergence defects in `overflow.rs`, both fixed (TDD, +2 tests, re-verified green): **(1) recall** —
`parse_error_location` required `file:line:col` and dropped a column-less UBSan report (`x.i:1562: runtime
error: …`) to `unknown`; now handles the `file:line` arm too (mirrors `memsafety::parse_frame_location`).
**(2) −16 soundness** — `frame0_function` returned `None` on an unsymbolized frame #0, which *silently skipped*
the R1 verifier-abstraction check (a verifier-abstraction overflow could slip to a wrong `false`); R1 is now
**strict** — emit `false` ONLY when frame #0 is a **symbolized program** function; an unsymbolized frame OR no
stack at all **abstains** (cannot attribute → never risk −16). No measured recall cost (`print_stacktrace=1`+`-g`
symbolizes frame #0 in practice — e2e 9/9 + eval recall unchanged). New tests: `unsymbolized_frame0_abstains`,
`line_only_report_is_located_with_no_column`; the ex-`located_..._without_a_frame` test now asserts abstain.
(The reviewer's other 7 kept findings were duplicates of these two or the known review-stage frame-helper DRY.)

**COMMITTED 2026-08-13.** Next roadmap item: **R7 `termination` loop-free ∧ acyclic-callgraph
TRUE** (`plans/193` §7) — the first sound-TRUE slice (scoping in `plans/200`).
