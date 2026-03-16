# Plan 029: Abstract Interpretation Framework

**Epic:** E15 — Abstract Interpretation
**Status:** approved
**Created:** 2026-01-30

## Overview

Build an abstract interpretation framework for SAF: a generic `AbstractDomain` trait, an interval abstract domain (with widening/narrowing), a forward fixpoint iterator over CFG, and two concrete checkers — **buffer overflow** (CWE-120) and **integer overflow** (CWE-190). Python bindings let AI agents query computed invariants and compose declarative numeric checkers against the abstract state. Two tutorials demonstrate real-world usage.

This opens **Pillar 3** (Abstract Interpretation) — an entirely independent analysis methodology from the existing pointer/value-flow (Pillar 2) and IFDS (Pillar 1) approaches. It matches capabilities found in IKOS and Infer but absent from SVF and PhASAR.

## Research Summary

### Design Influences

- **IKOS (NASA)**: Pipeline architecture (LLVM → AR → fixpoint → checkers), 15 checkers, multiple numeric domains (intervals, congruences, octagons, DBM). `AbstractDomain` template with `join_with`/`meet_with`/`widen_with`/`narrow_with`/`leq`/`is_top`/`is_bottom`/`set_to_top`/`set_to_bottom`/`normalize`. Interleaved forward fixpoint iterator with widening at loop heads, followed by narrowing phase. Numerical Execution Engine interprets AR statements on abstract states.
- **SPARTA (Meta/Facebook)**: Rust crate with `AbstractDomain` trait, `Graph` trait for fixpoint iterator, Patricia-tree containers. Weak Partial Ordering for iteration strategy. Language-independent. 60% speedup vs conventional dataflow on ReDex.
- **Astrée**: Industrial abstract interpreter (Airbus). Threshold widening, trace partitioning, relational domains. Key insight: widening with thresholds from program constants dramatically improves precision.
- **Wrapped Intervals (Navas et al.)**: Interval domain that correctly handles modular arithmetic / integer wraparound. Fits naturally with LLVM IR's fixed-width integer semantics.

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| `AbstractDomain` trait in Rust | Follows IKOS/SPARTA pattern. Rust's trait system enforces lattice contract at compile time. Static dispatch for performance. |
| Interval domain with wrapped semantics | Fixed-width integer intervals (respecting bit-width) for soundness with LLVM IR's modular arithmetic. Covers signed and unsigned interpretations. |
| Widening with thresholds from program constants | Extract comparison constants from branch conditions; use as widening thresholds. Standard improvement over naive widening (Astrée, IKOS). Dramatically reduces precision loss at loops. |
| Forward worklist-based fixpoint iterator | Standard approach: iterate blocks in worklist order, apply transfer functions, widen at loop headers (detected via back edges in CFG), narrow in descending phase. Matches IKOS's `InterleavedFwdFixpointIterator`. |
| Abstract state = map from `ValueId` → `AbstractValue` | Each SSA value maps to an abstract element. Fits SAF's SSA-based AIR naturally. Memory accessed via Load/Store modeled conservatively (top for loads from unknown pointers, or precise when PTA available). |
| Checkers query invariants at specific program points | Buffer overflow checker: at GEP/Load/Store, check index ∈ [0, array_size). Integer overflow checker: at arithmetic ops, check result fits in bit-width. Both query the computed abstract state. |
| Python API: query invariants + declarative numeric checks | Agents can query `invariant_at(inst_id)` → dict of value→interval. Declarative checker specs (like E14) but for numeric properties. |
| No external dependencies (no Apron/ELINA) | Self-contained implementation following SAF's no-external-analysis-lib pattern. Interval domain is simple enough; more complex domains (octagons, polyhedra) can wrap Apron via FFI in future. |

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  Layer 4: Built-in Numeric Checkers                         │
│  buffer_overflow (CWE-120): index ∈ [0, size) at GEP/mem   │
│  integer_overflow (CWE-190): result fits bit-width at arith │
├──────────────────────────────────────────────────────────────┤
│  Layer 3: Checker Query Interface                           │
│  invariant_at(inst) → Map<ValueId, AbstractValue>           │
│  check_condition(inst, value, predicate) → Safe|Warning|Err │
├──────────────────────────────────────────────────────────────┤
│  Layer 2: Fixpoint Iterator                                 │
│  Forward worklist on CFG + transfer functions               │
│  Widening at loop headers (threshold-based)                 │
│  Narrowing phase (configurable iterations)                  │
├──────────────────────────────────────────────────────────────┤
│  Layer 1: Abstract Domain Layer                             │
│  AbstractDomain trait (join/meet/widen/narrow/leq/top/bot)  │
│  IntervalDomain: [lo, hi] per bit-width, wrapped semantics  │
│  AbstractState: Map<ValueId, AbstractValue>                 │
├──────────────────────────────────────────────────────────────┤
│  Foundation: AIR (saf-core) + CFG (saf-analysis)            │
│  Optional: PtaResult for memory modeling precision          │
└──────────────────────────────────────────────────────────────┘
```

## Abstract Domain Trait

```rust
/// Core lattice operations for abstract interpretation.
/// Follows IKOS/SPARTA design: in-place mutation + comparison.
pub trait AbstractDomain: Clone + Eq + std::fmt::Debug {
    /// The bottom element (unreachable / empty set).
    fn bottom() -> Self;

