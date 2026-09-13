# SAF — SV-COMP 2027 registration runbook

Status: **DRAFT. Nothing has been cloned, forked, pushed or opened.** This file
records the exact steps, in the exact order, that a human must perform.

Dates that bound everything (source: <https://sv-comp.sosy-lab.org/2027/index.php>,
cross-checked in memory note `svcomp-2027-deadlines-verified`):

| Date | Gate |
| --- | --- |
| 2026-10-08 | Tool **registration** — the merge requests must exist |
| 2026-10-20 | Tool **submission** — all CI checks must be green |

Everything below was derived by reading the live upstream sources on 2026-09-13
(`main` of each repo). Where a check is quoted, the file and line are given so
you can re-read it if upstream moves.

---

## 0. The dependency chain, in one picture

```
 MR 0  BenchExec PR: add benchexec/tools/saf.py          (github.com/sosy-lab/benchexec)
   |        must be merged to main
   v
 STEP A  Publish the archive on Zenodo, mint a VERSION DOI
   |        exactly one .zip file in the record
   v
 MR 1  fm-tools MR: add data/saf.yml                      (gitlab.com/sosy-lab/benchmarking/fm-tools)
   |        must be MERGED, because...
   v
 MR 2  bench-defs MR: ONE commit that contains all three of
          - benchmark-defs/category-structure.yml   (the opt_in hunk)
          - benchmark-defs/saf.xml                  (GENERATED, never hand-written)
          - the fm-tools submodule pointer bumped to the commit that has saf.yml
```

There are **four** upstream changes, not two. MR 0 is the one that is easy to
forget and the one with the longest lead time, because it is a third-party
review on GitHub.

---

## 1. Placeholders a human must fill in

`submission/fm-tools-saf.yml` is schema-valid as written, but every value below
is a deliberate placeholder. Replace all of them, then delete the header comment
block and the trailing `# REPLACE-ME:` comments, then re-run the format check
(section 6.2) — the file is a `ruamel.yaml` round-trip fixed point both with and
without those comments, so removing them is safe.

| Field | Placeholder in the draft | Why CI rejects the placeholder |
| --- | --- | --- |
| `fmtools_entry_maintainers[0]` | `REPLACE-ME-gitlab-username` | `ci/check-data.py:21-52` `_check_gitlab_handle` queries `https://gitlab.com/api/v4/users?username=<handle>` and fails if the response is empty. Must be a real gitlab.com account. |
| `maintainers[0].orcid` | `0000-0000-0000-0000` | schema pattern `^[0-9]{4}-[0-9]{4}-[0-9]{4}-[0-9]{3}[0-9X]$` (`data/schema.yml:127`). The all-zero ORCID is pattern-valid, so CI will **not** catch it — it is a social contract, not a machine check. Upstream uses `0000-0000-0000-0000` only for the "Hors Concours" non-participation stubs. |
| `maintainers[0].{name,institution,country}` | `REPLACE-ME …` | required by `data/schema.yml:139-144`; not machine-checked for content. |
| `competition_participations[0].jury_member.*` | `REPLACE-ME …` | `jury_member` is required (`data/schema.yml:264-268`). SV-COMP requires every active participant to nominate a jury member. |
| `competition_participations[0].participants[]` | one `REPLACE-ME Participant` | `fm-tools/ci/.gitlab-ci.yml:34-54` fails `check-data-schema` with "Declaration of team members missing for SV-COMP <year> in <file>" when an *active* (unlabelled) participation has no `participants[].name`. List every author. |
| `versions[0].doi` | `10.5281/zenodo.0000000` | see STEP A. `ci/check-data.py:75-91` `check_require_doi` hard-fails without a DOI for any participation year >= 2024, and forbids `url` entirely. |

Additionally, decide two things:

* **License string.** The draft says `spdx_license_identifier: MIT`, matching the
  only license file in the repo (`LICENSE`, "MIT License", VM
  `~/static-analyzer-factory/LICENSE`). `Cargo.toml:20` says
  `license = "MIT OR Apache-2.0"` but there is no `LICENSE-APACHE` file. Either
  add the Apache file and change the entry to `MIT OR Apache-2.0`, or fix
  `Cargo.toml`. `ci/check_archive.py:139-146` separately requires a file whose
  name starts with `license`/`licence` in the archive's root directory.
* **`base_container_images`.** The draft says `docker.io/ubuntu:24.04`, which is
  what every SV-COMP 2026 entry uses and what
  `competition-scripts/test/Dockerfile.user.2026:9` builds from. There is no
  `…/competition-scripts/user:2027` image yet. Re-confirm once it is published.

