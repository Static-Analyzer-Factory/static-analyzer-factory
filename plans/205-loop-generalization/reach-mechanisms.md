# Plan 205 — ReachSafety / unreach-call mechanism study (mechanisms for SAF to reimplement)

**Date:** 2026-08-17  **Branch:** `svcomp`  **Status:** research + design (no code)
**Hard constraint:** REQ-IP-001 — extract ALGORITHMS only; NEVER copy/vendor code. Reimplement independently on SAF's own substrate (AIR + PTA + value-flow + interval-AI + Z3 + native replay). Every mechanism below is cited to primary sources so the algorithm can be re-derived, not copied.

## Purpose
On `unreach-call`/ReachSafety, SAF is WEAK on novel tasks: on genuinely-new svcomp26 benchmarks its confirmed recall is **0%** — it confirms zero new reachability violations. Its train-pool recall (~3.8% confirmed) is mostly repeated Juliet-style tasks. This report extracts, from the top SV-COMP ReachSafety performers, the BUG-FINDING (refutation) MECHANISMS that generalize to unseen programs, maps each onto SAF's soundness model, and specifies concrete `levers.tsv` capability levers the autonomous loop can build over several arms.

## SAF's soundness model (the single filter every mechanism must satisfy)
SAF is SOUND + FALSE-only. It emits `false(unreach-call)` ONLY when it can (a) prove `main` must-reach `reach_error()` unconditionally, or (b) **concretely reproduce it**: SAF compiles the program and **natively replays the binary** (ASan/UBSan as oracles). A candidate INPUT that makes the replayed binary hit `reach_error()` is a confirmed FALSE **and doubles as the violation witness**. Over-approximate findings alone are NEVER emitted (a wrong FALSE = −16). Z3 is a path/input **oracle feeding replay** — replay decides; a wrong model → replay fails → SAF abstains.

**Consequence — the reusable primitive every mechanism below reduces to.** SAF wants exactly one thing: **a concrete assignment to the `__VERIFIER_nondet_*` call sites that drives the binary to `reach_error()`.** There are only two ways to produce it, and the entire ReachSafety field is a taxonomy over these two:

| Producer | Mechanism family | Tools | Confirmation cost for SAF |
|---|---|---|---|
| **Solve** a path/program formula → SMT model → read nondet values | BMC, symbolic execution, CEGAR feasibility, k-induction base case, trace abstraction, IC3/PDR preimage | CBMC, ESBMC, CPAchecker, UAutomizer, Symbiotic/KLEE | Z3 proposes; **native replay decides** (spurious model → abstain) |
| **Search** the input space directly, observing coverage / target-hit | greybox fuzzing, concolic, portfolio | AFL, FuSeBMC, VeriFuzz, VeriAbs, Bubaak | **The fuzzer run IS the confirmation** — a target-hit input is already replay-verified |

Both feed SAF's existing native-replay confirmer. The **fuzzing (search) family is the highest-ROI direction for a FALSE-only tool that already owns a build+run+oracle harness** — it produces a concrete reaching input with NO separate confirmation step, and it generalizes because it is search over the program's own branch structure, not pattern-matching a known pool. The **solve family** is the complement: it reaches *guarded* targets (magic numbers, tight equalities, deep counting) that blind/greybox search cannot, and SAF already has Z3 + AIR to build it. The winning move is to build BOTH and let a cheap static selector route per task (that is literally what VeriAbs/Bubaak do).

---

## 0. Decisive scoring & witness facts (verified 2026-08-17 against sv-comp.sosy-lab.org/2025/rules.php)

- **Scoring:** correct FALSE **+1** (only if a validator confirms the violation witness), incorrect FALSE **−16**, correct TRUE **+2** (needs a correctness witness), incorrect TRUE **−32**, UNKNOWN **0**.
- **unreach-call violation witnesses use YAML v2.0** (and GraphML v1.0) — this is the format SAF ALREADY emits at 67% CPAchecker-confirmed (plan 194), lifted toward 86% by the cpa-witness2test execution validator (plan 195). So **for sequential ReachSafety the witness pipeline is already solved** — unlike concurrency (sibling report `conc-term-mechanisms.md`), no new witness format is needed. The bottleneck is purely **generating the reaching input**. Every lever below ends by handing a concrete nondet-input vector to the existing replay+witness path.
- **Replay-confirmation makes wrong-FALSE (−16) structurally impossible:** a real native execution hit `reach_error()`. This is why SAF can safely use *deliberately under-precise / unsound-in-isolation* solving tricks (optimistic constraint solving, coarse memory encodings, bounded unrolling) — replay is the backstop. Top tools cannot do this; they must be exact. **This is SAF's structural advantage and every lever should exploit it.**

---

## 1. The convergence: all four tool families reduce to "path/program formula → Z3 model → nondet vector → replay"

