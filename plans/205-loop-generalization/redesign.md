# Plan 205 — Loop redesign: reward GENERALIZATION, not pool memorization

**Branch:** `svcomp`  **Date:** 2026-08-17  **Status:** DESIGN / proposal only (no builds, no edits to
production files). Supersedes the plan-204 gate wherever they differ; preserves every plan-204 soundness
and anti-cheat invariant. REQ-IP-001 (independent implementations) holds throughout.

Context read in full before writing: `scripts/loop/{supervisor.sh,levers.tsv,arm_prompt.md,loop.env}`,
`scripts/loop/lib/{gates.py,verify_arm.py}`, `scripts/svcomp_split.py`, `scripts/svcomp_split_eval.py`,
`plans/203-*`, `plans/204-*` (+ research brief).

---

## 0. The problem, stated precisely (measured)

The loop **raises the TRAIN score but the genuinely-new svcomp26 holdout stays flat** (confirmed 7→7→8 /
max 1770 = 0.45%; new-task unreach-call recall **0%**, memsafety 6.1%, overflow 9.5%). Meanwhile the
per-arm gate's TRAIN sample confirms memsafety 47% / overflow 23% / unreach 3.8%.

Root cause is structural, in the gate, not in SAF:

1. **The reward is pool-recall on near-duplicate clusters.** The gate keeps an arm iff `confirmed_delta > 0`
   on a stride-sampled TRAIN subset (`EVAL_SAMPLE=1000`/property). TRAIN is dominated by
   **Juliet_Test/CWE\*** and other generator families — thousands of near-identical CWE tasks. A change that
   confirms *k* more members of one CWE cluster scores `+k` and is kept, whether or not it encodes any
   transferable reasoning. This is textbook benchmark memorization (SpecBench/EvilGenie), and plan-204 §A2
   already names it "arguably THE dominant exploit" — but the gate still measures exactly it.

2. **Capability levers that build novel solving power score ~0 on the Juliet sample and get parked.**
   `conc-shim-m1`, `symbolic-oracle`, `harness-havoc`, `air-slicing` target reasoning-heavy tasks the
   Juliet-heavy sample barely contains. Their honest per-arm `confirmed_delta` on that sample is ≈0, so
   under the current rule they REVERT (delta≤0, and often no ACCUMULATE because they *do* touch `crates/`
   but produce no new confirmed FALSE on the sampled families). After `LEVER_BUDGET=6` consecutive REVERTs
   they PARK. The loop therefore **never invests in generalization** — the one thing it needs.

3. **The only real generalization signal is read every 3rd kept arm and is non-actionable.**
   `maybe_heldout_check` runs the svcomp26 holdout eval every `HELDOUT_EVERY_K` kept arms and writes it to
   disk with a comment "a train gain that does not reproduce on held-out is a HARD reject (Addendum A2)" —
   but **nothing consumes it**. It is journalled for a human, never fed back into keep/revert or lever
   priority. So the loop is blind to overfitting in-loop.

The fix is to **redefine "progress" as movement on a validation set that excludes the generator clusters**,
add a **cluster-dedup reweighting** so a single generator family cannot dominate the number, and **make the
frozen svcomp26 holdout an in-loop actionable signal** (still read-forbidden to the worker). All while
keeping the soundness predicate (FP=0 / wrong-TRUE=0 intrinsic to the score) and both anti-cheat gates
byte-identical.

---

## 1. Core idea (one paragraph)

Carve, **from TRAIN only**, a **held-IN VALIDATION set (`val`) that excludes every generator/Juliet cluster**
— i.e. the reasoning-heavy, mostly-hand-written families (loops, arrays, floats, bitvectors, recursive,
product-lines, ntdrivers/ldv device drivers, systemc, sequentialized, TDX firmware, the algorithmic
categories). The per-arm gate's KEEP decision is driven by a **generalization score** computed on `val` with
**per-cluster dedup weighting**, not by raw pool-recall on the Juliet-dominated sample. Pool-recall on the
full TRAIN sample is kept only as a **soundness/no-regression guard and a tie-breaker**, never as the thing
being maximized. Capability arms are additionally credited with an **ACCUMULATE-plus** signal so genuine
solving-power that lands a first `val` or holdout confirmation is preserved and rewarded even when the
Juliet pool number is flat. The read-forbidden svcomp26 holdout becomes an **actionable trend gate** that
deprioritizes (parks) any lever whose `val` gains do not reproduce on truly-new tasks.

