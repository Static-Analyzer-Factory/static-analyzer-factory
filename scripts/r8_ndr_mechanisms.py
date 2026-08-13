#!/usr/bin/env python3
"""R8 Slice-0 de-risk: enumerate concurrency-creation mechanisms across the
SV-COMP `no-data-race` reservoir, and estimate the provably-sequential subset.

SOURCE-LEVEL scan only (cheap, no compile). Feeds two de-risk questions:
  (1) prevalence  — how many no-data-race=true tasks look sequential (no spawn token)?
  (2) completeness — which spawn/concurrency mechanisms actually occur (the -32 surface)?

The authoritative prevalence number comes from a later IR-level probe using SAF's
real `reachable_spawns_threads`; this source scan bounds/targets it.
"""
import os
import re
import sys
from collections import defaultdict

ROOT = os.path.expanduser("~/static-analyzer-factory/tests/benchmarks/sv-benchmarks/c")

# Mechanism -> regex over source text. Grouped by whether the CURRENT gate
# (SPAWN_FUNCTIONS = pthread_create, thrd_create) recognizes it.
MECHS = {
    # recognized by reachable_spawns_threads
    "pthread_create": re.compile(r"\bpthread_create\b"),
    "thrd_create": re.compile(r"\bthrd_create\b"),
    # NOT recognized -> the -32 completeness surface
    "GOMP_parallel": re.compile(r"\bGOMP_parallel\b"),
    "__kmpc_fork": re.compile(r"\b__kmpc_fork(_call|_teams)?\b"),
    "pragma_omp": re.compile(r"#\s*pragma\s+omp"),
    "fork": re.compile(r"\bfork\s*\("),
    "clone": re.compile(r"\bclone\s*\("),
    "pthread_kill_signal": re.compile(r"\b(pthread_kill|sigaction|signal)\s*\("),
    # concurrency indicators (already-threaded context; not spawns themselves)
    "verifier_atomic": re.compile(r"__VERIFIER_atomic"),
    "pthread_t_decl": re.compile(r"\bpthread_t\b"),
    "pthread_mutex": re.compile(r"\bpthread_mutex_(lock|unlock|init)\b"),
    # SV-COMP thread-creation aliases sometimes used
    "pthread_create_alias": re.compile(r"\b__VERIFIER_thread|pthread_create_2\b"),
}

INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)['\"]?")


def find_ymls(root):
    out = []
    for dp, _dn, fns in os.walk(root):
        for fn in fns:
            if fn.endswith((".yml", ".yaml")):
                out.append(os.path.join(dp, fn))
    return out


def yml_ndr_verdict(text):
    """Return expected_verdict ('true'/'false'/None) for the no-data-race property,
    or 'ABSENT' if the yml doesn't reference no-data-race."""
    if "no-data-race" not in text:
        return "ABSENT"
    # find the property block mentioning no-data-race and its expected_verdict
    # blocks look like:  - property_file: ../properties/no-data-race.prp\n    expected_verdict: true
    for m in re.finditer(r"property_file:\s*\S*no-data-race\.prp(.*?)(?=property_file:|\Z)",
                         text, re.DOTALL):
        v = re.search(r"expected_verdict:\s*(true|false)", m.group(1))
        if v:
            return v.group(1)
    return None


def main():
    ymls = find_ymls(ROOT)
    tasks = []  # (yml, srcpath, verdict)
    for y in ymls:
        try:
            t = open(y, "r", errors="replace").read()
        except OSError:
            continue
        verdict = yml_ndr_verdict(t)
        if verdict == "ABSENT":
            continue
        im = INPUT_RE.search(t)
        src = os.path.join(os.path.dirname(y), im.group(1).strip()) if im else None
        tasks.append((y, src, verdict))

    print(f"no-data-race tasks: {len(tasks)}")
    vc = defaultdict(int)
    for _, _, v in tasks:
        vc[v] += 1
    print(f"  expected_verdict: {dict(vc)}")

    # scan sources
    mech_task = defaultdict(lambda: defaultdict(int))  # mech -> verdict -> count
    no_spawn_token = defaultdict(int)  # verdict -> count of tasks with NO spawn-creating token
    missing = 0
    SPAWNY = ["pthread_create", "thrd_create", "GOMP_parallel", "__kmpc_fork",
              "pragma_omp", "fork", "clone", "pthread_create_alias"]
    per_dir_seq = defaultdict(lambda: defaultdict(int))  # dir -> verdict -> count seq-looking
    for y, src, verdict in tasks:
        text = ""
        if src and os.path.isfile(src):
            try:
                text = open(src, "r", errors="replace").read()
            except OSError:
                text = ""
        else:
            missing += 1
        present = set()
        for mech, rx in MECHS.items():
            if rx.search(text):
                present.add(mech)
                mech_task[mech][verdict] += 1
        # "sequential-looking" = none of the SPAWNY tokens present
        if not (present & set(SPAWNY)):
            no_spawn_token[verdict] += 1
            top = os.path.relpath(os.path.dirname(y), ROOT).split(os.sep)[0]
            per_dir_seq[top][verdict] += 1

    print(f"\nsource files missing/unreadable: {missing}")
    print("\n=== mechanism presence (tasks with >=1 match), by expected_verdict ===")
    for mech in MECHS:
        d = mech_task[mech]
        if d:
            print(f"  {mech:24s} true={d.get('true',0):5d} false={d.get('false',0):5d}")

    print("\n=== 'sequential-looking' tasks (NO pthread_create/thrd_create/GOMP/__kmpc/omp/fork/clone token) ===")
    print(f"  by verdict: {dict(no_spawn_token)}")
    print("  (these are R8 TRUE candidates if verdict=true; the -32 risk if verdict=false)")
    print("\n  breakdown by top dir (seq-looking only):")
    for d in sorted(per_dir_seq):
        print(f"    {d:26s} {dict(per_dir_seq[d])}")


if __name__ == "__main__":
    main()
