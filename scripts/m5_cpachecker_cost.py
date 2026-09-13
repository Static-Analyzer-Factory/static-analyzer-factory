#!/usr/bin/env python3
"""Movement 5: what does dropping the bundled CPAchecker cost, in weighted points?

SAF gates every `no-overflow` and `unreach-call` TRUE on in-process CPAchecker
confirmation of SAF's OWN witness (plans/209, plans/210). Remove CPAchecker from
the archive and those verdicts have no gate. Two ways to cash that out:

  ABSTAIN  - SAF emits `unknown` wherever the gate would have run.
             Keeps wrong_true == 0. Costs whatever those TRUEs were worth.
  UNGATED  - SAF emits TRUE on absint alone.
             Cheaper on paper, but plans/209 measured absint alone is NOT
             wrong-TRUE-safe, so this can go negative at -32 a pop.

This prints the weighted score under each, against the 137 baseline.
"""
import collections
import json
import sys

sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score, weighted_confirmed_summary  # noqa: E402
from svcomp_witness_rules import base_categories, load_set_membership  # noqa: E402

GATED = {"no-overflow", "unreach-call"}

base = [json.loads(l) for l in open("m0a-2027-pertask.jsonl")]
fresh = {}
for l in open("0b3-term-pertask.jsonl"):
    r = json.loads(l)
    fresh[(r["rel_yml"], r["property"], r["data_model"])] = r

rows = []
for r in base:
    k = (r["rel_yml"], r["property"], r["data_model"])
    rows.append(fresh[k] if k in fresh else r)

membership = load_set_membership("tests/benchmarks/sv-benchmarks")
for r in rows:
    if "base_categories" not in r:
        r["base_categories"] = sorted(
            base_categories(r["property"], membership.get(r["rel_yml"], frozenset())))


def score(rows, field="confirmed"):
    for r in rows:
        sufs = set(r["base_categories"])
        r[field] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                   sufs, r.get("sub"), r.get("witness_format"), False)
    w = weighted_confirmed_summary(rows, 1, field=field)
    per = {k: v["confirmed_weighted"] for k, v in sorted(w["per_property_weighted"].items())}
    return w["confirmed_score_weighted"], per


baseline, per_base = score(rows)
print(f"BASELINE (CPAchecker bundled)          weighted = {baseline}")
print(f"    {per_base}")

# Which rows does the gate actually touch?
gated_true = [r for r in rows if r["property"] in GATED and r["outcome"] == "TrueCorrect"]
gated_scoring = [r for r in gated_true if r["confirmed"] > 0]
clusters = collections.Counter((r["property"], r["group"]) for r in gated_scoring)
print(f"\nCPAchecker-gated TRUE rows: {len(gated_true)} total, "
      f"{len(gated_scoring)} currently scoring, in {len(clusters)} clusters")
for (prop, grp), n in clusters.most_common(40):
    print(f"    {prop:14s} {grp:46s} {n}")

# ABSTAIN: the gate is gone, so SAF says nothing.
abst = [dict(r) for r in rows]
n_abst = 0
for r in abst:
    if r["property"] in GATED and r["outcome"] == "TrueCorrect":
        r["outcome"] = "Unknown"
        r["witness"] = None
        n_abst += 1
a_score, a_per = score(abst, field="confirmed_abstain")
print(f"\nABSTAIN  ({n_abst} TRUEs -> unknown)   weighted = {a_score}   "
      f"delta = {a_score - baseline:+d}")
print(f"    {a_per}")

fa = sum(1 for r in abst if r["outcome"] == "FalseIncorrect")
wt = sum(1 for r in abst if r["outcome"] == "TrueIncorrect")
print(f"    false_alarms={fa} wrong_true={wt}   [MUST be 0]")

# What does the whole FALSE side of those properties still carry?
print("\nFor context, the FALSE side of the gated properties is untouched:")
for prop in sorted(GATED):
    fc = sum(1 for r in rows if r["property"] == prop and r["outcome"] == "FalseCorrect"
             and r["confirmed"] > 0)
    tc = sum(1 for r in rows if r["property"] == prop and r["outcome"] == "TrueCorrect"
             and r["confirmed"] > 0)
    print(f"    {prop:14s} scoring FALSE rows = {fc:6d}   scoring TRUE rows = {tc}")
