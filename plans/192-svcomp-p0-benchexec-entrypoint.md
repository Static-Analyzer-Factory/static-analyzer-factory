# Plan 192: SV-COMP P0 — the BenchExec entry point (tracer bullet)

**Status:** design / awaiting approval
**Date:** 2026-08-09
**Branch:** `svcomp`
**Increment from:** plan 191 (strategy). This is the first *implementation* plan.
**Reference competition:** SV-COMP 2026 rules (2027 unreleased).
**Scope:** Make SAF *runnable and scoring* by SV-COMP in `C.FalseOverall` — the thinnest
end-to-end vertical slice, **unreach-call first**, test-first. Implements plan 191 P0.1–P0.8
plus P1.1–P1.3, structured as three slices. Everything builds/runs on the VM
(`ubuntu@cd-vm-15-ai-vm`, Docker-only), never on the laptop.

---

## 0. Executive summary

`saf verify --property <p.prp> --data-model {ILP32,LP64} [--witness <w.yml>] <file.c|.i>`
becomes a new subcommand of the `saf` binary that is **blind to the expected verdict** and prints
exactly one of `true` / `false(<prop>)` / `unknown` on stdout. It compiles C in-tool, runs the
*reconnected* in-process analyzer (the sophisticated `analyze_property` path, retiring the
`svf_assert` subprocess oracle for SV-COMP), and — for a proven-reachable `reach_error` — writes a
YAML witness 2.0 violation witness. Per the anti-−32 posture it emits **only `false`/`unknown`** in
this plan (every analyzer `TRUE`, including "no `reach_error` present", maps to `unknown`); the
sound-TRUE side is deferred to plan 191 P2.

**The one non-obvious structural decision:** the engine to reconnect (`analyze_property`) lives in
`saf-bench`, but `saf-bench` **depends on** `saf-cli` (`saf-bench/Cargo.toml:23`), so `saf-cli`
(home of `verify`) cannot call it without a dependency cycle. We therefore **extract the SV-COMP
analysis engine into a new `saf-svcomp` crate** that sits below both. This is slice 0 and is a pure,
behavior-preserving refactor.

### Corrections to plan 191's audit (found by reading the code this session)

| Plan 191 claim | Reality in code | Consequence |
|---|---|---|
| CLI `tracing` fmt layer writes to **stdout** (`main.rs:12`) | fmt layer defaults to **stderr** (no `.with_writer` override); `RUST_LOG` ignored (hardcoded `info`) | P0.8 "route tracing to stderr" already satisfied. Only remaining stdout-hygiene: `verify` prints *only* the verdict. |
| Z3 wall-clock timeout is a latent −16 (→ FALSE) landmine | timeout → `SatResult::Unknown` → `FeasibilityResult::Unknown` → `PropertyResult::Unknown`; **FALSE requires an explicit `Sat`** | Not a −16 landmine. It is a *cross-machine TRUE↔UNKNOWN* nondeterminism hazard (fast box decides, slow box → UNKNOWN). Fix = Z3 `rlimit` (deterministic). |
| Reconnecting the analyzer is "the single highest-leverage code change" (implying effort) | `analyze_property(&Property, &AirModule, &PropertyAnalysisConfig)` builds the **entire** pipeline lazily via `OnceCell` | Reconnection is ~trivial *once the crate boundary is fixed*: load module, call one function. |
| `__VERIFIER_assume` "unmodeled = false alarms + can't prove TRUE" | Confirmed: **completely ignored** in absint *and* the Z3 path engine | Real **FALSE-soundness** hazard — ignoring `assume(c)` lets an infeasible error path look reachable → false alarm (−16). Must model in slice 1. |

### Decisions locked with the user (2026-08-09)

- Slice-1 property = **unreach-call only** (canonical `reach_error` verdict path; cleanest witness).
- Witness sequencing = **target waypoint + metadata first**, enrich with assumption/branching
  waypoints after (slice 1b-a then 1b-b).
- Validation = **witnesslint gate now**; provision **CPAchecker** to measure confirmation-% in 1b-b.
- TRUE posture = **emit only `false`/`unknown`** in this plan (zero −32 exposure).

---

## 1. Architecture: the `saf-svcomp` crate extraction (slice 0)

### 1.1 Why (the dependency cycle)

