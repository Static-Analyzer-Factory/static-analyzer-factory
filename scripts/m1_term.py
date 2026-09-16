import json, os, collections
os.chdir("/workspace" if os.path.isdir("/workspace/crates") else os.path.expanduser("~/static-analyzer-factory"))
base = {}
for l in open("saf-alone-20260913T031843Z-pertask.jsonl"):
    r = json.loads(l)
    if r["property"] == "termination":
        base[(r["rel_yml"], r["data_model"])] = r
new = {}
for l in open("saf-alone-20260914T103834Z-pertask.jsonl.partial"):
    r = json.loads(l)
    if r["property"] == "termination":
        new[(r["rel_yml"], r["data_model"])] = r
print(f"termination rows: base={len(base)} new={len(new)}")
fields = ["outcome", "witness", "witness_format", "confirmed_va", "raw"]
diff = collections.Counter()
examples = collections.defaultdict(list)
for k in new:
    if k not in base: continue
    for f in fields:
        if base[k].get(f) != new[k].get(f):
            diff[f] += 1
            if len(examples[f]) < 4:
                examples[f].append((k[0], base[k].get(f), new[k].get(f)))
print("fields that changed:", dict(diff) if diff else "NONE")
for f, ex in examples.items():
    print(f"  --- {f} ---")
    for rel, a, b in ex:
        print(f"    {rel[:56]:58s} {str(a)[:26]:28s} -> {str(b)[:26]}")
