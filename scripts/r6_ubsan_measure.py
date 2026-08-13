#!/usr/bin/env python3
"""Slice 0c/0d — R6 UBSan-gate no-overflow reservoir measurement (measurement only; NO production code).

Mirrors scripts/r5_asan_measure.py, swapping the arbiter to UBSan signed-overflow.
For a stratified sample of real `no-overflow` sv-benchmarks tasks (safe + buggy,
ILP32 + LP64), compile the ORIGINAL program natively with
    clang-18 -O0 -g -fsanitize=signed-integer-overflow
             -fno-sanitize-recover=signed-integer-overflow {-m32|-m64}
             -include <stub> -I<srcdir> <src> <constdriver.c>
run it under a multi-constant mini-fuzz (0 first, then a spread), and record whether
UBSan reports a signed-integer overflow (+ the faulting FRAME #0 function via
print_stacktrace=1). Classify vs the .yml expected_verdict (no-overflow has NO
sub-property — the verdict is just false(no-overflow)):
    exp=safe (True)  & UBSan trap -> FP  (the -16 false alarm; MUST be ~0)   <-- characterize EVERY one
    exp=safe         & no trap    -> TN
    exp=buggy (False)& UBSan trap -> TP  (confirmed FALSE = mini-fuzz recall)
    exp=buggy        & no trap    -> FN  (miss: guarded/needs-steering/loop-deep/out-of-scope)

Plan-198 discipline: the FP surface must be characterized AT SCALE (the LDV-model FP
and SEGV-in-free -16 only surfaced at N=100-200 in R5, not the small de-risk sample).
So for EVERY safe-task trap we print frame #0's function + located line + message, and
emit a frame histogram, so the R1-analog harness-frame rejection can be designed to
drive threaded-safe FP -> 0.

Run (throwaway root container is NOT needed — the ubsan runtime already ships in the
image via the R5 libclang-rt layer):
    docker compose run --rm -T --entrypoint sh dev -c 'python3 scripts/r6_ubsan_measure.py'
Env: N (per class x data-model, default 60), RUN_TIMEOUT (default 10),
     CONC=1 (measure ONLY the concurrency dirs, to probe the thread FP surface).
"""
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
STUB = "share/saf/stubs/sv-comp-stubs.h"
DRIVER = "scripts/r5_asan_constdriver.c"           # sanitizer-agnostic nondet defs
CLANG = os.environ.get("SAF_CLANG", "clang-18")
N = int(os.environ.get("N", "60"))                 # per class; total ~= 4*N (safe/buggy x dm)
RUN_TIMEOUT = float(os.environ.get("RUN_TIMEOUT", "10"))
# Deterministic non-coredumping exit + a symbolized frame #0 for R1 attribution.
# Do NOT set external_symbolizer_path — a bad value breaks symbolization; the runtime
# auto-finds llvm-symbolizer-18/addr2line (verified in the toolchain probe).
UBSAN_OPTIONS = "halt_on_error=1:abort_on_error=0:print_stacktrace=1"
# The mini-fuzz spread: 0 FIRST (unconditional case), then boundary/overflow-prone
# constants. INT_MIN/INT_MAX added for -x, x/-1, x-1, x*large overflow triggers.
NONDET_CONSTS = [0, 1, 2, 42, 255, 256, 1024, 65535, 2147483647, -1, -2147483648, 2147483648]
# UBSan signed-overflow message forms (all three are genuine signed overflow):
#   "signed integer overflow: A op B cannot be represented in type 'T'"  (+,-,*)
#   "negation of X cannot be represented in type 'T'"                    (unary -)
#   "division of X by -1 cannot be represented in type 'T'"              (INT_MIN/-1)
OVF_RE = re.compile(r"runtime error: (signed integer overflow|negation of|division of)")
# A UBSan print_stacktrace frame:  #0 0x... in <func> <file>:<line>:<col>
FRAME_RE = re.compile(r"^\s*#(\d+)\s+0x[0-9a-fA-F]+\s+in\s+(\S+)\s+(.+)$")
LOC_RE = re.compile(r"runtime error:")
# Dedicated concurrency benchmark dirs (real threads, not Juliet dead scaffolding).
# Excluded from the primary FP/recall pass; CONC=1 measures ONLY these.
CONC_DIRS = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/ldv-linux-3.14-races/",
             "/libvsync/", "/locks/", "/ddv-machzwd/")
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")

# R1-analog harness/library helper predicates (mirrors memsafety.rs is_* helpers) —
# used ONLY to LABEL the FP surface here; the production R1 lives in Rust.
def is_harness_frame(func):
    f = func
    if f.startswith("ldv_") and ("alloc" in f or "free" in f):
        return "ldv_model"
    if "printf" in f or "scanf" in f or f in ("puts", "fputs", "fwrite", "putchar", "putc"):
        return "libc_io"
    if f.startswith("print") and "Line" in f:
        return "juliet_print"
    if f.startswith("__VERIFIER") or f.startswith("ldv_"):
        return "verifier_model"
    return None


def is_concurrency(src):
    return any(d in src for d in CONC_DIRS)


def collect():
    tasks = []
    conc = os.environ.get("CONC") == "1"
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "no-overflow" not in text:
            continue
        exp = None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "no-overflow" in ln and "property_file" in ln:
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
        if is_concurrency(str(src)) != conc:
            continue
        md = DM_RE.search(text)
        tasks.append({"src": str(src), "exp": exp, "dm": md.group(1) if md else "LP64",
                      "size": src.stat().st_size,
                      "fam": str(src).split("sv-benchmarks/c/")[-1].split("/")[0]})
    return tasks


