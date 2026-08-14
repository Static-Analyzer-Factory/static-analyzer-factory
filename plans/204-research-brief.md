# ENRICHMENT BRIEF — Plan 204 Autonomous Loop: ConcurrencySafety-FALSE, Recall/Witness Levers, and the Capability-Track Gate

Prepared for folding into SAF plan 204 and the loop's arm prompt. Confidence is stated per section. Two load-bearing briefs were **REFUTED** by the adversarial verdicts; those refutations are surfaced prominently below, not smoothed over.

---

## 1. SCORING TRUTH — Does ConcurrencySafety-FALSE count in C.FalseOverall?

**VERIFIED ANSWER: YES. A correct, confirmed FALSE on a ConcurrencySafety task counts in C.FalseOverall.** (Confidence: **high** — verified against SV-COMP 2025 + 2026 primary rules/benchmarks pages and both TACAS reports.)

- C.FalseOverall is defined as "**all verification tasks** [where] the results 'correct TRUE' and 'incorrect TRUE' are **not counted**." It is partitioned by **result type** (TRUE results dropped), not by task subset. C.Concurrency is explicitly one of its six base categories (C.ReachSafety, C.MemSafety, C.Concurrency, C.NoOverflows, C.Termination, C.SoftwareSystems).
- Therefore a **reachable-violation FALSE in a concurrent program** (reach_error under some interleaving / ASan / UBSan) earns **+1** in C.FalseOverall; a wrong FALSE is **−16**.
- **no-data-race TRUE** scores only in the dual **C.TrueOverall** — worthless to SAF. But **no-data-race FALSE** (a real, reachable data race) DOES count in C.FalseOverall (needs its own race confirmer — later slice).

**Refuted/corrected sub-claim (do NOT paper over):** The brief's phrasing "C.FalseOverall = all **non-Termination** categories" is **STALE for 2026**. SV-COMP **2026 renamed FalsificationOverall → C.FalseOverall and now ALSO includes Termination** (and introduced C.TrueOverall). This does not weaken the go signal — concurrency FALSE still counts in both editions — but the plan text must say "all C categories, TRUE results dropped," not "all non-Termination." (This also means SAF's R7 termination-TRUE scores in C.TrueOverall/Termination, not C.FalseOverall, unchanged.)

**Implication:** Lifting the blanket concurrency abstention is **GO**. Three of the four ConcurrencySafety sub-categories (Main/MemSafety/NoOverflows) test unreach-call/memsafety/overflow — properties SAF already confirms concretely. This is real in-scope C.FalseOverall points SAF is currently forfeiting 100%.

---

## 2. THE CAPABILITY-TRACK GATE FIX (the core plan-204 change)

**Problem (verified against plan-204 lines 64/113):** Gate (d) requires `CONFIRMED-score delta on TRAIN > 0` (strict `>`) for **every** kept arm, and arms are atomic keep-or-revert with **no cross-arm accumulation**. A multi-slice capability (concurrency confirmer; slicing+symex engine) whose Slice-0 spine scores **0** gets reverted before the payoff slice can land. The loop is structurally confined to greedy tuning of already-wired properties — exactly the wrong shape for SAF's two biggest gaps (concurrency abstention; ~0% on new-2026 unreach). Confidence: **high**.

### The minimal sound fix: split gate (d) into two orthogonal predicates

Keep the **SOUNDNESS predicate ABSOLUTE and byte-identical for every arm**; make only the **PROGRESS predicate** mode-dependent.

- **SOUNDNESS (immutable, both modes):** gate (a) immutable-file sha256 intact; gate (b) holdout-not-read audit; gate (c) re-run the immutable scorer with **FP=0 AND wrong-TRUE=0**; nextest/clippy/fmt green. The non-LLM supervisor computes these; the agent can never run or bypass them.
- **PROGRESS (mode-dependent):**
  - **tuning arm** → unchanged: `CONFIRMED train-delta > 0`.
  - **capability arm** → milestone contract instead of score-delta.

### Supervisor decision rule (tuning vs capability)

Deterministic, supervisor-owned, **logged every arm** — never an LLM judgment, never a silent default:

