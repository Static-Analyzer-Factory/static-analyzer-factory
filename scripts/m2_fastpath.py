#!/usr/bin/env python3
"""Price the Anchored-Object prover's staged gate over the valid-memsafety corpus.

Stage 0 (near-free, module declaration scan): reject if the program calls any
        external a memsafety policy cannot admit -- notably the heap (obligation 1)
        and the pointer-dereferencing libc (obligation 2).
Stage 1 (cheap, CallGraph + BFS): universe gate with a NON-spawn-admitting policy.
Stage 2 (expensive, Andersen PTA + ICFG + MTA): only for programs whose reachable
        universe really does spawn.

Approximated on source text by call-shaped occurrences `name(`. This over-counts
stage-2 escalation (a call in dead code still counts), so the stage-2 figure is an
UPPER bound and the stage-0 rejection figure a LOWER bound.
"""
import collections, os, re, sys

SPLITS = "tests/benchmarks/svcomp-splits"
INPUT_RE = re.compile(r"""input_files:\s*['"]?([^'"\n]+)['"]?""")

# Obligation 1: any allocation or free at all.
HEAP = ["malloc", "calloc", "realloc", "free", "alloca", "__builtin_alloca",
        "valloc", "aligned_alloc", "posix_memalign", "strdup", "strndup"]
# Obligation 2: libc that dereferences a caller-supplied pointer.
NON_INERT = ["memcpy", "memmove", "memset", "memcmp", "strcpy", "strncpy", "strcat",
             "strncat", "strcmp", "strncmp", "strlen", "sprintf", "snprintf", "sscanf",
             "scanf", "fscanf", "fgets", "fread", "fwrite", "read", "write", "qsort",
             "bsearch", "wcscpy", "wcsncpy", "wcscat", "swprintf", "printf", "fprintf",
             "puts", "fopen", "fclose", "getenv", "atoi", "strtol"]
SPAWN = ["pthread_create", "thrd_create"]


def call_re(names):
    return re.compile(r"\b(" + "|".join(re.escape(n) for n in names) + r")\s*\(")


HEAP_RE, NI_RE, SPAWN_RE = call_re(HEAP), call_re(NON_INERT), call_re(SPAWN)


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
                ymls.add(line.lstrip("./").removeprefix("../"))

    stage = collections.Counter()
    esc_by_group = collections.Counter()
    survivors = []
    for y in sorted(ymls):
        try:
            txt = open(y, errors="ignore").read()
        except OSError:
            stage["unresolved"] += 1; continue
        m = INPUT_RE.search(txt)
        if not m:
            stage["unresolved"] += 1; continue
        src = os.path.join(os.path.dirname(y), m.group(1).strip())
        try:
            body = open(src, errors="ignore").read()
        except OSError:
            stage["unresolved"] += 1; continue

        heap = bool(HEAP_RE.search(body))
        ni = bool(NI_RE.search(body))
        spawn = bool(SPAWN_RE.search(body))
        if heap or ni:
            stage["stage0-reject (heap/non-inert libc)"] += 1
            continue
        if spawn:
            stage["stage2-ESCALATE to PTA"] += 1
            esc_by_group[os.path.basename(os.path.dirname(y))] += 1
            survivors.append(y)
            continue
        stage["stage1-cheap universe only"] += 1
        survivors.append(y)

    tot = sum(stage.values())
    print(f"valid-memsafety corpus: {tot} tasks\n")
    for k, v in stage.most_common():
        print(f"  {k:38s} {v:6d}  ({100.0 * v / tot:5.2f}%)")
    pta = stage["stage2-ESCALATE to PTA"]
    print(f"\n  tasks reaching Andersen PTA (UPPER bound): {pta}  ({100.0 * pta / tot:.2f}%)")
    print(f"  tasks never building a call graph at all : {stage['stage0-reject (heap/non-inert libc)']}")
    print("\nescalating tasks by group:")
    for g, c in esc_by_group.most_common(15):
        print(f"   {g:32s} {c}")


if __name__ == "__main__":
    main()
