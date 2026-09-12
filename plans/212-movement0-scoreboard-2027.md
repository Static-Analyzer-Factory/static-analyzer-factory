# Plan 212 — Movement 0: score against the 2027 rulebook (defends 36 weighted)

**Status:** **0A LANDED 2026-09-12.** 0B designed, not started.
**Branch:** `movement0/scoreboard-2027`, cut off `lever1/cbmc-loop-free` @ `038e4122`.
**Track:** TRUE-side. Movement 0 of 4 — **nothing else in the sequence may start before this lands.**
**Design source:** `plans/211` §5.1(d), §5.2, §5.3. Memory: `svcomp-2027-witness-rules-50pts-at-risk`.

---

## MEASURED RESULT — slice 0A, 2026-09-12

Dedup-weighted over the authoritative 55,690-task dump. `false_alarms == 0` and
`wrong_true == 0` throughout, verified on every row — a witness rule gates only
POSITIVE credit, and `confirmed_score`'s penalty tail is rule-independent.

| scoring rule | weighted | ndr | nov | term | unreach | mem |
|---|---:|---:|---:|---:|---:|---:|
| 2026 rule (the old scoreboard) | **151** | 14 | 34 | 36 | 52 | 15 |
| 2027, TRUE side only | 115 | 14 | 34 | **0** | 52 | 15 |
| **2027 full table, version-blind — the published number** | **110** | 9 | 34 | 0 | 52 | 15 |
| 2027 full table, version-aware | 107 | 9 | 34 | 0 | 49 | 15 |
| 2027 full, restricted to tasks SV-COMP actually runs | 108 | | | | | |

**Kill criterion PASSED, in the strong sense.** The TRUE-side-only switch costs exactly
**−36** — an integer, not "roughly" — so the rules reading is right. Landing number
**110** (−41). Artifacts: `m0a-2027-rescore.json`, `m0a-2027-pertask.jsonl`, regenerate
with `python3 scripts/m0_rescore.py`.

**Restate the gate as TWO assertions** — as written below it would fire on a correct
implementation, because the full table costs 41, not 36:
- (a) *rules-reading check* — switch only the TRUE side: exactly `151 → 115`. Any other
  number means the correctness column was misread.
- (b) *landing number* — full table: exactly `151 → 110`. A fall of exactly 36 **here**
  means the `no-data-race` violation requirement was missed.

`plans/211` §5.6's "roughly 50" threshold is stale and contradicts its own §5.3; it would
have killed the plan on a passing result.

Three §1–§2 claims corrected in place below. §3's four claims were **refuted by
measurement** — see the `0B — MEASURED CORRECTIONS` block.

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
read 2026-09-12). The requirement is per **base category** `C.<property>.<suffix>`.

> **CORRECTION 1 — the suffix is NOT the `.set` basename.** It is the 2027 base-category
> name, and the two differ: `LinkedLists.set` sits inside `C.unreach-call.Heap` (there is
> no `C.unreach-call.LinkedLists`), `termination` uses `MainControlFlow` / `MainHeap` /
> `Other`, and `no-overflow` has a `Main` catch-all. The same `.set` maps to different
> suffixes per property — `Heap.set` is `unreach-call.Heap`, `no-overflow.Main`,
> `termination.Other`, `valid-memsafety.Heap`, `valid-memcleanup.Main`. The map is now in
> `scripts/svcomp_witness_rules.py` §1, from the SV-COMP 2027 bench-defs
> (`gitlab.com/sosy-lab/sv-comp/bench-defs`), kept deliberately separate from the
> requirement table in §2, which comes only from the rules page.
>
> **CORRECTION 2 — the table below omits the VIOLATION column, and it moves points.**
> `C.no-data-race.*` violation is `2.2` (it was already `1.0` in 2026), i.e. REQUIRED,
> while the harness exempted it outright. Worth **−5**. Also: `C.valid-memsafety.
> {Concurrency,Huawei*}` violation is `2.2 (demo mode)` = free, the footnote `#` makes
> `valid-memtrack` violations free (no format covers them), and `C.valid-memcleanup.all`
> is free on BOTH columns — the only such property in 2027.
>
> **CORRECTION 3 — the `Huawei*` rows are missing.** `C.unreach-call.Huawei*` and
> `C.no-overflow.Huawei*` are `2.1+ (demo mode)` on correctness and `2.2 (demo mode)` on
> violation — all witness-free. Separately, every `C.*.Huawei-Concurrency-Challenges` is
> in `demo_categories`, so its score does not count toward Overall at all.