---

## 2. MR 0 — BenchExec tool-info module (do this FIRST)

**Repository:** <https://github.com/sosy-lab/benchexec> (GitHub, not GitLab).
**Change:** add `benchexec/tools/saf.py`.

Verified: `benchexec/tools/` on `main` contains 196 modules and `saf.py` is not
one of them (paginated listing of
`/api/v4/projects/sosy-lab%2Fsoftware%2Fbenchexec/repository/tree?path=benchexec/tools`).
The candidate module already exists on the VM at
`~/static-analyzer-factory/benchexec/tools/saf.py` — it is currently vendored for
SAF's own BenchExec runs and is what should be upstreamed.

**There is no way to avoid this.** `benchexec/model.py:151-183` `load_tool_info`
does `tool_module = tool_name if "." in tool_name else f"benchexec.tools.{tool_name}"`.
`fm-tools/ci/check_archive.py:202-216` `_get_tim_name` strips
`benchexec.tools.`, strips a URL down to its basename, and strips `.py`, so all
three spellings the schema allows collapse to the bare name `saf` and then to
`benchexec.tools.saf`:

* `benchexec_toolinfo_module: benchexec.tools.saf` → `benchexec.tools.saf`
* `benchexec_toolinfo_module: "saf.py"` (what `fizzer` writes) → `benchexec.tools.saf`
  — and `benchexec/tools/fizzer.py` **does** exist upstream, which is why fizzer works
* `benchexec_toolinfo_module: "https://…/benchexec/tools/saf.py"` (what `owic`
  writes) → `benchexec.tools.saf` — and `owic`'s URL points at
  `benchexec/-/raw/main/benchexec/tools/owic.py`, i.e. the module is upstream anyway

Shipping `saf.py` inside the archive does **not** work for
`fm-tools`' `check-archives-*` job.

Checks the module must survive (`ci/check_archive.py:252-301`, run against the
unpacked archive with the tool directory as CWD):

* `tool.name()` must be non-empty — SAF returns `"SAF"`
* `tool.version(exe)` must be non-empty, single-line, <= 100 chars, and must
  **not** start with `tool.name()`. `--version` currently prints
  `saf 0.1.0 (LLVM 18.1)`, which escapes only because `"saf" != "SAF"` by case.
  The digit-first fix (`0.1.0+svcomp27 (llvm18)`) that is being done separately
  removes that coin-flip. Land it before MR 1.
* `tool.executable(ToolLocator(use_path=True, use_current=True))` must return an
  existing executable file
* `list(tool.program_files(exe))` must not raise

Allow several weeks. This PR is the long pole.

---

## 3. STEP A — Zenodo (do this SECOND, before MR 1)

A Zenodo **version** DOI is mandatory and a plain `url` is forbidden:

```
ci/check-data.py:75-91  check_require_doi
  "From 2024 all tools must upload their tools to zenodo.org and provide an DOI
   instead of an URL. The URL tag is forbidden."
```

The schema enforces the shape `10\.5281/zenodo\.[0-9]+$` (`data/schema.yml:155`)
and `versions[]` has a `oneOf` that accepts either `{version,
benchexec_toolinfo_options, required_ubuntu_packages, doi}` or the same with
`url` (`data/schema.yml:170-180`) — `check_require_doi` then rules out the `url`
branch for 2027.

Hard requirements on the record, from
`lib-fm-tools/python/src/fm_tools/zenodo.py`:

1. The DOI must be the **version** DOI, not the concept DOI. `zenodo.py:18-36`
   fetches `https://zenodo.org/api/records/<id>` and raises
   `UnsupportedDOIException` with the message *"The DOI must point to a specific
   version and not redirect to the latest version."* if that is not a 200.
2. The record must contain **exactly one file** (`zenodo.py:56-61`: "There are
   more than one file in the Zenodo record, but only one is allowed").
3. That file must be a `.zip` with an md5 checksum (`zenodo.py:41-49`).
4. The record must be **published**, not a draft — `check-archives-*` downloads
   and unzips it.

The archive layout is fixed by SAF itself: `resolve_svcomp_stub()` walks
`current_exe().ancestors()` joined with `share/saf/stubs/sv-comp-stubs.h`, so
the zip must be `<anything>/bin/saf` + `<anything>/share/saf/{stubs,specs}/…`
plus a `README*` and a `LICENSE*` in that same top directory
(`ci/check_archive.py:131-146`). `competition-scripts/execute_runs/mkInstall.sh:38-48`
unzips and then flattens the single top directory away — it errors out with
"Archive does not contain exactly one folder" if there is more than one — so
nothing may depend on that directory's name. (Archive construction is covered by
the archive part of this submission, not here.)

