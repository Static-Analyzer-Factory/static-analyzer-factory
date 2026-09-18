# Plan 218 — `unreach-call`: the largest unknown

**Status:** QUEUED, not started. Scoped 2026-09-17 after Movement 2 and M4 landed.
**Branch:** cut off `svcomp` (the competition mainline).
**Track:** both sides. This is the property every other movement has routed around.
**Design source:** measured below. Supersedes the `unreach-call` framing in
`plans/211` §5.6 and `plans/213` §4-§5, both of which were written while mechanism
M4 was open and the `refine_eq_false` wall was live.

---

## 0. Why this, and why now

`unreach-call` is **48 of SAF's 142 dedup-weighted points — 34% of the score — and
the only property no movement has touched.** Every figure quoted since Movement 0
holds it fixed at 48 and splices the other four properties around it. It is
simultaneously the largest contributor and the least understood.

Measured over the 55,690-task baseline (`saf-alone-20260913`):

| | |
|---|---:|
| tasks | 22,631 |
| weighted | **48** |
| scoring tasks | 740, in exactly **48 clusters** (dedup cap 1 each) |
| `FalseCorrect` | 1,208 |
| `Unknown` | **21,423 (94.7%)** |
| **TRUE-side scoring** | **0** |

Two facts should shape everything that follows.

**Every one of the 48 points is FALSE-side.** The TRUE arm contributes exactly
nothing, because `try_unreach_true` is a compile-time stub:
`fn try_unreach_true(_ctx: &VerifyCtx) -> Option<VerdictOutcome> { None }`.

**94.7% of the property ends `Unknown`**, and `plans/213` measured where the time
goes: `unreach-call` is **75.2% of the entire 55,690-task run's CPU** at a 32.8 s
mean, and **96.4% of that CPU buys an `Unknown`**. This is where SAF spends almost
all of its compute and earns almost none of its score.

## 1. What is stale, and why the old conclusions cannot be trusted

`plans/213` §4 concluded that leaving the TRUE arm disabled "costs nothing
measurable": the 3 `unreach-call` TRUEs it found landed in clusters already at the
dedup cap, so marginal contribution 0. That conclusion was **argued, not
measured**, and it was argued under conditions that no longer hold:

* **mechanism M4 was still open** — a live wrong TRUE in the shared absint. Fixed
  2026-09-17 (`7d425c6c`).
* **`refine_eq_false` was broken** — SAF could not prove a 3-line program. Fixed in
  Movement 1.
* `plans/213` §5 then re-measured and found the predicted histogram shift **did not
  materialise**: PROVE 3 → 5 tasks, clusters 3 → 4, newly bankable **0**,
  `ABSTAIN:error-reachable` 412 → 417. Its own instruction was "scope the next plan
  on this histogram, not on a hoped-for shift."

So the honest position is: the TRUE arm's value is **unknown**, not zero, and the
last attempt to estimate it came back at zero for reasons that have since changed.

## 2. Shape of the opportunity

Upside is **clusters**, not tasks. 740 tasks already score and produce 48 points
because the dedup cap is 1 per cluster. Concentration in the top clusters is heavy —
`product-lines` 227, `aws-c-common` 84, `array-fpi` 66, `neural-networks` 36,
`nla-digbench-scaling` 35, `recursive-simple` 34 — so **more tasks in those clusters
are worth nothing.** Every point must come from a cluster that currently scores
zero.

That reframes the whole property: the target is the 21,423 `Unknown` tasks, filtered
to clusters with no scoring task at all.

## 3. Staged, with a kill criterion at every stage

The full property is **~17 h** (22,631 tasks at a 32.8 s mean). Do not start there.

**Stage A — measure what we have (no code).** Re-run a stride sample and produce the
per-cluster ledger the property has never had: which clusters score, which are
entirely `Unknown`, and what the dominant abstain reason is in each zero-scoring
cluster. `scripts/m2_ab.py` and the funnel instruments already exist.
→ **KILL if** the zero-scoring clusters are dominated by reasons no reachable
increment addresses (indirect calls, unsupported frontend constructs, timeouts).
That answer is worth having and costs a day.

**Stage B — decide the TRUE arm on evidence.** Re-run the `prove_unreachable`
sentinel over expected-TRUE tasks now that M4 and `refine_eq_false` are fixed, and
count **newly bankable clusters** — clusters that would gain a point, not tasks.
`plans/213` §5's answer was 0; the inputs have changed.
→ **KILL if** newly bankable is still 0. Then delete the stub and its plumbing
rather than leaving dead code that looks like a feature, and say so in `plans/213`.

**Stage C — the FALSE side**, which is where all 48 points already come from and
where the 21,423 `Unknown` tasks are. Scope only after Stage A names the reasons.

## 4. Cost discipline, learned the hard way

* **Never run the full 55,690 gate for this.** Property-scoped runs spliced into the
  baseline are a quarter of the cost for the same evidence
  (`scripts/m4_splice.py`, `scripts/m2_ab.py`).
* **A 200-task sample is not a measurement.** It showed 8 → 8 for the §8 net while
  the full 9,063-task gate showed −1 weighted and −27 TRUEs.
* **Do not rebuild `target/release/saf` while a gate is running.** One run was
  discarded for exactly this; check the binary's mtime against the gate's start.
* `rsync -a` preserves mtimes, so cargo can miss a synced change — `touch` after
  syncing or a "revert and re-test" will silently test the unreverted binary.

## 5. What this plan is NOT

Not a request to re-enable `try_unreach_true` on principle. `plans/213` §4 may well
be right that it is worth 0 — the point of Stage B is to find out on current
evidence and then either bank it or **delete it**, instead of carrying a stub that
reads like an unfinished feature.

Not a precision project. The 9× funnel gap (`unreach-call` PROVE 2/150 vs
`no-overflow` 18/150) is a symptom; Stage A decides whether it is the binding
constraint or whether the binding constraint is reachability/frontend coverage.
