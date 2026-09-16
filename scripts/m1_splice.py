#!/usr/bin/env python3
"""Movement 1: score the MEASURED no-overflow half spliced into the baseline.

The merge-gate run completed `no-overflow` 100% (9,063/9,063) before dying on a
full disk. That is the ONLY property `try_overflow_true` can move, so splicing
those measured rows into the baseline gives the +12 answer now, without waiting
for a re-run. Everything else is held at baseline, so any delta is attributable
to the re-enabled arm alone.
"""
import json, os, sys
os.chdir("/workspace" if os.path.isdir("/workspace/crates") else os.path.expanduser("~/static-analyzer-factory"))
sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score, weighted_confirmed_summary  # noqa: E402
from svcomp_witness_rules import base_categories, load_set_membership      # noqa: E402

base = [json.loads(l) for l in open("saf-alone-20260913T031843Z-pertask.jsonl")]
fresh = {}
for l in open("saf-alone-20260914T103834Z-pertask.jsonl.partial"):
    r = json.loads(l)
    fresh[(r["rel_yml"], r["property"], r["data_model"])] = r

membership = load_set_membership("tests/benchmarks/sv-benchmarks")

def prep(rs):
    for r in rs:
        if "base_categories" not in r or not r["base_categories"]:
            r["base_categories"] = sorted(
                base_categories(r["property"], membership.get(r["rel_yml"], frozenset())))
    return rs

def rescore(rs, field):
    for r in rs:
        r[field] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                   set(r["base_categories"]), r.get("sub"),
                                   r.get("witness_format"), False)
    w = weighted_confirmed_summary(rs, 1, field=field)
    return (w["confirmed_score_weighted"],
            {k: v["confirmed_weighted"] for k, v in sorted(w["per_property_weighted"].items())})

def report(tag, rs, field):
    total, per = rescore(prep(rs), field)
    fa = sum(1 for r in rs if r["outcome"] == "FalseIncorrect")
    wt = sum(1 for r in rs if r["outcome"] == "TrueIncorrect")
    flag = "" if (fa == 0 and wt == 0) else "   *** INVARIANT BROKEN ***"
    print(f"{tag:<46s} {total:>4d}   false_alarms={fa} wrong_true={wt}{flag}")
    print(f"{'':46s}      {per}")
    return total

b = report("BASELINE  saf-alone-20260913 (120)", [dict(r) for r in base], "b")

# splice: measured no-overflow rows only
spliced, n = [], 0
for r in base:
    k = (r["rel_yml"], r["property"], r["data_model"])
    if r["property"] == "no-overflow" and k in fresh:
        spliced.append(dict(fresh[k])); n += 1
    else:
        spliced.append(dict(r))
s = report(f"MOVEMENT 1  (no-overflow measured, {n} rows)", spliced, "s")

print()
print(f"  no-overflow arm delta: {s - b:+d} weighted   ({b} -> {s})")
print()
# how many no-overflow rows changed, and in which direction
import collections
d = collections.Counter()
for r in base:
    k = (r["rel_yml"], r["property"], r["data_model"])
    if r["property"] == "no-overflow" and k in fresh:
        a, c = r["outcome"], fresh[k]["outcome"]
        if a != c:
            d[f"{a} -> {c}"] += 1
print("  no-overflow outcome transitions:")
for k, v in d.most_common():
    print(f"    {v:5d}  {k}")
