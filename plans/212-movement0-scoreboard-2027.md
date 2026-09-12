# Plan 212 — Movement 0: score against the 2027 rulebook (defends 36 weighted)

**Status:** DESIGNED, not started. Awaiting sign-off together with [`plans/211`](211-saf-native-prover.md) §5.3.
**Branch:** cut `movement0/scoreboard-2027` off `lever1/cbmc-loop-free` @ the plan-211 design commit.
**Track:** TRUE-side. Movement 0 of 4 — **nothing else in the sequence may start before this lands.**
**Design source:** `plans/211` §5.1(d), §5.2, §5.3. Memory: `svcomp-2027-witness-rules-50pts-at-risk`.

---

## 0. Goal

Make SAF's reported score mean what it says under the **SV-COMP 2027** rules, then win
back the points that re-derivation exposes.

Two independent deliverables:
- **0A (defensive, must-do):** re-derive the harness's witness-requirement rule per BASE
  CATEGORY. This will make the score **fall** — expected ~151 → ~115. That is the point:
  every A/B in the loop campaign is currently steering on a number ~36 too high.
- **0B (offensive):** emit a 2.1 correctness witness on the termination path so the 36
  weighted points come back legitimately.

**Do 0A first and land it on its own.** If 0B lands first the drop is masked and we never
learn whether the rule reading was right.

## 1. The rule, and where it came from

Authoritative source: the published 2027 rules page (`sv-comp.sosy-lab.org/2027/rules.php`,
read 2026-09-12). The requirement is per **base category** `C.<property>.<suffix>`, where
the suffix is the benchmark family — i.e. the `.set` file the task belongs to.

| base category | correctness witness | TRUE on verdict alone? |
|---|---|---|
| `C.unreach-call.{Arrays,Heap}` | not supported | **YES** |
| `C.unreach-call.Floats` | 2.0+ (demo mode) | **YES** |
| `C.unreach-call.Concurrency` | 2.1+ | no — new in 2027 |
| `C.unreach-call.<all others>` | 2.0+ | no |
| `C.valid-memsafety.<any suffix>` | not supported | **YES** |
| `C.valid-memcleanup.all` | not supported | **YES** |
| `C.no-overflow.Concurrency` | 2.1+ | no — new in 2027 |
| `C.no-overflow.<all others>` | 2.0+ | no |
| `C.no-data-race.all` | not supported | **YES** |
| **`C.termination.all`** | **2.1+** | **no — NEW in 2027** |

**⚠️ METHOD WARNING.** Do NOT re-derive this from `benchmark-defs/category-structure.yml`
or from each validator's `<rundefinition>`s. That method gives the WRONG answer for
`no-data-race`: five validators declare a `SV-COMP27_no-data-race` correctness
rundefinition, yet the rules table says "not supported", so it is verdict-only. Read the
rules page.

Two more facts from the same page that shape later movements: the correctness-witness
validator budget was **cut from 900 s to 300 s**, and an unconfirmed-but-correct TRUE
scores **0, never −32** — a weak witness costs points but is not a soundness hazard.

## 2. Slice 0A — re-derive the harness rule

`scripts/svcomp_split_eval.py:65` currently has:
```python
TRUE_WITNESS_NOT_REQUIRED = {"termination", "valid-memsafety", "valid-memcleanup", "no-data-race"}
```
That is the 2026 rule. Replace the flat property set with a `(property, suffix)` decision:

1. **Teach the harness `.set` membership.** `scripts/p211_headroom.py` already has the
   loader (glob every `c/*.set`, expand the patterns, map `rel_yml -> {suffix}`). Lift it
   into the harness, built once per run and cached — it is ~50k globs, not free per task.
2. `true_witness_required(property, suffixes) -> bool` implementing the §1 table. Keep the
   table as a literal dict with the rules-page URL and the read-date in a comment, so the
   next edition is a one-line diff.
3. Thread it through `confirmed_score` and `witness_validator_for` — both currently branch
   on `prop in TRUE_WITNESS_NOT_REQUIRED`.
