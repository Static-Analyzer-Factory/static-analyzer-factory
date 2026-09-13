#!/usr/bin/env python3
"""Movement 5: derive SAF's SV-COMP 2027 `opt_in` base-category list from data.

User chose `opt_in` on base categories rather than meta-category membership, because
joining a meta enrols SAF in every base category under it and the meta score is
normalised per sub-category -- so a base category SAF answers `unknown` on actively
drags the meta score down. `opt_out` does NOT help: the generator ignores it, and
submission.php says an opted-out category's score still counts toward the meta.

This scores SAF-alone (no CPAchecker, no CBMC -- the 123 configuration) and reports,
per base category: scoring clusters, scoring tasks, and total tasks in competition.
"""
import collections
import json
import sys

sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score  # noqa: E402
from svcomp_witness_rules import (DEMO_CATEGORIES, base_categories,  # noqa: E402
                                  load_set_membership)

GATED_TRUE = {"no-overflow", "unreach-call"}
CBMC_CLUSTERS = {"xcsp", "recursified_nla-digbench"}

base = [json.loads(l) for l in open("m0a-2027-pertask.jsonl")]
fresh = {}
for l in open("0b3-term-pertask.jsonl"):
    r = json.loads(l)
    fresh[(r["rel_yml"], r["property"], r["data_model"])] = r
cbmc_scope = {json.loads(l)["rel_yml"] for l in open("splits/lever1_scoped.jsonl")}

rows = []
for r in base:
    k = (r["rel_yml"], r["property"], r["data_model"])
    rows.append(dict(fresh[k] if k in fresh else r))

membership = load_set_membership("tests/benchmarks/sv-benchmarks")
for r in rows:
    if "base_categories" not in r:
        r["base_categories"] = sorted(
            base_categories(r["property"], membership.get(r["rel_yml"], frozenset())))
    # SAF-alone: both competitor tools removed.
    if r["property"] in GATED_TRUE and r["outcome"] == "TrueCorrect":
        r["outcome"], r["witness"] = "Unknown", None
    elif (r["property"] == "unreach-call" and r["group"] in CBMC_CLUSTERS
          and r["rel_yml"] in cbmc_scope and r["outcome"] == "FalseCorrect"):
        r["outcome"], r["witness"] = "Unknown", None
    r["confirmed"] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                     set(r["base_categories"]), r.get("sub"),
                                     r.get("witness_format"), False)

total = collections.Counter()
scoring = collections.Counter()
clusters = collections.defaultdict(set)
for r in rows:
    for suf in r["base_categories"]:
        cat = f"C.{r['property']}.{suf}"
        total[cat] += 1
        if r["confirmed"] > 0:
            scoring[cat] += 1
            clusters[cat].add(r["group"])

print(f"{'base category':<52} {'clus':>5} {'scoring':>8} {'tasks':>7} {'hit%':>6}")
print("-" * 82)
opt_in, skipped = [], []
for cat in sorted(total, key=lambda c: (-len(clusters[c]), -scoring[c], c)):
    n_cl, n_sc, n_t = len(clusters[cat]), scoring[cat], total[cat]
    demo = "  (demo - does not count toward Overall)" if cat in DEMO_CATEGORIES else ""
    print(f"{cat:<52} {n_cl:>5} {n_sc:>8} {n_t:>7} {100*n_sc/n_t:>5.1f}%{demo}")
    (opt_in if n_cl > 0 and cat not in DEMO_CATEGORIES else skipped).append(cat)

print()
print(f"OPT IN to {len(opt_in)} base categories (SAF scores in each):")
for c in sorted(opt_in):
    print(f"    - {c}")
print()
print(f"SKIP {len(skipped)} (zero scoring clusters, or demo):")
for c in sorted(skipped):
    print(f"    # {c}   ({len(clusters[c])} clusters, {scoring[c]}/{total[c]} tasks)")
