#!/usr/bin/env python3
"""Movement 2 score: the Movement 1 result with the fresh valid-memsafety rows
spliced in, scored by the harness's own confirmed_score / weighted_confirmed_summary
so the total is directly comparable to the 120 and 132 baselines."""
import collections, json, os, sys

os.chdir("/workspace" if os.path.isdir("/workspace/crates") else os.path.expanduser("~/static-analyzer-factory"))
sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score, weighted_confirmed_summary  # noqa: E402
from svcomp_witness_rules import base_categories, load_set_membership      # noqa: E402

BASE = "saf-alone-20260913T031843Z-pertask.jsonl"
# Movement 1's measured rows (no-overflow + termination + no-data-race + memsafety)
M1 = ["saf-alone-20260914T103834Z-pertask.jsonl.partial",
      "m1-memsafety-20260915T003702Z-pertask.jsonl"]
M2 = sys.argv[1] if len(sys.argv) > 1 else "m2-memsafety-20260916T123651Z-pertask.jsonl"

base = [json.loads(l) for l in open(BASE) if l.strip()]
membership = load_set_membership("tests/benchmarks/sv-benchmarks")


def load(paths):
    d = {}
    for p in paths:
        for l in open(p):
            l = l.strip()
            if not l:
                continue
            r = json.loads(l)
            d[(r["rel_yml"], r["property"], r["data_model"])] = r
    return d


m1, m2 = load(M1), load([M2])


def report(tag, rs, field):
    for r in rs:
        if "base_categories" not in r or not r["base_categories"]:
            r["base_categories"] = sorted(
                base_categories(r["property"], membership.get(r["rel_yml"], frozenset())))
        r[field] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                   set(r["base_categories"]), r.get("sub"),
                                   r.get("witness_format"), False)
    w = weighted_confirmed_summary(rs, 1, field=field)
    total = w["confirmed_score_weighted"]
    per = {k: v["confirmed_weighted"] for k, v in sorted(w["per_property_weighted"].items())}
    fa = sum(1 for r in rs if r["outcome"] == "FalseIncorrect")
    wt = sum(1 for r in rs if r["outcome"] == "TrueIncorrect")
    flag = "" if (fa == 0 and wt == 0) else "   *** INVARIANT BROKEN ***"
    print(f"{tag:<46s} {total:>4d}   false_alarms={fa} wrong_true={wt}{flag}")
    print(f"{'':46s}      {per}")
    return total


MEASURED = {"no-overflow", "valid-memsafety", "termination", "no-data-race"}


def splice(overlay):
    out = []
    for r in base:
        k = (r["rel_yml"], r["property"], r["data_model"])
        out.append(dict(overlay[k]) if (r["property"] in MEASURED and k in overlay) else dict(r))
    return out


b = report("BASELINE    saf-alone-20260913", [dict(r) for r in base], "b")
s1 = report("MOVEMENT 1  (4 properties measured)", splice(m1), "s1")
merged = dict(m1)
merged.update(m2)                      # M2's memsafety rows win
s2 = report("MOVEMENT 2  (+ Anchored-Object memsafety)", splice(merged), "s2")

print()
print(f"  Movement 1 delta : {s1 - b:+d}   ({b} -> {s1})")
print(f"  Movement 2 delta : {s2 - s1:+d}   ({s1} -> {s2})")
print()
d = collections.Counter()
for r in base:
    k = (r["rel_yml"], r["property"], r["data_model"])
    if r["property"] == "valid-memsafety" and k in m1 and k in m2 \
            and m1[k]["outcome"] != m2[k]["outcome"]:
        d[f"{m1[k]['outcome']} -> {m2[k]['outcome']}"] += 1
print(f"  valid-memsafety transitions (Movement 1 -> Movement 2): {dict(d) if d else 'none'}")
gained = [k for k in m2 if k in m1 and m1[k]["outcome"] != m2[k]["outcome"]]
byg = collections.Counter(m2[k].get("group", "?") for k in gained)
print("  by cluster:")
for g, c in byg.most_common(15):
    print(f"      {g:32s} {c}")
