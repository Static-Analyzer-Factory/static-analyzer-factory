# Plan 216 — Movement 4: the FALSE-side witness upgrade (+8 weighted)

> # 🛑 PROBE COMPLETE 2026-09-13 — DO NOT BUILD. The +8 is UNREACHABLE in 2027.
>
> The §2 oracle probe below was run (~20 min, no Rust written) and it says stop:
>
> 1. **`witnesslint` cannot confirm anything.** Upstream
>    `benchexec/tools/witnesslint.py::determine_result` returns `result.RESULT_DONE`,
>    never a `false(...)` status. Confirmation needs a validator reporting the SAME
>    STATUS as the verifier, so a linter is structurally incapable of it.
> 2. **The YAML (v2) violation track does not even COVER these targets.**
>    `witnesslint-validate-violation-witnesses-v2` declares no `no-data-race`
>    rundefinition at all, and zero `C.unreach-call.Concurrency` task blocks. The three
>    real violation validators (`cpachecker`, `dartagnan`, `uautomizer`) are all on the
>    v1 track and all consume `witness.graphml`.
> 3. **No in-flight 2027 branch fixes it** — all eight bench-defs branches, including
>    ones from 2026-09-01 and 2026-09-03, list only `witnesslint` on `C.Concurrency`.
>
> So a perfectly-formed 2.2 witness for either target has nowhere to be validated, and
> the emitter would produce artifacts no 2027 validator ever reads.
>
> **The corollary is worth raising with the organizers rather than engineering around:**
> SAF's EXISTING GraphML 1.0 `no-data-race` witnesses are exactly what the v1
> infrastructure validates. They score 0 only because the rules page marks GraphML
> legacy. The rules demand a format with no validator; the format with validators the
> rules forbid. **As configured, no tool can score a `no-data-race` FALSE in 2027.**
>
> Re-check before submission — if a v2 violation validator appears, or the v2
> rundefinitions are extended, this verdict flips and the plan is live again.
> Evidence and repro: memory `saf-movement4-concurrency-validator-gap`.

**Status:** PROBED 2026-09-13 → **BLOCKED on competition infrastructure, not on us.**
Was: "DESIGNED, not started. Unblocked — depends on nothing".
**Branch:** cut `movement4/false-witness-2.2` off `movement0/scoreboard-2027`.
**Track:** **FALSE-side.** The only movement in the sequence that is.
**Design source:** Movement 0A's measured ledger (`plans/212`, commit `12eb25b9`) and
`plans/212` §5 follow-ups. Memory: `saf-score-137-under-2027-rules`,
`svcomp-2027-witness-rules-50pts-at-risk`.

---

## 0. Why this exists and why it is separate

Movements 1–3 are all TRUE-side prover work. Movement 0A's re-derivation of the 2027
rulebook surfaced **8 weighted points on the FALSE side** that no TRUE-side movement can
reach, on verdicts SAF **already emits correctly today**. Nothing needs to be proven that
is not already proven; what is wrong is the witness FORMAT.

This is the cheapest offensive item on the board and it is unblocked, which — with the
2026-10-20 tool-submission gate 37 days out — makes its sequencing position, not its
size, the argument for it.

## 1. The measured exposure

From the 0A ledger over the authoritative 55,690-task dump, dedup-weighted:

| base category | 2027 violation cell | what SAF emits | rows | weighted |
|---|---|---|---:|---:|
| `C.no-data-race.*` | **2.2** | GraphML 1.0 | 131 NOT_CONFIRMED | **−5** |
| `C.unreach-call.Concurrency` | **2.2** | YAML 2.0 | 14 CONFIRMED-but-underversioned | **−3** |
| `C.no-overflow.Concurrency` | 2.2 | YAML 2.0 | 0 scoring | 0 |

The `no-data-race` −5 is a hard loss the harness now scores correctly: those 5 clusters
(`libvsync`, `pthread-atomic`, `pthread-divine`, `pthread-nondet`,
`pthread-race-challenges`) were being given away free under the pre-2027 rule, which
exempted `no-data-race` FALSE outright on the premise that "there is no agreed data-race
witness format or validator". **Format 2.2 retired that premise** — its changelog adds
"support for concurrent (POSIX threads) violation witnesses and the `no-data-race`
property" — and the 2026 rules page had already required format 1.0 there, so the harness
was wrong about this before 2027 too.

The `unreach-call.Concurrency` −3 is the version FLOOR, visible only in the
version-aware column (`confirmed_va`, 137 vs 134): 14 rows whose witness the local
validator confirms but which declare 2.0 against a cell demanding 2.2. A lenient local
CPAchecker does not enforce the floor; SV-COMP will.

