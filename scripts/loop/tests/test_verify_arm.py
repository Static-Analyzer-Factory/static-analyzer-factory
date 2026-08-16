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


def _run(root: Path, before, after, manifest, transcript_lines, forbidden, progressed=0,
         check_all_families=False):
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
    if check_all_families:
        argv += ["--check-all-families"]
    for f in forbidden:
        argv += ["--forbidden", f]
    r = subprocess.run(argv, capture_output=True, text=True)
    return r.stdout.strip().splitlines()[-1] if r.stdout.strip() else f"(no-output rc={r.returncode} err={r.stderr[-200:]})"


def _scorep(**fams):
    per = {f: {"confirmed": c} for f, c in fams.items()}
    return {"confirmed_score": sum(fams.values()), "false_alarms": 0, "wrong_true": 0,
            "max_score": 1000, "per_property": per}


def _scorew(cw):
    """A weighted (deduped) scorer JSON — the shape --group-weight produces."""
    return {"confirmed_score": cw, "confirmed_score_weighted": cw,
            "false_alarms": 0, "wrong_true": 0, "max_score": 1000}


def _run_gen(root, val_before, val_after, pool_before, pool_after, manifest, transcript_lines,
             forbidden, novel=0):
    d = root
    (d / "vb.json").write_text(json.dumps(val_before))
    (d / "va.json").write_text(json.dumps(val_after) if val_after is not None else "")
    if val_after is None:
        (d / "va.json").unlink(missing_ok=True)
    (d / "pb.json").write_text(json.dumps(pool_before))
    (d / "pa.json").write_text(json.dumps(pool_after))
    (d / "manifest.json").write_text(json.dumps(manifest))
    (d / "t.jsonl").write_text("\n".join(transcript_lines))
    argv = [sys.executable, str(CLI), "--gen-mode",
            "--before", str(d / "vb.json"), "--after", str(d / "va.json"),
            "--pool-before", str(d / "pb.json"), "--pool-after", str(d / "pa.json"),
            "--immutable-manifest", str(d / "manifest.json"), "--repo-root", str(d),
            "--transcript", str(d / "t.jsonl"), "--novel-solved", str(novel)]
    for f in forbidden:
        argv += ["--forbidden", f]
    r = subprocess.run(argv, capture_output=True, text=True)
    return r.stdout.strip().splitlines()[-1] if r.stdout.strip() else f"(no-output rc={r.returncode} err={r.stderr[-200:]})"


def test_verify_arm_gen_mode():
    with tempfile.TemporaryDirectory() as dd:
        root = Path(dd)
        imm = root / "scorer.py"; imm.write_text("SCORER\n")
        good = {"scorer.py": hashlib.sha256(imm.read_bytes()).hexdigest()}
        clean = [json.dumps({"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Read", "input": {"file_path": "splits/val.jsonl"}}]}})]
        F = ["splits/holdout"]
        # reasoning-set gain, pool flat -> KEEP
        assert _run_gen(root, _scorew(40), _scorew(43), _scorew(200), _scorew(200), good, clean, F) == "KEEP"
        # val flat, pool up (memorized Juliet points) -> KEEP_POOL
        assert _run_gen(root, _scorew(40), _scorew(40), _scorew(200), _scorew(205), good, clean, F) == "KEEP_POOL"
        # val up but the deduped pool REGRESSED -> REVERT (never trade pool away)
        assert _run_gen(root, _scorew(40), _scorew(45), _scorew(200), _scorew(198), good, clean, F) == "REVERT"
        # capability novel-solving progress, net-neutral -> ACCUMULATE_PLUS
        assert _run_gen(root, _scorew(40), _scorew(40), _scorew(200), _scorew(200), good, clean, F, novel=1) == "ACCUMULATE_PLUS"
        # broken val eval (no after) -> big-negative gen delta -> REVERT
        assert _run_gen(root, _scorew(40), None, _scorew(200), _scorew(200), good, clean, F) == "REVERT"
        # holdout read still rejected in gen mode
        holdout_t = [json.dumps({"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Read", "input": {"file_path": "splits/holdout.jsonl"}}]}})]
        assert _run_gen(root, _scorew(40), _scorew(43), _scorew(200), _scorew(200), good, holdout_t, F) == "REJECT_HOLDOUT"


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

        # CROSS-CUTTING arm, all-property eval: overall +20 but memsafety dropped -> REVERT (no trades)
        b = _scorep(**{"valid-memsafety": 200, "unreach-call": 100})
        a_trade = _scorep(**{"valid-memsafety": 190, "unreach-call": 130})   # total +20, memsafety -10
        assert _run(root, b, a_trade, good, clean_t, F, check_all_families=True) == "REVERT"
        # a Pareto improvement under the same all-property gate -> KEEP
        a_pareto = _scorep(**{"valid-memsafety": 205, "unreach-call": 130})
        assert _run(root, b, a_pareto, good, clean_t, F, check_all_families=True) == "KEEP"
        # WITHOUT the flag (a local arm), the per-family drop is invisible; total +20 -> KEEP
        assert _run(root, b, a_trade, good, clean_t, F, check_all_families=False) == "KEEP"


