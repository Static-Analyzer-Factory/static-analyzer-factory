import json, collections, os
os.chdir(os.path.expanduser("~/static-analyzer-factory"))
# The two properties whose verdicts the universe gate can move.
CALLERS = {"no-data-race": "race_true.rs", "termination": "termination.rs"}
src = "saf-alone-20260914T103834Z-pertask.jsonl.partial"   # post-Movement-1 binary
rows = [json.loads(l) for l in open(src)]
sel = [r for r in rows if r["property"] in CALLERS]
print(f"post-Movement-1 rows for the two universe-gate callers: {len(sel)}")
for p in CALLERS:
    sub = [r for r in sel if r["property"] == p]
    c = collections.Counter(r["outcome"] for r in sub)
    print(f"  {p:16s} n={len(sub):5d}  {dict(c)}")
out = "m2-universe-baseline.jsonl"
with open(out, "w") as fh:
    for r in sel:
        fh.write(json.dumps({k: r.get(k) for k in
                 ("rel_yml", "property", "data_model", "outcome", "witness", "sub")}) + "\n")
print(f"\nwrote {out} ({len(sel)} rows) -- the behaviour-preservation oracle for the")
print("universe.rs extraction. Re-run these two properties after the refactor and diff;")
print("ANY outcome change is a regression in code that currently holds FP=0.")