## 2. ⚠️ Run the oracle probe FIRST — this is the lesson of Movement 0

**Do not build the emitter before pricing it.** Movement 0B's ranking-witness slice hit
the exact ceiling of CPAchecker's own producer+validator pipeline (10 of 19 clusters), and
that ceiling was discoverable in 45 minutes of probing *before* ~750 lines of Rust. The
same probe applies here and is cheaper:

1. Find a tool that PRODUCES YAML-2.2 concurrency violation witnesses. Check
   `.svtools/CPAchecker-4.2.2-unix/NEWS.md` (it validates termination 2.1 — check what it
   does for `no-data-race`/concurrency 2.2), then the 2027 bench-defs
   `*-validate-violation-witnesses*` rundefinitions for who declares
   `SV-COMP27_no-data-race`.
2. For each of the 5 `no-data-race` clusters and the `unreach-call.Concurrency` rows:
   produce a witness with the oracle, feed it back to a validator, record TRUE/UNKNOWN.
3. **Decision rule.** If the oracle cannot produce-and-reconfirm a cluster, SAF cannot
   score it either — drop it from the estimate before writing code. If fewer than 3 of the
   5 `no-data-race` clusters reconfirm, re-scope to the `unreach-call.Concurrency` version
   bump alone (+3) and redirect the remaining days.

Expect the +8 to shrink. Record what it shrinks to, and why, before slicing.

## 3. Slice 4A — the version floor (+3, hours not days)

Cheapest possible: `C.unreach-call.Concurrency` and `C.no-overflow.Concurrency` violation
cells want 2.2; SAF hardcodes 2.0 at `crates/saf-svcomp/src/witness_yaml.rs:31`.

Movement 0B already built the plumbing — `FORMAT_VERSION_2_1` plus
`build_metadata_versioned`, with the version folded into the deterministic uuid seed
(`witness_yaml.rs`, commit `39ef150f`). Add `FORMAT_VERSION_2_2` and select it per base
category using `scripts/svcomp_witness_rules.py`'s existing table, which already knows
every cell's minimum version.

**The trap:** a version string is not a format. 2.2 added `cycle` waypoints and
concurrency constructs; declaring 2.2 over a 2.0-shaped document may lint clean and still
be rejected, or — worse — be *accepted* locally and rejected by SV-COMP's validator.
Gate this slice on witnesslint with `--expectedWitnessVersion 2.2` **plus** a real
validator, not on the version string appearing.

**Gate.** `confirmed_score_weighted_version_aware` rises by 3 (134 → 137, converging with
the blind column); `false_alarms == 0`; `wrong_true == 0`.

## 4. Slice 4B — `no-data-race` violation witnesses (+5, the real work)

SAF writes **GraphML 1.0** on the concurrency violation path (`commands.rs` ~905), so this
is a FORMAT CONVERSION, not a version bump — the single most commonly mis-scoped item in
this movement's estimate.

What a 2.2 concurrency violation witness needs beyond 2.0: thread identity on waypoints,
and a follow-sequence that survives interleaving. SAF already has the underlying
information — `conc_replay` / `conc_seq` produce a concrete racing schedule, which is
strictly more than the witness format asks for. The work is rendering, not analysis.

Sequence: read the 2.2 violation schema in the vendored `.svwitnesses` (commit `5297b58`,
witnesslint 2.1.3-dev — the GitLab repo is the live one; the GitHub mirror is frozen at
format 0.1 and useless); get a reference witness from the §2 oracle; render; validate.

**Gate.** At least 3 of the 5 clusters CONFIRMED, `false_alarms == 0`, `wrong_true == 0`,
and `witnesslint` clean at 2.2.

## 5. Soundness note

This movement cannot affect the hard invariants, and the reason is structural rather than
incidental: `confirmed_score`'s penalty tail (`return SCORE[outcome]`) is
rule-independent, so a witness change gates only POSITIVE credit. `false_alarms` and
`wrong_true` are untouchable from here — the same argument that made all of Movement 0
safe. Verified per row across 55,690 tasks.

## 6. Definition of done

- [ ] Oracle probe run and the +8 re-priced to a measured number BEFORE any emitter code.
- [ ] `FORMAT_VERSION_2_2` selected per base category from the shared rules table.
- [ ] `no-data-race` emits a YAML 2.2 violation witness (converted from GraphML 1.0).
- [ ] witnesslint clean at the declared version; a real validator confirms.
- [ ] `false_alarms == 0`, `wrong_true == 0`, `make lint` clean, TDD-green.
- [ ] Both score columns published; the version-aware and version-blind totals converge.