    /// The top element (no information / all values).
    fn top() -> Self;

    /// Check if this is the bottom element.
    fn is_bottom(&self) -> bool;

    /// Check if this is the top element.
    fn is_top(&self) -> bool;

    /// Partial order: self ⊑ other (self is less than or equal to other).
    fn leq(&self, other: &Self) -> bool;

    /// Least upper bound (union): self ⊔ other.
    fn join(&self, other: &Self) -> Self;

    /// Greatest lower bound (intersection): self ⊓ other.
    fn meet(&self, other: &Self) -> Self;

    /// Widening: over-approximate join for convergence.
    /// Default: same as join (correct for finite domains).
    fn widen(&self, other: &Self) -> Self { self.join(other) }

    /// Narrowing: refine after widening for precision.
    /// Default: same as meet (correct for any domain).
    fn narrow(&self, other: &Self) -> Self { self.meet(other) }
}
```

## Interval Domain

```rust
/// A closed interval [lo, hi] over fixed-width integers.
/// Respects bit-width for wraparound semantics.
pub struct Interval {
    lo: i128,        // Lower bound (or -∞ sentinel)
    hi: i128,        // Upper bound (or +∞ sentinel)
    bits: u8,        // Bit-width (8, 16, 32, 64)
    is_bottom: bool, // Empty interval flag
}
```

### Transfer Functions (Interval Arithmetic)

| AIR Operation | Interval Transfer |
|---|---|
| `BinaryOp::Add` | `[a+c, b+d]` with overflow check |
| `BinaryOp::Sub` | `[a-d, b-c]` with overflow check |
| `BinaryOp::Mul` | `[min(ac,ad,bc,bd), max(ac,ad,bc,bd)]` |
| `BinaryOp::SDiv`/`UDiv` | Standard interval division (handle zero) |
| `BinaryOp::Shl` | Shift bounds by shift amount interval |
| `BinaryOp::And`/`Or`/`Xor` | Conservative (bitwise ops on intervals are imprecise, use range bounds) |
| `BinaryOp::ICmpSlt` etc. | Refine operand intervals on true/false branches |
| `Cast::ZExt`/`SExt`/`Trunc` | Widen/narrow bit-width, adjust bounds |
| `Operation::Load` | Return top (or PTA-refined if available) |
| `Operation::Store` | Update memory model (conservative) |
| `Operation::Phi` | Join of incoming values |
| `Operation::Select` | Join of true/false values |
| `Constant::Int` | Singleton interval `[v, v]` |

### Widening with Thresholds

```
Standard widening:
  [a, b] ▽ [a', b'] = [a' < a ? -∞ : a, b' > b ? +∞ : b]

Threshold widening (T = set of constants from program):
  [a, b] ▽_T [a', b'] = [a' < a ? max{t ∈ T : t ≤ a'} : a,
                          b' > b ? min{t ∈ T : t ≥ b'} : b]

Threshold extraction: collect all integer constants from:
  - ICmp comparisons (branch conditions)
  - Array size declarations
  - Loop bounds
  - Function arguments to allocation functions
