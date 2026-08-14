# Plan 204 — Autonomous improvement loop for SAF on SV-COMP (design; NOT yet started)

**Branch:** `svcomp`  **Date:** 2026-08-14  **Status:** DESIGN — awaiting approval to build; the
loop is NOT running. Research-backed (workflow `wf_a8522d4f`, verified vs primary sources).

## Goal
Run Claude Code on the VM (`cd-vm-15-ai-vm`) to **continuously and autonomously raise SAF's
CONFIRMED `C.FalseOverall` score on svcomp25 (train), validated on a frozen svcomp26 held-out
set**, surviving rate limits / disconnects / reboots, **without reward-hacking** the metric.

Baseline to beat (plan 203): svcomp25 CONFIRMED C.FalseOverall **≈ 3960** + ~396 termination;
sound (FP=0 except 1 inherent overflow label-noise), deterministic. Loudest signal to attack:
unreach holdout 13%→**0%**; memsafety confirmed-% ~58% (witness-confirmation is a real lever).

## User decisions (2026-08-14)
1. **External supervisor = a plain, NON-LLM bash/python process — NOT a Claude instance.** The
   scorer, loop-control, ALL rate-limit handling, and ALL HARD gates live in it. **Rationale
   (user): worker `claude` sessions WILL hit rate/usage limits — often. A controller that was
   itself a Claude instance would be throttled at the same moment and could not manage the
   rate-limited worker, detect the reset window, or sleep-and-resume.** So loop control and
   rate-limit recovery are 100% deterministic and never depend on a second Claude. (This is also
   the strongest reward-hacking defense: the agent cannot edit or bypass a scorer it never runs.)
2. **Auto-commit to a scratch branch** (`auto/loop-<date>`); the user reviews + cherry-picks to
   `svcomp`. **Never `git push`, never auto-merge to `svcomp`.**
3. **systemd service** (`Restart=always`) — survives SSH drops, crashes, reboots.

**Claude is used for exactly two things, both driven by the bash supervisor and both subject to its
rate-limit handling:** (a) the per-arm improvement attempt (`act`), and (b) an OPTIONAL adversarial
diff review. Neither is on the critical path for soundness — the hard gates below are all
deterministic bash, so even a multi-hour rate-limit window cannot let a bad change through or wedge
the loop; the supervisor simply sleeps-to-reset and resumes.

## Verified operational facts (Claude Code 2.1.191 on the VM, proxy auth)
- `/goal` is a real built-in command; `/loop` is a bundled skill; the **Agent SDK** (Python/TS) is
  a cleaner programmatic alternative to shelling `claude -p`. We drive `claude -p` from the
  supervisor (the loop lives in bash, not in `/goal`, so control stays external).
- Headless: `claude -p`, `--output-format stream-json`, `--max-turns N`,
  `--dangerously-skip-permissions` (= bypassPermissions; required unattended), `--allowedTools`.
- Resume/continuity: `--continue`/`-c`, `--resume`/`-r`, `--session-id`, `--fork-session`;
  transcripts at `~/.claude/projects/...`.
- Rate limits: the CLI **auto-retries transient 429/529** (`CLAUDE_CODE_MAX_RETRIES` default 10;
  set `CLAUDE_CODE_RETRY_WATCHDOG=1` unattended). **Hard usage-window / quota exhaustion is NOT
  retried — it blocks until reset**, so the supervisor must detect it and **sleep-to-reset**.
  Proxy: retry still applies; raise `API_TIMEOUT_MS` (default 600000); the proxy may omit unified
  quota headers → 429s treated as retryable throttles.
- Best practice (verified): honor `Retry-After` → exponential backoff → jitter (AWS); 529-overload
  ≠ transient 429 ≠ hard quota window.