def _run_verdict(root: Path, before, after, manifest, transcript_lines, forbidden, progressed=0,
                 check_all_families=False):
    """Like _run but also returns the parsed verdict.json (for the revert_reason tag, §5d)."""
    d = root
    (d / "before.json").write_text(json.dumps(before))
    if after is not None:
        (d / "after.json").write_text(json.dumps(after))
    else:
        (d / "after.json").unlink(missing_ok=True)
    (d / "manifest.json").write_text(json.dumps(manifest))
    (d / "t.jsonl").write_text("\n".join(transcript_lines))
    argv = [sys.executable, str(CLI),
            "--before", str(d / "before.json"), "--after", str(d / "after.json"),
            "--immutable-manifest", str(d / "manifest.json"), "--repo-root", str(d),
            "--transcript", str(d / "t.jsonl"), "--progressed", str(progressed),
            "--verdict-out", str(d / "verdict.json")]
    if check_all_families:
        argv += ["--check-all-families"]
    for f in forbidden:
        argv += ["--forbidden", f]
    subprocess.run(argv, capture_output=True, text=True)
    return json.loads((d / "verdict.json").read_text())


def test_revert_reason_pure_function():
    import importlib.util
    spec = importlib.util.spec_from_file_location("verify_arm", CLI)
    va = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(va)
    # a keep-family decision has no revert reason
    assert va.revert_reason(decision="KEEP", after_present=True, family_regressed=False, delta=3) is None
    assert va.revert_reason(decision="ACCUMULATE", after_present=True, family_regressed=False, delta=0) is None
    # anti-cheat rejects
    assert va.revert_reason(decision="REJECT_TAMPER", after_present=True, family_regressed=False, delta=0) == "tamper"
    assert va.revert_reason(decision="REJECT_HOLDOUT", after_present=True, family_regressed=False, delta=0) == "holdout"
    # REVERT causes, most-proximate first
    assert va.revert_reason(decision="REVERT", after_present=False, family_regressed=False, delta=-10**9) == "build_broken"
    assert va.revert_reason(decision="REVERT", after_present=True, family_regressed=True, delta=5) == "family_trade"
    assert va.revert_reason(decision="REVERT", after_present=True, family_regressed=False, delta=-3) == "regression"
    assert va.revert_reason(decision="REVERT", after_present=True, family_regressed=False, delta=0) == "no_gain"


def test_revert_reason_written_into_verdict():
    with tempfile.TemporaryDirectory() as dd:
        root = Path(dd)
        imm = root / "scorer.py"; imm.write_text("SCORER\n")
        good = {"scorer.py": hashlib.sha256(imm.read_bytes()).hexdigest()}
        clean = [json.dumps({"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Read", "input": {"file_path": "splits/train.jsonl"}}]}})]
        F = ["splits/holdout"]
        # score-neutral no-op -> REVERT tagged no_gain
        v = _run_verdict(root, _score(100), _score(100), good, clean, F, progressed=0)
        assert v["decision"] == "REVERT" and v["revert_reason"] == "no_gain", v
        # broken build (no after.json) -> build_broken
        v2 = _run_verdict(root, _score(100), None, good, clean, F, progressed=1)
        assert v2["revert_reason"] == "build_broken", v2
        # a KEEP has revert_reason null
        v3 = _run_verdict(root, _score(100), _score(101), good, clean, F)
        assert v3["decision"] == "KEEP" and v3["revert_reason"] is None, v3
        # crosscut family trade -> family_trade
        b = _scorep(**{"valid-memsafety": 200, "unreach-call": 100})
        a = _scorep(**{"valid-memsafety": 190, "unreach-call": 130})
        v4 = _run_verdict(root, b, a, good, clean, F, check_all_families=True)
        assert v4["decision"] == "REVERT" and v4["revert_reason"] == "family_trade", v4


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