Naming: `train` = full pool (unchanged; still the soundness reservoir). `val` = the **non-generator subset
of TRAIN**, held-IN, freely readable by the worker (it's TRAIN). `holdout` = svcomp26-new, **read-forbidden**,
supervisor-only (unchanged).

---

## 2. What changes, file by file

### 2.1 `scripts/svcomp_split.py` — emit a validation set and a cluster map (additive, immutable)

Add two outputs; change nothing existing (so old manifests keep working and determinism is preserved).

**A. A `generator`/`cluster` tag per task, and a `val` manifest.**
Classify each TRAIN task as `generator` (near-duplicate machine-generated cluster) or `reasoning` (the
generalization surface). The classification is **structural and deterministic** — no hand-list of "good"
tasks (that would itself be overfitting). Two signals, combined:

- **Directory-family signal (primary).** The task's `group` (first `--group-depth` components under `c/`,
  already computed) whose top component is a known **generator root** is `generator`. Generator roots are a
  small, stable, *category-level* denylist, not a task list:
  `Juliet_Test`, `weaver` (generated concurrency), `goblint-regression`/`aws-c-common` only if
  generated — **kept minimal and reviewed**. Everything else (`loops*`, `array-*`, `float*`,
  `bitvector*`, `recursive*`, `product-lines`, `ntdrivers*`, `ldv-*`, `systemc`, `seq-*`, `eca-*`,
  `hardness*`/TDX, `nla-digbench`, `termination-*`, the `*-crafted` and algorithmic dirs) is `reasoning`.
- **Cluster-size signal (backstop, catches unlabeled generators).** Any `group` with **> `--cluster-max`
  tasks for a single property** (default 200) is treated as a `generator` cluster regardless of name — a
  family that large is, by construction, near-duplicate generator output and must not dominate the
  validation number. This is the robust part: it needs no maintenance as new generated families appear.

  A task is `reasoning` iff (dir-family says reasoning) **and** (its cluster is not oversized). This is
  fail-safe toward *excluding* from `val`: when unsure, a task stays out of `val` (still in `train`), so
  `val` can only get *cleaner*, never contaminated, if the denylist is imperfect.

Emit, under `--out-dir`, exactly as `train`/`holdout` are emitted today:
- `val.jsonl` — the reasoning subset of TRAIN (one task/line, same schema, **plus** a new
  `cluster` field = the `group`, and a new `generator: false` field; `train.jsonl` rows also get
  `cluster` + `generator` for the reweighting in §2.3).
- `val.<property>.set` + `val.bench.xml` (parity with the other splits; lets a human BenchExec it).
- Extend `split_manifest.json` with a `validation` block: per-property counts, number of reasoning
  clusters, and the achieved `val` fraction of TRAIN, so the split is auditable.

**B. A per-task cluster weight for dedup (written into the manifests).**
For the reweighted scoring (§2.3) add a field `cluster` to every emitted row (train + val). The
**weight is not stored** (so the manifests stay label-only and deterministic); the scorer derives it at
eval time from the sampled rows (§2.3), which keeps the weight a pure function of the sample and avoids a
second immutable to maintain.

**Determinism:** all new listings sorted; `val` membership is a pure function of
`(pool, group-depth, generator-roots, cluster-max)`. Same inputs ⇒ byte-identical `val.jsonl`. The
generator-roots list and `cluster-max` are **arguments with fixed defaults recorded in
`split_manifest.json`** — reproducible, and part of the frozen immutable set (§4) so a worker can't widen
`val` to sneak Juliet back in.

> Why define "novel/reasoning" this way and not by a curated allow-list: a curated list of tasks is exactly
> the memorization surface we're trying to escape, and it rots. Deriving `reasoning` from
> (category root ∉ generator-roots) ∧ (cluster ≤ cluster-max) is a *structural* definition of "not a
> near-duplicate generator cluster" — the property we actually care about — and self-maintains as the pool
> grows. The cluster-size backstop is what makes it robust to an incomplete denylist.

### 2.2 `scripts/svcomp_split_eval.py` — add a weighted / grouped score (additive, immutable)

The scorer is immutable and must not change *semantics* of existing fields. Add **new output fields only**,
gated behind a new flag so every current call is byte-identical:

