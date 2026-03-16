# Plan 049: Array Index Sensitivity & Path-Sensitive Alias

**Epic:** E29 — Array Index Sensitivity & Path-Sensitive Alias
**Status:** approved
**Created:** 2026-01-31

## Goal

Improve PTABen pass rate by adding:
1. Symbolic array index tracking with Z3 refinement (~10 failures → passes)
2. Path-sensitive alias queries (~20 failures → passes)

**Target:** PTABen 18% → ~25-30% pass rate (30+ additional tests)

## Background

### Current Limitation: Array Index Collapse

SAF currently collapses all array indices to the same location:
- `a[0]` and `a[1]` map to identical `LocId`
- Causes false `MustAlias` when comparing pointers to different array elements
- Affects ~10 PTABen `basic_c_tests` failures

### Current Limitation: Path-Insensitive Alias

SAF's alias queries don't consider branch conditions:
```c
if (cond) { p = &a; q = &b; }  // NoAlias
else      { p = &a; q = &a; }  // MustAlias
MAYALIAS(p, q);  // Path-insensitive says MayAlias
```
- Affects ~20 PTABen `path_tests` failures

### SVF Comparison

SVF's approach:
- Default array-insensitive (all indices collapse)
- Optional `-locMM` for constant index sensitivity only
- **No Z3 integration** for symbolic index aliasing
- Variable indices → "VariantGepEdge" → field-insensitive collapse

Our approach is **more advanced than SVF**: symbolic index tracking with Z3 query-time refinement.

## Design

### 1. Symbolic Index Representation

**Current:**
```rust
pub enum PathStep {
    Field { index: u32 },
    Index,  // Collapsed
}
```

**Proposed:**
```rust
pub enum PathStep {
    Field { index: u32 },
    Index(IndexExpr),
}

pub enum IndexExpr {
    Unknown,           // Collapsed (legacy behavior, backward compatible)
    Constant(i64),     // Known constant index (a[0] vs a[1])
    Symbolic(ValueId), // SSA value (a[i] where i is variable)
}
```

### 2. GEP Constraint Extraction

Extend `GepConstraint` to capture index operands:
```rust
pub struct GepConstraint {
    pub dst: ValueId,
    pub src_ptr: ValueId,
    pub path: FieldPath,
    pub index_operands: Vec<Option<ValueId>>,  // One per Index step
}
```

Index resolution during constraint extraction:
- Check `AirModule.constants` for constant values → `IndexExpr::Constant(val)`
- Otherwise → `IndexExpr::Symbolic(operand_id)`

### 3. Location Factory Changes

`Location` equality respects `IndexExpr`:
- `Constant(0)` ≠ `Constant(1)` → different `LocId`s
- `Symbolic(v1)` ≠ `Symbolic(v2)` → different `LocId`s (conservative)
- `Unknown` = `Unknown` → same `LocId` (legacy)

### 4. Z3 Index Alias Refinement

Query-time Z3 check for symbolic index equality:
```rust
pub enum Z3IndexResult {
    Equal,      // Proven equal
    NotEqual,   // Proven not equal
    MayEqual,   // Unknown
    Timeout,    // Z3 exceeded budget
}

fn indices_may_equal(p: &IndexExpr, q: &IndexExpr, z3: &Z3Context, timeout_ms: u64) -> Z3IndexResult {
    match (p, q) {
        (IndexExpr::Unknown, _) | (_, IndexExpr::Unknown) => Z3IndexResult::MayEqual,
        (IndexExpr::Constant(a), IndexExpr::Constant(b)) => {
            if a == b { Z3IndexResult::Equal } else { Z3IndexResult::NotEqual }
        }
        (IndexExpr::Symbolic(v1), IndexExpr::Symbolic(v2)) if v1 == v2 => Z3IndexResult::Equal,
        _ => z3.check_may_equal_with_timeout(p, q, timeout_ms)
    }
}
```

**Timeout handling:** Conservative fallback to `MayAlias` on timeout/unknown.

### 5. Path-Sensitive Alias Queries