4. A task in no `.set` file (the `/todo` directories — 325 of them) is **not run by
   SV-COMP at all**. Decide explicitly: either drop them from the manifest or keep them
   and exclude from the weighted total. Do not leave it implicit.

**Merge gate.** Re-run the authoritative 55,690-task eval. Expect:
- `confirmed_score_weighted` **151 → ~115**: termination loses 36 (SAF emits no witness),
  partly offset by the new `C.unreach-call.{Arrays,Heap,Floats}` carve-out.
- `false_alarms == 0`, `wrong_true == 0` unchanged — this slice cannot affect verdicts.
- Publish the corrected number and update every memory that quotes 151.

**Kill criterion.** If the score does **not** fall by roughly 36, the rule reading is
wrong. Stop; re-read the rules page and the 2027 report before touching anything else.

## 3. Slice 0B — a 2.1 correctness witness for termination

SAF emits nothing today: `termination_strategy`
(`crates/saf-cli/src/commands.rs:5529-5535`) returns `correctness: None`.

The good news is that the hard part is already computed and thrown away:

1. **Surface the ranking function.** `ranking.rs` synthesises linear ranking-function
   coefficients via Farkas and discards them — `loops_are_ranked` returns `bool`. Change
   it to return `Option<Vec<LoopRanking>>` carrying the coefficients it already has.
2. **Parameterise the format version.** `correctness_witness.rs` hardcodes
   `format_version: '2.0'`; 2.1 is required for termination.
3. **Render the transition invariant.** One entry per ranked loop, `\at(L, AnyPrev) > L`
   at the loop's controlling expression, per the 2.1 `terminating-program-example` in the
   sv-witnesses repo. Note `program_structurally_terminates` proves via a loop-free
   reachable CFG plus an acyclic call graph, so for those tasks there is no loop to rank
   and the entry set may be empty — check the schema permits it (`content` has no
   `minItems`) and confirm an empty `invariant_set` is accepted for this property.
4. **Wire** `termination_strategy` to return `correctness: Some(..)`.

**Validate with witnesslint plus a real 2.1-capable validator.** NOT the bundled
CPAchecker 4.2.2 — it rejects format 2.1 outright. Candidates from the 2027 definitions:
`uautomizer-`, `goblint-`, `mopsa-`, `theta-validate-correctness-witnesses-v2`. Getting
one of these provisioned is part of this slice and is a prerequisite for Movement 3 too.

**Merge gate.** All currently-proven termination tasks emit a witnesslint-clean 2.1
witness; the weighted score returns toward 151; `wrong_true == 0`.

## 4. Risks

- **Deadline.** A field agent reported SV-COMP 2027 registration 2026-10-08 and submission
  2026-10-20, but `2027/dates.php` returns 404, so this is **UNVERIFIED**. Confirm it
  first: if it holds, Movement 0 is the only movement that fits before the gate, and 0B
  matters more than anything offensive.
- **The rules page is an early draft.** It carries an editorial note and has `??`
  placeholders for Java/Python/MoXI. The C rows used here are filled in, but re-check
  before submission.
- **A second validator is a new dependency.** It must be bundled or provisioned the same
  way CBMC and CPAchecker are — and per `saf-lever1-cbmc-loop-free`, `.svtools/` is
  gitignored and version-skewed today. Do not repeat that.

## 5. Definition of done

- [ ] Harness decides the witness requirement per `(property, .set suffix)` from a literal
      table citing the rules page and read-date.
- [ ] `/todo` tasks handled explicitly.
- [ ] Authoritative re-run published; the ~36 drop observed and explained; every memory
      and plan quoting 151 updated.
- [ ] Termination emits a witnesslint-clean 2.1 correctness witness.
- [ ] At least one non-CPAchecker 2.1-capable correctness validator provisioned and
      confirming SAF's termination witnesses.
- [ ] `make lint` clean, TDD-green, `false_alarms == 0`, `wrong_true == 0`.
