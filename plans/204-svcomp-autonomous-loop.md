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
