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
from svcomp_split import (  # noqa: E402
    assign_groups, origin_group, stable_key, category_root, is_reasoning_task)
from svcomp_split_eval import (  # noqa: E402
    confirmed_score, weighted_confirmed_summary, witness_validator_for)


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


def test_category_root():
    assert category_root("Juliet_Test/CWE190_x") == "Juliet_Test"
    assert category_root("loops") == "loops"
    assert category_root("array-examples/sanfoundry") == "array-examples"


def test_is_reasoning_task_excludes_generators_and_oversized_clusters():
    # a small non-generator cluster -> reasoning (in val)
    sizes = {("loops", "unreach-call"): 40, ("Juliet_Test/CWE190_x", "no-overflow"): 500,
             ("array-examples", "valid-memsafety"): 900}
    assert is_reasoning_task({"group": "loops", "property": "unreach-call"}, sizes) is True
    # denylisted category root -> excluded even if the cluster is small
    assert is_reasoning_task({"group": "Juliet_Test/CWE190_x", "property": "no-overflow"}, sizes) is False
    # oversized cluster (backstop) -> excluded even though its root isn't denylisted
    assert is_reasoning_task({"group": "array-examples", "property": "valid-memsafety"}, sizes) is False
    # honor a custom denylist + threshold
    assert is_reasoning_task({"group": "weaver/x", "property": "no-data-race"}, {}, cluster_max=10) is False


def test_weighted_confirmed_caps_positives_per_cluster_keeps_penalties():
    # cluster A = a Juliet family with 4 confirmed FALSEs (+1 each) -> capped to 1 (dedup)
    res = [{"group": "Juliet/A", "property": "no-overflow", "confirmed": 1, "rel_yml": f"A/{i}"} for i in range(4)]
    # cluster B = one confirmed reasoning task (+1)
    res.append({"group": "loops/B", "property": "unreach-call", "confirmed": 1, "rel_yml": "B/0"})
    # cluster C = a false alarm (-16) that must pass through WHOLE (never hidden by dedup)
    res.append({"group": "arr/C", "property": "valid-memsafety", "confirmed": -16, "rel_yml": "C/0"})
    w = weighted_confirmed_summary(res, cap=1)
    assert w["confirmed_score_weighted"] == 1 + 1 - 16   # A capped to 1, B 1, C -16 whole
    assert w["per_property_weighted"]["no-overflow"] == {"confirmed_weighted": 1, "confirmed_clusters": 1}
    assert w["per_property_weighted"]["unreach-call"]["confirmed_clusters"] == 1
    # raising the cap lets a big cluster count more (up to cap)
    assert weighted_confirmed_summary(res, cap=3)["confirmed_score_weighted"] == 3 + 1 - 16


def test_true_on_witness_required_properties_uses_the_correctness_validator():
    """unreach-call / no-overflow TRUE score 2 only with a CONFIRMED *correctness*
    witness, so the harness must actually run the correctness validator on them.
    Before this existed the harness validated FALSE only, so every TRUE row was
    `witness: None` and scored 0 no matter how good the witness was."""
    for prop, suffix in (("unreach-call", "Loops"), ("no-overflow", "Main")):
        v = witness_validator_for("true", prop, {suffix})
        assert v is not None and v.endswith("validate_correctness_witness.sh"), (prop, v)


def test_termination_true_now_needs_the_correctness_validator():
    """SV-COMP 2027 moved `C.termination.*` from "2.1 (demo mode)" (free) to
    "2.1 or higher" (required). This is the 36-weighted-point change; if this test
    passes under the OLD rule the harness is still scoring the 2026 rulebook."""
    for suffix in ("Other", "MainHeap", "MainControlFlow", "BitVectors"):
        v = witness_validator_for("true", "termination", {suffix})
        assert v is not None and v.endswith("validate_correctness_witness.sh"), suffix


def test_true_on_verdict_only_categories_needs_no_validator():
    """Base categories whose correctness column reads "not supported" or "(demo mode)"
    score on the verdict alone — validating them would be wasted wall-clock."""
    free = (("valid-memsafety", "Heap"), ("valid-memcleanup", "Main"),
            ("no-data-race", "Concurrency"), ("unreach-call", "Arrays"),
            ("unreach-call", "Heap"), ("unreach-call", "Floats"),
            ("no-overflow", "Huawei-Concurrency-Challenges"))
    for prop, suffix in free:
        assert witness_validator_for("true", prop, {suffix}) is None, (prop, suffix)


def test_confirmed_score_gates_termination_true_on_a_confirmed_witness():
    # The whole point of Movement 0A, at the smallest scale that shows it.
    assert confirmed_score("TrueCorrect", "termination", None, {"Other"}) == 0
    assert confirmed_score("TrueCorrect", "termination", "CONFIRMED", {"Other"}) == 2
    # ...while a witness-free category still scores on the verdict alone.
    assert confirmed_score("TrueCorrect", "valid-memsafety", None, {"Heap"}) == 2


def test_confirmed_score_gates_no_data_race_false_on_a_confirmed_witness():
    # 2027 violation column for C.no-data-race is "2.2", not "not supported".
    assert confirmed_score("FalseCorrect", "no-data-race", "NOT_CONFIRMED", {"Concurrency"}) == 0
    assert confirmed_score("FalseCorrect", "no-data-race", "CONFIRMED", {"Concurrency"}) == 1


def test_confirmed_score_keeps_penalties_whole_regardless_of_the_rule():
    # The hard invariant: a witness rule can never turn a wrong verdict into points,
    # so false_alarms / wrong_true are untouched by anything in Movement 0A.
    for suffixes in (set(), {"Other"}, {"Concurrency"}):
        assert confirmed_score("TrueIncorrect", "termination", "CONFIRMED", suffixes) == -32
        assert confirmed_score("FalseIncorrect", "unreach-call", "CONFIRMED", suffixes) == -16
        assert confirmed_score("Unknown", "unreach-call", None, suffixes) == 0


def test_version_floor_is_off_by_default_and_bites_when_asked():
    # SAF emits 2.0; C.unreach-call.Concurrency's violation cell demands 2.2.
    blind = confirmed_score("FalseCorrect", "unreach-call", "CONFIRMED", {"Concurrency"},
                            witness_format="2.0", version_aware=False)
    aware = confirmed_score("FalseCorrect", "unreach-call", "CONFIRMED", {"Concurrency"},
                            witness_format="2.0", version_aware=True)
    assert (blind, aware) == (1, 0)


def test_false_still_uses_the_violation_validator():
    v = witness_validator_for("false", "unreach-call")
    assert v is not None and v.endswith("validate_witness.sh"), v


def test_non_scoring_verdicts_need_no_validator():
    for kind in ("unknown", "timeout", "error"):
        assert witness_validator_for(kind, "unreach-call") is None, kind


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