- New flag `--group-weight` (off by default): when set, in addition to the existing `confirmed_score`,
  compute **`confirmed_score_weighted`** = the confirmed score with **per-cluster dedup weighting**: within
  each `(property, cluster)` bucket present in the *evaluated rows*, each confirmed FALSE contributes
  `1 / n_cluster_confirmed_max`… — concretely, cap each cluster's contribution so **one generator family
  cannot contribute more than `--cluster-credit` points** (default `1.0`) to the weighted score. i.e. a
  cluster of 300 confirmed CWE tasks contributes `min(confirmed_in_cluster, cluster_credit·)` — dedup to
  "did this *family* get solved," not "how many near-duplicates." Reasoning tasks are (by construction)
  singleton-ish clusters, so they keep ~full weight.
- New field `confirmed_by_cluster`: `{property: {cluster: confirmed_count}}` — lets the gate and journal see
  *which families* moved, not just the total. This is the observability the current gate lacks.
- The per-property block already carries `per_property[prop].confirmed`; add
  `per_property[prop].confirmed_weighted` and `per_property[prop].confirmed_clusters` (count of distinct
  clusters with ≥1 confirmed FALSE) under the same flag.

Everything above is **strictly additive and flag-gated** → the immutable sha256 of the *behavior on existing
flags* is unchanged for the human-merge full runs, and `verify_immutables` still guards the file byte-for-byte
(the file content does change, so its hash is re-frozen once at rollout — see §4 rollout note).

### 2.3 `scripts/loop/supervisor.sh` — the new gate wiring (this is the heart of the change)

Replace the single TRAIN-sample eval + `confirmed_delta>0` KEEP with a **three-eval, validation-anchored
decision**. All three evals already exist as `saf_eval` calls; we add the `val` one and change what the gate
maximizes.

Per arm, after the worker returns:

```
before_val   = saf_eval(VAL_MANIFEST,   --property FAM  --group-weight)     # generalization baseline
after_val    = saf_eval(VAL_MANIFEST,   --property FAM  --group-weight)     # generalization result
before_train = saf_eval(TRAIN_MANIFEST, --property FAM  --group-weight)     # pool guard (dedup-weighted)
after_train  = saf_eval(TRAIN_MANIFEST, --property FAM  --group-weight)     # pool guard (dedup-weighted)
```

(In practice: cache the two `before_*` from the orient step, as today; a `val` eval is *cheaper* than the
TRAIN sample because `val` excludes the huge Juliet clusters — so this is not a cost regression, it is close
to cost-neutral. `EVAL_SAMPLE` still applies per property on each manifest.)

**New KEEP metric — the "generalization delta":**
```
gen_delta   = after_val.confirmed_score           - before_val.confirmed_score            # raw val recall
gen_delta_w = after_val.confirmed_score_weighted  - before_val.confirmed_score_weighted   # dedup val recall
pool_delta_w= after_train.confirmed_score_weighted - before_train.confirmed_score_weighted # dedup pool guard
```

Decision (replaces `decide(delta>0 → KEEP)`):

- **KEEP** iff `gen_delta_w > 0` **AND** `pool_delta_w ≥ 0` **AND** anti-cheat clean **AND** (for crosscut)
  no family regressed. The arm moved *distinct reasoning families/clusters* on the held-IN validation set
  and did not regress the deduped pool. This is a real generalization gain by construction.
- **KEEP (pool-tie-break)** iff `gen_delta_w == 0` **AND** `pool_delta_w > 0` **AND** `gen_delta ≥ 0` —
  a pool-recall gain that does **not** hurt validation is still worth keeping (it's real confirmed FALSEs,
  sound, +score at the human-merge boundary), but it is recorded distinctly (`KEEP_POOL`) and it does **not**
  reset the lever stall the way a generalization gain does (§2.5). This preserves the honest pool gains SAF
  already gets (memsafety 47%) without letting them *drive* the loop.
- **ACCUMULATE+** iff `gen_delta_w == 0 AND pool_delta_w == 0` AND the arm is `capability`-mode AND it
  compiled + `saf-svcomp` tests pass AND it produced **≥1 newly-confirmed FALSE on `val` OR on the holdout
  probe** that the baseline missed (even if net-zero because it lost one elsewhere — unlikely for additive
  confirmers, but this is the "first novel task solved" milestone credit). This is the plan-204 milestone
  contract, now *anchored to a reasoning task*, not to a unit test the arm writes for itself.