1. The supervisor picks the mode at **lever-selection time** from a **human-curated lever taxonomy**: lever targets an already-wired scoring property (witness-confirmation %, memsafety input steering, overflow Slice-2) → **tuning**; lever names a not-yet-wired property or new engine (ConcurrencySafety-FALSE, slicing+symex) → **capability** with a declared milestone ladder.
2. Each arm writes an `arm.json` contract declaring `mode ∈ {tuning, capability}`, target property/family, and (capability) the milestone predicate: which SAFE slice to run for FP=0 and which structural unit-test/target-family FALSE to check.
3. **Consistency check** (removes the incentive to mislabel): a capability-flagged arm that *does* move aggregate score is folded in as success (no harm); a tuning-flagged arm that fails to move score reverts as today. The only asymmetry a capability flag buys: a **zero-delta arm may survive IFF capability-flagged AND meets its milestone AND touches only a `cap-*` branch**.

### Milestone acceptance for a capability arm

Replace "score rose this arm" with a deterministically-checked contract:

1. **FP=0 on the target family's held-out SAFE set** — run the immutable scorer restricted to that property's expected-TRUE tasks. **Run on the FULL immutable SAFE reservoir for that family, never a subsample** (see §2 refutation below).
2. **Either** a declared structural milestone advanced (a unit test the arm adds), **or** (terminal arm) **≥1 newly-confirmed FALSE** on the target family that baseline missed, validated by the **same immutable witness/confirmation pipeline** the aggregate score uses.
3. Capability arms land on an **accumulating `cap-*/<capability>/<n>` branch lineage** (never reverted between scaffolding arms, never auto-merged to svcomp), tagged `not-yet-scoring` in the journal, so arm N+1 builds on arm N.

### Anti-gaming safeguards

- Gates (a),(b),(c) fire on capability arms too — a fabricated "newly-confirmed FALSE" fails witness confirmation and/or the SAFE-set FP audit.
- Any unit test a capability arm adds is **run under the FP audit on the SAFE set** (a test that weakens a confirmer surfaces as an FP).
- **No benchmark-path / function-name / task-id keying** — add this as an explicit redline in `arm_prompt.md`; confirmers must be property-general. (This closes the memorization exploit — see §2 refutations.)
- WebSearch/WebFetch scoped to ACT step only, deny-by-default egress (model + search endpoint), no arbitrary curl, no git push; every fetched URL logged into the same transcript audit that checks holdout-not-read.

### De-prioritization rule (non-converging track)

**Milestone-budgeted circuit breaker with a PARKED state, not a hard kill.** Each capability track gets a budget (≈6–8 arms / bounded tokens per milestone). Advance a milestone within budget → refill for the next. Consume budget with **no milestone advance** → **PARK**: keep the `cap-*` branch for human review, deprioritize the lever in ROI selection, rotate to another lever. **Graduation:** once a capability's confirmed FALSEs raise the **aggregate svcomp26 HELD-OUT score** (step 6, the real signal), its confirmer joins the baseline and future arms on it are scored as tuning. This matches SAF history where R4/R8 were correctly abandoned but their scaffolding taught the next slice.

### Two REFUTATIONS the plan must absorb (do not paper over)

- **REFUTED — "no new exploit surface is opened because every reward-hacking exploit attacks soundness/scorer integrity, not the score-delta."** This universal premise is **false**. SpecBench and EvilGenie document the dominant coding-agent exploit class — **memorization/overfitting that legitimately raises the measured score without touching the scorer** (hash-table "compilers" memorizing test inputs; benchmark-keyed heuristics). The train-score-delta **IS** an exploit surface — arguably THE dominant one. **Fix:** do NOT justify the relaxation with "no exploit attacks score-delta." Justify it by (i) soundness is enforced by (a)–(c) independently of (d), and (ii) the real anti-overfit control is the **frozen svcomp26 held-out gate + origin-family (not per-task) splitting**, which must be a **HARD reject** for any arm whose train gain does not reproduce on held-out — plus the no-keying redline above. (Confidence: high.)
- **REFUTED — "a capability arm's milestone is a *strictly stronger* soundness bar, so accepting zero-delta capability arms cannot ship a wrong verdict."** **False as stated.** Capability and tuning arms clear the **same** gate (c); the capability arm is only *weaker on progress*. Worse, **FP=0-on-a-sample ≠ sound**: SAF's own plan-203 history shows sample FP=0 (N=40–60) that was FALSE at scale — 38 CWE761 false alarms (−608), each **confirmed by the immutable ASan pipeline** because the sv-benchmarks ldv_str*/ldv_mem* models omit the terminating NUL. **"Confirmed by the witness pipeline" does NOT imply "correct verdict."** And a new confirmer is the *higher-risk* class, not lower. **Fix (already folded into milestone acceptance above):** run gate (c) on the **FULL** immutable SAFE reservoir for the family, never a subsample; scan any new confirmer against the entire SAFE pool for latent FP classes before keep; the value comes from full-set gate (c) + the held-out gate, not from any intrinsic "capability > tuning" ordering. (Confidence: high.)

