#!/usr/bin/env python3
"""Current dedup-weighted score: the baseline with every directly-measured
property spliced in, scored by the harness's own confirmed_score /
weighted_confirmed_summary so it is comparable to the published 120 / 132 / 142."""
import collections, glob, json, os, sys

os.chdir("/workspace" if os.path.isdir("/workspace/crates") else os.path.expanduser("~/static-analyzer-factory"))
sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score, weighted_confirmed_summary  # noqa: E402
from svcomp_witness_rules import base_categories, load_set_membership      # noqa: E402

BASE = "saf-alone-20260913T031843Z-pertask.jsonl"
LAYERS = [
    ("MOVEMENT 1  (nov/term/ndr + mem)", ["saf-alone-20260914T103834Z-pertask.jsonl.partial",
                                          "m1-memsafety-20260915T003702Z-pertask.jsonl"]),
    ("MOVEMENT 2  (+ Anchored-Object memsafety)", ["m2-memsafety-20260916T123651Z-pertask.jsonl"]),
    ("M4 FIX      (+ SCCP phi-revisit)", sorted(glob.glob("m4-nooverflow-*-pertask.jsonl"))[-1:]
                                          + sorted(glob.glob("m4-memsafety-*-pertask.jsonl"))[-1:]),
]
MEASURED = {"no-overflow", "valid-memsafety", "termination", "no-data-race"}

base = [json.loads(l) for l in open(BASE) if l.strip()]
membership = load_set_membership("tests/benchmarks/sv-benchmarks")


def report(tag, rs, field):
    for r in rs:
        if not r.get("base_categories"):
            r["base_categories"] = sorted(base_categories(r["property"], membership.get(r["rel_yml"], frozenset())))
        r[field] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                   set(r["base_categories"]), r.get("sub"), r.get("witness_format"), False)
    w = weighted_confirmed_summary(rs, 1, field=field)
    per = {k: v["confirmed_weighted"] for k, v in sorted(w["per_property_weighted"].items())}
    fa = sum(1 for r in rs if r["outcome"] == "FalseIncorrect")
    wt = sum(1 for r in rs if r["outcome"] == "TrueIncorrect")
    flag = "" if (fa == 0 and wt == 0) else "  *** INVARIANT BROKEN ***"
    print(f"{tag:<44s} {w['confirmed_score_weighted']:>4d}   fa={fa} wt={wt}{flag}")
    print(f"{'':44s}      {per}")
    return w["confirmed_score_weighted"]


overlay, prev = {}, report("BASELINE    saf-alone-20260913", [dict(r) for r in base], "b")
for i, (tag, paths) in enumerate(LAYERS):
    for p in paths:
        for l in open(p):
            if l.strip():
                r = json.loads(l)
                overlay[(r["rel_yml"], r["property"], r["data_model"])] = r
    spliced = [dict(overlay.get((r["rel_yml"], r["property"], r["data_model"]), r))
               if r["property"] in MEASURED else dict(r) for r in base]
    cur = report(tag, spliced, f"s{i}")
    print(f"{'':44s}      delta {cur - prev:+d}")
    prev = cur

n = collections.Counter(r["property"] for r in base)
print(f"\n  properties measured directly: {sorted(MEASURED)}")
print(f"  unreach-call HELD at its 55,690-run baseline ({n['unreach-call']} tasks, not re-measured)")