- **ACCUMULATE** (unchanged) iff `delta==0` and the source diff is useful (compiles + tests) — preserves
  scaffolding for the next arm.
- **REVERT** otherwise; **REJECT_TAMPER / REJECT_HOLDOUT** on anti-cheat (first, unchanged).

**Why `pool_delta_w ≥ 0` and not `= 0`:** a generalization gain that *also* raises deduped pool recall is
ideal; we only forbid a `val` gain that comes at the cost of *deduped* pool recall (which would signal the
arm traded a broad family away for a narrow reasoning win — rare, but the guard is free). Note we compare the
**deduped** pool score, so a change that merely shuffles which 3 of 300 near-duplicate CWE tasks confirm does
not register as pool movement at all — exactly the memorization we want to stop rewarding.

### 2.4 `scripts/loop/supervisor.sh` — make the holdout an ACTIONABLE gate (still read-forbidden)

`maybe_heldout_check` currently writes the svcomp26 eval to disk and does nothing with it. Change it to feed
**lever priority**, never per-arm KEEP (the holdout stays a low-frequency, supervisor-only signal so it can't
be gamed and stays statistically meaningful):

- Every `HELDOUT_EVERY_K` kept arms, run the holdout eval **with `--group-weight`** and record, **per lever**,
  the holdout `confirmed_score_weighted` at that checkpoint (a small `STATE_DIR/heldout_trend.<lever>.jsonl`).
- **Actionable rule (the plan-204 "HARD reject for train gains that don't reproduce", finally wired):**
  a lever whose cumulative `val`/pool KEEPs have raised the train score by ≥ `HELDOUT_MIN_LIFT` since its last
  holdout checkpoint but whose **holdout weighted score did not increase** across **two** consecutive holdout
  checkpoints is **PARKED as overfitting** (`lever.<id>.parked` + `ALERT_OVERFIT_<id>`, journalled). Two
  checkpoints, not one, to avoid parking on holdout-eval noise. This is the in-loop overfit brake that is
  currently missing.
- Symmetrically, a lever that produced a **holdout weighted increase** gets its stall counter reset and a
  small **priority boost** (see §2.5) — the loop chases what generalizes.

The holdout is **never** read by the worker (the PreToolUse denylist + `audit_forbidden_reads` are unchanged),
and the *per-arm* KEEP never depends on it (so a worker can't fabricate a holdout number it never runs). The
holdout only ever *reduces* a lever's priority (park on non-reproduction) or *raises* it (boost on
reproduction) — it cannot cause a bad change to be kept.

### 2.5 `scripts/loop/supervisor.sh` — lever selection weighted toward generalization

Two small changes to `pick_lever` / `record_lever_outcome`:

- **Stall reset semantics:** only a `KEEP` (generalization gain) or an `ACCUMULATE+`/holdout-boost resets the
  consecutive-revert stall. A `KEEP_POOL` (pool-only tie-break gain) advances the arm but **does not** reset
  the stall — so a lever that only ever moves Juliet duplicates still eventually parks, freeing budget for
  reasoning levers. (Today any KEEP/ACCUMULATE resets the stall, which lets a pure-memorization lever run
  forever.)
- **Priority nudge:** keep the family round-robin (fairness across properties) but, *within a family*, order
  non-parked levers by a persisted `lever.<id>.gen_credit` (holdout-boosts + generalization KEEPs) instead of
  pure ROI-file order, so a lever that has demonstrably moved novel tasks is tried before one that hasn't.
  Ties fall back to file order (deterministic). This directly counters "capability levers score 0 → park":
  the *first* time a capability lever lands a `val`/holdout confirmation it earns priority and budget instead
  of losing it.

### 2.6 `scripts/loop/levers.tsv` — scope/priority to steer toward general mechanisms

- Re-order so **generalization-bearing crosscut levers lead their families** (they touch shared
  frontend/PTA/AIR/slicing — the machinery that moves *many* reasoning tasks at once): `air-slicing`,
  `symbolic-oracle`, `harness-havoc`, `frontend-unions` move up; pure per-cluster confirmers
  (`exec-validator-gate`, `overflow-recall`) stay but are no longer the default first pick in their family.
- Add a `weight` column (5th→6th; description shifts right) read by `pick_lever` as the initial `gen_credit`
  seed, so the human can prime "this lever builds transferable power." Purely additive; parsing stays
  `cut -f`.
