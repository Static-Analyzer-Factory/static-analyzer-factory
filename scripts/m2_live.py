import json, os, sys, collections
os.chdir(os.path.expanduser("~/static-analyzer-factory"))
sys.path.insert(0, "scripts")
from svcomp_witness_rules import load_set_membership, base_categories, true_witness_requirement
mem = load_set_membership("tests/benchmarks/sv-benchmarks")
rows = [json.loads(l) for l in open("m2-FINAL-false.jsonl")]
live, juliet_only, noset = 0, 0, 0
for r in rows:
    sets = mem.get(r["rel_yml"])
    if not sets:
        noset += 1; continue
    cats = base_categories("valid-memsafety", sets)
    if cats: live += 1
    else: juliet_only += 1
print(f"expected-FALSE rows probed : {len(rows)}")
print(f"  in a 2027 base category  : {live}   <- the corpus that actually scores")
print(f"  in NO competition .set   : {juliet_only}")
print(f"  no .set membership at all: {noset}")
surv = [r for r in rows if r.get("ok") is True]
print(f"  SURVIVORS overall        : {len(surv)}")
print()
# and the verdict-only claim for the 43 target clusters
ycl = collections.Counter()
for l in open("m2-FINAL-yield.jsonl"):
    r = json.loads(l)
    if r.get("ok") is True:
        ycl[r.get("group") or "?"] += 1
cats_seen = set()
for l in open("m2-FINAL-yield.jsonl"):
    r = json.loads(l)
    s = mem.get(r["rel_yml"])
    if s: cats_seen |= base_categories("valid-memsafety", s)
print("base categories the yield population occupies, and whether TRUE needs a witness:")
for c in sorted(cats_seen):
    need, ver = true_witness_requirement("valid-memsafety", {c})
    print(f"   C.valid-memsafety.{c:22s} witness_required={need} min_version={ver}")
