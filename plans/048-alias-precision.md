# Plan 048: Alias Precision Improvements (E28)

## Overview

Improve alias query precision to pass more PTABen benchmark tests. Currently SAF uses a three-valued `AliasResult` (`May`, `No`, `Unknown`) while PTABen expects four-valued results (`MustAlias`, `PartialAlias`, `MayAlias`, `NoAlias`). This causes 128 test failures.

## Problem Analysis

### Current State
- `AliasResult` enum: `{May, No, Unknown}`
- `may_alias()` returns `May` when points-to sets overlap (even if identical)
- No detection of definite aliasing (MustAlias)
- No detection of partial aliasing (struct/field overlap)

### PTABen Failures (128 total)
| Pattern | Count | Cause |
|---------|-------|-------|
| MustAlias → Unknown | ~40 | Missing points-to info or empty sets |
| MustAlias → May | ~30 | No MustAlias detection (identical sets return May) |
| NoAlias → May | ~50 | Field sensitivity collapse, array index collapse |
| PartialAlias → Unknown/May | ~8 | No PartialAlias detection |

### SVF Reference Implementation
From [SVF CondPTAImpl](https://svf-tools.github.io/SVF-doxygen/html/classSVF_1_1CondPTAImpl.html):
- **MustAlias**: Bidirectional containment — `pts(p) == pts(q)` (sets are equal)
- **PartialAlias**: One location is a field/subobject of another
- **MayAlias**: Sets overlap but aren't equal
- **NoAlias**: Disjoint sets

## Design

### Phase 1: Add MustAlias to AliasResult

**File:** `crates/saf-analysis/src/pta/result.rs`

Extend the `AliasResult` enum:

```rust
pub enum AliasResult {
    Must,     // Identical singleton or equal sets (definite alias)
    Partial,  // One location is subfield of another
    May,      // Sets overlap but not identical
    No,       // Disjoint sets
    Unknown,  // One or both pointers unknown
}
```

Add helper methods:
- `must_alias()` — Returns true for `Must`
- `partial_alias()` — Returns true for `Must` or `Partial`
- Update `may_alias_conservative()` to include `Must` and `Partial`

**Scope:** ~50 lines

### Phase 2: MustAlias Detection Logic

**File:** `crates/saf-analysis/src/pta/result.rs`

Update `may_alias()` method:

```rust
pub fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
    let p_pts = self.pts.get(&p);
    let q_pts = self.pts.get(&q);

    match (p_pts, q_pts) {
        (None, _) | (_, None) => AliasResult::Unknown,
        (Some(p_set), Some(q_set)) => {
            if p_set.is_empty() || q_set.is_empty() {
                AliasResult::Unknown
            } else if p_set.is_disjoint(q_set) {
                AliasResult::No
            } else if p_set == q_set {
                // Bidirectional containment: sets are identical
                AliasResult::Must
            } else if self.is_partial_alias(p_set, q_set) {
                AliasResult::Partial
            } else {
                AliasResult::May
            }
        }
    }
}
```

Key insight: Set equality (`p_set == q_set`) implies bidirectional containment, which is SVF's MustAlias criterion.

**Scope:** ~30 lines

### Phase 3: PTABen Validator Update

**File:** `crates/saf-bench/src/ptaben.rs`

Update `check_alias_result()`:

```rust
fn check_alias_result(expected: AliasKind, actual: AliasResult) -> Outcome {
    let matches = match expected {
        AliasKind::MustAlias => actual == AliasResult::Must,
        AliasKind::PartialAlias => matches!(actual, AliasResult::Must | AliasResult::Partial),
        AliasKind::MayAlias => actual != AliasResult::No,
        AliasKind::NoAlias => actual == AliasResult::No,
        AliasKind::ExpectedFailMayAlias => actual == AliasResult::No,
        AliasKind::ExpectedFailNoAlias => actual != AliasResult::No,
    };
    // ...
}
```

**Scope:** ~15 lines

### Phase 4: Field-Based Partial Alias Detection

**File:** `crates/saf-analysis/src/pta/result.rs`

Add field path prefix detection for PartialAlias:

```rust
fn is_partial_alias(&self, p_set: &BTreeSet<LocId>, q_set: &BTreeSet<LocId>) -> bool {
    // Fast path: subset relationship
    let p_subset_q = p_set.is_subset(q_set) && p_set.len() < q_set.len();
    let q_subset_p = q_set.is_subset(p_set) && q_set.len() < p_set.len();

    if p_subset_q || q_subset_p {
        return true;
    }

    // Detailed: field path prefix relationship
    for &p_loc in p_set {
        for &q_loc in q_set {
            if self.locations_partially_overlap(p_loc, q_loc) {
                return true;
            }
        }
    }
    false
}

fn locations_partially_overlap(&self, p: LocId, q: LocId) -> bool {
    let (Some(p_loc), Some(q_loc)) = (self.locations.get(&p), self.locations.get(&q)) else {
        return false;
    };

    // Must be same base object
    if p_loc.obj != q_loc.obj {
        return false;
    }

    // Check if one path is a prefix of the other (but not equal)
    let p_path = &p_loc.path.steps;
    let q_path = &q_loc.path.steps;

    if p_path.len() == q_path.len() {
        return false;
    }

    let (shorter, longer) = if p_path.len() < q_path.len() {
        (p_path, q_path)
    } else {
        (q_path, p_path)
    };

    longer.starts_with(shorter)
}
```

**Scope:** ~50 lines

### Phase 5: Testing and Documentation

**Tests to add:**
1. `may_alias_must_alias_singleton` — Identical singleton sets → Must
2. `may_alias_must_alias_equal_sets` — Equal multi-element sets → Must
3. `may_alias_partial_field_prefix` — Struct vs field pointer → Partial
4. `may_alias_partial_nested_fields` — Nested field prefix → Partial
5. Update existing tests for new enum variants

**Documentation updates:**
- `docs/tool-comparison.md`: Mark alias precision as implemented
- `plans/FUTURE.md`: Update alias precision entry
- `plans/PROGRESS.md`: Add E28 epic

**Scope:** ~100 lines of tests, documentation updates

## Expected Outcomes

| Metric | Before | After | Notes |
|--------|--------|-------|-------|
| PTABen Pass Rate | 19% (122/652) | 35-40% | +50-80 tests |
| MustAlias Tests | ~0% pass | ~70% pass | Set equality detection |
| PartialAlias Tests | ~0% pass | ~60% pass | Field prefix detection |
| NoAlias Tests | ~40% pass | ~50% pass | Some improvement from precision |

## Known Limitations (Post-Implementation)

These will remain as documented gaps:
1. **Array index collapse**: All `a[i]` map to same location regardless of `i`
2. **Deep field paths**: Structures deeper than `max_depth` (default 2) collapse
3. **Flow-insensitivity**: CI-PTA doesn't track control-flow-dependent aliasing
4. **Context-insensitivity**: Base CI-PTA over-approximates across call sites

## Implementation Order

| Session | Phases | Deliverable |
|---------|--------|-------------|
| 1 | Phase 1-2 | Core `AliasResult` changes + MustAlias detection |
| 2 | Phase 3 | PTABen validator integration |
| 3 | Phase 4 | PartialAlias field path detection |
| 4 | Phase 5 | Testing validation + documentation |

## References

- [SVF CondPTAImpl](https://svf-tools.github.io/SVF-doxygen/html/classSVF_1_1CondPTAImpl.html) — MustAlias via bidirectional containment
- [SVF Test-Suite aliascheck.h](https://github.com/SVF-tools/Test-Suite/blob/master/aliascheck.h) — PTABen oracle definitions
- SAF `crates/saf-analysis/src/pta/result.rs` — Current AliasResult implementation
- SAF `crates/saf-bench/src/ptaben.rs` — PTABen validator