| base category | correctness witness | TRUE on verdict alone? |
|---|---|---|
| `C.unreach-call.{Arrays,Heap}` | not supported | **YES** |
| `C.unreach-call.Floats` | 2.0+ (demo mode) | **YES** |
| `C.unreach-call.Concurrency` | 2.1+ | no — new in 2027 |
| `C.unreach-call.Huawei*` | 2.1+ (demo mode) | **YES** |
| `C.unreach-call.<all others>` | 2.0+ | no |
| `C.valid-memsafety.<any suffix>` | not supported | **YES** |
| `C.valid-memcleanup.all` | not supported | **YES** |
| `C.no-overflow.Concurrency` | 2.1+ | no — new in 2027 |
| `C.no-overflow.Huawei*` | 2.1+ (demo mode) | **YES** |
| `C.no-overflow.<all others>` | 2.0+ | no |
| `C.no-data-race.all` | not supported | **YES** |
| **`C.termination.all`** | **2.1+** | **no — NEW in 2027** |

The violation column, which the original table omitted entirely:

| base category | violation witness | FALSE on verdict alone? |
|---|---|---|
| `C.unreach-call.Concurrency`, `C.no-overflow.Concurrency` | 2.2 | no |
| `C.unreach-call.Huawei*`, `C.no-overflow.Huawei*` | 2.2 (demo mode) | **YES** |
| `C.unreach-call.<others>`, `C.no-overflow.<others>` | 2.0+ | no |
| `C.valid-memsafety.{Concurrency,Huawei*}` | 2.2# (demo mode) | **YES** |
| `C.valid-memsafety.<all others>` | 2.0+ # | no |
| `C.valid-memsafety.*` where the violated subproperty is `valid-memtrack` | (no format exists) | **YES** |
| `C.valid-memcleanup.all` | not supported | **YES** |
| **`C.no-data-race.all`** | **2.2** | **no — the harness gave this away free** |
| `C.termination.all` | 2.1+ | no (inert: SAF emits no termination FALSE) |

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

**Merge gate — MET.** A full re-run turned out not to be required: the 2027 change is a
pure scoring-function change that alters no SAF behaviour and no verdict, and the dominant
term is deterministic rather than sampled — all 806 termination TrueCorrect rows carry
`witness: null` because `termination_strategy` returns `correctness: None`, so they are
guaranteed `NO_WITNESS → 0` under any re-run. Re-scoring the existing dump through the
harness's own `confirmed_score` / `weighted_confirmed_summary` is therefore exact, and was
cross-checked against two independent reimplementations that agreed on every rung.

A full 24 h re-run is still wanted before any number is defended publicly as "SAF at
HEAD" — not for this slice, but because it is the only thing that can pick up clusters
newly solved since 2026-09-11 (notably by `d2ee9313`, the 19 GB → 244 MB OOM fix). It was
deliberately NOT started here: it occupies the VM for a day, and with the 2026-10-20 tool
deadline 38 days out, 0B outranks it.