```

## Fixpoint Iterator

```
Algorithm: Forward Interleaved Widening/Narrowing

Input: CFG, entry block, transfer function T, widening thresholds
Output: Abstract state at each block entry (invariant map)

1. Initialize: state[entry] = initial_state; all others = bottom
2. Compute loop headers via back-edge detection (DFS on CFG)
3. Extract widening thresholds from branch conditions

ASCENDING PHASE (widening):
4. worklist = [entry]
5. while worklist not empty:
     b = worklist.pop()
     old = state[b]
     new = T(b, join of predecessor states)
     if b is loop header:
       new = old.widen_with_thresholds(new, thresholds)
     if new != old:
       state[b] = new
       worklist.extend(successors(b))

DESCENDING PHASE (narrowing, configurable iterations):
6. for i in 0..narrowing_iterations:
     changed = false
     for b in reverse_postorder:
       old = state[b]
       new = T(b, join of predecessor states)
       narrowed = old.narrow(new)
       if narrowed != old:
         state[b] = narrowed
         changed = true
     if !changed: break

7. Return state map
```

## Buffer Overflow Checker (CWE-120)

**Approach**: At each memory access (Load, Store, GEP with dynamic index), check that the computed index interval is within `[0, allocation_size)`.

```
For each instruction in module:
  if inst is GEP with dynamic index:
    idx_interval = invariant_at(inst).get(index_value)
    size = allocation_size_of(base_pointer)  // from PTA or type info
    if idx_interval.lo < 0 || idx_interval.hi >= size:
      report BufferOverflow(inst, idx_interval, size)
    if idx_interval ⊆ [0, size):
      report Safe
    else:
      report Warning (possible overflow)

  if inst is Memcpy/Memset:
    len_interval = invariant_at(inst).get(length_value)
    dest_size = allocation_size_of(dest_pointer)
    if len_interval.hi > dest_size:
      report BufferOverflow
```

**Allocation size tracking**: Track sizes from `malloc(n)`, `calloc(n, size)`, `alloca`, array declarations. Map `ValueId` → allocation size interval.

## Integer Overflow Checker (CWE-190)

**Approach**: At each arithmetic operation, check that the mathematical result fits within the target bit-width.

```
For each BinaryOp(Add|Sub|Mul) instruction:
  lhs_interval = invariant_at(inst).get(lhs)
  rhs_interval = invariant_at(inst).get(rhs)
  result_interval = compute_unwrapped(op, lhs_interval, rhs_interval)
  target_range = range_of(result_bits)  // e.g., [-2^31, 2^31-1] for i32

  if result_interval ⊆ target_range:
    report Safe (no overflow possible)
  if result_interval ∩ target_range = ∅:
    report Error (always overflows — likely dead code or definite bug)
  else:
    report Warning (may overflow)
```

**Signed vs unsigned**: Use the signedness from the AIR operation kind (SDiv vs UDiv, ICmpSlt vs ICmpUlt) to determine the valid range.

## Configuration

```rust
pub struct AbstractInterpConfig {
    /// Maximum widening iterations before forcing convergence.
    pub max_widening_iterations: u32,  // default: 100
    /// Number of narrowing iterations after ascending phase.
    pub narrowing_iterations: u32,     // default: 3
    /// Whether to extract widening thresholds from program constants.
    pub use_threshold_widening: bool,  // default: true
    /// Whether to use PTA results for memory modeling.
    pub use_pta: bool,                 // default: false (conservative)
    /// Maximum number of blocks to process (scalability bound).
    pub max_blocks: u64,               // default: 100_000
}
```

## Python API

```python
# High-level: run analysis and get invariants
project = saf.Project.open("program.ll")
result = project.abstract_interp()  # returns AbstractInterpResult

# Query invariants at specific instructions
inv = result.invariant_at(inst_id)   # → dict[str, Interval]
# e.g. {"value_0x1234": Interval(lo=0, hi=9), "value_0x5678": Interval.top()}

# Query invariants at block entry
block_inv = result.invariant_at_block(block_id)  # → dict[str, Interval]

# Run built-in numeric checkers
findings = project.check_numeric("buffer_overflow")   # → list[NumericFinding]
findings = project.check_numeric("integer_overflow")
all_findings = project.check_all_numeric()

