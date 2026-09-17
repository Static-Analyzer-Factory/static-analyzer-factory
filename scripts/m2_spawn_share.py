#!/usr/bin/env python3
"""How much of the valid-memsafety population would a syntactic spawn pre-check skip?

A spawn-admitting reachable universe forces Andersen PTA + ICFG + MTA per task
(race_true.rs). race_true pays that on ~1k no-data-race tasks; a memsafety prover
would pay it on ~20.5k. This measures the population a cheap pre-check can exempt.
"""
import collections, os, re, sys

SPAWN = re.compile(rb"pthread_create|thrd_create")
INPUT_RE = re.compile(r"""input_files:\s*['"]?([^'"\n]+)['"]?""")
SPLITS = "tests/benchmarks/svcomp-splits"


def main():
    prop = sys.argv[1] if len(sys.argv) > 1 else "valid-memsafety"
    ymls = set()
    for split in ("train", "val", "holdout"):
        p = os.path.join(SPLITS, f"{split}.{prop}.set")
        if not os.path.exists(p):
            continue
        for line in open(p):
            line = line.strip()
            if line:
                # BenchExec .set paths are written `../tests/benchmarks/...`
                # relative to the splits dir; the tail is repo-root-relative.
                ymls.add(line.lstrip("./").removeprefix("../"))
    print(f"unique {prop} ymls: {len(ymls)}")

    spawn = seq = miss = 0
    byg, seqbyg = collections.Counter(), collections.Counter()
    for y in sorted(ymls):
        try:
            txt = open(y, errors="ignore").read()
        except OSError:
            miss += 1; continue
        m = INPUT_RE.search(txt)
        if not m:
            miss += 1; continue
        src = os.path.join(os.path.dirname(y), m.group(1).strip())
        try:
            with open(src, "rb") as fh:
                body = fh.read()
        except OSError:
            miss += 1; continue
        g = os.path.basename(os.path.dirname(y))
        if SPAWN.search(body):
            spawn += 1; byg[g] += 1
        else:
            seq += 1; seqbyg[g] += 1

    tot = spawn + seq
    print(f"  spawn-bearing : {spawn:6d}  ({100.0 * spawn / tot:.2f}%)")
    print(f"  sequential    : {seq:6d}  ({100.0 * seq / tot:.2f}%)")
    print(f"  unresolved    : {miss}")
    print("\nspawn-bearing by group:")
    for g, c in byg.most_common(20):
        print(f"   {g:34s} {c}")


if __name__ == "__main__":
    main()