- No new levers invented here (that's the mechanism-study tasks #20–26); this is scope/priority only.

### 2.7 `scripts/loop/arm_prompt.md` — steer the worker toward general mechanisms

Add a short **"YOUR TARGET IS TRANSFERABLE SOLVING POWER"** block (keeps every existing redline verbatim):

- "Your arm is measured on a **held-IN validation set of reasoning-heavy, NON-generator tasks** (loops,
  arrays, floats, recursion, drivers, firmware harnesses — *not* Juliet CWE clusters). Confirming more
  near-duplicate members of one generator family will **not** move your score (the gate deduplicates
  generator clusters). Build a **mechanism** that solves a *class* of tasks: a sounder must-reach, a
  bounded symbolic input oracle feeding replay, backward slicing, harness/havoc synthesis — not a
  pattern that keys on one cluster's shape."
- "**No benchmark-path / function-name / task-id / cluster keying**" (already a redline — reinforce it,
  now with the reason: cluster keying scores zero under dedup and is caught by the holdout).
- Keep the `val.jsonl` path readable to the worker and **point at it**: "You MAY read `val.jsonl` and its
  per-task diagnostics to see which reasoning tasks you miss and why (stderr tails)." The worker steering
  toward the *validation* misses is exactly the generalization pressure we want — and it's sound because
  `val` ⊂ TRAIN (never the holdout).

### 2.8 `scripts/loop/lib/gates.py` + `verify_arm.py` — the new decision, unit-tested

`gates.py` gets new **pure functions** (keeps the "no Claude/network/Docker, unit-tested" property):

```python
def generalization_delta(before_val, after_val) -> tuple[int,int]:
    """(gen_delta, gen_delta_w) from val evals' confirmed_score / confirmed_score_weighted."""

def pool_guard_delta_w(before_train, after_train) -> int:
    """dedup-weighted pool delta; the no-regression guard (must be >= 0)."""

def novel_confirm_gain(before_val, after_val, holdout_probe=None) -> int:
    """count of (property,cluster) buckets that went 0 -> >=1 confirmed FALSE on val
       (or holdout probe): the ACCUMULATE+ 'first novel task solved' milestone."""

def decide_v2(*, immutable_violations, forbidden_reads, gen_delta_w, gen_delta,
              pool_delta_w, capability, progressed, novel_gain,
              family_regressed=False) -> str:
    # REJECT_TAMPER / REJECT_HOLDOUT first (unchanged), then:
    #   family_regressed                                   -> REVERT
    #   gen_delta_w > 0  and pool_delta_w >= 0             -> KEEP
    #   gen_delta_w == 0 and pool_delta_w > 0 and gen_delta >= 0 -> KEEP_POOL
    #   gen_delta_w == 0 and pool_delta_w == 0 and capability
    #        and progressed and novel_gain > 0             -> ACCUMULATE_PLUS
    #   delta == 0 and progressed                          -> ACCUMULATE
    #   otherwise                                          -> REVERT
```

`verify_arm.py` gains `--before-val/--after-val` (+ the existing `--before/--after` become the *train*
guard), `--capability 0|1`, and emits the decision word `KEEP | KEEP_POOL | ACCUMULATE_PLUS | ACCUMULATE |
REVERT | REJECT_*`. `soundness_ok` stays informational (FP=0/wrong-TRUE=0 read from the *train* eval — the
full reservoir soundness). The exit-code contract widens: `0` for any keep-ish decision
(`KEEP|KEEP_POOL|ACCUMULATE_PLUS|ACCUMULATE`).

The supervisor maps the decision to branch actions:
`KEEP|KEEP_POOL` → `keep_arm` (advance work branch); `ACCUMULATE_PLUS|ACCUMULATE` → `keep_arm` with the
score-neutral tag; `REVERT`/`REJECT_*` unchanged.

---

## 3. The new metric, in one place

| Signal | Manifest | Weighting | Used for |
|---|---|---|---|
| **gen_delta_w** (primary KEEP driver) | `val` (non-generator TRAIN) | per-cluster dedup | KEEP: real generalization gain |
| gen_delta (raw) | `val` | none | tie-break / ACCUMULATE+ sanity |
| **pool_delta_w** (guard) | `train` (full pool) | per-cluster dedup | no-regression guard; `KEEP_POOL` tie-break |
| false_alarms / wrong_true | `train` (full pool) | n/a | soundness (intrinsic to score; reported) |
| **holdout weighted trend** (per lever) | `holdout` (svcomp26-new) | per-cluster dedup | PARK overfit levers / BOOST generalizers |