Current DAG: `saf-core ← saf-frontends ← saf-analysis ← saf-cli ← saf-bench`
(`saf-bench/Cargo.toml:23` = `saf-cli { path = "../saf-cli" }`, to share `bench_types`).
The SV-COMP engine (`crates/saf-bench/src/svcomp/{property,fast_paths,summaries,witness}.rs` +
`Property`/`DataModel` in `task.rs`) is therefore **above** `saf-cli`. `verify` in `saf-cli` cannot
reach it.

### 1.2 Approaches considered

- **(A) New `saf-svcomp` crate below both (RECOMMENDED).** Clean DAG, single source of truth,
  matches the strategic framing (SV-COMP is a first-class capability, not a benchmark detail).
  Cost: move ~4 files + two enums; update `saf-bench` imports. Mechanical.
- **(B) Put `verify` in `saf-bench`.** Rejected — BenchExec must invoke the *one* self-contained
  `saf` binary (`saf-cli`); `saf-bench` is a separate binary. The competition tool is `saf`.
- **(C) Duplicate the engine into `saf-cli`.** Rejected — two sources of truth, drift.
- **(D) Move the engine into `saf-analysis`** (already a `saf-cli` dep). Lighter (no new crate) but
  pollutes the general, reusable analysis library with SV-COMP application semantics (`reach_error`
  names, `.prp` parsing, SV-COMP witnesses). Prefer (A)'s clean boundary.

**Recommendation: (A).** New DAG:
`saf-core ← saf-frontends ← saf-analysis ← saf-svcomp ← saf-cli ← saf-bench`.

### 1.3 Extraction manifest (what moves, what stays)

**New crate `crates/saf-svcomp`** — *pure engine: given `AirModule` + `Property` + config →
`PropertyResult` (+ witness). No subprocess, no `expected_verdict`, no scoring.* Depends on
`saf-core`, `saf-frontends`, `saf-analysis` (features `z3-solver`, `analysis-mta`).

Moved from `saf-bench/src/svcomp/`:
- `property.rs` → `analyze_property`, `analyze_property_with_context` (make **`pub`** — currently
  module-private, `property.rs:397`), `AnalysisContext`, all `analyze_*`, `PropertyAnalysisConfig`,
  `PropertyResult`, `build_c_library_specs`/`build_analyzed_specs`.
- `fast_paths.rs`, `summaries.rs` (only used by `property.rs`).
- `witness.rs` (legacy GraphML — kept for the `valid-memtrack` 1.0 exception) + the **new**
  `witness_yaml.rs` (slice 1b).
