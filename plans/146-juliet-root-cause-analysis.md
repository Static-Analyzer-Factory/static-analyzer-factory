# Plan 146: Juliet Root Cause Analysis — Cross-CWE Investigation

## Status: done (research)
## Epic: checker-precision

## Executive Summary

Five investigation agents examined both Juliet test source code and SAF's analysis pipeline across all 15 CWE categories. The ~50% precision / 65.8% recall aggregate result stems from **7 root causes** that cluster into **3 fix groups** — each group would improve multiple CWE categories simultaneously with general-purpose analysis improvements (not Juliet-specific).

## Current Performance by Category

| Group | CWEs | Precision | Recall | Dominant Root Cause |
|-------|------|-----------|--------|-------------------|
| Temporal Safety | 122,124,126,127,401,415,416,690,761,789 | ~50% | ~100% | Path-insensitive SVFG |
| Buffer Overflow | 121 | 53.3% | 64.2% | Abstract domain + mem2reg |
| Integer Overflow | 190,191 | 0-11.8% | 0-0.1% | Interval widening + bitwise ops |
| NULL Deref | 476 | 50% | 9.8% | Missing NULL source classification |
| Free Non-Heap | 590 | 64.7% | 100% | Best — type-based, not path-based |

---

## Root Causes

### RC1: Path-Insensitive SVFG BFS (10 CWEs, ~50% precision)

**Affects:** CWE122, 124, 126, 127, 401, 415, 416, 690, 761, 789

**The problem:** The SVFG checker uses BFS (`solver.rs:48-129`) to find if any source (malloc) can reach any sink (free/use/deref) without passing through a sanitizer. This explores **all** successor edges equally — it cannot distinguish mutually exclusive branches.

**Why good variants look identical to bad:**
Juliet good variants use **conditional path diversification**, not syntactic fixes. For example, CWE415 (double-free):
- **Bad:** `malloc -> free -> call badSink() -> free again` (double free)
- **Good:** Two separate paths — `goodG2BSource()` does malloc+free, `goodB2GSource()` does malloc only — each called on different branches, so no pointer is freed twice

The SVFG merges both branches: it shows malloc reaching 2 different free calls, indistinguishable from the bad variant. The BFS reports a finding for both.

**Code location:** `crates/saf-analysis/src/checkers/solver.rs:48-129` (path-insensitive BFS), `pathsens_runner.rs:499-507` (cross-function temporal filter gives up)

```rust
// pathsens_runner.rs:499-503 — THE EXACT POINT WHERE ANALYSIS GIVES UP
if src_pp.function != sink_pp.function {
    return true;  // Keep finding conservatively
}
```

### RC2: Incomplete Guard Extraction for Z3 (All CWEs using Z3)

**Affects:** All CWEs that go through Z3 path feasibility checking

**The problem:** The Z3 path feasibility layer (`pathsens_runner.rs:188-200`) extracts branch conditions from SVFG traces and checks satisfiability. But guard extraction misses critical patterns:

| Pattern | Status | Impact |
|---------|--------|--------|
| Explicit `if (x < N)` comparisons | Extracted | Works |
| Constant conditions (`GLOBAL_CONST_FIVE == 5`) | Not resolved | Z3 treats as free variable |
| Complementary guards (`if(A)` / `if(!A)`) | Not recognized as mutually exclusive | Both paths appear feasible |
| Implicit control-flow guards | Not in def-use chain | Missed entirely |

**Code location:** `crates/saf-analysis/src/checkers/pathsens_runner.rs:188-200`

### RC3: Cross-Function Temporal Analysis Missing (4 CWEs)

**Affects:** CWE415, 416, 401, 761

**The problem:** The temporal filter that checks "does use happen after free?" only works within a single function. Cross-function findings (the majority in Juliet, where bad behavior occurs in called sink functions) are conservatively kept.

**Code location:** `crates/saf-analysis/src/checkers/pathsens_runner.rs:499-507`

### RC4: Abstract Interpretation Domain Limitations (7 CWEs)

**Affects:** CWE121-127 (buffer overflow), CWE190-191 (integer overflow)

Four sub-causes:

