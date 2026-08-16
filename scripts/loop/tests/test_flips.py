#!/usr/bin/env python3
"""Unit test for lib/flips.py — the per-arm task-flip instrument (plan 205 §5a).

flips() diffs the scorer's before/after --per-task JSONL dumps by `rel_yml` (the stable task id)
and tells the operator EXACTLY which individual tasks changed outcome: solved-new vs re-confirmed
vs traded-for-a-false-alarm. Expected values below are hand-authored, not recomputed by the code.
"""
import json
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "lib"))
import flips as F  # noqa: E402


def _write(path: Path, rows: list[dict]) -> str:
    path.write_text("".join(json.dumps(r) + "\n" for r in rows))
    return str(path)


def _row(rel, outcome, witness=None):
    return {"rel_yml": rel, "property": "unreach-call", "outcome": outcome, "witness": witness}


def test_flips_gained_lost_and_new_fp():
    with tempfile.TemporaryDirectory() as dd:
        d = Path(dd)
        before = _write(d / "b.jsonl", [
            _row("t/already.yml", "FalseCorrect", "CONFIRMED"),  # confirmed in both — NOT gained
            _row("t/regress.yml", "FalseCorrect", "CONFIRMED"),  # confirmed before, lost after
            _row("t/miss.yml", "unknown"),                        # never solved
            _row("t/becomes_fp.yml", "unknown"),                  # will become a false alarm
        ])
        after = _write(d / "a.jsonl", [
            _row("t/already.yml", "FalseCorrect", "CONFIRMED"),  # unchanged
            _row("t/regress.yml", "unknown"),                     # LOST its confirmation
            _row("t/miss.yml", "FalseCorrect", "CONFIRMED"),      # GAINED (new solve)
            _row("t/becomes_fp.yml", "FalseIncorrect"),           # NEW false alarm
        ])
        out = F.flips(before, after)
        assert out["gained_confirmed"] == ["t/miss.yml"], out
        assert out["lost_confirmed"] == ["t/regress.yml"], out
        assert out["new_false_alarms"] == ["t/becomes_fp.yml"], out


def test_flips_raw_correct_but_unconfirmed_is_not_a_gain():
    # A FALSE that is correct but whose witness was NOT confirmed earns 0 points — it must NOT
    # count as a gained (solved) task. Only outcome==FalseCorrect AND witness==CONFIRMED counts.
    with tempfile.TemporaryDirectory() as dd:
        d = Path(dd)
        before = _write(d / "b.jsonl", [_row("t/x.yml", "unknown")])
        after = _write(d / "a.jsonl", [_row("t/x.yml", "FalseCorrect", "NOT_CONFIRMED")])
        out = F.flips(before, after)
        assert out["gained_confirmed"] == [], out
        assert out["lost_confirmed"] == [], out
        assert out["new_false_alarms"] == [], out


def test_flips_missing_after_reports_nothing_not_mass_regression():
    # A broken-build arm never writes after.pertask.jsonl. With a real `before` (confirmed tasks) and a
    # MISSING `after`, flips must report NOTHING — NOT flag every before-confirmed task as lost (which
    # would fabricate a full mass regression, the exact bug the plan-205 §5a review caught).
    with tempfile.TemporaryDirectory() as dd:
        d = Path(dd)
        before = _write(d / "b.jsonl", [_row("t/x.yml", "FalseCorrect", "CONFIRMED"),
                                        _row("t/y.yml", "FalseCorrect", "CONFIRMED")])
        out = F.flips(before, str(d / "does_not_exist.jsonl"))
        assert out == {"gained_confirmed": [], "lost_confirmed": [], "new_false_alarms": []}, out
        # both missing -> all empty, no crash
        out2 = F.flips(str(d / "nope1.jsonl"), str(d / "nope2.jsonl"))
        assert out2 == {"gained_confirmed": [], "lost_confirmed": [], "new_false_alarms": []}, out2


def test_flips_sorted_and_deterministic():
    with tempfile.TemporaryDirectory() as dd:
        d = Path(dd)
        before = _write(d / "b.jsonl", [_row("t/z.yml", "unknown"), _row("t/a.yml", "unknown")])
        after = _write(d / "a.jsonl", [
            _row("t/z.yml", "FalseCorrect", "CONFIRMED"),
            _row("t/a.yml", "FalseCorrect", "CONFIRMED"),
        ])
        out = F.flips(before, after)
        assert out["gained_confirmed"] == ["t/a.yml", "t/z.yml"], out  # sorted, not insertion order


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