Reuse E19's dominator-based guard extraction:
```rust
fn may_alias_path_sensitive(&self, p: ValueId, q: ValueId, query_point: InstId) -> AliasResult {
    let guards = self.extract_dominating_guards(query_point);

    for path in self.enumerate_feasible_paths(&guards) {
        let p_pts = self.pts_under_path(p, &path);
        let q_pts = self.pts_under_path(q, &path);
        // Accumulate alias results per path
    }

    // Combine: No if none alias, Must if all alias, May otherwise
}
```

### 6. Configuration

```rust
pub struct PtaConfig {
    // Existing fields...

    // New: Array index sensitivity
    pub index_sensitivity: IndexSensitivity,  // None, Constant, Symbolic
    pub z3_index_enabled: bool,               // Enable Z3 refinement
    pub z3_index_timeout_ms: u64,             // Per-query timeout (default: 100ms)

    // New: Path-sensitive alias
    pub path_sensitive_alias: bool,           // Enable path-sensitive queries
    pub path_sensitive_timeout_ms: u64,       // Per-query timeout (default: 500ms)
    pub path_sensitive_max_paths: usize,      // Max paths to enumerate (default: 16)
}

pub enum IndexSensitivity {
    None,      // All indices collapse (legacy)
    Constant,  // Distinguish constant indices only
    Symbolic,  // Track symbolic indices, Z3 at query time
}
```

**Default for PTABen:** `IndexSensitivity::Symbolic`, Z3 enabled, 100ms timeout.

### 7. Python API

```python
class PtaResult:
    # Existing
    def may_alias(self, p: str, q: str) -> bool: ...
    def no_alias(self, p: str, q: str) -> bool: ...

    # New: Full five-valued result with Z3 refinement
    def alias_result(self, p: str, q: str) -> str:
        """Returns 'must', 'partial', 'may', 'no', or 'unknown'"""

    # New: Path-sensitive query at specific program point
    def may_alias_at(self, p: str, q: str, query_inst: str) -> str:
        """Path-sensitive alias query"""
```

### 8. PTABen Validator Updates

```rust
fn validate_alias_oracle(result: &PtaResult, oracle: &AliasOracle, z3_ctx: &Z3Context) -> TestResult {
    // Use Z3-refined query for all alias tests
    let alias = result.may_alias_z3(oracle.p, oracle.q, z3_ctx);

    // For path_tests: use path-sensitive query
    if oracle.category.contains("path") {
        alias = result.may_alias_path_sensitive(oracle.p, oracle.q, oracle.query_point, z3_ctx);
    }

    // Match expected vs actual
}
```

## Implementation Phases

### Phase 1: IndexExpr Type & PathStep Extension

**Scope:**
- Add `IndexExpr` enum to `saf-core` or `saf-analysis/pta`
- Extend `PathStep::Index` to include `IndexExpr`
- Update `Location` equality/hashing to respect `IndexExpr`
- Update `FieldPath` serialization for new index types
- Unit tests for new types

**Deliverable:** Core types compile, existing tests pass (backward compatible with `IndexExpr::Unknown`)

**Files:**
- `crates/saf-analysis/src/pta/location.rs`
- `crates/saf-analysis/src/pta/mod.rs` (if new module needed)

### Phase 2: GEP Constraint Extraction

**Scope:**
- Extend `GepConstraint` with `index_operands: Vec<Option<ValueId>>`
- Update `convert_field_path` to capture index operands from AIR
- Add index resolution helper using `AirModule.constants`
- Update `extract_constraints()` to populate index operands
- Unit tests for extraction

**Deliverable:** Constraints capture symbolic indices

**Files:**
- `crates/saf-analysis/src/pta/constraint.rs`
- `crates/saf-analysis/src/pta/extract.rs`

### Phase 3: Solver & LocationFactory Updates

**Scope:**
- Update `LocationFactory.get_or_create()` to handle `IndexExpr` variants
- Update solver GEP processing to resolve and propagate `IndexExpr`
- Ensure CI-PTA, CS-PTA, FS-PTA, DDA all handle new index types
- Add `IndexSensitivity` to `PtaConfig`
- Unit tests for solver with constant/symbolic indices

**Deliverable:** PTA produces distinct locations for different constant indices

**Files:**
- `crates/saf-analysis/src/pta/location.rs`
- `crates/saf-analysis/src/pta/solver.rs`
- `crates/saf-analysis/src/pta/cspta/solver.rs`
- `crates/saf-analysis/src/pta/fspta/solver.rs`
- `crates/saf-analysis/src/pta/dda/engine.rs`
- `crates/saf-analysis/src/pta/config.rs`

