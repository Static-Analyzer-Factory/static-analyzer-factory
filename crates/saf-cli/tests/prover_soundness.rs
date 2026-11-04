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

// ---------------------------------------------------------------------------
// M4 — SCCP must re-visit a block's phis when a NEW EDGE reaches it
// ---------------------------------------------------------------------------

/// The last of the seven Movement-1 mechanisms, and the only one that stayed
/// open long enough to reach production as a live wrong TRUE.
///
/// `mark_edge_executable` scheduled a block only when the BLOCK became newly
/// executable, never when a new EDGE arrived at one already executable. A `Phi`
/// meets over the edges executable *when it is evaluated*, so the join phi
/// carrying `s` froze at the value it had before the deepest dispatch arm's edge
/// existed. SCCP published that frozen singleton, the interval solver adopted it
/// via `constant_map`, and the sentinel proved an `INT_MAX * 2` it never saw.
///
/// RED before the fix: this printed `PROVE`, and `saf verify --property
/// no-overflow` printed `true`, on a program that overflows. −32 points, uncapped.
#[test]
#[ignore]
fn does_not_prove_no_overflow_on_frozen_dispatch_phi() {
    prove_no_overflow("overflow_false_dispatch_chain.c", "ILP32")
        .stdout(predicate::str::starts_with("PROVE").not());
}

/// GUARD: re-visiting phis makes them less constant, which could have paid for
/// M4 with the whole `no-overflow` TRUE arm — 33 of SAF's 142 dedup-weighted
/// points. A fix that closed M4 by making everything abstain would be a bad trade
/// dressed as a soundness win, so pin a program that must keep PROVEing.
///
/// Measured A/B over 200 stride-sampled expected-TRUE `no-overflow` tasks:
/// PROVE 8 → 8, the only delta being one task moving between two abstain reasons.
#[test]
#[ignore]
fn still_proves_no_overflow_after_the_sccp_revisit_fix() {
    prove_no_overflow("overflow_true_safe.c", "ILP32").stdout(predicate::str::starts_with("PROVE"));
}

// ---------------------------------------------------------------------------
// no-data-race TRUE — element-sensitive aliasing
//
// The structural race prover (`race_true.rs`) skips a pair of accesses when the
// points-to analysis says they cannot alias. `PtaResult::may_alias` answered `No`
// for `data[4]` vs `data[i]` (two locations of one object with incomparable field
// paths) although they overlap whenever `i == 4`; locations in one object now always
// count as overlapping. Reduced from a wrong TRUE in the SV-COMP 2027 training run.
// ---------------------------------------------------------------------------

/// `saf verify --property no-data-race.prp --data-model ILP32 <fixture>`, asserting exit 0.
fn verify_no_data_race(name: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("saf")
        .args([
            "verify",
            "--property",
            fixture("no-data-race.prp").as_str(),
            "--data-model",
            "ILP32",
            fixture(name).as_str(),
        ])
        .assert()
        .success()
}

/// `data[4]` in the thread vs `data[i]` in `main` must never be proved race-free.
#[test]
#[ignore]
fn does_not_prove_race_free_const_vs_unknown_index() {
    verify_no_data_race("race_false_const_vs_unknown_index.c")
        .stdout(predicate::str::contains("true").not());
}

/// GUARD: accesses to two distinct objects are disjoint and must still be proved.
#[test]
#[ignore]
fn still_proves_race_free_distinct_globals() {
    verify_no_data_race("race_true_distinct_globals.c").stdout(predicate::eq("true\n"));
}

// ---------------------------------------------------------------------------
// Allocation may fail (SV-COMP 2027)
//
// SV-COMP 2027 dropped the "allocation always succeeds" rule: `malloc`, `calloc`
// and `realloc` may return NULL (sv-benchmarks !1686). The exhaustive engine
// modelled every allocation as a fresh object, so an error reachable only after a
// failed allocation was "proved" unreachable — a wrong TRUE on
// `ldv-regression/rule57_ebda_blast_2` in the 2027 training run.
// ---------------------------------------------------------------------------

