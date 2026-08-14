# Kickstart prompt — build & run SAF's autonomous improvement loop (plan 204)

> Paste this as the opening message of a fresh session on the SAF repo. It is the operator brief for
> BUILDING plan 204 test-first, doing ONE watched dry-run, then (on the user's standing authorization)
> starting the loop on the VM. All soundness redlines hold throughout.

---

Build the autonomous improvement loop for SAF (static-analyzer-factory), a sound SV-COMP bug-finder.
The design is written, approved, and research-enriched — your job is to BUILD it test-first, do ONE
watched dry-run, then start the loop on the VM. Do NOT enable the unattended service until the watched
dry-run passes and you have reported it.

## READ FIRST (spec + context — do not re-derive)
- `plans/204-svcomp-autonomous-loop.md` — THE SPEC. Follow it exactly, **including `Addendum A`** (the
  capability track, ConcurrencySafety-FALSE, research/learn arms, verified headless mechanics — these
  SUPERSEDE the base plan where they differ).
- `plans/204-research-brief.md` — the verified mechanism brief behind Addendum A (per-claim verdicts,
  two refuted "obvious" levers you must NOT re-introduce).
- `plans/203-svcomp-nonoverfit-harness.md` — the eval harness + the baseline you must beat.
- `CLAUDE.md` → "SV-COMP Competition Work" and its 8 "Soundness redlines" (non-negotiable).
- Memories (via MEMORY.md): `saf-svcomp-nonoverfit-harness`, `saf-svcomp-vm-env`, `saf-svcomp-workflow`,
  `saf-svcomp-branch-sequencing`, `saf-svcomp-198-reachability-gate`, `saf-svcomp-202-r8-nodatarace-defer`.
- The IMMUTABLE scorer/harness you must NEVER weaken: `scripts/svcomp_split_eval.py` (RAW-vs-CONFIRMED
  score + FP/wrong-TRUE audit + `--per-task` + `--max-rss-mb`), `scripts/svcomp_split.py`,
  `scripts/validate_witness.sh`, and the split manifests generated on the VM at
  `tests/benchmarks/svcomp-splits/{train,holdout}.jsonl` (+ per-property `.set`).

## LOCKED DESIGN DECISIONS (plan 204 — do not revisit)
1. The supervisor is a PLAIN bash/python process — NOT a Claude instance. ALL loop control, ALL
   rate-limit handling, and ALL HARD gates are deterministic (no Claude), because worker `claude`
   sessions WILL hit rate limits and a Claude-based controller would be throttled at the same moment.
2. Claude is used only for (a) the per-arm improvement attempt and (b) an OPTIONAL adversarial diff
   review — both launched by the bash supervisor and subject to its rate-limit handling. Soundness never
   depends on either being available.
3. Kept arms auto-commit to a scratch branch `auto/loop-<date>`. NEVER `git push`, NEVER auto-merge to
   `svcomp`. The human cherry-picks wins.
4. systemd service (`Restart=always`) — but do NOT enable it until the watched dry-run passes.

## HARD GATES (all deterministic bash, no Claude — must hold through a rate-limit window)
Split gate (d) into a SOUNDNESS predicate (absolute, both arm modes) and a PROGRESS predicate
(mode-dependent). Every kept arm, both modes, clears the SOUNDNESS predicate:
  ① immutable-file sha256 unchanged (tamper → reject)
  ② holdout not read by the arm (PreToolUse denylist hook + post-hoc transcript audit)
  ③ re-run the IMMUTABLE scorer on TRAIN: **FP=0 AND wrong-TRUE=0**, on the **FULL** SAFE reservoir for
     the arm's family (NEVER a subsample — plan-203: sample FP=0 was 38 real FPs at scale)
  ④ nextest + clippy + fmt green
PROGRESS predicate:
  • **tuning arm** → CONFIRMED-score delta on TRAIN > 0
  • **capability arm** → milestone contract (FP=0 on the family's full held-out SAFE set + either a
    declared structural milestone advanced or ≥1 newly-confirmed FALSE the baseline missed), landing on
    an accumulating `cap-<capability>/<n>` branch, tagged not-yet-scoring.
  ⑤ (optional, SOFT) a fresh `claude` reviews the diff for redline breaches / copied code.
Otherwise revert the arm. Every K kept arms, the supervisor runs the svcomp26 HELD-OUT eval — a train
gain that does not reproduce on held-out is a **HARD reject** (the real anti-overfit control).

## CAPABILITY TRACK (the core enrichment — Addendum A2)
The loop must build NEW capabilities, not only tune existing recall. The SOUNDNESS gates ①–④ stay
ABSOLUTE and byte-identical for both modes; only the PROGRESS predicate is mode-dependent. The
supervisor picks tuning vs capability at lever-selection time from the human-curated taxonomy
(Addendum A7). A capability track gets a milestone budget (~6–8 arms); no advance → PARK the branch for
human review and rotate. A capability graduates to "tuning" once its confirmed FALSEs raise the
aggregate svcomp26 held-out score. Flagship capability: **ConcurrencySafety-FALSE** (Addendum A3).

## ANTI-REWARD-HACKING (critical)
- `chmod a-w` + sha256-gate the scorer / audit / labels / holdout each arm.
- The agent gets TRAIN + per-task diagnostics only; the SUPERVISOR (not the agent) owns and runs the
  held-out eval — the agent can't fake a number it never runs. Audit that no arm reads the holdout
  manifest.
- Verify-before-victory: never trust the agent's self-report; keep only on a supervisor-measured, sound,
  held-out-checked gain (or a met capability milestone).
- Do NOT justify the capability relaxation with "no exploit attacks the score-delta" — memorization/
  benchmark-keying IS the dominant exploit; the defense is soundness gates ①–④ (independent of the
  progress gate) + the frozen held-out gate + origin-family split + a **no benchmark-path/function-name/
  task-id keying** redline in `arm_prompt.md`.
- A new FALSE-emitting confirmer is the higher-risk class: scan it against the ENTIRE SAFE pool for
  latent FP classes before keep. "Confirmed by the pipeline" ≠ "correct verdict".
- Never edit the scorer/tests/labels/holdout; never emit `true` off gated paths.

## RATE-LIMIT HANDLING (bash, verified — Addendum A6)
Run workers with `claude -p --output-format stream-json --verbose --allowedTools <explicit>
--max-turns N`; capture `session_id`; drive all retries/resumes by it (never `--continue`).
- Classify from the structured `system`/`api_retry` stream event (field `error`), NOT prose: transient
  `{overloaded, server_error, rate_limit-throttle, null-status}` → jittered exp backoff + resume-by-id;
  hard `{authentication_failed, billing_error, oauth_org_not_allowed, invalid_request, model_not_found,
  max_output_tokens}` → abort/park; `unknown` → conservative bounded retry.
- Hard usage-window/quota is NOT auto-retried → parse the **Unix-epoch** reset (pipe-delimited, may be a
  non-JSON line — guard the parse) or poll status-line `rate_limits.*.resets_at`; else capped exp
  backoff; **sleep-to-reset**, then `claude -p --resume <session_id>`.
- Gate per-arm success on the terminal result `subtype=="success"` (NOT `is_error`, NOT exit code;
  `is_error` stays false on `error_max_turns`). On `error_max_turns` resume from `session_id`; treat
  exit 143 as supervisor kill.
- Add a **wall-clock watchdog independent of stream events** (stdout can stall). Env:
  `CLAUDE_CODE_MAX_RETRIES=3–5`, `API_TIMEOUT_MS≈1200000`, unset telemetry vars. Behind the VM proxy
  (`ANTHROPIC_BASE_URL`+`ANTHROPIC_AUTH_TOKEN`) don't trust CLI auto-classification — use the api_retry
  event + fixed-sleep fallback; **verify `--bare` compatibility with proxy auth before relying on it**.

## ARTIFACTS to build under `scripts/loop/`
`supervisor.sh` (the loop) · `arm_prompt.md` (fixed per-arm instructions + redlines + capability-track
+ research/learn directive from Addendum A5 + the no-keying redline) · `immutable.sha256` (checksum
manifest) · `review_arm.sh` (adversarial diff review) · `progress-journal.md` (durable shift-handoff) ·
`report.sh` (human score view) · `saf-loop.service` (systemd) · run-config (env, `--max-turns`,
`--allowedTools`, caps) · `tests/` (bash/python gate unit tests).

## OPERATIONAL FACTS
- VM: `ssh ubuntu@cd-vm-15-ai-vm` (16 cores / 62 GB, NO swap). ALL builds + evals are Docker-only on the
  VM: `docker compose run --rm dev sh -c '...'`. Code on branch `svcomp`; repo at
  `~/static-analyzer-factory`; sync `crates/`+`scripts/` via rsync per the `saf-svcomp-vm-env` memory
  (NEVER whole-tree `--delete`). Confirmers live in `crates/saf-svcomp/src/`.
- **VM baseline prerequisite (Addendum A8):** the VM `svcomp` checkout is at a stale HEAD (plan-191 era)
  with a large uncommitted working tree. Before Stage 1, bring it to a clean `svcomp` baseline matching
  the laptop HEAD via a `git bundle` transfer (no push to origin), then regenerate the splits.
- Claude Code installed + authed on the VM (`~/.claude-code.env`, proxy auth). Headless: `claude -p`,
  `--output-format stream-json --verbose`, `--max-turns`, `--dangerously-skip-permissions`,
  `--allowedTools`, `--resume/--session-id`. Hard usage windows are NOT auto-retried (sleep-to-reset in
  bash).
- Baseline to beat (plan 203): svcomp25 CONFIRMED `C.FalseOverall` ≈ 3960 + ~396 termination; sound (1
  inherent overflow FP), deterministic. Top levers: witness-confirmation % via an execution validator in
  the confirmation gate (Addendum A4), ConcurrencySafety-FALSE (currently 100% forfeited), and the
  2026-new unreach batch (STRUCTURAL/TDX-firmware — harness synthesis + frontend robustness, NOT
  BMC-dominant). unreach holdout is ~0% (generalization gap).

## WORK DISCIPLINE
- TDD the gate logic FIRST — tests for: (a) an injected fake FP → auto-revert; (b) tamper an immutable
  file → reject; (c) an arm that reads the holdout → reject; (d) a no-gain tuning arm → not kept;
  (e) a zero-delta capability arm that MEETS its milestone → kept on a `cap-*` branch; (f) a zero-delta
  capability arm that MISSES its milestone → reverted. Only then wire the real loop.
- Rollout: **Stage 0** build + gate tests → **Stage 1** bring the VM to a clean baseline, freeze
  immutables, run ONE arm manually, foreground, watched (a real lever, e.g. wire the execution validator
  into the confirmation gate, or the ConcurrencySafety Milestone-1 shim de-risk) — confirm
  orient→act→verify→held-out→checkpoint + the tamper/holdout gates all fire → STOP and report →
  **Stage 2** enable the systemd service and confirm it survives → **Stage 3** autonomous; review
  `auto/loop-<date>` / `cap-*` periodically; cherry-pick wins.
- Commit harness code to `svcomp`; keep kept arms on `auto/loop-<date>` and capability scaffolding on
  `cap-*`; never push; never merge to `svcomp`.

Start by re-reading `plans/204` (incl. Addendum A) + `plans/204-research-brief.md` +
`scripts/svcomp_split_eval.py`, then propose the Stage-0 build breakdown before writing code.