---

## 4. MR 1 — fm-tools: add `data/saf.yml`

**Repository:** <https://gitlab.com/sosy-lab/benchmarking/fm-tools>
**Change:** one new file, `data/saf.yml`, byte-identical to
`submission/fm-tools-saf.yml` after you fill in section 1.

Nothing else changes. `data/*.yml` is already covered by `.reuse/dep5`
(`Files: data/*.yml … License: CC-BY-4.0`), so no SPDX header is needed and
`check-file-reuse` passes without further work. `ci/check_file_conventions.py:17-23`
requires the file stem to match `[a-z0-9\-\+]+` — `saf` does.

### 4.1 Key order — this is the thing that trips people

`ci/check-data.py:113-132` `validate_property_order` hard-codes this list and
fails with "Properties must follow the given order […]" on any deviation:

```python
expected_order = [
    "id",
    "name",
    "description",
    "input_languages",
    "project_url",
    "repository_url",
    "spdx_license_identifier",
    "benchexec_toolinfo_module",
    "fmtools_format_version",
    "fmtools_entry_maintainers",
    "maintainers",
    "versions",
    "competition_participations",
    "techniques",
    "frameworks_solvers",
    "literature",
    "supplementary",
]
```

Of those 17, **14 are required** (`data/schema.yml:851-866`): `id`, `name`,
`input_languages`, `project_url`, `spdx_license_identifier`,
`benchexec_toolinfo_module`, `fmtools_format_version`,
`fmtools_entry_maintainers`, `maintainers`, `versions`,
`competition_participations`, `techniques`, `frameworks_solvers`, `literature`.
`description`, `repository_url` and `supplementary` are optional; the draft sets
the first two and omits `supplementary` (SAF has no proceedings paper yet).

### 4.2 The participation must carry NO label

```yaml
competition_participations:
  - competition: "SV-COMP 2027"
    track: "Verification"
    tool_version: "svcomp27"
    ...                       # <- no `label:` key at all
```

`bench-defs/.gitlab-ci.yml:44-50` (`check-definitions`) selects tools with

```
.competition=="SV-COMP 2027" and .track=="Verification" and (.label | length == 0)
```

Any label — including `inactive` — drops SAF out of that list and
`check-benchdef.py` never runs on `benchmark-defs/saf.xml`. Note that upstream
has already pre-seeded every existing tool with an *inactive* "Hors Concours"
`SV-COMP 2027 / Verification` entry (see `data/racerf.yml`, `data/mopsa.yml`);
SAF's entry must be the unlabelled kind.

Separately, `fm-tools/ci/.gitlab-ci.yml:34-54` requires `participants[].name` for
exactly the unlabelled participations — so dropping the label also switches on
the team-members requirement from section 1.

### 4.3 `benchexec_toolinfo_options: []` — why empty

Leave it empty and let the tool-info module build the command line.

* `create-benchdefs.py:262-313` turns each string in `benchexec_toolinfo_options`
  into a `<option name="…"/>` element, and `benchmark-defs/reference/reference-verifier.xml`
  substitutes them at **`<benchmark>` level**, i.e. globally for every
  rundefinition. There is nothing SAF wants applied unconditionally to all five
  properties.
* Never put `${witness}` there for the Verification track. BenchExec does not
  substitute it in option values; it passes it through literally with only a
  warning. `create-benchdefs.py:303-310` only lifts `${witness}` into a
  `<option name="…">{witness_placeholder}</option>` for the *validation* tracks
  (`_get_validation_track_id`), which SAF does not enter.
* SAF's clap default for `--witness` is already `witness.yml`, which is what the
  SV-COMP 2027 rules require, and the generated benchmark definition already
  carries `<resultfiles>**/witness.*</resultfiles>` (inherited from the reference
  template — confirmed in the generated file, section 6.3).
* The tool-info module already derives `--property` from `task.property_file` and
  `--data-model` from `task.options["data_model"]`, which cannot be expressed as
  static options anyway.

Empirically: `racerf` and `sv-sanitizers` both ship `benchexec_toolinfo_options: []`
and their generated benchdefs contain zero `<option>` elements. So does the
`saf.xml` generated in section 6.3.

### 4.4 `required_ubuntu_packages` — every entry verified installed

