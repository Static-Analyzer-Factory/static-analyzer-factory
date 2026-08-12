#!/usr/bin/env python3
"""Plan 198 de-risk — composition of the thread-spawn valid-memsafety reservoir
(excluding the dedicated concurrency dirs): Juliet (dead pthread scaffolding,
sequential-in-practice) vs NON-Juliet (the residual where a spawn may actually be
reachable and the gate must keep abstaining for soundness)."""
import re
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
CONC = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/ldv-linux-3.14-races/",
        "/libvsync/", "/locks/", "/ddv-machzwd/")
INPUT = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")


def spawns(s):
    try:
        b = s.read_text(errors="replace")
    except OSError:
        return False
    return b.count("pthread_create") > 1 or b.count("thrd_create") > 1


def main():
    c = Counter()
    examples = []
    for yml in SVB.rglob("*.yml"):
        try:
            t = yml.read_text(errors="replace")
        except OSError:
            continue
        if "valid-memsafety" not in t:
            continue
        exp = None
        lines = t.splitlines()
        for i, ln in enumerate(lines):
            if "valid-memsafety" in ln and "property_file" in ln:
                for j in range(i, min(i + 5, len(lines))):
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                break
        if exp is None:
            continue
        mi = INPUT.search(t)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists() or any(d in str(src) for d in CONC):
            continue
        if not spawns(src):
            continue
        juliet = "Juliet_Test" in str(src)
        key = "juliet" if juliet else "nonjuliet"
        c[key] += 1
        c[key + ("_safe" if exp else "_buggy")] += 1
        if not juliet:
            top = str(src).split("/c/")[-1].split("/")[0]
            c["dir:" + top] += 1
            if len(examples) < 20:
                examples.append(("safe" if exp else "buggy", str(src).split("/c/")[-1]))
    print("thread-spawn (excl conc_dir) valid-memsafety composition:")
    for k in ("juliet", "juliet_safe", "juliet_buggy",
              "nonjuliet", "nonjuliet_safe", "nonjuliet_buggy"):
        print(f"  {k:<16} = {c[k]}")
    print("\n  non-Juliet by top dir:")
    for k in sorted(c):
        if k.startswith("dir:"):
            print(f"    {k[4:]:<28} = {c[k]}")
    print("\n  non-Juliet examples (residual — gate must abstain if a spawn is reachable):")
    for tag, p in examples:
        print(f"    {tag:<5} {p[:95]}")


if __name__ == "__main__":
    main()
