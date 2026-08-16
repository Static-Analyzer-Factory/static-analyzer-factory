# SAF Loop — Worker-Log Forensics (READ-ONLY audit)

Date: 2026-08-17. Auditor: forensics teammate. Scope: `.loop-state/` on `cd-vm-15-ai-vm`,
work branch `auto/loop-20260815`, arms 1–30. Method: parsed stream-json transcripts, verdict/before/after
JSON, git log/stat, `docker ps -a`/`inspect`/`top`. No writes, no builds, no container/branch changes.

---

## A. Prioritized BUGS (with evidence + fix)

### BUG-1 [HIGH] — `conc-shim-m1` lever is MIS-SCOPED: `family=unreach-call, scope=local`. It regresses unreach-call and can never win.
**Evidence.** `scripts/loop/levers.tsv:15`:
```
conc-shim-m1  capability  unreach-call  local  ConcurrencySafety-FALSE Milestone 1 ...
```
Because `scope=local` and `family=unreach-call`, the supervisor evals the arm on `--property unreach-call`
ONLY (`run_arm`, supervisor.sh:272-273). The concurrency confirmer is wired into the SHARED `unreach_strategy`
verdict path (`crates/saf-cli/src/commands.rs`), so it fires on real `unreach-call` reservoir tasks. In all
three arms it *added* a false alarm on unreach-call:

| arm | before confirmed | after confirmed | before FP | after FP | Δconfirmed | decision |
|----|----|----|----|----|----|----|
| 17 | 19 | 3  | 0 | 1 | **−16** | REVERT |
| 22 | 19 | 3  | 0 | 1 | **−16** | REVERT |
| 27 | 19 | 5  | 0 | 1 | **−14** | REVERT |

(`.loop-state/arm-{17,22,27}/{before,after,verdict}.json`; `verdict.json` shows `sound:false, false_alarms:1`.)
Note `emitted_false` rose 33→39/40 (it DID confirm ~6 new concurrent FALSEs) but one of them is a wrong
FALSE on a safe/TRUE unreach task, and −16 for the one FP swamps the +recall. A negative delta on a
`local` lever can only REVERT (there is no ACCUMULATE path for a regression; supervisor.sh:302 gates
ACCUMULATE on `delta==0`).

**Root problems bundled here:**
1. The lever's *intent* (ConcurrencySafety-FALSE recall) is scored against the wrong property. There is no
   `ConcurrencySafety`/concurrency `--property` in the scorer, so concurrency recall is invisible; the only
   thing the scorer sees is the collateral damage to unreach-call.
2. It is marked `local` but it edits the shared `unreach_strategy` path → it is functionally `crosscut`.
   Even as crosscut it would REVERT (unreach-call regressed), but at least the classification would be honest.
3. The confirmer is not FA=0 in the reservoir despite the worker measuring "FA=0" on its own 9 hand-picked
   fixtures (arm-17 result text). The scorer's stratified 1000-task sample found the FP the worker's tests missed.

**Fix.** Decide what this lever is FOR and make scoring match:
- If the goal is concurrency-FALSE recall: it needs its OWN scored property/manifest slice
  (ConcurrencySafety-FALSE tasks with a witness format that the scorer confirms), OR it must be benchmarked
  by a capability-track metric that is NOT the unreach-call confirmed score. As-is it is structurally
  incapable of a KEEP and should be **parked** until a concurrency scorer exists.
- The immediate FP is a soundness bug in `concurrent_replay_confirms_false` (a wrong FALSE on a
  safe/TRUE unreach task). If concurrency confirmation stays wired into `unreach_strategy`, that path
  must be fail-closed (abstain unless the sentinel drop is a genuine SC interleaving) before it can ever
  be kept — otherwise it will keep injecting −16.

### BUG-2 [HIGH] — Reverted work is DISCARDED, so each `conc-shim-m1` arm re-derives ~470 lines of `concurrency.rs` from scratch.
**Evidence.** `revert_arm` (supervisor.sh:351-359) does `git reset --hard` + `git clean -fdq`, dropping ALL
of the arm's work. The three concurrency workers each independently rebuilt the identical shim:
- arm-17 result: "a deterministic scheduler shim ... `crates/saf-svcomp/src/concurrency.rs` (new, ~470 lines + 12 unit tests)".
- arm-22 result: *"I discovered arm/17's scheduler shim was **never committed** (`git log --all` finds no `concurrency.rs`), so it had been scoring 0. **I rebuilt it fresh**"* (51 turns, $5.17).
- arm-27 result: *"The two prior conc-shim arms (17, 22) built the scheduler shim ... but scored 0 and **were never even accumulated**"* — rebuilt again + added a witness (57 turns, $6.59).

