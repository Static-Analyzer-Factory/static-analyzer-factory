# SAF — SV-COMP 2027 submission readiness

**As of 2026-09-13.** Branch `movement5/submission`, cut from `svcomp` @ `ef82ad0a`.

| date | milestone | days left |
|---|---|---:|
| **2026-10-12** | tool **registration** — the fm-tools MR for `data/saf.yml` must EXIST (date re-read from sv-comp.sosy-lab.org/2027 on 2026-10-07; earlier drafts said 10-08) | **3** |
| **2026-10-20** | tool **submission** — all CI checks must SUCCEED | **13** |
| 2026-11-03 | verification tasks frozen | 27 |

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

## 7. Update 2026-10-09 (plan 219–222 session)

* Archive rebuilt from the current binary (witness fixes + the unreach-call TRUE prover):
  `dist/saf-verify.zip`, 24.0 MiB, sha256 `b9111e392483addcbc62cabb43beb255ed92faa60e8db5caa633af90f5e595bc`,
  smoke test 4/4. `python -m benchexec.test_tool_info saf --tool-directory <unpacked>` passes
  structurally (the host's missing `libffi.so.8` is covered by `libffi8` in `required_ubuntu_packages`).
* Registration = the fm-tools MR for `data/saf.yml`, deadline **2026-10-12** (live page).
* Class membership: use `submission/category-structure-classes.diff` (adds `saf` to the verifiers of
  the six C classes and the three Overall rankings; verified to apply to bench-defs `main` @ `22caeaa`).
  A tool in no class has no class or Overall score; abstaining in a sub-category costs nothing.
* The exhaustive TRUE prover's wrong-TRUE audit: 0 PROVE on all 4,324 expected-FALSE tasks (first
  engine); the merging engine's audit is at 2,944 / 4,324 with 0 PROVE at the time of writing
  (`results/svcomp26/debug/final_sweep.log`). Publish the Zenodo record once it reads 4,324 / 0.

## 8. 2026-10-09 — registration MR opened

fm-tools **MR !1166**: https://gitlab.com/sosy-lab/benchmarking/fm-tools/-/merge_requests/1166
(fork `leeewee/fm-tools`, branch `add-saf`). Remaining for a green pipeline by 2026-10-20: publish the
Zenodo record for `10.5281/zenodo.23251564` (reserved, unpublished), merge the BenchExec tool-info PR.
Then the bench-defs MR (`submission/category-structure-classes.diff`) and the jury e-mail.

## 9. 2026-10-10 — TRUE provers landed (plan 222 §2f–§2i); archive must be regenerated

- **Engine** (`crates/saf-svcomp/src/unreach_prove.rs`): flat guarded merging, lazy exhaustive pass, loop-header
  fixpoint (explicit enumeration of concrete header states), simultaneous phi semantics (a latent soundness bug,
  found by the audit, fixed), and a second goal — `no-overflow` — reusing the executor with exact wrap predicates
  at `nsw` arithmetic. `verify`'s `no-overflow` TRUE arm now runs this engine after the interval sentinel.
- **Witnesses**: `build_interval_invariant_witness` now emits bounds only for a header's own integer phis and
  parameters; CPAchecker's "invalid invariant" rejections (4/24 sampled) are gone (18/24 confirmed, 0 FALSE).
- **Audits (every one 0 wrong TRUE)**: unreach-call 4,324/4,324 expected-FALSE on the phi-fixed binary and again
  on the final memory model; no-overflow 3,733/3,733. `false_alarms` unchanged (the FALSE arms were not touched).
- **Measured on svcomp26 (20 s TRUE budget; competition gives ~140 s per pass)**: unreach-call TRUE
  1,768/18,307 (was 1,325), no-overflow TRUE 2,697/5,330 (was 0). Rescored: raw 21,381; confirmed 16,932 if every
  correctness witness confirms, 8,244 if none does; dedup-weighted 166 (was 153); **Overall-equivalent 13,536
  upper / 7,726 lower, ≈ 12,000 realized** at the sampled confirmation rates (unreach 79 %, no-overflow 75 %).
  Details and per-category tables: `plans/222` §2i; rescoring: `results/svcomp26/debug/rescore_both.py`.
