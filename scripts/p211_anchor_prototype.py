"""Plan 211 Movement 2 spike: the Anchored-Object `valid-memsafety` decision procedure,
prototyped over LLVM IR in Python so it can be measured before any Rust is written.

Anchor = {base: Global|Stack(alloca), offset: const}. A `Load` result never anchors, so a
pointer that came out of memory can never carry a discharged obligation. Every obligation
is SYNTACTIC (SSA pointer graph + static layout), which is why thread interference — which
only changes values — cannot invalidate it.

Required IR recipe (without mem2reg the -O0 pointer spill defeats anchoring outright):
    clang-18 -O0 -Xclang -disable-O0-optnone ... && opt-18 -passes=sroa,mem2reg,instcombine

Measured 2026-09-12: >=1 provable task in 10 of 16 concurrent unsolved clusters and 3 of
27 sequential. STILL MISSING (see plans/214 §2): the layout/bounds obligation and the
non-inert-libc-call gate.
"""
import os,re,glob,subprocess,sys,collections,random,tempfile
BENCH="/workspace/tests/benchmarks/sv-benchmarks/c"
def parse_yml(p):
    txt=open(p,errors="ignore").read()
    inp=re.search(r"input_files:\s*'([^']+)'",txt) or re.search(r'input_files:\s*"([^"]+)"',txt)
    dm=re.search(r"data_model:\s*(\w+)",txt)
    props=[(os.path.basename(m.group(1)),m.group(2)) for m in
        re.finditer(r"property_file:\s*(\S+)\s*\n\s*expected_verdict:\s*(\w+)",txt)]
    return (inp.group(1) if inp else None, dm.group(1) if dm else "ILP32", props)
HEAPFNS={"malloc","calloc","realloc","free","reallocarray","strdup","strndup","memalign",
         "posix_memalign","aligned_alloc","valloc","alloca"}
# CALLS that touch caller memory through a pointer arg -> must be modelled or abstain
MEMFNS={"memset","memcpy","memmove","strcpy","strncpy","strcat","strncat","sprintf",
        "snprintf","scanf","sscanf","fgets","read","recv","bzero","bcopy","qsort","bsearch"}