The thing the loop **maximizes** moves from "confirmed FALSEs on a Juliet-dominated TRAIN sample" to
"distinct reasoning families/clusters confirmed on a held-IN validation set, that also reproduce on the
frozen new-edition holdout." Pool recall is demoted from *objective* to *guardrail + tie-break*.

---

## 4. Anti-overfit guardrails preserved (and strengthened)

Nothing here weakens a soundness or anti-cheat control; several are strengthened:

1. **svcomp26 holdout stays a READ-FORBIDDEN true holdout.** Unchanged: PreToolUse denylist +
   `audit_forbidden_reads` on `svcomp-splits/holdout`. The new actionable use is **supervisor-side only**
   (park/boost a lever); the worker still never reads it and the per-arm KEEP never depends on it. It can
   only *lower* a lever's standing or *raise* it — never keep a bad change.
2. **`val` comes from TRAIN, never the holdout.** `val.jsonl` ⊂ `train.jsonl` by construction (§2.1); the
   splitter derives it from the train side only. It is freely readable (it's TRAIN) — no new leak.
3. **Immutables extended.** Add `tests/benchmarks/svcomp-splits/val.jsonl` and the generator-roots /
   `cluster-max` config (recorded in `split_manifest.json`) to `IMMUTABLE_GLOBS` + the frozen manifest, so a
   worker cannot redefine `val` to re-admit Juliet or shrink the reasoning set. `svcomp_split.py` and
   `svcomp_split_eval.py` are already immutable; their **new content is re-frozen once at rollout** (the
   freeze is a supervisor action from the clean baseline, §2.2/§2.3 are additive/flag-gated so the
   human-merge full-run behavior is unchanged).
4. **Soundness is still intrinsic to the score and read on the FULL reservoir.** FP=0 / wrong-TRUE=0 come
   from the `train` (full-pool) eval, per plan-204 §A2's "gate (c) on the FULL reservoir, never a subsample"
   — the `val`/pool split changes what we *maximize*, not the soundness read. The human-merge boundary
   full-reservoir FP=0 redline is untouched.
5. **Dedup weighting itself is an anti-memorization control:** the deduped pool guard means "confirm 3 more
   of 300 near-duplicates" registers as **zero** movement, so an arm cannot even *tie-break* on
   memorization — it must move a *new* family.
6. **Determinism preserved end-to-end:** `val` membership, cluster weights, and every new score are pure
   functions of (sorted sample, fixed config). No `hash()`, no wall-clock. Byte-identical for identical
   inputs — the NFR-DET invariant and the plan-203 determinism fixes hold.

---

## 5. Expected effect on holdout generalization

- **Capability levers stop getting parked for the wrong reason.** `symbolic-oracle`, `air-slicing`,
  `harness-havoc`, `conc-shim-m1` are now measured on the reasoning surface they target, and the *first*
  novel confirmation earns priority + budget (§2.5) instead of triggering the park countdown. This is the
  single biggest expected lift: the loop finally *invests* in the mechanisms that move new-2026 tasks
  (per plan-204 §A4 the new-2026 hardness is structural — harness/havoc/slicing — exactly these levers).
- **Pure-memorization arms stop driving the loop.** Confirming more Juliet duplicates yields `gen_delta_w=0`
  and (deduped) `pool_delta_w≈0` → at best `KEEP_POOL` (kept for the honest points, but no stall reset), so
  a memorization lever parks and frees budget. Train↑/holdout↔ patterns get the lever PARKED as overfit.
- **The holdout number should start moving** because the objective is now literally "move reasoning tasks
  that reproduce on new tasks," and the loop deprioritizes anything that doesn't. Realistic near-term target:
  new-task unreach-call recall off the **0%** floor (any transferable must-reach/harness mechanism that lands
  even a handful of the ~11.7k new unreach tasks is a large *relative* holdout gain), and memsafety/overflow
  holdout recall rising toward their train-sample rates as the deduped objective forces family-general
  confirmers.
- **No expected regression at the human-merge boundary:** `KEEP_POOL` still preserves every sound pool gain
  SAF already gets (the full-reservoir FP=0 confirmed C.FalseOverall ≈ 3960 from plan-203 is protected by the
  `pool_delta_w ≥ 0` guard and the unchanged soundness read).

---

## 6. Risks & mitigations

| Risk | Mitigation |
|---|---|
| **`val` too small per property** (esp. unreach after removing Juliet) → noisy gen_delta | `val` still has the large *reasoning* pools (loops/arrays/floats/drivers/ldv/firmware = thousands of tasks); report per-property `val` counts in `split_manifest.json` at rollout and tune `cluster-max` if a property comes out thin. `EVAL_SAMPLE` stratifies as today. If a property's `val` is genuinely tiny, fall back to `gen_delta` (raw) for that property and lean on the holdout trend. |
| **Generator denylist is incomplete** → a generated family leaks into `val` | The **cluster-size backstop** (`cluster > cluster-max` ⇒ generator) catches unlabeled generators automatically; and the fail-safe direction excludes-from-`val` when unsure, so `val` only gets cleaner. |
| **Dedup cap mis-set** → over/under-credits families | `--cluster-credit` default 1.0 (solved-or-not per family) is the conservative anti-memorization choice; expose it and record it in the manifest. Reasoning tasks are singleton clusters so they're unaffected by the cap. |
| **Holdout-eval noise** parks a good lever | Require **two consecutive** non-reproducing checkpoints before PARK-overfit (§2.4); the park is reversible by the human (it's a scratch-branch lever file). |
| **Cost**: three evals/arm instead of one | `val` excludes the huge Juliet clusters ⇒ the `val` eval is *cheaper* than the current TRAIN sample; `before_*` are cached from orient. Net ≈ cost-neutral, and the deduped pool guard reuses the same family eval a crosscut arm already runs all-property. |
| **Immutable re-freeze at rollout** could mask a bad edit | The re-freeze is a one-time supervisor action from a **clean, reviewed** baseline (the additive/flag-gated changes are diff-reviewed by a human before freezing); after that, `verify_immutables` guards byte-for-byte as today. |
| **Worker games `val`** by cluster-keying reasoning tasks | Reasoning clusters are near-singletons so keying one buys ~0; the **holdout trend** catches any keyed gain that doesn't reproduce; the `no-*-keying` redline + adversarial diff review remain. |

---

## 7. Rollout (staged, reversible — matches plan-204 discipline)

1. **Split + scorer (offline, deterministic):** implement §2.1/§2.2 additively; unit-test the pure split
   classifier (a generated cluster → `generator`; a loops task → `reasoning`; oversized cluster backstop;
   determinism) and the pure scorer weighting (`confirmed_score_weighted`, cluster cap). Regenerate the
   split on the VM; **eyeball `split_manifest.json`** per-property `val` counts before wiring the gate.
2. **Gate logic (pure, unit-tested):** implement §2.8 `decide_v2` + helpers in `gates.py` with tests
   mirroring the current gate tests (inject fake val/train evals → assert KEEP/KEEP_POOL/ACCUMULATE_PLUS/
   REVERT; inject an immutable edit → REJECT_TAMPER; a holdout-path read → REJECT_HOLDOUT).
3. **Supervisor wiring (§2.3–2.6) behind a flag** `LOOP_GEN_MODE=1` (default on for the loop, off preserves
   the plan-204 gate for A/B) — so the change is reversible and comparable.
4. **Freeze immutables** (`val.jsonl` + config) from the clean baseline; **Stage-1 watched dry-run** of one
   arm on a capability lever (e.g. `symbolic-oracle`) → confirm it is now measured on `val`, that a
   reasoning confirmation KEEPs, and that a Juliet-only change gets `KEEP_POOL`/parks.
5. **Stage-2 supervised** short run; watch `heldout_trend.<lever>.jsonl` move; then Stage-3 autonomous.

---

## 8. What this explicitly does NOT change (guardrails intact)

- The two anti-cheat gates (immutable sha256; holdout-not-read) — byte-identical.
- Soundness = FP=0 / wrong-TRUE=0 on the FULL reservoir, intrinsic to the score.
- Determinism (BTree/sorted/stable-hash), Docker-only builds/evals on the VM, no push / no auto-merge to
  `svcomp`, REQ-IP-001 independent implementations.
- The worker never runs the scorer, never reads the holdout, self-report never trusted.
- `train.jsonl` / `holdout.jsonl` semantics — unchanged; `val.jsonl` is purely additive.
