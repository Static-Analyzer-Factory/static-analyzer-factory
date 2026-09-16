import json, os, collections
os.chdir(os.path.expanduser("~/static-analyzer-factory"))
base = {}
for l in open("m2-universe-baseline.jsonl"):
    r = json.loads(l); base[(r["rel_yml"], r["property"], r["data_model"])] = r
new = {}
for p in ("no-data-race", "termination"):
    f = f"m2-oracle-{p}-pertask.jsonl"
    if not os.path.exists(f):
        print(f"!! missing {f}"); continue
    for l in open(f):
        r = json.loads(l); new[(r["rel_yml"], r["property"], r["data_model"])] = r
common = [k for k in new if k in base]
print(f"baseline={len(base)}  new={len(new)}  comparable={len(common)}")
byprop = collections.defaultdict(lambda: [0, 0])
diffs = []
for k in common:
    byprop[k[1]][0] += 1
    a, b = base[k].get("outcome"), new[k].get("outcome")
    if a != b:
        byprop[k[1]][1] += 1
        diffs.append((k[1], k[0], a, b))
print(f"{'property':16s} {'compared':>9s} {'CHANGED':>8s}")
for p, (n, c) in sorted(byprop.items()):
    print(f"{p:16s} {n:9d} {c:8d}")
if diffs:
    print("\n*** REGRESSION CANDIDATES ***")
    for p, rel, a, b in diffs[:40]:
        print(f"  {p:14s} {rel:52s} {a} -> {b}")
else:
    print("\nBEHAVIOUR PRESERVED: 0 outcome changes across both callers.")
bad = [k for k in new if new[k].get("outcome") in ("FalseIncorrect", "TrueIncorrect")]
print(f"false_alarms + wrong_true in the new run: {len(bad)}")
