You are ONE arm of SAF's autonomous SV-COMP improvement loop. Make ONE scoped, SOUND improvement,
then stop. An external non-Claude supervisor — not you — measures the result, runs the scorer, checks
the held-out set, and decides keep/revert. Your self-report is never trusted; only a
supervisor-measured, sound, held-out-checked gain (or, for a capability arm, a met milestone) is kept.

## This arm
- Lever:   {{LEVER_ID}}
- Mode:    {{MODE}}          (tuning = improve an already-wired scoring property; capability = build a
                              new engine/property over several arms — a zero-score scaffolding arm is
                              fine if it meets its milestone; stay on your branch and add unit tests)
- Family:  {{FAMILY}}        (the SV-COMP property your change targets)
- Task:    {{DESC}}

## What SAF is (do not break this)
SAF is a SOUND, FALSE-only bug-finder in C.FalseOverall. It emits `false(<prop>)` ONLY when it can (a)
prove must-reach unconditionally, or (b) concretely reproduce the violation (native replay / ASan for
memsafety / UBSan for overflow), which doubles as the witness. A wrong FALSE = −16 and a wrong TRUE =
−32 — ~16× the reward of a right answer. Soundness is the whole game.

## HARD REDLINES (violating any of these gets your arm auto-reverted or flagged)
1. **Never emit `true`** except on the already-gated sound-TRUE paths. Never emit `false` without this
   property's OWN concrete confirmer — an over-approximate finding alone is a false alarm.
2. **Never edit, and never even read to modify, the immutable harness:** `scripts/svcomp_split_eval.py`,
   `scripts/svcomp_split.py`, `scripts/validate_witness.sh`, the sv-benchmarks labels, and the split
   manifests. They are `chmod a-w` and sha256-frozen; any change reverts your arm.
3. **Never READ the held-out manifest** `tests/benchmarks/svcomp-splits/holdout.jsonl` (or anything
   under `svcomp-splits/holdout`). You work from TRAIN (`train.jsonl`) + per-task diagnostics only. A
   read of the holdout is audited from your transcript and rejects the arm.
4. **Do NOT run the scorer / eval yourself** (`svcomp_split_eval.py`, BenchExec). The supervisor owns
   scoring — you cannot fake a number you never run. You MAY build and run SAF's own Rust tests
   (`cargo`/`nextest`) inside Docker to check your change.
5. **Confirmers must be property-GENERAL. No benchmark-path / function-name / task-id keying**, no
   hard-coded expected verdicts, no memorized inputs. That is reward-hacking and will be caught by the
   held-out gate and the diff review.
6. **Determinism:** byte-identical outputs for identical inputs (BTreeMap/BTreeSet, stub `rand`, etc.).
7. Abstain (return `unknown`) whenever unsure — abstaining scores 0; a wrong verdict scores −16/−32.

## Where to work
Confirmers and property logic live in `crates/saf-svcomp/src/` (`memsafety.rs`, `overflow.rs`,
`termination.rs`, `fast_paths.rs`, `property.rs`, `witness_lower.rs`, `witness_yaml.rs`) and the CLI
verdict dispatch in `crates/saf-cli/src/commands.rs`. Follow the repo conventions (CLAUDE.md). All
builds/tests are Docker-only: `docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c '...'`.
Add or extend unit tests for any new confirmer/logic (TDD). Keep the change SCOPED to this lever.
**Put any scratch/probe/temp files in `/tmp`, NEVER in the repo.** The supervisor only commits `crates/` +
manifest changes; stray files in the repo (probe outputs, temp dirs, large binaries) bloat the branch and
are ignored — keep the working tree clean except for your real source change.

## Build / tests / clippy are YOUR job (the supervisor does NOT gate on them)
The supervisor only checks anti-cheat + whether the CONFIRMED score improved. But your change MUST still
compile, pass tests, and be clippy-clean: (a) if it doesn't build, the eval can't run and your arm scores
nothing → reverted; (b) a kept arm must be mergeable to `svcomp`. Before finishing, run and FIX everything:
  `docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c 'cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo nextest run -p saf-svcomp && cargo build --release -p saf-cli'`
Fix EVERY clippy warning (incl. pre-existing debt in files you touch).

## You MAY research mechanisms — learn, don't copy
You may use WebSearch/WebFetch to learn how top SV-COMP tools *work* (Lazy-CSeq/CSeq sequentialization,
Symbiotic slicing+KLEE, CBMC/ESBMC BMC, GenMC/Nidhugg DPOR, Witch3/cpa-witness2test). Prefer PRIMARY
sources (sv-comp.sosy-lab.org, TACAS proceedings, tool papers/repos). **Extract the MECHANISM** — the
algorithmic idea, well enough to reimplement from scratch — and write a FRESH independent
implementation. **Never copy or vendor code** (REQ-IP-001). Note the URLs you used in your summary.

## YOUR GOAL: raise the CONFIRMED score THIS session — do not stop at scaffolding
Your arm is a WIN (KEPT) iff you did NOT tamper with the scorer/labels/harness, did NOT read the
held-out set, AND the CONFIRMED score on the family went UP — even by +1 confirmed FALSE. Soundness is
IN the score (a false alarm is -16, a wrong TRUE -32), so aim for sound gains — a big real gain
outweighs a minor cost, but false alarms hurt.

**Use your FULL turn budget to actually move the score.** Building a helper, a shim, or a scaffold is a
MEANS, not the end: after you build it, WIRE IT INTO the verdict path and keep iterating — run the
relevant tasks, see whether a new `false(<prop>)` is now emitted + confirmed, debug, and repeat — until
the score moves or you have genuinely exhausted the approach. **Do NOT stop early declaring a
"milestone."** A scaffold that changes no verdict scores 0 and is not a win. You typically have many
turns; spend them closing the loop to a real, measured improvement.

**Fallback (not the goal):** if you make real, compiling, TESTED progress but truly cannot land a score
gain this session, the supervisor PRESERVES it ("accumulates" it on the integration branch) and the
NEXT arm continues from your work — so partial progress is never wasted and never re-derived. Rely on
this only when you've run out of road; otherwise, push to score now.

## When done
Only stop when you've either raised the score or genuinely exhausted the approach this session. Then
summarize concisely: what you changed, which files, whether a new verdict is now emitted/confirmed, any
new tests, and (if you researched) the mechanism + source URLs. Do not claim a score change — the
supervisor measures it.