The salvageable kernel: accepting zero-**train**-delta capability arms is sound **only** when "delta" is redirected to held-out recall / witness-confirmation-% (positive held-out signal, not literal zero progress) AND gate (c) runs on the full reservoir.

---

## 3. CONCURRENCYSAFETY-FALSE — Recommended Sound Mechanism

**Ranked mechanism (soundness × reuse × cost):** Confidence **high** — the sequentialization soundness direction and the replay-composability are primary-source confirmed.

1. **Bounded round-robin lazy sequentialization (AIR→AIR transform) as the FINDER.** Reimplement the Lal-Reps / La Torre-Madhusudan-Parlato / Lazy-CSeq schema from scratch (REQ-IP-001): pc-labeled re-enterable thread functions; per-thread **static local cells** so locals keep real concrete values across yields; round-robin driver with nondeterministic yield; bounds K rounds, N threads. **Soundness for FALSE is proven:** the transform is an **under-approximation of the schedule space with NO data/state over-approximation** — "all reachable states of the sequential program correspond to reachable states of the concurrent program." Any violation it finds is a **genuine** concurrent violation; tightening K/L costs only recall, **never a spurious FALSE**.
2. **Deterministic-replay scheduler shim as the CONFIRMER** (the composability lever). The violating path yields an explicit **(round, thread, steps-before-yield) schedule vector**. A small pthread-interposer (intercept create/join/mutex/cond + yield points, gate each op on a shared turn-counter) forces that exact interleaving on the **real, un-transformed binary** under ASan/UBSan. **Confirmation happens on the real binary**, so a mis-modeling transform can only cost recall (replay fails → abstain), **never a wrong FALSE** — this is the −16/−32 guarantee, identical to SAF's sequential "prove-reach OR concretely-reproduce" doctrine. Existence proof: Sthread (drop-in pthread replacement, deterministic replay from a thread-id witness).
3. **CHESS-style preemption-bounded concrete testing** — the cheapest first-pass finder AND the replay substrate (it *is* the shim). Bug reachability is shallow: most SV-COMP concurrency bugs manifest within few context switches, so a c≤1 then c≤2 preemption sweep catches a meaningful fraction at zero new oracle.
4/5. **BMC-with-memory-model** and **SMC/DPOR** are later recall boosters kept behind the same replay confirmer — NOT trusted as oracles.

**First milestone (de-risk arm, throwaway VM branch, plan-198/202 style):** build the **replay shim ALONE**, run it as a CHESS-style c≤1→c≤2 blind sweep over the ConcurrencySafety-FALSE reservoir with ASan/UBSan on. **Metric: FALSE recall at FA=0.** This de-risks the confirmer independently of the transform, reuses SAF's oracle verbatim. If a shallow sweep already catches a non-trivial fraction at 0 false alarms, ship it before building the transform. Milestone 2 = the AIR→AIR sequentialization pass, every proposed FALSE re-confirmed by the shim on the original binary before emission.

**Why it composes with SAF's replay confirmer:** the transform makes concurrency "sequential," so ASan-on-a-reachable-overflow / UBSan-on-signed-overflow / plain reachable-assert all fire in the transformed program with **zero new confirmer machinery**; the shim re-confirms on the real binary for −16/−32 safety.

**SC is sufficient (verified):** SV-COMP ConcurrencySafety-C is **sequentially consistent** (interleaving semantics; Lazy-CSeq won the category 2014/15/20/21 as a pure-SC tool). **Do NOT build weak-memory (TSO/PSO/C11-relaxed) machinery for the first capability.**