**RC4a: No field-sensitive struct tracking**
- Checker sees `malloc(32)` for a struct with a 16-byte `char[]` field
- Bad: `memcpy(field, src, 32)` — overflows the 16-byte field
- Good: `memcpy(field, src, 16)` — fits exactly
- Checker compares memcpy size against **total allocation** (32), not **field size** (16)
- Both variants produce `32 >= 32` -> WARNING
- **Code:** `crates/saf-analysis/src/absint/checker.rs` (memcpy overflow), `state.rs` (maps `ValueId -> Interval`, no field tracking)

**RC4b: Widening loses loop bounds**
- `for (i = 0; i < 100; i++) buf[i] = src[i]` — widening pushes `i` to `[0, inf)` after a few iterations
- Once interval is full range, `[INT_MIN, INT_MAX] * [INT_MIN, INT_MAX]` mathematically can't overflow (wraps to fit)
- Integer overflow checker sees no overflow and generates no finding
- **Code:** `crates/saf-analysis/src/absint/fixpoint.rs` (threshold widening), `interval.rs:218-254`

**RC4c: Bitwise operations not modeled**
- Juliet constructs input via `(rand() << 30) | (rand() << 15) | rand()`
- SAF sees only TOP or full range from intermediate `rand()` calls
- Can't reconstruct the actual bounded range from bitwise structure
- **Code:** `crates/saf-analysis/src/absint/transfer.rs` (no shift/or/and transfer functions)

**RC4d: Stack allocation sizes lost after mem2reg**
- `Alloca` instructions surviving mem2reg have `size_bytes` set
- Promoted locals: `size_bytes = None` -> fallback to `Interval::make_top(64)`
- Checker skips TOP operands -> no finding generated
- Explains CWE121's lower recall (64.2%) vs CWE122's 100% (heap malloc always has explicit size)
- **Code:** `crates/saf-analysis/src/absint/checker.rs:323` (skips TOP)

### RC5: Missing NULL Source Classification (CWE476)

**Affects:** CWE476 (9.8% recall)

**The problem:** SAF only recognizes function calls (malloc, calloc, fopen) as `NullSource` in `site_classifier.rs:200-330`. Explicit NULL assignments (`ptr = NULL`) compile to `Copy` or `Constant` operations in AIR — **never checked for NULL values**.

~30-40% of CWE476 tests use explicit `ptr = NULL` -> no source node -> no findings -> UNKNOWN verdict.

**Code location:** `crates/saf-analysis/src/checkers/site_classifier.rs:200-330`

### RC6: Missing Dereference Sink Patterns (CWE476)

**Affects:** CWE476

**The problem:** GEP instructions (`ptr->field`, member access) are not classified as dereference sinks. Many CWE476 tests dereference NULL via member access.

**Code location:** `crates/saf-analysis/src/checkers/site_classifier.rs`

### RC7: Conservative NULL Deref Verdict (CWE476)

**Affects:** CWE476

**The problem:** Even when null-deref findings are discovered, the verdict is UNKNOWN instead of FALSE:

```rust
// property.rs:810-858
if null_deref_count > 0 {
    return PropertyResult::Unknown {
        reason: "potential null-deref(s) from heap allocation without null check; \
                 cannot confirm malloc returns NULL"
    };
}
```

This is correct for malloc (may or may not return NULL), but wrong for explicit `ptr = NULL` (definitely NULL).

---

## Fix Groups (Common Fixes Across CWEs)

### Fix Group A: Path-Sensitive SVFG Analysis
**Addresses:** RC1 + RC2 + RC3
**Improves:** 11 CWEs (CWE122, 124, 126, 127, 401, 415, 416, 590, 690, 761, 789)
**Expected impact:** Precision from ~50% -> 70-85% on temporal safety CWEs

| Fix | Description | Complexity |
|-----|-------------|-----------|
| A1: Condition-labeled SVFG edges | Label each SVFG edge with the branch condition that enables it | High |
| A2: Guard-aware BFS | During BFS, accumulate path conditions and prune infeasible paths | Medium |
| A3: Constant propagation in guards | Resolve `GLOBAL_CONST_FIVE` to actual value before Z3 | Low |
| A4: Complementary guard recognition | Detect `if(C)` + `if(!C)` as mutually exclusive | Medium |
| A5: Interprocedural temporal ordering | Track program points across function calls via callgraph | High |
| A6: Function summary for sinks | Summarize callee behavior (does it free? does it deref?) | Medium |

