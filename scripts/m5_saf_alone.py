#!/usr/bin/env python3
"""Movement 5: SAF's score with NO bundled SV-COMP-participant tools.

User decision 2026-09-13: SAF must not bundle any tool that is itself an SV-COMP
participant. That is exactly two -- CPAchecker (fm-tools id `cpachecker`) and
CBMC (`cbmc`, `cbmc-path`). clang/opt/Z3/sanitizers are compilers, a linked SMT
library and runtime instrumentation; none is a participant, all stay.

Removing them, measured:

  CPAchecker  the TRUE gate is FAIL-CLOSED (verified empirically,
              scripts/m5_gate_probe3.sh: absent -> `unknown`, present -> `true`).
              So SAF ABSTAINS on every gated TRUE. It cannot emit a wrong TRUE.
  CBMC        Lever::Cbmc carried exactly two unreach-call clusters,
              `xcsp` and `recursified_nla-digbench`, and every scoring task in
              both lies inside the lever's scope -- so both clusters are lost.

This prints the combined result. `false_alarms` and `wrong_true` must stay 0.
"""
import json
import sys

sys.path.insert(0, "scripts")
from svcomp_split_eval import confirmed_score, weighted_confirmed_summary  # noqa: E402
from svcomp_witness_rules import base_categories, load_set_membership  # noqa: E402

GATED_TRUE = {"no-overflow", "unreach-call"}       # what the CPAchecker gate guards
CBMC_CLUSTERS = {"xcsp", "recursified_nla-digbench"}

base = [json.loads(l) for l in open("m0a-2027-pertask.jsonl")]
fresh = {}
for l in open("0b3-term-pertask.jsonl"):
    r = json.loads(l)
    fresh[(r["rel_yml"], r["property"], r["data_model"])] = r

cbmc_scope = set()
for l in open("splits/lever1_scoped.jsonl"):
    r = json.loads(l)
    cbmc_scope.add(r["rel_yml"])

rows = []
for r in base:
    k = (r["rel_yml"], r["property"], r["data_model"])
    rows.append(dict(fresh[k] if k in fresh else r))

membership = load_set_membership("tests/benchmarks/sv-benchmarks")
for r in rows:
    if "base_categories" not in r:
        r["base_categories"] = sorted(
            base_categories(r["property"], membership.get(r["rel_yml"], frozenset())))


def rescore(rs, field):
    for r in rs:
        r[field] = confirmed_score(r["outcome"], r["property"], r["witness"],
                                   set(r["base_categories"]), r.get("sub"),
                                   r.get("witness_format"), False)
    w = weighted_confirmed_summary(rs, 1, field=field)
    return (w["confirmed_score_weighted"],
            {k: v["confirmed_weighted"] for k, v in sorted(w["per_property_weighted"].items())})


def report(tag, rs, field):
    total, per = rescore(rs, field)
    fa = sum(1 for r in rs if r["outcome"] == "FalseIncorrect")
    wt = sum(1 for r in rs if r["outcome"] == "TrueIncorrect")
    flag = "" if (fa == 0 and wt == 0) else "   *** INVARIANT BROKEN ***"
    print(f"{tag:<44s} {total:>4d}   fa={fa} wt={wt}{flag}")
    print(f"{'':44s}      {per}")
    return total


b = report("BASELINE (both tools bundled)", rows, "c0")

# --- drop CPAchecker: the fail-closed gate makes SAF abstain -------------------
no_cpa = [dict(r) for r in rows]
n1 = 0
for r in no_cpa:
    if r["property"] in GATED_TRUE and r["outcome"] == "TrueCorrect":
        r["outcome"], r["witness"] = "Unknown", None
        n1 += 1
c1 = report(f"- CPAchecker  ({n1} TRUEs abstain)", no_cpa, "c1")

# --- drop CBMC: the two levered clusters lose every scoring task ---------------
no_cbmc = [dict(r) for r in rows]
n2 = 0
for r in no_cbmc:
    if (r["property"] == "unreach-call" and r["group"] in CBMC_CLUSTERS
            and r["rel_yml"] in cbmc_scope and r["outcome"] == "FalseCorrect"):
        r["outcome"], r["witness"] = "Unknown", None
        n2 += 1
c2 = report(f"- CBMC        ({n2} FALSEs abstain)", no_cbmc, "c2")

# --- drop both ----------------------------------------------------------------
neither = [dict(r) for r in rows]
n3 = 0
for r in neither:
    if r["property"] in GATED_TRUE and r["outcome"] == "TrueCorrect":
        r["outcome"], r["witness"] = "Unknown", None
        n3 += 1
    elif (r["property"] == "unreach-call" and r["group"] in CBMC_CLUSTERS
          and r["rel_yml"] in cbmc_scope and r["outcome"] == "FalseCorrect"):
        r["outcome"], r["witness"] = "Unknown", None
        n3 += 1
c3 = report(f"- BOTH        (SAF alone, {n3} rows abstain)", neither, "c3")

print()
print(f"  CPAchecker costs {c1 - b:+d}    CBMC costs {c2 - b:+d}    "
      f"together {c3 - b:+d}   (additive: {(c1 - b) + (c2 - b):+d})")
print(f"  THE NUMBER TO PUBLISH: {c3}")
