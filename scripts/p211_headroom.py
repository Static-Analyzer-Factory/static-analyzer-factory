#!/usr/bin/env python3
"""TRUE-side dedup-weighted headroom priced against the PUBLISHED SV-COMP 2027 rules.

The witness requirement is per BASE CATEGORY = `C.<property>.<suffix>`, where the suffix
is the benchmark family = the `.set` file the task belongs to. From
https://sv-comp.sosy-lab.org/2027/rules.php (verified 2026-09-12):

    C.unreach-call.{Arrays,Heap}     not supported        -> verdict alone
    C.unreach-call.Floats            2.0+ (demo mode)     -> verdict alone
    C.unreach-call.Concurrency       2.1+                 -> witness REQUIRED
    C.unreach-call.<other>           2.0+                 -> witness REQUIRED
    C.valid-memsafety.<any>          not supported        -> verdict alone
    C.valid-memcleanup.all           not supported        -> verdict alone
    C.no-overflow.Concurrency        2.1+                 -> witness REQUIRED
    C.no-overflow.<other>            2.0+                 -> witness REQUIRED
    C.no-data-race.all               not supported        -> verdict alone
    C.termination.all                2.1+                 -> witness REQUIRED  (NEW in 2027)

Usage: python3 headroom_final.py <svbench-c-root>
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

WITNESS_FREE = {
    "unreach-call": {"Arrays", "Heap", "Floats"},
    "valid-memsafety": None,     # None = always witness-free
    "valid-memcleanup": None,
    "no-data-race": None,
}


def needs_witness(prop, suffixes):
    if prop in WITNESS_FREE:
        free = WITNESS_FREE[prop]
        if free is None:
            return False
        return not (suffixes & free)
    return True                   # no-overflow, termination: always required in 2027


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
