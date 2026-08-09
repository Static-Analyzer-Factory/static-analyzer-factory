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