Two CI jobs guard this list, both running **inside**
`registry.gitlab.com/sosy-lab/benchmarking/competition-scripts/user:<year>`
(`fm-tools/ci/.gitlab-ci.yml:126-204`):

* `check-valid-required-packages-<year>` — `apt-cache show -q=0 <pkg>`: the name
  must exist in that image's apt sources.
* `check-installed-required-packages-<year>` — `dpkg --get-selections <pkg> | grep install`:
  the package must already be **installed** in the image.

The authoritative snapshot of what is installed is
`competition-scripts/test/Ubuntu-packages.txt` (Ubuntu 24.04 "noble"), and
`competition-scripts/test/.gitlab-ci.yml` keeps it honest by failing if any line
is not marked `[installed`. Line numbers below are from that file as of
2026-09-13:

| Package | Ubuntu-packages.txt | Why SAF needs it |
| --- | --- | --- |
| `clang-18` | L29 `clang-18/noble-updates,now 1:18.1.3-1ubuntu1 amd64 [installed,automatic]` | `DEFAULT_CLANG = "clang-18"` (`crates/saf-cli/src/commands.rs:1074`), spawned to compile every native replay harness |
| `llvm-18` | L577 `[installed,automatic]` | provides `opt-18`; `DEFAULT_OPT = "opt-18"` (`commands.rs:1076`) |
| `libc6-dev-i386` | L198 `[installed]` | SAF compiles ILP32 harnesses with `-m32` (`commands.rs:302`, `:410`) |
| `libclang-rt-18-dev` | L223 `[installed,automatic]` | compiler-rt runtimes for `-fsanitize=address` (`commands.rs:4172`), `-fsanitize=thread` (`:5590`), `-fsanitize=signed-integer-overflow` (`:2024`) and `-fsanitize-coverage=…` (`:2040`), in both the x86-64 and i386 flavours |
| `libffi8` | L278 `[installed,automatic]` | the 70 MB `saf` binary links `libffi.so.8` through LLVM-18; the portability-sensitive dep |
| `libgcc-s1` | L291 | measured dynamic dep |
| `libstdc++6` | L476 | measured dynamic dep (statically-linked LLVM/Z3 C++ runtime) |
| `libtinfo6` | L497 | measured dynamic dep (`libtinfo.so.6`) |
| `libzstd1` | L557 | measured dynamic dep (`libzstd.so.1`) |
| `zlib1g` | L756 | measured dynamic dep (`libz`) |

Note that `clang-18` and `libclang-rt-18-dev` are currently only present as
*automatic* dependencies of the `clang` metapackage
(`Dockerfile.user.2026` installs `clang`, which on noble is `1:18.0-59~exp2`).
`dpkg --get-selections` still reports them as `install`, so the CI check passes —
but listing them explicitly is what guarantees they survive if another tool drops
`clang` from its list. That is exactly what `mopsa` does
(`data/mopsa.yml` lists `libclang-rt-18-dev` and `python3-clang-18`).

