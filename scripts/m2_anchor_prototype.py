#!/usr/bin/env python3
"""Movement 2 spike: the Anchored-Object `valid-memsafety` decision procedure.

Successor to `scripts/p211_anchor_prototype.py`, which is KEPT as the record of the
2026-09-12 measurement. That script's `analyze()` carries a *boolean* anchor ("is this
pointer derived from a global/alloca?"), which cannot express the byte offset obligation 3
needs, so this is a rewrite rather than an edit.

Anchor = {base: Global|Alloca, offset: const bytes}. A `Load` result NEVER anchors, so a
pointer that came out of memory can never carry a discharged obligation. Every obligation
is SYNTACTIC (SSA pointer graph + static layout), which is why thread interference --
which only changes values -- cannot invalidate one.

Obligations (plans/214 sec 1), all fail-closed:
  0. universe -- a defined `main`, no global ctors/dtors, reachable = main-tree U
                 thread-entry trees, no reachable indirect call, no unknown opcode.
  1. heap     -- no reachable malloc/calloc/realloc/free/alloca()/VLA.
  2. libc     -- every reachable external is memsafety-inert, or is a MODELLED external
                 whose pointer arguments are individually discharged.
  3. bounds   -- every reachable Load/Store/GEP/atomic/mem-intrinsic address is
                 `base + constant` with `0 <= off` and `off + width <= size_of(base)`.

Layout comes from the module's own `target datalayout`, so ILP32 vs LP64 is exact. Any
size that cannot be computed exactly => abstain.

IR recipe. plans/214 sec 2 prescribes `opt -passes=sroa,mem2reg,instcombine`. MEASURED
2026-09-16 over all 10,087 expected-FALSE valid-memsafety tasks, that recipe is WRONG for
this purpose: `sroa` and `instcombine` EXPLOIT the undefined behaviour the prover exists
to detect, deleting an out-of-bounds `memset` into a dead alloca outright and leaving IR
that is genuinely safe.

    survivors (would-be wrong TRUEs) by pipeline, obligations 2+3 on, no globals gate:
        sroa,mem2reg,instcombine   11
        sroa,mem2reg                8
        mem2reg                     2   <- SAF's own production pipeline
        (no opt)                    0

    concurrent-cluster yield is 10 of 16 under ALL FOUR, so the plan's premise that
    "without mem2reg the -O0 pointer spill defeats anchoring outright" does not hold for
    the concurrent clusters -- they anchor on globals, not on promoted stack pointers.

Default here: `clang-18 -S -emit-llvm -O0 -Xclang -disable-O0-optnone [-m32]` then
`opt-18 -S -passes=mem2reg`, with GLOBALS_ONLY on.
"""
from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile

# `llvm.lifetime.end` can kill a stack object while an SSA pointer to it is still live
# (use-after-scope). Rejected by default; set M2_ALLOW_LIFETIME=1 to measure the cost.
ALLOW_LIFETIME_END = os.environ.get("M2_ALLOW_LIFETIME") == "1"

# Restrict anchor BASES to globals. clang hoists every block-scoped C local to a
# function-entry `alloca` and, without lifetime intrinsics, the C scope is simply gone
# from the IR -- so `{ int y; p=&y; } *p=1;` is indistinguishable from an in-scope access
# and the prover would answer TRUE on a use-after-scope FALSE task. A global object is
# live for the whole program, so restricting the base kind removes the question entirely.
# This is a tightening of obligation 1's "first increment proves only heap-free programs"
# to "heap-free AND stack-anchor-free", not a new special case.
GLOBALS_ONLY = os.environ.get("M2_GLOBALS_ONLY", "1") == "1"

# ---------------------------------------------------------------------------
# lexing helpers
# ---------------------------------------------------------------------------

_OPEN = {"(": ")", "[": "]", "{": "}"}
_CLOSE = {v: k for k, v in _OPEN.items()}


def split_top(s: str, sep: str = ",") -> list[str]:
    """Split `s` on `sep` at nesting depth 0 (respects () [] {} and string literals)."""
    out, depth, cur, instr = [], 0, [], False
    i = 0
    while i < len(s):
        c = s[i]
        if instr:
            cur.append(c)
            if c == "\\" and i + 1 < len(s):
                cur.append(s[i + 1])
                i += 1
            elif c == '"':
                instr = False
            i += 1
            continue
        if c == '"':
            instr = True
            cur.append(c)
        elif c in _OPEN:
            depth += 1
            cur.append(c)
        elif c in _CLOSE:
            depth -= 1
            cur.append(c)
        elif c == sep and depth == 0:
            out.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
        i += 1
    out.append("".join(cur).strip())
    return out


def balanced(s: str, start: int) -> int:
    """Index just past the bracket group opened at `s[start]`, or -1."""
    depth, i, instr = 0, start, False
    while i < len(s):
        c = s[i]
        if instr:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                instr = False
        elif c == '"':
            instr = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return -1


def strip_meta(s: str) -> list[str]:
    """Top-level comma parts of `s`, minus trailing `!meta` / `align N` parts."""
    return [
        p for p in split_top(s)
        if p and not p.startswith("!") and not re.match(r"^align\s+\d+$", p)
    ]


_CONSTEXPR = re.compile(r"\b(getelementptr|bitcast|addrspacecast|inttoptr|ptrtoint)\b")
GEP_MODIFIER = re.compile(r"^(?:inbounds|nuw|nusw|inrange\([^)]*\))\s+")


def ptr_operand(part: str) -> str:
    """The pointer value inside `[ptr] [attrs] <value> [ordering]`.

    Keeps a constant expression whole so its byte offset is not silently dropped --
    returning the bare `@g` out of `getelementptr inbounds (T, ptr @g, i64 0, i32 3)`
    would claim offset 0 for a field at offset 8, which is a wrong-TRUE generator.
    """
    p = part.strip()
    m = _CONSTEXPR.search(p)
    if m:
        return p[m.start():]
    p = re.sub(r"^ptr\b\s*", "", p)
    for tok in p.split():
        if tok.startswith(("%", "@")) or tok in ("null", "undef", "poison"):
            return tok.rstrip(",")
    return ""