def stride(lst, n):
    lst = sorted(lst, key=lambda t: (t["size"], t["src"]))
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def ubsan_run(src, dm):
    """Compile once, mini-fuzz over constants (0 first). Returns:
       ('trap', k, frame_func, loc_line, msg) | ('clean', ...) | ('compile_fail'/'timeout', ...)."""
    flag = "-m32" if dm.upper() == "ILP32" else "-m64"
    binp = "/tmp/r6_ubsan_h"
    srcdir = str(Path(src).parent)
    try:
        c = subprocess.run(
            [CLANG, "-O0", "-g", "-fsanitize=signed-integer-overflow",
             "-fno-sanitize-recover=signed-integer-overflow",
             flag, "-include", STUB, "-I", srcdir, "-Wno-everything", src, DRIVER, "-o", binp],
            capture_output=True, text=True, errors="replace", timeout=60)
    except subprocess.TimeoutExpired:
        return ("compile_fail", None, None, None, None)
    if c.returncode != 0:
        return ("compile_fail", None, None, None, None)
    env0 = dict(os.environ, UBSAN_OPTIONS=UBSAN_OPTIONS)
    for k in NONDET_CONSTS:
        env = dict(env0, SAF_NONDET_CONST=str(k))
        try:
            r = subprocess.run([binp], capture_output=True, text=True, errors="replace",
                               timeout=RUN_TIMEOUT, env=env)
        except subprocess.TimeoutExpired:
            continue
        err = r.stderr or ""
        ovf = OVF_RE.search(err)
        if ovf:
            # frame #0 function (skip the runtime's own frames if any)
            frame_func = None
            for line in err.splitlines():
                m = FRAME_RE.match(line)
                if m and m.group(1) == "0":
                    frame_func = m.group(2)
                    break
            loc = next((ln.strip() for ln in err.splitlines() if LOC_RE.search(ln)), "")
            return ("trap", k, frame_func, loc[:130], ovf.group(1))
    return ("clean", None, None, None, None)


def main():
    conc = os.environ.get("CONC") == "1"
    tasks = collect()
    safe = [t for t in tasks if t["exp"]]
    buggy = [t for t in tasks if not t["exp"]]
    sample = []
    for pool in (safe, buggy):
        for dm in ("ILP32", "LP64"):
            sample += stride([t for t in pool if t["dm"] == dm], N)
    scope = "CONCURRENCY-ONLY" if conc else "sequential+juliet (conc dirs excluded)"
    fam = Counter(t["fam"] for t in tasks)
    print(f"no-overflow tasks [{scope}]: {len(tasks)} (safe={len(safe)} buggy={len(buggy)}); "
          f"sampling {len(sample)} (N={N}/class/dm)")
    print(f"  families: {dict(fam.most_common(12))}\n")

    conf = Counter()
    fp_frames = Counter()
    fps, misses = [], []
    zero_recall = 0  # buggy TPs caught at constant 0 (unsteered)
    tp_by_dm = Counter()
    for t in sample:
        name = Path(t["src"]).name
        outcome, k, frame, loc, msg = ubsan_run(t["src"], t["dm"])
        if outcome in ("compile_fail", "timeout"):
            conf[outcome] += 1
            tag = outcome
        elif outcome == "trap":
            frm = frame or "?"
            harness = is_harness_frame(frm)
            if t["exp"]:                       # SAFE + trap = FALSE ALARM
                conf["FP"] += 1
                fp_frames[f"{frm}{'['+harness+']' if harness else ''}"] += 1
                fps.append(f"{name}[{t['dm']}] @const={k} frame0={frm} :: {loc}")
                tag = f"FP!! frame0={frm}{'/'+harness if harness else ''}"
            else:                              # BUGGY + trap = TP
                conf["TP"] += 1
                tp_by_dm[t["dm"]] += 1
                if k == 0:
                    zero_recall += 1
                tag = f"TP @const={k} frame0={frm}"
        else:  # clean
            if t["exp"]:
                conf["TN"] += 1
                tag = "TN"
            else:
                conf["FN"] += 1
                misses.append(f"{name}[{t['dm']}]({t['fam']})")
                tag = "FN(miss)"
        print(f"  exp={'safe ' if t['exp'] else 'buggy'} {t['dm']:<5} {tag:<40} {name}")

    tp, fn, fp, tn = conf["TP"], conf["FN"], conf["FP"], conf["TN"]
    print(f"\n=== R6 UBSan-gate no-overflow ({scope}) ===")
    print(f"  TP={tp}  FN={fn}  FP(false alarm on safe)={fp}  TN={tn}  "
          f"compile_fail={conf['compile_fail']}  timeout={conf['timeout']}")
    if tp + fn:
        print(f"  mini-fuzz RECALL = {tp}/{tp+fn} = {tp/(tp+fn):.0%}  "
              f"(unsteered@const0 = {zero_recall}/{tp} of the TPs)")
    if fp + tn:
        print(f"  FALSE-ALARM rate on safe = {fp}/{fp+tn}  <-- MUST be ~0 (post-R1) for a sound gate")
    print(f"  TP by data-model: ILP32={tp_by_dm['ILP32']}  LP64={tp_by_dm['LP64']}")
    if fp_frames:
        print(f"\n  !!! FP FRAME #0 HISTOGRAM (the R1-rejection design surface):")
        for frm, c in fp_frames.most_common():
            print(f"      {c:>3}  {frm}")
        print("  --- every FP (const, frame0, located line) ---")
        for s in fps:
            print("      " + s)
    if misses:
        print(f"\n  sample misses (first 20): {', '.join(misses[:20])}")


if __name__ == "__main__":
    main()