> **CORRECTION 4 — the offset is +0, so the stated mechanism is wrong even though the
> number is right.** "Termination loses 36, partly offset by the
> `C.unreach-call.{Arrays,Heap,Floats}` carve-out" only works if termination's gross loss
> exceeded 36. Measured: the gross loss IS exactly 36 (all of termination's points), and
> applying the carve-outs alone moves the score `151 → 151`. SAF emits 3 unreach-call
> TRUEs in 22,631 tasks; two are in `ldv-regression`, whose cluster already scores from
> the FALSE side and is capped at 1, and the third is in `BitVectors`, which is not carved
> out. Anyone re-deriving from this prose will mis-predict.

**Kill criterion — see the restatement at the top of this file.** As written ("if the
score does not fall by roughly 36") it fires on a CORRECT implementation of the full
table, which falls by 41. Use the two-assertion form instead.

## 3. Slice 0B — a 2.1 correctness witness for termination

SAF emits nothing today: `termination_strategy`
(`crates/saf-cli/src/commands.rs:5529-5535`) returns `correctness: None`.

> ## 0B — MEASURED CORRECTIONS (scouted 2026-09-12, before any code)
>
> Four of this section's claims were checked empirically. Two hold, two do not.
>
> **(i) "ranking.rs synthesises coefficients via Farkas and discards them, so
> `loops_are_ranked` need only return them instead" — PARTLY TRUE, materially
> optimistic.** The Farkas system is fully built and Z3 answers `Sat`, but the code
> never asks for a model: both kernels end in `matches!(solver.check(), SatResult::Sat)`
> (`ranking.rs:1898`, `:3897`) — zero occurrences of `get_model`/`eval`/`as_i64` in 8,154
> lines. Extraction is genuinely additive (~10 lines per kernel; the coefficient AST
> handles are already in a `BTreeMap<ValueId, z3::ast::Int>` at `:1796-1801`, `:3809-3815`;
> z3 0.19.7 has the API). But the *structure* is discarded too, not just the coefficients:
> `greedy_lex_subset` knows the round order and returns `bool`, and
> `disjunctive_scc_rank` produces a set of tuples over branch SCCs that is **not
> expressible as a single 2.1 transition invariant** — that path must abstain. The
> plumbing spans ~11 functions, not 2. Estimate: 9 files, ~750 lines.
>
> **(ii) "correctness_witness.rs hardcodes `format_version: '2.0'`; parameterise it" —
> CORRECTED.** The real constant is `crates/saf-svcomp/src/witness_yaml.rs:31`;
> `correctness_witness.rs:312` is a test-expectation string.
>
> **(iii) "the transition invariant is `\at(L, AnyPrev) > L`" — CONFIRMED, with the
> shape pinned.** A termination correctness witness is an ordinary
> `entry_type: invariant_set` whose `content` carries invariants of
> `type: loop_transition_invariant`, `format: ext_c_expression`. There is no
> `ranking_function` key anywhere in the schema — the ranking argument is *encoded into*
> a transition invariant. So `InvariantSetWitness` is the right vehicle; no new entry
> type. The spec lives at GitLab `sosy-lab/benchmarking/sv-witnesses` (the GitHub mirror
> is frozen at format 0.1 and is useless), **and is already vendored on the VM at
> `.svwitnesses` (commit 5297b58, witnesslint 2.1.3-dev)** — nothing to download.
>
> **(iv) "for `program_structurally_terminates` tasks the entry set may be empty, and the
> schema permits it" — SCHEMA-VALID BUT REFUTED IN PRACTICE.** `content: []` inside one
> entry is indeed schema-legal (no `minItems` anywhere), though a zero-ENTRY document is
> rejected by witnesslint. It does not matter: fed to a validator, an empty invariant set
> returns **UNKNOWN, not TRUE** — so it confirms nothing and scores 0, exactly as
> `correctness: None` does today. The loop-free escape hatch is closed.
>
> **(v) BONUS — the validator premise below is FALSE, and 0B is not blocked.**
> CPAchecker 4.2.2 **accepts and meaningfully validates** format-2.1 termination
> correctness witnesses. Demonstrated on the VM against the vendored reference pair: the
> correct witness → `Verification result: TRUE`; the same witness with the invariant
> reversed → `UNKNOWN`; the same witness relabelled `format_version: 2.0` → hard parse
> error. So it discriminates, and it is a real 2.1 parser. A second, non-CPAchecker
> backend is still wanted for independence (Mopsa `svcomp26`, 57 MB, Zenodo
> 10.5281/zenodo.17696794 — its transition-invariant support is UNVERIFIED; fallback
> MetaVal, Apache-2.0) but it is no longer a prerequisite. **Strike "NOT the bundled
> CPAchecker 4.2.2 — it rejects format 2.1 outright" from the paragraph below.**
> Note also a live infrastructure bug: the bundled z3 is missing its exec bit, so
> CPAchecker's SMT backend cannot start while `bin/cpachecker --version` still passes —
> `chmod -R +X` the provisioned tree and add a solver-exercising smoke test, or every
> validation measurement is untrustworthy in both directions.
>
> **Where the 36 points actually are.** The 806 termination TRUEs span exactly 36
> clusters, split by which proof artefact is available inside each:
>
> | | clusters | recoverable how |
> |---|---:|---|
> | ranked-loop present | **22** | a real `loop_transition_invariant` — needs (i) |
> | loop-free only | 7 | **blocked by (iv)** — an empty witness confirms nothing |
> | mixed loop-free + ranked | 4 | the ranked task carries the cluster |
> | recursion-only, no loop anywhere | **3** | no loop to hang an invariant on; needs function contracts |
>
> So the confident recovery is **22–26 of 36**, not 36. `recursive-simple` (58 tasks),
> `recursive` (10) and `termination-memory-linkedlists` (1) are proven only by
> `recursion_is_ranked` / `mutual_recursion_is_ranked`; 2.1 added `function_contract`
> entries, which is the lead to follow for those.

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
witness; the weighted score returns from 110 toward 132-136 (22-26 of the 36 are
confidently recoverable — see the correction block above); `wrong_true == 0`.

## 4. Risks

- **Deadline — VERIFIED 2026-09-12, the field agent was right.** Registration and
  verification-task submission **2026-10-08**; tool submission **2026-10-20** (all CI
  checks must pass); task freeze 2026-11-03. The dates are on
  `sv-comp.sosy-lab.org/2027/index.php` — `dates.php` really is 404, which is what made
  them look unverifiable. Movement 0 is the only movement that fits before the gate, and
  0B matters more than anything offensive.
- **The benchmark pin is part of the score.** `tests/benchmarks/sv-benchmarks` is at
  `7efe28dd` (tag `svcomp26`) and `.gitmodules` sets `ignore = dirty`, so a local edit to
  a `.set` would silently move every number and never show in `git status`. The `.set`
  *inventory* is 2027-compatible — all 37 names are byte-identical to upstream `main`,
  and there is no `svcomp27` tag yet because the task freeze is 2026-11-03 — but the task
  *content* is the 2026 freeze. Any published number is "SAF on the SV-COMP 2026
  benchmark set, scored under 2027 rules". `svbench_commit` is now recorded in the
  output JSON.
- **The 2027 scoring schema is still a draft.** The points table (+2/+1/−16/−32) sits
  under a heading the page itself labels "Obsolete stuff", and the page's editorial TODO
  lists the scoring schema, witness voting and normalization as unwritten. Two in-table
  cells are commented out (`<!-- 2.3 (demo mode) -->` on `valid-memsafety` and
  `valid-memcleanup` correctness), signalling a format 2.3 in flight — score-neutral,
  since demo mode is still not-required. Re-verify before submission.
- **Resuming the loop needs a reset.** `.loop-state/heldout_weighted_prev` holds `4`,
  computed under the 2026 rule. Delete it before `rm .loop-state/STOP`, or the first
  checkpoint reads the rule change as a holdout regression and parks levers. (The loop is
  currently STOPPED, so this is a resume-time action, not a live hazard.)
- **The rules page is an early draft.** It carries an editorial note and has `??`
  placeholders for Java/Python/MoXI. The C rows used here are filled in, but re-check
  before submission.
- **A second validator is a new dependency.** It must be bundled or provisioned the same
  way CBMC and CPAchecker are — and per `saf-lever1-cbmc-loop-free`, `.svtools/` is
  gitignored and version-skewed today. Do not repeat that.

## 5. Definition of done

Slice 0A — **DONE 2026-09-12**, commit on `movement0/scoreboard-2027`:

- [x] Harness decides the witness requirement per 2027 **base category**, from a literal
      table citing the rules page and read-date (`scripts/svcomp_witness_rules.py`), with
      the base-category ← `.set` map kept separate and sourced from the 2027 bench-defs.
- [x] Both columns implemented, not just correctness; the `valid-memtrack` footnote and
      the `Huawei*` demo-mode rows included.
- [x] Format-version floor modelled (`confirmed_va`), reported alongside the headline.
- [x] Out-of-competition tasks handled explicitly: they fall back to the generic cell
      rather than getting a free pass, and `--out-of-competition drop` models the
      competition population. Both measured — net −41 either way. (The plan's "325 /todo
      tasks" is 315 rows / 283 distinct `.yml`; the larger population is the 19,923 rows
      in no 2027 base category, mostly `Unused_Juliet`.)
- [x] Corrected score published (`m0a-2027-rescore.json`): **151 → 110**, with the
      TRUE-side-only rung at exactly 115 so the kill criterion is directly testable.
- [x] `scripts/p211_headroom.py`'s divergent second copy of the table removed.
- [x] `make lint` clean, 50 tests green, `false_alarms == 0`, `wrong_true == 0`.
- [ ] Every memory and plan quoting 151 updated. *(memories done; `plans/211` §5.6's
      "roughly 50" threshold still needs correcting to 36/41.)*
- [ ] Full 55,690-task re-run, for a number defensible as "SAF at HEAD" rather than as a
      re-score of the 2026-09-11 run. Deliberately deferred behind 0B — see the merge-gate
      note in §2.

Slice 0B — not started:

- [ ] Termination emits a witnesslint-clean 2.1 correctness witness.
- [ ] ~~At least one non-CPAchecker 2.1-capable validator provisioned~~ — no longer a
      prerequisite (CPAchecker 4.2.2 validates 2.1; see correction (v)). Still wanted for
      independence, and still a prerequisite for Movement 3.
- [ ] The z3 exec-bit fix + a solver-exercising smoke test in the provisioner, BEFORE any
      validation measurement is trusted.
- [ ] An answer for the 7 loop-free-only and 3 recursion-only clusters, or an explicit
      decision to leave those 10 points on the table.

Follow-ups this slice surfaced but did NOT fix (each is out of 0A's scope, all flagged
rather than silently absorbed):

- **Confirmation provenance.** `validate_witness.sh` Stage 4 confirms by having CBMC
  re-verify the PROGRAM; the witness is never passed to CBMC. ~310 of 940 unreach-call
  CONFIRMEDs and an unmeasured share of 5,405 memsafety ones rest on it. Measured
  exposure on a 255-task sample: a further **−2** weighted. The published 110 is the
  correct 2027 re-score of the dump but is **not yet defensible as "SAF's honest score"**
  until this is measured properly.
- **`valid-memcleanup` is excluded from the manifest** by `svcomp_split.py:62-63`, not
  absent from the corpus: 93 tasks across 10 groups exist, and 2027 makes it the only
  property needing no witness on *either* side. Highest points-per-effort row on the board.
- **A cheaper offensive lever than 0B exists**: a 2.2 violation witness for `no-data-race`
  plus Concurrency `unreach-call`/`no-overflow` is worth **+8** (5 + 3) and needs
  thread-id waypoints on a witness SAF already emits, not a new prover. Note SAF writes
  GraphML 1.0 on the concurrency violation path, so this is a format conversion, not a
  version-string bump.