So arms 22 and 27 spent their first ~15-20 turns re-discovering and re-writing what 17 already wrote.
Combined waste on the concurrency lever: **3 arms, ~111 turns, ~$13.2, ~53 min wall**, net product = 0
(all reverted). It will burn **3 MORE** before parking: `lever.conc-shim-m1.stall=3`, budget=6.

**Fix.** For a `capability` lever whose collateral score is negative but whose *artifact* is valuable and
reusable, either (a) let capability arms ACCUMULATE their source onto a **separate capability branch**
(not the score-gated integration branch) so the next arm builds on it, or (b) preserve the reverted arm's
diff as a patch in `.loop-state/arm-N/` and feed it into the next same-lever arm's prompt. The current
"reset --hard on any non-win" is correct for the integration branch's soundness but causes total
re-derivation for multi-arm capabilities.

### BUG-3 [MEDIUM] — Orphaned `docker compose run` containers leak from worker Bash probes; two have been "Up 7 hours".
**Evidence.** `docker ps`:
```
static-analyzer-factory-dev-run-4f9d480bc85a | Up 7 hours | 2026-08-16 07:48:22
static-analyzer-factory-dev-run-3e9881ceaaae | Up 7 hours | 2026-08-16 07:45:04
```
`docker inspect` → `com.docker.compose.oneoff:True`, image `saf-dev:llvm18`. `docker top` shows each is a
worker-authored TSan probe: `sh -c 'cd /workspace; D=/workspace/tmp_tsan_probe2 ... clang-18 ... -fsanitize=thread
... timeout 8 $D/h ...'` with a **`[h] <defunct>` zombie** child. The 07:45/07:48 timestamps fall in the
**arm-18 race-confirmer** window (arm-18 dir mtime 08:02). Root cause: the worker launched `docker compose run`
(no `--rm`) via its Bash tool to compile+run a TSan binary; the test binary forked pthreads that outlived
`timeout 8`, leaving a defunct child; the container's PID 1 (`sh -c`) blocks reaping it, so the container
never exits. `saf_worker`'s wall-timeout (supervisor.sh:162) kills the `claude` process tree but NOT the
detached grandchild `docker compose run` container. Each orphan holds a container slot + a small CPU idle
(TIME 00:00:06 and climbing).

**Fix.** (1) Worker guidance/prompt: any `docker compose run` in a probe MUST use `--rm` AND a hard outer
`timeout` on the `docker` invocation itself (not just the inner binary), e.g.
`timeout 60 docker compose run --rm -T dev sh -c '... timeout 8 ./h ...'`. (2) Supervisor: after each arm,
reap oneoff containers labeled `com.docker.compose.oneoff=True` older than the arm start
(`docker ps -q --filter label=com.docker.compose.oneoff=True | xargs -r docker rm -f`) as part of
`revert_arm`/`keep_arm` cleanup. (3) Consider `TSAN_OPTIONS=...:die_after_fork=0` and `report_thread_leaks=0`
(already set) plus killing the process group on timeout (`timeout -k`).

### BUG-4 [LOW] — Capability levers with structurally-zero payoff keep consuming round-robin slots (race-confirmer).
**Evidence.** `race-confirmer` (no-data-race) arms 18/23/28 all `confirmed_delta=0`, `confirmed=0`
before and after (`.loop-state/arm-28/{before,after}.json`: `confirmed:0`, `emitted_false:129`,
`raw:131→129`). The scorer gives 0 confirmed because no-data-race FALSE needs a validated concurrency
witness the scorer can't confirm, and — per [[saf-svcomp-202-r8-nodatarace-defer]] — the reservoir is 100%
dedicated concurrency tasks where the sound gate can't help. These arms ACCUMULATE (build compiles + tests
pass) so they're preserved, but they will NEVER move the score. `stall=0` (ACCUMULATE resets it), so unlike
conc-shim-m1 they will run until `MAX_LEVER_ARMS=12` — i.e. **9 more** score-neutral arms.
**Fix.** An ACCUMULATE-only lever (N consecutive ACCUMULATEs, 0 KEEPs) should be de-prioritized/parked the
same way consecutive REVERTs park a lever. Add an "accumulate-without-a-keep" stall counter with its own
budget (e.g. park after 3 ACCUMULATEs and 0 KEEPs).

