#!/usr/bin/env python3
"""Is SAF's TRUE-side wall the PER-FUNCTION TOP ENTRY STATE? Cross-tab test.

Hypothesis (from reading crates/saf-analysis/src/absint/fixpoint.rs:336-338 — every
function's params are seeded `Interval::make_top(64)`, and
crates/saf-analysis/src/absint/checker.rs:1180 abstains the moment any add/sub/mul
operand is TOP):

    both walls -- `ABSTAIN:top-or-bottom-operand` for no-overflow and
    `ABSTAIN:error-reachable` for unreach-call -- are the SAME root cause: a callee is
    analysed from a TOP entry, so its parameters are unconstrained.

PREDICTION, which this script tests: the sentinel's PROVE rate should be far higher on
single-function programs (everything in `main`, no callee to lose context across) than on
multi-function programs. If the prove rate is flat across function count, the hypothesis
is WRONG and context sensitivity is not the lever.

Emits per-task JSONL so the cross-tab can be re-cut without re-running.

    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \\
        'python3 /workspace/probe_context.py no-overflow 200'
"""
from __future__ import annotations

import collections
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

SAF = os.environ.get("SAF_BIN", "./target/release/saf")
SVB = os.environ.get("SAF_SVB", "tests/benchmarks/sv-benchmarks/c")
SUBCMD = {"unreach-call": "prove-unreachable", "no-overflow": "prove-no-overflow"}

# A C function DEFINITION at top level: type-ish tokens, name, (params), then `{` before
# any `;`. Crude but adequate: we only need a monotone proxy for "how many callees".
FUNCDEF = re.compile(
    rb"^[A-Za-z_][A-Za-z0-9_ \t\*\(\)]*?\b([A-Za-z_][A-Za-z0-9_]*)\s*\([^;{}]*\)\s*\{",
    re.MULTILINE)
# SV-COMP boilerplate that is not "the program's own structure".
BOILER = {b"reach_error", b"__VERIFIER_assert", b"assume_abort_if_not", b"abort",
          b"__assert_fail", b"main"}


def features(src: str) -> dict:
    s = open(src, "rb").read()
    names = set(FUNCDEF.findall(s))
    own = names - BOILER
    return {
        "n_funcdefs": len(names),
        "n_own_funcdefs": len(own),
        "has_assert_wrapper": bool(re.search(rb"void\s+__VERIFIER_assert\s*\(", s)),
        "sloc": len(s.splitlines()),
        "has_loop": bool(re.search(rb"\b(for|while)\s*\(|goto\s+\w+\s*;", s)),
        "has_array": bool(re.search(rb"\w+\s*\[\s*\d", s)),
        "has_float": bool(re.search(rb"\b(float|double)\b", s)),
        "has_recursion_hint": bool(re.search(rb"recursi", src.encode())),
    }


def main() -> int:
    prop = sys.argv[1] if len(sys.argv) > 1 else "no-overflow"
    want = int(sys.argv[2]) if len(sys.argv) > 2 else 200
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 8
    subcmd = SUBCMD[prop]

    import random
    random.seed(0)
    by_cluster: dict[str, list[dict]] = collections.defaultdict(list)
    for line in open("lever1-pertask.jsonl"):
        r = json.loads(line)
        if r["property"] == prop and r["expected"]:
            by_cluster[r.get("group", "")].append(r)
    pool = []
    for _, rs in sorted(by_cluster.items()):
        pool += random.sample(rs, min(3, len(rs)))
    tasks = random.sample(pool, min(want, len(pool)))
    print(f"{len(by_cluster)} clusters; probing {len(tasks)} with `saf {subcmd}`", flush=True)

    def run(r):
        base = r["rel_yml"][:-4]
        src = next((f"{SVB}/{base}{e}" for e in (".c", ".i") if os.path.exists(f"{SVB}/{base}{e}")), None)
        if not src:
            return None
        try:
            p = subprocess.run([SAF, subcmd, "--data-model", r["data_model"], src],
                               capture_output=True, text=True, timeout=150)
            lines = (p.stdout + p.stderr).strip().splitlines()
            out = next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))),
                       "(no output)")
        except subprocess.TimeoutExpired:
            out = "PROBE_TIMEOUT"
        try:
            feat = features(src)
        except Exception:  # noqa: BLE001
            feat = {}
        return {"group": r["group"], "rel_yml": r["rel_yml"], "outcome": out.split(" ")[0],
                **feat}

    res = [x for x in ThreadPoolExecutor(max_workers=jobs).map(run, tasks) if x]
    out_path = f"probe_context_{prop}.jsonl"
    with open(out_path, "w") as fh:
        for r in res:
            fh.write(json.dumps(r) + "\n")
    print(f"wrote {out_path} ({len(res)} rows)\n")

    def bucket(n):
        return "1 (main only)" if n <= 0 else "2-3" if n <= 2 else "4-9" if n <= 8 else "10+"

    print("PROVE rate by number of the program's OWN function definitions")
    print("(excludes reach_error/__VERIFIER_assert/assume_abort_if_not/abort/main)")
    tab = collections.defaultdict(collections.Counter)
    for r in res:
        tab[bucket(r.get("n_own_funcdefs", 0))][r["outcome"]] += 1
    order = ["1 (main only)", "2-3", "4-9", "10+"]
    for b in order:
        c = tab[b]
        n = sum(c.values())
        if not n:
            continue
        pr = c["PROVE"]
        print(f"  {b:15s} n={n:4d}  PROVE={pr:3d} ({100*pr/n:5.1f}%)   "
              + "  ".join(f"{k}={v}" for k, v in c.most_common() if k != "PROVE"))
    print()
    print("PROVE rate by presence of the __VERIFIER_assert wrapper")
    tab2 = collections.defaultdict(collections.Counter)
    for r in res:
        tab2[bool(r.get("has_assert_wrapper"))][r["outcome"]] += 1
    for k, c in tab2.items():
        n = sum(c.values())
        print(f"  wrapper={str(k):5s} n={n:4d}  PROVE={c['PROVE']:3d} ({100*c['PROVE']/n:5.1f}%)   "
              + "  ".join(f"{a}={b}" for a, b in c.most_common() if a != "PROVE"))
    print()
    print("outcome x structural feature (share of tasks with the feature)")
    feats = ["has_loop", "has_array", "has_float"]
    outs = sorted({r["outcome"] for r in res})
    print(f'  {"outcome":32s} {"n":>5s} ' + " ".join(f"{f:>10s}" for f in feats))
    for o in outs:
        rs = [r for r in res if r["outcome"] == o]
        print(f"  {o:32s} {len(rs):5d} "
              + " ".join(f"{100*sum(1 for r in rs if r.get(f))/len(rs):9.0f}%" for f in feats))
    return 0


if __name__ == "__main__":
    sys.exit(main())
