#!/usr/bin/env python3
"""Unit test for lib/record.py — the per-arm observability record assembler (plan 205 §1b/§5a/§5b).

record.build() assembles ONE machine-readable row from artifacts the supervisor already produced
(before/after scorer JSON, verdict, worker telemetry, git diffstat) + folds in the task-flip diff
and the worker's tool fingerprint. Pure + deterministic. Expected values are hand-authored.
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
LIB = HERE.parent / "lib"
sys.path.insert(0, str(LIB))
import record as R  # noqa: E402


def _scorer(pairs, confirmed, fp=0, wt=0):
    """pairs: {fam: (confirmed, confirmed_false, false_total)} -> scorer -o JSON shape."""
    per = {f: {"confirmed": c, "confirmed_false": cf, "false_total": ft} for f, (c, cf, ft) in pairs.items()}
    return {"confirmed_score": confirmed, "false_alarms": fp, "wrong_true": wt, "per_property": per}


def test_build_assembles_per_property_delta_and_worker_cost():
    before = _scorer({"unreach-call": (100, 100, 200), "valid-memsafety": (50, 50, 120)}, 150)
    after = _scorer({"unreach-call": (103, 103, 200), "valid-memsafety": (50, 50, 120)}, 153, fp=1)
    verdict = {"decision": "KEEP", "regressed_families": [], "revert_reason": None}
    result = {"subtype": "success", "num_turns": 31, "total_cost_usd": 1.9, "duration_ms": 1000,
              "api_retries": 0, "hit_max_turns": False, "summary": "widened the reach interval"}
    diffstat = {"files": 2, "insertions": 40, "deletions": 3, "paths": ["crates/saf-svcomp/src/lib.rs"]}
    row = R.build(n=14, lever="overflow-recall", mode="tuning", family="no-overflow", scope="local",
                  decision="KEEP", delta=3, progressed=True, before=before, after=after,
                  verdict=verdict, result=result, diffstat=diffstat, ts="2026-08-16T00:00:00Z")
    assert row["arm"] == 14 and row["lever"] == "overflow-recall" and row["decision"] == "KEEP"
    assert row["confirmed_delta"] == 3 and row["progressed"] is True
    assert row["per_property_before"] == {"unreach-call": 100, "valid-memsafety": 50}, row["per_property_before"]
    assert row["per_property_after"] == {"unreach-call": 103, "valid-memsafety": 50}
    assert row["per_property_delta"] == {"unreach-call": 3, "valid-memsafety": 0}
    assert row["per_property_recall_after"]["unreach-call"] == "103/200"
    assert row["overall_confirmed_before"] == 150 and row["overall_confirmed_after"] == 153
    assert row["false_alarms_after"] == 1
    assert row["worker"]["num_turns"] == 31 and abs(row["worker"]["total_cost_usd"] - 1.9) < 1e-9
    assert row["diff"] == diffstat
    assert row["worker_summary"] == "widened the reach interval"
    assert row["regressed_families"] == []


def test_build_broken_build_missing_after_is_recorded_not_crashed():
    before = _scorer({"no-overflow": (10, 10, 40)}, 10)
    row = R.build(n=9, lever="x", mode="tuning", family="no-overflow", scope="local",
                  decision="REVERT", delta=-(10 ** 9), progressed=False, before=before, after={},
                  verdict={"revert_reason": "build_broken"}, result={}, diffstat={}, ts="t")
    assert row["overall_confirmed_after"] is None
    assert row["per_property_after"] == {}
    assert row["confirmed_delta"] == -(10 ** 9)
    assert row["revert_reason"] == "build_broken"


def test_build_folds_in_flips_and_fingerprint():
    fl = {"gained_confirmed": ["t/new.yml"], "lost_confirmed": [], "new_false_alarms": []}
    fp = {"edits": 1, "writes": 0, "bash": 2, "ran_tests": True, "ran_clippy": False, "files": ["a.rs"]}
    row = R.build(n=1, lever="l", mode="tuning", family="f", scope="local", decision="KEEP",
                  delta=1, progressed=True, before={}, after={}, verdict={}, result={},
                  diffstat={}, ts="t", flips=fl, fingerprint=fp)
    assert row["flips"] == fl
    assert row["fingerprint"] == fp


def test_fingerprint_counts_tools_and_detects_self_checks():
    tx = [
        {"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Edit", "input": {"file_path": "crates/x/src/a.rs"}}]}},
        {"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Write", "input": {"file_path": "crates/x/src/b.rs"}}]}},
        {"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Bash", "input": {"command": "cargo nextest run -p saf-svcomp"}}]}},
        {"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Bash", "input": {"command": "cargo clippy --all-targets"}}]}},
    ]
    with tempfile.TemporaryDirectory() as dd:
        p = Path(dd) / "t.jsonl"
        p.write_text("\n".join(json.dumps(x) for x in tx))
        f = R.fingerprint(str(p))
        assert f["edits"] == 1 and f["writes"] == 1 and f["bash"] == 2, f
        assert f["ran_tests"] is True and f["ran_clippy"] is True, f
        assert f["files"] == ["crates/x/src/a.rs", "crates/x/src/b.rs"], f


def test_fingerprint_missing_transcript_is_empty():
    f = R.fingerprint("/no/such.jsonl")
    assert f["edits"] == 0 and f["writes"] == 0 and f["bash"] == 0
    assert f["ran_tests"] is False and f["files"] == []


def test_build_output_is_deterministic():
    before = _scorer({"b": (1, 1, 2), "a": (2, 2, 3)}, 3)
    after = _scorer({"a": (2, 2, 3), "b": (2, 2, 2)}, 4)
    kw = dict(n=1, lever="l", mode="tuning", family="a", scope="local", decision="KEEP",
              delta=1, progressed=True, before=before, after=after, verdict={}, result={},
              diffstat={}, ts="t")
    s1 = json.dumps(R.build(**kw), sort_keys=True)
    s2 = json.dumps(R.build(**kw), sort_keys=True)
    assert s1 == s2


def test_cli_roundtrip_writes_full_row():
    with tempfile.TemporaryDirectory() as dd:
        d = Path(dd)
        before = _scorer({"unreach-call": (100, 100, 200)}, 100)
        after = _scorer({"unreach-call": (101, 101, 200)}, 101)
        (d / "before.json").write_text(json.dumps(before))
        (d / "after.json").write_text(json.dumps(after))
        (d / "verdict.json").write_text(json.dumps({"decision": "KEEP", "revert_reason": None}))
        (d / "result.json").write_text(json.dumps({"num_turns": 5, "total_cost_usd": 0.2, "summary": "s"}))
        # per-task dumps: one task flips unknown -> confirmed
        (d / "bpt.jsonl").write_text(json.dumps({"rel_yml": "t/x.yml", "outcome": "unknown", "witness": None}) + "\n")
        (d / "apt.jsonl").write_text(json.dumps({"rel_yml": "t/x.yml", "outcome": "FalseCorrect", "witness": "CONFIRMED"}) + "\n")
        # a transcript with one edit
        (d / "t.jsonl").write_text(json.dumps({"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Edit", "input": {"file_path": "crates/x/a.rs"}}]}}) + "\n")
        scalars = json.dumps(dict(n=3, lever="l", mode="tuning", family="unreach-call",
                                  scope="local", decision="KEEP", delta=1, progressed=1, ts="t"))
        diffstat = json.dumps({"files": 1, "insertions": 5, "deletions": 0, "paths": ["crates/x/a.rs"]})
        cli = str(LIB / "record.py")
        out = subprocess.run([sys.executable, cli, scalars,
                              str(d / "before.json"), str(d / "after.json"),
                              str(d / "verdict.json"), str(d / "result.json"), diffstat,
                              str(d / "t.jsonl"), str(d / "bpt.jsonl"), str(d / "apt.jsonl")],
                             capture_output=True, text=True)
        assert out.returncode == 0, out.stderr
        row = json.loads(out.stdout)
        assert row["arm"] == 3 and row["confirmed_delta"] == 1
        assert row["flips"]["gained_confirmed"] == ["t/x.yml"], row["flips"]
        assert row["fingerprint"]["edits"] == 1, row["fingerprint"]
        assert row["diff"]["files"] == 1


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
