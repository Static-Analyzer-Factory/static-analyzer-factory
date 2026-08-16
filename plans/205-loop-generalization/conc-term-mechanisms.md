# Plan 205 — Concurrency & Termination mechanism study (mechanisms for SAF to reimplement)

**Date:** 2026-08-17  **Branch:** `svcomp`  **Status:** research + design (no code)
**Hard constraint:** REQ-IP-001 — extract ALGORITHMS only; NEVER copy/vendor code. Reimplement independently on SAF's own substrate. Every mechanism below is cited to primary sources so the algorithm can be re-derived, not copied.

## Purpose
SAF scores **0** on the concurrency tracks (ConcurrencySafety / no-data-race). Its autonomous loop already declares concurrency "levers" (`conc-shim-m1`, `conc-seq-m2`, `race-confirmer`, `graphml-witness`) but they score nothing, and its structural termination-TRUE proof has ~16% recall. This report extracts, from the top SV-COMP tools, the MECHANISMS that would make those levers score, mapped onto SAF's AIR + PTA + native-replay soundness model, with a refined `levers.tsv` spec for each.

## SAF's soundness model (what every mechanism must satisfy)
SAF is SOUND + FALSE-only. It emits `false(<prop>)` ONLY on a **concrete reproduction**: it compiles the program and **natively replays the binary** under ASan/UBSan/TSan oracles. For a concurrency violation it therefore needs an **explicit, deterministic SCHEDULE** (an interleaving / context-switch vector) that drives the replayed binary into the violation. That schedule both *confirms* the bug and *is* the witness. Any FINDER that produces only a yes/no (not a schedule) is insufficient on its own — it must be paired with a schedule extractor. This is the single most important filter on the mechanisms below.

---

## 0. Decisive scoring & witness facts (verified 2026-08-17 against sv-comp.sosy-lab.org/2025/rules.php)

