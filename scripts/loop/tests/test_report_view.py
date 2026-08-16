#!/usr/bin/env python3
"""Unit test for lib/report_view.py — derived read-only views over .loop-state/arms.jsonl
(plan 205 §2c/§4b/§5c). Every number is recomputed from the spine, so a view can never disagree
with recorded history. Pure; no side effects. Expected values are hand-authored.
"""
import json
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
LIB = HERE.parent / "lib"
sys.path.insert(0, str(LIB))
import report_view as rv  # noqa: E402


def _arm(n, lever, decision, delta=0, mode="tuning", cost=1.0, turns=10, maxturns=False,
         paths=None, files=2, insertions=10, summary=""):
    return {"arm": n, "lever": lever, "mode": mode, "decision": decision,
            "confirmed_delta": delta,
            "worker": {"total_cost_usd": cost, "num_turns": turns, "hit_max_turns": maxturns},
            "diff": {"files": files, "insertions": insertions, "paths": paths or []},
            "worker_summary": summary}


ROWS = [
    _arm(1, "overflow-recall", "KEEP", delta=3, cost=1.0, paths=["a", "b"]),
    _arm(2, "overflow-recall", "KEEP", delta=2, cost=2.0, paths=["a", "b"]),         # same paths -> re-derivation
    _arm(3, "air-slicing", "REVERT", mode="capability", cost=3.0, turns=60, maxturns=True,
         paths=["p%d" % i for i in range(9)], files=9, insertions=820),             # big revert + max_turns
    _arm(4, "air-slicing", "REVERT", mode="capability", cost=3.0),
    _arm(5, "air-slicing", "REVERT", mode="capability", cost=3.0),
    _arm(6, "air-slicing", "ACCUMULATE_PLUS", mode="capability", cost=1.0, insertions=470,
         summary="built the slicing pass"),
    _arm(7, "mem-recall", "KEEP_POOL", delta=0, cost=1.0),
]


def test_lever_roi_buckets_and_dollars_per_point():
    roi = rv.lever_roi(ROWS)
    o = roi["overflow-recall"]
    assert o["arms"] == 2 and o["KEEP"] == 2 and o["netDelta"] == 5, o
    assert o["cost"] == 3.0 and o["dollars_per_point"] == 0.6, o
    a = roi["air-slicing"]
    assert a["arms"] == 4 and a["REVERT"] == 3 and a["ACCUMULATE"] == 1, a  # ACCUMULATE_PLUS -> ACCUMULATE bucket
    assert a["netDelta"] == 0 and a["dollars_per_point"] is None, a
    assert a["maxturns"] == 1, a
    # KEEP_POOL is a KEEP-bucket win (banks points)
    m = roi["mem-recall"]
    assert m["KEEP"] == 1, m
    # sorted by (-netDelta, cost): overflow-recall (netΔ5) first
    assert list(roi.keys())[0] == "overflow-recall", list(roi.keys())


def test_waste_flags_stuck_maxturns_bigreverts_and_rederivation():
    w = rv.waste(ROWS)
    assert "air-slicing" in w["stuck_levers"], w             # >=3 REVERTs, net<=0
    assert "overflow-recall" not in w["stuck_levers"], w
    assert 3 in w["max_turns_arms"], w
    assert any(b[0] == 3 for b in w["big_reverts"]), w       # arm 3 has 9 files
    assert "overflow-recall" in w["rederived"], w
    assert sorted(w["rederived"]["overflow-recall"]) == [1, 2], w


def test_capability_milestones_track_accumulation():
    caps = rv.capability_milestones(ROWS)
    assert "air-slicing" in caps, caps
    c = caps["air-slicing"]
    assert c["arms"] == 4 and c["accumulate_arms"] == 1, c
    assert c["last_summary"] == "built the slicing pass", c
    assert "overflow-recall" not in caps  # tuning lever, not capability


def test_holdout_trend_reads_sorted_by_arm_number():
    with tempfile.TemporaryDirectory() as dd:
        st = Path(dd)
        (st / "heldout-3.json").write_text(json.dumps({"confirmed_score": 8, "false_alarms": 0, "wrong_true": 0}))
        (st / "heldout-1.json").write_text(json.dumps({"confirmed_score": 7, "false_alarms": 0, "wrong_true": 0}))
        (st / "heldout-10.json").write_text(json.dumps({"confirmed_score": 9, "false_alarms": 0, "wrong_true": 0}))
        tr = rv.holdout_trend(str(st))
        # numeric sort by the arm suffix (1,3,10 — NOT lexicographic 1,10,3)
        assert [p[1] for p in tr] == [7, 8, 9], tr


def test_lift_since_sums_keep_bucket_deltas_after_checkpoint():
    # per-lever val lift banked AFTER a given checkpoint arm — the holdout brake's attribution input.
    lift = rv.lift_since(ROWS, since_arm=0)
    assert lift["overflow-recall"] == 5, lift          # arms 1(+3)+2(+2), both KEEP
    assert lift["mem-recall"] == 0, lift               # arm 7 KEEP_POOL, delta 0 -> KEEP bucket, +0
    assert "air-slicing" not in lift, lift             # only REVERT/ACCUMULATE_PLUS -> not a KEEP bucket
    # a later checkpoint excludes earlier arms
    lift2 = rv.lift_since(ROWS, since_arm=1)
    assert lift2.get("overflow-recall") == 2, lift2    # only arm 2 counts now
    # nothing after the last arm
    assert rv.lift_since(ROWS, since_arm=7) == {}, "no arms after the last checkpoint"


def test_recent_returns_last_n():
    r = rv.recent(ROWS, 2)
    assert [x["arm"] for x in r] == [6, 7], r


def test_roi_oneline_summarizes_top_payers():
    line = rv.roi_oneline(ROWS, top=2)
    assert line.startswith("lever ROI:"), line
    assert "overflow-recall" in line and "+5" in line, line   # top payer, netΔ 5
    assert rv.roi_oneline([], top=3) == "lever ROI: n/a"


def test_load_arms_tolerates_garbage_and_missing():
    with tempfile.TemporaryDirectory() as dd:
        p = Path(dd) / "arms.jsonl"
        p.write_text(json.dumps(ROWS[0]) + "\nnot json\n\n" + json.dumps(ROWS[1]) + "\n")
        rows = rv.load_arms(str(p))
        assert [x["arm"] for x in rows] == [1, 2], rows
        assert rv.load_arms(str(Path(dd) / "nope.jsonl")) == []


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
