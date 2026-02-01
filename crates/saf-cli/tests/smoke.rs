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
