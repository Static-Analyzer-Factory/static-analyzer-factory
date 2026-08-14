# Plan 203 — Non-overfitting SV-COMP benchmark harness (train/holdout + BenchExec)

**Branch:** `svcomp`  **Date:** 2026-08-13  **Status:** scripts written on the laptop;
pure split logic unit-tested green locally; **NOT yet run on the VM against the real
sv-benchmarks pool** (laptop is source-only, submodule not cloned — see
[[saf-svcomp-vm-env]], [[saf-svcomp-workflow]]).

## Motivation

The sv-benchmarks task pool (SV-COMP 2025 = **33,353 C tasks**, tag `svcomp25`) is
**fully public with NO hidden/held-out test set** (adversarially verified against the
SV-COMP 2025/2026 rules + TACAS reports this session). A tool tuned against the pool
can therefore silently **overfit**, and any recall/score measured on the tuned-on
tasks overstates real competition performance. We need an *honest* split so numbers
reported on a holdout are a genuine generalization estimate.

Two facts drive the design:
1. **Split by ORIGIN family, never per task.** Tasks in one directory are
   near-duplicates from one generator (Juliet CWE families, `array-examples/*`, …),
   so a per-task random split leaks the test set into training. Whole origin groups
   must go to one side only.
2. **Year/edition holdout is the most honest estimate.** Tune on an older edition,
   test on tasks *added* in the next edition (genuinely-new benchmarks). This is the
   published methodology for measuring verifier generalization.

Also verified + corrected this session: **CONFIRMED vs RAW scoring gap** — a correct
FALSE scores +1 only if its violation witness is validator-CONFIRMED; unconfirmed-
but-correct = 0. Reporting raw verdict accuracy overstates the competition score, so
the harness must score confirmed, not raw.

## Deliverables (all committed to `svcomp`)

| File | Role |
|------|------|
| `scripts/svcomp_split.py` | Splitter. Pure `assign_groups` (origin-grouped, stratified, deterministic via SHA-256) + git-tag edition holdout. Emits per-split `.jsonl`, per-property `.set`, a per-split `.bench.xml`, and `split_manifest.json` (audit). |
| `scripts/test_svcomp_split.py` | Unit tests for the pure split core (no leakage / determinism / fraction / depth). Runs with plain `python3` — **7/7 green locally**. |
| `scripts/svcomp_split_eval.py` | Manifest-driven, property-generic blind `saf verify` eval. Reports **RAW vs CONFIRMED** SV-COMP score + recall + witness-confirmation %, and the −16/−32 soundness audit, per split. |
| `benchmark-defs/saf.xml` | Full official-style BenchExec definition, competition limits (900 s / 15 GB / 4 cores), one rundefinition per property, tasks from the official category `.set` files. |
| `scripts/run_benchexec_svcomp.sh` | Runner — installs the vendored `benchexec/tools/saf.py` tool-info, puts `saf` on PATH, runs BenchExec + `table-generator`. |

Reuses existing conventions verbatim: the `saf verify` blind entry point,
`benchexec/tools/saf.py` (the exact competition interface), `scripts/validate_witness.sh`
(witnesslint → CPAchecker → cpa-witness2test), the `.yml` regex parse and `stride()`
sampling from `scripts/svcomp_verify_eval*.py`, and the SV-COMP 2026 scoring from
`crates/saf-bench/src/svcomp/scoring.rs`.

## Methodology

### Split modes (auto-selected)
- **edition** (`--holdout-tag T2`): train = target-property tasks present at
  `--train-tag`; holdout = tasks whose `.yml` was ADDED between the tags
  (`git diff --diff-filter=A T1..T2`). Requires the submodule checked out at (or
  after) the holdout tag. The most honest estimate.
- **grouped** (default, single edition): whole origin groups (first `--group-depth`
  path components under `c/`, default **2** → Juliet splits by CWE but variants
  within a CWE stay together) assigned train/holdout, stratified so each property
  keeps ~`--holdout-frac` (default 0.2) of its tasks in holdout, in stable-hash
  order. A hard guard aborts if any group straddles the split.

### Scoring (per split)
- **RAW**: `SvCompOutcome` semantics (+2/+1/−16/−32/0) — the naive verdict score.
- **CONFIRMED**: a correct FALSE earns +1 only if its violation witness is CONFIRMED
  (`validate_witness.sh`); a correct TRUE earns +2 on the verdict alone for the
  witness-not-required properties (termination / valid-memsafety / valid-memcleanup /
  no-data-race) and needs a confirmed correctness witness for unreach-call /
  no-overflow (SAF never emits TRUE there). Penalties (−16 / −32) apply regardless.
- Soundness audit: false alarms (−16) and wrong TRUEs (−32) MUST be 0.

### Faithful full run (BenchExec)
`benchmark-defs/saf.xml` at competition limits, driven by `run_benchexec_svcomp.sh`.
Witnesses are collected (`<resultfiles>`); validator confirmation is a separate,
heavier stage — for a confirmed number without that pipeline, use
`svcomp_split_eval.py --confirm-witness` on the same split.

## Exact VM run (on `ubuntu@cd-vm-15-ai-vm`, in the dev container)