One thing I did *not* verify and you should: that `libclang-rt-18-dev` really
ships the **i386** sanitizer runtimes, not just x86-64. SAF compiles ILP32 replay
harnesses with `clang-18 -m32 -fsanitize=address`; TSan replay is always LP64
(`commands.rs:5614`: "`-fsanitize=thread` + `-m32` fails: unsupported option for
target"). Check it inside the competition image:

```bash
podman run --rm registry.gitlab.com/sosy-lab/benchmarking/competition-scripts/user:2026 \
  bash -c 'ls /usr/lib/llvm-18/lib/clang/18/lib/*/ | grep -i "asan.*i386\|i386.*asan"; \
           printf "int main(){return 0;}" > /tmp/t.c; \
           clang-18 -m32 -fsanitize=address /tmp/t.c -o /tmp/t && echo "m32+asan OK"'
```

If the i386 runtime is missing, add `gcc-multilib` (Ubuntu-packages.txt L101,
installed) and/or fall back to `-m64` for the ASan replay.

**Before opening MR 1, re-verify against the real image** (see 6.1).

### 4.5 Techniques and frameworks — evidence for each claim

`techniques[]` values must come verbatim from the `oneOf` enum at
`data/schema.yml:270-626`; `frameworks_solvers[]` from `data/schema.yml:628-783`.
Both lists must be sorted case-insensitively (`ci/format_data_sort.py:35-41`) or
`check-data-format` fails on `git diff --exit-code data/`.

| Claimed technique | Evidence on the VM |
| --- | --- |
| Abstract Interpretation | `crates/saf-analysis/src/absint/` (domain, fixpoint, octagon, interval, threshold, …) |
| Numeric Interval Analysis | `crates/saf-analysis/src/absint/interval.rs` |
| Bounded Model Checking | `crates/saf-svcomp/src/bmc.rs`, `bmc_incremental.rs`, `ssa_encode.rs` |
| Bit-Precise Analysis | 32 bit-vector references in `crates/saf-svcomp/src/ssa_encode.rs` |
| Symbolic Execution | `crates/saf-svcomp/src/se_interp.rs` |
| Fuzzing | `crates/saf-svcomp/src/fuzz.rs` |
| Guidance by Coverage Measures | `-fsanitize-coverage=inline-8bit-counters,pc-table,trace-cmp` (`commands.rs:2040`) |
| Targeted Input Generation | input synthesis that feeds the native replay harnesses |
| Concurrency Support | `crates/saf-svcomp/src/{race.rs,race_true.rs,conc_replay.rs,conc_seq.rs,conc_shim.rs}` |
| Ranking Functions | `crates/saf-svcomp/src/{ranking.rs,termination.rs,termination_witness.rs}` |
| Portfolio | `crates/saf-svcomp/src/portfolio.rs`; `saf verify` logs "portfolio plan …" (`commands.rs:1874`) |

`frameworks_solvers: [Z3]` only. Z3 is statically **linked** as a library
(`crates/saf-cli/Cargo.toml`: `saf-analysis` with `features = ["z3-solver"]`) and
never spawned. **`CPAchecker` and `CProver` must not appear** — both were removed
from the verify path on `movement5/submission`; `crates/saf-svcomp/src/cbmc.rs`
is deleted in the working tree and the archive contains no `.svtools/` directory.
Claiming either would be false and would also make SAF look like a meta-tool.

`AFL++` is likewise **not** listed: SAF's greybox fuzzer is its own code, it does
not shell out to `afl-fuzz`.

`literature: []` is intentional and legal. `ci/format_data_consistency.py:44-45`
returns early for an empty list; a non-empty list is checked against CrossRef
(title fuzzy-match >= 85, exact year) and would fail for anything unpublished.
Add the SV-COMP 2027 competition-contribution paper under `supplementary:` (or
`literature:`) once it has a DOI — **after** the competition, in a follow-up MR.

---

## 5. MR 2 — bench-defs: ONE commit, three things

**Repository:** <https://gitlab.com/sosy-lab/sv-comp/bench-defs>
**Commit contents (all three, together):**

1. `benchmark-defs/category-structure.yml` — apply `submission/category-structure.diff`
2. `benchmark-defs/saf.xml` — **generated**, see 5.2
3. `fm-tools` submodule pointer bumped to the fm-tools commit that contains
   `data/saf.yml`

### 5.1 Why the submodule bump must be in the same commit

`bench-defs/.gitlab-ci.yml:17-35` checks out its **pinned** `fm-tools` submodule
(`REQUIRED_SUBMODULES: "scripts benchexec sv-benchmarks fm-tools"`). All three
jobs read tool data from that pinned checkout, not from fm-tools `main`.

What actually breaks, in each direction — I simulated all of these locally:

* **Bump the submodule, forget the `opt_in` hunk** → hard failure.
  `check-categories.py:94-105` `_check_category_benchdef_for_tool_exists`
  iterates *every* verifier returned by
  `verifiers_of_competition(fm_tools, SV-COMP, 2027)` and requires
  `benchmark-defs/<tool>.xml` to exist:
  `"Cannot find benchmark definition for benchmark-defs/saf.xml."`
  And `create-benchdefs.py` will not create it for you — with `saf` absent from
  `opt_in` it prints
  `Skipping saf for language C because: No rundefinitions left in benchmark definition`
  and writes nothing (verified by running the real script).
  Note that `verifiers_of_competition` (`competition-scripts/prepare_tables/utils.py:818-827`
  → `fm_tools/query.py:44-50` → `competition_participation.py:82-90`) does **not**
  filter on labels, so even the inactive Hors-Concours stubs count as verifiers.

* **Add the `opt_in` hunk, forget the submodule bump** → CI goes green and SAF
  silently does not participate. This is the dangerous direction. The failure
  message quoted in the task brief, *"Verifiers listed in category X, but not
  participating"*, comes from `check-categories.py:84-89` and only fires for a
  tool named in some `categories.<Meta>.verifiers:` list. SAF is deliberately in
  **no** meta category, so that message will never appear for SAF. Instead:
  `check-definitions` never selects `saf` (it reads participations from the
  pinned fm-tools), `check-benchdefs-consistency` never regenerates `saf.xml`,
  and `_check_info_consistency` is happy because all 26 opt-in categories do
  exist in meta categories. You would discover the problem only when the
  competition runs nothing for SAF. Treat the bump as non-optional.

* **Land `opt_in` + a hand-written `saf.xml` without the bump** → the next
  unrelated MR that advances the submodule regenerates a *different* `saf.xml`
  and blows up `check-benchdefs-consistency`'s `git diff --exit-code
  benchmark-defs/` (`bench-defs/.gitlab-ci.yml:66`) in someone else's pipeline.

### 5.2 `benchmark-defs/saf.xml` is generated, never hand-written

`benchmark-defs/README.md` says so, and `check-benchdefs-consistency`
(`bench-defs/.gitlab-ci.yml:57-66`) enforces it by regenerating every benchdef
and running `git diff --exit-code benchmark-defs/`.

```bash
cd bench-defs
python3 ./scripts/test/create-benchdefs.py \
    --competition 'SV-COMP 2027' \
    --track 'Verification' \
    --xml-template-directory benchmark-defs/reference/ \
    --category-structure benchmark-defs/category-structure.yml \
    --fm-data fm-tools/data \
    --output benchmark-defs/ \
    --ignore witnessmap
```

Do **not** confuse this with `~/static-analyzer-factory/benchmark-defs/saf.xml`
on the VM. That file is SAF's own plan-203 local benchmark definition; it points
at `../tests/benchmarks/sv-benchmarks/…`, uses the old `ReachSafety-*` task names
and `timelimit="900 s"`, and has nothing to do with the upstream one. It must not
be copied into bench-defs.

### 5.3 What the generated `saf.xml` will look like

I ran the real `create-benchdefs.py` against the proposed `data/saf.yml` and the
patched `category-structure.yml`. Result: 5 rundefinitions, exactly 26 `<tasks>`
blocks, zero `<option>` elements.

```xml
<benchmark tool="saf" displayName="SAF" timelimit="15 min" hardtimelimit="16 min" memlimit="15 GB" cpuCores="4">
  <require cpuModel="Intel Xeon E3-1230 v5 @ 3.40 GHz" />
  <resultfiles>**/witness.*</resultfiles>
  <rundefinition name="SV-COMP27_unreach-call">   <!-- 14 tasks -->
  <rundefinition name="SV-COMP27_no-data-race">   <!--  1 task  -->
  <rundefinition name="SV-COMP27_valid-memsafety"><!--  5 tasks -->
  <rundefinition name="SV-COMP27_no-overflow">    <!--  2 tasks -->
  <rundefinition name="SV-COMP27_termination">    <!--  4 tasks -->
```

`tool="saf"` comes from `benchexec_toolinfo_module`, `displayName="SAF"` from
`name:`. The 15 min / 15 GB / 4 cores limits and the `<resultfiles>` element come
from `benchmark-defs/reference/reference-verifier.xml` — SAF does not get to
choose them.

---

## 6. Reproducing every CI job locally, before you open anything

### 6.1 fm-tools jobs

```bash
git clone https://gitlab.com/sosy-lab/benchmarking/fm-tools.git
cd fm-tools
cp /path/to/submission/fm-tools-saf.yml data/saf.yml   # after filling in section 1

python3 -m venv .venv && . .venv/bin/activate
pip install --requirement ci/requirements.txt
pip install ruamel.yaml habanero httpx rapidfuzz

# check-data-schema  (the year-pinned invocations; add 2027 once upstream does)
./ci/check-data.py --schema data/schema.yml --year 2027 --fm-data data/saf.yml

# check-data-format
./ci/format_data.py --directory ./data/ --check
./ci/format_data_consistency.py
./ci/format_data_sort.py
git diff --exit-code data/

# check-file-conventions
./ci/check_file_conventions.py

# check-file-reuse
podman run --rm -v "$PWD":/data docker.io/fsfe/reuse:3 --root /data lint

# check-valid/installed-required-packages  (inside the competition image!)
podman run --rm -v "$PWD":/w -w /w \
  registry.gitlab.com/sosy-lab/benchmarking/competition-scripts/user:2026 \
  bash -c 'apt-get update >/dev/null;
           for p in clang-18 libc6-dev-i386 libclang-rt-18-dev libffi8 libgcc-s1 \
                    libstdc++6 libtinfo6 libzstd1 llvm-18 zlib1g; do
             apt-cache show -q=0 "$p" >/dev/null || echo "INVALID: $p";
             dpkg --get-selections "$p" | grep -q install || echo "NOT INSTALLED: $p";
           done; echo done'
# repeat with :2027 as soon as that image exists

# smoke-test  (this is the one that actually downloads and runs the archive)
pipx install "fm-weck>=1.5.3"
fm-weck smoke-test --competition-year 2027 data/saf.yml

# check-archives  (downloads from Zenodo by DOI, unzips, loads the tool-info module)
git clone https://gitlab.com/sosy-lab/software/benchexec.git
PYTHONPATH=benchexec ci/check-archives.sh "SV-COMP 2027" saf
```

Note that `ci/check-archives.sh:39-42` skips any DOI already listed in
`ci/check-archives-skip-list.csv`; a fresh DOI is never skipped.

Local verification already done for the draft (on the laptop, against upstream
`main` of 2026-09-13):

* `ci/format_data.py --check` equivalent (ruamel round-trip, `width=4096`,
  `indent(mapping=2, sequence=4, offset=2)`): **PASS**
* `ci/format_data_sort.py` fixed point (techniques + frameworks sorted
  case-insensitively, blank line before `frameworks_solvers` and before
  `literature`): **PASS**
* `validate_property_order`: **order OK**, 0 missing required, 0 extra properties
* `jsonschema` against `data/schema.yml`: **0 errors**
* `validate_no_repetitions_in_techniques`: none
* `check_participation` for 2024/2025/2026/2027 and `check_require_doi`: pass
* the bench-defs label filter selects the SAF participation: **true**
* Not runnable offline: `_check_gitlab_handle` (needs a real handle),
  `smoke-test`, `check-archives` (needs the published DOI).

### 6.2 bench-defs jobs

```bash
git clone https://gitlab.com/sosy-lab/sv-comp/bench-defs.git
cd bench-defs
git submodule update --init --depth 50 scripts benchexec fm-tools
# sv-benchmarks is huge; the CI uses a blobless sparse clone:
git clone --no-checkout --filter=blob:none --depth=1 \
    https://gitlab.com/sosy-lab/benchmarking/sv-benchmarks.git sv-benchmarks
git -C sv-benchmarks sparse-checkout set --no-cone '/*' '!/*/*/' '/*/*/*.yml' '/*/*/*/*.yml'
git -C sv-benchmarks checkout

python3 -m venv .venv && . .venv/bin/activate
pip install hatch coloredlogs yq
pip install fm-tools/lib-fm-tools/python/

# --- the three edits of MR 2 ---
git apply /path/to/submission/category-structure.diff
git -C fm-tools fetch origin && git -C fm-tools checkout <commit-with-data/saf.yml>
./scripts/test/create-benchdefs.py --competition 'SV-COMP 2027' --track 'Verification' \
    --xml-template-directory benchmark-defs/reference/ \
    --category-structure benchmark-defs/category-structure.yml \
    --fm-data fm-tools/data --output benchmark-defs/ --ignore witnessmap
git add benchmark-defs/category-structure.yml benchmark-defs/saf.xml fm-tools

# --- job 1: check-definitions ---
yq --raw-output --slurp 'map( select( .competition_participations[]?
                                      | .competition=="SV-COMP 2027"
                                        and .track=="Verification"
                                        and (.label | length == 0) ) )
                         | sort_by([.input_languages[0], .id]) [] .id' ./fm-tools/data/*.yml \
  | xargs --replace={} ./scripts/test/check-benchdef.py benchmark-defs/{}.xml