# Custom numeric check (declarative)
custom = project.check_numeric_custom({
    "name": "array_bounds",
    "cwe": 129,
    "description": "Array index out of bounds",
    "check_at": "gep_index",  # or "arithmetic", "memory_access"
    "condition": "in_range",   # value must be in [0, size)
    "severity": "error",
})

# Inspect analysis diagnostics
diag = result.diagnostics()
# {"blocks_analyzed": 42, "widening_applications": 5,
#  "narrowing_iterations": 3, "converged": True, ...}

# Export results
result.export()  # JSON export of all invariants
```

## Implementation Phases

### Phase 1: AbstractDomain Trait + Interval Domain

**Files:**
- `crates/saf-analysis/src/absint/mod.rs` — module root, re-exports
- `crates/saf-analysis/src/absint/domain.rs` — `AbstractDomain` trait
- `crates/saf-analysis/src/absint/interval.rs` — `Interval` type + arithmetic

**Tests:**
- Unit tests for `Interval`: lattice laws (join/meet associative, commutative, idempotent), widening monotonicity, narrowing ≤ identity, arithmetic correctness (add, sub, mul, div overflow cases)
- Property tests: `∀ a, b: a.join(b).leq(a) == false || a.join(b) == a` etc.
- Edge cases: bottom/top, singleton intervals, max-width intervals, wraparound at i8/i16/i32/i64 boundaries

### Phase 2: Abstract State + Transfer Functions

**Files:**
- `crates/saf-analysis/src/absint/state.rs` — `AbstractState` (ValueId → Interval map)
- `crates/saf-analysis/src/absint/transfer.rs` — Transfer functions for all AIR operations

**Tests:**
- Unit tests for each transfer function: Add, Sub, Mul, Div, Rem, Shift, Cast, Phi, Select, Load, Store, GEP
- Branch condition refinement: ICmpSlt/Sle/Sgt/Sge/Eq/Ne narrow operand intervals on true/false edges
- Constant propagation: `Int{value: 42, bits: 32}` → `[42, 42]`

### Phase 3: Fixpoint Iterator

**Files:**
- `crates/saf-analysis/src/absint/config.rs` — `AbstractInterpConfig`
- `crates/saf-analysis/src/absint/threshold.rs` — Threshold extraction from branch conditions
- `crates/saf-analysis/src/absint/fixpoint.rs` — Forward worklist iterator with widening/narrowing
- `crates/saf-analysis/src/absint/result.rs` — `AbstractInterpResult` with query API

**Tests:**
- Straight-line code: constants propagated exactly
- Simple loop: widening produces sound over-approximation, narrowing refines
- Nested loops: correct widening at both loop headers
- Branch conditions: intervals refined on true/false edges
- Convergence: fixpoint reached within max iterations
- Determinism: identical results for identical inputs

### Phase 4: Buffer Overflow Checker

**Files:**
- `crates/saf-analysis/src/absint/checker.rs` — `NumericChecker` trait + `BufferOverflowChecker`
- `crates/saf-analysis/src/absint/alloc_tracker.rs` — Allocation size tracking from malloc/calloc/alloca

**Tests:**
- Unit tests for allocation size extraction
- Safe access: `a[i]` where `i ∈ [0, 9]` and `a` has 10 elements → Safe
- Definite overflow: `a[10]` → Error
- Possible overflow: `a[i]` where `i ∈ [0, 15]` and `a` has 10 elements → Warning

### Phase 5: Integer Overflow Checker

**Files:**
- `crates/saf-analysis/src/absint/checker.rs` — `IntegerOverflowChecker` (same file as Phase 4)

**Tests:**
- Safe addition: `i32 [0, 100] + [0, 100]` → `[0, 200]` fits i32 → Safe
- Definite overflow: `i8 [127, 127] + [1, 1]` → overflows i8 → Error
- Possible overflow: `i32 [0, 2^30] + [0, 2^30]` → may overflow → Warning
- Unsigned: `u32 [0, UINT_MAX-1] + [0, 1]` → safe vs `+ [2, 2]` → overflow

### Phase 6: E2E Tests (C/C++/Rust Source → Findings)

**Source programs** (compiled to LLVM IR, tested end-to-end):

1. `absint_buffer_overflow.c` — Array access in a loop with bounds check; one safe access, one overflow
2. `absint_integer_overflow.c` — Arithmetic on user input with/without overflow guards
3. `absint_loop_bounds.c` — Loop counter analysis, array iteration within bounds
4. `absint_nested_loops.c` — Nested loop with matrix access, tests widening precision
5. `absint_cpp_vector.cpp` — C++ vector-like struct with size/capacity, manual bounds checking
6. `absint_rust_unsafe.rs` — Rust unsafe block with raw pointer arithmetic and bounds

**Rust E2E tests:** ~12 tests covering build, invariant queries, checker findings, determinism, export
**Python E2E tests:** ~15 tests covering all API surfaces

### Phase 7: Python Bindings

**Files:**
- `crates/saf-python/src/absint.rs` — `PyAbstractInterpResult`, `PyInterval`, `PyNumericFinding`
- Update `crates/saf-python/src/project.rs` — `project.abstract_interp()`, `project.check_numeric()`
- Update `crates/saf-python/src/lib.rs` — register module

**Tests:** Python E2E tests from Phase 6

### Phase 8: Tutorials

**Tutorial 1:** `tutorials/checkers/03-buffer-overflow/`
- Real C program: HTTP request parser that reads a URL path into a fixed-size buffer using a loop. The loop has an off-by-one error allowing a 1-byte overflow.
- `detect.py`: Compiles source, runs abstract interpretation, checks for buffer overflow findings
- `detect.rs`: Same pipeline in Rust
- `README.md`: Explains abstract interpretation concepts, interval domain, how widening/narrowing work, why this catches bugs that taint analysis misses

**Tutorial 2:** `tutorials/checkers/04-integer-overflow/`
- Real C program: Image dimension calculator that multiplies user-supplied width × height for buffer allocation. Integer overflow causes undersized allocation.
- `detect.py` / `detect.rs`: Compile, analyze, detect integer overflow at multiplication
- `README.md`: Explains CWE-190, how interval analysis tracks value ranges through arithmetic

### Phase 9: Documentation Updates

- Update `docs/tool-comparison.md`: Mark abstract interpretation, numeric domains, buffer overflow checker, integer overflow checker as implemented
- Update `plans/FUTURE.md`: Add extension points for more domains (congruence, octagons, polyhedra), more checkers (division-by-zero, shift-count), interprocedural abstract interpretation
- Update `plans/PROGRESS.md`: E15 status, session log

## Test Strategy

### Unit Tests (Phases 1-5)
- **Lattice law tests**: Join/meet commutativity, associativity, idempotency, absorption. Widening monotonicity (`a ⊑ a ▽ b`). Narrowing refinement (`a △ b ⊑ a`).
- **Interval arithmetic tests**: All BinaryOp variants with concrete interval inputs, verifying output intervals contain all concrete results.
- **Property tests (proptest)**: For any random intervals a, b: `a.join(b) == b.join(a)`, `a.leq(a.join(b))`, `a.meet(b).leq(a)`, `a.widen(b).leq(a) == false || a.widen(b) == a.join(b)`.
- **Transfer function tests**: One test per AIR operation kind with known input→output intervals.

### E2E Tests (Phase 6)
- Compile real C/C++/Rust → LLVM IR → SAF analysis → check findings match expected
- Test programs designed around real vulnerability patterns (not artificial)
- Both Rust and Python API validation

### Integration Tests
- Determinism: same input → byte-identical output
- JSON export round-trip
- Python binding completeness (every Rust API has Python equivalent)

## Future Extensions (tracked in FUTURE.md)

- **Congruence domain**: `x ≡ c (mod m)` — combines with intervals for alignment analysis
- **Interval-congruence reduced product**: More precise than either alone
- **Octagon domain**: `±x ± y ≤ c` — relational constraints between variables
- **Polyhedra domain**: Full linear constraints — maximum precision, expensive
- **Division-by-zero checker**: Check divisor interval excludes 0
- **Shift-count checker**: Check shift amount ∈ [0, bit_width)
- **Interprocedural abstract interpretation**: Function summaries, bottom-up on call graph
- **Widening strategies**: Delayed widening, widening with landmarks
- **Memory abstract domain**: Track array contents abstractly (not just scalar values)