def scalar_operand(part: str) -> str:
    """The value token of a non-pointer `<ty> [attrs] <value>` argument."""
    toks = part.strip().split()
    return toks[-1] if toks else ""


def const_int(tok: str):
    tok = tok.strip()
    if re.fullmatch(r"-?\d+", tok):
        return int(tok)
    if tok == "true":
        return 1
    if tok in ("false", "null", "zeroinitializer"):
        return 0
    return None


# ---------------------------------------------------------------------------
# target datalayout -> exact sizes and alignments
# ---------------------------------------------------------------------------

class Layout:
    def __init__(self, dl: str, named: dict[str, str]):
        # LLVM DataLayout::reset() defaults, in BYTES (note i64 ABI align defaults to 4;
        # x86-64 overrides it with `i64:64`, i386 does not -- which is the C ABI).
        self.ptr_size, self.ptr_align = 8, 8
        self.ints = {1: 1, 8: 1, 16: 2, 32: 4, 64: 4}
        self.floats = {16: 2, 32: 4, 64: 8, 128: 16}
        self.named = named
        self._cache: dict[str, object] = {}
        for ent in dl.split("-"):
            f = ent.strip().split(":")
            head = f[0]
            try:
                if head in ("p", "p0"):
                    self.ptr_size = int(f[1]) // 8
                    self.ptr_align = int(f[2]) // 8
                elif head.startswith("p") and head[1:].isdigit():
                    pass
                elif re.fullmatch(r"i\d+", head):
                    self.ints[int(head[1:])] = max(1, int(f[1]) // 8)
                elif re.fullmatch(r"f\d+", head):
                    self.floats[int(head[1:])] = max(1, int(f[1]) // 8)
            except (IndexError, ValueError):
                continue

    def int_align(self, bits: int) -> int:
        ge = [k for k in self.ints if k >= bits]
        return self.ints[min(ge)] if ge else self.ints[max(self.ints)]

    # -- type parsing ------------------------------------------------------
    def parse_type(self, s: str, i: int = 0):
        while i < len(s) and s[i].isspace():
            i += 1
        if i >= len(s):
            return None, i
        c = s[i]
        if c == "%":
            j = i + 1
            if j < len(s) and s[j] == '"':
                k = s.find('"', j + 1)
                if k < 0:
                    return None, i
                return ("named", s[j + 1 : k]), k + 1
            while j < len(s) and (s[j].isalnum() or s[j] in "._$"):
                j += 1
            return ("named", s[i + 1 : j]), j
        if c == "[":
            e = balanced(s, i)
            if e < 0:
                return None, i
            m = re.match(r"\s*(\d+)\s+x\s+(.*)$", s[i + 1 : e - 1], re.S)
            if not m:
                return None, i
            el, _ = self.parse_type(m.group(2))
            return ((("array", int(m.group(1)), el)), e) if el else (None, i)
        if c == "<":
            e = s.find(">", i)
            if e < 0:
                return None, i
            inner = s[i + 1 :].lstrip()
            if inner.startswith("{"):                      # packed struct <{ ... }>
                b = i + 1 + (len(s[i + 1 :]) - len(inner))
                fs = self._struct_body(s, b)
                if fs is None:
                    return None, i
                e2 = balanced(s, b)
                return ("struct", True, fs), s.find(">", e2) + 1
            return None, i                                  # vector: abstain
        if c == "{":
            fs = self._struct_body(s, i)
            if fs is None:
                return None, i
            return ("struct", False, fs), balanced(s, i)
        m = re.match(r"i(\d+)\b", s[i:])
        if m:
            return ("int", int(m.group(1))), i + m.end()
        for kw, bits in (("x86_fp80", 80), ("ppc_fp128", 128), ("fp128", 128),
                         ("bfloat", 16), ("double", 64), ("float", 32), ("half", 16)):
            if s.startswith(kw, i):
                return ("float", bits), i + len(kw)
        if s.startswith("ptr", i):
            return ("ptr",), i + 3
        return None, i

    def _struct_body(self, s: str, i: int):
        e = balanced(s, i)
        if e < 0:
            return None
        inner = s[i + 1 : e - 1].strip()
        if not inner:
            return []
        fields = []
        for part in split_top(inner):
            t, _ = self.parse_type(part)
            if t is None:
                return None
            fields.append(t)
        return fields

    # -- sizes -------------------------------------------------------------
    def align(self, t):
        if t is None:
            return None
        k = t[0]
        if k == "int":
            return self.int_align(t[1])
        if k == "float":
            return self.floats.get(t[1])
        if k == "ptr":
            return self.ptr_align
        if k == "array":
            return self.align(t[2])
        if k == "struct":
            if t[1]:
                return 1
            best = 1
            for f in t[2]:
                a = self.align(f)
                if a is None:
                    return None
                best = max(best, a)
            return best
        if k == "named":
            return self.align(self.resolve(t[1]))
        return None

    def store_size(self, t):
        if t is None:
            return None
        k = t[0]
        if k == "int":
            return (t[1] + 7) // 8
        if k == "float":
            return 10 if t[1] == 80 else t[1] // 8
        if k == "ptr":
            return self.ptr_size
        if k == "array":
            es = self.alloc_size(t[2])
            return None if es is None else t[1] * es
        if k == "struct":
            lay = self.struct_layout(t)
            return None if lay is None else lay[1]
        if k == "named":
            return self.store_size(self.resolve(t[1]))
        return None

    def alloc_size(self, t):
        ss, a = self.store_size(t), self.align(t)
        if ss is None or a is None or a <= 0:
            return None
        return ((ss + a - 1) // a) * a

    def struct_layout(self, t):
        packed, fields = t[1], t[2]
        offs, off, salign = [], 0, 1
        for f in fields:
            fa = 1 if packed else self.align(f)
            fs = self.alloc_size(f)
            if fa is None or fs is None:
                return None
            off = ((off + fa - 1) // fa) * fa
            offs.append(off)
            off += fs
            salign = max(salign, fa)
        return offs, ((off + salign - 1) // salign) * salign

    def resolve(self, name: str):
        if name in self._cache:
            return self._cache[name]
        body = self.named.get(name)
        if body is None:
            return None
        self._cache[name] = None            # break cycles => abstain
        t, _ = self.parse_type(body)
        self._cache[name] = t
        return t

    def deref(self, t):
        seen = 0
        while t is not None and t[0] == "named" and seen < 32:
            t = self.resolve(t[1])
            seen += 1
        return t


# ---------------------------------------------------------------------------
# external-function policy (obligations 1 and 2)
# ---------------------------------------------------------------------------

INERT = {
    "abort", "exit", "_exit", "_Exit",
    "reach_error", "__VERIFIER_error", "__VERIFIER_assume",
    "__VERIFIER_atomic_begin", "__VERIFIER_atomic_end", "assume_abort_if_not",
    "pthread_self", "pthread_equal", "pthread_exit", "pthread_yield", "sched_yield",
    "thrd_current", "thrd_equal", "thrd_exit", "thrd_yield",
    "rand", "srand", "abs", "labs", "llabs", "getchar", "putchar", "sleep", "usleep",
    "fabs", "fabsf", "sqrt", "sqrtf", "floor", "ceil", "round", "fmod", "pow", "exp",
    "log", "sin", "cos", "tan", "atan", "atan2",
}
INERT_LLVM = (
    "llvm.lifetime.start", "llvm.dbg.", "llvm.assume", "llvm.trap", "llvm.debugtrap",
    "llvm.expect", "llvm.experimental.noalias.scope.decl", "llvm.is.constant",
    "llvm.ctlz", "llvm.cttz", "llvm.ctpop", "llvm.bswap", "llvm.abs", "llvm.smax",
    "llvm.smin", "llvm.umax", "llvm.umin", "llvm.fshl", "llvm.fshr", "llvm.fabs",
    "llvm.sqrt", "llvm.fmuladd", "llvm.sadd.with.overflow", "llvm.uadd.with.overflow",
    "llvm.ssub.with.overflow", "llvm.usub.with.overflow", "llvm.smul.with.overflow",
    "llvm.umul.with.overflow", "llvm.sadd.sat", "llvm.uadd.sat", "llvm.ssub.sat",
    "llvm.usub.sat", "llvm.donothing", "llvm.sideeffect",
)

HEAP = {
    "malloc", "calloc", "realloc", "reallocarray", "free", "strdup", "strndup",
    "memalign", "posix_memalign", "aligned_alloc", "valloc", "pvalloc", "alloca",
    "__builtin_alloca", "_Znwm", "_Znam", "_ZdlPv", "_ZdaPv", "kmalloc", "kzalloc",
    "kcalloc", "kfree", "vmalloc", "vfree", "ldv_malloc", "ldv_zalloc", "ldv_calloc",
    "ldv_xmalloc", "ldv_init_zalloc", "ldv_successful_malloc", "__kmalloc",
}

# glibc object sizes (LP64, ILP32). Used as a LOWER BOUND the anchored handle must
# satisfy -- defence in depth on top of the C type system.
SYNC_SIZE = {
    "pthread_mutex_t": (40, 24), "pthread_cond_t": (48, 48),
    "pthread_rwlock_t": (56, 32), "pthread_barrier_t": (32, 20),
    "pthread_spinlock_t": (4, 4), "sem_t": (32, 16),
    "pthread_attr_t": (56, 36), "pthread_mutexattr_t": (4, 4),
    "pthread_condattr_t": (4, 4), "pthread_rwlockattr_t": (8, 8),
    "pthread_barrierattr_t": (4, 4), "pthread_key_t": (4, 4),
    "pthread_once_t": (4, 4), "pthread_t": (8, 4), "voidp": (8, 4),
}

# Per-argument obligations. 'x' ignore, '<T>' sync handle, '?' suffix = null allowed,
# 'fn' direct reference to a defined function, 'str' constant NUL-terminated global
# string, 'fmt' printf format (constant, no %s/%n), 'file' a stdio stream, 'mem'
# buffer bounded by the constant 'len' argument.
MODEL: dict[str, tuple[str, ...]] = {
    "pthread_create": ("pthread_t", "pthread_attr_t?", "fn", "x"),
    "thrd_create": ("pthread_t", "fn", "x"),
    "pthread_join": ("x", "voidp?"),
    "thrd_join": ("x", "voidp?"),
    "pthread_mutex_lock": ("pthread_mutex_t",),
    "pthread_mutex_unlock": ("pthread_mutex_t",),
    "pthread_mutex_trylock": ("pthread_mutex_t",),
    "pthread_mutex_destroy": ("pthread_mutex_t",),
    "pthread_mutex_init": ("pthread_mutex_t", "pthread_mutexattr_t?"),
    "pthread_mutexattr_init": ("pthread_mutexattr_t",),
    "pthread_mutexattr_destroy": ("pthread_mutexattr_t",),
    "pthread_mutexattr_settype": ("pthread_mutexattr_t", "x"),
    "pthread_cond_init": ("pthread_cond_t", "pthread_condattr_t?"),
    "pthread_cond_destroy": ("pthread_cond_t",),
    "pthread_cond_signal": ("pthread_cond_t",),
    "pthread_cond_broadcast": ("pthread_cond_t",),
    "pthread_cond_wait": ("pthread_cond_t", "pthread_mutex_t"),
    "pthread_rwlock_init": ("pthread_rwlock_t", "pthread_rwlockattr_t?"),
    "pthread_rwlock_destroy": ("pthread_rwlock_t",),
    "pthread_rwlock_rdlock": ("pthread_rwlock_t",),
    "pthread_rwlock_wrlock": ("pthread_rwlock_t",),
    "pthread_rwlock_unlock": ("pthread_rwlock_t",),
    "pthread_rwlock_tryrdlock": ("pthread_rwlock_t",),
    "pthread_rwlock_trywrlock": ("pthread_rwlock_t",),
    "pthread_spin_init": ("pthread_spinlock_t", "x"),
    "pthread_spin_destroy": ("pthread_spinlock_t",),
    "pthread_spin_lock": ("pthread_spinlock_t",),
    "pthread_spin_trylock": ("pthread_spinlock_t",),
    "pthread_spin_unlock": ("pthread_spinlock_t",),
    "pthread_barrier_init": ("pthread_barrier_t", "pthread_barrierattr_t?", "x"),
    "pthread_barrier_destroy": ("pthread_barrier_t",),
    "pthread_barrier_wait": ("pthread_barrier_t",),
    "pthread_attr_init": ("pthread_attr_t",),
    "pthread_attr_destroy": ("pthread_attr_t",),
    "sem_init": ("sem_t", "x", "x"),
    "sem_destroy": ("sem_t",),
    "sem_wait": ("sem_t",),
    "sem_trywait": ("sem_t",),
    "sem_post": ("sem_t",),
    "__assert_fail": ("str", "str", "x", "str"),
    "__assert": ("str", "str", "x"),
    "__assert_perror_fail": ("x", "str", "x", "str"),
    "puts": ("str",),
    "perror": ("str",),
    "printf": ("fmt",),
    "fprintf": ("file", "fmt"),
    "fflush": ("file",),
    "memcpy": ("mem", "mem", "len"),
    "memmove": ("mem", "mem", "len"),
    "memset": ("mem", "x", "len"),
    "bzero": ("mem", "len"),
    "bcopy": ("mem", "mem", "len"),
}

# Everything NOT listed forces abstain -- notably strcpy/strncpy/strcat/sprintf/snprintf/
# scanf/sscanf/fgets/read/write/qsort/bsearch AND the read-only-looking strlen/strcmp/
# memcmp/strchr/atoi family. This is STRICTER than `race_true.rs::libc_model`, which
# admits all of those as ReadAll/WriteDst. The divergence is deliberate: race_true only
# needs to know WHICH memory a call touches, memsafety needs to know the touch is IN
# BOUNDS, and every str*/implicit-length function runs to a NUL whose position is not
# statically known. The heap family, which race_true admits (fresh memory is race-inert),
# is rejected here by obligation 1.


# ---------------------------------------------------------------------------
# module parsing
# ---------------------------------------------------------------------------

KNOWN_OPS = {
    "add", "sub", "mul", "udiv", "sdiv", "urem", "srem", "shl", "lshr", "ashr",
    "and", "or", "xor", "icmp", "fcmp", "fadd", "fsub", "fmul", "fdiv", "frem", "fneg",
    "zext", "sext", "trunc", "bitcast", "addrspacecast", "ptrtoint", "inttoptr",
    "fptosi", "fptoui", "sitofp", "uitofp", "fptrunc", "fpext",
    "select", "phi", "getelementptr", "alloca", "load", "store", "call", "freeze",
    "ret", "br", "switch", "unreachable", "atomicrmw", "cmpxchg", "fence",
    "extractvalue", "insertvalue",
}
# Absent => abstain: invoke, resume, indirectbr, callbr, va_arg, landingpad, catchswitch,
# catchpad, cleanuppad, extractelement, insertelement, shufflevector.

DEFINE_RE = re.compile(r"^define\b.*?@([\w.$]+|\"[^\"]*\")\s*\(")
DECLARE_RE = re.compile(r"^declare\b.*?@([\w.$]+|\"[^\"]*\")\s*\(")
NAMEDTY_RE = re.compile(r"^%([\w.$]+|\"[^\"]*\")\s*=\s*type\s+(.*)$")
GLOBAL_RE = re.compile(r"^@([\w.$]+|\"[^\"]*\")\s*=\s*(.*)$")
ASSIGN_RE = re.compile(r"^(%[\w.$]+|%\"[^\"]*\")\s*=\s*(.*)$")
LABEL_RE = re.compile(r"^[-\w.$]+:")

GLOBAL_KEYWORDS = {
    "private", "internal", "available_externally", "linkonce", "weak", "common",
    "appending", "extern_weak", "linkonce_odr", "weak_odr", "external", "dllimport",
    "dllexport", "default", "hidden", "protected", "local_unnamed_addr",
    "unnamed_addr", "thread_local", "dso_local", "dso_preemptable", "constant",
    "global", "externally_initialized",
}
MODIFIER_TOKENS = {
    "tail", "musttail", "notail", "fast", "nnan", "ninf", "nsz", "arcp", "contract",
    "afn", "reassoc",
}


def opcode_of(body: str) -> str:
    toks = body.split()
    i = 0
    while i < len(toks) and toks[i] in MODIFIER_TOKENS:
        i += 1
    return toks[i] if i < len(toks) else ""


class Module:
    def __init__(self, text: str):
        self.named: dict[str, str] = {}
        self.globals: dict[str, dict] = {}
        self.funcs: dict[str, list[str]] = {}
        self.declared: set[str] = set()
        self.has_ctors = False
        dl, cur, pending = "", None, None
        for raw in text.splitlines():
            line = raw.strip()
            if not line or line.startswith(";"):
                continue
            if pending is not None:                 # multi-line `switch ... [ ... ]`
                pending += " " + line
                if pending.count("[") <= pending.count("]"):
                    self.funcs[cur].append(pending)
                    pending = None
                continue
            if cur is not None:
                if line == "}":
                    cur = None
                elif line.count("[") > line.count("]"):
                    pending = line
                else:
                    self.funcs[cur].append(line)
                continue
            if line.startswith("target datalayout"):
                m = re.search(r'"([^"]*)"', line)
                if m:
                    dl = m.group(1)
                continue
            m = NAMEDTY_RE.match(line)
            if m:
                self.named[m.group(1).strip('"')] = m.group(2).strip()
                continue
            m = DEFINE_RE.match(line)
            if m:
                cur = m.group(1).strip('"')
                self.funcs.setdefault(cur, [])
                continue
            m = DECLARE_RE.match(line)
            if m:
                self.declared.add(m.group(1).strip('"'))
                continue
            m = GLOBAL_RE.match(line)
            if m:
                name = m.group(1).strip('"')
                if name in ("llvm.global_ctors", "llvm.global_dtors"):
                    self.has_ctors = True
                self.globals[name] = {"rest": m.group(2)}
                continue
        self.layout = Layout(dl, self.named)
        for g in self.globals.values():
            g["type"], g["init"], g["is_const"] = self._global_type(g["rest"])
            g["size"] = self.layout.alloc_size(g["type"]) if g["type"] else None

    def _global_type(self, rest: str):
        head = split_top(rest)[0]
        toks = head.split()
        i, is_const = 0, False
        while i < len(toks) and (
            toks[i] in GLOBAL_KEYWORDS
            or toks[i].startswith("addrspace(")
            or toks[i].startswith("thread_local(")
        ):
            if toks[i] == "constant":
                is_const = True
            i += 1
        tail = " ".join(toks[i:])
        t, n = self.layout.parse_type(tail)
        return t, tail[n:].strip(), is_const


# ---------------------------------------------------------------------------
# per-function anchor analysis
# ---------------------------------------------------------------------------

BOT = ("bot",)
TOP = ("top",)
NULLP = ("null",)


def join(a, b):
    if a == TOP:
        return b
    if b == TOP:
        return a
    if a == NULLP or b == NULLP:
        return a if a == b else BOT
    return a if a == b else BOT


class FuncAnalysis:
    def __init__(self, mod: Module, fname: str):
        self.m = mod
        self.L = mod.layout
        self.lines = mod.funcs[fname]
        self.defs: dict[str, str] = {}
        for line in self.lines:
            a = ASSIGN_RE.match(line)
            if a:
                self.defs[a.group(1)] = a.group(2).strip()
        self.sizes: dict[str, int] = {}
        self.bad_alloca = False
        self.nonconvergent = False
        self.anchor: dict[str, tuple] = {r: TOP for r in self.defs}
        self._seed_allocas()
        self._fixpoint()

    def base_size(self, base):
        kind, name = base
        if kind == "g":
            g = self.m.globals.get(name)
            return g["size"] if g else None
        return self.sizes.get(name)

    def _seed_allocas(self):
        for reg, body in self.defs.items():
            if opcode_of(body) != "alloca":
                continue
            parts = strip_meta(body[body.index("alloca") + 6:])
            if not parts:
                self.bad_alloca = True
                continue
            t, _ = self.L.parse_type(parts[0])
            es = self.L.alloc_size(t) if t else None
            count = 1
            if len(parts) > 1:
                c = const_int(scalar_operand(parts[1]))
                if c is None:
                    self.bad_alloca = True           # VLA / dynamic alloca
                    continue
                count = c
            if es is not None:
                self.sizes[reg] = es * count

    # -- lattice -----------------------------------------------------------
    def oper(self, tok: str) -> tuple:
        tok = tok.strip()
        if tok in ("null", "zeroinitializer"):
            return NULLP
        if tok.startswith("@"):
            name = tok[1:].strip('"')
            return (("g", name), 0) if name in self.m.globals else BOT
        if tok.startswith("%"):
            m = re.match(r"%[\w.$]+|%\"[^\"]*\"", tok)
            return self.anchor.get(m.group(0), BOT) if m else BOT
        if tok.startswith("getelementptr"):
            i = tok.find("(")
            e = balanced(tok, i) if i >= 0 else -1
            if e < 0:
                return BOT
            return self._gep_from(tok[i + 1: e - 1])
        if tok.startswith(("bitcast", "addrspacecast")):
            i = tok.find("(")
            e = balanced(tok, i) if i >= 0 else -1
            if e < 0:
                return BOT
            return self.oper(ptr_operand(split_top(tok[i + 1: e - 1])[0]))
        return BOT

    def _gep_from(self, argstr: str) -> tuple:
        parts = strip_meta(argstr)
        if len(parts) < 2:
            return BOT
        head = parts[0]
        while True:                     # `getelementptr inbounds i32, ptr %a, i64 3`
            stripped = GEP_MODIFIER.sub("", head)
            if stripped == head:
                break
            head = stripped
        srcty, _ = self.L.parse_type(head)
        if srcty is None:
            return BOT
        base = self.oper(ptr_operand(parts[1]))
        if base in (BOT, NULLP):
            return BOT
        if base == TOP:
            return TOP
        idxs = []
        for p in parts[2:]:
            v = const_int(scalar_operand(p))
            if v is None:
                return BOT
            idxs.append(v)
        if not idxs:
            return base
        es = self.L.alloc_size(srcty)
        if es is None:
            return BOT
        off = base[1] + idxs[0] * es
        cur = self.L.deref(srcty)
        for ix in idxs[1:]:
            if cur is None:
                return BOT
            if cur[0] == "array":
                s = self.L.alloc_size(cur[2])
                if s is None:
                    return BOT
                off += ix * s
                cur = self.L.deref(cur[2])
            elif cur[0] == "struct":
                lay = self.L.struct_layout(cur)
                if lay is None or ix < 0 or ix >= len(lay[0]):
                    return BOT
                off += lay[0][ix]
                cur = self.L.deref(cur[2][ix])
            else:
                return BOT
        return (base[0], off)

    def _eval(self, reg: str) -> tuple:
        body = self.defs[reg]
        op = opcode_of(body)
        if op == "alloca":
            return (("a", reg), 0)
        if op == "getelementptr":
            return self._gep_from(body[body.index("getelementptr") + 13:])
        if op in ("bitcast", "addrspacecast"):
            m = re.match(r"\s*\w+\s+(.*?)\s+to\b", body[body.index(op) + len(op):], re.S)
            return self.oper(ptr_operand(m.group(1))) if m else BOT
        if op == "freeze":
            return self.oper(ptr_operand(body[body.index("freeze") + 6:]))
        if op == "select":
            parts = strip_meta(body[body.index("select") + 6:])
            if len(parts) != 3:
                return BOT
            return join(self.oper(ptr_operand(parts[1])),
                        self.oper(ptr_operand(parts[2])))
        if op == "phi":
            acc = TOP
            for m in re.finditer(r"\[([^,\]]*),", body):
                acc = join(acc, self.oper(m.group(1).strip()))
                if acc == BOT:
                    break
            return acc
        return BOT                       # load / call / inttoptr / argument

    def _fixpoint(self):
        order = list(self.defs)
        for _ in range(256):
            changed = False
            for reg in order:
                cur = self.anchor[reg]
                if cur == BOT:
                    continue
                nxt = self._eval(reg)
                if nxt != cur:
                    self.anchor[reg] = nxt
                    changed = True
            if not changed:
                break
        else:
            self.nonconvergent = True
        for reg, v in self.anchor.items():
            if v == TOP:
                self.anchor[reg] = BOT


# ---------------------------------------------------------------------------
# constant strings / formats
# ---------------------------------------------------------------------------

def decode_cstr(init: str, size: int | None):
    """Exact initializer bytes of a byte-array global, or None if not decodable.

    `zeroinitializer` only yields bytes when the object's size is known, so a
    zero-initialised MUTABLE buffer can never masquerade as a 1-byte "" literal.
    """
    m = re.search(r'c"((?:[^"\\]|\\.)*)"', init)
    if not m:
        if init.strip() == "zeroinitializer" and size is not None:
            return b"\x00" * size
        return None
    s, out, i = m.group(1), bytearray(), 0
    while i < len(s):
        if s[i] == "\\" and i + 2 < len(s) + 1:
            h = s[i + 1: i + 3]
            if re.fullmatch(r"[0-9A-Fa-f]{2}", h):
                out.append(int(h, 16))
                i += 3
                continue
            if h[:1] == "\\":
                out.append(0x5C)
                i += 2
                continue
        out.append(ord(s[i]) & 0xFF)
        i += 1
    return bytes(out)


_FMT = re.compile(r"%[-+ #0']*[\d*]*(?:\.[\d*]*)?(?:hh|h|ll|l|L|j|z|t)?(.)")


def fmt_ok(b: bytes) -> bool:
    s = b.split(b"\x00")[0].decode("latin-1")
    i = 0
    while i < len(s):
        if s[i] != "%":
            i += 1
            continue
        if s[i:i + 2] == "%%":
            i += 2
            continue
        m = _FMT.match(s, i)
        if not m or m.group(1) in ("s", "S", "n", "["):
            return False
        i = m.end()
    return True


# ---------------------------------------------------------------------------
# the decision procedure
# ---------------------------------------------------------------------------

class Prover:
    def __init__(self, mod: Module, lp64: bool):
        self.m = mod
        self.L = mod.layout
        self.lp64 = lp64
        self.fa: dict[str, FuncAnalysis] = {}
        self.bad: list[str] = []

    def reject(self, why: str):
        if len(self.bad) < 6:
            self.bad.append(why)

    # -- universe ----------------------------------------------------------
    def reachable(self):
        if "main" not in self.m.funcs:
            self.reject("no-defined-main")
            return None
        if self.m.has_ctors:
            self.reject("global-ctors-dtors")
            return None
        seen, work = {"main"}, ["main"]
        for g in self.m.globals.values():
            for nm in re.findall(r"@([\w.$]+)", g["rest"]):
                if nm in self.m.funcs and nm not in seen:
                    seen.add(nm)
                    work.append(nm)
        while work:
            for line in self.m.funcs.get(work.pop(), []):
                for nm in re.findall(r"@([\w.$]+)", line):
                    if nm in self.m.funcs and nm not in seen:
                        seen.add(nm)
                        work.append(nm)
        return seen

    # -- pointer obligations ----------------------------------------------
    def need(self, fa: FuncAnalysis, tok: str, nbytes: int, optional: bool):
        """None when the obligation is discharged, else a short failure reason."""
        a = fa.oper(tok)
        if a == NULLP:
            return None if optional else "null-pointer"
        if a in (BOT, TOP):
            return "unanchored"
        base, off = a
        if GLOBALS_ONLY and base[0] == "a":
            return "stack-anchor"
        sz = fa.base_size(base)
        if sz is None:
            return "unknown-object-size"
        if off < 0 or off + nbytes > sz:
            return "out-of-bounds"
        return None

    def conststr(self, fa: FuncAnalysis, tok: str):
        """The NUL-terminated bytes a pointer argument reads, or None.

        Requires an IMMUTABLE (`constant`) global: a `global [5 x i8] zeroinitializer`
        is a mutable buffer whose NUL can be overwritten at run time, and accepting it
        would let `puts(buf)` read past the end.
        """
        a = fa.oper(tok)
        if a in (BOT, TOP, NULLP):
            return None
        base, off = a
        if base[0] != "g":
            return None
        g = self.m.globals.get(base[1])
        if not g or not g.get("is_const"):
            return None
        b = decode_cstr(g["init"], g["size"])
        if b is None or g["size"] is None or len(b) != g["size"]:
            return None
        if off < 0 or off >= len(b) or b"\x00" not in b[off:]:
            return None
        return b[off:]

    def is_stream(self, fa: FuncAnalysis, tok: str) -> bool:
        if not tok.startswith("%"):
            return bool(re.fullmatch(r"@(stdout|stderr|stdin)", tok))
        body = fa.defs.get(tok, "")
        return opcode_of(body) == "load" and bool(
            re.search(r"@(stdout|stderr|stdin)\b", body))

    # -- calls -------------------------------------------------------------
    def check_call(self, fa: FuncAnalysis, callee: str, argstr: str):
        name = callee[1:].strip('"')
        if "byval(" in argstr or "sret(" in argstr:
            self.reject("byval-or-sret-argument")
            return
        args = [a for a in split_top(argstr) if a.strip()]
        if name in self.m.funcs:
            return
        if name.startswith("llvm."):
            stem = ".".join(name.split(".")[:2])
            if stem in ("llvm.memcpy", "llvm.memmove", "llvm.memset"):
                self.check_memintr(fa, stem, args)
                return
            if name.startswith("llvm.lifetime.end"):
                if not ALLOW_LIFETIME_END:
                    self.reject("llvm.lifetime.end")
                return
            if name.startswith(INERT_LLVM):
                return
            self.reject(f"non-inert-external:{name}")
            return
        if name in HEAP:
            self.reject(f"heap:{name}")
            return
        if name.startswith("__VERIFIER_nondet_") or name in INERT:
            return
        spec = MODEL.get(name)
        if spec is None:
            self.reject(f"non-inert-external:{name}")
            return
        self.check_modelled(fa, name, spec, args)

    def check_memintr(self, fa: FuncAnalysis, stem: str, args: list[str]):
        ptrs = [a for a in args if a.strip().startswith("ptr")]
        lens = [a for a in args if re.match(r"^i(8|16|32|64)\b", a.strip())
                and not a.strip().startswith("i1 ")]
        if not ptrs or not lens:
            self.reject(f"{stem}:unparsed")
            return
        n = const_int(scalar_operand(lens[-1]))
        if n is None:
            self.reject(f"{stem}:non-constant-length")
            return
        for p in ptrs:
            r = self.need(fa, ptr_operand(p), n, optional=False)
            if r:
                self.reject(f"{stem}:{r}")
                return

    def check_modelled(self, fa, name, spec, args):
        nlen = None
        for s, a in zip(spec, args):
            if s == "len":
                nlen = const_int(scalar_operand(a))
        for idx, s in enumerate(spec):
            if idx >= len(args):
                if s.endswith("?") or s == "x":
                    continue
                self.reject(f"{name}:missing-arg{idx}")
                return
            a = args[idx]
            opt = s.endswith("?")
            s = s[:-1] if opt else s
            if s == "x":
                continue
            if s == "len":
                if nlen is None:
                    self.reject(f"{name}:non-constant-length")
                    return
                continue
            val = ptr_operand(a)
            if s == "mem":
                if nlen is None:
                    self.reject(f"{name}:non-constant-length")
                    return
                r = self.need(fa, val, nlen, optional=False)
                if r:
                    self.reject(f"{name}:{r}")
                    return
                continue
            if s == "fn":
                if val[1:].strip('"') not in self.m.funcs or not val.startswith("@"):
                    self.reject(f"{name}:thread-entry-not-a-defined-function")
                    return
                continue
            if s == "str":
                if val == "null" and opt:
                    continue
                if self.conststr(fa, val) is None:
                    self.reject(f"{name}:non-constant-string-arg{idx}")
                    return
                continue
            if s == "fmt":
                b = self.conststr(fa, val)
                if b is None or not fmt_ok(b):
                    self.reject(f"{name}:unmodelable-format")
                    return
                for extra in args[idx + 1:]:
                    if extra.strip().startswith("ptr"):
                        self.reject(f"{name}:pointer-vararg")
                        return
                continue
            if s == "file":
                if not self.is_stream(fa, val):
                    self.reject(f"{name}:unknown-stream")
                    return
                continue
            lo = SYNC_SIZE.get(s)
            if lo is None:
                self.reject(f"{name}:unmodelled-handle")
                return
            r = self.need(fa, val, lo[0] if self.lp64 else lo[1], optional=opt)
            if r:
                self.reject(f"{name}:handle-{r}:{s}")
                return

    # -- memory instructions ----------------------------------------------
    def check_mem(self, fa: FuncAnalysis, body: str, kind: str):
        rest = body[body.index(kind) + len(kind):]
        rest = re.sub(r"^(\s*(?:atomic|volatile)\b)+", " ", rest)
        parts = strip_meta(rest)
        if len(parts) < 2:
            self.reject(f"{kind}:unparsed")
            return
        t, _ = self.L.parse_type(parts[0])
        w = self.L.store_size(t) if t else None
        addr = ptr_operand(parts[1])
        if not addr or w is None:
            self.reject(f"{kind}:unparsed")
            return
        r = self.need(fa, addr, w, optional=False)
        if r:
            self.reject(f"{kind}-{r}")

    def check_atomic(self, fa: FuncAnalysis, body: str, op: str):
        parts = strip_meta(body[body.index(op) + len(op):])
        if len(parts) < 2:
            self.reject(f"{op}:unparsed")
            return
        addr = ptr_operand(parts[0])
        t, _ = self.L.parse_type(parts[1])
        w = self.L.store_size(t) if t else None
        if not addr or w is None:
            self.reject(f"{op}:unparsed")
            return
        r = self.need(fa, addr, w, optional=False)
        if r:
            self.reject(f"{op}-{r}")

    # -- driver ------------------------------------------------------------
    def check_func(self, fname: str):
        fa = self.fa.setdefault(fname, FuncAnalysis(self.m, fname))
        if fa.bad_alloca:
            self.reject("dynamic-alloca")
            return
        if fa.nonconvergent:
            self.reject("anchor-fixpoint-nonconvergent")
            return
        for line in fa.lines:
            if self.bad:
                return
            if LABEL_RE.match(line):
                continue
            a = ASSIGN_RE.match(line)
            body = a.group(2).strip() if a else line
            op = opcode_of(body)
            if op not in KNOWN_OPS:
                self.reject(f"unknown-opcode:{op or '?'}")
                continue
            if op == "inttoptr":
                self.reject("inttoptr")
            elif op == "call":
                callee, argstr = callee_and_args(body)
                if callee is None or not callee.startswith("@"):
                    self.reject("reachable-indirect-call")
                else:
                    self.check_call(fa, callee, argstr)
            elif op in ("load", "store"):
                self.check_mem(fa, body, op)
            elif op in ("atomicrmw", "cmpxchg"):
                self.check_atomic(fa, body, op)
            elif op == "getelementptr" and a:
                anc = fa.anchor.get(a.group(1), BOT)
                if anc not in (BOT, TOP, NULLP):
                    sz = fa.base_size(anc[0])
                    if sz is None or anc[1] < 0 or anc[1] > sz:
                        self.reject("gep-out-of-bounds")
                    elif GLOBALS_ONLY and anc[0][0] == "a":
                        self.reject("gep-stack-anchor")

    def run(self) -> list[str]:
        reach = self.reachable()
        if reach is None:
            return self.bad
        for f in sorted(reach):
            self.check_func(f)
            if self.bad:
                break
        return self.bad


def callee_and_args(body: str):
    """(callee token, raw argument string) for a call, or (None, '') if indirect."""
    m = re.search(r"\b(?:call|invoke)\b", body)
    if not m:
        return None, ""
    rest = body[m.end():]
    for i, ch in enumerate(rest):
        if ch != "(":
            continue
        pre = rest[:i].rstrip().split()
        if pre and pre[-1].startswith(("@", "%")):
            e = balanced(rest, i)
            return (pre[-1], rest[i + 1: e - 1]) if e > 0 else (None, "")
    return None, ""


def analyze(ll_text: str, lp64: bool = True) -> list[str]:
    """`[]` == PROVE (valid-memsafety holds); non-empty == abstain, with reasons."""
    try:
        return Prover(Module(ll_text), lp64).run()
    except Exception as exc:                        # fail-closed on any parser surprise
        return [f"analyzer-exception:{type(exc).__name__}"]


# ---------------------------------------------------------------------------
# compile + analyze one source
# ---------------------------------------------------------------------------

CLANG = os.environ.get("M2_CLANG", "clang-18")
OPT = os.environ.get("M2_OPT", "opt-18")
# plans/214's recipe is `sroa,mem2reg,instcombine`. MEASURED 2026-09-16: `sroa` and
# `instcombine` EXPLOIT the very undefined behaviour we are trying to detect -- they
# delete an out-of-bounds `memset` into a dead alloca outright, leaving IR that is
# genuinely safe, which turns an expected-FALSE task into a wrong TRUE. `mem2reg` alone
# promotes only `isAllocaPromotable` allocas (direct, type-matching load/store uses) and
# never removes a dereference, so it is the largest UB-neutral prefix of that recipe.
PASSES = os.environ.get("M2_PASSES", "mem2reg")


def compile_ll(src: str, data_model: str, timeout: int = 120):
    m32 = ["-m32"] if data_model.upper() == "ILP32" else []
    fd, t1 = tempfile.mkstemp(suffix=".ll")
    os.close(fd)
    t2 = t1 + ".o.ll"
    try:
        # The `-Wno-error=` pair is required for the CIL-preprocessed LDV Linux drivers:
        # clang 16+ made implicit declarations and int/pointer conversion hard errors, and
        # without them 84 expected-FALSE tasks fail to compile and go UNVERIFIED.
        r = subprocess.run(
            [CLANG, "-S", "-emit-llvm", "-O0", "-Xclang", "-disable-O0-optnone", "-w",
             "-Wno-error=int-conversion", "-Wno-error=implicit-function-declaration",
             "-Wno-error=incompatible-pointer-types", "-Wno-error=return-type"]
            + m32 + ["-o", t1, src],
            capture_output=True, text=True, timeout=timeout)
        if r.returncode != 0 or not os.path.exists(t1):
            return None
        if not PASSES:
            with open(t1, errors="ignore") as fh:
                return fh.read()
        r = subprocess.run(
            [OPT, "-S", f"-passes={PASSES}", "-o", t2, t1],
            capture_output=True, text=True, timeout=timeout)
        if r.returncode != 0 or not os.path.exists(t2):
            return None
        with open(t2, errors="ignore") as fh:
            return fh.read()
    except subprocess.TimeoutExpired:
        return None
    finally:
        for p in (t1, t2):
            try:
                os.unlink(p)
            except OSError:
                pass


def prove_source(src: str, data_model: str):
    ll = compile_ll(src, data_model)
    if ll is None:
        return False, "compile-failed"
    bad = analyze(ll, lp64=data_model.upper() != "ILP32")
    return (not bad), (bad[0] if bad else "PROVE")


def parse_yml(path: str):
    txt = open(path, errors="ignore").read()
    # `input_files:` appears quoted AND unquoted in the corpus (all 43
    # `coreutils-v9.5-units` tasks are unquoted). The inherited p211 regex only matched
    # the quoted form, silently skipping them -- which on the FALSE side means a task
    # that was never actually checked.
    inp = (re.search(r"input_files:\s*'([^']+)'", txt)
           or re.search(r'input_files:\s*"([^"]+)"', txt)
           or re.search(r"input_files:[ \t]*([^\s'\"\[\n][^\n]*)", txt))
    dm = re.search(r"data_model:\s*(\w+)", txt)
    props = [(os.path.basename(m.group(1)), m.group(2))
             for m in re.finditer(
                 r"property_file:\s*(\S+)\s*\n\s*expected_verdict:\s*(\w+)", txt)]
    return (inp.group(1) if inp else None, dm.group(1) if dm else "ILP32", props)


if __name__ == "__main__":
    for path in sys.argv[1:]:
        dm = "LP64"
        if path.endswith(".yml"):
            inp, dm, _ = parse_yml(path)
            path = os.path.join(os.path.dirname(path), inp)
        ok, why = prove_source(path, dm)
        print(f"{'PROVE  ' if ok else 'ABSTAIN'} {why:56s} {path}")