- **Scoring:** correct FALSE **+1**, incorrect FALSE **−16**, correct TRUE **+2**, incorrect TRUE **−32**, UNKNOWN **0**. "Counted correct only if at least one validator successfully validated it."
- **Concurrency & no-data-race & termination violation witnesses use GraphML v1.0 ONLY.** YAML v2.0 is NOT listed for `ConcurrencySafety-*`, `no-datarace`, or `termination`. (Resolves an internal disagreement: SAF's YAML-2.0 sequential witness pipeline **cannot** be reused for concurrency; the `graphml-witness` lever is correctly targeting the only accepted format.) Source: https://sv-comp.sosy-lab.org/2025/rules.php (witness-format table).
- **Three working concurrency violation-witness validators** in SV-COMP 2025: **CPAchecker Validator 4.0**, **Dartagnan** (SMT, SC memory model as *input* — validates witnesses CPAchecker cannot), **ConcurrentWitness2Test** (execution/harness-based, strict/normal/permissive modes). Source: TACAS 2025 SV-COMP report https://www.sosy-lab.org/research/pub/2025-TACAS.Improvements_in_Software_Verification_and_Witness_Validation_SV-COMP_2025.pdf ; ConcurrentWitness2Test https://github.com/ftsrg/ConcurrentWitness2Test ; Dartagnan validator https://hernanponcedeleon.github.io/pdfs/svcomp2022.pdf .
  - **Implication:** the ConcurrentWitness2Test *execution-based* validator is a natural fit for SAF — SAF already produces the schedule by replaying, so the same schedule serializes into a GraphML witness that CW2T re-executes to confirm. This is the concurrency analogue of the cpa-witness2test lever that lifted sequential confirmation 43%→86% (plan 195).
- **Termination TRUE needs NO witness** (scored on verdict alone, +2; Termination is excluded from *correctness*-witness validation). **Termination FALSE needs a validated GraphML violation witness** for +1 — and non-termination witness support was still maturing (SV-COMP 2026 added format-2.1 non-termination witnesses, a 17-task pilot). Source: rules.php + SV-COMP 2026 report https://www.researchgate.net/publication/403845184 . **Consequence: termination TRUE (ranking-function recall) is far higher-ROI than termination FALSE for SAF.**

---

## 1. TRACK A — Concurrency safety (reach an assertion/error under some schedule)

Ranked by ROI for SAF. The central finding: **a blind schedule sweep cannot score; the schedule must be DIRECTED (by BMC) or PRUNED (by DPOR). SAF's best fit is lazy sequentialization + bounded solving, because the solver returns an explicit schedule SAF can replay.**

### A1 (TOP ROI) — Lazy sequentialization (Lazy-CSeq / bLMP schema) → fixes `conc-seq-m2`
**Algorithm.** Translate a K-round, N-thread concurrent program into ONE sequential nondeterministic program, then hand it to a bounded solver.
1. **Thread simulation functions.** Each thread body becomes `sim_i()`. At entry, `switch(pc_i)` jumps to the resume label from the previous round (re-enterable).
2. **Static locals = free context save/restore.** ALL of thread i's locals are `static` inside `sim_i()`, so they persist across yields with no explicit save/restore.
3. **Run-length nondeterminism (the only nondeterminism).** For each thread i and round r, a nondeterministic `rl[i][r]` = number of *visible statements* (shared-mem access / sync / create / join) thread i runs this round. At most K·N such choices.
4. **Round-robin driver.** K rounds; each round calls `sim_1..sim_N` in order; each `sim_i` runs `rl[i][r]` visible statements then stores `pc_i` and returns. Globals are shared naturally (they are C globals).
5. **Yield only at visible statements.** Between visible statements a thread runs atomically. A `remaining--; if(remaining==0){pc_i=HERE; return;}` guard precedes each visible statement.
6. **Bounded solving.** Encode the sequential program to SAT/SMT (loops unwound to depth U). SAT ⇒ the model fixes every `rl[i][r]` — i.e. an **explicit schedule**.
7. **Schedule extraction.** Read the `rl` matrix off the model → "round r: thread i runs N_ir visible steps" → a `(threadId, steps)` sequence.

**Why it works.** bLMP is *complete for all K-round round-robin schedules up to unwind U* and is an **under-approximation of the schedule space with NO data over-approximation** — every violation found is a genuine concurrent violation; shrinking K/U/L costs only recall, never a spurious FALSE. This is exactly SAF's soundness doctrine. Lazy-CSeq won ConcurrencySafety in 2014/2015/2020/2021.

**Produces a replayable schedule?** YES — the `rl` matrix IS a deterministic interleaving.

**Maps to SAF.** SAF already has the skeleton (`conc-seq-m2`: pc-labeled re-enterable thread fns + static local cells + round-robin driver). The gaps that make it score 0 (see §3): (a) `rl` must be a **genuine nondeterministic choice the solver assigns**, not a runtime constant/random; (b) yields must fire **only at PTA-identified shared accesses**, not every statement (SAF has Andersen PTA to classify shared vs thread-local); (c) mutex/create/join must be sequentialized as visible statements; (d) the model must be decoded back to a schedule and **re-confirmed by native replay** on the ORIGINAL binary; (e) that schedule serialized to a GraphML witness.

**Primary sources.** Lazy-CSeq CAV 2014 https://link.springer.com/content/pdf/10.1007/978-3-319-08867-9_39.pdf ; TOPLAS 2021 https://dl.acm.org/doi/full/10.1145/3478536 ; ASE tool paper https://eprints.soton.ac.uk/379595/1/lazy-cseq-ase.pdf ; SV-COMP 2016 https://eprints.soton.ac.uk/387011/1/SV-COMP_2016_paper_28.pdf .

**Difficulty MEDIUM (skeleton exists) / ROI VERY HIGH.** This is THE concurrency lever. It replaces the blind shim as the FINDER because the solver *directs* the search over schedule AND data jointly.

### A2 — Bounded solving backend: ordering-consistency (Deagle) vs weak-memory CAT (Dartagnan)
**Algorithm (Deagle ordering-consistency theory).** Instead of enumerating interleavings, encode shared-memory order as three relations solved inside DPLL(T): **rf** (each load reads-from exactly one store), **ws/co** (total order of stores per location), **fr** (derived reads-before). A dedicated theory solver checks acyclicity of rf∪ws∪fr incrementally and does consistency-preserving propagation (derives negated orders proactively to cut backtracking). One satisfying assignment covers a whole class of interleavings → faster than explicit-order BMC. Deagle won ConcurrencySafety 2022/2023, and led 2025.

**Produces a replayable schedule?** YES — rf + ws determine a concrete execution order.

**Maps to SAF.** This is the *solving backend* under A1 (or a direct encoding of the concurrent program). SAF has Z3; the partial-order encoding can be built over Z3 with PTA identifying shared accesses. Higher effort than plain bounded unrolling but the winning-tool encoding.

**Dartagnan / CAT.** Encodes the memory model itself as axioms (acyclicity of `hb`, coherence) compiled to SMT, with a *relation analysis* pre-pass pruning which event pairs can be related (2 orders of magnitude fewer variables). **For SAF the key Dartagnan fact is that it is a VALIDATOR** of SAF's GraphML witness (encodes the witness `threadId` ordering as constraints, checks the violation is reachable). SAF does NOT need to reimplement Dartagnan — it needs to emit a witness Dartagnan/CW2T accept.

**ConcurrencySafety-C is sequentially consistent** (interleaving semantics) — **do NOT build TSO/PSO/C11-relaxed machinery for the first capability.** Native x86 replay is TSO, so **abstain on any pthread-wmm / C11-relaxed / dropped `#pragma omp` task** (fail-closed, per plan-202 lesson).

**Primary sources.** Deagle TACAS 2022 https://feihe.github.io/materials/tacas-svcomp22.pdf ; OOPSLA 2022 https://feihe.github.io/materials/oopsla22.pdf ; TOPLAS https://dl.acm.org/doi/10.1145/3579835 ; Dartagnan TACAS 2020 https://link.springer.com/chapter/10.1007/978-3-030-45237-7_24 ; Dartagnan validator https://hernanponcedeleon.github.io/pdfs/svcomp2022.pdf .

**Difficulty HIGH / ROI HIGH** — a Milestone-3 upgrade of A1's backend. Not first.

### A3 — DPOR / stateless model checking (Nidhugg source-DPOR, GenMC) → an alternative FINDER (and the correct fix if SAF keeps a native-execution shim)
**Algorithm (source-DPOR).** Execute the program concretely (DFS, one thread chosen per step). At a terminal state, scan the trace backward for **races** = pairs of *dependent* transitions (same location, ≥1 write) run in one order that could run in the other. For each race compute a **source set** — the minimal set of threads that, scheduled first at the race point, realize the reversed order — and add them as backtrack points. **Sleep sets** avoid re-exploring done transitions. Explore exactly one interleaving per Mazurkiewicz (independence) class → avoids the combinatorial blowup a blind sweep suffers. GenMC does the same over execution graphs (po/rf/co) with an optimal revisit algorithm; for SC it enumerates each SC-consistent execution once.

**Why it works.** Independent (non-conflicting) transitions commute, so only race orderings need exploring. This is precisely what a blind context-switch sweep lacks — DPOR *directs* backtracking at real conflicts.

**Produces a replayable schedule?** YES — each explored execution is a concrete thread trace; the violating one is the counterexample.

**Maps to SAF.** DPOR is execution-based, matching SAF's replay, BUT Nidhugg/GenMC run an LLVM-IR *interpreter* under SC, not the native x86 binary. To adopt DPOR soundly SAF would drive its scheduler shim with DPOR backtracking (races detected via a shadow-memory pass over PTA-shared locations), replacing the blind c≤1→c≤2 sweep. This is the correct evolution of `conc-shim-m1` **if** SAF wants a native-execution finder; otherwise A1 (sequentialization+BMC) is simpler and jointly reasons about data.

**Primary sources.** Source Sets / Optimal-DPOR JACM 2017 https://dl.acm.org/doi/10.1145/2535838.2535845 ; Nidhugg https://github.com/nidhugg/nidhugg ; SMC for TSO/PSO https://arxiv.org/pdf/1501.02069 ; GenMC CAV 2021 https://link.springer.com/chapter/10.1007/978-3-030-81685-8_20 , https://plv.mpi-sws.org/genmc/ .

**Difficulty MEDIUM-HIGH / ROI HIGH** — the principled fix for the shim, but a bigger build than A1.

### A4 — Context-bounded / preemption-bounded testing (CHESS) → what `conc-shim-m1` currently IS, and why it must change
**Algorithm.** Bound the number of preemptions p; DFS over "continue current thread vs preempt-and-switch" at each preemption point; iterate p=0,1,2,… Empirically most real bugs surface within p≤2–3 (CHESS OSDI 2008; schedule-bounding study PPoPP 2014). Produces an explicit preemption sequence (replayable).

**Why it is SAF's `conc-shim-m1`, and why it scores 0.** CHESS works only when preemption points sit at **shared-memory accesses** and the search is small. SAF's shim runs on the native binary with no PTA-informed yield placement → yield-point explosion; and it is **data-blind** — it cannot pick the schedule that also needs a specific nondeterministic input to reach the bug. A blind sweep therefore times out or misses (see §3).

**Fix.** PTA-filter yield points to shared accesses; keep it only as a cheap first-pass / the **replay substrate** for A1's extracted schedule. It is NOT a standalone scorer.

**Primary sources.** CHESS OSDI 2008 https://www.usenix.org/legacy/event/osdi08/tech/full_papers/musuvathi/musuvathi.pdf ; PPoPP 2014 https://www.doc.ic.ac.uk/~afd/homepages/papers/pdfs/2014/PPoPP.pdf .

### A5 — GraphML-1.0 concurrency-witness emitter → fixes `graphml-witness` (mandatory, else every FALSE is +0)
**Algorithm.** Serialize the replayed schedule to GraphML v1.0: per error-path CFA transition emit a `threadId`; at each `pthread_create` emit a `createThread` edge (new thread id); include `enterFunction`/`returnFromFunction` for callstack/termination and `startline`. Validate with CPAchecker Validator 4.0 / Dartagnan / ConcurrentWitness2Test. **The execution-based ConcurrentWitness2Test is SAF's best target** — it inserts yield/release at threadId-change points, compiles, runs up to 100×; SAF's replayed schedule maps directly onto those change points.

**Maps to SAF.** The `threadId` sequence comes straight from A1's `rl` matrix (or A3's trace). Without this, a correct concurrency FALSE lands +0 (raw), not +1.

