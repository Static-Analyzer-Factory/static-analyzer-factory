# SV-COMP COMPLIANCE MEMO — for the SAF team

**Date:** 2026-08-17 **Plan:** 205 (loop generalization) **Status:** decisive, evidence-backed
**Scope:** Is SAF's sound, FALSE-only, native-replay + validated-witness bug-finder inside SV-COMP's rules? Under what conditions? Which capability levers the autonomous loop is considering are COMPLIANT / COMPLIANT-WITH-CONDITIONS / AVOID?

**One-line answer:** Yes — fuzzing and native execution are unambiguously permitted, and SAF's exact architecture (compile → harness the nondet inputs → run natively → confirm on the property's real violation event → emit a validator-confirmed witness) is a *blessed* SV-COMP methodology with direct in-competition precedent (VeriAbs/VeriFuzz). Compliance is **conditional, not free**: the binding constraints are on accountability (a confirmed witness), on machine model (ILP32 vs LP64), and on matching the observed runtime event to the exact property. Get those wrong and you eat −16/−32. Every condition below is a fail-closed rule that costs only recall when it triggers, never a wrong verdict.

---

## 1. Does SV-COMP allow fuzzing? — YES. Decisive.

**Verdict: YES, without qualification on technique.** SV-COMP is fully method-agnostic. Nothing in the 2024 or 2025 rules prescribes or forbids any verification technique; the rules define only the *input* (a verification task = a C program + a specification) and the required *output* (an answer + a witness).

**Exact basis (primary sources):**

