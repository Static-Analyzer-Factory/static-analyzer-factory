//! Movement 1 (`plans/213`) — CLI-level regressions for the two absint fixes that
//! let SAF emit a TRUE verdict on its own authority.
//!
//! The unit tests in `saf-analysis` pin the domain; these pin the whole pipeline
//! (clang → mem2reg → AIR → interval fixpoint → sentinel), because both bugs were
//! only visible end to end. Two shapes, two directions:
//!
//! * **recall** (`§2a`, `Interval::refine_eq_false`) — `x != c` where `x` is already
//!   the singleton `c` must kill the edge, so a program whose only error is behind
//!   that guard must `PROVE`. This was the `error-reachable` wall.
//! * **soundness** (`§2b`, unsigned refinement) — an UNSIGNED comparison must never
//!   be refined with the SIGNED refiners unless both operands are provably
//!   non-negative. Each negative fixture is reduced from a real ground-truth-FALSE
//!   SV-COMP task that SAF wrongly PROVEd. A wrong PROVE is a wrong TRUE: −32
//!   points, uncapped by the per-cluster dedup.
//!
//! These need clang-18/opt-18 (Docker only), so they are `#[ignore]`d like the rest
//! of the LLVM-dependent suite. Run them with `make test-ignored`.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

fn fixture(name: &str) -> String {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/saf-cli -> crates
    p.pop(); // crates -> workspace root
    p.join("tests/programs/c/svcomp")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// `saf prove-unreachable --data-model ILP32 <fixture>`, asserting exit 0.
fn prove_unreachable(name: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("saf")
        .args([
            "prove-unreachable",
            "--data-model",
            "ILP32",
            fixture(name).as_str(),
        ])
        .assert()
        .success()
}

/// `saf prove-no-overflow --data-model <dm> <fixture>`, asserting exit 0.
fn prove_no_overflow(name: &str, data_model: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("saf")
        .args([
            "prove-no-overflow",
            "--data-model",
            data_model,
            fixture(name).as_str(),
        ])
        .assert()
        .success()
}

// ---------------------------------------------------------------------------
// §2a recall — these must PROVE
// ---------------------------------------------------------------------------

/// `if (!(x == 0))` with `x` the singleton 0. RED before the `refine_eq_false`
/// fix: returned `ABSTAIN:error-reachable` on a three-line program.
#[test]
#[ignore]
fn proves_unreachable_behind_eq_false_singleton_guard() {
    prove_unreachable("prove_unreach_eq_false_singleton.c")
        .stdout(predicate::str::starts_with("PROVE"));
}

/// The `!=` spelling of the same guard — shares the refinement arm.
#[test]
#[ignore]
fn proves_unreachable_behind_ne_singleton_guard() {
    prove_unreachable("prove_unreach_ne_singleton.c").stdout(predicate::str::starts_with("PROVE"));
}

// ---------------------------------------------------------------------------
// §2b soundness — these must NOT prove. Both are reduced from ground-truth-FALSE
// SV-COMP tasks that SAF wrongly PROVEd before the guard landed.
// ---------------------------------------------------------------------------

/// `unsigned 1 < (int)-1` is TRUE in C, so the error IS reachable.
/// Reduced from `bitvector-regression/implicitunsignedconversion-1`.
#[test]
#[ignore]
fn does_not_prove_reachable_unsigned_lt_negative() {
    prove_unreachable("unreach_false_unsigned_lt.c")
        .stdout(predicate::str::starts_with("PROVE").not());
}

/// `0 <u 0xfffffffe` is TRUE, so the body IS reachable.
/// Reduced from `loop-simple/deep-nested`.
#[test]
#[ignore]
fn does_not_prove_reachable_uintmax_bound() {
    prove_unreachable("unreach_false_uintmax_bound.c")
        .stdout(predicate::str::starts_with("PROVE").not());
}

// ---------------------------------------------------------------------------
// The completeness gate — "absence is not unreachability"
//
// Both sentinels read the converged fixpoint, and the fixpoint PRE-SEEDS every
// block with ⊥ (`fixpoint.rs`). So a block the analysis never visited is
// indistinguishable from one a branch condition refuted. `prove_no_signed_overflow`
// turned that ambiguity into a vacuous proof: `state_at_inst == None => continue`
// silently DROPPED the obligation. On `openssl-simplified/s3_srvr_1a.cil` all four
// `add`s were absent, so `Proven` was returned having checked zero arithmetic.
// The gate fails closed instead.
// ---------------------------------------------------------------------------

/// The reduced `openssl-simplified` state machine. `n + 1` with `n` nondet must
/// never be PROVEd — before the gate, its only obligation was skipped.
#[test]
#[ignore]
fn does_not_prove_no_overflow_when_arithmetic_was_never_visited() {
    prove_no_overflow("overflow_false_statemachine.c", "ILP32")
        .stdout(predicate::str::starts_with("PROVE").not());
}

/// GUARD: the identical shape with small state constants is analysed correctly
/// today and must keep abstaining for the RIGHT reason (a TOP operand), not
/// because the gate fired. This pins that the gate did not simply blanket-abstain.
#[test]
#[ignore]
fn small_constant_state_machine_still_abstains_on_top_operand() {
    prove_no_overflow("overflow_false_statemachine_small.c", "ILP32")
        .stdout(predicate::str::starts_with("ABSTAIN:top-or-bottom-operand"));
}

// ---------------------------------------------------------------------------
// M3 — stale refinement persistence
//
// `collect_refinements` caches the INTERVAL a branch predicate produced on one
// fixpoint iteration; `apply_refinements` re-imposes it by MEET on every later
// one. The predicate is an edge invariant; the interval it produced given an
// early, smaller pre-state is not. Re-imposing it UNDER-approximates, and an
// under-approximated state is a wrong-TRUE generator that the completeness gate
// cannot see — the instruction IS visited, its state is simply wrong.
// ---------------------------------------------------------------------------

/// `s` reaches INT_MAX and `s + 1` overflows. Must never be PROVEd.
#[test]
#[ignore]
fn does_not_prove_no_overflow_under_stale_refinement() {
    prove_no_overflow("overflow_false_stale_refinement.c", "ILP32")
        .stdout(predicate::str::starts_with("PROVE").not());
}

/// GUARD: the same shape ending at 100 genuinely cannot overflow and must still
/// PROVE. Stops the M3 fix from degenerating into "never persist a refinement",
/// which would be trivially sound and cost real recall.
#[test]
#[ignore]
fn still_proves_no_overflow_when_refinement_is_not_stale() {
    prove_no_overflow("overflow_true_stale_refinement_safe.c", "ILP32")
        .stdout(predicate::str::starts_with("PROVE"));
}

// ---------------------------------------------------------------------------
// M2 — SCCP binary folding must normalise to the result width
// ---------------------------------------------------------------------------

/// `2147483648u + 2147483648u` wraps to 0, so reach_error IS reachable.
/// Folding in i128 without wrapping at the result width makes SCCP decide the
/// comparison wrongly and mark a live block dead.
#[test]
#[ignore]
fn does_not_prove_unreachable_under_wrapped_add() {
    prove_unreachable("unreach_false_wrapped_add.c")
        .stdout(predicate::str::starts_with("PROVE").not());
}
