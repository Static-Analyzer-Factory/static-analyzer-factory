#!/usr/bin/env python3
"""Unit test for lib/heldout_action.py — the actionable svcomp26-holdout brake (plan 205 §1a/§2.4).

Exercises the real IO orchestrator against a temp STATE_DIR: first-checkpoint baseline, BOOST on a
reproduced gain, PARK-OVERFIT only after TWO consecutive non-reproducing checkpoints, and the
graceful no-op when the scorer emitted no weighted score. Plain-python3 (no pytest), mirrors the
other loop tests' TESTS=[...] + main() runner.
"""
import json
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
LIB = HERE.parent / "lib"
sys.path.insert(0, str(LIB))
import heldout_action as ha  # noqa: E402


def _arms(path: Path, rows: list[dict]) -> None:
    path.write_text("\n".join(json.dumps(r) for r in rows) + "\n")


def _keep(n, lever, delta, family="famBig"):
    return {"arm": n, "lever": lever, "decision": "KEEP", "confirmed_delta": delta, "family": family}


def _holdout(path: Path, weighted, families=None) -> None:
    # families: {family: n_held_out_tasks} -> per_property (a family is "resolvable" iff n >= min_family_tasks)
    d: dict = {"confirmed_score": 99}
    if weighted is not None:
        d["confirmed_score_weighted"] = weighted
    if families:
        d["per_property"] = {fam: {"n": n} for fam, n in families.items()}
    path.write_text(json.dumps(d))


def test_first_checkpoint_is_baseline_no_action():
    with tempfile.TemporaryDirectory() as dd:
        sd = Path(dd)
        _arms(sd / "arms.jsonl", [_keep(1, "la", 3)])
        _holdout(sd / "h1.json", 5)
        msg = ha.run(str(sd), 1, str(sd / "h1.json"), min_lift=2)
        assert "first weighted checkpoint = 5" in msg, msg
        assert (sd / "heldout_weighted_prev").read_text().strip() == "5"
        assert (sd / "last_heldout_arm").read_text().strip() == "1"
        assert not (sd / "lever.la.parked").exists()
        assert not (sd / "lever.la.gen_credit").exists()


def test_boost_on_reproduced_gain():
    with tempfile.TemporaryDirectory() as dd:
        sd = Path(dd)
        # baseline checkpoint at arm 1
        _arms(sd / "arms.jsonl", [_keep(1, "la", 3)])
        _holdout(sd / "h1.json", 5)
        ha.run(str(sd), 1, str(sd / "h1.json"), min_lift=2)
        # arm 2 banks more val lift; holdout weighted rises 5 -> 8 -> the lever reproduced -> BOOST
        _arms(sd / "arms.jsonl", [_keep(1, "la", 3), _keep(2, "la", 3)])
        _holdout(sd / "h2.json", 8)
        msg = ha.run(str(sd), 2, str(sd / "h2.json"), min_lift=2)
        assert "BOOST la" in msg, msg
        assert (sd / "lever.la.gen_credit").read_text().strip() == "1"
        assert (sd / "lever.la.stall").read_text().strip() == "0"   # revert-stall reset
        assert not (sd / "lever.la.parked").exists()
        assert (sd / "heldout_weighted_prev").read_text().strip() == "8"


def test_flat_holdout_is_inconclusive_never_parks():
    # 2026-08-20 fix: a FLAT holdout (the default outcome on a tiny dedup-weighted holdout) is NOT overfit
    # evidence -> a lever that banks >= min_lift is neither stalled nor parked, however many times it repeats.
    with tempfile.TemporaryDirectory() as dd:
        sd = Path(dd)
        (sd / "heldout_weighted_prev").write_text("10")
        (sd / "last_heldout_arm").write_text("0")
        _arms(sd / "arms.jsonl", [_keep(1, "la", 4)])
        _holdout(sd / "h1.json", 10, families={"famBig": 50})   # flat 10 -> 10
        msg1 = ha.run(str(sd), 1, str(sd / "h1.json"), min_lift=2, stall_to_park=2)
        assert "PARK-OVERFIT" not in msg1 and "inconclusive" in msg1, msg1
        assert not (sd / "lever.la.holdout_stall").exists()
        _arms(sd / "arms.jsonl", [_keep(1, "la", 4), _keep(2, "la", 4)])
        _holdout(sd / "h2.json", 10, families={"famBig": 50})   # flat again
        msg2 = ha.run(str(sd), 2, str(sd / "h2.json"), min_lift=2, stall_to_park=2)
        assert "PARK-OVERFIT" not in msg2, msg2
        assert not (sd / "lever.la.parked").exists()