#   -> `saf` must appear in that id list. If it does not, your label is wrong
#      or the submodule is not bumped.

# --- job 2: check-category-structure ---
./scripts/test/check-categories.py \
    --tasks-directory sv-benchmarks \
    --category-structure benchmark-defs/category-structure.yml \
    --reference-benchdef-c benchmark-defs/cpachecker.xml \
    --reference-benchdef-java benchmark-defs/jbmc.xml \
    --benchdef-xmls benchmark-defs/ \
    --witnesslint-allowed-missing-categories C.unreach-call.CorrectnessWitnesses,C.unreach-call.ViolationWitnesses \
    --allow-unused sv-benchmarks/c/CorrectnessWitnesses.set,sv-benchmarks/c/ViolationWitnesses.set,sv-benchmarks/Invalid-TaskDefs.set,sv-benchmarks/c/Unused_DeviceDriversLinux64Regression.set

# --- job 3: check-benchdefs-consistency ---
for T in 'Verification' \
         'Validation of Violation Witnesses v1' 'Validation of Violation Witnesses v2' \
         'Validation of Correctness Witnesses v1' 'Validation of Correctness Witnesses v2' \
         'Validation of SV-LIB Witnesses v1'; do
  ./scripts/test/create-benchdefs.py --competition 'SV-COMP 2027' --track "$T" \
      --xml-template-directory benchmark-defs/reference/ \
      --category-structure benchmark-defs/category-structure.yml \
      --fm-data fm-tools/data --output benchmark-defs/ --ignore witnessmap