## Architecture — the arm loop (all control in `supervisor.sh`, systemd `Restart=always`)
```
loop forever:
  1. ORIENT     read progress-journal.md + git log + last eval JSON (fresh, resumable state)
  2. BASELINE   run the IMMUTABLE eval on TRAIN → require FP=0, wrong-TRUE=0, nextest green,
                clippy/fmt clean   (smoke-test the base BEFORE building; catch regressions)
  3. PICK LEVER supervisor selects the next lever by ROI/feasibility (see levers) or rotates on stall
  4. ACT        git checkout -b arm/<n>;  claude -p --resume <sid> --max-turns M
                  --dangerously-skip-permissions --allowedTools <scoped> < arm_prompt.md
                (the agent makes ONE scoped improvement; may NOT run the scorer or touch immutables)
  5. VERIFY     supervisor (NOT the agent) gates the arm. HARD gates are ALL deterministic bash —
                no Claude, so they hold through any rate-limit window:
                  a. immutable-file sha256 unchanged   (else REJECT — tamper)       [bash]
                  b. holdout not read by the arm        (path/transcript audit)       [bash]
                  c. re-run the IMMUTABLE scorer on TRAIN: FP=0 AND wrong-TRUE=0      [bash]
                  d. CONFIRMED-score delta on TRAIN > 0 AND nextest/clippy/fmt green  [bash]
                  e. (SOFT, optional) adversarial-review: a FRESH `claude` reads the diff for
                     redline breaches. It is itself rate-limit-managed by the supervisor; soundness
                     never depends on it — if unavailable, a-d still fully enforce correctness.
  6. HELD-OUT   every K kept arms: supervisor runs the svcomp26 held-out eval → the REAL signal.
                overfit shows as train↑ / holdout↔ → that lever is deprioritized.   [bash]
  7. CHECKPOINT keep → commit to auto/loop-<date> + append journal + heartbeat; else revert + note.
  8. RATE-LIMIT the supervisor parses the worker's `--output-format stream-json` (error/result
                events) + exit code to CLASSIFY, then handles it deterministically (no Claude):
                  • transient 429 / 529 overload → short jittered backoff, retry (the CLI also
                    auto-retries via CLAUDE_CODE_RETRY_WATCHDOG);
                  • HARD usage-window / quota exhausted → parse the reset time if present, else a
                    capped exponential backoff (e.g. 5min→…→1h), SLEEP-TO-RESET, then `--resume`;
                  • real error → log, revert the arm, continue.
  9. CRASH      systemd Restart=always → a fresh supervisor process → step 1 rebuilds state from the
                journal + git (each arm is an idempotent, resumable unit).
 10. STOP-GUARD no held-out gain after N arms on a lever → rotate; global max-arms/max-days cap.
```

## Anti-reward-hacking — defense in depth (the user's explicit concern)
Reward-hacking is the dominant long-horizon failure (SpecBench: val↔held-out gap grows ~28pp per
10× code size). Controls, made specific to SAF:
1. **Immutable scorer & ground truth, read-only + hash-gated.** `scripts/svcomp_split_eval.py`, the
   −16/−32 FP audit, `scripts/validate_witness.sh`, the split manifests, and the sv-benchmarks
   `.yml` expected-verdicts are `chmod a-w` and **sha256-checked before/after each arm**; any change
   → arm rejected. The agent may ADD confirmers/slices; never weaken the audit or edit a label.
2. **Frozen held-out svcomp26 the agent never trains on.** The supervisor owns the holdout eval; the
   agent only gets TRAIN + per-task diagnostics. Primary defense = the **supervisor-computed
   held-out gate** (the agent cannot fake a number it never runs); secondary = a read/transcript
   audit that rejects an arm touching the holdout manifest.
3. **The 8 CLAUDE.md soundness redlines as hard invariants.** The HARD enforcement is the
   deterministic FP=0/wrong-TRUE=0 scorer gate (bash, no Claude — holds through rate limits). An
   **independent adversarial-review Claude** (fresh session per arm — the pattern that caught 3 real
   −32 bugs the scaled audit missed) is an ADDITIONAL soft layer, rate-limit-managed by the
   supervisor; correctness never depends on it being available.
4. **Verify-before-victory**: the agent's self-report is never trusted; kept only if the EXTERNAL
   score rises with FP=0. (Anthropic: "read the transcripts before declaring victory.")
5. **No auto-push, no auto-merge to `svcomp`**; kept arms land on `auto/loop-<date>` for human review.
   Hard token/turn/arm caps as circuit breakers; a no-progress guard as the stuck-loop detector.

