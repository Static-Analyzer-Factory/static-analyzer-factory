#!/usr/bin/env python3
"""Plan 198 de-risk — characterize the ASan behaviour of THREAD-SPAWNING
valid-memsafety tasks (the reservoir R5 abstains on).

Mirrors `asan_confirm` in crates/saf-cli/src/commands.rs EXACTLY (same nondet
driver, clang flags, `-include` stub, `ASAN_OPTIONS`, and NONDET_CONSTS mini-fuzz
with 0 first / first-trap-wins) so the confirm/abstain decision matches production
— but, unlike the real binary (which discards the raw report), it SURFACES the
signals the plan-198 gate design needs:

  * ASan error class + sub-property (R1 I/O-frame reject + R2 high-fidelity map),
  * the faulting frame function,
  * the THREAD the fault is on (`T0` = main, `Tn` = a worker),
  * DETERMINISM: re-run the confirming input K times and count how many trap with
    the same class (K/K = deterministic worker bug; < K = schedule-dependent race).

The decisive question: over thread-spawning SAFE tasks, how many CONFIRM (a −16
false alarm if the abstain were dropped), and are those confirmations deterministic
worker faults (soundly catchable / mislabels) or intermittent races (must abstain)?
Over thread-spawning BUGGY tasks, how many confirm (the recall prize) and are they
deterministic single-worker faults (→ a structural gate is sound)?

Run (container, workspace root):
    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev python3 scripts/r5_thread_derisk.py
Env: N (per class x data-model, default 40), K (repetitions, default 5),
     REPLAY_TIMEOUT (s, default 5), CWE_STRATIFY (1 = one task per CWE dir first).
"""
import os
import re
import subprocess
import tempfile
from collections import Counter, defaultdict
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
STUB = Path("share/saf/stubs/sv-comp-stubs.h")
CLANG = os.environ.get("SAF_CLANG", "clang-18")
N = int(os.environ.get("N", "40"))
K = int(os.environ.get("K", "5"))
REPLAY_TIMEOUT = int(os.environ.get("REPLAY_TIMEOUT", "5"))
CONC_DIRS = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/ldv-linux-3.14-races/",
             "/libvsync/", "/locks/", "/ddv-machzwd/")
# Faithful to commands.rs: ASAN_OPTS + NONDET_CONSTS (0 first).
ASAN_OPTS = "exitcode=1:abort_on_error=0:detect_leaks=0:check_printf=0"
NONDET_CONSTS = [0, 1, 2, 42, 255, 256, 1024, 65535, 2147483647, -1]
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")

# Faithful replica of synthesize_asan_driver() (commands.rs).
SCALAR_NONDET = [
    ("int", "int"), ("uint", "unsigned int"), ("long", "long"),
    ("ulong", "unsigned long"), ("longlong", "long long"),
    ("ulonglong", "unsigned long long"), ("short", "short"),
    ("ushort", "unsigned short"), ("char", "char"), ("uchar", "unsigned char"),
    ("bool", "_Bool"), ("size_t", "size_t"),
]


def driver_src():
    s = ["/* plan-198 de-risk ASan driver (mirrors synthesize_asan_driver) */",
         "#include <stddef.h>", "#include <stdlib.h>",
         "extern void _exit(int) __attribute__((noreturn));",
         'static long __saf_c(void){const char*e=getenv("SAF_NONDET_CONST");return e?atol(e):0;}']
    for suffix, cty in SCALAR_NONDET:
        s.append(f"{cty} __VERIFIER_nondet_{suffix}(void){{return ({cty})__saf_c();}}")
    s += ["void* __VERIFIER_nondet_pointer(void){return (void*)0;}",
          "float __VERIFIER_nondet_float(void){return 0.0f;}",
          "double __VERIFIER_nondet_double(void){return 0.0;}",
          "void __VERIFIER_atomic_begin(void){}",
          "void __VERIFIER_atomic_end(void){}",
          "void __VERIFIER_assume(int c){if(!c)_exit(0);}"]
    return "\n".join(s) + "\n"


# ---- R1/R2 parsing (mirror crates/saf-svcomp/src/memsafety.rs) ----
def frame_function(descr):
    end = len(descr)
    for i, ch in enumerate(descr):
        if ch in "( ":
            end = i
            break
    return descr[:end]


def is_format_io(f):
    return ("printf" in f or "scanf" in f
            or f in ("puts", "fputs", "fwrite", "putchar", "putc"))


def is_print_helper(f):
    return f.startswith("print") and "Line" in f


def is_harness_memory_model(f):
    # sv-benchmarks LDV allocator MODELS (ldv_reference_realloc &c.) — a native fault
    # inside one is a model artifact (e.g. realloc's memcpy(new, old, NEW_size) OOB
    # read), not a program memory-safety violation. Mirrors memsafety.rs R1.
    return f.startswith("ldv_") and ("alloc" in f or "free" in f)