The single most important cross-cutting finding. CEGAR (CPAchecker), trace abstraction (UAutomizer), the k-induction base case (BMC), and each IC3/PDR preimage query differ ONLY in *which* path and *how many* steps they hand the solver — the solve-and-extract step is identical:

```
concrete ICFG path/program π  (ℓ_init → … → reach_error), loops unrolled to k
  └─ build SSA path/program formula Φ(π):
       assignment  x = e            ⇒  x@(idx+1) = e[v → v@idx(v)]     (fresh SSA index on LHS)
       branch/assume [c]            ⇒  c[v → v@idx(v)]                 (guard; no index bump)
       nondet x = __VERIFIER_nondet_T()  ⇒  x@(idx+1) = ν_j            (ν_j = FRESH FREE bitvector of width(T))
       array a[i]=v / x=a[i]        ⇒  store/select (McCarthy theory of arrays)
       pointer *p                   ⇒  (object,offset) tuple, PTA-scoped
  └─ assert reach_error block reachable:  ⋁ guard(error_block)
  └─ Z3 check-sat
       ├─ UNSAT → path infeasible within k  (interpolate + refine = PROVE side — SKIP for a FALSE-finder)
       └─ SAT   → model M ; read M(ν_j) per nondet call site IN CALL ORDER → input vector (v0,v1,…)
                → emit as __VERIFIER_nondet_* return stream (static switch on a call counter)
                → COMPILE + native-replay → reach_error hit ⇒ CONFIRMED FALSE + YAML-2.0 witness
```

The free variables `ν_j` are exactly where reaching inputs live; the model pins them. This is **theory-complete over the path** (SMT decides the exact bit-precise semantics, not a syntactic pattern), which is why it transfers to novel tasks. CBMC/ESBMC are the industrial implementations of exactly this loop and both already emit SV-COMP violation witnesses + runnable tests from the SMT model — the closest existing reference for SAF to mirror. **SAF already has 4 of the 5 pieces (AIR/ICFG, PTA, interval-AI, Z3, replay); the missing piece is the AIR→SSA path/program encoder + nondet-model extractor.** This encoder is shared infrastructure that ALL solve-family levers (2, 3, 4) reuse — build it once.

For a FALSE-finder specifically, only the **SAT branch** of each mechanism matters. The prove-side machinery (Craig interpolation / predicate discovery, k-induction inductive step, IC3/PDR frame maintenance + clause generalization, correctness-witness generation) contributes NOTHING to a concrete reaching input and is explicitly **low-ROI / do-not-build** for SAF (see §7).

---

## 2. TRACK A — Search family: greybox fuzzing (HIGHEST ROI; the biggest gap in the current levers.tsv)

**The current `levers.tsv` has NO fuzzing lever.** All four research streams independently concluded this is the single highest-ROI direction for a FALSE-only tool that already has a build+native-replay harness with an oracle. A greybox fuzzer is mostly a mutation loop + coverage instrumentation bolted onto machinery SAF already owns, and it produces a concrete reaching input *directly* — the fuzzer run IS the confirmation (no separate replay step, no −16 risk by construction).