done
git diff --exit-code benchmark-defs/
```

All three must be green **with the submodule bump staged**, otherwise you are
testing a different tree than CI will.

### 6.3 What I verified about the diff itself

* `git apply --check` and `patch -p1 --dry-run` against
  `benchmark-defs/category-structure.yml` @ `main`: both clean.
* All 26 opt-in categories exist in some meta category's `categories:` list, so
  `check-categories.py:57-64` `_check_info_consistency` does not fire
  "Category used in opt_in, but missing in meta categories".
* All 26 are three-part names, so `create-benchdefs.py:359-367`
  `get_category_name` does not raise `ValueError("Non-leaf category name")`.
* `saf` appears in zero `categories.<Meta>.verifiers:` / `validators:` lists, so
  SAF joins no meta category and appears in no Overall ranking.
* The reference template `benchmark-defs/reference/reference-verifier.xml`
  contains a `<tasks>` block for every one of the 26 (55 blocks in total), so the
  opt-in produces all 26 and nothing is silently dropped.
* Running the real `create-benchdefs.py` end to end produced the `saf.xml`
  described in 5.3.

---

## 7. Ordering summary with dates

| When | Action | Blocked by |
| --- | --- | --- |
| **ASAP** (weeks of lead time) | Open the BenchExec PR adding `benchexec/tools/saf.py` | nothing |
| ASAP | Land the digit-first `--version` fix in SAF | nothing |
| after the `--version` fix | Build the final archive, publish the Zenodo record, note the **version** DOI | archive build |
| after Zenodo | Open fm-tools MR 1 with `data/saf.yml` | MR 0 merged (CI imports `benchexec.tools.saf`), DOI minted |
| by **2026-10-08** | Both MR 1 and MR 2 must **exist** (registration = merge requests created) | — |
| after MR 1 is merged | Open bench-defs MR 2 with the submodule bumped to that merge commit | MR 1 merged |
| by **2026-10-20** | All CI green on both MRs | everything above |

If MR 0 stalls, MR 1's `check-archives-*` and `smoke-test` jobs cannot pass, and
MR 2 cannot even be written correctly. Open MR 0 first and chase it.

---

## 8. Things that will silently go wrong

1. **A label on the 2027 participation.** Green CI, zero tasks generated, SAF
   never runs. See 4.2.
2. **`opt_in` without the submodule bump.** Green CI, SAF not a participant. See 5.1.
3. **Concept DOI instead of version DOI.** `zenodo.py` raises only at download
   time, i.e. in `check-archives-*`, long after everything else looks fine.
4. **Two files in the Zenodo record** (e.g. the zip plus a README). Hard fail in
   `zenodo.py:56-61`.
5. **Re-formatting `data/saf.yml` by hand.** `check-data-format` compares against
   a `ruamel.yaml` round-trip *and* re-sorts `techniques`/`frameworks_solvers`
   and re-inserts the blank lines before `frameworks_solvers` and `literature`.
   Always finish with `./ci/format_data.py --file data/saf.yml` (no `--check`)
   followed by `./ci/format_data_sort.py`.
6. **`--version` starting with the tool-info `name()`.** `check_archive.py:279-281`
   fails with "tool 'saf' is part of its own version number". Today SAF escapes
   only because `name()` returns `"SAF"` while clap prints `saf`.
7. **Assuming the CI year jobs already say 2027.** As of 2026-09-13 the fm-tools
   pipeline pins 2024/2025/2026 (`check-data-schema`, `check-*-required-packages-*`,
   `check-archives-*`, `smoke-test`'s `YEAR: "2026"`), while bench-defs already
   targets `SV-COMP 2027`. Upstream will bump fm-tools; re-read
   `fm-tools/ci/.gitlab-ci.yml` before opening MR 1 in case a 2027 job adds a
   requirement that does not exist today.