def parse_frame_location(descr):
    for raw in descr.split():
        tok = raw.strip("()")
        if "/" not in tok or "+" in tok:
            continue
        parts = tok.rsplit(":", 2)
        if len(parts) == 3 and parts[1].isdigit() and parts[2].isdigit():
            return parts[0], int(parts[1])
        if len(parts) == 2 and parts[1].isdigit():
            return parts[0], int(parts[1])
    return None


def class_to_subproperty(cls):
    if "double-free" in cls:
        return "valid-free"
    if any(x in cls for x in ("buffer-overflow", "buffer-underflow",
                              "buffer-overread", "buffer-underread",
                              "use-after-free", "use-after-scope",
                              "use-after-return", "use-after-poison")) or cls.startswith("SEGV"):
        return "valid-deref"
    return None


def parse_report(stderr):
    """Return dict(class, subprop, frame, r1_rejected, located) or None (no banner)."""
    MARK = "ERROR: AddressSanitizer: "
    i = stderr.find(MARK)
    if i < 0:
        return None
    cls_line = stderr[i + len(MARK):].splitlines()[0]
    subprop = class_to_subproperty(cls_line)
    out = {"cls": cls_line.split(" ")[0], "subprop": None, "frame": None,
           "r1": False, "located": False}
    if subprop is None:
        return out  # trapped but R2-abstain (unmapped class)
    for line in stderr.splitlines():
        t = line.lstrip()
        if not t.startswith("#") or " in " not in t:
            continue
        descr = t.split(" in ", 1)[1]
        fn = frame_function(descr)
        if is_format_io(fn) or is_print_helper(fn) or is_harness_memory_model(fn):
            out["r1"] = True
            out["frame"] = fn
            return out  # R1 reject (would abstain)
        loc = parse_frame_location(descr)
        if loc:
            out["subprop"] = subprop
            out["frame"] = fn
            out["located"] = True
            return out
    return out  # trapped, class mapped, but no locatable program frame -> abstain


def fault_thread(stderr):
    m = re.search(r"thread (T\d+)", stderr)
    return m.group(1) if m else "?"


# ---- task collection ----
def collect():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "valid-memsafety" not in text:
            continue
        exp, sub = None, None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "valid-memsafety" in ln and "property_file" in ln:
                for j in range(i, min(i + 5, len(lines))):
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                    ms = re.search(r"subproperty:\s*(valid-[a-z]+)", lines[j])
                    if ms:
                        sub = ms.group(1)
                break
        if exp is None:
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists() or any(d in str(src) for d in CONC_DIRS):
            continue
        try:
            body = src.read_text(errors="replace")
        except OSError:
            continue
        if not (body.count("pthread_create") > 1 or body.count("thrd_create") > 1):
            continue  # THREAD-SPAWNERS only
        md = DM_RE.search(text)
        # CWE family key (Juliet files sit directly under Juliet_Test/ with the CWE
        # encoded as the `CWEnnn_Name---sNN---...` filename prefix, NOT a subdir).
        base = Path(str(src)).name
        cwe = base.split("---")[0] if base.startswith("CWE") else src.parent.name
        tasks.append({"src": str(src), "exp": exp, "sub": sub, "cwe": cwe,
                      "dm": md.group(1) if md else "LP64", "size": src.stat().st_size})
    return tasks


def stride(lst, n):
    lst = sorted(lst, key=lambda t: (t["size"], t["src"]))
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def cwe_stratified(lst, n):
    """One task per CWE dir first (round-robin), then size-stride to fill n."""
    by = defaultdict(list)
    for t in lst:
        by[t["cwe"]].append(t)
    picked, seen = [], set()
    order = sorted(by)
    round_i = 0
    while len(picked) < n and any(round_i < len(by[c]) for c in order):
        for c in order:
            col = sorted(by[c], key=lambda t: (t["size"], t["src"]))
            if round_i < len(col) and len(picked) < n:
                picked.append(col[round_i])
                seen.add(col[round_i]["src"])
        round_i += 1
    return picked


def run_once(harness, k, errpath):
    with open(errpath, "w") as f:
        try:
            subprocess.run([harness], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                           stderr=f, env=dict(os.environ, ASAN_OPTIONS=ASAN_OPTS,
                                              SAF_NONDET_CONST=str(k)),
                           timeout=REPLAY_TIMEOUT)
        except subprocess.TimeoutExpired:
            return ""
        except OSError:
            return ""
    return Path(errpath).read_text(errors="replace")


