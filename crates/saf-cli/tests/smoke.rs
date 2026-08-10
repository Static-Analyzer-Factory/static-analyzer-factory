use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

// ---------------------------------------------------------------------------
// Help command tests
// ---------------------------------------------------------------------------

#[test]
fn help_succeeds_and_shows_subcommands() {
    cargo_bin_cmd!("saf")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("index"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("query"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("schema"));
}

#[test]
fn help_overview_shows_commands_section() {
    cargo_bin_cmd!("saf")
        .args(["help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("COMMANDS"))
        .stdout(predicate::str::contains("HELP TOPICS"));
}

#[test]
fn help_checkers_shows_memory_leak() {
    cargo_bin_cmd!("saf")
        .args(["help", "checkers"])
        .assert()
        .success()
        .stdout(predicate::str::contains("memory-leak"))
        .stdout(predicate::str::contains("BUILT-IN SVFG CHECKERS"));
}

#[test]
fn help_pta_shows_andersen() {
    cargo_bin_cmd!("saf")
        .args(["help", "pta"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Andersen"))
        .stdout(predicate::str::contains("ANALYSIS VARIANTS"));
}

#[test]
fn help_unknown_topic_fails() {
    cargo_bin_cmd!("saf")
        .args(["help", "nonexistent-topic"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown help topic"));
}

// ---------------------------------------------------------------------------
// Schema command tests
// ---------------------------------------------------------------------------

#[test]
fn schema_shows_checkers_section() {
    cargo_bin_cmd!("saf")
        .args(["schema"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checkers:"));
}

#[test]
fn schema_json_starts_with_brace() {
    cargo_bin_cmd!("saf")
        .args(["schema", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"));
}

#[test]
fn schema_checkers_lists_checker_names() {
    cargo_bin_cmd!("saf")
        .args(["schema", "--checkers"])
        .assert()
        .success()
        .stdout(predicate::str::contains("memory_leak"))
        .stdout(predicate::str::contains("null_deref"));
}

// ---------------------------------------------------------------------------
// LLVM-dependent commands (ignored — require Docker + LLVM 18)
// ---------------------------------------------------------------------------

/// `saf run` requires LLVM 18 to ingest bitcode, which is only
/// available inside the Docker dev container.  Run via `make test`.
#[test]
#[ignore]
fn run_on_fixture_succeeds() {
    cargo_bin_cmd!("saf")
        .args(["run", "tests/fixtures/llvm/e2e/simple.ll"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// verify command tests (SV-COMP blind entry point, plan 192)
// ---------------------------------------------------------------------------

/// Writes a throwaway `.prp` file with the given contents and returns the handle
/// (kept alive by the caller so the path stays valid during the command run).
fn write_prp(contents: &str) -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().expect("create temp .prp");
    write!(f, "{contents}").expect("write .prp");
    f
}

/// `verify` appears in the top-level help.
#[test]
fn help_lists_verify() {
    cargo_bin_cmd!("saf")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("verify"));
}

/// `saf verify` prints exactly one verdict line on stdout (nothing else) and
/// exits 0. The slice-0 skeleton always emits the safe `unknown`. This locks the
/// verdict-only stdout contract required by `BenchExec`.
#[test]
fn verify_prints_verdict_only_on_stdout() {
    let prp = write_prp("CHECK( init(main()), LTL(G ! call(reach_error())) )");
    cargo_bin_cmd!("saf")
        .args([
            "verify",
            "--property",
            prp.path().to_str().unwrap(),
            "--data-model",
            "LP64",
            "dummy.c",
        ])
        .assert()
        .success()
        .stdout("unknown\n");
}

/// The property file is a required parameter.
#[test]
fn verify_requires_property() {
    cargo_bin_cmd!("saf")
        .args(["verify", "dummy.c"])
        .assert()
        .failure();
}

/// `--data-model` accepts the SV-COMP spellings ILP32 / LP64.
#[test]
fn verify_accepts_ilp32_data_model() {
    let prp = write_prp("CHECK( init(main()), LTL(G ! call(reach_error())) )");
    cargo_bin_cmd!("saf")
        .args([
            "verify",
            "--property",
            prp.path().to_str().unwrap(),
            "--data-model",
            "ILP32",
            "dummy.c",
        ])
        .assert()
        .success()
        .stdout("unknown\n");
}

// ---------------------------------------------------------------------------
// verify end-to-end (slice 1) — compile C → analyze → real unreach-call verdict.
//
// These require clang-18/opt-18 + LLVM 18 (only in Docker), so they are
// #[ignore]d and run via `make test`. Paths are workspace-root-relative (the CWD
// under `make test`), matching `run_on_fixture_succeeds` above.
// ---------------------------------------------------------------------------

/// Resolve a workspace-root-relative path to an absolute one. Tests run with
/// CWD = the crate dir, so relative paths handed to the `saf` binary won't
/// resolve; `CARGO_MANIFEST_DIR` (this crate) is two levels below the root.
fn ws(rel: &str) -> String {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/saf-cli -> crates
    p.pop(); // crates -> workspace root
    p.join(rel).to_string_lossy().into_owned()
}

fn unreach_prp() -> String {
    ws("tests/programs/c/svcomp/unreach-call.prp")
}

fn svcomp_fixture(name: &str) -> String {
    ws(&format!("tests/programs/c/svcomp/{name}"))
}

/// Run `saf verify <fixture>` against the unreach-call property, asserting exit 0,
/// and return the `Assert` so the caller can lock the exact stdout verdict line.
fn verify_unreach(fixture: &str) -> assert_cmd::assert::Assert {
    let prp = unreach_prp();
    let fx = svcomp_fixture(fixture);
    cargo_bin_cmd!("saf")
        .args([
            "verify",
            "--property",
            prp.as_str(),
            "--data-model",
            "LP64",
            fx.as_str(),
        ])
        .assert()
        .success()
}

/// `main()` calls `reach_error()` unconditionally → `false(unreach-call)`.
#[test]
#[ignore]
fn verify_false_direct() {
    verify_unreach("unreach_false_direct.c").stdout("false(unreach-call)\n");
}

/// Nondet-guarded reachable error → `false(unreach-call)` via slice-1c concrete
/// replay. must-reach yields `unknown` (the error is behind a branch), but the
/// Z3 model (`x = 6` satisfies `x > 5`) seeds a native run that pins
/// `__VERIFIER_nondet_int` to `6` and actually reaches `reach_error` — an
/// irrefutable, sound FALSE.
#[test]
#[ignore]
fn verify_nondet_guarded_reproduces_false() {
    verify_unreach("unreach_false_nondet.c").stdout("false(unreach-call)\n");
}

/// Error in a callee reachable from main → `false(unreach-call)` (interprocedural,
/// requires aggressive/conservative=false).
#[test]
#[ignore]
fn verify_false_interproc() {
    verify_unreach("unreach_false_interproc.c").stdout("false(unreach-call)\n");
}

/// `reach_error()` present but provably unreachable → we never emit `true`, so
/// the verdict is `unknown`.
#[test]
#[ignore]
fn verify_true_simple_is_unknown() {
    verify_unreach("unreach_true_simple.c").stdout("unknown\n");
}

/// `__VERIFIER_assume(x == 0)` makes the `x != 0` error path infeasible — must
/// NOT be a false alarm. The verdict is `unknown` (never `false`).
#[test]
#[ignore]
fn verify_assume_guarded_error_is_not_false() {
    verify_unreach("assume_guards_error.c").stdout("unknown\n");
}

// ---------------------------------------------------------------------------
// Over-approx false-alarm shapes (slice 1c soundness): the Z3 path engine
// PROPOSES each as a FALSE candidate, but concrete native replay does NOT reach
// reach_error, so the verdict is `unknown`. The stderr assertion confirms a
// candidate was actually enumerated and rejected by replay (guarding against a
// trivial `unknown` from, e.g., a compile failure).
// ---------------------------------------------------------------------------

/// Bit-arithmetic false alarm (integerpromotion-2 shape): Z3 models `x & 0xFF`
/// as a fresh variable and proposes FALSE, but the real run computes
/// `0 & 0xFF == 0 != 256` → `unknown`.
#[test]
#[ignore]
fn verify_bitarith_false_alarm_is_unknown() {
    verify_unreach("false_alarm_bitarith.c")
        .stdout("unknown\n")
        .stderr(predicate::str::contains("candidate(s) enumerated"));
}

/// Pointer-identity false alarm (test01 shape): Z3 models `p == q` as equal
/// fresh integers, but distinct objects never alias at runtime → `unknown`.
#[test]
#[ignore]
fn verify_ptrid_false_alarm_is_unknown() {
    verify_unreach("false_alarm_ptrid.c")
        .stdout("unknown\n")
        .stderr(predicate::str::contains("candidate(s) enumerated"));
}

/// Uncomposed-recursion false alarm (afterrec-2 shape): intraprocedural Z3 sees
/// `n < 0` as satisfiable, but the composed chain f(5)->..->f(0) never makes n
/// negative at runtime → `unknown`.
#[test]
#[ignore]
fn verify_recursion_false_alarm_is_unknown() {
    verify_unreach("false_alarm_recursion.c")
        .stdout("unknown\n")
        .stderr(predicate::str::contains("candidate(s) enumerated"));
}

/// A task that DEFINES its own `reach_error` (via `__assert_fail`) — the
/// canonical sv-benchmarks pattern. The replay driver's `reach_error` is weak
/// (yields to the task's) and `__assert_fail` is intercepted, so the guarded
/// nondet violation still reproduces → `false(unreach-call)`. Regression for the
/// link collision the blind eval exposed.
#[test]
#[ignore]
fn verify_selfdefined_reach_error_reproduces_false() {
    verify_unreach("unreach_false_selfdef_nondet.c").stdout("false(unreach-call)\n");
}

/// ILP32 replay: the native harness must link a 32-bit binary (`clang -m32`),
/// which requires 32-bit multilib in the image. The guarded nondet violation
/// reproduces under ILP32 the same as under LP64 → `false(unreach-call)`.
/// Regression for the 32-bit-multilib requirement the blind eval exposed (most
/// sv-benchmarks unreach-call tasks are ILP32, so without multilib the `-m32`
/// link fails and every ILP32 replay is silently inconclusive).
#[test]
#[ignore]
fn verify_ilp32_nondet_reproduces_false() {
    let prp = unreach_prp();
    let fx = svcomp_fixture("unreach_false_nondet.c");
    cargo_bin_cmd!("saf")
        .args([
            "verify",
            "--property",
            prp.as_str(),
            "--data-model",
            "ILP32",
            fx.as_str(),
        ])
        .assert()
        .success()
        .stdout("false(unreach-call)\n");
}

/// Re-running a task yields a byte-identical verdict (determinism / NFR-DET).
#[test]
#[ignore]
fn verify_is_deterministic() {
    let prp = unreach_prp();
    let fx = svcomp_fixture("unreach_false_direct.c");
    let run = || {
        cargo_bin_cmd!("saf")
            .args([
                "verify",
                "--property",
                prp.as_str(),
                "--data-model",
                "LP64",
                fx.as_str(),
            ])
            .output()
            .expect("run saf verify")
            .stdout
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "verdict must be byte-identical across runs");
    assert_eq!(a, b"false(unreach-call)\n");
}

/// The verdict-affecting env toggles are pinned off, so setting them does not
/// change the verdict (plan 192 §2.3 determinism pins).
#[test]
#[ignore]
fn verify_ignores_pta_env_toggles() {
    let prp = unreach_prp();
    let fx = svcomp_fixture("unreach_false_direct.c");
    let out = cargo_bin_cmd!("saf")
        .env("SAF_PTA_FIELD_MINTING", "1")
        .env("SAF_DECOMPOSE_POINTER_ARRAYS", "1")
        .args([
            "verify",
            "--property",
            prp.as_str(),
            "--data-model",
            "LP64",
            fx.as_str(),
        ])
        .output()
        .expect("run saf verify");
    assert_eq!(out.stdout, b"false(unreach-call)\n");
}

// ---------------------------------------------------------------------------
// verify witness emission (plan 194) — YAML 2.0 violation witness written to
// `--witness`, only alongside a `false` verdict.
// ---------------------------------------------------------------------------

/// Run `saf verify <fixture> --witness <path>` and return the `Assert`.
fn verify_unreach_witness(
    fixture: &str,
    data_model: &str,
    witness: &std::path::Path,
) -> assert_cmd::assert::Assert {
    let prp = unreach_prp();
    let fx = svcomp_fixture(fixture);
    cargo_bin_cmd!("saf")
        .args([
            "verify",
            "--property",
            prp.as_str(),
            "--data-model",
            data_model,
            "--witness",
            witness.to_str().unwrap(),
            fx.as_str(),
        ])
        .assert()
        .success()
}

/// A `false` verdict writes a YAML 2.0 violation witness to `--witness`.
#[test]
#[ignore]
fn verify_false_direct_writes_witness() {
    let dir = tempfile::tempdir().unwrap();
    let w = dir.path().join("w.yml");
    verify_unreach_witness("unreach_false_direct.c", "LP64", &w).stdout("false(unreach-call)\n");
    let yaml = std::fs::read_to_string(&w).expect("witness written for a false verdict");
    assert!(yaml.contains("entry_type: violation_sequence"), "{yaml}");
    assert!(yaml.contains("type: target"), "{yaml}");
    assert!(yaml.contains("format_version:"), "{yaml}");
    assert!(
        yaml.contains("creation_time: 2024-01-01T00:00:00Z"),
        "{yaml}"
    );
}

/// An `unknown` verdict (reach_error present but unreachable) writes NO witness.
#[test]
#[ignore]
fn verify_unknown_writes_no_witness() {
    let dir = tempfile::tempdir().unwrap();
    let w = dir.path().join("w.yml");
    verify_unreach_witness("unreach_true_simple.c", "LP64", &w).stdout("unknown\n");
    assert!(
        !w.exists(),
        "no witness may be written for an unknown verdict"
    );
}

/// The emitted witness is byte-identical across re-runs (NFR-DET determinism).
#[test]
#[ignore]
fn verify_witness_is_byte_stable() {
    let dir = tempfile::tempdir().unwrap();
    let w1 = dir.path().join("w1.yml");
    let w2 = dir.path().join("w2.yml");
    verify_unreach_witness("unreach_false_direct.c", "LP64", &w1).stdout("false(unreach-call)\n");
    verify_unreach_witness("unreach_false_direct.c", "LP64", &w2).stdout("false(unreach-call)\n");
    assert_eq!(
        std::fs::read(&w1).expect("w1"),
        std::fs::read(&w2).expect("w2"),
        "witness bytes must be deterministic across runs"
    );
}

/// The emitted witness passes `witnesslint` (the SV-COMP 2.0 witness syntactic
/// gate, baked into the dev image at `$SAF_SVWITNESSES`). Runs the syntactic
/// stage of `scripts/validate_witness.sh` (CPAchecker skipped).
#[test]
#[ignore]
fn verify_witness_passes_witnesslint() {
    let dir = tempfile::tempdir().unwrap();
    let w = dir.path().join("w.yml");
    let fx = svcomp_fixture("unreach_false_direct.c");
    verify_unreach_witness("unreach_false_direct.c", "LP64", &w).stdout("false(unreach-call)\n");
    let out = std::process::Command::new("bash")
        .arg(ws("scripts/validate_witness.sh"))
        .arg(&w)
        .arg(&fx)
        .env("SAF_SKIP_CPACHECKER", "1")
        .output()
        .expect("run validate_witness.sh");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("LINT_OK"),
        "witnesslint rejected the emitted witness:\n--stdout--\n{stdout}\n--stderr--\n{stderr}"
    );
}
