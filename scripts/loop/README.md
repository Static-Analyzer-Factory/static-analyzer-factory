# SAF autonomous improvement loop (plan 204)

An **external, non-Claude supervisor** (`supervisor.sh`) that repeatedly asks a `claude` worker to make
ONE sound improvement to SAF's SV-COMP results, then **deterministically gates** the result (no Claude
in the gates, so a worker hitting a rate limit can never wedge the loop or slip a bad change through).
Kept arms land on scratch branches for human cherry-pick. It **never pushes** and **never merges** to
`svcomp`. Full design: `plans/204-svcomp-autonomous-loop.md` (+ `Addendum A`) and
`plans/204-research-brief.md`.

## Files
| File | Role |
|------|------|
| `supervisor.sh` | The loop. `--once` (watched dry-run, one arm), `--loop` (service), `--baseline` (score HEAD). Delegates every decision to the tested Python in `lib/`. |
| `lib/gates.py` | Pure gate logic: immutable sha256 check, holdout-read audit, soundness (FP=0 ∧ wrong-TRUE=0), tuning delta, capability milestone, keep/revert/reject. |
| `lib/verify_arm.py` | The bash↔python boundary: collects inputs, runs the gates, prints one decision word. |
| `lib/ratelimit.py` / `lib/worker_status.py` | Classify stream-json `api_retry`/`result` events → transient / abort / max_turns / sleep-to-reset(epoch). |
| `lib/compute_evidence.py` | Supervisor-computed capability milestone evidence (never the agent's self-report). |
| `arm_prompt.md` | The fixed brief piped to each worker (redlines, capability semantics, research/learn directive). |
| `levers.tsv` | ROI-ordered lever menu (`id⇥mode⇥family⇥desc`). `#`-prefix a line to PARK a lever. |
| `worker-settings.json` + `hooks/deny-holdout.sh` | Preventive PreToolUse denylist blocking any worker access to the held-out manifest. |
| `loop.env` | Run-config (turns, jobs, caps, timeouts). |
| `report.sh` | Human score view (baseline, kept arms, held-out checks, journal, alerts). |
| `progress-journal.md` | Durable per-arm log; the loop reconstructs state from `git log` + this on restart. |
| `saf-loop.service` | systemd unit (Restart=always). **Do NOT enable until a dry-run passes.** |
| `tests/` | `run_tests.sh` runs the gate unit tests + the end-to-end supervisor smoke (no Docker/Claude). |

## Gates (every kept arm; deterministic bash/python, no Claude)
SOUNDNESS (absolute, both arm modes): ① immutable sha256 intact · ② holdout not read (hook + audit) ·
③ re-run the immutable scorer on TRAIN: **FP=0 ∧ wrong-TRUE=0** on the FULL family SAFE set · ④ nextest
+ clippy + fmt green. PROGRESS (mode-dependent): **tuning** → CONFIRMED train-delta > 0; **capability**
→ milestone (FP=0 on the SAFE set + a structural advance or ≥1 newly-confirmed FALSE), kept on an
accumulating `cap-*` branch. Every K kept arms, the svcomp26 held-out eval runs — a train gain that
doesn't reproduce there is a HARD reject.

## Run
```bash
# 0. Stage-0 tests (laptop or VM; no Docker needed)
bash scripts/loop/tests/run_tests.sh

# On the VM (cd-vm-15-ai-vm), from a CLEAN, current svcomp baseline (see plan 204 Addendum A8):
# 1. score the baseline (freezes immutables on first run)
scripts/loop/supervisor.sh --baseline

# 2. Stage 1 — ONE watched arm, foreground, then STOP (pick the safe tuning lever)
scripts/loop/supervisor.sh --once exec-validator-gate

# 3. Stage 2/3 — enable the service ONLY after the dry-run is approved
systemctl --user enable --now saf-loop
scripts/loop/report.sh            # monitor;  journalctl --user -u saf-loop -f
```

## Redlines
Never push; never merge to `svcomp`; never edit the scorer/splitter/validator/labels/holdout; never
emit `true` off gated paths; FP=0 ∧ wrong-TRUE=0 every kept arm; byte-determinism; all builds/evals in
Docker on the VM; confirmers stay property-general (no benchmark-path/function/task-id keying).
