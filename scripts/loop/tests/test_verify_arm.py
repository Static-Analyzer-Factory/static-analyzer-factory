#!/usr/bin/env python3
"""Integration test for the bash<->python boundary: scripts/loop/lib/verify_arm.py.

This is the exact CLI supervisor.sh shells out to after an arm runs. It exercises file IO +
argparse + the (already unit-tested) gate functions end to end, and prints ONE decision word.
"""
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
CLI = HERE.parent / "lib" / "verify_arm.py"


def _score(confirmed=0, fp=0, wt=0):
    return {"confirmed_score": confirmed, "false_alarms": fp, "wrong_true": wt, "max_score": 100}


def _run(root: Path, before, after, manifest, transcript_lines, forbidden, progressed=0):
    d = root
    (d / "before.json").write_text(json.dumps(before))
    if after is not None:
        (d / "after.json").write_text(json.dumps(after))
    else:
        (d / "after.json").unlink(missing_ok=True)   # simulate a broken-build arm: no after.json
    (d / "manifest.json").write_text(json.dumps(manifest))
    (d / "t.jsonl").write_text("\n".join(transcript_lines))
    argv = [sys.executable, str(CLI),
            "--before", str(d / "before.json"),
            "--after", str(d / "after.json"),
            "--immutable-manifest", str(d / "manifest.json"),
            "--repo-root", str(d),
            "--transcript", str(d / "t.jsonl"),
            "--progressed", str(progressed)]
    for f in forbidden:
        argv += ["--forbidden", f]
    r = subprocess.run(argv, capture_output=True, text=True)
    return r.stdout.strip().splitlines()[-1] if r.stdout.strip() else f"(no-output rc={r.returncode} err={r.stderr[-200:]})"


def test_verify_arm_end_to_end_decisions():
    with tempfile.TemporaryDirectory() as dd:
        root = Path(dd)
        imm = root / "scorer.py"
        imm.write_text("SCORER\n")
        good = {"scorer.py": hashlib.sha256(imm.read_bytes()).hexdigest()}
        tamper = {"scorer.py": "deadbeef" * 8}  # wrong hash -> tamper
        clean_t = [json.dumps({"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Read", "input": {"file_path": "splits/train.jsonl"}}]}})]
        holdout_t = [json.dumps({"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Read", "input": {"file_path": "splits/holdout.jsonl"}}]}})]
        F = ["splits/holdout"]

        # score improved -> KEEP (progressed irrelevant)
        assert _run(root, _score(3960), _score(3961), good, clean_t, F) == "KEEP"
        # a BIG real gain that also carries an FP still nets positive -> KEEP (aim for soundness, big gains ok)
        assert _run(root, _score(3960), _score(4100, fp=1), good, clean_t, F) == "KEEP"
        # score-NEUTRAL + useful (compiles/tests/diff -> progressed=1) -> ACCUMULATE (preserved for future arms)
        assert _run(root, _score(3960), _score(3960), good, clean_t, F, progressed=1) == "ACCUMULATE"
        # score-neutral, NOT useful (progressed=0) -> REVERT
        assert _run(root, _score(3960), _score(3960), good, clean_t, F, progressed=0) == "REVERT"
        # an FP lowers the score -> regression -> REVERT even if it 'progressed' (never lower the score)
        assert _run(root, _score(3960), _score(3950, fp=1), good, clean_t, F, progressed=1) == "REVERT"
        # broken build: no after.json -> huge-negative delta -> REVERT (build health is the agent's job)
        assert _run(root, _score(3960), None, good, clean_t, F, progressed=1) == "REVERT"
        # tamper an immutable -> REJECT_TAMPER (even with a gain)
        assert _run(root, _score(3960), _score(3961), tamper, clean_t, F) == "REJECT_TAMPER"
        # arm read the holdout -> REJECT_HOLDOUT
        assert _run(root, _score(3960), _score(3961), good, holdout_t, F) == "REJECT_HOLDOUT"


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