/// The error is reachable exactly when `calloc` returns NULL: never PROVE.
#[test]
#[ignore]
fn does_not_prove_unreachable_when_allocation_may_fail() {
    prove_unreachable("prove_false_alloc_may_fail.c")
        .stdout(predicate::str::starts_with("PROVE").not());
}

/// GUARD: a program that aborts on a NULL `malloc` is safe on both outcomes and
/// must still PROVE.
#[test]
#[ignore]
fn still_proves_unreachable_with_null_guarded_allocation() {
    prove_unreachable("prove_unreach_alloc_null_guarded.c")
        .stdout(predicate::str::starts_with("PROVE"));
}

// ---------------------------------------------------------------------------
// valid-memsafety leak check at a termination
//
// At `abort`/`exit` the call stack still exists: a block an active function can
// still use is not lost, one whose last pointer was overwritten is.
// ---------------------------------------------------------------------------

/// `saf verify --property valid-memsafety.prp --data-model LP64 <fixture>`, asserting exit 0.
fn verify_memsafety(name: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("saf")
        .args([
            "verify",
            "--property",
            fixture("valid-memsafety.prp").as_str(),
            "--data-model",
            "LP64",
            fixture(name).as_str(),
        ])
        .assert()
        .success()
}

/// `safe_malloc` aborts on NULL while `main` holds the earlier block: TRUE.
#[test]
#[ignore]
fn memsafety_true_when_abort_happens_while_the_caller_holds_the_block() {
    verify_memsafety("memsafety_true_abort_while_caller_holds.c").stdout(predicate::eq("true\n"));
}

/// The block's only pointer is overwritten before `abort()`: never TRUE.
#[test]
#[ignore]
fn memsafety_not_true_when_the_block_is_lost_before_abort() {
    verify_memsafety("memsafety_false_lost_before_abort.c")
        .stdout(predicate::str::contains("true").not());
}

// ---------------------------------------------------------------------------
// valid-memcleanup
//
// Every allocated block must be freed before the program terminates; both
// verdicts score without a witness. FALSE comes from a native run that
// terminates normally with blocks still allocated, TRUE from the engine.
// ---------------------------------------------------------------------------

/// `saf verify --property valid-memcleanup.prp --data-model LP64 <fixture>`, asserting exit 0.
fn verify_memcleanup(name: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("saf")
        .args([
            "verify",
            "--property",
            fixture("valid-memcleanup.prp").as_str(),
            "--data-model",
            "LP64",
            fixture(name).as_str(),
        ])
        .assert()
        .success()
}

/// A block still allocated (though reachable) when `main` returns: FALSE.
#[test]
#[ignore]
fn memcleanup_false_when_a_block_outlives_main() {
    verify_memcleanup("memcleanup_false_global_kept.c")
        .stdout(predicate::eq("false(valid-memcleanup)\n"));
}

/// The program's own `atexit` handler frees the block: never FALSE.
#[test]
#[ignore]
fn memcleanup_not_false_when_freed_in_atexit_handler() {
    verify_memcleanup("memcleanup_true_freed_in_atexit.c")
        .stdout(predicate::str::contains("false").not());
}

/// The only unfreed block is on a path a false assumption cuts: never FALSE, and
/// the engine proves it.
#[test]
#[ignore]
fn memcleanup_true_when_the_leak_is_assumed_away() {
    verify_memcleanup("memcleanup_true_assume_cuts_leak.c").stdout(predicate::eq("true\n"));
}

/// `reach_error()` aborts while the block is allocated: an abort is a
/// termination for valid-memcleanup, so FALSE.
#[test]
#[ignore]
fn memcleanup_false_when_an_error_call_aborts_with_a_block_allocated() {
    verify_memcleanup("memcleanup_false_error_leaves_block.c")
        .stdout(predicate::eq("false(valid-memcleanup)\n"));
}