- `Property`, `DataModel` enums from `task.rs` → new `saf-svcomp/src/property.rs` module
  (`Property::from_prp` added; `Property::from_property_file` filename matcher retired from the
  verify path but kept for the harness's YAML task loader).

**Stays in `saf-bench`** (the benchmark/self-grading harness): `SvCompRunner`, `discover_tasks`,
`SvCompTask` YAML parsing incl. `expected_verdict`, `scoring.rs` (`SvCompVerdict`, `SvCompOutcome`,
points), and the `run_bench_mode`/`runner.rs` subprocess machinery (**still used by PTABen, Juliet,
CruxBC** — do not delete). `saf-bench` gains a dep on `saf-svcomp` and re-exports its types so
existing call sites keep compiling. `SvCompTask` imports `Property`/`DataModel` from `saf-svcomp`.

### 1.4 Slice-0 acceptance (behavior-preserving)

`make test` green on the VM; `make test-svcomp`, `make test-juliet CWE=…`, and a PTABen smoke run
produce **identical** results to pre-extraction (no verdict/score change). This is the guardrail
that the refactor changed nothing.

---

## 2. The `verify` I/O contract (P0.1, P0.8, P7.3)

### 2.1 CLI surface (`saf-cli/src/commands.rs`, `main.rs`)

Add `Verify(VerifyArgs)` to `Commands` (`commands.rs:333`, after `Run`); dispatch
`Commands::Verify(args) => commands::verify(&args)` (`main.rs:34`). Handler
`pub fn verify(args: &VerifyArgs) -> anyhow::Result<()>`.

```rust
#[derive(Args)]
pub struct VerifyArgs {
    /// The C program under verification (.c or preprocessed .i). Exactly one.
    #[arg(required = true)]
    pub input: PathBuf,
    /// SV-COMP property file (.prp). Parsed for the CHECK(...LTL...) form — NOT the filename.
    #[arg(long, required = true)]
    pub property: PathBuf,
    /// Data model; selects clang -m32/-m64 and the LLVM target.
    #[arg(long, value_enum, default_value_t = CliDataModel::Lp64)]
    pub data_model: CliDataModel,
    /// Where to write the violation witness (YAML 2.0). BenchExec passes ${witness}.
    #[arg(long, default_value = "witness.yml")]
    pub witness: PathBuf,
    /// Optional full machine-readable report (JSON) to a file. stdout stays verdict-only.
    #[arg(long)]
    pub output: Option<PathBuf>,
}
```

`CliDataModel { Ilp32, Lp64 }` (new `ValueEnum`) → `saf_svcomp::DataModel`.

### 2.2 stdout / stderr / exit contract (hard rules)

- **stdout = exactly one line**, one of: `true` · `false(unreach-call)` · `unknown`. Nothing else.
  (tracing already on stderr; the handler must use `eprintln!` for all diagnostics and `println!`
  only for the verdict. ≤2 MB trivially.)
- **Exit code 0 whenever a verdict is printed** (including `unknown` and `false`). Any internal
  error (missing clang, parse failure, panic caught) → print `unknown` to stdout + detail to stderr
  + exit 0. `verify` **never** aborts to a nonzero/crash that BenchExec would score as ERROR.
- Full JSON report (verdict, timings, witness path, reason) → `--output` file only.

### 2.3 Determinism pins, executed at handler entry (P0.8)

1. `std::env::remove_var("SAF_PTA_FIELD_MINTING")` and `remove_var("SAF_DECOMPOSE_POINTER_ARRAYS")`
   (both are `.is_ok()` presence-reads: `cg_refinement.rs:297`, `mapping.rs:131`). Log to stderr if
   either was set. This pins both verdict-affecting toggles **OFF** (the sound, plan-190-default
   config) regardless of the environment BenchExec provides. *(Follow-up, non-blocking: convert
   these reads to explicit `Config` fields so the pin is structural, not env-scrubbing.)*
2. Load specs **binary-relative only**: resolve `current_exe()/../share/saf/specs` and pass just
   that to `SpecRegistry::load_from(&[path])`, bypassing the `$HOME` / CWD `./saf-specs` /
   CWD `./share/saf/specs` / `$SAF_SPECS_PATH` search order (`registry.rs:222-256`) that varies by
   machine/invocation. (Slice 1: audit whether the unreach-call path loads YAML specs at all — it
   primarily uses the programmatic `build_analyzed_specs`; memsafety/no-overflow slices will need
   this pin.)

---

## 3. Slice plan

### Slice 0 — extraction + skeleton + tool-info (no verdict yet)

| # | Change | Files (from the code map) |
|---|---|---|
| 0.1 | Extract `saf-svcomp` crate (§1.3); keep `saf-bench` green | new `crates/saf-svcomp/*`; edit `saf-bench/src/svcomp/{mod,task,scoring}.rs`, `saf-bench/Cargo.toml`, workspace `Cargo.toml` |
| 0.2 | `.prp` parser `Property::from_prp(&str) -> Option<Property>` (or `Vec<Property>` for memsafety's 3 CHECKs) — match the `LTL(...)` body, not the filename | `saf-svcomp/src/property.rs` |
| 0.3 | `saf verify` skeleton: `VerifyArgs`, dispatch, determinism pins (§2.3), `.prp` parse, print `unknown` for all; blind (no `expected_verdict` in this path) | `saf-cli/src/commands.rs:327-346`, `main.rs:32-41` |
| 0.4 | BenchExec tool-info `benchexec/tools/saf.py` (cmdline builder, stdout→result, `--version`) + `smoketest.sh` (exit 0) | new `benchexec/tools/saf.py`, `smoketest.sh` |

**`.prp` forms parsed** (map LTL body → `Property`):
`G ! call(reach_error())` → `UnreachCall`; `G valid-free|valid-deref|valid-memtrack` →
`ValidMemsafety`; `G valid-memcleanup` → `ValidMemcleanup`; `G ! overflow` → `NoOverflow`;
`G ! data-race` → `NoDataRace`; `F end` → `Termination`. Unrecognized → `None` → verdict `unknown`.

**TDD (slice 0):** unit tests for all 6 `.prp` forms (+ malformed → `None`); `assert_cmd` tests for
the CLI contract (verdict-only stdout, exit 0, `--property` required); a pytest for
`saf.py`'s output→result mapping (`true`/`false(...)`/`unknown`/garbage → UNKNOWN); `smoketest.sh`
exits 0 in-container. **Acceptance:** slice-0 refactor guardrail (§1.4) + these tests green.

### Slice 1 — unreach-call end-to-end verdict (`false`/`unknown`)

| # | Change | Files |
|---|---|---|
| 1.1 | In-tool C→IR: shell `clang-18 -g -emit-llvm -c -Wno-everything {-m32\|-m64} -include <stubs.h> -I<srcdir> <input> -o tmp.bc` then `opt-18 -passes=mem2reg tmp.bc -o tmp.bc`; ingest via `AnalysisDriver::ingest(&[tmp], Llvm)`. `-g` preserves spans/`#line`. Temp files in a `tempfile::tempdir()`. Bundle `sv-comp-stubs.h` at `share/saf/sv-comp-stubs.h` (binary-relative). | `saf-cli/src/commands.rs` (verify handler); reuse `scripts/{compile-svcomp.sh,sv-comp-stubs.h}` logic; `driver.rs:299` `ingest` |
| 1.2 | Reconnect analyzer (P0.5): `saf_svcomp::analyze_property(&Property::UnreachCall, &module, &cfg)` → map `False{..}` → `false(unreach-call)`; `True`/`Unknown` → `unknown`. In-process; svf_assert path unused for SV-COMP (kept for PTABen `ae_assert`). Also switch `saf-bench` `run_task` (`mod.rs:225`) to call the same in-process engine, retiring `runner::run_saf_bench`+`bench_result_to_verdict` for SV-COMP. | `saf-cli` verify; `saf-bench/src/svcomp/mod.rs:225-274,335-359` |
| 1.3 | Semantics (P0.6): **model `__VERIFIER_assume(c)`** in the Z3 path engine — add `c` as a path constraint so infeasible error paths are pruned (prevents false FALSE). Extend the nondet interval specs (`property.rs:333-337`) to the full `__VERIFIER_nondet_*` set (add `_long/_ulong/_bool/_size_t/_uchar/_ushort/_longlong/_ulonglong` etc.; currently only int/uint/short/char). `reach_error`/`__VERIFIER_error` already matched (`property.rs:465`). | `saf-svcomp/src/{property.rs}`, `saf-analysis/src/z3_utils/{reachability,assertions}.rs` |
| 1.4 | Determinism (P0.8): switch Z3 from ms wall-clock `set_u32("timeout", ms)` to deterministic **`rlimit`** at `solver.rs:108,167`; keep the invariant `Z3 Unknown → PropertyResult::Unknown` (never FALSE). | `saf-analysis/src/z3_utils/solver.rs:104-190` |
| 1.5 | Budget/scheduler (P0.4): deterministic step-bounds (`max_iterations`/`max_paths`/`max_guards`) sized so worst-case CPU < 900 s; **wall-clock watchdog**: run `analyze_property` on a worker thread; if a deadline (`--timeout`, default 850 s) elapses, print `unknown` + exit 0 (graceful UNKNOWN before BenchExec SIGKILL). Watchdog only ever yields UNKNOWN (sound). | `saf-cli` verify handler; existing bounds in `property.rs:93-117`, `reachability.rs`, `assertions.rs:192` |
| 1.6 | Cross-check every FALSE (P1.2): unreach-call FALSE already comes from a Z3-*feasible* (`Sat`) path. Guard: emit FALSE only if the `reach_error` target span is resolvable (witness constructible); else downgrade to `unknown`. Concrete replay deferred. | `saf-svcomp/src/property.rs` |

**TDD (slice 1) fixtures** (`tests/programs/c/svcomp/` + `.prp`), each run **blind** then audited:
- `unreach_true_simple.c` — `reach_error` present but unreachable → expect `unknown` (we never
  emit TRUE).
- `unreach_false_direct.c` — `main` calls `reach_error()` unconditionally → expect
  `false(unreach-call)`.
- `unreach_false_nondet.c` — `if (__VERIFIER_nondet_int() > N) reach_error()` → `false`.
- `assume_guards_error.c` — `assume(x==0); if (x!=0) reach_error()` → **must not** be `false`
  (soundness of `assume`) → `unknown`.
- `unreach_false_interproc.c` — error in a callee reachable from `main` → `false` (exercises
  `compute_error_summaries`).

**Acceptance (slice 1):** on a curated ~30–50 task slice of `sv-benchmarks`
`unreach-call`/ReachSafety, `saf verify` produces blind verdicts; **ZERO unsound TRUE** (there can
be none — we emit no TRUE) and **ZERO false alarms** on the `expected_verdict==true` subset (audited
*after* the blind run). Re-running a task yields a byte-identical verdict (determinism).

### Slice 1b — YAML violation witness + confirmation (P1.1)

- **1b-a (target + metadata):** new `saf-svcomp/src/witness_yaml.rs` emitting witness format 2.0:
  a `violation_sequence` whose final segment holds a single `target` waypoint at the `reach_error`
  call `(file, line, column)` — from `Instruction.span` (`air.rs:656`, has line **and** col) +
  `AirModule.source_files` path (`span.rs:130`). Metadata: `producer`, `sha256`
  `input_file_hashes` (of the un-preprocessed input given to `verify`), `specification` (from the
  `.prp`), `architecture`/`data_model`, and a **fixed/omitted** `creation_time` (replace
  `SystemTime::now()` at `witness.rs:238` → byte-stable). Uses `serde_yaml 0.9` + `sha2 0.10`
  (already in-tree). Thread the structured `reach_error` span out of `analyze_unreachability`
  (currently stringified to `block_{n}` at `property.rs:536`). **Gate:** every witness passes
  `witnesslint` (provisioned in the image).
- **1b-b (enrich + confirm):** implement Z3-**model extraction** (nondet var → concrete value) in
  the reachability engine; emit `assumption` waypoints fixing nondet inputs + `branching`
  waypoints for control decisions, so bounded validators replay within 90 s. Provision
  **CPAchecker** in the VM image; measure and report the **confirmation %** over the slice.

**TDD (slice 1b):** golden-file test that a known `false` fixture emits a byte-stable witness that
`witnesslint` accepts; then the CPAchecker confirmation-rate harness (1b-b).

---

## 4. Soundness & determinism invariants (must hold; mapped to code)

1. **Never an unsound TRUE.** This plan emits no `true`; every analyzer `TRUE` → `unknown`
   (verify handler mapping). (−32 exposure = 0.)
2. **Never a wall-clock verdict.** Verdicts derive only from deterministic step-bounds + Z3
   `rlimit` (1.4). The wall-clock watchdog (1.5) may only produce `unknown`, never `true`/`false`.
3. **Timeout/crash/OOM → `unknown`, exit 0** (§2.2, 1.5) — graceful under BenchExec SIGKILL.
4. **`assume` is honored** before any FALSE (1.3) — no infeasible-path false alarms.
5. **Verdict = deterministic function of (program, property, envelope).** Env toggles pinned (2.3),
   specs binary-relative (2.3), no RNG / no `HashMap`-iteration / no `SystemTime` in the verdict
   path (confirmed by the map). Witness bytes deterministic (fixed timestamp, 1b-a).
6. **Every FALSE is cross-checked** (Z3-feasible + witness-constructible) or downgraded (1.6).

---

## 5. BenchExec tool-info + smoketest (P0.2)

`benchexec/tools/saf.py` (subclass `benchexec.tools.template.BaseTool2`):
- `executable()` → the `saf` binary; `version(exe)` → parse `saf --version`; `name()` → `"SAF"`.
- `cmdline(exe, options, task, ...)` → `[exe, "verify", "--property", task.property_file,
  "--data-model", <from task/options>, "--witness", "${witness}" or the passed path, task.input]`.
- `determine_result(run)` → map stdout: `false(...)` → `RESULT_FALSE_PROP` (+ property);
  `true` → `RESULT_TRUE_PROP`; else / unparsable / nonzero → `RESULT_UNKNOWN`.
`smoketest.sh`: compile+verify a bundled trivial `false` task, assert stdout `false(unreach-call)`,
exit 0. Runs offline in-container.

---

## 6. VM workflow (all builds/experiments here; never the laptop)

```bash
# one-time
ssh ubuntu@cd-vm-15-ai-vm
cd <repo>/static-analyzer-factory
git checkout svcomp
git submodule update --init tests/benchmarks/sv-benchmarks
make shell            # confirm toolchain (clang-18, opt-18, libLLVM-18 in container)
make build            # confirm image builds

# per-change loop (main agent only; subagents never call make)
make fmt && make lint
make test                                   # full Rust+Python
docker compose run --rm dev sh -c 'cargo nextest run -p saf-svcomp'   # focused
make test-svcomp                            # harness regression guardrail (slice 0)
```

Clang/opt/libLLVM live only in the container (LLVM 18). `verify` shells to `clang-18`/`opt-18`
inside that environment. Native self-contained ZIP packaging (plan 191 P0.7) is **out of scope**
here — a Docker dev build on the VM is sufficient for P0 measurement.

---

## 7. Acceptance criteria & evidence to collect

1. **Blind verdicts, zero unsound TRUE.** Run `saf verify` on a ~30–50 task `unreach-call` slice
   *blind*; audit `expected_verdict` **after**. Report: #`false` / #`unknown`, and confirm zero
   false alarms on the `expected==true` subset. (No TRUE emitted ⇒ −32 impossible.)
2. **Witness-confirmation rate.** % of emitted `false` witnesses confirmed by CPAchecker
   (slice 1b-b), reported as a fraction of FALSE verdicts.
3. **Determinism.** Re-run the slice; verdicts and witness bytes identical. Confirm both env
   toggles are unset at runtime (logged) and specs resolved binary-relative.
4. **Harness guardrail.** `make test`, `make test-svcomp`, `make test-juliet` green; PTABen
   unchanged (extraction behavior-preserving).
5. **PROGRESS.md** updated (Plan 192 entry + Next Steps + Session Log).

---

## 8. Out of scope (deferred)

- Sound TRUE side + correctness witnesses (plan 191 P2.1/P2.7); memsafety & no-overflow verify
  slices; termination/data-race; portfolio/scheduler beyond the single-phase budget; native ZIP
  (P0.7); fm-tools YAML / Zenodo / `category-structure.yml` MR; any LLM/name-hint work (P3).
- `valid-memtrack` GraphML 1.0 exception (kept available via the moved legacy `witness.rs`, not
  wired).

---

## 9. Risks & open questions

- **Extraction churn.** Mitigated by the behavior-preserving guardrail (§1.4) and doing 0.1 first,
  alone, verified green before any verify code.
- **`opt -passes=mem2reg` availability / version.** Container ships LLVM 18 (`opt-18`); confirm in
  `make shell` during slice 1.1. Fallback: run mem2reg in-process via inkwell's pass manager.
- **`.i` + `-include`.** Preprocessed `.i` with `-include stubs.h` — stubs are decls only, so
  prepending is safe; verify in 1.1.
- **Witness confirmation rate may be low with target-only waypoints** on nondet-driven tasks — this
  is *why* 1b-b (assumption/branching waypoints from the Z3 model) exists; 1b-a just proves the
  pipeline and confirms unconditional-reach tasks.
- **Z3 `rlimit` calibration.** `rlimit` units ≠ ms; pick a value giving comparable coverage without
  nondeterminism; measure on the slice.

---

## 10. Ordered task checklist

- [ ] **0.1** Extract `saf-svcomp` crate; `saf-bench` re-exports; VM guardrail green (§1.4).
- [ ] **0.2** `.prp` parser + unit tests (6 forms).
- [ ] **0.3** `saf verify` skeleton (args, dispatch, determinism pins, prints `unknown`) + CLI tests.
- [ ] **0.4** `benchexec/tools/saf.py` + `smoketest.sh` + pytest.
- [ ] **1.1** In-tool clang+mem2reg → ingest; bundle stubs binary-relative.
- [ ] **1.2** Reconnect `analyze_property` in verify + in `saf-bench` `run_task`.
- [ ] **1.3** Model `__VERIFIER_assume`; complete nondet interval specs.
- [ ] **1.4** Z3 `rlimit` (deterministic); keep Unknown→UNKNOWN.
- [ ] **1.5** Step-bounds + wall-clock watchdog → graceful `unknown`.
- [ ] **1.6** FALSE cross-check / downgrade guard.
- [ ] **1.1–1.6 fixtures** + blind-run acceptance (§7.1, §7.3).
- [ ] **1b-a** YAML witness 2.0 (target + metadata, byte-stable) + witnesslint gate.
- [ ] **1b-b** Z3 model extraction → assumption/branching waypoints; CPAchecker confirmation %.
- [ ] Update `plans/PROGRESS.md`.
