#!/usr/bin/env python3
"""Re-score the authoritative dump through the REAL harness functions.

Validates the 0A integration end-to-end: imports `confirmed_score` and
`weighted_confirmed_summary` from `scripts/svcomp_split_eval.py` itself, rather
than reimplementing them, so a divergence between module and harness shows up.

Writes the published artifact. Run from the repo root on cd-vm-15.
"""
import collections
import json
import subprocess
import sys

sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score, weighted_confirmed_summary  # noqa: E402
from svcomp_witness_rules import base_categories, load_set_membership  # noqa: E402

TRUE_FREE_2026 = {"termination", "valid-memsafety", "valid-memcleanup", "no-data-race"}
FALSE_FREE_2026 = {"no-data-race"}
SCORE = {"TrueCorrect": 2, "FalseCorrect": 1, "TrueIncorrect": -32,
         "FalseIncorrect": -16, "Unknown": 0}


def score_2026(r):
    """The rule this harness implemented before Movement 0A, kept here so the
    published delta is measured rather than remembered."""
    o = r["outcome"]
    if o == "FalseCorrect":
        return 1 if r["property"] in FALSE_FREE_2026 else (1 if r["witness"] == "CONFIRMED" else 0)
    if o == "TrueCorrect":
        return 2 if r["property"] in TRUE_FREE_2026 else (2 if r["witness"] == "CONFIRMED" else 0)
    return SCORE[o]


def emitted_format(r):
    """SAF's witness format, verified in-source: YAML 2.0 everywhere
    (witness_yaml.rs:31, correctness_witness.rs), GraphML 1.0 on the concurrency
    violation path. The dump predates the `witness_format` field."""
    if r.get("witness_format"):
        return r["witness_format"]
    if r["kind"] != "false":
        return "2.0"
    return "graphml-1.0" if r["property"] == "no-data-race" else "2.0"


def main():
    rows = [json.loads(l) for l in open("lever1-pertask.jsonl")]
    fix = {}
    for l in open("lever1-true99-pertask.jsonl"):
        r = json.loads(l)
        fix[(r["rel_yml"], r["property"], r["data_model"])] = r
    spliced = [fix.get((r["rel_yml"], r["property"], r["data_model"]), r) for r in rows]

    membership = load_set_membership("tests/benchmarks/sv-benchmarks")
    for r in spliced:
        sets = membership.get(r["rel_yml"], frozenset())
        sufs = base_categories(r["property"], sets)
        fmt = emitted_format(r)
        r["sets"] = sorted(sets)
        r["base_categories"] = sorted(sufs)
        r["witness_format"] = fmt
        r["confirmed_2026"] = score_2026(r)
        r["confirmed"] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                         sufs, r.get("sub"), fmt, version_aware=False)
        r["confirmed_va"] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                            sufs, r.get("sub"), fmt, version_aware=True)

    w26 = weighted_confirmed_summary(spliced, 1, field="confirmed_2026")
    w27 = weighted_confirmed_summary(spliced, 1, field="confirmed")
    w27va = weighted_confirmed_summary(spliced, 1, field="confirmed_va")
    in_comp = [r for r in spliced if r["base_categories"]]

    fa = sum(1 for r in spliced if r["outcome"] == "FalseIncorrect")
    wt = sum(1 for r in spliced if r["outcome"] == "TrueIncorrect")
    assert fa == 0 and wt == 0, f"SOUNDNESS: false_alarms={fa} wrong_true={wt}"

    def per(w):
        return {k: v["confirmed_weighted"] for k, v in sorted(w["per_property_weighted"].items())}

    print(f"n = {len(spliced)}   in-competition = {len(in_comp)}   "
          f"out-of-competition = {len(spliced) - len(in_comp)}")
    print(f"false_alarms = {fa}   wrong_true = {wt}   [both MUST be 0]\n")
    print(f"  2026 rule (baseline)        weighted = {w26['confirmed_score_weighted']:4d}   {per(w26)}")
    print(f"  2027 rules, version-blind   weighted = {w27['confirmed_score_weighted']:4d}   {per(w27)}")
    print(f"  2027 rules, version-aware   weighted = {w27va['confirmed_score_weighted']:4d}   {per(w27va)}")
    print(f"\n  delta (blind)  = {w27['confirmed_score_weighted'] - w26['confirmed_score_weighted']}")
    print(f"  delta (aware)  = {w27va['confirmed_score_weighted'] - w26['confirmed_score_weighted']}")

    rev = subprocess.run(["git", "-C", "tests/benchmarks/sv-benchmarks", "rev-parse", "HEAD"],
                         capture_output=True, text=True).stdout.strip()
    head = subprocess.run(["git", "rev-parse", "HEAD"],
                          capture_output=True, text=True).stdout.strip()

    lost = collections.Counter()
    for r in spliced:
        if r["confirmed_2026"] > 0 and r["confirmed"] == 0:
            lost[(r["property"], tuple(r["base_categories"]))] += 1
    print("\n  rows that lost their points (property, base categories):")
    for k, v in sorted(lost.items(), key=lambda x: -x[1]):
        print(f"    {k[0]:16s} {list(k[1])!s:45s} {v}")

    out = {
        "rules_edition": "SV-COMP 2027",
        "source": "re-score of lever1-pertask.jsonl + the 99 spliced rows of lever1-true99-pertask.jsonl",
        "saf_commit": head, "svbench_commit": rev,
        "n": len(spliced), "in_competition": len(in_comp),
        "false_alarms": fa, "wrong_true": wt,
        "weighted_2026_rule": w26["confirmed_score_weighted"],
        "per_property_weighted_2026_rule": per(w26),
        "confirmed_score_weighted": w27["confirmed_score_weighted"],
        "per_property_weighted": per(w27),
        "confirmed_score_weighted_version_aware": w27va["confirmed_score_weighted"],
        "per_property_weighted_version_aware": per(w27va),
    }
    with open("m0a-2027-rescore.json", "w") as f:
        json.dump(out, f, indent=2)
    with open("m0a-2027-pertask.jsonl", "w") as f:
        for r in spliced:
            f.write(json.dumps(r) + "\n")
    print("\n  wrote m0a-2027-rescore.json + m0a-2027-pertask.jsonl")


if __name__ == "__main__":
    main()
