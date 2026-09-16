#!/usr/bin/env python3
"""Movement 1 final score: baseline with BOTH measured properties spliced in."""
import json, os, sys, collections
os.chdir("/workspace" if os.path.isdir("/workspace/crates") else os.path.expanduser("~/static-analyzer-factory"))
sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score, weighted_confirmed_summary  # noqa: E402
from svcomp_witness_rules import base_categories, load_set_membership      # noqa: E402

base = [json.loads(l) for l in open("saf-alone-20260913T031843Z-pertask.jsonl")]
fresh = {}
for path in ("saf-alone-20260914T103834Z-pertask.jsonl.partial",
             "m1-memsafety-20260915T003702Z-pertask.jsonl"):
    for l in open(path):
        r = json.loads(l)
        fresh[(r["rel_yml"], r["property"], r["data_model"])] = r

membership = load_set_membership("tests/benchmarks/sv-benchmarks")

def prep(rs):
    for r in rs:
        if "base_categories" not in r or not r["base_categories"]:
            r["base_categories"] = sorted(
                base_categories(r["property"], membership.get(r["rel_yml"], frozenset())))
    return rs

def report(tag, rs, field):
    prep(rs)
    for r in rs:
        r[field] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                   set(r["base_categories"]), r.get("sub"),
                                   r.get("witness_format"), False)
    w = weighted_confirmed_summary(rs, 1, field=field)
    total = w["confirmed_score_weighted"]
    per = {k: v["confirmed_weighted"] for k, v in sorted(w["per_property_weighted"].items())}
    fa = sum(1 for r in rs if r["outcome"] == "FalseIncorrect")
    wt = sum(1 for r in rs if r["outcome"] == "TrueIncorrect")
    flag = "" if (fa == 0 and wt == 0) else "   *** INVARIANT BROKEN ***"
    print(f"{tag:<44s} {total:>4d}   false_alarms={fa} wrong_true={wt}{flag}")
    print(f"{'':44s}      {per}")
    return total

b = report("BASELINE  saf-alone-20260913", [dict(r) for r in base], "b")

MEASURED = {"no-overflow", "valid-memsafety", "termination", "no-data-race"}
spliced, n = [], collections.Counter()
for r in base:
    k = (r["rel_yml"], r["property"], r["data_model"])
    if r["property"] in MEASURED and k in fresh:
        spliced.append(dict(fresh[k])); n[r["property"]] += 1
    else:
        spliced.append(dict(r))
s = report(f"MOVEMENT 1  (4 properties measured, {sum(n.values())} rows)", spliced, "s")

print()
print(f"  MEASURED DELTA: {s - b:+d} weighted   ({b} -> {s})")
print()
for prop in sorted(MEASURED):
    d = collections.Counter()
    for r in base:
        k = (r["rel_yml"], r["property"], r["data_model"])
        if r["property"] == prop and k in fresh and r["outcome"] != fresh[k]["outcome"]:
            d[f"{r['outcome']} -> {fresh[k]['outcome']}"] += 1
    print(f"  {prop} transitions: {dict(d) if d else 'none'}")