## Improvement levers (agent picks by ROI; from memory/plans, all sound-gated)
- **Witness-confirmation %** (highest-confidence lever): raise confirmed FALSEs via cpa-witness2test
  execution validator (43%→86% per plan 195) + witness enrichment (Tier-A branching). memsafety
  confirmed ~58% and unreach ~67% → direct CONFIRMED-score lift with zero soundness risk.
- **unreach-call recall / holdout generalization** (13%→0% on 2026-new): why do must-reach/replay
  shapes not transfer? per-task diagnostics on the 2026-new misses.
- **memsafety recall** (input steering; R5 gated Slice-2, deferred).
- **no-overflow** (R6 gated Slice-2: `-fsanitize=shift`, threaded inclusion) — NOT the 1 inherent
  `twisted` label-noise FP.
- Each lever = a throwaway `arm/<n>` branch, kept only on a measured, sound, held-out-checked gain.

## Robustness / survivability
- **systemd** unit `saf-loop.service` (`Restart=always`, `EnvironmentFile=~/.claude-code.env`,
  `WorkingDirectory=<repo>`, `CLAUDE_CODE_RETRY_WATCHDOG=1`, raised `API_TIMEOUT_MS`) → survives
  drops/crashes/reboots. Each arm is a journal-reconstructable unit → crash-safe mid-goal.