DEF=re.compile(r"^\s*(%[\w.]+)\s*=\s*(.*)$")
CALL=re.compile(r"(?:tail\s+)?(?:musttail\s+)?call\s+[^@]*@([\w.$\"]+)\s*\(")
def analyze(ll):
    defs={}
    for line in ll.splitlines():
        m=DEF.match(line)
        if m: defs[m.group(1)]=m.group(2).strip()
    # OPTIMISTIC greatest fixpoint over the pointer graph
    joinish={r for r,d in defs.items() if d.startswith("phi ") or d.startswith("select ")}
    cur={r:True for r in joinish}
    def base(op,memo,depth=0):
        op=op.strip()
        if depth>40: return False
        if op.startswith("@"): return True
        if not op.startswith("%"): return False
        reg=op.split()[0]
        if reg in memo: return memo[reg]
        if reg in joinish: memo[reg]=cur.get(reg,False)
        else: memo[reg]=False
        d=defs.get(reg)
        if d is None: return memo[reg]
        r=False
        if d.startswith("alloca "): r = not re.search(r",\s*i\d+\s+%", d)
        elif d.startswith("bitcast ") or d.startswith("addrspacecast "):
            m=re.search(r"(?:bitcast|addrspacecast)\s+\S+\s+(\S+)\s+to",d); r=bool(m) and base(m.group(1),memo,depth+1)
        elif d.startswith("getelementptr"):
            parts=[p.strip() for p in d.split("(",1)[0].split(",")]
            b=parts[1].split()[-1] if len(parts)>1 else ""
            r = all(not re.search(r"\bi\d+\s+%",p) for p in parts[2:]) and base(b,memo,depth+1)
        elif d.startswith("select "):
            ops=re.findall(r"ptr\s+(\S+?)[,\s]",d); r=len(ops)>=2 and all(base(o,memo,depth+1) for o in ops)
        elif d.startswith("phi "):
            ops=[o.strip() for o in re.findall(r"\[\s*([^,]+),",d)]
            r=bool(ops) and all(base(o,memo,depth+1) for o in ops)
        if reg not in joinish: memo[reg]=r
        return r
    for _ in range(6):
        memo={}
        nxt={r:base(r,memo) for r in joinish}
        if nxt==cur: break
        cur=nxt
    memo={}
    anchored=lambda op: base(op,memo)
    bad=collections.Counter()
    for line in ll.splitlines():
        s=line.strip()
        cm=CALL.search(s)
        if cm:
            fn=cm.group(1).strip('"'); b=fn.split(".")[0]
            if b in HEAPFNS or fn in HEAPFNS: bad["heap"]+=1
            if b in MEMFNS or fn in MEMFNS: bad["mem-libc-call"]+=1
            if fn.startswith("llvm.mem"):
                m=re.search(r"\(\s*ptr[^,]*\s(\S+?),",s)
                if not (m and anchored(m.group(1))): bad["mem-intrinsic"]+=1
                if re.search(r"i\d+\s+%\w+\s*,\s*i1",s): bad["mem-varlen"]+=1
        if re.search(r"\bcall\b[^@\n]*%[\w.]+\s*\(",s) and "@" not in s: bad["indirect-call"]+=1
        if " inttoptr " in s: bad["inttoptr"]+=1
        m=re.match(r"(?:%[\w.]+\s*=\s*)?load\s+(?:atomic\s+)?[^,]+,\s*ptr\s+(\S+?)[,\s]",s)
        if m and not anchored(m.group(1)): bad["load-unanchored"]+=1
        m=re.match(r"store\s+(?:atomic\s+)?.*,\s*ptr\s+(\S+?)[,\s]",s)
        if m and not anchored(m.group(1)): bad["store-unanchored"]+=1
    return bad
def run(groups,prop,cap,expect):
    tot=collections.Counter()
    for g in groups:
        cands=[]
        for y in sorted(glob.glob(os.path.join(BENCH,g,"*.yml"))):
            inp,dm,props=parse_yml(y)
            if not inp: continue
            for pf,ev in props:
                if pf==prop+".prp" and ev==expect: cands.append((os.path.join(os.path.dirname(y),inp),dm))
        random.seed(11); random.shuffle(cands); cands=cands[:cap]
        n=ok=0; reasons=collections.Counter()
        for src,dm in cands:
            m32=["-m32"] if dm.upper()=="ILP32" else []
            t=tempfile.NamedTemporaryFile(suffix=".ll",delete=False); t.close()
            r=subprocess.run(["clang-18","-S","-emit-llvm","-O0","-Xclang","-disable-O0-optnone","-w"]+m32+["-o",t.name,src],capture_output=True,text=True)
            if r.returncode!=0 or not os.path.exists(t.name): continue
            o2=t.name+".o.ll"
            subprocess.run(["opt-18","-S","-passes=sroa,mem2reg,instcombine","-o",o2,t.name],capture_output=True,text=True)
            os.unlink(t.name)
            if not os.path.exists(o2): continue
            ll=open(o2,errors="ignore").read(); os.unlink(o2); n+=1
            b=analyze(ll)
            if not b: ok+=1
            else: reasons[max(b,key=b.get)]+=1
        if n: print(f"{g:24s} n={n:3d} CLEAN={ok:3d}  {reasons.most_common(3)}",flush=True)
        if ok: tot["clusters"]+=1
    print("clusters with >=1 clean:",tot["clusters"],"of",len(groups))
if __name__=="__main__":
    run(sys.argv[1].split(","),sys.argv[2],int(sys.argv[3]),sys.argv[4] if len(sys.argv)>4 else "true")
