#!/usr/bin/env python3
"""Movement 5, part 3: realign the tests and docs that described the removed gates.

Three smoke tests asserted `true` on paths that now abstain, and one asserted a
CBMC-specific stderr line. The behaviour change is intentional, so the tests change
with it -- and each keeps its guard value: if anyone re-enables a TRUE arm without
first fixing the absint soundness bugs, these fire.
"""
import sys
from pathlib import Path

SMOKE = Path("crates/saf-cli/tests/smoke.rs")
PROPERTY = Path("crates/saf-svcomp/src/property.rs")

REPLACEMENTS = [
    (SMOKE, '''/// Rank-3 sound TRUE (END-TO-END MILESTONE): `x > 0 && x < 0` is a contradiction
/// over a nondet value, so the interval sentinel proves the `reach_error` block ⊥
/// and the in-process CPAchecker confirmation gate agrees → `saf verify` emits
/// `true` and writes an `invariant_set` correctness witness. (The guards are over a
/// nondet value, so they survive as real `icmp`s the interval refinement can prune,
/// unlike a constant-folded `if (0)`.)
#[test]
#[ignore]
fn verify_unreach_true_infeasible() {
    verify_unreach("unreach_true_infeasible.c").stdout("true\\n");
}''',
     '''/// Rank-3 TRUE is DISABLED, so even a cleanly provable contradiction abstains.
///
/// `x > 0 && x < 0` is a contradiction over a nondet value and the interval sentinel
/// still proves the `reach_error` block ⊥ — but the sentinel alone is not
/// wrong-TRUE-safe, its only confirmer was a bundled CPAchecker, and SAF ships no
/// SV-COMP participant. So `saf verify` abstains. This is the DELIBERATE price of
/// that decision; see `try_unreach_true`. Movement 1 (`plans/213`) makes the proof
/// SAF-native, and this assertion flips back to `true` then.
#[test]
#[ignore]
fn verify_unreach_true_infeasible_abstains_without_a_native_prover() {
    verify_unreach("unreach_true_infeasible.c").stdout("unknown\\n");
}'''),

    (SMOKE, '''/// Rank-3 wrong-TRUE guard (soundness): `signextension-1` is a genuine FALSE task
/// the interval sentinel WRONGLY proves (it cannot model the sign/unsigned
/// conversions that satisfy the error guard), but the in-process CPAchecker gate
/// REJECTS it, so `saf verify` must NOT emit `true` — it falls through to the FALSE
/// pipeline. ILP32 (the task's declared data model).''',
     '''/// Rank-3 wrong-TRUE guard (soundness), and now the load-bearing regression for the
/// disabled TRUE arm: `signextension-1` is a genuine FALSE task the interval sentinel
/// WRONGLY proves (it cannot model the sign/unsigned conversions that satisfy the
/// error guard). It used to be caught by the in-process CPAchecker gate; today the
/// TRUE arm abstains outright. Either way `saf verify` must NOT emit `true`.
/// **If anyone re-enables `try_unreach_true` before fixing that absint bug, this test
/// is what fires.** ILP32 (the task's declared data model).'''),

    (SMOKE, '''/// LOOP-FREE CBMC-oracle over-approximation: `cbmc_precheck` admits acyclic
/// programs, and CBMC runs with `--no-standard-checks`, so it models `x + y` as
/// wrapping and PROPOSES the overflowing vector. The native replay compiles with
/// `-fsanitize-trap=signed-integer-overflow`, traps at the addition and never
/// reaches the sentinel → `unknown`. The stderr assertion pins the replay gate as
/// the SOLE arbiter (R6) for the loop-free population: short-circuit it and this
/// fixture becomes a false alarm. Needs the provisioned CBMC (`$SAF_CBMC`).
#[test]
#[ignore]
fn verify_loopfree_cbmc_overapprox_is_unknown() {
    verify_unreach("false_alarm_cbmc_loopfree.c")
        .stdout("unknown\\n")
        .stderr(predicate::str::contains(
            "CBMC-proposed vector did not re-confirm deterministically",
        ));
}''',
     '''/// A loop-free fixture on which an over-approximating oracle would propose an
/// overflowing vector that the native replay then refuses.
///
/// This was the CBMC lever's regression: CBMC ran with `--no-standard-checks`, modelled
/// `x + y` as wrapping, and proposed a vector the `-fsanitize-trap=signed-integer-overflow`
/// replay trapped on. The lever is gone (CBMC is an SV-COMP participant and SAF bundles
/// no competitor), so no lever proposes anything here and the verdict is `unknown` for
/// the simpler reason. Kept because the fixture still pins the invariant that matters:
/// **whatever proposes a vector, the native replay is the SOLE arbiter (R6)** — this
/// must never become a false alarm.
#[test]
#[ignore]
fn verify_loopfree_overapprox_is_never_a_false_alarm() {
    verify_unreach("false_alarm_cbmc_loopfree.c").stdout("unknown\\n");
}'''),

    (SMOKE, '''/// A safe, PROVABLE program: rank-2 sound no-overflow TRUE — the interval sentinel
/// proves every reachable signed op in-bounds (the guard bounds `x` to [1,99]
/// before `x+1`), and the emitted witness is confirmed in-process by real
/// CPAchecker -> `true`. (Before rank 2 this was `unknown`.)
#[test]
#[ignore]
fn verify_overflow_safe_provable_is_true() {
    verify_overflow("overflow_true_safe.c", "LP64").stdout("true\\n");
}''',
     '''/// Rank-2 no-overflow TRUE is DISABLED, so a safe, provable program still abstains.
///
/// The interval sentinel does prove every reachable signed op in-bounds (the guard
/// bounds `x` to [1,99] before `x+1`), but proving is not enough on its own — see
/// `try_overflow_true`. This is the single largest line item in the -12 weighted the
/// no-competitor-tools decision costs, and it flips back to `true` when Movement 1
/// (`plans/213`) makes the proof sound without an external confirmer.
#[test]
#[ignore]
fn verify_overflow_safe_provable_abstains_without_a_native_prover() {
    verify_overflow("overflow_true_safe.c", "LP64").stdout("unknown\\n");
}'''),

    (SMOKE, '''/// A rank-2 no-overflow TRUE writes a YAML-2.0 `invariant_set` CORRECTNESS witness
/// (the artifact CPAchecker confirmed in-process before the verdict was emitted).
#[test]
#[ignore]
fn verify_overflow_true_writes_correctness_witness() {
    let dir = tempfile::tempdir().unwrap();
    let w = dir.path().join("w.yml");
    verify_overflow_witness("overflow_true_safe.c", "LP64", &w).stdout("true\\n");
    let yaml = std::fs::read_to_string(&w).expect("correctness witness written for a true verdict");
    assert!(yaml.contains("entry_type: invariant_set"), "{yaml}");
}''',
     '''/// With the rank-2 TRUE arm disabled, an abstaining run writes NO correctness witness.
///
/// The `invariant_set` builder itself is untouched and still unit-tested in
/// `saf-svcomp`; what is asserted here is that SAF does not leave a stray witness file
/// beside an `unknown` verdict, which would confuse the competition's `<resultfiles>`
/// collection into validating a witness for a result SAF never claimed.
#[test]
#[ignore]
fn verify_overflow_abstain_writes_no_correctness_witness() {
    let dir = tempfile::tempdir().unwrap();
    let w = dir.path().join("w.yml");
    verify_overflow_witness("overflow_true_safe.c", "LP64", &w).stdout("unknown\\n");
    assert!(
        !w.exists(),
        "an abstaining verdict must not leave a witness file behind"
    );
}'''),

    (SMOKE, '''/// SOUNDNESS REGRESSION (wrong-TRUE=0): a COMPILE-TIME constant overflow that clang
/// folds away (no IR arithmetic for the sentinel to see) must NEVER yield `true`.
/// The TRUE arm abstains (the `-Winteger-overflow` probe fires, and the CPAchecker
/// gate would reject anyway); the UBSan FALSE path then confirms the overflow.''',
     '''/// SOUNDNESS REGRESSION (wrong-TRUE=0): a COMPILE-TIME constant overflow that clang
/// folds away (no IR arithmetic for the sentinel to see) must NEVER yield `true`.
/// The TRUE arm abstains (doubly so now that it is disabled outright, but the
/// `-Winteger-overflow` probe fires independently); the UBSan FALSE path then
/// confirms the overflow.'''),

    (PROPERTY, '''/// NOTE (rank-3): the sentinel is a FILTER; a wrong `Proven` from an absint
/// unsoundness bug is caught downstream by the in-process CPAchecker confirmation
/// gate (the FINAL verdict authority). This function emits no verdict.''',
     '''/// NOTE (rank-3): the sentinel is a FILTER, and a KNOWN-UNSOUND one — it returns a
/// wrong `Proven` on at least the sign/unsigned-conversion class. It used to be
/// backstopped by an in-process CPAchecker confirmation gate; SAF now bundles no
/// SV-COMP participant, so there is no backstop and `try_unreach_true` abstains
/// unconditionally. **Do not wire this into a verdict** until Movement 1
/// (`plans/213`) fixes the underlying bugs. This function emits no verdict.'''),

    (PROPERTY, '''    //! Rank-3 read-out core (`prove_unreachable`) — the SOUND rule (error block
    //! PRESENT and ⊥, never absence) + fail-closed gates. The full PROVE→confirm
    //! pipeline (incl. the wrong-TRUE class the interval absint gets wrong) is
    //! covered by the in-process-CPAchecker e2e in `saf-cli/tests/smoke.rs`.''',
     '''    //! Rank-3 read-out core (`prove_unreachable`) — the SOUND rule (error block
    //! PRESENT and ⊥, never absence) + fail-closed gates. The wrong-TRUE class the
    //! interval absint gets wrong is pinned by
    //! `verify_unreach_wrongprove_is_not_true` in `saf-cli/tests/smoke.rs`, which is
    //! now the regression guarding the disabled TRUE arm.'''),
]


def main():
    counts = {}
    for path, old, new in REPLACEMENTS:
        text = path.read_text()
        if old not in text:
            sys.exit(f"pattern not found in {path}:\n---\n{old[:220]}\n---")
        if text.count(old) != 1:
            sys.exit(f"pattern is not unique in {path} ({text.count(old)} hits)")
        path.write_text(text.replace(old, new))
        counts[path] = counts.get(path, 0) + 1
    for p, n in counts.items():
        print(f"  {p}: {n} replacement(s)")


if __name__ == "__main__":
    main()