### BUG-5 [LOW] — Arms exceed MAX_TURNS silently and resume; several run to the 60-turn cap.
**Evidence.** `num_turns`: arm-5=61, arm-8=60, arm-12=60, arm-23=57, arm-25=56, arm-27=57, arm-24=56.
`MAX_TURNS=60` (loop.env). The supervisor's resume-on-max_turns loop (supervisor.sh:172) means an arm can
consume multiple 60-turn windows; the final result event only reports the LAST window's `num_turns`, so
total turns/cost per arm are under-counted in any single result event. Not a correctness bug, but the
journal/telemetry has no per-arm turn/cost/resume accounting → efficiency is invisible without parsing raw
transcripts (this audit).
**Fix.** Have the supervisor sum `num_turns`/`total_cost_usd` across all result events per arm and journal
them (turns, cost, wall, resume-count, decision). Feeds Task #18 (observability).

---

## B. Root cause of the concurrency reverts (precise)

The `conc-shim-m1` arms do NOT revert because the build broke, ran out of turns, or hit a rate limit — all
three completed cleanly (`subtype:success`, `.err` empty, no api_retry/rate-limit anywhere in any transcript;
the 44 `is_error:true` events across arms are ordinary worker-bash tool failures, 1–7 per arm). They revert
because of a **scoring/soundness mismatch baked into the lever definition**:

1. The lever is declared `family=unreach-call, scope=local`, so it is scored ONLY on unreach-call.
2. The worker's deliverable wires a concurrency confirmer into the shared `unreach_strategy` path, which
   runs on real unreach-call reservoir tasks.
3. That confirmer emits at least one **wrong FALSE** on a safe/TRUE unreach task (FP 0→1). One FP = −16
   confirmed. It also confirms ~6 new true concurrent FALSEs (emitted_false 33→39), but +6 recall << −16 FP.
4. Net `confirmed_delta = −16/−16/−14` < 0 → `verify_arm.py` returns REVERT (a `local` lever has no
   ACCUMULATE-on-regression path).
5. `revert_arm` hard-resets, discarding `concurrency.rs`, so the NEXT concurrency arm re-derives it.

So the pattern "ALWAYS REVERT, never ACCUMULATE" is fully explained: ACCUMULATE requires `delta==0`; these
arms have `delta<0`. The work is wasted AND re-derived each time. The concurrency capability itself may be
sound-in-principle, but it is being measured by a property it can only hurt.

---

## C. Waste / re-derivation patterns

- **conc-shim-m1 (17,22,27): pure re-derivation.** ~470-line `concurrency.rs` rewritten 3× because revert
  discards it. ~111 turns / ~$13.2 / ~53min, net 0. Will repeat 3 more times before parking (stall 3/6). [BUG-2]
- **race-confirmer (18,23,28): genuine accumulation, but zero score ROI.** These DO build on each other
  (git stat: arm18 creates `datarace.rs`+236L `commands.rs`; arm23 extends `commands.rs` +92/−22, adds
  `fast_paths.rs` + smoke tests; arm28 extends `commands.rs` +80 + tests). Not re-derivation — but the
  scorer confirms 0 and structurally always will, so it's polishing an unscoreable capability. ~119 turns /
  ~$13.5 across the three, +0 confirmed, will run to 12 arms. [BUG-4]
- **exec-validator-gate ACCUMULATE tail (arms 4,8,10,11,12,15,20,25 = 8 of its 11 arms delta=0).** After the
  early wins (arms 1,2,3,5,6,7,9 = +5+1+9+7+27+6+9 = +64 confirmed), the lever plateaued: 8 subsequent arms
  produced 0 additional confirmed but kept accumulating. Diminishing returns not detected/curbed. [relates BUG-4]
- **Productive levers (real, non-wasted):** exec-validator-gate (+64 early), overflow-recall (arm16 +3),
  termination-recall (arms 19/24/29 = +64/+24/+6 = +94). The overall checkpoint at arm 24 recorded
  CONFIRMED=601 (unreach=19, no-overflow=115, termination=230, valid-memsafety=237, no-data-race=0), up from
  511 — so the loop IS making real progress, concentrated in termination + exec-validator + overflow.

