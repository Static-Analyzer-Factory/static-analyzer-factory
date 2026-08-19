#!/usr/bin/env python3
"""Plain-python3 unit tests for the loop's gate logic (mirrors scripts/test_svcomp_split.py style)."""
import hashlib, json, os, sys, tempfile
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lib"))
import gates  # noqa: E402


def _sha(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def test_verify_immutables_detects_tamper():
    with tempfile.TemporaryDirectory() as d:
        root = Path(d)
        f = root / "scorer.py"
        f.write_text("original\n")
        manifest = {"scorer.py": _sha(f)}
        # unchanged -> no violations
        assert gates.verify_immutables(manifest, root) == [], "clean tree must pass"
        # tamper -> reported
        f.write_text("original\n# sneaky edit\n")
        assert gates.verify_immutables(manifest, root) == ["scorer.py"], "tamper must be caught"
        # missing listed file -> also a violation
        f.unlink()
        assert gates.verify_immutables(manifest, root) == ["scorer.py"], "missing immutable must be caught"


def _assistant(*blocks: dict) -> str:
    return json.dumps({"type": "assistant", "message": {"content": list(blocks)}})


def _read(path: str) -> dict:
    return {"type": "tool_use", "name": "Read", "input": {"file_path": path}}


def _bash(cmd: str) -> dict:
    return {"type": "tool_use", "name": "Bash", "input": {"command": cmd}}


def test_audit_forbidden_reads_flags_holdout_across_tools():
    forbidden = ["tests/benchmarks/svcomp-splits/holdout"]
    clean = [
        _assistant(_read("tests/benchmarks/svcomp-splits/train.jsonl")),
        _assistant({"type": "text", "text": "inspecting train diagnostics only"}),
        "not-json-line-should-be-ignored",
    ]
    assert gates.audit_forbidden_reads(clean, forbidden) == [], "reading train only is fine"

    via_read = clean + [_assistant(_read("tests/benchmarks/svcomp-splits/holdout.jsonl"))]
    assert gates.audit_forbidden_reads(via_read, forbidden) == [
        "tests/benchmarks/svcomp-splits/holdout"
    ], "a Read of the holdout must be caught"

    via_bash = clean + [_assistant(_bash("cat tests/benchmarks/svcomp-splits/holdout.jsonl | head"))]
    assert gates.audit_forbidden_reads(via_bash, forbidden) == [
        "tests/benchmarks/svcomp-splits/holdout"
    ], "a Bash cat of the holdout must be caught too (not just Read)"


def _score(confirmed=0, fp=0, wt=0, raw=0):
    """A minimal stand-in for scripts/svcomp_split_eval.py's -o JSON."""
    return {"confirmed_score": confirmed, "false_alarms": fp, "wrong_true": wt,
            "raw_score": raw, "max_score": 100}


def test_soundness_ok_requires_zero_fp_and_zero_wrong_true():
    assert gates.soundness_ok(_score(fp=0, wt=0)) is True
    assert gates.soundness_ok(_score(fp=1, wt=0)) is False   # a false alarm (-16)
    assert gates.soundness_ok(_score(fp=0, wt=1)) is False   # a wrong TRUE (-32)
    assert gates.soundness_ok(_score(fp=2, wt=3)) is False


def test_confirmed_delta_measures_tuning_progress():
    assert gates.confirmed_delta(_score(confirmed=3960), _score(confirmed=3965)) == 5
    assert gates.confirmed_delta(_score(confirmed=3960), _score(confirmed=3960)) == 0
    assert gates.confirmed_delta(_score(confirmed=3960), _score(confirmed=3950)) == -10


def test_confirmed_delta_tolerates_missing_after():
    # a broken-build arm writes no after.json -> after == {} -> big negative -> reverts (not a crash)
    assert gates.confirmed_delta(_score(confirmed=3960), {}) < 0
    assert gates.confirmed_delta(_score(confirmed=100), _score(confirmed=100)) == 0
    assert gates.confirmed_delta(_score(confirmed=100), _score(confirmed=101)) == 1


def _scorep(**fams):
    """A scorer JSON with a per_property breakdown: _scorep(**{'unreach-call': 100, ...})."""
    per = {f: {"confirmed": c} for f, c in fams.items()}
    return {"confirmed_score": sum(fams.values()), "false_alarms": 0, "wrong_true": 0,
            "max_score": 1000, "per_property": per}


def test_family_regression_flags_any_dropped_property():
    before = _scorep(**{"valid-memsafety": 200, "unreach-call": 100, "no-overflow": 50})
    # a pure Pareto improvement -> no regression
    after_up = _scorep(**{"valid-memsafety": 210, "unreach-call": 100, "no-overflow": 50})
    assert gates.family_regression(before, after_up) == []
    # +unreach but -memsafety (a cross-family trade) -> memsafety flagged, even though total rose
    after_trade = _scorep(**{"valid-memsafety": 190, "unreach-call": 130, "no-overflow": 50})
    assert gates.family_regression(before, after_trade) == ["valid-memsafety"]
    # two families drop -> both, sorted
    after_two = _scorep(**{"valid-memsafety": 190, "unreach-call": 90, "no-overflow": 50})
    assert gates.family_regression(before, after_two) == ["unreach-call", "valid-memsafety"]
    # no per_property (a family-scoped eval) -> nothing detectable (checkpoint is the backstop)
    assert gates.family_regression(_score(confirmed=100), _score(confirmed=90)) == []


def test_decide_score_anticheat_and_accumulate():
    D = dict(immutable_violations=[], forbidden_reads=[])
    # Anti-cheat FIRST (precedence) — these ALERT, not merely revert.
    assert gates.decide(immutable_violations=["scripts/svcomp_split_eval.py"],
                        forbidden_reads=[], delta=5, progressed=True) == "REJECT_TAMPER"
    assert gates.decide(immutable_violations=[], forbidden_reads=["svcomp-splits/holdout"],
                        delta=5, progressed=True) == "REJECT_HOLDOUT"
    # a real score gain -> KEEP (regardless of progressed)
    assert gates.decide(**D, delta=5, progressed=False) == "KEEP"
    # score-NEUTRAL but USEFUL (compiles + tests + diff) -> ACCUMULATE (preserved for future arms)
    assert gates.decide(**D, delta=0, progressed=True) == "ACCUMULATE"
    # score-neutral, not useful -> REVERT
    assert gates.decide(**D, delta=0, progressed=False) == "REVERT"
    # a regression -> REVERT even if it compiles (never lower the score)
    assert gates.decide(**D, delta=-3, progressed=True) == "REVERT"
    # CROSS-CUTTING arm that regressed another family -> REVERT, even with a positive overall delta
    assert gates.decide(**D, delta=5, progressed=False, family_regressed=True) == "REVERT"
    # family_regressed=False leaves the normal ladder intact
    assert gates.decide(**D, delta=5, progressed=False, family_regressed=False) == "KEEP"


def _scorew(confirmed_weighted=0, **kw):
    """A scorer JSON carrying the deduped weighted score (svcomp_split_eval --group-weight)."""
    d = {"confirmed_score": confirmed_weighted, "confirmed_score_weighted": confirmed_weighted,
         "false_alarms": 0, "wrong_true": 0, "max_score": 1000}
    d.update(kw)
    return d


def test_generalization_and_pool_deltas():
    assert gates.generalization_delta(_scorew(40), _scorew(43)) == 3
    assert gates.pool_guard_delta_w(_scorew(200), _scorew(200)) == 0
    assert gates.pool_guard_delta_w(_scorew(200), _scorew(195)) == -5
    # a broken-build after (no weighted score) -> big negative -> regression, not a crash
    assert gates.generalization_delta(_scorew(40), {}) < 0


def test_decide_v2_rewards_generalization_not_memorization():
    D = dict(immutable_violations=[], forbidden_reads=[])
    # reasoning-set gain, pool not regressed -> KEEP (the real win)
    assert gates.decide_v2(**D, gen_delta_w=3, pool_delta_w=0) == "KEEP"
    assert gates.decide_v2(**D, gen_delta_w=3, pool_delta_w=5) == "KEEP"
    # pool-only Juliet gain, val flat -> KEEP_POOL (banked, but supervisor won't reset the stall)
    assert gates.decide_v2(**D, gen_delta_w=0, pool_delta_w=7) == "KEEP_POOL"
    # a KEEP must never lower the deduped pool, even with a val gain -> REVERT
    assert gates.decide_v2(**D, gen_delta_w=3, pool_delta_w=-1) == "REVERT"
    # memorization that also regresses the reasoning set -> REVERT (can't tie-break on Juliet)
    assert gates.decide_v2(**D, gen_delta_w=-1, pool_delta_w=9) == "REVERT"
    # a capability arm's FIRST novel solve, net-neutral weighted -> ACCUMULATE_PLUS (preserve + priority)
    assert gates.decide_v2(**D, gen_delta_w=0, pool_delta_w=0, novel_solved=True) == "ACCUMULATE_PLUS"
    # score-neutral but useful -> ACCUMULATE
    assert gates.decide_v2(**D, gen_delta_w=0, pool_delta_w=0, progressed=True) == "ACCUMULATE"
    # no-op -> REVERT
    assert gates.decide_v2(**D, gen_delta_w=0, pool_delta_w=0) == "REVERT"
    # anti-cheat precedence + crosscut family regression
    assert gates.decide_v2(immutable_violations=["s"], forbidden_reads=[], gen_delta_w=9, pool_delta_w=9) == "REJECT_TAMPER"
    assert gates.decide_v2(immutable_violations=[], forbidden_reads=["h"], gen_delta_w=9, pool_delta_w=9) == "REJECT_HOLDOUT"
    assert gates.decide_v2(**D, gen_delta_w=9, pool_delta_w=9, family_regressed=True) == "REVERT"


def test_holdout_updates_boosts_reproducing_levers():
    # holdout weighted ROSE -> every lever that banked val lift this window reproduced -> boost + clear stall.
    upd = gates.holdout_lever_updates(holdout_direction="up",
                                      per_lever_lift={"la": 3, "lb": 0, "lc": 5},
                                      holdout_stall={"la": 1, "lc": 0}, min_lift=2,
                                      resolvable_levers={"la", "lc"})
    assert upd["boost"] == ["la", "lc"]  # lb had no lift -> not boosted
    assert upd["park"] == []
    assert upd["new_stall"] == {"la": 0}  # la's stall cleared; lc already 0 -> unchanged/omitted


def test_holdout_flat_is_inconclusive_no_stall_no_park():
    # 2026-08-20 fix: a FLAT holdout is the DEFAULT outcome on a tiny dedup-weighted holdout, NOT evidence
    # of overfit -> a lever that banked >= min_lift is neither stalled nor parked (the core fix).
    upd = gates.holdout_lever_updates(holdout_direction="flat",
                                      per_lever_lift={"la": 4}, holdout_stall={"la": 1}, min_lift=2,
                                      resolvable_levers={"la"})
    assert upd["park"] == [] and upd["boost"] == [] and upd["new_stall"] == {}


def test_holdout_regression_parks_after_two_when_family_resolvable():
    # only an ACTUAL weighted regression ('down') counts against a lever, and only for a resolvable family.
    upd = gates.holdout_lever_updates(holdout_direction="down",
                                      per_lever_lift={"la": 4}, holdout_stall={"la": 0}, min_lift=2,
                                      resolvable_levers={"la"})
    assert upd["park"] == [] and upd["new_stall"] == {"la": 1}
    upd2 = gates.holdout_lever_updates(holdout_direction="down",
                                       per_lever_lift={"la": 4}, holdout_stall={"la": 1}, min_lift=2,
                                       resolvable_levers={"la"})
    assert upd2["park"] == ["la"] and upd2["new_stall"] == {"la": 2}


def test_holdout_regression_never_parks_unresolvable_family():
    # a family the holdout cannot resolve (e.g. ~1 held-out task) can NEVER park a lever, even on a drop.
    upd = gates.holdout_lever_updates(holdout_direction="down",
                                      per_lever_lift={"term": 9}, holdout_stall={"term": 1}, min_lift=2,
                                      resolvable_levers=set())  # term's family below min_family_tasks
    assert upd["park"] == [] and upd["new_stall"] == {}


def test_holdout_updates_ignores_sub_threshold_and_zero_lift():
    # a tiny lift below min_lift is NOT strong evidence -> no stall even on a regression; zero-lift untouched.
    upd = gates.holdout_lever_updates(holdout_direction="down",
                                      per_lever_lift={"la": 1, "lb": 0}, holdout_stall={}, min_lift=2,
                                      resolvable_levers={"la", "lb"})
    assert upd["park"] == [] and upd["boost"] == [] and upd["new_stall"] == {}


def _ls(decision, state, *, gen_mode=True, lever_budget=6, accumulate_budget=3, max_lever_arms=12):
    return gates.lever_state_after(decision, state, gen_mode=gen_mode, lever_budget=lever_budget,
                                   accumulate_budget=accumulate_budget, max_lever_arms=max_lever_arms)


def test_lever_state_after_revert_streak_parks_at_lever_budget():
    # unchanged behavior the multi_arm_budget smoke asserts: N consecutive reverts -> park 'revert'.
    st, reason = {"attempts": 0, "revert_stall": 0, "accum_stall": 0}, None
    for _ in range(3):
        st, reason = _ls("REVERT", st, lever_budget=3)
    assert st["revert_stall"] == 3 and reason == "revert"


def test_lever_state_after_keep_resets_both_stalls():
    st, reason = _ls("KEEP", {"attempts": 5, "revert_stall": 2, "accum_stall": 2})
    assert st["revert_stall"] == 0 and st["accum_stall"] == 0 and reason is None


def test_lever_state_after_accumulate_forever_parks_gen_mode():
    # BUG-4: an ACCUMULATE-only lever (race-confirmer) parks on its own budget, not at max_lever_arms.
    st, reason = {"attempts": 0, "revert_stall": 0, "accum_stall": 0}, None
    for _ in range(3):
        st, reason = _ls("ACCUMULATE", st, gen_mode=True, accumulate_budget=3)
    assert st["accum_stall"] == 3 and reason == "accumulate"


def test_lever_state_after_accumulate_forever_parks_legacy_mode():
    # BUG-4 (the real gap): in LEGACY mode a plain ACCUMULATE reset the revert-stall so the lever
    # never parked; the dedicated accumulate budget parks it regardless of mode.
    st, reason = {"attempts": 0, "revert_stall": 0, "accum_stall": 0}, None
    for _ in range(3):
        st, reason = _ls("ACCUMULATE", st, gen_mode=False, accumulate_budget=3)
    assert reason == "accumulate"
    assert st["revert_stall"] == 0  # legacy still resets the revert-stall each ACCUMULATE (unchanged)


def test_lever_state_after_keep_pool_banks_points_but_still_parks_on_revert_side():
    # KEEP_POOL banks honest pool points -> resets accum_stall (not an unscoreable polish), but does
    # NOT reset the revert-stall (a pure-memorization lever still parks on the revert side, gen mode).
    st, reason = _ls("KEEP_POOL", {"attempts": 0, "revert_stall": 0, "accum_stall": 2}, gen_mode=True)
    assert st["accum_stall"] == 0 and st["revert_stall"] == 1 and reason is None


def test_lever_state_after_accumulate_plus_resets_both():
    st, reason = _ls("ACCUMULATE_PLUS", {"attempts": 0, "revert_stall": 2, "accum_stall": 2})
    assert st["revert_stall"] == 0 and st["accum_stall"] == 0 and reason is None


def test_lever_state_after_cap_takes_precedence():
    # even a KEEP that resets the stalls parks if it hits the hard total-arms cap.
    st, reason = _ls("KEEP", {"attempts": 11, "revert_stall": 0, "accum_stall": 0}, max_lever_arms=12)
    assert reason == "cap"


def test_lever_state_after_keep_interrupts_accumulate_streak():
    # plateaued-KEEP lever (exec-validator-gate's tail): an early KEEP resets accum_stall; later plain
    # ACCUMULATEs count from there and eventually park.
    st = {"attempts": 0, "revert_stall": 0, "accum_stall": 0}
    st, _ = _ls("ACCUMULATE", st)
    st, _ = _ls("ACCUMULATE", st)
    st, _ = _ls("KEEP", st)
    assert st["accum_stall"] == 0
    reason = None
    for _ in range(3):
        st, reason = _ls("ACCUMULATE", st, accumulate_budget=3)
    assert reason == "accumulate"


def test_lever_state_after_zero_budget_disables_accumulate_park():
    st, reason = {"attempts": 0, "revert_stall": 0, "accum_stall": 0}, None
    for _ in range(9):
        st, reason = _ls("ACCUMULATE", st, gen_mode=True, accumulate_budget=0, lever_budget=0)
    assert reason is None and st["accum_stall"] == 9


TESTS = [v for k, v in sorted(globals().items()) if k.startswith("test_")]


def main() -> int:
    passed = 0
    for t in TESTS:
        try:
            t()
            print(f"  ok   {t.__name__}")
            passed += 1
        except AssertionError as e:
            print(f"  FAIL {t.__name__}: {e}")
        except Exception as e:  # noqa: BLE001
            print(f"  ERR  {t.__name__}: {type(e).__name__}: {e}")
    print(f"\n{passed}/{len(TESTS)} green")
    return 0 if passed == len(TESTS) else 1


if __name__ == "__main__":
    sys.exit(main())
