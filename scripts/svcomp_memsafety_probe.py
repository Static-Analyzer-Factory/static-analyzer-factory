#!/usr/bin/env python3
"""Memsafety checker false-alarm/recall probe (audit session).

Samples real valid-memsafety sv-benchmarks tasks, compiles each with the `verify`
recipe, runs `saf run --checkers all --format json`, and classifies whether SAF's
memory checkers fire. Mirrors the harness memsafety derivation (any relevant
finding => would-emit FALSE). Measures the FALSE-ALARM rate on expected-true (safe)
programs = the -16 exposure if memsafety were wired blind into `verify`.

Container, workspace root:
    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev python3 scripts/_probe_memsafety.py
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
SAF = "target/release/saf"
STUB = "share/saf/stubs/sv-comp-stubs.h"
N = int(os.environ.get("N", "12"))
DUMP1 = os.environ.get("DUMP1") == "1"

INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")
MEM_KW = ("null", "leak", "memtrack", "use-after", "uaf", "double-free",
          "double_free", "free", "deref", "buffer", "overflow", "oob", "dangling")


def collect():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "valid-memsafety" not in text:
            continue
        # expected verdict of the valid-memsafety property block
        exp = None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "valid-memsafety" in ln and "property_file" in ln:
                for j in range(i, min(i + 4, len(lines))):
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                        break
                break
        if exp is None:
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists():
            continue
        md = DM_RE.search(text)
        dm = md.group(1) if md else "LP64"
        tasks.append({"src": str(src), "exp": exp, "dm": dm, "size": src.stat().st_size})
    return tasks


def stride(lst, n):
    lst = sorted(lst, key=lambda t: (t["size"], t["src"]))  # small first
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def compile_ir(src, dm):
    flag = "-m32" if dm.upper() == "ILP32" else "-m64"
    ir = "/tmp/probe.ll"
    c = subprocess.run(["clang-18", "-g", "-S", "-emit-llvm", "-O0", "-Xclang",
                        "-disable-O0-optnone", flag, "-include", STUB,
                        "-Wno-everything", src, "-o", ir],
                       capture_output=True, text=True)
    if c.returncode != 0:
        return None
    o = subprocess.run(["opt-18", "-passes=mem2reg", ir, "-o", ir, "-S"],
                       capture_output=True, text=True)
    return ir if o.returncode == 0 else None


def has_mem_finding(src, dm):
    ir = compile_ir(src, dm)
    if ir is None:
        return None  # compile failure -> excluded
    r = subprocess.run([SAF, "run", ir, "--checkers", "all", "--format", "json"],
                       capture_output=True, text=True, timeout=60)
    out = r.stdout
    if DUMP1:
        Path("/tmp/probe_first.json").write_text(out)
        sys.stderr.write(f"[dump] wrote /tmp/probe_first.json ({len(out)} bytes); "
                         f"stderr tail: {r.stderr[-300:]}\n")
    try:
        data = json.loads(out)
    except json.JSONDecodeError:
        # fall back to keyword scan of raw output
        return any(k in out.lower() for k in MEM_KW)
    findings = []
    if isinstance(data, dict):
        for key in ("findings", "results", "nodes"):
            if key in data and isinstance(data[key], list):
                findings = data[key]
                break
    elif isinstance(data, list):
        findings = data
    if not findings:
        # some exports nest; keyword-scan as backup
        return any(k in out.lower() for k in ("null", "leak", "use-after", "double", "dangling"))
    txt = json.dumps(findings).lower()
    return any(k in txt for k in MEM_KW)


def main():
    tasks = collect()
    trues = stride([t for t in tasks if t["exp"]], N)
    falses = stride([t for t in tasks if not t["exp"]], N)
    print(f"valid-memsafety tasks: {len(tasks)}; sampling {len(trues)} safe + {len(falses)} buggy (small-first)\n")
    conf = {"TP": 0, "FP": 0, "FN": 0, "TN": 0, "compile_fail": 0}
    fps = []
    for t in trues + falses:
        fired = has_mem_finding(t["src"], t["dm"])
        if fired is None:
            conf["compile_fail"] += 1
            cls = "compile_fail"
        elif t["exp"] and fired:
            conf["FP"] += 1; cls = "FP(false-alarm)"; fps.append(Path(t["src"]).name)
        elif t["exp"] and not fired:
            conf["TN"] += 1; cls = "TN"
        elif (not t["exp"]) and fired:
            conf["TP"] += 1; cls = "TP(caught)"
        else:
            conf["FN"] += 1; cls = "FN(missed)"
        print(f"  exp={'safe ' if t['exp'] else 'buggy'} fired={str(fired):<5} {cls:<16} {Path(t['src']).name}")
    print("\n=== memsafety checker posture (findings-as-FALSE) ===")
    print(f"  TP(caught buggy)={conf['TP']}  FN(missed buggy)={conf['FN']}  "
          f"FP(false alarm on safe)={conf['FP']}  TN(safe clean)={conf['TN']}  compile_fail={conf['compile_fail']}")
    tp, fp, fn = conf["TP"], conf["FP"], conf["FN"]
    if tp + fn:
        print(f"  recall (buggy caught)     = {tp}/{tp+fn}")
    if conf["FP"] + conf["TN"]:
        print(f"  FALSE-ALARM rate on safe  = {fp}/{fp+conf['TN']}  <-- the -16 exposure if wired blind")
    if fps:
        print("  false-alarm tasks:", ", ".join(fps))


if __name__ == "__main__":
    main()