- **Heartbeat + per-arm logs** (`status.json`, `journalctl -u saf-loop`, `report.sh` for score history).
- **Sleep-to-reset** on hard usage windows (don't hammer); jittered backoff on 529.

## Artifacts to build (under `scripts/loop/`)
`supervisor.sh` (the loop) · `arm_prompt.md` (fixed per-arm instructions + redlines + "never touch
the scorer/holdout") · `immutable.sha256` (checksum manifest) · `review_arm.sh` (adversarial-review
sub-agent) · `progress-journal.md` (durable shift-handoff) · `report.sh` (human score view) ·
`saf-loop.service` (systemd) · run-config (env, `--max-turns`, `--allowedTools`, caps).

## Rollout (staged, reversible — each stage needs a go)
- **Stage 0 (build):** write the artifacts; unit tests for the gate logic (inject a fake FP →
  confirm auto-revert; edit an immutable file → confirm reject).
- **Stage 1 (dry-run):** run ONE arm manually, foreground, watched, on a scratch branch — confirm
  the orient/act/verify/held-out/checkpoint + tamper/holdout gates all fire correctly. Still NOT
  the autonomous service.
- **Stage 2 (supervised enable):** enable systemd; run a few arms; monitor closely.
- **Stage 3 (autonomous):** let it run; review `auto/loop-<date>` periodically; cherry-pick wins.

## Redlines for the loop itself (non-negotiable)
Never push; never merge to `svcomp` without the human; never edit the scorer/audit/labels/holdout;
never emit `true` outside the gated paths; FP=0 & wrong-TRUE=0 every kept arm; byte-determinism;
all builds/evals in Docker on the VM.

---

# Addendum A (2026-08-14) — capability track, ConcurrencySafety-FALSE, research/learn arms, verified headless mechanics

Research-backed and adversarially verified (workflow `wf_35fcca7f`, primary sources: SV-COMP
2025/2026 rules+benchmarks pages, TACAS reports, Lazy-CSeq/CSeq + Symbiotic + Claude Code headless
docs). Full brief + per-claim verdicts: **`plans/204-research-brief.md`**. This addendum SUPERSEDES
the base plan wherever they differ. Two verified corrections to the base plan and two refuted
"obvious" levers are flagged inline — do not re-introduce them.

## A0. Motivation (user, 2026-08-14)
The loop must do more than tune recall of already-wired properties: it must be able to **build NEW
enabling capabilities** — flagship example **ConcurrencySafety-FALSE** (SAF abstains on all concurrent
programs today) — and it must **learn mechanisms from the best SV-COMP tools via WebSearch** (learn
the *mechanism*, never copy code; REQ-IP-001). The base plan's per-arm gate structurally forbids
both; §A2 fixes that without weakening any soundness gate.

## A1. Scoring truth (verified) — ConcurrencySafety-FALSE is a real C.FalseOverall lever
- **A correct, confirmed FALSE on a ConcurrencySafety task counts in `C.FalseOverall`** (+1; wrong
  FALSE −16). `C.FalseOverall` = **all C categories with the TRUE results dropped** (not "all
  non-Termination" — SV-COMP **2026 renamed FalsificationOverall→C.FalseOverall and now INCLUDES
  Termination**; base plan wording is stale but the GO signal is unchanged).
- ConcurrencySafety is **4 sub-categories over one `Concurrency.set`**: Main (`unreach-call`),
  MemSafety, NoOverflows, NoDataRace-Main. **Three of the four test properties SAF ALREADY confirms
  concretely** (reach_error / ASan / UBSan) — the value is NOT limited to the hard data-race property.
- `no-data-race` **TRUE** scores only in `C.TrueOverall` (worthless to SAF, per [[saf-svcomp-202-r8-nodatarace-defer]]); `no-data-race` **FALSE** (a real reachable race) DOES count in
  `C.FalseOverall` but needs its own race confirmer (late slice §A7 lever 9). **Verdict: GO.**

## A2. The capability-track gate fix (THE core plan-204 change)
**Verified problem:** base gate ⑤(d) requires `CONFIRMED train-delta > 0` for **every** kept arm, and
arms are atomic keep-or-revert with no cross-arm accumulation → any multi-slice capability whose
Slice-0 spine scores 0 is reverted before the payoff. The loop is structurally confined to greedy
tuning. **Fix: split gate (d) into two orthogonal predicates.**

- **SOUNDNESS predicate — ABSOLUTE, byte-identical for BOTH modes, always the non-LLM supervisor:**
  (a) immutable-file sha256 intact · (b) holdout-not-read audit · (c) re-run the immutable scorer with
  **FP=0 AND wrong-TRUE=0** · nextest/clippy/fmt green. **Gate (c) runs on the FULL immutable SAFE
  reservoir for the arm's family — NEVER a subsample** (plan-203 lesson: sample FP=0 at N=40–60 was
  38 real FPs at scale, each "confirmed" by the ASan pipeline — *"confirmed by the pipeline" ≠ "correct
  verdict"*). A new FALSE-emitting confirmer is the **higher-risk** class; scan it against the entire
  SAFE pool for latent FP classes before keep.
- **PROGRESS predicate — mode-dependent:**
  - **tuning arm** (targets an already-wired scoring property) → unchanged: `CONFIRMED train-delta > 0`.
  - **capability arm** (a not-yet-wired property / new engine, declared milestone ladder) → a
    **milestone contract** instead of a train-delta.

**Supervisor decision rule (deterministic, logged every arm, never an LLM judgment):** the supervisor
picks the mode at **lever-selection time** from the **human-curated lever taxonomy (§A7)**. Each arm
writes an `arm.json` declaring `mode∈{tuning,capability}`, target family, and (capability) the
milestone predicate. Consistency check removes any incentive to mislabel: a capability-flagged arm
that *does* move score folds in as success; a tuning-flagged arm that fails to move score reverts as
today. The **only** asymmetry a capability flag buys: a **zero-delta arm may survive IFF
capability-flagged AND it meets its milestone AND it touches only a `cap-*` branch.**

**Milestone acceptance for a capability arm** (replaces "score rose this arm"):
1. **FP=0 on the family's held-out SAFE set** via gate (c) on the **FULL** reservoir (never a subsample).
2. **Either** a declared structural milestone advanced (a unit test the arm adds, itself run under the
   FP audit), **or** (terminal arm) **≥1 newly-confirmed FALSE** on the target family that baseline
   missed, validated by the **same immutable witness/confirmation pipeline** the aggregate score uses.
3. Capability arms accumulate on a **`cap-<capability>/<n>` branch lineage** (never reverted between
   scaffolding arms, never auto-merged to `svcomp`), tagged `not-yet-scoring` in the journal, so arm
   N+1 builds on arm N.

**Anti-gaming (absorb both REFUTED premises — do not re-introduce):**
- **DO NOT** justify the relaxation with *"no reward-hacking exploit attacks the score-delta."* FALSE:
  memorization/benchmark-keyed overfitting raises the measured score without touching the scorer and is
  **the dominant** coding-agent exploit (SpecBench/EvilGenie). The relaxation is safe **because** (i)
  soundness is enforced by (a)–(c) independently of (d), and (ii) the real anti-overfit control is the
  **frozen svcomp26 HELD-OUT gate + origin-family (not per-task) split** — which must be a **HARD
  reject** for any arm whose train gain does not reproduce on held-out.
- **DO NOT** claim a capability milestone is a *"stronger soundness bar."* FALSE: both modes clear the
  same gate (c); a capability arm is only weaker on *progress*. Soundness comes from full-reservoir
  gate (c) + the held-out gate, never from any "capability > tuning" ordering.
- **`arm_prompt.md` redline:** confirmers must be property-general — **no benchmark-path / function-name
  / task-id keying** (closes the memorization exploit). WebSearch/WebFetch scoped to the ACT step,
  deny-by-default egress (model + search endpoint only), no arbitrary curl, no `git push`; every fetched
  URL logged into the same transcript audit that checks holdout-not-read.

**De-prioritization (non-converging track): milestone-budgeted circuit breaker with a PARKED state, not
a hard kill.** Each capability track gets a budget (~6–8 arms / bounded tokens per milestone). Advance
within budget → refill. Budget consumed with no advance → **PARK** the `cap-*` branch for human review,
deprioritize the lever, rotate. **Graduation:** once a capability's confirmed FALSEs raise the
**aggregate svcomp26 HELD-OUT score** (step 6, the real signal), its confirmer joins the baseline and
future arms on it are scored as **tuning**. (This matches SAF history: R4/R8 correctly abandoned, but
their scaffolding taught the next slice.)

## A3. ConcurrencySafety-FALSE — recommended sound mechanism (ranked)
1. **Bounded round-robin lazy sequentialization (AIR→AIR transform) as the FINDER.** Reimplement the
   Lal-Reps / La-Torre-Madhusudan-Parlato / Lazy-CSeq schema from scratch (REQ-IP-001): pc-labeled
   re-enterable thread functions, per-thread **static local cells** (locals keep real concrete values
   across yields), round-robin driver with nondeterministic yield, bounds *K* rounds × *N* threads.
   **Soundness for FALSE is proven:** it is an **under-approximation of the schedule space with NO
   data/state over-approximation** — every violation it finds is a genuine concurrent violation;
   tightening *K*/*L* costs only recall, never a spurious FALSE.
2. **Deterministic-replay scheduler shim as the CONFIRMER** (the composability lever). A violating path
   yields an explicit **(round, thread, steps-before-yield) schedule vector**; a small pthread
   interposer (intercept create/join/mutex/cond + yield points, gate each op on a shared turn-counter)
   forces that exact interleaving on the **real, un-transformed binary** under ASan/UBSan. Confirmation
   on the real binary ⇒ a mis-modeling transform can only cost recall (replay fails → abstain), **never
   a wrong FALSE** — identical to SAF's sequential "prove-reach OR concretely-reproduce" doctrine.
3. **CHESS-style preemption-bounded concrete testing** — the cheapest first-pass finder AND the replay
   substrate (it *is* the shim). Most SV-COMP concurrency bugs manifest within few context switches.

**FIRST de-risk milestone (throwaway VM branch, plan-198/202 style): build the replay shim ALONE**, run
it as a CHESS-style c≤1→c≤2 **blind sweep** over the ConcurrencySafety-FALSE reservoir with ASan/UBSan
on. **Metric: FALSE recall at FA=0.** This de-risks the confirmer independently of the transform and
reuses SAF's oracle verbatim; if a shallow sweep already catches a non-trivial fraction at 0 false
alarms, ship it before building the sequentialization. Milestone 2 = the AIR→AIR pass, every proposed
FALSE re-confirmed by the shim on the original binary before emission.

**SC is sufficient (verified):** ConcurrencySafety-C is sequentially consistent (interleaving
semantics). **Do NOT build weak-memory (TSO/PSO/C11-relaxed) machinery for the first capability.**

**Fail-closed guardrails (arm invariants):** (a) native x86 replay is TSO → **ABSTAIN on
pthread-wmm / C11-relaxed**; (b) reachability-gate dead `pthread_create` scaffolding (reuse plan-198
`reachable_spawns_threads`); (c) dropped `#pragma omp`/OpenMP → **abstain** (plan-202 lesson);
(d) model `__VERIFIER_atomic_begin/end` uninterruptible, any un-modeled sync primitive → abstain
(encoding fidelity, not the round bound, is the real spurious-FALSE risk); (e) abstain on any divergence
between shim replay and the recorded schedule.

**Witness — a REAL correction to the base plan.** SAF's YAML-2.0 witness pipeline and its
`cpa-witness2test` execution validator **DO NOT WORK for concurrency violations** (Witness Format 2.0:
*"Violation witnesses have not yet been defined for concurrency safety."*). Concurrency FALSE must be
witnessed in **GraphML format 1.0** carrying thread-schedule keys (`threadId` per transition,
`createThread` edges, `startline`), confirmed by **CPAchecker-ThreadingCPA / Dartagnan /
ConcurrentWitness2Test** (execution-based — SAF's replay identity is NOT disqualified). **Build an
independent GraphML-1.0 emitter** that serializes the exact interleaving SAF replayed; without it every
concurrency FALSE lands **+0 (raw), not +1**. Confirm the year-2027 active concurrency validators
before relying on it (2024 had <3).

## A4. Recall + witness-confirmation levers (mechanisms — two refuted "obvious" bets flagged)
- **REFUTED — "the 2026-new unreach miss is BMC/KLEE nondet-solving territory."** Inverted: the marquee
  new-2026 content is the **Intel TDX Module firmware set (~418 tasks, HarnessForge)** and its hardness
  is **STRUCTURAL** — frontend fragility on anonymous unions/nested types (causes *quiet wrong answers*
  in top tools), complex-object havocking (motivated the new `__VERIFIER_nondet_memory()` primitive),
  harness construction + preconditions; ~30% of cover points are unreachable/expected-TRUE (out of scope
  for a FALSE-finder). **Directive:** treat bounded path-solving as ONE lever, not the dominant bet;
  prioritize **harness/entry-point synthesis + complex-object havocking** and **frontend robustness on
  unions/nested types**; **split the 2026-new batch by shape and attribute recall per shape before
  betting heavily.**
- **REFUTED — "emit the replay's nondet values as assumption/value waypoints to lift confirmation-%."**
  Refuted on every link: SAF's `witness_lower.rs` emits only `branching`+`target` (the internal
  `FalseCandidate.nondet_sequence` is never lowered); an assumption `x==6` needs the C-level name SAF's
  frontend drops after mem2reg; and plan-195 **already measured** Tier-A branching = zero change and
  declared Tier-B assumption-values NO-GO. **The REAL confirmation lever (measured 3/7→6/7): wire an
  execution-based validator (cpa-witness2test-style) into SAF's OWN confirmation gate** — accept CONFIRM
  if analysis OR execution replay agrees (validator-side, SAF already does concrete replay internally,
  so it can self-confirm). Before any witness-content work, **dump a few unconfirmed-but-correct SAF
  witnesses and diff against what the validator needs** to locate the true bottleneck.
- **Sound recall mechanisms (all keep SAF's replay as the SOLE arbiter → soundness-neutral):**
  backward **AIR program slicing** from the sink (Symbiotic mechanism; scalability multiplier, shrinks
  witnesses); **bounded symbolic path-solving as an input oracle** (collect path condition → solve with
  SAF's SMT for a concrete nondet model → feed the **existing native replay**; wrong model → replay
  fails → abstain → no −16/−32 risk; incremental bound raising k=1,2,4…). **Redline:** replay must
  exercise the **unmodified** program under SV-COMP-faithful semantics (correct 32/64 bit-width,
  in-source violation attribution, full nondet determinization, abstain-on-UB/divergence).

## A5. Research/learn arm directive (reusable `arm_prompt.md` block)
> **During the ACT step you MAY use WebSearch/WebFetch** to learn how top SV-COMP tools *work*
> (Lazy-CSeq/CSeq sequentialization, Symbiotic slicing+KLEE, CBMC/ESBMC BMC, GenMC/Nidhugg DPOR,
> Witch3/cpa-witness2test). **Prefer PRIMARY sources** (sv-comp.sosy-lab.org, TACAS proceedings, tool
> papers/repos). **Extract the MECHANISM** (the algorithmic idea, well enough to reimplement from
> scratch) — **never code; no vendoring, no copy** (REQ-IP-001). Write a **fresh independent
> implementation** that passes SAF's own tests. **Log every fetched URL** into the per-arm transcript
> audit. Deny-by-default egress: model provider + search endpoint only; no arbitrary curl, no `git
> push`. Web access does not weaken gates (a)–(c) (immutables are local/hash-gated; the holdout is a
> local manifest you must not READ). *(Enforcement is a review property — the adversarial-review Claude
> flags any diff that looks copied — plus the frozen scorer + FP audit, which validate the result
> regardless of source.)* Overfit note: a sound concrete-replay confirmer is a low-complexity,
> mechanism-level object that cannot overfit the way a tuned threshold can, so capability arms importing
> a mechanism are **lower** overfit risk than tuning arms.

## A6. Verified Claude Code headless / rate-limit mechanics (corrects §"Verified operational facts")
- **Classify transient-vs-hard from the structured `system`/`api_retry` stream event, NOT prose.**
  Requires `--output-format stream-json --verbose`. Event carries `attempt`, `max_retries`,
  `retry_delay_ms`, `error_status` (HTTP code or **null** for connection errors), and
  `error∈{authentication_failed, oauth_org_not_allowed, billing_error, rate_limit, overloaded,
  invalid_request, model_not_found, server_error, max_output_tokens, unknown}`. Switch on `.error`:
  transient `{overloaded, server_error, rate_limit-throttle, null-status}` → jittered exp backoff +
  resume-by-id; hard/abort `{authentication_failed, billing_error, oauth_org_not_allowed,
  invalid_request, model_not_found, max_output_tokens}`; `unknown` → conservative bounded retry.
- **Gate per-arm success on the terminal result `subtype=="success"`, NOT `is_error`, NOT exit code.**
  `is_error` stays `false` on `error_max_turns` (silent-truncation footgun); the `result` text is
  present only on `success`. Subtypes: `success | error_max_turns | error_max_budget_usd |
  error_during_execution | error_max_structured_output_retries`. On `error_max_turns`, **resume from
  `session_id`** (all subtypes carry it). Treat exit 143 as supervisor-initiated kill; do not assert
  other non-zero codes — branch on subtype.
- **Launch:** `claude -p --output-format stream-json --verbose --allowedTools <explicit> --max-turns N`
  (`--max-budget-usd B` if available); capture `session_id`; drive all retries/resumes by it; **never
  `--continue`** in a multi-arm loop (races on "most recent in dir"). `--bare` isolates from
  hook/skill/plugin/MCP/CLAUDE.md auto-discovery but **requires `ANTHROPIC_API_KEY`** — the VM uses
  PROXY auth (`ANTHROPIC_BASE_URL` + `ANTHROPIC_AUTH_TOKEN`), so **verify `--bare` compatibility on the
  VM before relying on it**; otherwise use an explicit `--allowedTools` allowlist + a PreToolUse
  denylist hook as the isolation control.
- **Reset time is a Unix epoch in headless, not "3:45pm" prose** — parse the pipe-delimited epoch (may
  arrive on a non-JSON line; guard `JSON.parse`), or poll the status-line JSON
  `rate_limits.five_hour.resets_at` / `.seven_day.resets_at` (Pro/Max only). Sleep-to-reset conclusion
  stands; the parsing guidance inverts the base plan.
- **stream-json stdout can stall mid-session** → the supervisor needs a **wall-clock watchdog
  independent of stream events**. **Proxy gotcha:** a non-first-party `ANTHROPIC_BASE_URL` may strip the
  unified quota headers → **don't trust the CLI's auto-classification behind the proxy**; classify from
  the `api_retry` event + a conservative fixed-sleep fallback.
- **Env:** `CLAUDE_CODE_MAX_RETRIES` low (3–5) so the supervisor sees hard blocks fast and owns the
  sleep policy; `API_TIMEOUT_MS≈1200000` for the proxy; **unset** telemetry vars (any non-empty value
  incl. `"0"` turns them ON).
- **Isolation = BOTH** a preventive PreToolUse-hook/denylist on the holdout path AND a post-hoc audit
  scanning `~/.claude/projects/<sanitized-cwd>/<session-id>.jsonl` for `tool_use` Reads of that path.
  **Pin the CLI version** (the JSONL per-line schema is internal/version-coupled).
- **systemd:** the **plain-process supervisor** (not a claude worker) is the long-lived unit —
  `Restart=always`, `RestartSec` backoff, `EnvironmentFile` (chmod 600), generous timeouts. **Persist
  per-arm `session_id` to a state file so reboot resumes arms by id**, not restart.

## A7. Lever taxonomy / punch list (the supervisor's ROI-ordered menu; tag decides tuning vs capability)
1. **[tuning]** Wire an execution-based validator into SAF's confirmation gate — CONFIRM if analysis OR
   execution replay agrees (measured 3/7→6/7; no witness-content change). *(Replaces the refuted
   "emit nondet values" lever.)*
2. **[capability]** ConcurrencySafety-FALSE **Milestone 1** — deterministic-replay scheduler shim as a
   CHESS c≤1→c≤2 blind sweep w/ ASan/UBSan; metric FALSE-recall at FA=0.
3. **[capability]** Backward AIR program slicing from the sink (control+data closure via existing
   points-to) — scalability multiplier for symbolic solving + smaller witnesses.
4. **[capability]** Harness/entry-point synthesis + complex-object havocking (`__VERIFIER_nondet_memory`
   -style) — the binding constraint on the 2026-new TDX firmware batch. *(Replaces the refuted
   "path-solving is dominant" lever.)*
5. **[capability]** Bounded symbolic path-solving as an input oracle → existing native replay (replay
   decides; wrong model → abstain; incremental bound raising; bit-width/UB/semantics-faithful).
6. **[capability]** ConcurrencySafety-FALSE **Milestone 2** — lazy round-robin sequentialization AIR→AIR
   pass; every proposed FALSE re-confirmed by the Milestone-1 shim on the original binary.
7. **[capability]** GraphML-1.0 concurrency-witness emitter (threadId/createThread/startline from the
   replayed schedule) → CPAchecker-ThreadingCPA / Dartagnan / ConcurrentWitness2Test; without it every
   concurrency FALSE is +0.
8. **[tuning]** Frontend robustness on anonymous unions / nested types — prevents quiet wrong answers,
   unlocks TDX-family reachability.
9. **[capability, later]** Dedicated concrete race confirmer (TSan-style happens-before replay) for the
   ~235 racy NoDataRace-FALSE tasks.
10. **[deferred]** Weak-memory encodings & full DPOR engine — unnecessary for SC-only ConcurrencySafety-C.

**Cross-cutting invariants for EVERY arm:** gate (c) on the FULL immutable SAFE reservoir (never a
subsample); scan each new FALSE-emitter against the whole SAFE pool for latent FP classes before keep;
replay the **unmodified** program under SV-COMP-faithful semantics; abstain on any divergence /
un-modeled primitive / weak-memory / dropped pragma; **held-out svcomp26 is a HARD reject** for train
gains that don't reproduce; **no benchmark-path/function-name/task-id keying**.

## A8. VM baseline prerequisite (discovered 2026-08-14)
The VM (`cd-vm-15-ai-vm`) `svcomp` checkout is at a **stale HEAD (plan-191 era) with a large uncommitted
working tree** (R1–R8 arrived via `rsync` of `crates/`+`scripts/`, never committed there). The loop does
`git checkout -b arm/<n>`, commits kept arms, and **reconstructs state from `git log`** — it needs a
**clean, current baseline**. **Setup step before Stage 1:** bring the VM to a clean `svcomp` baseline
matching the laptop HEAD via a **`git bundle`** transfer (no push to origin), working tree clean, then
regenerate the splits deterministically. Only then freeze immutables and run the dry-run.
