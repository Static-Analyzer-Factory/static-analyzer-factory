#!/usr/bin/env python3
"""Unit tests for the pure split core of scripts/svcomp_split.py (plan 203).

These test ONLY the leakage-invariant / determinism / stratification logic on
synthetic task lists — no sv-benchmarks checkout, no git, no LLVM required, so
they run locally with plain `python3`:

    python3 scripts/test_svcomp_split.py            # or: pytest scripts/test_svcomp_split.py
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from svcomp_split import assign_groups, origin_group, stable_key  # noqa: E402


def _synthetic_tasks():
    """3 origins per property x 5 properties, uneven sizes, plus one mixed-property
    origin (`shared`) that declares two properties — exercises the dominant-property
    bucketing and the no-straddle guarantee for cross-property groups."""
    props = ["unreach-call", "valid-memsafety", "no-overflow",
             "termination", "no-data-race"]
    tasks = []
    for p in props:
        for gi in range(3):
            g = f"{p}-origin{gi}"
            for k in range(10 + gi * 5):  # 10, 15, 20 tasks
                tasks.append({"group": g, "property": p,
                              "expected": (k % 2 == 0)})
    # A single origin that mixes two properties (dominant = unreach-call, 8 vs 3).
    for k in range(8):
        tasks.append({"group": "shared", "property": "unreach-call",
                      "expected": False})
    for k in range(3):
        tasks.append({"group": "shared", "property": "no-overflow",
                      "expected": False})
    return tasks


def test_origin_group_depth():
    assert origin_group("Juliet_Test/CWE190_x/s01", 2) == "Juliet_Test/CWE190_x"
    assert origin_group("array-examples", 2) == "array-examples"
    assert origin_group("a/b/c/d", 1) == "a"
    assert origin_group("a/b/c/d", 3) == "a/b/c"
    assert origin_group("", 2) == "."


def test_stable_key_is_deterministic():
    # Stable across calls (and, by construction, across processes/machines).
    assert stable_key("foo", 0) == stable_key("foo", 0)
    assert stable_key("foo", 0) != stable_key("foo", 1)
    assert stable_key("foo", 0) != stable_key("bar", 0)


def test_no_group_straddles_split():
    """The core anti-leakage invariant: every origin group is wholly train XOR
    wholly holdout."""
    tasks = _synthetic_tasks()
    side = assign_groups(tasks, holdout_frac=0.2, seed=0)
    all_groups = {t["group"] for t in tasks}
    assert set(side) == all_groups
    assert all(side[g] in ("train", "holdout") for g in side)
    # Reconstruct per-task side from the group map; no group appears on both sides.
    train_groups = {g for g, s in side.items() if s == "train"}
    holdout_groups = {g for g, s in side.items() if s == "holdout"}
    assert train_groups.isdisjoint(holdout_groups)
    assert train_groups | holdout_groups == all_groups


def test_determinism_same_seed():
    tasks = _synthetic_tasks()
    a = assign_groups(tasks, 0.2, seed=7)
    b = assign_groups(tasks, 0.2, seed=7)
    assert a == b


def test_seed_changes_split():
    tasks = _synthetic_tasks()
    a = assign_groups(tasks, 0.2, seed=1)
    b = assign_groups(tasks, 0.2, seed=2)
    # Different seeds should generally produce a different assignment.
    assert a != b


def test_holdout_fraction_is_approximately_met():
    """Each property should land near the requested holdout fraction (grouping makes
    it approximate, never exact — we allow a generous band and require it is neither
    empty nor everything)."""
    tasks = _synthetic_tasks()
    frac = 0.34
    side = assign_groups(tasks, frac, seed=0)
    per_prop_total = {}
    per_prop_holdout = {}
    for t in tasks:
        per_prop_total[t["property"]] = per_prop_total.get(t["property"], 0) + 1
        if side[t["group"]] == "holdout":
            per_prop_holdout[t["property"]] = per_prop_holdout.get(t["property"], 0) + 1
    for p, tot in per_prop_total.items():
        got = per_prop_holdout.get(p, 0) / tot
        assert 0.0 < got < 0.75, f"{p}: holdout frac {got:.2f} out of band"


def test_frac_zero_and_one_extremes():
    tasks = _synthetic_tasks()
    none = assign_groups(tasks, 0.0, seed=0)
    assert all(s == "train" for s in none.values())
    allout = assign_groups(tasks, 1.0, seed=0)
    # frac=1.0 targets every task; with whole-group assignment all groups move over.
    assert all(s == "holdout" for s in allout.values())


def _run_all():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    failed = 0
    for t in tests:
        try:
            t()
            print(f"  PASS {t.__name__}")
        except AssertionError as e:
            failed += 1
            print(f"  FAIL {t.__name__}: {e}")
    print(f"\n{len(tests) - failed}/{len(tests)} passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(_run_all())