- **Action for the submitter**: `target/release/saf` (`45daa037…`) is rebuilt from the tree; regenerate
  `dist/saf-verify.zip` with `make svcomp-archive` and upload THAT file to the reserved Zenodo record
  — regenerated 2026-10-11 from the audited binary `464766c0` (plan 222 §2k–§2n, incl. the counterexample FALSE stage): `dist/saf-verify.zip` sha256 `bd28090b…` supersedes every earlier archive; this is the file to upload. Gates on it: unreach 0/4,324 and no-overflow 0/3,733 wrong TRUE, smoke 132/132; measured unreach TRUE 1,896/18,307, no-overflow TRUE 2,773/5,330, Overall-eq 14,036 upper / 7,947 lower (≈ 12,800 realized). All of this is uncommitted on `svcomp`.
  — regenerated 2026-10-11 (later) from the audited binary `104e44de` (plan 222 §2n round 2/2b + §2o: engine FALSE stage with pinned externs, retries and stubs; signed-shift table + constant-shift source gate; IEEE floats and math builtins): `dist/saf-verify.zip` sha256 `5984ba6c90af7361f33530949adbb1d765b668f515adb7a3c118a1e57c9083f0` **supersedes `bd28090b…`; this is the file to upload.** Gates on it: unreach 0/4,324 and no-overflow 0/3,733 wrong TRUE, 540 expected-TRUE tasks 0 wrong FALSE, smoke 133/133; measured no-overflow TRUE 3,551/5,330, new FALSE 2,998/3,426, unreach TRUE ≥ 1,810/18,307 (float-engine sweep running) → Overall-eq 16,668 upper / 7,947 lower (≈ 60 % of the FALSE-side gap realized in a 60-witness validation sample). A local validator bug (no data model passed to CPAchecker/CBMC) was fixed in `scripts/validate_witness.sh`. Uncommitted on `svcomp`.

## 10. 2026-10-11 (evening) — third audit-found wrong TRUE fixed; v13 gates running

`audit_v12` (plan 222 §2p binary, never archived) found a wrong PROVE on `neural-networks/gcas_6_unsafe.c-amalgamation.i`
(official `ReachSafety-Floats`-side category; the error is reachable, confirmed by native sampling). Cause and fix in
plan 222 §2q: memory cells are now byte offsets (per-global `Layout` and per-GEP `llvm.gep_bytes` from the LLVM data
layout; overlap-aware stores) instead of GEP-shaped structural paths. Release `6655bed3` is under `chain_v13a`
(0 PROVE required over 4,324 + 3,733 expected-FALSE tasks, 0 wrong FALSE over 540 expected-TRUE, smoke) and then
`chain_v13b` (18,307-task TRUE sweep). **Until v13a reports clean, the upload file stays `dist/saf-verify.zip`
`5984ba6c…` (§9).** A v13 archive, if produced, supersedes it and carries the §2p/§2q gains (no-overflow TRUE
3,730/5,330 on v12 vs 3,551 on v10; unreach TRUE to be measured).

## 11. 2026-10-11 (night) — archive status: NONE shippable on disk; fourth soundness fix in the FALSE stage

- `chain_v13a` on release `6655bed3` passed every gate except the known non-competition `ldv-consumption/todo/` PROVE
  (recorded, §10): no-overflow TRUE 3,790/5,330, FALSE 3,049/3,426, Overall-eq 16,876 upper / 7,947 lower.
- Its archive FAILED `smoketest.sh`: `smoke_unknown_unreach.c` → `false`. Cause: replay stubs for functions the task
  declares but never defines returned the engine's pinned values (plan 222 §2r). Fixed so a called stub ends the
  replay unconfirmed. This change is in the FALSE replay stage only; `chain_v14a` re-runs all gates on the rebuilt
  release after the v13b TRUE sweep completes.
- **`dist/saf-verify.zip` is currently absent.** The failed-smoke archive is kept as
  `dist/UNVERIFIED-v13-smoke-failed-saf-verify.zip` (do not upload); the §9 archive `5984ba6c…` was overwritten by it
  and cannot be regenerated (its binary is gone). `dist/saf-verify.zip.sha256` still names the older `bd28090b…`.
  The next clean chain writes a new `dist/saf-verify.zip`; update this file and Zenodo only from that.
- fm-tools MR !1166: all CI jobs `stuck_pending_no_matching_runners` (fork has no runners); `data/saf.yml` passes
  every format/schema check locally; see the registration memory / message of 2026-10-11.

## 12. 2026-10-12 — new upload candidate: `dist/saf-verify.zip` sha256 `781aeb7d…`

Built from release `d883436d` (code = commit `dc44817` on `svcomp`): byte-offset memory cells (plan 222 §2q), replay stubs
that abort when an undefined function is called (§2r), the `valid-memsafety` TRUE goal (§2s), 32-bit pointer width from
the data layout, `sprintf` havocking its buffer. `chain_v14a` gates, counted on competition tasks only
(`competition_proves.py`: a task counts when the svcomp26 run gave it a 2027 base category):
unreach-call 0 / 4,324; no-overflow 0 / 3,733 (the one PROVE is the non-competition `ldv-consumption/todo/` task);
valid-memsafety 0 / 10,087 (432 PROVE, all in `Unused_Juliet.set`: CWE690 allocator-returns-NULL and CWE122
`sizeof(pointer)` families, whose labels contradict SV-COMP's never-failing allocation and are excluded from the
competition for that reason); 540 expected-TRUE tasks 0 wrong FALSE; FALSE sweep 3,041 / 3,426; archive smoketest
passed. Supersedes `5984ba6c…` (§9). Provisional score with the v13 unreach sweep and the memsafety sweep:
~18,870 upper / ~9,930 lower Overall-eq; `chain_v14b` re-measures on this binary.