---

## D. Worker-efficiency stats (arms 1–29; arm-30 was mid-flight, turns=0)

- **Total spend:** ~Σ$118 across 29 arms (range $1.42–$7.76/arm), ~Σ 5.4 hrs wall of worker time.
- **Turns vs cap (60):** median ~31; at/over cap: arms 5(61), 8(60), 12(60); near cap (55-57): 6,23,24,25,27.
  Fast bail-outs: arm-17 (3 turns/$1.46 — did full shim in one shot!), arm-20 (4), arm-15 (9), arm-9 (15).
- **No rate-limiting at all:** zero api_retry / 429 / 529 / overloaded / usage-window events in any
  transcript or `.err` file. `.err` files all 0 bytes. The supervisor's elaborate rate-limit/resume
  machinery has not been exercised this run.
- **No worker aborts/failures:** every arm's result event is `subtype:success, is_error:false`. No
  WORKER_FAIL journal entries. No arm hit `WORKER_MAX_ATTEMPTS`.
- **is_error:true = worker bash tool errors** (its own failed probe commands), 1–7 per arm, benign.
- **Cost concentration:** the two structurally-doomed capability levers (conc-shim-m1 + race-confirmer)
  consumed ~$26.7 / ~230 turns (≈23% of total spend) for **0 net confirmed score**.
- **Efficiency verdict:** workers are NOT bailing early or thrashing; they complete their scoped task
  competently. The waste is not worker-level — it's **supervisor/lever-level**: the loop keeps spending on
  levers that cannot score (mis-scoped conc, unscoreable race) and throws away reusable capability artifacts.

---

## E. Recommendations for the loop

1. **[BUG-1] Fix or park `conc-shim-m1` immediately.** It is guaranteed to REVERT and inject a −16 FP each
   run, and will burn 3 more arms before self-parking. Either give concurrency-FALSE its own scored
   property/witness, or park the lever (`touch .loop-state/lever.conc-shim-m1.parked`) until a concurrency
   scorer exists. Also reclassify it `crosscut` (it edits shared `unreach_strategy`), not `local` — the
   `local` tag is a lie that suppresses cross-family visibility.
2. **[BUG-1 soundness] The FP is a real soundness regression.** `concurrent_replay_confirms_false` emits a
   wrong FALSE on a safe unreach task. Before ANY concurrency confirmer is wired into a competition surface,
   it must be fail-closed. This is exactly the redline in CLAUDE.md ("never emit false without concrete
   confirmation"); the loop's stratified-sample gate caught it (good), but the lever will keep re-injecting it.
3. **[BUG-2] Stop discarding reusable capability artifacts.** Let capability-mode levers ACCUMULATE their
   source onto a dedicated capability branch (isolated from the score-gated integration branch), or persist
   the reverted diff and prime the next same-lever arm with it. Kills the 3×/6× re-derivation of `concurrency.rs`.
4. **[BUG-3] Reap orphaned oneoff containers + fix probe hygiene.** Add a post-arm sweep of
   `com.docker.compose.oneoff=True` containers; instruct workers to always `docker compose run --rm` with an
   outer `timeout` and process-group kill. Two are leaking CPU/slots right now (safe to `docker rm -f` the
   two "Up 7 hours" ones — outside my read-only remit).
5. **[BUG-4] Add an ACCUMULATE-without-KEEP budget.** Park a lever after K consecutive ACCUMULATEs with 0
   KEEPs (race-confirmer will otherwise run to 12 arms at $4-5 each for 0 score). Same for a KEEP lever that
   plateaus (exec-validator-gate's 8-arm delta=0 tail).
6. **[BUG-5 / Task #18] Journal per-arm efficiency.** Sum turns/cost/wall/resume-count/decision per arm into
   the journal. Right now the only way to see that 23% of spend produced 0 score is to hand-parse transcripts.
7. **Round-robin is over-fair to dead families.** Family round-robin gives no-data-race and (mis-scoped)
   concurrency equal turns with the 3 productive families, guaranteeing ~40% of arms target structurally-0
   levers. Weight family selection by realized ROI (recent Δconfirmed), or drop families with no scoreable
   witness path until the scorer supports them. This is the core "generalization" issue for plan 205 /
   Task #19: the loop optimizes pool-recall on families it can score and spins uselessly on families it can't.
