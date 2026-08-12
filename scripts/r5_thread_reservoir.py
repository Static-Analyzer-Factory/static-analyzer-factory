#!/usr/bin/env python3
"""Size the threaded valid-memsafety reservoir (plan 197 Slice 3 / plan 198 de-risk).

Counts valid-memsafety tasks by expected verdict and whether the program actually
SPAWNS a thread (a pthread_create/thrd_create CALL, i.e. the token appears more than
once: the header prototype + >=1 call). The buggy+thread-spawning bucket is the upper
bound on what concurrency-aware confirmation could recover (R5 currently abstains on
all of it). Excludes the dedicated concurrency benchmark dirs (already out of scope).

    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev python3 scripts/r5_thread_reservoir.py
"""
import re
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
CONC_DIRS = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/libvsync/",
             "/locks/", "/ddv-machzwd/")
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")


def spawns(src):
    try:
        b = src.read_text(errors="replace")
    except OSError:
        return False
    return b.count("pthread_create") > 1 or b.count("thrd_create") > 1


def main():
    c = Counter()
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "valid-memsafety" not in text:
            continue
        exp = None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "valid-memsafety" in ln and "property_file" in ln:
                for j in range(i, min(i + 5, len(lines))):
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                break
        if exp is None:
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists():
            continue
        conc_dir = any(d in str(src) for d in CONC_DIRS)
        thr = (not conc_dir) and spawns(src)
        c["total"] += 1
        c["safe" if exp else "buggy"] += 1
        if conc_dir:
            c["conc_dir"] += 1
        elif thr:
            c["thread_spawn"] += 1
            c["thread_spawn_buggy" if not exp else "thread_spawn_safe"] += 1
        else:
            c["sequential"] += 1
            c["sequential_buggy" if not exp else "sequential_safe"] += 1
    print("valid-memsafety reservoir:")
    for k in ("total", "safe", "buggy", "conc_dir", "thread_spawn",
              "thread_spawn_buggy", "thread_spawn_safe",
              "sequential", "sequential_buggy", "sequential_safe"):
        print(f"  {k:<20} = {c[k]}")
    if c["buggy"]:
        print(f"\n  thread-spawning buggy (R5 abstains; the recoverable lever) "
              f"= {c['thread_spawn_buggy']}/{c['buggy']} = {c['thread_spawn_buggy']/c['buggy']:.0%} of buggy")


if __name__ == "__main__":
    main()