def confirm(src, dm, d):
    """Mirror asan_confirm: compile ASan; sweep NONDET_CONSTS (0 first); first
    R1+R2 located hit wins. Then re-run the winning const K times for determinism."""
    drv = os.path.join(d, "drv.c")
    Path(drv).write_text(driver_src())
    harness = os.path.join(d, "h")
    errp = os.path.join(d, "e.txt")
    flag = "-m32" if dm == "ILP32" else "-m64"
    cc = subprocess.run(
        [CLANG, "-O0", "-g", "-fsanitize=address", "-fno-sanitize-recover=address",
         "-Wno-everything", flag, "-include", str(STUB), "-I", str(Path(src).parent),
         src, drv, "-o", harness],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if cc.returncode != 0:
        return {"status": "compile_fail"}
    winning = None
    for k in NONDET_CONSTS:
        rep = run_once(harness, k, errp)
        info = parse_report(rep)
        if info and info["located"]:  # R1+R2 pass, locatable program frame
            winning = (k, info, fault_thread(rep))
            break
        if info and (info["r1"] or info["subprop"] is None):
            # trapped but would ABSTAIN (R1 reject / unmapped class); keep looking
            # (mirrors parse_asan_report returning None -> next const)
            continue
    if winning is None:
        return {"status": "no_confirm"}
    kwin, info, thr = winning
    # Determinism: re-run the winning const K times, count same-class traps.
    same = 0
    threads = Counter()
    for _ in range(K):
        rep = run_once(harness, kwin, errp)
        r = parse_report(rep)
        if r and r["located"] and r["subprop"] == info["subprop"]:
            same += 1
            threads[fault_thread(rep)] += 1
    return {"status": "confirm", "subprop": info["subprop"], "cls": info["cls"],
            "frame": info["frame"], "thread": thr, "const": kwin,
            "determinism": f"{same}/{K}", "det_n": same,
            "threads": dict(threads)}


def main():
    tasks = collect()
    safe = [t for t in tasks if t["exp"]]
    buggy = [t for t in tasks if not t["exp"]]
    strat = cwe_stratified if os.environ.get("CWE_STRATIFY", "0") == "1" else \
        (lambda l, n: stride(l, n))
    sample = []
    for pool in (safe, buggy):
        for dm in ("ILP32", "LP64"):
            sample += strat([t for t in pool if t["dm"] == dm], N)
    print(f"thread-spawning valid-memsafety: {len(tasks)} "
          f"(safe={len(safe)} buggy={len(buggy)}); probing {len(sample)} "
          f"(K={K} reps, first-trap-wins const sweep)\n")

    agg = Counter()
    fp_detail, tp_detail = [], []
    for t in sample:
        name = Path(t["src"]).name
        with tempfile.TemporaryDirectory() as d:
            r = confirm(t["src"], t["dm"], d)
        st = r["status"]
        agg[st] += 1
        exps = "safe " if t["exp"] else "buggy"
        if st == "confirm":
            det = r["determinism"]
            row = (f"{name}[{t['dm']},{t['cwe']}] {r['subprop']} cls={r['cls']} "
                   f"thr={r['thread']} det={det} const={r['const']} frame={r['frame']}")
            if t["exp"]:
                agg["FP"] += 1
                fp_detail.append(row)
                tag = f"FP!! {r['subprop']} {r['thread']} det={det}"
            else:
                agg["TP"] += 1
                tp_detail.append(row)
                if r["det_n"] == K:
                    agg["TP_deterministic"] += 1
                if r["thread"] != "T0":
                    agg["TP_worker_thread"] += 1
                tag = f"TP {r['subprop']} {r['thread']} det={det}"
        else:
            if t["exp"]:
                agg["safe_no_confirm"] += 1
            else:
                agg["buggy_no_confirm"] += 1
            tag = st
        print(f"  {exps} {t['dm']:<5} {tag:<44} {name}")

    print("\n=== plan-198 de-risk: thread-spawning ASan characterization ===")
    print(f"  compile_fail={agg['compile_fail']}")
    print(f"  SAFE:  FP(confirm)={agg['FP']}  no_confirm={agg['safe_no_confirm']}")
    print(f"  BUGGY: TP(confirm)={agg['TP']}  no_confirm={agg['buggy_no_confirm']}")
    print(f"    of TP: deterministic(K/K)={agg['TP_deterministic']}  "
          f"worker-thread(Tn)={agg['TP_worker_thread']}")
    print(f"\n  *** THE −16 SURFACE: thread-spawning SAFE confirmations = {agg['FP']} "
          f"(each is a potential false alarm if the abstain is dropped) ***")
    if fp_detail:
        print("  FP detail (characterize: deterministic worker fault / race / mislabel):")
        for r in fp_detail:
            print("    -", r)
    else:
        print("  (no SAFE confirmations — the Juliet worker pattern is ASan-clean)")
    if tp_detail:
        print(f"\n  recall prize sample (first {min(12, len(tp_detail))} of {len(tp_detail)} TP):")
        for r in tp_detail[:12]:
            print("    +", r)


if __name__ == "__main__":
    main()