The universal SV-COMP adaptation (every fuzzer does this): model each `__VERIFIER_nondet_T()` as a **byte-stream cursor** — each call consumes `sizeof(T)` bytes from a fuzzer-controlled buffer (the `FuzzedDataProvider` pattern), deterministic per input so replays reproduce (essential for witness soundness, matches SAF's rand-determinism fixes in plan 203). Compile `reach_error()`/`__assert_fail`/`__VERIFIER_error` to `abort()`/`__builtin_trap()` so a target-hit surfaces exactly like a crash. Coverage via **SanitizerCoverage** (`-fsanitize-coverage=inline-8bit-counters,pc-table,trace-cmp`) since SAF is LLVM-based — the counter array IS the AFL coverage map.

### A1 (TOP ROI) — Blind mutation fuzzer over the nondet byte-stream

**Algorithm (AFL, minus coverage).** Seed corpus {all-zeros, all-ones, random-in-range, IR-constant dictionary} → mutate (bit/byte flips, arithmetic ±35, "interesting" boundary values {0, 1, −1, INT_MAX, INT_MIN, …}, havoc = random stacked mutations, splicing) → run the native binary via the existing harness → an abort at the target means the consumed byte-sequence IS a confirmed FALSE and IS the witness value-assignment for every nondet call. Interval-AI bounds each nondet's domain `[lo,hi]` (map consumed bytes into range) so draws are always feasible — VeriFuzz's "restricted range / bounding the unbounded" trick that produced the most counterexamples in SV-COMP'19.

**Why it generalizes.** No coverage feedback needed to catch shallow bugs; blind search over the input space works on any program SAF can compile — seen before or not. Directly produces the concrete reaching input the pipeline consumes.

**Maps to SAF.** Reuses the build+replay harness verbatim. New code = (a) a nondet→byte-cursor shim linked into the compiled program, (b) a mutation loop, (c) map abort/signal → FALSE. Zero over-approximation possible (the binary really aborted). This is the fastest path to *any* new-benchmark recall.

**Difficulty:** LOW–MEDIUM. **ROI:** VERY HIGH (reuses the oracle; self-confirming; catches shallow/wide bugs the current levers miss entirely).
**Primary sources.** AFL technical whitepaper https://lcamtuf.coredump.cx/afl/technical_details.txt ; libFuzzer / FuzzedDataProvider https://llvm.org/docs/LibFuzzer.html ; VeriFuzz (range bounding) https://sv-comp.sosy-lab.org/2019/talks/36_VeriFuzz.pdf .

### A2 (CORE) — Coverage-guided greybox via SanitizerCoverage, optionally directed at reach_error

**Algorithm.** Add feedback so inputs reaching *new* edges are kept, accumulating a corpus that drills deeper. Instrument with `-fsanitize-coverage=inline-8bit-counters,pc-table,trace-cmp`; `__sanitizer_cov_8bit_counters_init(start,end)` gives the coverage map (apply AFL's 8-bucket hit-count coarsening); `__sanitizer_cov_pcs_init` maps counter index → PC (→ free witness waypoints); a global "virgin" map detects new coverage. Loop: pick seed → assign energy (AFLFast **FAST** schedule `min((α/β)·2^{s(i)}/f(i), M)`, steering energy to low-frequency paths) → mutate → run → keep if new coverage. To target the *specific* `reach_error`, layer **AFLGo directed** scheduling: precompute per-block harmonic-mean distance to the target over the callgraph/CFG and use the simulated-annealing power schedule to drive seeds toward the target rather than exploring blindly.

**Why it generalizes.** Coverage feedback is defined purely over the program's own CFG edges — the search adapts to whatever branch structure a novel program has. This is the canonical "solve programs you've never seen" engine.

**Maps to SAF.** Purely additive to A1: the compile step adds SanCov flags; the counter-map reader + new-coverage check + energy schedule are the only new logic. `pc-table` gives source locations for the existing YAML-2.0 witness waypoints. AFLGo directedness targets the exact `reach_error` block.

**Difficulty:** MEDIUM. **ROI:** VERY HIGH (turns A1 into a real deep-target engine).
**Primary sources.** SanitizerCoverage https://clang.llvm.org/docs/SanitizerCoverage.html ; AFLFast (energy) https://mboehme.github.io/paper/TSE18.pdf ; AFLGo (distance + annealing) https://mboehme.github.io/paper/CCS17.pdf ; libFuzzer value profile https://llvm.org/docs/LibFuzzer.html .

---

## 3. TRACK B — Solve family: bounded model checking + symbolic execution (the guarded-target complement)

Blind/greybox fuzzing cannot cross `if (x == 0xDEADBEEF) reach_error()` (all-or-nothing, 2^-32 blind). The solve family reaches exactly these targets. SAF's `levers.tsv` already has `symbolic-oracle` — the mechanisms below SHARPEN it into a full BMC/SE engine and specify the shared AIR→SSA encoder.

### B1 (HIGHEST solve-ROI) — Bounded-k BMC: unwind to SSA → Z3 → nondet model → replay

**Algorithm (CBMC/ESBMC core; = k-induction BASE CASE = CEGAR feasibility = trace-check, unified).** Unwind each loop `while(c){B}` into nested `if(c){B; if(c){B;…}}` to bound k (recursion/function-pointers similarly, resolved via PTA to a guarded switch). Convert the unwound region to guarded SSA `⋀_j (g_j → x_{n_j}=e_j)`; each `__VERIFIER_nondet_T()` → a fresh free bitvector; assert `⋁ guard(reach_error block)`. Solve. SAT ⇒ a path violates the property; `(get-model)` → the free-variable values in nondet-call order = the concrete input vector → replay. **Crucially for SAF (FALSE-only): the entire k-induction bug-finding capability = the BMC base case.** SAF does NOT need the forward-condition or inductive-step (those prove TRUE). ESBMC-kind placed 2nd in ReachSafety at SV-COMP 2025 on essentially this.

**Why it generalizes.** Bit-precise semantic reasoning, not benchmark pattern-matching; finds ANY bug reachable within k iterations regardless of program shape; complete-for-bugs up to k (never misses a shallow bug). Because replay confirms, SAF can use a fast under-precise encoding and stay sound.

**Maps to SAF.** SAF's AIR = the "GOTO program" CBMC builds by hand — the front-end work is already done. New code = AIR-loop-unwinder → guarded SSA → Z3 emit → nondet-model read-out (fresh Z3 bitvector per nondet call site; read model in call order). Interval-AI seeds `assume` ranges to shrink search; value-flow slices the cone of influence before encoding (CBMC's single biggest historical speedup). This is the concrete build-out of the existing `symbolic-oracle` lever.

**Difficulty:** MEDIUM. **ROI:** VERY HIGH — lifts SAF from "must-reach unconditionally" to "any bounded feasible path," directly attacking the recall bottleneck; reuses AIR + Z3 + replay.
**Primary sources.** CBMC (Kroening/Schrammel/Tautschnig 2023) https://arxiv.org/abs/2302.02384 ; ESBMC (Cordeiro/Fischer/Marques-Silva, IEEE TSE 2012) https://ssvlab.github.io/lucasccordeiro/papers/tse2012.pdf ; k-induction (Rocha/Ismail/Cordeiro/Barreto 2015) https://arxiv.org/abs/1502.02327 ; k-induction (Beyer/Dangl/Wendler, CAV 2015) https://arxiv.org/abs/1502.00096 ; SV-COMP 2025 report https://www.sosy-lab.org/research/pub/2025-TACAS.Improvements_in_Software_Verification_and_Witness_Validation_SV-COMP_2025.pdf .

### B2 — Incremental unwinding (grow k, reuse Z3 state via activation literals)

**Algorithm.** Instead of re-encoding per k, grow the formula monotonically in one persistent Z3 context; gate each depth's error assertion with a fresh Boolean activation literal α_k and solve `check-sat-assuming [¬α_k]` (later depths assume α_k true, emulating clause deletion while keeping learnt clauses valid). Bound with k_max. ~10× throughput vs "restart per k" in the source study → reaches deep-loop bugs (k≫10) B1's fixed small-k misses.

**Maps to SAF.** Z3 `push`/`pop` + `check-sat-assuming` are native; requires append-only unwinding construction with error clauses behind activation literals. Pure follow-on to B1.
**Difficulty:** MEDIUM–HIGH. **ROI:** HIGH (multiplies B1's reach per time budget).
**Primary sources.** Schrammel et al., Incremental BMC 2014 https://arxiv.org/abs/1409.5872 ; ESBMC 7.4 interval pass (TACAS 2024) https://link.springer.com/chapter/10.1007/978-3-031-57256-2_21 .

### B3 — KLEE-style symbolic interpreter over sliced AIR (path-explosion-resistant SE)

**Algorithm (KLEE).** Forward symbolic interpreter: state = regs/stack/heap of Z3 expression trees + path condition. Concrete fast-path for constant operands. At a branch, query Z3 for feasibility of each side; fork the state only when both are feasible (add `c` / `¬c` to the two PCs). `__VERIFIER_nondet_*` → fresh symbolic vars. Dangerous ops (div-by-0, OOB) are implicit branches. On reaching `reach_error`, solve the accumulated PC for a model → concrete nondet vector → replay. Symbolic memory = per-object Z3 arrays (segmented). Tractability comes from KLEE's query optimizations (>10× in the paper): **constraint independence** (solve only the sub-constraints touching the queried vars), **counterexample cache** (UBTree subset/superset reuse), constraint-set simplification, implied-value concretization.

**Why it generalizes vs B1.** SE explores feasible paths lazily instead of unrolling everything, so it handles complex control flow and doesn't blow up on wide programs; it is deductive reasoning about actual semantics. B1 (BMC) is better for tight-bounded loops; B3 (SE) for branchy control. A portfolio wants both.

**Maps to SAF.** SAF's AIR is the same LLVM substrate KLEE interprets; Z3 is the feasibility oracle; replay decides. Reuses the B1 path-formula encoder per path. **Prerequisite: property-directed slicing (§4) to keep SE tractable** — Symbiotic's winning formula is *slice-then-KLEE*.
**Difficulty:** HIGH (state cloning, symbolic memory, external calls). Start intraprocedural + loop-bounded on already-sliced code; add constraint-independence + cex-cache early. **ROI:** HIGH (the recall engine for branchy programs; slicing makes it feasible).
**Primary sources.** KLEE (OSDI 2008) https://www.doc.ic.ac.uk/~cristic/papers/klee-osdi-08.pdf ; Symbiotic 9 (TACAS 2022, slice→KLEE→replay-on-unsliced) https://www.fi.muni.cz/~xstrejc/publications/tacas2022symbiotic.pdf .

---

## 4. TRACK C — Scalability multiplier: property-directed backward slicing (force-multiplies EVERYTHING)

Already in `levers.tsv` as `air-slicing`. This report sharpens it: it is not optional polish — it is the mechanism that makes previously-intractable ReachSafety tasks solvable, for BOTH the fuzzing and solve families, and it shrinks witnesses.

**Algorithm (Symbiotic's `dg` slicer).** Build a Program Dependence Graph on the AIR: nodes = instructions; edges = **use-dep** (SSA def-use, trivial), **data-dep** (r reads memory w may write — the may/must read-write sets come from SAF's **Andersen PTA**; built via memory-SSA), **control-dep** (SCD first; NTSCD — non-termination-sensitive — later). Slicing criterion = every `reach_error`/`__assert_fail`/`__VERIFIER_error` call. **Slice = the set of PDG nodes backward-reachable from the criteria** (reverse graph reachability / worklist). Emit *executable* sliced AIR. Add coarse pre-pruning: blocks with no syntactic path to any criterion → `abort()`. Measured sliced size ≈ 67% average, many below 30%.

**CRITICAL correctness detail — secondary criteria.** A plain backward slice deletes `__VERIFIER_assume(x>0)` on a path to the error because the assume doesn't *modify* x — discarding a crucial path constraint and producing wrong reaching inputs. Symbiotic marks `assume` a **secondary criterion**: include it iff it lies on a path into a primary criterion (data-secondary additionally requires touching the same memory). **SAF must replicate this or it will slice away range-narrowing guards and generate spurious inputs that fail replay** (safe but recall-killing).

**Maps to SAF.** SAF's value-flow graph ≈ data-dependence edges; ICFG + callgraph ≈ control-dependence structure; PTA already supplies the may/must annotations. This is assembling existing pieces into a PDG + backward reachability. It pays off even before any new solver exists (shrinks the interval-AI/value-flow/replay scope), and is the prerequisite that makes B3 (SE) and A2 (directed fuzzing) finish in budget.
**Difficulty:** MEDIUM (PDG + NTSCD are the hard parts; SCD + secondary criteria first). **ROI:** VERY HIGH (multiplies every other lever; low standalone risk — it's a scope reduction, replay still decides).
**Primary sources.** DG — Analysis and Slicing of LLVM Bitcode (ATVA 2020) https://www.fi.muni.cz/~xchalup4/dg_atva20_preprint.pdf ; dg llvm-slicer docs (criteria + secondary criteria) https://github.com/mchalupa/dg/blob/master/doc/llvm-slicer.md ; Symbiotic (TACAS 2013, synergy of slicing+SE) https://link.springer.com/chapter/10.1007/978-3-642-36742-7_50 .

---

## 5. TRACK D — Hybrid + selection: concolic branch-flipping, hybrid fuzz+BMC, portfolio routing

### D1 — Z3 concolic branch-flipping past magic-number / equality guards

**Algorithm.** When greybox (A2) is *stuck* (Driller criterion: mutation budget elapses with zero new edges), take an interesting input, symbolically execute its concrete path over the AIR, pick a branch whose flipped edge is not yet covered, solve `Φ_prefix ∧ ¬guard` with Z3, and feed the model's bytes back as a NEW SEED to the fuzzer. Escalate cheaply first: **CmpLog/Redqueen input-to-state** (SAF knows the exact buffer offset feeding each compared value — it can patch offset→constant directly, no colorization search) and **laf-intel** (IR pass splitting N-byte compares into cascaded 1-byte compares). Apply QSYM cost controls: **relevant-constraint slicing**, **optimistic last-constraint solving** on UNSAT/timeout (solve just the negated final predicate — native replay discards duds, *safe precisely because SAF replay-confirms*), power-of-two basic-block back-off to stop loops flooding Z3.

**Why it generalizes.** Path constraints are discovered dynamically from real executions — the same loop solving a toy `if(x==10)` solves a real checksum gate. This is exactly the guard class where a Z3-backed tool beats blind mutation, and correctness is outsourced to replay.

**Maps to SAF.** SAF has Z3 + AIR + value-flow; the "flipped-edge-uncovered" test uses A2's coverage set; concretization = read model bytes into a new seed. Optimistic-solving unsoundness is FREE for SAF (every candidate is replay-gated). Best sequenced after A2.
**Difficulty:** MEDIUM–HIGH. **ROI:** HIGH on constraint-heavy tasks; lower on shallow (A1/A2 win those).
**Primary sources.** Driller (NDSS 2016) https://www.ndss-symposium.org/wp-content/uploads/2017/09/driller-augmenting-fuzzing-through-selective-symbolic-execution.pdf ; QSYM (USENIX Sec 2018) https://www.usenix.org/system/files/conference/usenixsecurity18/sec18-yun.pdf ; Redqueen/CmpLog https://www.ndss-symposium.org/ndss-paper/redqueen-fuzzing-with-input-to-state-correspondence/ ; laf-intel https://lafintel.wordpress.com/2016/08/15/circumventing-fuzzing-roadblocks-with-compiler-transformations/ .

### D2 — Static portfolio selector: route fuzz-vs-BMC-vs-SE per task under a budget

**Algorithm.** The evidence is decisive that **1–2 cheap syntactic Booleans beat any constant strategy** (Beyer/Dangl reach 97% of a perfect oracle with 4 Booleans — no ML needed). Extract O(IR-size) features from the AIR: `hasLoop`, `loopBounded`, `hasArray`, `hasFloat`, `hasRecursion`, `hasNonlinearArith`, `inputRangeNarrow`, `#branches × #nondet` (a fuzz-difficulty proxy — VeriAbs removes fuzzing when this grows). First-match rule cascade (VeriAbs style): narrow ranges/few branches → fuzz first; no loop → BMC (complete, cheap); bounded scalar loops → BMC/k-induction; unbounded/array/nonlinear → SE + fuzz in parallel. Run as a ranked sequential portfolio under a time budget with early abstention, or parallel first-to-succeed. Upgrade: once the loop has "solved-by" labels, train a classifier on the same Booleans to predict the ranking (VeriAbsL generalizes to unseen categories *provided every capability appears in the labels*, else it gets starved).

**Maps to SAF.** SAF's autonomous loop IS the "improvement loop" this targets. Model each lever as a typed actor `Program → Verdict×Witness` (CoVeriTeam pattern); the learnable surface is the routing predicates. Build only AFTER ≥2 productive levers exist to choose between.
**Difficulty:** LOW (rule cascade v0) → MEDIUM (learned ranking). **ROI:** MEDIUM–HIGH as a multiplier (fires the right lever, stops wasting budget); low standalone.
**Primary sources.** Beyer/Dangl Boolean features (ISoLA 2018) https://www.sosy-lab.org/research/pub/2018-ISoLA.Strategy_Selection_for_Software_Verification_Based_on_Boolean_Features.pdf ; VeriAbs (TACAS 2021) https://link.springer.com/content/pdf/10.1007/978-3-030-72013-1_32.pdf ; VeriAbsL (TACAS 2023) https://link.springer.com/chapter/10.1007/978-3-031-30820-8_41 ; PeSCo https://link.springer.com/chapter/10.1007/978-3-030-17502-3_19 ; CoVeriTeam (TACAS 2022) https://www.sosy-lab.org/research/pub/2022-TACAS.CoVeriTeam_On-Demand_Composition_of_Cooperative_Verification_Systems.pdf .

### D3 (note, not a standalone lever) — FuSeBMC hybrid cooperation contract
If SAF ever runs fuzzing (A) and BMC (B) together: share a seed pool — BMC's counterexample → fuzzer seed to reach guarded targets; fuzzer's partial inputs → BMC subgoals; a "Tracer" ranker salvages timed-out solver progress as partial seeds. For a FALSE-only tool, A1–A2 + B1 + D1 already deliver the concrete-input-then-replay contract natively; adopt the FuSeBMC handshake only once both engines exist. Watch its footgun: constant-folding/slicing that optimizes away a nondet init → replay reads uninitialized state; SAF's native replay is the correct arbiter. Source: FuSeBMC https://arxiv.org/abs/2206.14068 .

---

## 6. Ranked capability levers for SAF (proposed `levers.tsv` rows)

Schema: `id <TAB> mode <TAB> family <TAB> scope <TAB> description`. `mode ∈ {tuning, capability}`; `family = unreach-call`; `scope ∈ {local, crosscut}` (crosscut = touches shared frontend/AIR/harness → eval'd on ALL properties, reverted if any family regresses). Ordered by ROI. **Bold = genuinely new (not yet in levers.tsv); the rest sharpen existing rows.**

| # | id | mode | scope | one-liner (worker-actionable) | difficulty / ROI |
|---|---|---|---|---|---|
| L1 | **`fuzz-mut-nondet`** | capability | local | Link a deterministic byte-stream shim for `__VERIFIER_nondet_*` (each call consumes `sizeof(T)` bytes; ranges bounded by interval-AI; dictionary = IR constants), compile `reach_error→abort`, run an AFL-style blind mutation loop under the existing native-replay harness; an abort = confirmed FALSE + witness (no separate confirm step). | LOW–MED / **VERY HIGH** |
| L2 | **`fuzz-covguided`** | capability | local | Compile with SanitizerCoverage (`inline-8bit-counters,pc-table,trace-cmp`); build an 8-bit coverage map + new-coverage seed-keeping + AFLFast energy; optionally AFLGo distance-to-`reach_error` annealing; `pc-table` feeds YAML-2.0 witness waypoints. Turns L1 into a deep-target engine. | MED / **VERY HIGH** |
| L3 | `air-slicing` (sharpen) | capability | crosscut | Backward AIR slice from every `reach_error` criterion via a PDG (use+data+control deps; PTA-annotated may/must sets; NTSCD). **Add `__VERIFIER_assume` as a secondary criterion** (else range guards are sliced away → spurious inputs). Emit executable sliced AIR + coarse abort-prune. Multiplies L2/L4/L5. | MED / **VERY HIGH** |
| L4 | `symbolic-oracle` → **`bmc-fixed-k`** (build out) | capability | local | Unwind AIR loops to bound k (callees inlined to depth d), encode guarded SSA + `reach_error` reachability to Z3 (nondet = fresh bitvectors), on SAT read the model in nondet-call order → input vector → native-replay confirm. This is the k-induction BASE CASE; skip forward-condition/inductive-step. | MED / **VERY HIGH** |
| L5 | **`se-interp`** | capability | local | KLEE-style bounded forward symbolic interpreter over **sliced** AIR (fork on Z3-feasible branches, per-object symbolic arrays, constraint-independence + counterexample cache); solve PC at `reach_error` for a model → replay-confirm. Complements L4 on branchy control-flow. | HIGH / HIGH |
| L6 | **`bmc-incremental`** | capability | local | Grow the L4 unwinding bound in ONE persistent Z3 context, gating each depth's `reach_error` check with a `check-sat-assuming` activation literal, until SAT or k_max. ~10× deeper reach per budget vs fixed-k. | MED–HIGH / HIGH |
| L7 | **`fuzz-concolic-z3`** | capability | local | On coverage plateau (Driller stuck-detector), symbolically execute a stuck seed's path over the AIR, Z3-solve the negation of an uncovered branch (constraint-slicing + optimistic fallback — safe because replay gates), feed the solved input back as a seed; CmpLog/laf-intel handle cheap magic-number cases first. | MED–HIGH / HIGH (constraint-heavy tasks) |
| L8 | `harness-havoc` (relate) | capability | crosscut | Entry-point/harness synthesis + complex-object havocking so nondet structs/pointers/arrays become symbolic/fuzzable inputs (KLEE symbolic-memory + FuzzedDataProvider for aggregates). Unblocks L1/L4/L5 on the TDX firmware batch (structural, not solving). | MED / HIGH |
| L9 | **`portfolio-select`** | tuning | local | Extract cheap AIR Booleans (loop/array/float/recursion/nonlinear/narrow-range/branch×nondet), route each task to a ranked sequence of levers (fuzz L1–2 / BMC L4 / SE L5) under a time budget with early abstention; later learn the ranking from solved-by labels. Build after ≥2 levers exist. | LOW→MED / MED–HIGH (multiplier) |

### Shared infrastructure (build once, reused by L4/L5/L6/L7)
The **AIR→SSA path/program encoder + nondet-model extractor** from §1. One Z3-encoding module: assignment→SSA equality, guard→constraint, nondet call site→fresh bitvector of width(T), array→select/store (McCarthy), pointer→PTA-scoped (object,offset) tuple with interval-bounded offset, lazy Ackermann array-consistency refinement. `(get-model)` → per-nondet-site value in call order. This is the reusable primitive; every solve-family lever is a different *driver* over it.

### Recommended build sequence (arms)
1. **L1 `fuzz-mut-nondet`** — smallest new surface, reuses the oracle, immediate new-benchmark recall, self-confirming. Highest-ROI single move.
2. **L3 `air-slicing`** (with secondary-assume fix) — independent, multiplies everything, de-risks L4/L5. Can run in parallel with L1.
3. **L2 `fuzz-covguided`** — turns L1 into a deep-target engine (SanCov = compile flags + a few callbacks).
4. **L4 `bmc-fixed-k`** — build the shared SSA encoder here; reaches guarded/counting targets fuzzing can't.
5. **L5 `se-interp`** on sliced AIR, then **L6 `bmc-incremental`**, then **L7 `fuzz-concolic-z3`** (needs L2's coverage + L4's encoder), then **L9 `portfolio-select`** once there's a choice to route.

---

## 7. Prove-oriented parts SAF should NOT build (explicit low-ROI for a FALSE-finder)
None of these produce a concrete reaching input; all exist to prove TRUE, which SAF never emits.
- **Craig interpolation / predicate discovery / lazy abstraction refinement** (CPAchecker CEGAR, UAutomizer Floyd-Hoare / interpolant automata / two-track proofs). Fires only on UNSAT (infeasible) paths.
- **k-induction inductive step + auxiliary-invariant generation** (only the base case finds bugs).
- **IC3/PDR frame maintenance, relative-inductive clause generalization, convergence detection** (the CTI→preimage→init chain is theoretically borrowable for *deep* counterexamples, but heavy and only worth it if measurement proves recall is lost specifically to counterexample DEPTH that L6 can't reach affordably — evidence-gated, deferred).
- **Backward SE with loop folding / BSELF** (Symbiotic) — proves unbounded-loop *correctness*; out of scope for FALSE-only and doesn't scale by the authors' own admission.
- **Correctness-witness (TRUE-side) generation** — irrelevant to unreach-call FALSE.

---

## 8. Consolidated primary sources
**BMC / SMT:** CBMC 2023 https://arxiv.org/abs/2302.02384 · ESBMC TSE 2012 https://ssvlab.github.io/lucasccordeiro/papers/tse2012.pdf · k-induction 2015 https://arxiv.org/abs/1502.02327 · Incremental BMC https://arxiv.org/abs/1409.5872 · ESBMC 7.4 https://link.springer.com/chapter/10.1007/978-3-031-57256-2_21 · original BMC (Biere et al. 1999) https://link.springer.com/chapter/10.1007/3-540-49059-0_14
**Symbolic execution / slicing:** KLEE OSDI 2008 https://www.doc.ic.ac.uk/~cristic/papers/klee-osdi-08.pdf · DG/dg slicer ATVA 2020 https://www.fi.muni.cz/~xchalup4/dg_atva20_preprint.pdf · dg slicer docs https://github.com/mchalupa/dg/blob/master/doc/llvm-slicer.md · Symbiotic 9 TACAS 2022 https://www.fi.muni.cz/~xstrejc/publications/tacas2022symbiotic.pdf · Symbiotic TACAS 2013 https://link.springer.com/chapter/10.1007/978-3-642-36742-7_50 · Directed SE (SDSE/CCBSE) https://www.cs.tufts.edu/~jfoster/papers/cs-tr-4979.pdf
**CEGAR / k-induction / trace abstraction:** CPAchecker CAV 2011 https://www.sosy-lab.org/research/pub/2011-CAV.CPAchecker_A_Tool_for_Configurable_Software_Verification.pdf · ABE FMCAD 2010 https://www.sosy-lab.org/research/pub/2010-FMCAD.Predicate_Abstraction_with_Adjustable-Block_Encoding.pdf · Lazy Abstraction w/ Interpolants (McMillan CAV 2006) https://people.eecs.berkeley.edu/~alanmi/courses/2007_290N/papers/inter_mcmillan_cav06.pdf · k-induction CAV 2015 https://arxiv.org/abs/1502.00096 · Trace Abstraction SAS 2009 https://jochen-hoenicke.de/docs/hhp09-sas.pdf · Tests-from-Witnesses TAP 2018 https://www.sosy-lab.org/research/pub/2018-TAP.Tests_from_Witnesses_Execution-Based_Validation_of_Verification_Results.pdf
**IC3/PDR (deferred):** Bradley VMCAI 2011 http://theory.stanford.edu/~arbrad/papers/IC3.pdf · PDR FMCAD 2011 https://people.eecs.berkeley.edu/~alanmi/publications/2011/fmcad11_pdr.pdf · Software IC3 CAV 2012 https://es-static.fbk.eu/people/griggio/papers/cav12.pdf · Spacer CAV 2014 https://link.springer.com/chapter/10.1007/978-3-319-08867-9_2
**Fuzzing / hybrid / portfolio:** AFL https://lcamtuf.coredump.cx/afl/technical_details.txt · libFuzzer https://llvm.org/docs/LibFuzzer.html · SanitizerCoverage https://clang.llvm.org/docs/SanitizerCoverage.html · AFLFast https://mboehme.github.io/paper/TSE18.pdf · AFLGo https://mboehme.github.io/paper/CCS17.pdf · Driller https://www.ndss-symposium.org/wp-content/uploads/2017/09/driller-augmenting-fuzzing-through-selective-symbolic-execution.pdf · QSYM https://www.usenix.org/system/files/conference/usenixsecurity18/sec18-yun.pdf · Redqueen https://www.ndss-symposium.org/ndss-paper/redqueen-fuzzing-with-input-to-state-correspondence/ · laf-intel https://lafintel.wordpress.com/2016/08/15/circumventing-fuzzing-roadblocks-with-compiler-transformations/ · VeriFuzz https://sv-comp.sosy-lab.org/2019/talks/36_VeriFuzz.pdf · FuSeBMC https://arxiv.org/abs/2206.14068 · Beyer/Dangl Boolean features https://www.sosy-lab.org/research/pub/2018-ISoLA.Strategy_Selection_for_Software_Verification_Based_on_Boolean_Features.pdf · VeriAbs https://link.springer.com/content/pdf/10.1007/978-3-030-72013-1_32.pdf · VeriAbsL https://link.springer.com/chapter/10.1007/978-3-031-30820-8_41 · PeSCo https://link.springer.com/chapter/10.1007/978-3-030-17502-3_19 · CoVeriTeam https://www.sosy-lab.org/research/pub/2022-TACAS.CoVeriTeam_On-Demand_Composition_of_Cooperative_Verification_Systems.pdf
**SV-COMP 2025 report / rules:** https://www.sosy-lab.org/research/pub/2025-TACAS.Improvements_in_Software_Verification_and_Witness_Validation_SV-COMP_2025.pdf · https://sv-comp.sosy-lab.org/2025/rules.php