### Phase 4: Z3 Index Alias Refinement

**Scope:**
- Add `Z3IndexChecker` module with timeout-aware `indices_may_equal()`
- Add `PtaResult.may_alias_z3()` method
- Add configuration: `z3_index_enabled`, `z3_index_timeout_ms`
- Integrate with existing `z3_utils` module from E18-E19
- Unit tests with constant and symbolic index comparisons

**Deliverable:** Z3-refined alias queries working

**Files:**
- `crates/saf-analysis/src/pta/z3_index.rs` (new)
- `crates/saf-analysis/src/pta/result.rs`
- `crates/saf-analysis/src/pta/config.rs`

### Phase 5: Path-Sensitive Alias Queries

**Scope:**
- Add `PathSensitiveAliasChecker` reusing E19's `z3_utils` guard extraction
- Add `PtaResult.may_alias_path_sensitive()` method
- Add configuration: `path_sensitive_alias`, `max_paths`, timeout
- Timeout handling for path enumeration
- Unit tests for path-sensitive scenarios

**Deliverable:** Path-sensitive alias queries working

**Files:**
- `crates/saf-analysis/src/pta/path_sensitive.rs` (new)
- `crates/saf-analysis/src/pta/result.rs`
- `crates/saf-analysis/src/pta/config.rs`

### Phase 6: Python Bindings & PTABen Integration

**Scope:**
- Add Python methods: `alias_result()`, `may_alias_at()`
- Update PTABen validator to use Z3-refined queries
- Enable Z3 refinement by default for PTABen runs
- Run full PTABen suite, document pass rate improvement
- Add E2E tests for new Python methods

**Deliverable:** PTABen pass rate improvement measured and documented

**Files:**
- `crates/saf-python/src/pta.rs`
- `crates/saf-bench/src/ptaben/validator.rs`
- `python/tests/test_pta_*.py`

### Phase 7: Documentation & Cleanup

**Scope:**
- Update `docs/tool-comparison.md` — mark array index sensitivity as implemented
- Update `plans/FUTURE.md` — add tutorial entry, update status
- Update `plans/PROGRESS.md` — mark E29 complete
- Final test suite verification (`make test`)
- Code cleanup (remove any TODO comments, ensure clippy clean)

**Deliverable:** Documentation complete, all tests green

**Files:**
- `docs/tool-comparison.md`
- `plans/FUTURE.md`
- `plans/PROGRESS.md`

## Test Strategy

### Unit Tests (per phase)
- `IndexExpr` equality and hashing
- `FieldPath` with symbolic indices
- GEP constraint extraction with index operands
- Z3 index equality checker (constant vs constant, symbolic vs constant, timeout)
- Path-sensitive guard extraction

### E2E Tests
- C programs with constant array indices (`a[0]` vs `a[1]`)
- C programs with symbolic array indices (`a[i]` vs `a[j]`)
- C programs with path-dependent aliasing (if/else branches)
- Timeout handling tests (complex index expressions)

### PTABen Validation
- Run full PTABen suite with Z3 refinement enabled
- Compare pass rates: before (18%) vs after (target 25-30%)
- Document which test categories improved

## Success Criteria

1. **Constant index sensitivity working:** `a[0]` and `a[1]` produce different `LocId`s
2. **Symbolic index Z3 refinement working:** `a[i]` vs `a[j]` correctly uses Z3
3. **Timeout handling correct:** Z3 timeout → conservative `MayAlias`
4. **Path-sensitive queries working:** Branch-dependent alias correctly resolved
5. **PTABen improvement:** Pass rate increases from 18% to 25%+ (30+ new passes)
6. **All existing tests pass:** No regressions
7. **Clippy clean:** No new warnings

## References

- SVF GEP handling: `SVFIRBuilder::computeGepOffset()`
- SVF field sensitivity: `SymbolTableInfo`, `LocationSet`
- SAF E18-E19: Z3 integration, guard extraction, timeout handling
- SAF E28: Five-valued `AliasResult` (Must/Partial/May/No/Unknown)
- PTABen test categories: `basic_c_tests`, `path_tests`, `cs_tests`, `fs_tests`
