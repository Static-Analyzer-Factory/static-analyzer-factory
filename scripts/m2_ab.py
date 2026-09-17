#!/usr/bin/env python3
"""A/B a fresh per-task run against the 55,690-task baseline, task for task.

Compares only the tasks present in BOTH files, so a sampled run is directly
comparable. Reports every outcome transition and, separately, the two invariants
that must hold unconditionally: `false_alarms == 0` and `wrong_true == 0`.
"""
import collections, json, os, sys

os.chdir("/workspace" if os.path.isdir("/workspace/crates") else os.path.expanduser("~/static-analyzer-factory"))

BASE = "saf-alone-20260913T031843Z-pertask.jsonl"


def load(path):
    out = {}
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        out[(r["rel_yml"], r["property"], r["data_model"])] = r
    return out


def main():
    fresh_path = sys.argv[1]
    base_path = sys.argv[2] if len(sys.argv) > 2 else BASE
    base, fresh = load(base_path), load(fresh_path)
    common = sorted(set(base) & set(fresh))
    print(f"baseline {len(base)} rows, fresh {len(fresh)} rows, {len(common)} comparable\n")

    trans = collections.Counter()
    moved = []
    for k in common:
        b, f = base[k]["outcome"], fresh[k]["outcome"]
        if b != f:
            trans[(b, f)] += 1
            moved.append((k, b, f))
    if not trans:
        print("NO OUTCOME TRANSITIONS — the change is inert on this population.")
    else:
        print(f"{sum(trans.values())} transitions of {len(common)} ({100.0*sum(trans.values())/len(common):.2f}%)")
        for (b, f), c in trans.most_common():
            print(f"   {b:16s} -> {f:16s} {c}")
        print("\n  by cluster:")
        byg = collections.Counter(fresh[k].get("group", "?") for k, _, _ in moved)
        for g, c in byg.most_common(12):
            print(f"      {g:30s} {c}")

    for tag, d in (("baseline", base), ("fresh", fresh)):
        rows = [d[k] for k in common]
        fa = sum(1 for r in rows if r["outcome"] == "FalseIncorrect")
        wt = sum(1 for r in rows if r["outcome"] == "TrueIncorrect")
        flag = "" if (fa == 0 and wt == 0) else "   *** INVARIANT BROKEN ***"
        print(f"\n{tag:9s} over the comparable set: false_alarms={fa} wrong_true={wt}{flag}")


if __name__ == "__main__":
    main()
