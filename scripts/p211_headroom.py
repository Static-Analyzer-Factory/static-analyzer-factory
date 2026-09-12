#!/usr/bin/env python3
"""TRUE-side dedup-weighted headroom priced against the PUBLISHED SV-COMP 2027 rules.

The witness requirement is per BASE CATEGORY `C.<property>.<suffix>`. This file used to
carry its own copy of the rules table; it now imports the single shared one from
`svcomp_witness_rules`, which also fixes two things the local copy got wrong: the
suffix is NOT the `.set` basename (`LinkedLists.set` sits inside `C.unreach-call.Heap`,
and there is no `C.unreach-call.LinkedLists`), and the Huawei rows are demo-mode.

Usage: python3 p211_headroom.py <svbench-c-root>
"""
import collections
import glob
import json
import os
import sys

SVB = sys.argv[1] if len(sys.argv) > 1 else "tests/benchmarks/sv-benchmarks/c"

# task .yml -> set of suffixes (a task can sit in several .set files)
suffix_of = collections.defaultdict(set)
for path in sorted(glob.glob(os.path.join(SVB, "*.set"))):
    suffix = os.path.basename(path)[:-4]
    for pat in open(path):
        pat = pat.strip()
        if not pat or pat.startswith("#"):
            continue
        for m in glob.glob(os.path.join(SVB, pat)):
            suffix_of[os.path.relpath(m, SVB)].add(suffix)
print(f"{len(suffix_of)} task .yml files are reachable from a .set file")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from svcomp_witness_rules import (  # noqa: E402
    base_categories, true_witness_requirement)


def needs_witness(prop, set_names):
    """TRUE-side requirement, from the single shared 2027 table. `set_names` are `.set`
    basenames, which must be mapped to base-category suffixes before the rule applies."""
    return true_witness_requirement(prop, base_categories(prop, set_names))[0]


rows = [json.loads(l) for l in open("lever1-pertask.jsonl")]
fix = {}
for l in open("lever1-true99-pertask.jsonl"):
    r = json.loads(l)
    fix[r["rel_yml"] + "|" + r["property"]] = r
for r in rows:
    k = r["rel_yml"] + "|" + r["property"]
    if k in fix and r["confirmed"] == 0 and fix[k]["confirmed"] > 0:
        r["confirmed"] = fix[k]["confirmed"]

score = collections.defaultdict(int)
tru = collections.defaultdict(list)
for r in rows:
    k = (r["group"], r["property"])
    score[k] += r["confirmed"]
    if r["expected"]:
        tru[k].append(r["rel_yml"])

out = []
for k in sorted(k for k in score if tru[k] and score[k] <= 0):
    g, p = k
    live = [y for y in tru[k] if y in suffix_of]
    if not live:
        continue
    # a cluster counts as witness-free if ANY of its live TRUE tasks is in a
    # witness-free base category (one confirmed task is enough to take the point)
    free = any(not needs_witness(p, suffix_of[y]) for y in live)
    sufs = collections.Counter(s for y in live for s in suffix_of[y])
    out.append((p, g, len(live), free, sufs.most_common(2)))

print(f"\n{len(out)} unsolved TRUE clusters survive the .set filter\n")
hdr = f'{"property":18s} {"clusters":>8s} {"VERDICT-ONLY":>13s} {"witness":>8s}'
print(hdr); print("-" * len(hdr))
tot = collections.Counter()
for p in sorted({r[0] for r in out}):
    rs = [r for r in out if r[0] == p]
    free = [r for r in rs if r[3]]
    print(f"{p:18s} {len(rs):8d} {len(free):13d} {len(rs)-len(free):8d}")
    tot["c"] += len(rs); tot["f"] += len(free)
print("-" * len(hdr))
print(f'{"TOTAL":18s} {tot["c"]:8d} {tot["f"]:13d} {tot["c"]-tot["f"]:8d}')

print("\nVERDICT-ONLY clusters — no validator can ever block these:")
for p, g, n, free, sufs in sorted(out):
    if free:
        print(f"  {p:16s} {g:32s} {n:4d} TRUE tasks   {[s for s, _ in sufs]}")