**Hard soundness guardrails (arm invariants, fail-closed):**
- (a) **Native x86 replay is TSO** — cannot witness weak-memory-only violations; **ABSTAIN on pthread-wmm / C11-relaxed** rather than risk a verdict.
- (b) Reachability-gate dead pthread_create scaffolding (reuse plan-198 `reachable_spawns_threads`).
- (c) Dropped `#pragma omp` / OpenMP → **fail-closed abstain** (plan-202 lesson).
- (d) `__VERIFIER_atomic_begin/end` must be modeled uninterruptible; any un-modeled sync primitive → abstain (encoding-fidelity, not the round bound, is the real spurious-FALSE risk).
- (e) Abstain on any divergence between shim replay and the recorded schedule.

### Concurrency witness emission & confirmation — a REAL correction

**Confidence high.** SAF's YAML-2.0 witness pipeline and its **cpa-witness2test execution validator DO NOT WORK for concurrency violations.** Witness Format 2.0 explicitly: "*Violation witnesses have not yet been defined for concurrency safety.*" Concurrency FALSE must be witnessed in **GraphML format 1.0** carrying thread-schedule keys (`threadId` = active thread per transition; `createThread` = thread-creation edge; `startline`).

- **Confirmers for concurrency violation witnesses:** CPAchecker-**ThreadingCPA** and **Dartagnan** — **plus** (the brief omitted this) **ConcurrentWitness2Test** (ftsrg), an execution-based concurrency validator. So SAF's execution-based/replay identity is **NOT inherently disqualified** — the disqualifier is specifically that cpa-witness2test targets *sequential* reachability and Witch3 targets a format that lacks concurrency.
- **Action:** build an independent **GraphML-1.0 emitter** that serializes the exact interleaving SAF replayed (threadId sequence + createThread edges + startline). SAF's replay **already knows the schedule** (it executed it) — that is exactly what the witness must serialize. Without this emitter, every concurrency FALSE lands **raw/unconfirmed (+0, not +1)**.
- **Confirm the active concurrency validators for the target year (2027)** before relying on GraphML-1.0 confirmation — the 2024 report noted ConcurrencySafety had <3 validators.

**Measurement:** reuse svcomp_split.py on a concurrency holdout; report **RAW-FALSE vs CONFIRMED-FALSE separately**; **wrong-FALSE=0 is the hard gate** on race-free/safe concurrent tasks. Do not count RAW gains as C.FalseOverall points until CPAchecker/Dartagnan/ConcurrentWitness2Test confirm.

---

## 4. RECALL + WITNESS-CONFIRMATION LEVERS (ranked mechanisms)

### (b) Fix SAF's ~0% on the new-2026 unreach batch — but with a corrected root cause