**Primary sources.** GraphML witness format https://github.com/sosy-lab/sv-witnesses/blob/main/README-GraphML.md ; CPAchecker multithreaded witnesses https://link.springer.com/chapter/10.1007/978-3-030-61362-4_26 ; ConcurrentWitness2Test https://github.com/ftsrg/ConcurrentWitness2Test .

---

## 2. TRACK B — Data races (no-data-race FALSE) — the missing FIND→SCHEDULE→CONFIRM pipeline

The core diagnosis: SAF has only the CONFIRM step (a TSan-style detector) and no FINDER to point it at a racing pair, and no forced schedule to make that pair concurrent. **A dynamic HB detector fires only if the two conflicting accesses actually run concurrently in the replayed schedule — under the default OS schedule they usually don't (short programs serialize; thread-exit/join creates HB). → 0% recall.** The fix is to add the two missing stages, reusing SAF's Andersen PTA and existing detector.

**SV-COMP `no-data-race` definition (rules.php):** `CHECK( init(main()), LTL(G ! data-race) )`. A data race = two concurrent accesses to the same location, ≥1 a write, not all atomic, not ordered by happens-before / not protected by a common lock.

### B1 (FIND, TOP ROI) — Static lockset analysis (Eraser/Goblint) on AIR + Andersen PTA → fixes half of `race-confirmer`
**Algorithm.** Forward dataflow on the ICFG with a lockset domain `2^{pts(mutex-arg)}`:
- `pthread_mutex_lock(m)` → add `pts(m)`; `unlock(m)` → remove `pts(m)`.
- At each load/store record `(instr, kind, current_lockset, threadId)`.
Then for each pair of accesses whose pointer points-to sets **intersect** (Andersen PTA = same location may-alias), with ≥1 write, and **empty must-lockset intersection** → **race candidate**. (Eraser's virgin/exclusive/shared/shared-modified state machine suppresses init-idiom false alarms.)

**Why it works.** A non-empty common lock serializes the accesses; empty intersection ⇒ no serializing lock ⇒ possibly concurrent. Over-approximates (SAF's CONFIRM step is the soundness gate).

**Maps to SAF.** SAF has Andersen PTA (same-location test) and ICFG (lock transfer functions). ~200–400 lines of dataflow. Output = ranked `(a0, a1, shared_location)` candidates.

**Primary sources.** Eraser SOSP 1997 https://cseweb.ucsd.edu/~savage/papers/Sosp97.pdf ; Goblint TACAS 2021 https://link.springer.com/chapter/10.1007/978-3-030-72013-1_28 ; digest paper https://arxiv.org/abs/2511.11055 ; region analysis https://goblint.cs.ut.ee/assets/papers/regions.pdf .

### B2 (FIND, prune) — May-happen-in-parallel / active-threads analysis
**Algorithm.** Build the thread-creation graph from `pthread_create` (PTA resolves the callback); track a may-active / must-active thread set per program point (create adds; join on all paths removes). Two accesses MHP iff each thread's point is live while the other thread is active. Goblint's TID digest (ST_main / MT_main / MT) + join digest prunes candidates that can only occur sequentially (the digest paper reports ~5× improvement combining lockset+TID+join). Filters B1's candidates so CONFIRM isn't wasted on non-concurrent pairs.

**Primary sources.** digest paper https://arxiv.org/abs/2511.11055 ; RacerF https://arxiv.org/pdf/2502.04905 ; improving thread-modular SAS 2021 https://link.springer.com/content/pdf/10.1007/978-3-030-88806-0_18.pdf .

### B3 (SCHEDULE, the critical missing piece) — force the racing pair concurrent
**Algorithm.** For a candidate `(a0∈T0, a1∈T1)`, stage a rendezvous just before each access so both fire "simultaneously" with **no HB edge between them**. Use a **relaxed-atomic busy-wait** (C11 `__ATOMIC_RELAXED` store/load spin), NOT `pthread_barrier_wait` (a barrier establishes HB and would hide the race). T0 sets flag_t0 (relaxed), spins on flag_t1; T1 symmetric; both proceed together. Relaxed atomics carry no synchronization semantics, so TSan sees the two conflicting accesses unordered → fires. Injected via an `LD_PRELOAD`/instrumentation shim keyed to the FIND locations. This is a CHESS-style targeted preemption specialized to two points.

**Produces a replayable schedule?** YES — the rendezvous IS the schedule; it serializes to a GraphML witness (threadId at each access).

### B4 (CONFIRM, already in SAF) — TSan-style happens-before replay
**Algorithm.** Vector-clock / FastTrack epochs + shadow memory: each sync op (lock/unlock, create/join, atomics) establishes HB; a race = two same-location accesses, ≥1 write, not HB-ordered, not atomic. This is SAF's existing substrate — improving it in isolation is 0 ROI; it needs B1+B2 to target it and B3 to make the pair concurrent.

**Primary sources.** TSan algorithm https://github.com/google/sanitizers/wiki/threadsanitizeralgorithm ; FastTrack (Flanagan & Freund, PLDI 2009); TSan paper https://research.google.com/pubs/archive/35604.pdf .

**Difficulty for the pipeline: MEDIUM (B1/B2) + MEDIUM-HARD (B3) / ROI HIGH but smaller pool** — do AFTER the ConcurrencySafety-Main reuse arms (per plan-202: no-data-race is a smaller, all-concurrency pool). Guardrails: abstain on weak-memory / OpenMP / un-modeled sync (fail-closed); every FALSE gated by the CONFIRM firing.

---

## 3. DIAGNOSIS — why SAF's current concurrency levers score 0

**`conc-shim-m1` (blind CHESS c≤1→c≤2 sweep) scores 0 because:**
1. **Yield-point explosion / no PTA filtering.** On the native binary it can't tell which instructions touch shared memory, so yields land everywhere. With Y yield points and bound c, schedules ≈ O(Y^c·T^c); for a trivial 3-thread program this is tens of thousands of full binary runs → SV-COMP 900 s timeout blown. PTA (which SAF has, but only on AIR) is not connected to the binary's yield placement.
2. **Data-blind search.** Most SV-COMP concurrency FALSEs need a *specific schedule AND specific nondet inputs* jointly. A schedule sweep cannot solve for the input; only a solver (A1 sequentialization+BMC / A2) reasons over schedule and data together.
3. **Architecture mismatch.** Analysis (PTA) runs on AIR; execution (shim) runs on the compiled binary; there is no map from "AIR var is shared" to "binary address is a yield point." Sequentialization avoids this by pushing the transformed program through the same compiler path.
4. **c≤2 is fine as coverage (≈K=3 rounds); the killer is 1–3, not the bound.**

**`conc-seq-m2` (lazy sequentialization) is architecturally CORRECT but scores 0 because the BMC integration is incomplete:** the run-length `rl[i][r]` must be a genuine nondeterministic choice the SOLVER assigns (not a runtime constant/random); yields must fire only at PTA-shared accesses; mutex/create/join must be modeled; and the SAT model must be decoded to a schedule, replay-confirmed on the original binary, and emitted as GraphML. Fix these and it becomes the Lazy-CSeq-class finder.

**`race-confirmer` (blind TSan replay) scores 0 because it is only the CONFIRM step** — no static lockset+MHP FINDER to name the racing pair, and no forced schedule (B3) to make the pair concurrent. Under the default schedule short programs serialize (thread-exit/join creates HB) so the detector never fires.

**`graphml-witness` scores 0 downstream of the above** — with no confirmed FALSE to witness, and (independently) it must emit correct `threadId`/`createThread` for CW2T/Dartagnan/CPAchecker to confirm, else every concurrency FALSE is +0 even when correct.

**Net:** the levers are individually plausible but were each deployed in isolation. Concurrency FALSE needs the **whole pipeline**: DIRECTED finder (A1 solver, not blind sweep) → explicit schedule → native replay confirm → GraphML witness. Data-race FALSE needs FIND(lockset+MHP) → SCHEDULE(forced rendezvous) → CONFIRM(TSan) → GraphML.

---

## 4. TRACK C — Termination (raise TRUE recall; optional FALSE)

SAF's plan-201 proof emits TRUE only on loop-free ∧ acyclic-callgraph (abstains on ~84% → ~16% recall). **The dominant lever is linear ranking-function synthesis for single loops** (TRUE needs no witness → pure recall, no −16/−32 risk if the ranking check is sound). FALSE (non-termination) is lower ROI (needs a maturing GraphML witness).

### C1 (TOP ROI) — Linear ranking-function synthesis for single loops (Podelski–Rybalchenko / Farkas+Motzkin+Z3) → extends `termination-recall`
**Algorithm.**
1. **Extract natural loops** from SAF's CFG (back-edge `u→h`, `h` dominates `u`).
2. **Build the transition relation** `LOOP(x,x')` as linear inequalities over the loop's integer AIR vars: guard `g(x)` + each linear update `x_i' = a·x + c`.
3. **Template** `f(x)=s0 + Σ s_i x_i`. Conditions: `LOOP → f(x) ≥ 0` (bounded below) and `LOOP → f(x) − f(x') ≥ 1` (strict decrease, δ=1 for integers).
4. **Eliminate the ∀** via **Farkas' lemma** (non-strict) / **Motzkin's transposition theorem** (mixed strict/non-strict): the "for all loop states" becomes an existential over ranking coeffs `s` and non-negative multipliers `λ,μ`. Result = an existential (bi)linear arithmetic constraint. Pragmatic path on SAF's Z3: assert the exists-forall directly and let Z3's `qe`/`nlsat` discharge it; or hand the Motzkin-reduced existential to `nlsat`.
5. **SAT ⇒ ranking function ⇒ loop terminates.** Extend plan-201: TRUE if `(loop-free ∧ acyclic)` OR `(every natural loop is ranked ∧ acyclic callgraph)`. Keep the structural fast-path.

**Why it works.** `f≥0` and `f` strictly decreasing by ≥δ ⇒ at most `f(x0)/δ` iterations ⇒ terminates. A valid ranking function is a complete termination proof (no approximation, no soundness risk). LinRF over rationals is PTIME/complete (Podelski–Rybalchenko).

**Maps to SAF.** Natural-loop extraction (CFG, exists), linear TR from AIR, Z3 (exists). No new deps. Handles the bulk of `termination-numeric` / `termination-crafted` (count-down-to-bound loops). Expected recall ~16% → 50%+ on MainControlFlow + numeric.

**Primary sources.** Podelski–Rybalchenko VMCAI 2004 https://link.springer.com/chapter/10.1007/978-3-540-24622-0_20 ; Leike–Heizmann ranking templates https://arxiv.org/abs/1503.00193 .

**Difficulty MEDIUM / ROI VERY HIGH.**

### C2 — Lexicographic / multiphase ranking (single linear fails) → follow-on to C1
**Algorithm.** A tuple `(f1..fk)` ordered lexicographically; on each transition the leftmost component strictly decreases while lower-indexed ones don't increase. Motzkin-reduce the templated disjunction to an existential SMT query; start k=2. 2LS does this bit-precisely (SAT/BV backend → correct on unsigned-wraparound loops). Adds ~+20–30% of terminating loops A1 misses.

**Primary sources.** Leike–Heizmann https://arxiv.org/abs/1503.00193 ; 2LS interprocedural https://arxiv.org/abs/1505.04581 .

### C3 — SCC-decomposition + per-SCC ranking (AProVE-style) → generalize C1 to nested/multiple loops
**Algorithm.** Decompose the (I)CFG into SCCs; each non-trivial SCC = a loop/recursion cycle → apply C1/C2 per SCC (as an integer transition system). All SCCs ranked (or unreachable) ⇒ program terminates. SAF's plan-201 already computes callgraph SCCs for recursion detection — reuse that spine.

**Primary sources.** AProVE modular https://arxiv.org/html/2302.02382 ; C-programs https://link.springer.com/chapter/10.1007/978-3-319-08587-6_13 .

### C4 (DEFERRED) — Büchi/trace-abstraction (Ultimate) — full framework
Decompose all infinite runs into lasso modules, rank each, subtract proven-terminating runs via Büchi automata difference until none remain. State-of-the-art recall but needs automata complementation/intersection infra SAF lacks. **Implement C1–C3 loop-wise first; Büchi is research-scale.** Sources: Ultimate PLDI 2018 https://dl.acm.org/doi/10.1145/3296979.3192405 ; Büchi Automizer https://ultimate.informatik.uni-freiburg.de/downloads/BuchiAutomizer/ .

### C5 (LOW ROI) — Non-termination FALSE via recurrent set / lasso
**Algorithm.** A **recurrent set** `G` (non-empty, reachable, closed under the loop transition: every `x∈G` has a successor in `G`) certifies non-termination. Template `G={x|c·x≥d}`, encode closure as a Z3 constraint, check reachability; or a trivial `while(1)`/monotone-away-from-guard check for the easy cases; or geometric non-termination arguments (eigenvalue ≥1). Emit a stem+loop lasso witness. **Deprioritized:** termination-FALSE needs a validated (still-maturing) GraphML witness and risks −16; recall lives mostly in TRUE.

**Primary sources.** Geometric non-termination https://arxiv.org/abs/1405.4413 ; recurrent sets https://link.springer.com/chapter/10.1007/978-3-662-49674-9_2 ; LoAT https://arxiv.org/abs/1905.11187 .

---

## 5. Refined `levers.tsv` specs (drop-in replacements / additions)

TAB-separated: `id <TAB> mode <TAB> family <TAB> scope <TAB> description`. These sharpen the existing concurrency/termination levers with the mechanism + fail-closed guardrails + the pipeline ordering. Sequence: A1→A5 (finder→witness) unlock ConcurrencySafety-Main; C1 unlocks termination recall; B1–B4 last (smaller pool).

```
conc-seq-m1	capability	unreach-call	crosscut	ConcurrencySafety-FALSE FINDER (Lazy-CSeq bLMP): AIR->AIR lazy round-robin sequentialization (pc-labeled re-enterable thread fns + static per-thread local cells + round-robin driver) where run-length rl[i][r] is a GENUINE nondeterministic choice the SOLVER assigns; yields fire ONLY at PTA-identified shared accesses + sync/create/join; encode to Z3 (loops unwound U), SAT -> read rl matrix as an explicit schedule. Under-approx of the K-round schedule space, no data over-approx -> tightening K/U/L costs recall not soundness. REPLACES the blind conc-shim sweep as the finder. Fail-closed: SC-only, abstain on pthread-wmm/C11-relaxed/dropped #pragma omp; reachability-gate dead pthread_create scaffolding. AIR->AIR pass -> all-property gated.
conc-replay-confirm	capability	unreach-call	local	ConcurrencySafety confirmer: decode the solver's rl schedule to a (threadId,steps) vector; a pthread interposer (intercept create/join/mutex/cond, gate each op on a shared turn-counter) forces THAT exact interleaving on the ORIGINAL unmodified binary under ASan/UBSan. Mis-modeling can only cost recall (replay fails -> abstain), never a wrong FALSE. Confirmation-on-real-binary = SAF's sequential doctrine. Abstain on any divergence between replay and recorded schedule.
conc-shim-firstpass	capability	unreach-call	local	Cheap PTA-filtered CHESS first-pass: preemption-bounded sweep (c<=1->c<=2) with yields ONLY at PTA-shared accesses, ASan/UBSan on; also serves as the replay substrate for conc-replay-confirm. NOT a standalone scorer (data-blind) -> keep behind conc-seq-m1. Metric: FALSE recall at FA=0.
graphml-witness	local	capability	unreach-call	local	GraphML-1.0 concurrency-witness emitter from the confirmed schedule: threadId per error-path CFA transition + createThread edges + enterFunction/returnFromFunction + startline. Target ConcurrentWitness2Test (execution-based, best SAF fit) / Dartagnan / CPAchecker-Validator-4.0. Without it every concurrency FALSE is +0 not +1. Verify >=1 concurrency validator active in the target year.
race-find	capability	no-data-race	local	no-data-race FALSE FINDER (Eraser/Goblint lockset + MHP): ICFG dataflow with lockset domain 2^pts(mutex-arg) (lock adds pts(m), unlock removes); at each load/store record (instr,kind,lockset,tid); Andersen-PTA same-location may-alias pairs with EMPTY must-lockset intersection + >=1 write + may-happen-in-parallel (create/join active-thread + TID/join digest) => race candidate. Over-approx; CONFIRM is the soundness gate. Eraser virgin/exclusive/shared/shared-modified suppresses init-idiom FPs.
race-schedule	capability	no-data-race	local	no-data-race SCHEDULE: for a candidate (a0@T0,a1@T1) stage a relaxed-atomic busy-wait rendezvous (C11 __ATOMIC_RELAXED spin, NOT pthread_barrier_wait which creates HB) just before each access so both fire concurrently with NO HB edge -> TSan observes the unordered conflicting pair and fires. Injected via LD_PRELOAD/instrumentation keyed to the FIND locations. The rendezvous IS the witness schedule.
race-confirmer	capability	no-data-race	local	no-data-race CONFIRM (TSan-style FastTrack HB): vector-clock epochs + shadow memory; race = same-location, >=1 write, not HB-ordered, not atomic. SAF's existing substrate — 0 ROI alone; needs race-find + race-schedule to target and stage the pair. Every FALSE gated on the detector firing. Smaller all-concurrency pool -> do AFTER the ConcurrencySafety-Main reuse arms. Abstain on weak-memory/OpenMP/un-modeled sync.
termination-recall	tuning	termination	local	Raise termination-TRUE recall via LINEAR RANKING-FUNCTION synthesis (Podelski-Rybalchenko / Farkas+Motzkin+Z3) for single natural loops: extract loop from CFG (back-edge/dominator), build linear transition relation LOOP(x,x') from AIR, template f=s0+sum(s_i x_i), require LOOP->f>=0 and LOOP->f-f'>=1, eliminate the forall via Motzkin, discharge the existential with Z3 (qe/nlsat), extract coeffs. TRUE if (loop-free AND acyclic) OR (every natural loop ranked AND acyclic callgraph). A valid ranking function is a COMPLETE termination proof (no soundness risk); TRUE needs no witness (+2 on verdict). PRESERVE plan-201 hardening (abstain on llvm.global_ctors/dtors + any reachable block with no terminator). Follow-ons: lexicographic/multiphase (k=2), SCC-decomposition per-loop.
```

Notes: (1) `graphml-witness` family field kept as `unreach-call` for the loop's family gate (concurrency FALSEs score in ConcurrencySafety-Main = unreach-call). (2) All concurrency arms remain **crosscut/all-property gated** where they touch AIR (sequentialization pass); the confirmer/witness are **local**. (3) Cross-cutting invariants unchanged: gate (c) on the FULL SAFE reservoir; no benchmark-path/function-name keying; held-out svcomp26 hard-reject; abstain on any un-modeled primitive.

---

## 6. Recommended build order (ROI-ranked)

1. **`termination-recall` C1 (linear ranking, single loop).** Highest ROI overall: pure recall, no witness, no −16/−32 risk, reuses CFG+Z3, big pool. Then C2/C3 as follow-ons.
2. **ConcurrencySafety-Main pipeline A1→A2(confirm)→A5(witness):** `conc-seq-m1` (directed finder) → `conc-replay-confirm` → `graphml-witness`. This is where the 3 existing concurrency levers actually start scoring. `conc-shim-firstpass` is a cheap fallback/substrate, not a scorer.
3. **A2/A3 backend upgrades (Deagle ordering-consistency / source-DPOR)** only if A1's plain unrolling is too weak on harder tasks.
4. **no-data-race pipeline B1→B2→B3→B4** (`race-find`→`race-schedule`→`race-confirmer`) — LAST, smaller pool, after the ConcurrencySafety-Main reuse arms pay off.
5. **Termination FALSE (C5) and weak-memory (A2-CAT full)** — deferred; low ROI / SC-sufficient.

## 7. Primary-source index
Concurrency: Lazy-CSeq https://dl.acm.org/doi/full/10.1145/3478536 · Deagle https://feihe.github.io/materials/tacas-svcomp22.pdf · Dartagnan https://link.springer.com/chapter/10.1007/978-3-030-45237-7_24 + validator https://hernanponcedeleon.github.io/pdfs/svcomp2022.pdf · Nidhugg/source-DPOR https://dl.acm.org/doi/10.1145/2535838.2535845 · GenMC https://link.springer.com/chapter/10.1007/978-3-030-81685-8_20 · CHESS https://www.usenix.org/legacy/event/osdi08/tech/full_papers/musuvathi/musuvathi.pdf · GraphML witness https://github.com/sosy-lab/sv-witnesses/blob/main/README-GraphML.md · ConcurrentWitness2Test https://github.com/ftsrg/ConcurrentWitness2Test · SV-COMP 2025 report https://www.sosy-lab.org/research/pub/2025-TACAS.Improvements_in_Software_Verification_and_Witness_Validation_SV-COMP_2025.pdf .
Data race: Eraser https://cseweb.ucsd.edu/~savage/papers/Sosp97.pdf · Goblint https://link.springer.com/chapter/10.1007/978-3-030-72013-1_28 · digest https://arxiv.org/abs/2511.11055 · TSan https://github.com/google/sanitizers/wiki/threadsanitizeralgorithm .
Termination: Podelski–Rybalchenko https://link.springer.com/chapter/10.1007/978-3-540-24622-0_20 · Leike–Heizmann https://arxiv.org/abs/1503.00193 · 2LS https://arxiv.org/abs/1505.04581 · AProVE https://arxiv.org/html/2302.02382 · Ultimate https://dl.acm.org/doi/10.1145/3296979.3192405 · recurrent sets https://arxiv.org/abs/1405.4413 .
Rules/scoring/witness: https://sv-comp.sosy-lab.org/2025/rules.php .
