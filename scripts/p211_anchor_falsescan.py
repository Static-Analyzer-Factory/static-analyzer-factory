"""Plan 211 Movement 2 spike: adversarial soundness scan for the Anchored-Object prover.

Runs the p211_anchor_prototype decision procedure over EXPECTED-FALSE valid-memsafety
tasks. Every task it accepts is a would-be wrong TRUE (-32, uncapped by the per-cluster
dedup rule), so the only acceptable result is zero.

Measured 2026-09-12: 1 escape in 1,180 tasks, and that escape was exactly the obligation
the prototype had not implemented yet (a non-inert libc call). The spike PASS bar in
plans/214 is 0 escapes over ALL 10,014 expected-FALSE tasks, not a sample.
"""
import os,re,glob,subprocess,sys,collections,random,tempfile
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from p211_anchor_prototype import analyze,parse_yml
BENCH="/workspace/tests/benchmarks/sv-benchmarks/c"
cap=int(sys.argv[1])
cands=[]
for y in glob.glob(os.path.join(BENCH,"*","*.yml")):
    try: inp,dm,props=parse_yml(y)
    except: continue
    if not inp: continue
    for pf,ev in props:
        if pf=="valid-memsafety.prp" and ev=="false":
            cands.append((os.path.join(os.path.dirname(y),inp),dm,os.path.basename(os.path.dirname(y))))
print("expected-FALSE memsafety tasks:",len(cands))
random.seed(3); random.shuffle(cands); cands=cands[:cap]
bad=[]; n=0
for src,dm,g in cands:
    m32=["-m32"] if dm.upper()=="ILP32" else []
    t=tempfile.NamedTemporaryFile(suffix=".ll",delete=False); t.close()
    r=subprocess.run(["clang-18","-S","-emit-llvm","-O0","-Xclang","-disable-O0-optnone","-w"]+m32+["-o",t.name,src],capture_output=True,text=True)
    if r.returncode!=0 or not os.path.exists(t.name): continue
    o2=t.name+".o.ll"
    subprocess.run(["opt-18","-S","-passes=sroa,mem2reg,instcombine","-o",o2,t.name],capture_output=True,text=True)
    os.unlink(t.name)
    if not os.path.exists(o2): continue
    ll=open(o2,errors="ignore").read(); os.unlink(o2); n+=1
    if not analyze(ll): bad.append((g,os.path.basename(src)))
print(f"scanned={n}  WOULD-PROVE(unsound)={len(bad)}")
for g,f in bad[:20]: print("   ",g,f)
