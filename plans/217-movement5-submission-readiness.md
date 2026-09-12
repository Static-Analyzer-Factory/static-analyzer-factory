# Plan 217 — Movement 5: submission readiness (the only HARD-GATED item)

**Status:** DESIGNED, not started. **Unblocked, and on the critical path.**
**Branch:** cut `movement5/submission` off `movement0/scoreboard-2027`.
**Track:** mechanics, not capability. **Expected weighted effect: 0 — and without it
every other movement's points are worth exactly 0.**
**Design source:** `plans/191` (strategy), `plans/192` (the BenchExec entry point).
Memory: `svcomp-2027-deadlines-verified`, `saf-svcomp-branch-sequencing`.

---

## 0. Why this is a movement and not a chore

Movements 0–4 raise a score. This one is the difference between that score COUNTING and
not existing. It is the only item in the whole roadmap with a hard external deadline that
nothing can move:

| date | what | days from 2026-09-13 |
|---|---|---:|
| **2026-10-08** | tool **registration** — via a merge request to the bench-defs repo | **25** |
| **2026-10-20** | tool **submission** — "all CI checks must succeed" | **37** |
| 2026-11-03 | verification tasks frozen | 51 |

Verified from `sv-comp.sosy-lab.org/2027/index.php` (`dates.php` 404s — that is why these
looked unverifiable; see `svcomp-2027-deadlines-verified`).

`plans/211`'s sequence is M1 → M2 → M3, all TRUE-side capability, none of it submittable
by itself. **M1 is explicitly worth ~0 weighted points** and M2/M3 are both blocked on it.
So on the current roadmap ordering, nothing that scores and nothing that submits happens
before 2026-10-20. That is the gap this movement closes.

## 1. What already exists (do not rebuild it)

Better than `plans/192`'s status line suggests — that line still says "design / awaiting
approval", dated 2026-08-09, and references the 2026 rules. It is stale:

* `benchexec/tools/saf.py` — the BenchExec tool-info module. EXISTS.
* `scripts/run_benchexec_svcomp.sh` — the runner. EXISTS.
* `saf verify` accepts `--property <prp> --data-model --witness --timeout` and writes
  `witness.yml`, the file name SV-COMP 2027 mandates. VERIFIED end-to-end in Movement 0B.

**Audit these against the 2027 rules before assuming they are current** — they were
written against 2026. Specifically: witness file naming (`witness.yml` /
`witness.svlibyml`), the resource limits (2 CPUs, 7 GB, 90 s violation / 300 s
correctness), and the `.prp` set.

## 2. What does NOT exist

1. **A tool archive.** No Makefile target produces the submittable ZIP. `make` has
   `compile-svcomp`, `test-svcomp`, … and nothing that packages. Per
   `saf-svcomp-branch-sequencing` this was deliberately deferred as "R9, LAST" — that
   sequencing decision was made before the dates were verified and should now be revisited.
2. **The bench-defs merge request.** Registration IS an MR against
   `gitlab.com/sosy-lab/sv-comp/bench-defs`: a `benchmark-defs/saf.xml` declaring
   rundefinitions per base category, plus the tool-info registration. Nothing exists yet.
3. **`.svtools/` is gitignored and version-skewed.** The archive must be self-contained,
   and `saf-lever1-cbmc-loop-free` records that this bit us before. Two live examples found
   during Movement 0B: `lib/native/x86_64-linux/{z3,ltl3ba}` shipped without exec bits
   (invisible — `bin/cpachecker --version` passes anyway), and `witnesslint` cannot be
   imported by `/usr/bin/python3` at all on the VM (missing lxml, pycparser, jsonschema,
   clang), so every lint outside the dev container silently reported a dependency failure
   as a content failure. **An archive built from the current `.svtools/` would ship both
   bugs.**
4. **CI verification.** "All CI checks must succeed" by 2026-10-20 is a pass/fail gate on
   someone else's infrastructure. It needs a dry run with budget to fix what it finds.

## 3. Slices

**5A — Rules audit (half a day, do first).** Diff the existing entry point against the
2027 rules page: witness file names, resource limits, the exact `.prp` files, verdict
strings. Cheap, and it decides whether 5B is packaging or repair.

**5B — The archive (1–2 days).** A `make svcomp-archive` target producing a self-contained
ZIP: the release binary, the tool-info module, `.svtools/` **with exec bits repaired and
checksums pinned**, and a provisioning script that is idempotent and asserts its own
success by exercising the SMT solver rather than by running `--version`. Pin versions; do
not repeat the skew.

**5C — Registration MR (1 day, must land by 2026-10-08).** `benchmark-defs/saf.xml` with
one rundefinition per base category SAF answers. Use
`scripts/svcomp_witness_rules.py`'s `BASE_CATEGORY_SETS` as the authority for the category
names and their `.set` composition — it was derived from the 2027 bench-defs and is
already correct (see `saf-2027-base-category-not-set-basename`; the suffix is NOT the
`.set` basename).

**5D — CI dry run (2 days, budgeted for repair).** Run the submission CI locally, fix,
resubmit. Leave slack: this is the slice most likely to surface something unknown.

## 4. Scope discipline

Do NOT expand this movement into capability work. It ships whatever SAF scores on the day
it runs. If Movement 4 lands first, the archive carries +8; if not, it carries 137. A
submitted 137 beats an unsubmitted 145.

## 5. Definition of done

- [ ] Entry point audited against the **2027** rules, not 2026.
- [ ] `make svcomp-archive` produces a self-contained, reproducible ZIP.
- [ ] `.svtools/` provisioning pins versions, repairs exec bits, and self-asserts by
      exercising the solver — with a test that FAILS if either bug returns.
- [ ] Registration MR opened against bench-defs **before 2026-10-08**.
- [ ] Submission CI green **before 2026-10-20**.
- [ ] The archive reproduces the published score on a sample of tasks.
