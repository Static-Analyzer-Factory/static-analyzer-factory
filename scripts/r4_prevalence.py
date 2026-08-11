import os, re, subprocess
from pathlib import Path
SVB = Path("tests/benchmarks/sv-benchmarks/c")
tasks = []
for yml in SVB.rglob("*.yml"):
    try: txt = yml.read_text(errors="ignore")
    except Exception: continue
    if "unreach-call.prp" not in txt: continue
    if not re.search(r"expected_verdict:\s*false", txt): continue
    m = re.search(r"input_files:\s*'?\"?([^'\"\n]+)", txt)
    if not m: continue
    src = (yml.parent / m.group(1).strip())
    if src.exists(): tasks.append(src)
tasks.sort()
small = [t for t in tasks if t.stat().st_size <= 300_000]
step = max(1, len(small)//500)
sample = small[::step]
print(f"total expected-false unreach: {len(tasks)}; <=300KB (R4-plausible): {len(small)}; sampling {len(sample)} (stride {step})", flush=True)
prp = "tests/programs/c/svcomp/unreach-call.prp"
caught=fired=fired_caught=fired_missed=0
for i, src in enumerate(sample):
    try:
        r = subprocess.run(["target/release/saf","verify","--property",prp,"--data-model","ILP32","--timeout","12",str(src)],
                           capture_output=True, text=True, timeout=28)
    except Exception: continue
    out = (r.stdout or "") + (r.stderr or "")
    verdict = (r.stdout or "").strip().splitlines()[-1] if r.stdout.strip() else ""
    r4 = "R4 enumerated" in out
    isfalse = verdict == "false(unreach-call)"
    if isfalse: caught+=1
    if r4:
        fired+=1
        if isfalse:
            fired_caught+=1
            print(f"  R4 FIRED+CAUGHT: {src.name}", flush=True)
        else:
            fired_missed+=1
            print(f"  R4 FIRED+missed: {src.name}", flush=True)
    if (i+1)%100==0: print(f"  ...{i+1}/{len(sample)} caught={caught} fired={fired}", flush=True)
print(f"=== PREVALENCE sample={len(sample)} caught_false={caught} R4_fired={fired} R4_fired_and_caught={fired_caught} R4_fired_but_missed={fired_missed} ===", flush=True)