**Recommended approach:** Start with A3 + A4 (low-hanging fruit for Z3 precision), then A6 (function summaries reduce cross-function blindness), then A1/A2 (architecture change).

### Fix Group B: Enhanced Abstract Domain
**Addresses:** RC4 (all sub-causes)
**Improves:** 7 CWEs (CWE121-127, CWE190-191)
**Expected impact:** Buffer overflow precision 50->65%+, integer overflow recall 0->30-50%

| Fix | Description | CWEs Helped | Complexity |
|-----|-------------|-------------|-----------|
| B1: Field-sensitive struct tracking | Track field offsets + sizes separately from total allocation | CWE121-127 | High |
| B2: Loop-aware widening | Detect bounded loops, preserve upper bound during widening | CWE121-127, 190-191 | Medium |
| B3: Bitwise transfer functions | Model `<<`, `\|`, `&` operations in interval domain | CWE190-191 | Medium |
| B4: mem2reg size preservation | Propagate `Alloca` size through register promotion | CWE121 | Low |

**Recommended approach:** B4 is a quick win (CWE121 recall). B2 has highest ROI across both buffer and integer overflow. B1 is the largest precision enabler for buffer overflow but requires wiring Plan 139's type system. B3 is specific to integer overflow patterns.

### Fix Group C: NULL Analysis Completeness
**Addresses:** RC5 + RC6 + RC7
**Improves:** CWE476 (currently 9.8% recall -> target 60-80%)

| Fix | Description | Impact | Complexity |
|-----|-------------|--------|-----------|
| C1: NULL constant as NullSource | Recognize `Copy`/`Constant` of NULL in site_classifier | +30-40% recall | Low |
| C2: GEP as dereference sink | Classify `ptr->field` (GEP) as potential NULL deref | +15-20% recall | Low |
| C3: Verdict distinction | Return FALSE for explicit NULL deref, keep UNKNOWN for malloc | +10-15% recall | Low |

**Recommended approach:** All three are low-complexity and independent — implement together.

---

## Priority Matrix

| Priority | Fix | CWEs Affected | Precision Delta | Recall Delta | Effort |
|----------|-----|---------------|-----------------|--------------|--------|
| **P0** | C1+C2+C3: NULL analysis | CWE476 | +40% | +50-70% | ~2 days |
| **P0** | A3+A4: Guard improvements | 11 CWEs | +5-10% | — | ~3 days |
| **P1** | B4: mem2reg size preservation | CWE121 | +5% | +15% | ~1 day |
| **P1** | A6: Function summaries | CWE415,416,401 | +10-15% | — | ~1 week |
| **P2** | B2: Loop-aware widening | CWE121-127, 190-191 | +10% | +20-30% | ~1-2 weeks |
| **P2** | B3: Bitwise transfer functions | CWE190-191 | — | +30-50% | ~1 week |
| **P3** | B1: Field-sensitive tracking | CWE121-127 | +15-20% | +10% | ~2-3 weeks |
| **P3** | A1+A2: Path-sensitive SVFG | 11 CWEs | +20-30% | — | ~3-4 weeks |

---

## Why CWE590 is the Exception (64.7% precision)

CWE590 (free of non-heap variable) is the **only type-based checker** — it asks "is this pointer from the heap?" rather than "is this sequence of operations temporal-safe?" PTA can answer heap-vs-stack as an intrinsic property, independent of execution paths. This confirms that **path-insensitivity is the fundamental bottleneck** — whenever SAF can answer a question without path sensitivity, precision is meaningfully higher.

---

## Key Insight

The ~50% precision across most CWEs is **not random** — it is the mathematical consequence of:
1. Both good and bad Juliet variants have structurally identical SVFG graphs
2. The only difference is which branch is taken at runtime
3. SAF's path-insensitive analysis merges all branches -> reports findings on both
4. Result: True positives (bad variants) ~ False positives (good variants) ~ 50%

Every improvement that adds **branch awareness** to the analysis — whether through path-sensitive SVFG, better Z3 guard extraction, or function summaries — directly attacks this fundamental bottleneck.