def test_regression_parks_after_two_when_family_resolvable():
    # only an ACTUAL weighted regression counts against a lever, and only for a resolvable family.
    with tempfile.TemporaryDirectory() as dd:
        sd = Path(dd)
        (sd / "heldout_weighted_prev").write_text("10")
        (sd / "last_heldout_arm").write_text("0")
        _arms(sd / "arms.jsonl", [_keep(1, "la", 4)])
        _holdout(sd / "h1.json", 9, families={"famBig": 50})    # down 10 -> 9
        msg1 = ha.run(str(sd), 1, str(sd / "h1.json"), min_lift=2, stall_to_park=2)
        assert "PARK-OVERFIT" not in msg1 and "la:1" in msg1, msg1
        assert (sd / "lever.la.holdout_stall").read_text().strip() == "1"
        _arms(sd / "arms.jsonl", [_keep(1, "la", 4), _keep(2, "la", 4)])
        _holdout(sd / "h2.json", 8, families={"famBig": 50})    # down 9 -> 8 (2nd consecutive)
        msg2 = ha.run(str(sd), 2, str(sd / "h2.json"), min_lift=2, stall_to_park=2)
        assert "PARK-OVERFIT la" in msg2, msg2
        assert (sd / "lever.la.parked").exists()
        assert (sd / "ALERT_OVERFIT_la").exists()


def test_regression_never_parks_unresolvable_family():
    # a family with too few held-out tasks (here n=1, like termination/no-data-race) can never park a
    # lever even on a sustained drop — the holdout cannot measure its generalization (power guard).
    with tempfile.TemporaryDirectory() as dd:
        sd = Path(dd)
        (sd / "heldout_weighted_prev").write_text("10")
        (sd / "last_heldout_arm").write_text("0")
        _arms(sd / "arms.jsonl", [_keep(1, "term", 9, family="termination")])
        _holdout(sd / "h1.json", 9, families={"termination": 1})   # down, but termination has 1 task
        msg1 = ha.run(str(sd), 1, str(sd / "h1.json"), min_lift=2, stall_to_park=2)
        assert "PARK-OVERFIT" not in msg1, msg1
        _arms(sd / "arms.jsonl", [_keep(1, "term", 9, family="termination"),
                                  _keep(2, "term", 9, family="termination")])
        _holdout(sd / "h2.json", 8, families={"termination": 1})
        msg2 = ha.run(str(sd), 2, str(sd / "h2.json"), min_lift=2, stall_to_park=2)
        assert "PARK-OVERFIT" not in msg2, msg2
        assert not (sd / "lever.term.parked").exists()


def test_no_weighted_score_is_graceful_noop():
    with tempfile.TemporaryDirectory() as dd:
        sd = Path(dd)
        _arms(sd / "arms.jsonl", [_keep(1, "la", 3)])
        _holdout(sd / "h.json", None)  # legacy scorer: no confirmed_score_weighted
        msg = ha.run(str(sd), 1, str(sd / "h.json"), min_lift=2)
        assert "no weighted score" in msg, msg
        assert not (sd / "heldout_weighted_prev").exists()  # pointers untouched
        assert not (sd / "last_heldout_arm").exists()


def test_sub_threshold_lift_does_not_stall_even_on_regression():
    with tempfile.TemporaryDirectory() as dd:
        sd = Path(dd)
        (sd / "heldout_weighted_prev").write_text("10")
        (sd / "last_heldout_arm").write_text("0")
        _arms(sd / "arms.jsonl", [_keep(1, "la", 1)])  # lift 1 < min_lift 2
        _holdout(sd / "h1.json", 9, families={"famBig": 50})  # a real regression, but lift is sub-threshold
        msg = ha.run(str(sd), 1, str(sd / "h1.json"), min_lift=2)
        assert not (sd / "lever.la.holdout_stall").exists(), msg
        assert not (sd / "lever.la.parked").exists()


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
