# SAF — SV-COMP 2027 submission readiness

**As of 2026-09-13.** Branch `movement5/submission`, cut from `svcomp` @ `ef82ad0a`.

| date | milestone | days left |
|---|---|---:|
| **2026-10-08** | tool **registration** — the merge requests must EXIST | **25** |
| **2026-10-20** | tool **submission** — all CI checks must SUCCEED | **37** |
| 2026-11-03 | verification tasks frozen | 51 |

**Score SAF would submit today: 123 dedup-weighted**, `false_alarms == 0`,
`wrong_true == 0`. See §5 for the caveat on that number.

---

## 1. What is done and verified

| | evidence |
|---|---|
| `svcomp` is the competition mainline | fast-forwarded to `ef82ad0a`, all 33 commits (auto-loop → lever1 → Movement 0), zero divergence |
| No SV-COMP participant is bundled or invoked | CPAchecker and CBMC removed from the verify path; `commands.rs` 8297 → 7786 lines, `cbmc.rs` → `nondet.rs` 636 → 153, `Lever::Cbmc` gone. The live binary logs `portfolio plan [Bmc, Se, Fuzz]` |
| Rust gates green | `cargo clippy --workspace -- -D warnings` 0 · `cargo fmt --check` 0 · **2702 tests run, 2702 passed** |
| The behaviour change is pinned by tests | 6 rewritten smoke tests pass, incl. `verify_unreach_wrongprove_is_not_true` — the regression that fires if anyone re-enables a TRUE arm before fixing the absint soundness bugs |
| `make svcomp-archive` works end to end | exit 0; `dist/saf-verify.zip`, 23.9 MiB, 35 members, single `saf/` root, byte-reproducible |
| The archive passes fm-tools' real rules | `ci/check_archive.py` fetched from upstream and applied: one top dir, README/LICENSE/smoketest.sh at root, no repo paths, no symlinks, mode bits survive |
| `smoketest.sh` passes on the UNPACKED archive | `false(unreach-call)` + a 1208-byte violation witness, then `unknown`; 4 checks |
| The tool-info module passes upstream CI | ruff 0.16.7 check + format, codespell 2.4.3, reuse 6.2.0 — all 0, run against upstream's own `pyproject.toml`/`REUSE.toml` |
| The version string is competition-legal | binary prints `saf 0.1.0 (LLVM 18.1)`; the module reports `0.1.0 (LLVM 18.1)` — verified against the real unpacked archive |
| Output fits the 2 MB cap | 40 tasks across the largest sources and biggest families: **max 808 bytes, 0.039 % of the cap** |
| The witness-rule model is correct | `svcomp_witness_rules.py` §2 matches the live 2027 rules page on **all 112 scoring cells**; §1 matches bench-defs on 50/50 base categories |

## 2. Registration artifacts — DRAFTED, nothing opened

Under `submission/`. Nothing has been cloned, forked, pushed or opened upstream.

* `fm-tools-saf.yml` — validates against the real `data/schema.yml`; key order matches
  `ci/check-data.py::validate_property_order`; both formatting scripts are fixed points.
* `category-structure.diff` — `opt_in` on **26 base categories** (derived from measured
  per-category scores, not from the property split). Applies cleanly (`git apply --check`).
  Running the real `create-benchdefs.py` emits `saf.xml` with 5 rundefinitions, exactly
  26 `<tasks>` blocks and zero `<option>` elements; removing the hunk emits nothing.
* `REGISTRATION.md` — the ordering runbook.

## 3. ⭐ The blocker nobody had listed

**The BenchExec tool-info module must be MERGED upstream, and there is no escape hatch.**
All three spellings the fm-tools schema permits collapse to `benchexec.tools.saf` in
`check_archive.py:202-216` + `model.py:151-183`, so shipping the module inside the archive
does not work. This is a third-party review queue we do not control, and it gates the other
three changes. **If the BenchExec PR is not merged by roughly late September, the fm-tools
MR cannot go green before 2026-10-20.**

Chain, strictly serial: **BenchExec PR merged → archive on Zenodo (version DOI minted) →
fm-tools MR merged → bench-defs MR (submodule bump + `opt_in` + generated `saf.xml`, one commit)**.

## 4. ⭐ Two silent failure modes

1. **A bench-defs MR that adds the `opt_in` block without bumping the fm-tools submodule
   produces a FULLY GREEN pipeline in which SAF is simply not a participant.**
   `check-categories.py:84-89` ("Verifiers listed in category X, but not participating")
   only fires for tools named in a meta-category `verifiers:` list, and SAF is in none.
   Green CI, no entry. The loud failure is the opposite order.
2. **fm-tools CI is year-pinned to 2024/2025/2026.** For a 2027-only entry,
   `check_participation`, the `participants[]` gate, both package jobs and `smoke-test` are
   all inert. **Green CI on the fm-tools MR today proves almost nothing.** Re-read
   `ci/.gitlab-ci.yml` immediately before submission and re-run the MR after upstream bumps.

## 5. Caveat on the 123

123 is the existing 55,690-task dump **re-scored** under the tool removal, not a fresh run:
the CPAchecker-gated TRUEs turned to `unknown` (−12) and CBMC's two clusters `xcsp` and
`recursified_nla-digbench` dropped (−2). Both deltas were verified independently — the
gate was proven fail-closed by running the binary with and without CPAchecker, and the CBMC
delta was measured by an A/B on the lever's scoped population (49 confirmed → 0).
A full re-run at HEAD is still wanted for a number defensible as "SAF on the day", and it is
the one measurement this movement did not do.

## 6. Open items, by owner

### Needs a human — registration cannot proceed without these

| item | why no CI catches it |
|---|---|
| **ORCID** for each maintainer and participant | `0000-0000-0000-0000` is pattern-valid; this is the placeholder most likely to slip through |
| **gitlab.com username** for `fmtools_entry_maintainers` | checked against the live API, so this one DOES fail loudly |
| **Jury member**: name, institution, country — and they must **e-mail the organizers** | no CI check exists for the e-mail step at all |
| Full **`participants[]`** list | the gate that would demand it is pinned to 2026 |
| **SPDX copyright holder** in `benchexec/tools/saf.py` | all four upstream CI jobs pass with `PLACEHOLDER-SAF-COPYRIGHT-HOLDER` in place. `make test-toolinfo` is RED until this is filled in — deliberately, it is the only thing that stops it reaching a public PR |
| **Zenodo publication** → version DOI | a plain `url` is forbidden for year ≥ 2024; the DOI must be the version DOI and the record must hold exactly one `.zip` |
| **Licence decision** | `Cargo.toml:20` declares `MIT OR Apache-2.0` but only an MIT `LICENSE` exists. Either add `LICENSE-APACHE` or narrow the declaration — I did not assert a licence on the project's behalf |
| **Who opens the BenchExec PR**, and when | see §3 — longest lead item |

### Known-imperfect, deliberately left

* `benchmark-defs/saf.xml` in this repo is SAF's own local harness file and is **dead**
  (every `<includesfile>` names a `.set` that no longer exists). It must never be submitted;
  the competition one is generated. Left alone rather than half-fixed.
* `.svtools/` (309 MB, CPAchecker + CBMC) stays in the repo as the **measurement instrument**
  the scoring harness uses to predict validator confirmation. It is not in the archive —
  verified, 35 members, none of them `.svtools`.
* The work on `movement5/submission` is **uncommitted**, per the standing "never auto-commit"
  rule. The archive is therefore built from a dirty tree and corresponds to no commit SHA.
* Movement 4 (plans/216, +8 weighted, FALSE-side witness 2.2) is not started.