- A verification run is defined by *execution*, not method: "A verification run is a non-interactive execution of a competition candidate on a single verification task." — https://sv-comp.sosy-lab.org/2025/rules.php
- The task license must permit running the program **for the purpose of finding a bug**: the license must allow one to "compile and execute the program (in particular, for the purpose of verifying that a specification violation exists)." — SV-COMP 2025 report §3, https://www.sosy-lab.org/research/pub/2025-TACAS.Improvements_in_Software_Verification_and_Witness_Validation_SV-COMP_2025.pdf . This is the closest the rules come to addressing "running the program under test," and it is *permissive* and names SAF's exact use case.
- The report welcomes "imprecise verification techniques (e.g., via machine learning)," made scoreable through witness validation. Table 8 ("Algorithms and techniques used") enumerates Symbolic Execution, Bounded Model Checking, Evolutionary Algorithms, Portfolio, etc. as coexisting, all-permitted techniques with no privileged one. — same PDF.
- **In-competition precedent for fuzzing + native replay:** VeriAbs / VeriAbsL run **AFL greybox fuzzing** inside ReachSafety (SV-COMP proper) and took **1st/2nd in ReachSafety**; VeriFuzz is a program-aware AFL greybox fuzzer that **compiles and natively executes** the program and emits an "Error-witness Automaton" from the crashing input. — VeriAbs SV-COMP 2018 (https://link.springer.com/content/pdf/10.1007/978-3-319-89963-3_32.pdf); VeriFuzz talk (https://sv-comp.sosy-lab.org/2019/talks/36_VeriFuzz.pdf).
- **The one behavioral restriction is anti-fingerprinting, not anti-execution:** "The verifier should not use identifiers contained in the verification task to fingerprint and identify individual tasks" / "Verifiers are forbidden from using the program name, its hash, or the current category to tune their parameters." — https://sv-comp.sosy-lab.org/2024/rules.php , https://sv-comp.sosy-lab.org/2025/rules.php .

**Adversarial cross-check:** all three verdicts agree — none refutes the core "fuzzing/native execution is permitted." Two mark the claim `conditional` (the conditions are in §2–§3 below); one marks it `refuted: false` (technique permitted, conditions load-bearing). There is *no* verdict claiming fuzzing is disallowed.

---

## 2. Under WHAT CONDITIONS — the accountability & semantics contract

Technique is free; **accountability is not.** Five conditions bind a FALSE-only tool. Each is a hard requirement with a citation.

### 2.1 A scored FALSE requires an *independently confirmed* violation witness (+1)

A FALSE only earns +1 if it agrees with the expected result **and** its violation witness is confirmed by at least one independent validator. "The result is counted as correct only if at least one validator successfully validated it." A violation witness is "confirmed by False and refuted by True" by the validator. An agreeing FALSE with an *unconfirmed* witness scores **0** ("correct-unconfirmed"), not +1. — https://sv-comp.sosy-lab.org/2025/rules.php ; report Table 2 + "Scoring Schema."

Scoring (report Table 2, since SV-COMP 2021):

| Result | Points | Meaning |
|---|---|---|
| True correct | +2 | reported TRUE, correctness witness confirmed **or not required** |
| **True incorrect** | **−32** | wrong proof (unsound TRUE) |
| **False correct** | **+1** | violation found AND violation witness confirmed |
| **False incorrect** | **−16** | false alarm (property actually holds) |
| UNKNOWN / abstain | 0 | — |

The asymmetry (−32 wrong-TRUE vs −16 wrong-FALSE vs 0 abstain) *mathematically rewards* SAF's fail-closed/abstain design. Abstaining is always strictly better than guessing.

### 2.2 Witness format & the correct validator per property

- Two formats are accepted: **1.0 (GraphML)** and **2.0 (YAML)**; a tool may emit both, only one need validate.
- **Sequential reachability FALSE (unreach-call, no-overflow, memsafety):** YAML 2.0 violation witness → validated by **CPAchecker** and **Witch3**. Note: **UAutomizer does NOT validate 2.0-violation** (it does 1.0-violation and 2.0-correctness) — do not target it for the 2.0 path. — report Table 6; verified memory `svcomp-yaml-witness-2.0-format`.
- **Concurrency / no-data-race / termination FALSE: GraphML 1.0 ONLY.** ConcurrencySafety-* violation witnesses show a v1.0 checkmark and "–" for v2.0 in report Table 1. A concurrency FALSE emitted as YAML 2.0 scores **0**. Working concurrency violation validators: **CPAchecker Validator 4.0, Dartagnan, ConcurrentWitness2Test** (the last is execution/harness-based and the natural fit for SAF's replayed schedule). — report Table 1 & Table 6; plan-205 `conc-term-mechanisms.md` §0.
- Every witness is first syntax-checked by **WitnessLint**; a syntactically invalid witness "is never considered as confirmed."

### 2.3 Respect `__VERIFIER_nondet_*` / `__VERIFIER_assume` / `reach_error` semantics

The harness that supplies concrete inputs MUST honor the SV-COMP C semantics, or a "confirmed" FALSE is out-of-model and unsound:

- `__VERIFIER_nondet_X()` returns "an arbitrary value of the indicated type … (no side effects)"; template `X val; return val;`. **Inputs must be within the declared C type range for that type/width/signedness.** Out-of-range inputs are out-of-model. — https://sv-comp.sosy-lab.org/2024/rules.php .
- `__VERIFIER_assume(cond)` is a **hard path filter** ("if (!expression) { LOOP: goto LOOP; }") — it blocks paths, it does not return a value. Any concrete assignment that an `assume` would have killed must be discarded; reaching `reach_error()` on an assumed-away path is a spurious FALSE. — same page.
- `reach_error()` (historically `__VERIFIER_error()`) marks the violation site for unreach-call: `CHECK( init(main()), LTL(G ! call(reach_error())) )`. The unreach-call violation event is *reaching that call site*, nothing else. — same page.
- Undocumented nondet functions exist in the benchmark set (`__VERIFIER_nondet_longlong/_charp/_u8/_u16`, sv-benchmarks issue #1304). If width/signedness is ambiguous, **abstain**, do not guess.

### 2.4 Machine data model is a per-category MANDATE (ILP32 vs LP64) — the sharpest condition

This is the condition all three adversarial verdicts converge on as load-bearing and under-weighted. The rules "specify whether the programs are written for an ILP32 (32-bit) or an LP64 (64-bit) architecture" and require analysis under **that** model; "if the verifier has a parameter to handle this aspect, it needs to be defined." — https://sv-comp.sosy-lab.org/2025/rules.php .

SAF's native replay on x86-64 Linux defaults to **LP64** (64-bit `long`/pointer). For an **ILP32** category task, running under LP64 uses the wrong integer/pointer widths: a computation may (a) fail to overflow/wrap the way 32-bit semantics require, or (b) reach `reach_error()` on a path the intended 32-bit semantics forbid. Result: an out-of-model FALSE a 32-bit validator refuses to confirm (→0), or — against a genuinely-correct program — a false alarm (**−16**). This is the same "64-bit gap" SAF already recorded for R6 overflow witnesses (memory `saf-svcomp-199-r6-overflow-derisk-go`). **Fix: compile/execute each task under its category-declared data model (`-m32` for ILP32).** The data model is *dictated*, not chosen — SAF must read it from the category, not pick freely.

### 2.5 Resource limits & self-contained archive

- Per-verifier resource limits are enforced by BenchExec/BenchCloud. Validator runs: 2 CPU, 7 GB memory, **90 s CPU for violation witnesses**, 15 min for correctness witnesses. — report "Computing Resources." SAF's witnesses must validate inside 90 s (favor small, concrete, replayable witnesses).
- **No compiler is guaranteed on the competition machine**; "all necessary libraries and external tools should be contained in the archive." SAF must **bundle its own gcc/clang toolchain** (and any 32-bit multilib it needs for §2.4) in the submission archive. — https://sv-comp.sosy-lab.org/2025/rules.php .

---

## 3. Does SAF's native-replay + witness model comply? — YES, with fail-closed rules

**Verdict: COMPLIANT.** SAF's model — compile the program, synthesize a harness that replaces the `__VERIFIER_nondet_*` functions, pick concrete inputs, run natively under ASan/UBSan/TSan, treat the property's actual violation event as FALSE, and serialize the reproducing run into a validator-confirmed witness — is *architecturally identical* to the blessed execution-based validators `cpa-witness2test` / `FShell-witness2test`, and to VeriAbs's "replay the crash on an instrumented program to record the nondet valuations → emit the witness." — TAP 2018 "Tests from Witnesses" (https://www.sosy-lab.org/research/pub/2018-TAP.Tests_from_Witnesses_Execution-Based_Validation_of_Verification_Results.pdf); VeriAbs SV-COMP 2018.

### Where it is SAFE (already compliant)
- **FALSE-only for safety properties.** SAF never emits TRUE from execution for unreach-call / valid-memsafety / no-overflow. One passing run cannot cover the unbounded input/loop space; execution is a FALSE-only oracle. (SAF's structural termination-TRUE and R5/R6 verdict-only TRUEs are separate and out of scope here.) Correct.
- **Confirm on a concrete, deterministic reproduction.** SAF emits `false(<prop>)` only on a concrete replay whose exact inputs are encoded in the witness — precisely what makes a witness confirmable. Correct.
- **The +2/−16/−32/0 asymmetry rewards abstention.** SAF's whole design is fail-closed → abstain (0) beats any wrong guess. Correct.
- **Format targeting is correct:** YAML 2.0 for sequential FALSE (CPAchecker/Witch3), GraphML 1.0 for concurrency/termination FALSE (CPAchecker-4.0/Dartagnan/ConcurrentWitness2Test).

### The PITFALLS that produce a WRONG verdict — and the fail-closed rule for each

These are the −16/−32 generators surfaced by the semantics analysis (dossier §D) and the adversarial verdicts. Each has a hard rule.

**P1 — Wrong-event confirmation (the dominant risk). The benchmark set is NOT UB-free.**
Maintainers confirm sv-benchmarks contain undefined behavior: signed overflow inside an *unreach-call* task (`jain_4_true-unreach-call.c` traps "signed integer overflow: 4 * 2147483647" under UBSan, yet its expected verdict is TRUE — issue #307) and division-by-zero (#504). A naive "any sanitizer trap → FALSE" pipeline confirms a FALSE off the *wrong* runtime event → wrong FALSE (−16) or, on a TRUE task, potentially worse.
> **Rule R1 (match event to property):** Confirm a FALSE ONLY on the property's exact violation event — `reach_error()`/`__assert_fail` call site for unreach-call; an **in-scope signed-integer-overflow** for no-overflow; a memory error (ASan) for valid-memsafety; a data race (TSan, under a forced schedule) for no-data-race. On ANY trap that is not the property under test, **abstain (UNKNOWN=0)**. Never run a catch-all sanitizer→FALSE.

**P2 — UBSan is BROADER than the no-overflow property.**
The no-overflow property is defined strictly on signed-integer *operations* whose result is out of range, and **explicitly excludes conversions**: "conversions to signed-integer types do not violate this property." But `-fsanitize=undefined` also traps on conversion truncation, shift-out-of-bounds, pointer overflow, etc. A UBSan trap on a *conversion* in a no-overflow task whose expected verdict is TRUE = wrong FALSE (−16). — https://sv-comp.sosy-lab.org/2025/rules.php .
> **Rule R2 (narrow the overflow oracle):** For no-overflow, arm UBSan narrowly (`-fsanitize=signed-integer-overflow` only) and confirm ONLY on a signed-integer-*operation* overflow. Ignore conversion/shift/pointer traps for this property. (SAF already narrowed CWE761 memsafety R1 similarly — memory plan-203.)

**P3 — ILP32/LP64 data-model mismatch (see §2.4).**
Native LP64 replay of an ILP32 task flips overflow/wrap/pointer behavior → out-of-model FALSE (0 or −16).
> **Rule R3 (compile to the declared model):** Read the category's data model; compile/run with `-m32` for ILP32 tasks. If the required model cannot be produced (missing multilib), **abstain**.

**P4 — `__VERIFIER_assume` ignored → spurious FALSE.**
Reaching the error on a path an `assume` would have killed is a false alarm.
> **Rule R4 (honor assume):** Treat `__VERIFIER_assume` as a hard path filter; discard any input assignment it would block before confirming.

**P5 — Out-of-model / wrong-width nondet inputs.**
Wrong type/width/signedness (esp. undocumented nondet fns, #1304) → out-of-model inputs → unconfirmable or wrong FALSE.
> **Rule R5 (in-model inputs):** Emit only in-range values for the declared C type; get width/signedness right for every nondet fn. If ambiguous, **abstain**.

**P6 — Non-deterministic / non-reproducible witness.**
Relying on the literal uninitialized-read nondet template at runtime yields garbage that differs across runs → the witness won't re-confirm.
> **Rule R6 (deterministic reproduction):** Provide real nondet implementations that inject the witness's exact concrete values; require the same inputs to re-trigger the violation deterministically before emitting.

**P7 — Concurrency FALSE without a forced schedule / wrong witness format.**
A dynamic race/assertion detector fires only if the conflicting accesses actually run concurrently under the replayed schedule; under the default OS schedule short programs serialize → 0 recall AND no reproducing schedule to witness. And a concurrency witness in YAML 2.0 scores 0.
> **Rule R7 (concurrency needs a directed schedule + GraphML):** A concurrency FALSE must carry an explicit, deterministic interleaving that drives the replay into the violation, serialized as a **GraphML 1.0** witness (threadId/createThread), targeted at CPAchecker-4.0/Dartagnan/ConcurrentWitness2Test. Additionally: **abstain on relaxed-memory** (pthread-wmm / C11-relaxed / dropped `#pragma omp`) — native x86 replay is TSO/SC-only; a fail-open OpenMP probe would produce a wrong verdict (plan-202 lesson).

---

## 4. Lever guidance — capability techniques SAF's loop is considering

Marking each **COMPLIANT** / **COMPLIANT-WITH-CONDITIONS** / **AVOID** for a *sound, FALSE-only* tool. "AVOID" here means "not for a FALSE-only bug-finder," not "forbidden by SV-COMP" — SV-COMP permits all of these; the question is whether they help a FALSE-only tool score without risking a wrong verdict.

| Technique | Verdict | Reason |
|---|---|---|
| **Greybox / coverage-guided fuzzing** (AFL/libFuzzer-style, native) | **COMPLIANT** | Directly precedented and competitive: VeriAbs/VeriFuzz do exactly this in ReachSafety (1st/2nd). It is a FALSE-only oracle by nature (a crash is a real reproduction). Conditions: obey R1–R6 (match event to property, in-model inputs, data model, deterministic replay, emit confirmed witness). This is SAF's core lever. |
| **Concolic / symbolic execution with Z3** | **COMPLIANT-WITH-CONDITIONS** | Permitted and used (Symbiotic/KLEE). For a FALSE-only tool, use symbolic reasoning to *pick the concrete nondet inputs* that reach the error, then **replay natively** and confirm on the real event (Symbiotic's own model: solve on sliced bitcode, "replay it on the unsliced code" to produce the witness). Conditions: the solver's model must be decoded to concrete, in-range inputs (R5) that reproduce deterministically (R6); never emit FALSE from a solver model alone without native replay confirmation (R1). Z3 path constraints must respect `__VERIFIER_assume` (R4). Higher build cost than fuzzing; use it where fuzzing stalls (deep/rare branches). |
| **Bounded model checking (BMC)** | **COMPLIANT-WITH-CONDITIONS** | Permitted and mainstream (CBMC/ESBMC). BMC is *unbounded-sound only for TRUE up to the bound* — irrelevant to a FALSE-only tool. Use BMC purely as a **FALSE finder**: a SAT model is a concrete counterexample. Critical condition for concurrency (plan-205 A1 lazy sequentialization): the model must fix an **explicit schedule** (`rl` matrix) that SAF **re-confirms by native replay** and serializes to a GraphML witness (R7). Never emit a TRUE from "no counterexample within the bound" — that is an unsound proof (−32). BMC's under-approximation (shrinking K/U/L) costs only recall, never a spurious FALSE — matches SAF's doctrine. |
| **Harness / nondet synthesis** (build the `__VERIFIER_nondet_*` bodies + drive inputs) | **COMPLIANT-WITH-CONDITIONS** | This is the blessed execution-validator mechanism itself (TAP 2018: replace all `__VERIFIER_nondet`-prefixed functions, compile, run in a container). It is *required* for native replay. Conditions are exactly R3–R6: correct type/width/signedness incl. undocumented fns (abstain if ambiguous), honor `__VERIFIER_assume`, declared data model, deterministic injection. The single most semantics-sensitive lever — most −16/−32 risk lives here, so the fail-closed rules are non-negotiable. |
| **Program slicing** | **COMPLIANT** | Semantics-preserving reduction to shrink the search/replay target (Symbiotic slices before KLEE, then replays on the *unsliced* program). For a FALSE-only tool it is purely an accelerator with no verdict risk **provided** the final confirmation + witness are produced against the **original, unsliced** program (Symbiotic's discipline). Condition: never confirm/emit a witness against the sliced program — a slice can drop a constraint that the real program enforces. Low risk, good ROI as a pre-pass under fuzzing/BMC. |

**Cross-cutting condition for every lever:** each must terminate in a **native replay on the original program** that confirms the property's *actual* violation event and produces a **validator-confirmed witness** (YAML 2.0 for sequential, GraphML 1.0 for concurrency/termination), within the 90 s validator budget, under the declared data model. Any lever that can only produce a yes/no (not a reproducing input/schedule) is insufficient alone and must be paired with a concrete-input/schedule extractor.

---

## 5. Net for plan-205 mechanism levers — bottom line

SV-COMP is method-agnostic and explicitly blesses compile-and-execute-to-find-a-bug, so **every mechanism plan-205 is considering — greybox fuzzing, concolic/Z3, BMC-driven lazy sequentialization, harness/nondet synthesis, and slicing — is inside the rules**, and the flagship concurrency lever (A1 lazy-CSeq: BMC that returns an explicit `rl` schedule SAF replays and serializes to GraphML) is the archetypal compliant design: an under-approximation that costs only recall, never a spurious FALSE, terminating in a native-replay-confirmed witness. The compliance risk is **not** the techniques; it is the accountability contract SAF must honor on *every* FALSE: confirm on the property's *actual* violation event (never an incidental UBSan/ASan trap — the benchmarks are not UB-free, issues #307/#504), narrow the overflow oracle to signed-integer *operations* excluding conversions, compile each task under its *declared* ILP32/LP64 data model (the sharpest under-weighted condition, and the same 64-bit gap SAF already hit in R6), honor `__VERIFIER_assume` and keep nondet inputs in-model (abstain on ambiguous/undocumented widths), reproduce deterministically, emit the right-format validator-confirmed witness (GraphML 1.0 for concurrency, and pair it with an explicit forced schedule since default-schedule replay yields 0 race recall), abstain on relaxed-memory, and bundle SAF's own toolchain in the archive. Do that, and SAF's FALSE-only native-replay model is not merely permitted but the exact pattern SV-COMP's own execution-based validators embody — with the abstain-beats-guess scoring (−32/−16/0) rewarding SAF's fail-closed design at every one of the pitfalls above.

---

## Honest uncertainty flags

- **VeriAbs AFL+witness quotes** are from the SV-COMP 2018 tool-paper *body* as surfaced by search of the Springer PDF (paywalled HTML abstract did not expose the body). Internally consistent across multiple VeriAbs papers; high-confidence but re-verifiable against that PDF if an exact page is needed for a citation.
- **No primary-source sentence globally asserts "ReachSafety programs contain no UB."** The opposite is documented (issues #307/#504/#1304). Treat any secondary summary claiming a UB-free benchmark set as false; the fail-closed R1/R2 rules exist precisely because it is not.
- **Data-model wording** ("ILP32/LP64 specified per category") is verbatim in the 2025 rules, but the rules do not enumerate *which* categories are ILP32 vs LP64 in one table — SAF must read the per-category `.cfg`/benchmark-def machine-model setting at task time rather than hardcode. Flagged as an engineering dependency, not a rules ambiguity.
- **Termination FALSE witnesses** were still maturing (non-termination witness format 2.1 was a small pilot in the 2026 cycle per plan-205 §0); if SAF pursues termination FALSE, re-verify current validator support before relying on +1. Termination TRUE is verdict-only (+2, no witness) and unaffected.

## Sources
- SV-COMP 2025 Rules: https://sv-comp.sosy-lab.org/2025/rules.php
- SV-COMP 2024 Rules (nondet/assume/reach_error semantics): https://sv-comp.sosy-lab.org/2024/rules.php
- Beyer & Strejček, "Improvements in Software Verification and Witness Validation: SV-COMP 2025" (TACAS 2025): https://www.sosy-lab.org/research/pub/2025-TACAS.Improvements_in_Software_Verification_and_Witness_Validation_SV-COMP_2025.pdf (DOI https://doi.org/10.1007/978-3-031-90660-2_9)
- Beyer et al., "Tests from Witnesses" (TAP 2018, execution-based validation): https://www.sosy-lab.org/research/pub/2018-TAP.Tests_from_Witnesses_Execution-Based_Validation_of_Verification_Results.pdf
- Beyer & Lemberger, "Six Years Later: Testing vs. Model Checking" (STTT 2024): https://www.sosy-lab.org/research/pub/2024-STTT.Six_Years_Later_Testing_vs_Model_Checking.pdf
- VeriAbs SV-COMP 2018: https://link.springer.com/content/pdf/10.1007/978-3-319-89963-3_32.pdf
- VeriFuzz talk (TOOLympics/TACAS 2019): https://sv-comp.sosy-lab.org/2019/talks/36_VeriFuzz.pdf
- Symbiotic 9 (TACAS 2022): https://www.fi.muni.cz/~xstrejc/publications/tacas2022symbiotic.pdf
- Witness format spec (GraphML 1.0 + YAML 2.0): https://github.com/sosy-lab/sv-witnesses
- Participating systems / validators: https://sv-comp.sosy-lab.org/2025/systems.php
- sv-benchmarks UB issues: #307 (signed overflow) https://github.com/sosy-lab/sv-benchmarks/issues/307 ; #504 (div-by-zero) https://github.com/sosy-lab/sv-benchmarks/issues/504 ; #1304 (undocumented nondet) https://github.com/sosy-lab/sv-benchmarks/issues/1304