**REFUTED — "the 2026-new miss is dominated by tasks needing a solved nondet input / data-dependent loop count (BMC/KLEE territory), so bounded path-solving is the *dominant* recall lever."** The verdict inverts this. The marquee new-2026 unreach content is the **Intel TDX Module firmware set (~418 tasks, HarnessForge)**; its hardness drivers are **STRUCTURAL**, not scalar-nondet-solving: (1) frontend fragility on anonymous unions / nested types (top verifiers had to fix these to avoid **quiet wrong answers**); (2) **complex-object havocking** (severe enough to motivate the new `__VERIFIER_nondet_memory()` primitive in 2026); (3) harness construction + preconditions on a physical-address metadata table; (4) property-directed slicing. Also ~30% of TDX cover points are **unreachable/expected-TRUE** — out of scope for a FALSE-finder entirely. (Confidence: high, though per-task attribution of SAF's exact misses was not directly available — measure before betting.)

**Directive:** treat bounded path-solving as **ONE** lever, **not the dominant bet** for 2026-new. **Prioritize / pair it with**: harness/entry-point synthesis, complex-object havocking / memory-state construction (`__VERIFIER_nondet_memory`-style), and robust frontend handling of unions/nested types. **Split the 2026-new batch by shape and attribute recall per shape** before committing heavily (ARM 6).

**Ranked recall mechanisms (all keep SAF's replay as the sole arbiter → soundness-neutral):**

1. **[capability] Harness/entry-point synthesis + complex-object havocking** — the actual binding constraint on the TDX firmware set. Build a valid harness and havoc complex structs so the sink is reachable, then replay. Highest ROI for 2026-new.
2. **[capability] Backward program SLICING over AIR** (Symbiotic mechanism) — instrument the property as reachability of one error location, compute the backward control+data-dependence closure via existing points-to, discard everything else. This is the **scalability multiplier** that makes symbolic path-solving finish in budget and shrinks the witness. Independent-implementation-friendly (standard PDG).
3. **[capability] Bounded symbolic PATH-SOLVING as an input oracle** (BMC/KLEE mechanism, sound because replay is the arbiter — **verdict: supported**): along the intended path to a must-reach-blocked sink, collect the path condition (branch predicates + bounded loop-trip constraints), solve with SAF's linked SMT solver for a concrete nondet-input model, **feed the model into the existing native replay**. Wrong model → replay fails → abstain → **no −16/−32 risk**. Use **incremental bound raising** (k=1,2,4…) for minimal counterexamples. **Redline (from the supported verdict):** the replay must exercise the **unmodified** program under **SV-COMP-faithful semantics** — correct 32/64 bit-width, in-source violation attribution, full nondet determinization, no environment-dependent "violations," abstain-on-UB/divergence. The only real risk is regressing the replay's fidelity, not the solver being wrong.

### (a) Push witness-confirmation toward ~86% — with a corrected lever

**REFUTED — "emit the replay's exact nondet values as assumption/value waypoints; near-zero new analysis; moves confirmed-% up sharply."** **False on every link** (verdict: refuted, high confidence):
- **Premise false:** SAF's `witness_lower.rs` emits only `branching`+`target` waypoints; the nondet values exist internally (`FalseCandidate.nondet_sequence` from the Z3 model) but are **never lowered**.
- **"Near-zero" false:** emitting an assumption `x==6` needs the **C-level variable name**, which SAF's frontend **drops after mem2reg** — documented plan-194/195 frontend work.
- **Already tried & refuted:** plan-195 Tier A branching enrichment = **ZERO confirmed-% change**; Tier B assumption-values declared **NO-GO** (the unconfirmed tasks were array-store / bool-mutex / float-maxpool / recursion — outside named-scalar-nondet scope; values wouldn't flip them).
- **The REAL lever (measured 3/7→6/7):** **wire an execution-based validator (cpa-witness2test-style) into SAF's OWN confirmation gate**, using existing target+branching witnesses — validator-side, not witness-enrichment. SAF already does concrete replay internally, so it can **self-confirm** without an external value harness.

**Directive:** the confirmation-% levers are **[tuning] wire an execution validator into the confirmation gate** (accept CONFIRM if analysis OR execution agrees) and **[capability] RECALL** (the actual bottleneck). Do **NOT** invest in value/assumption-waypoint enrichment expecting a confirmation lift. Before any witness-content work, **dump a few unconfirmed-but-correct SAF witnesses and diff against what the validator needs** to locate the real bottleneck (content vs validator-choice vs 90s-timeout).

---

## 5. THE RESEARCH/LEARN ARM DIRECTIVE (reusable arm instruction)

Purpose: import **algorithmic MECHANISMS** (a slicing formulation, a round-robin sequentialization schema, a DPOR concept) that SAF **reimplements from scratch** — REQ-IP-001, no vendoring, no code copy.

Reusable arm instruction:

> **You may use WebSearch/WebFetch during the ACT step only** to learn how top SV-COMP tools *work* (Lazy-CSeq/CSeq sequentialization, Symbiotic slicing+KLEE, CBMC/ESBMC BMC encodings, GenMC/Nidhugg DPOR, Witch3/cpa-witness2test validation). **Prefer PRIMARY sources**: sv-comp.sosy-lab.org rules/benchmarks/category pages, TACAS/SV-COMP proceedings, individual tool papers and repos. **Extract the MECHANISM** — the algorithmic idea described well enough to reimplement from scratch — **never code**. Then write a **fresh independent implementation** that passes SAF's own tests. Enforcement is a review property, not a network property: the adversarial-review Claude flags any diff that looks copied/vendored; the frozen scorer + FP audit validate the result regardless of source. **Log every fetched URL** into the per-arm transcript audit (same audit that checks holdout-not-read). Deny-by-default egress: model provider + search endpoint only; no arbitrary curl, no git push. Web access does not weaken gates (a)–(c): the immutable files are local/hash-gated and the holdout is a local manifest the agent must not READ.

The overfit literature is reassuring here: a sound concrete-replay confirmer is a **low-complexity, mechanism-level object** ("replay the violation; if ASan/UBSan traps, emit FALSE") that either genuinely reproduces a bug or does not — it **cannot overfit the way a tuned threshold can**. So capability arms importing a mechanism are **lower** overfit risk than tuning arms; weight scarce held-out re-checks toward tuning arms.

---

## 6. CLAUDE-CODE HEADLESS / RATE-LIMIT — corrections & confirmations

Confidence **high** on api_retry/subtype; **medium** on the exact headless reset-timestamp format and proxy behavior.

**CONFIRMED (use as designed):**
- **Classify transient-vs-hard from the structured `system`/`api_retry` stream event, NOT prose.** Over `--output-format stream-json --verbose`, the event carries `subtype:"api_retry"`, `attempt`, `max_retries`, `retry_delay_ms`, `error_status` (HTTP code or **null** for connection errors), and `error ∈ {authentication_failed, oauth_org_not_allowed, billing_error, rate_limit, overloaded, invalid_request, model_not_found, server_error, max_output_tokens, unknown}`. Switch on `.error`: transient `{overloaded, server_error, rate_limit-throttle, null-status connection}` → jittered exponential backoff (e.g. [30,60,120,240,300]s ±15%) + resume-by-id; hard/abort `{authentication_failed, billing_error, oauth_org_not_allowed, invalid_request, model_not_found, max_output_tokens}`; default `unknown` → conservative bounded retry. The `unknown` bucket + "ignore values you don't recognize" convention make this **version-stable**.
- **Gate per-arm success on the terminal result `subtype=="success"`, NOT `is_error`, NOT exit code alone.** `is_error` historically stayed `false` on `error_max_turns` (documented footgun), silently accepting truncated runs; the `result` text field is present ONLY on `success`. Documented subtypes: `success | error_max_turns | error_max_budget_usd | error_during_execution | error_max_structured_output_retries`. On `error_max_turns`, **resume from `session_id`** (all error subtypes carry it), don't accept. `total_cost_usd`/`usage` present on every subtype → global budget accounting. Treat exit 143 as supervisor-initiated kill.
- Launch workers `claude --bare -p --output-format stream-json --verbose --allowedTools <explicit> --max-turns N --max-budget-usd B`; capture `session_id`, drive all retries/resumes by it. **`--bare`** skips hook/skill/plugin/MCP/CLAUDE.md auto-discovery (isolation lever: prevents a stray hook/MCP leaking the held-out manifest) — but requires `ANTHROPIC_API_KEY` (no keychain/OAuth read). **Never use `--continue`** in a multi-arm loop (its "most recent in dir" semantics races).
- Set `CLAUDE_CODE_MAX_RETRIES` low (3–5) so the supervisor *sees* hard blocks fast and owns the sleep policy; reserve `CLAUDE_CODE_RETRY_WATCHDOG=1` for fire-and-forget arms only. `API_TIMEOUT_MS ≈ 1200000` for proxied/slow networks.
- **Isolation = BOTH** a preventive PreToolUse-hook/denylist on the forbidden holdout path AND a post-hoc audit scanning `~/.claude/projects/<sanitized-cwd>/<session-id>.jsonl` for tool_use Reads of that path. **Pin the CLI version** — the JSONL per-line schema is internal/version-coupled; prefer the hook denylist as the sound preventive control.
- **systemd:** run the plain-process supervisor (not a claude worker) as the long-lived unit — `Restart=always`, `RestartSec` backoff, `EnvironmentFile` (chmod 600) for `ANTHROPIC_API_KEY`, generous timeouts, optional sd_notify watchdog. Persist per-arm session-ids to a state file so reboot **resumes arms by id**, not restart. Telemetry vars (`DISABLE_TELEMETRY` etc.) turn ON for **any non-empty value including "0"** — **unset** to disable.

**CORRECTED (plan-204 assumptions to fix):**
- **Hard-block reset time is NOT only human-readable local-time prose in headless.** The interactive banner shows bare local time (`resets 3:45pm`), but in headless `-p --stream-json` the reset arrives as a **pipe-delimited Unix epoch** (e.g. `...usage limit reached|1762952400`), sometimes on a **non-JSON line** that breaks naive `JSON.parse`. Also the **status-line JSON exposes `rate_limits.five_hour.resets_at` / `.seven_day.resets_at` as clean Unix-epoch seconds** (Pro/Max only, after first response; absent for API accounts). **Parse the epoch / poll status-line `resets_at`, do NOT regex "3:45pm".** The sleep-to-reset conclusion stands; the parsing guidance inverts.
- **stream-json stdout can stall mid-session** — the supervisor needs a **wall-clock watchdog independent of stream events**.
- **Proxy gotcha:** if `ANTHROPIC_BASE_URL` points at a non-first-party gateway, it may **strip the unified quota headers** the CLI uses to distinguish a real usage-limit from a transient throttle, and it disables MCP tool-search / Remote Control. Behind a proxy, **do not trust the CLI's auto-classification** — classify from the `api_retry` event + conservative fixed-sleep fallback.
- `--allowedTools` vs `--allowed-tools` both accepted; confirm on the installed version. Do not assert specific non-zero exit codes beyond 0/143 — branch on result subtype.

---

## 7. PUNCH LIST — ranked levers for the autonomous loop

Each tagged `[tuning]` / `[capability]`, mechanism in one line. Ordered by ROI × soundness-safety.

1. **[tuning] Wire an execution-based validator into SAF's confirmation gate** — accept CONFIRM if analysis OR execution replay agrees; the measured 3/7→6/7 lever, no witness-content change. *(Corrects the refuted "emit nondet values" lever.)*
2. **[capability] ConcurrencySafety-FALSE Milestone 1 — deterministic-replay scheduler shim** as a CHESS-style c≤1→c≤2 blind sweep with ASan/UBSan; metric FALSE-recall at FA=0; de-risks the confirmer, reuses SAF's oracle verbatim.
3. **[capability] Backward AIR program slicing from the sink** (control+data dependence closure via existing points-to) — the scalability multiplier that makes symbolic path-solving finish in budget and shrinks witnesses.
4. **[capability] Harness/entry-point synthesis + complex-object havocking (`__VERIFIER_nondet_memory`-style)** — the actual binding constraint on the 2026-new TDX firmware batch. *(Corrects the refuted "path-solving is dominant" lever.)*
5. **[capability] Bounded symbolic path-solving as an input oracle → existing native replay** — solve the path condition for a nondet model, replay decides; wrong model → abstain; incremental bound raising; replay must stay bit-width/UB/semantics-faithful.
6. **[capability] ConcurrencySafety-FALSE Milestone 2 — lazy round-robin sequentialization AIR→AIR pass** (pc-labeled re-enterable thread fns + per-thread static local cells + round-robin driver); every proposed FALSE re-confirmed by the Milestone-1 shim on the original binary before emission.
7. **[capability] GraphML-1.0 concurrency-witness emitter** (threadId sequence + createThread edges + startline from the replayed schedule) targeting CPAchecker-ThreadingCPA / Dartagnan / ConcurrentWitness2Test — without it every concurrency FALSE is +0 not +1.
8. **[tuning] Frontend robustness on anonymous unions / nested types** — prevents quiet wrong answers and unlocks TDX-family reachability.
9. **[capability, later] Dedicated concrete race confirmer (TSan-style happens-before replay)** for the ~235 racy NoDataRace-FALSE tasks — smaller pool, needs a new confirmer; do after the reuse arms.
10. **[deferred] Weak-memory (TSO/PSO/C11-relaxed) encodings & full DPOR execution-graph engine** — unnecessary for SC-only ConcurrencySafety-C; defer unless Milestones 1/2 prove deep-interleaving bugs are being missed.

**Cross-cutting soundness invariants for every arm (tuning or capability):** gate (c) runs on the **FULL** immutable SAFE reservoir per family (never a subsample); each NEW FALSE-emitting confirmer scanned against the entire SAFE pool for latent FP classes before keep; replay on the **unmodified** program under SV-COMP-faithful semantics; **abstain on any divergence / un-modeled primitive / weak-memory / dropped pragma**; **held-out svcomp26 is a HARD reject** for train gains that don't reproduce; **no benchmark-path/function-name/task-id keying**.