```bash
# 0. one-time: clone the pool (+ fetch tags for edition holdout)
git submodule update --init tests/benchmarks/sv-benchmarks
git -C tests/benchmarks/sv-benchmarks fetch --tags

# 1. build the split (grouped 80/20 shown; add --holdout-tag svcomp26 for edition mode)
python3 scripts/svcomp_split.py --group-depth 2 --holdout-frac 0.2 --seed 0
#   -> tests/benchmarks/svcomp-splits/{train,holdout}.{jsonl,<prop>.set,bench.xml}
#      + split_manifest.json (per-property/per-category counts + achieved holdout %)

# 2. score TRAIN vs HOLDOUT separately (confirmed witnesses; --sample for a quick pass)
docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c '
  cargo build --release -p saf-cli &&
  python3 scripts/svcomp_split_eval.py --manifest tests/benchmarks/svcomp-splits/train.jsonl   --confirm-witness --jobs 4 -o train-score.json &&
  python3 scripts/svcomp_split_eval.py --manifest tests/benchmarks/svcomp-splits/holdout.jsonl --confirm-witness --jobs 4 -o holdout-score.json'
#   the HOLDOUT confirmed score is the honest generalization number;
#   train-minus-holdout is the overfit gap.

# 3. (optional) faithful full BenchExec run at competition limits
docker compose run --rm -T dev scripts/run_benchexec_svcomp.sh benchmark-defs/saf.xml SV-COMP_unreach-call
```

## Caveats / limitations
- **Not yet run at scale.** Only the pure split logic is verified (locally, 7/7). The
  blind-eval + BenchExec paths need the VM (LLVM + benchexec + the cloned pool).
- **Validator capability bounds the confirmed %.** If CPAchecker is absent the eval
  reports `LINT_ONLY` and treats confirmed as a lower bound; some property/witness
  classes a single validator can't confirm (the competition runs several).
- **Label noise:** expected verdicts are corrected edition-to-edition; pin to a tag.
- **Stratification is approximate** under whole-group assignment; `split_manifest.json`
  reports the achieved per-property holdout fraction — tune `--group-depth` / `--seed`
  / `--holdout-frac` if a property comes out empty or lopsided.
- **`.set` names in `benchmark-defs/saf.xml`** are the long-stable category files;
  confirm against the checked-out tag (`ls tests/benchmarks/sv-benchmarks/c/*.set`).

## Acceptance
Split: no origin group straddles train/holdout (guarded + unit-tested); deterministic;
achieved holdout fraction near target per property. Eval: RAW and CONFIRMED scores +
recall reported separately for train and holdout with a clean −16/−32 audit (FP=0,
wrong-TRUE=0). BenchExec: `saf.xml` validates and runs at competition limits producing
a `table-generator` score table.

## Outcome (2026-08-14) — full-pool runs found & fixed 7 issues; final sound + deterministic

The harness ran full-pool on svcomp25 (train) + svcomp26 (holdout). Each run surfaced a
real bug that the earlier stratified samples missed; all fixed and committed on `svcomp`:

1. `a40076e` split manifest stored abs host paths → invisible in Docker (`/workspace`).
2. `881ef8b` no-straddle guard scoped to grouped mode (edition holdout legitimately spans).
3. `3ad7c48` `--per-task` diagnostic JSONL (verdict/outcome/duration/stderr on misses).
4. `54349c3` narrowed CWE761 memsafety R1 to length-walk `ldv_strlen` (FP 38→0, keeps
   ~1000 real CWE12x buffer bugs that fault in `ldv_memcpy`).
5. `88ab2c8` stub `rand()/srand()` via `-Wl,--wrap` (Juliet `*_rand_*` were non-deterministic).
6. `992eac3` gate unreach FALSE replay on `reachable_spawns_threads` (2 goblint racefree FPs).
7. `604256b`/`7b0fff6` memory caps: `hard_rss_limit_mb=3072` (ASan RSS monitor) + eval
   RSS-subtree watchdog — a harness that OOM'd the 62 GB swap-less host six times is now bounded.
8. `bf71cda` overflow mini-fuzz uses `2^30` not `INT_MAX` (kills `INT_MAX+1` loop-counter/
   accumulator FPs: Parts, ESOP2008).

**Final measured svcomp25 (all fixes, capped, unreach sampled→extrapolated):** memsafety
5361 TP / **FP 0**, no-overflow 1201 TP / **FP 1**, unreach ~224 (6.6% of 3392) / FP 0,
termination 396 (198 TRUE), ndr 0 → **RAW ≈ 7166**; **confirmed C.FalseOverall ≈ 3960 +
~396 termination.** svcomp26-new (holdout) adds +33 raw. Versus the first run: **40 false
alarms → 1**, and **memsafety recall 4368 (rand-noisy) → 5361 (stable)**; deterministic;
**no OOM** (`rss_kills=0`, ran the previously-fatal memsafety at jobs=16 with 60 GB free).

**The 1 residual FP** = `termination-numeric/twisted` (`return i+j` where loop counters
`i,j` reach nondet bounds `k=l=2^30` → `2^30+2^30` overflow) — indistinguishable in the
UBSan report from a genuine `x+x` overflow (the exact pattern the confirmer relies on), so
unfixable without losing all addition-overflow recall → accepted as inherent label noise
(1/9060). Result JSONs on the VM: `final3-{train,holdout}-*.json` + `pertask4-*.jsonl`.