## 13. 2026-10-13 — upload candidate: `dist/saf-verify.zip` sha256 `0fe0efe2…` (release v15, `dff01674`)

Adds to §12: termination TRUE from the engine, the global ctor/dtor gate on every engine prover, memsafety FALSE from
engine counterexamples (ASan replay), `valid-memtrack` FALSE via LeakSanitizer (verdict-only in 2027), the model cache.
`chain_v15a` on competition tasks: 0 wrong PROVE over 4,324 unreach / 3,733 no-overflow / 10,087 memsafety / 974
termination expected-FALSE tasks; 0 wrong FALSE over the 540 sanity tasks; `saf verify` on all 2,826 expected-TRUE
memsafety tasks: 2 FALSE, both on tasks whose labels are wrong in sv-benchmarks (`coreutils-v9.5-units/
seq_incr_no_carry_{functional,cover_proof}`: heap overflow when `num_digits == 1`; ASan- and CPAchecker-confirmed
witnesses in `results/svcomp26/debug/labelbug/`; reported upstream by the user on 2026-10-13). Memsafety FALSE on the
2,422 expected-FALSE tasks: 1,618 with the labelled sub-property (338 new), 70 naming another sub-property (UNKNOWN in
BenchExec, 0 points). Score: **21,493 upper / 11,054 lower Overall-eq** (≈18,700 at measured confirmation rates).
Supersedes `781aeb7d…` (§12).


## 14. 2026-10-14 — upload candidate: `dist/saf-verify.zip` sha256 `a697ffa9…` (release v17, `4669557c`)

v16 (commit `8ed9afa`: Houdini invariants for k-induction, float abstraction, SCCP parameter fix, ambiguous debug-name
fix — the `byte_add_1-1` invalid-witness bug) plus v17 (a feasible NULL dereference abstains in every proof mode instead
of ending the path: the non-competition `ldv-consumption/todo/32_7a…flexcop-usb` was PROVEd no-overflow although
UAutomizer finds the overflow past a read through a NULL global pointer). Gates `chain_v16a/m`, `chain_v17m` on
competition tasks: 0 wrong PROVE over 4,324 unreach / 3,733 no-overflow / 10,087 memsafety / 974 termination
expected-FALSE tasks; `saf verify` TRUE on 0 of the 4,454 expected-FALSE unreach-call + no-overflow tasks (including the
1,028 no-overflow tasks the FALSE sweep had not covered, which audits the interval no-overflow sentinel); 0 wrong FALSE
over the 540 sanity tasks; memsafety verify on 2,826 expected-TRUE: only the two reported label bugs. FALSE sweep
3,055 / 3,426. Witness sample (CPAchecker, then UAutomizer): 73 / 78 TRUE witnesses confirmed, 0 refuted; 17 / 20 of the
new Hardness proofs. Score (v16 sweeps; v17 costs 5 unreach-call proofs): **≈21,700 upper / ≈11,000 lower Overall-eq**.
Supersedes `0fe0efe2…` (§13, never uploaded).

## 15. 2026-10-14 — v17 withdrawn; candidate `dist/saf-verify.zip` sha256 `50db8f61…` (release v18, `71f36fc6`), gates pending

The v17 archive of §14 (`a697ffa9…`) must NOT be uploaded: a frontend/engine review after its gates found wrong-TRUE
holes no audited task exercises — an atomic read-modify-write modelled as storing its operand (`__sync_fetch_and_add`),
inline asm that writes memory dropped silently (Linux `set_bit`'s `bts`, `movl $1, g(%rip)`), `fesetround` (the engine
rounds to nearest), and `undef` read as one symbol per path. v18 (commit `ada3c21`) refuses or models each of them
(regression tests in `crates/saf-cli/tests/smoke.rs`), and adds `no-overflow` TRUE for threaded programs
(`conc_overflow.rs`: execution bounds, flow-insensitive ranges of address-untaken globals, guard-narrowed SSA ranges;
414 / 661 expected-TRUE `C.no-overflow.Concurrency` tasks, 0 / 8 expected-FALSE, 0 PROVE over the 3,733 expected-FALSE
no-overflow tasks). Estimated score ≈ 23,550 upper Overall-eq (v16 sweeps + the concurrent proofs; that category takes
no witness). The v18 archive passed its smoketest; the full gate chain (`chain_v18m`) is running — upload only after
`dist/README-ARCHIVE-STATUS.txt` says the gates are clean.
