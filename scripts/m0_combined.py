#!/usr/bin/env python3
"""Combined 0A + 0B score: the 2027 re-score with the fresh termination run spliced in."""
import collections
import json
import sys

sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score, weighted_confirmed_summary  # noqa: E402
from svcomp_witness_rules import base_categories, load_set_membership  # noqa: E402

base = [json.loads(l) for l in open("m0a-2027-pertask.jsonl")]
fresh = {}
for l in open("0b-term-pertask.jsonl"):
    r = json.loads(l)
    fresh[(r["rel_yml"], r["property"], r["data_model"])] = r

n, out = 0, []
for r in base:
    k = (r["rel_yml"], r["property"], r["data_model"])
    if k in fresh:
        out.append(fresh[k]); n += 1
    else:
        out.append(r)
print(f"spliced {n} fresh termination rows of {len(fresh)}")

membership = load_set_membership("tests/benchmarks/sv-benchmarks")
for r in out:
    if "base_categories" not in r:
        r["base_categories"] = sorted(
            base_categories(r["property"], membership.get(r["rel_yml"], frozenset())))
    sufs = set(r["base_categories"])
    fmt = r.get("witness_format")
    r["confirmed"] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                     sufs, r.get("sub"), fmt, False)
    r["confirmed_va"] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                        sufs, r.get("sub"), fmt, True)

w = weighted_confirmed_summary(out, 1, field="confirmed")
va = weighted_confirmed_summary(out, 1, field="confirmed_va")
fa = sum(1 for r in out if r["outcome"] == "FalseIncorrect")
wt = sum(1 for r in out if r["outcome"] == "TrueIncorrect")


def per(x):
    return {k: v["confirmed_weighted"] for k, v in sorted(x["per_property_weighted"].items())}


print(f"false_alarms={fa} wrong_true={wt}  [MUST be 0]")
print(f"weighted (version-blind) = {w['confirmed_score_weighted']}   {per(w)}")
print(f"weighted (version-aware) = {va['confirmed_score_weighted']}   {per(va)}")

conf = [r for r in out if r["property"] == "termination" and r["confirmed"] > 0]
groups = {r["group"] for r in conf}
print(f"\ntermination confirmed: {len(conf)} tasks in {len(groups)} clusters")
for g, c in collections.Counter(r["group"] for r in conf).most_common(25):
    print(f"   {g:42s} {c}")

allterm = {r["group"] for r in out if r["property"] == "termination" and r["outcome"] == "TrueCorrect"}
print(f"\nclusters still UNCONFIRMED ({len(allterm - groups)}):")
for g in sorted(allterm - groups):
    tot = sum(1 for r in out if r["property"] == "termination"
              and r["group"] == g and r["outcome"] == "TrueCorrect")
    print(f"   {g:42s} {tot} proven tasks, 0 confirmed")

with open("m0-combined-rescore.json", "w") as f:
    json.dump({
        "rules_edition": "SV-COMP 2027",
        "source": "m0a-2027-pertask.jsonl with the fresh 0B termination run spliced in",
        "false_alarms": fa, "wrong_true": wt, "n": len(out),
        "confirmed_score_weighted": w["confirmed_score_weighted"],
        "per_property_weighted": per(w),
        "confirmed_score_weighted_version_aware": va["confirmed_score_weighted"],
        "per_property_weighted_version_aware": per(va),
    }, f, indent=2)
print("\nwrote m0-combined-rescore.json")
