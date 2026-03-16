# Plan 093: Correlated-Phi Path Sensitivity via Precise ICmp Transfer Functions

## Context

Of 80 remaining PTABen unsound cases, ae_assert (9) and ae_assert_fail (35) account for 44. The investigation (`docs/debug/investigation-synthesis.md`) identified ~22 as "correlated-phi" failures — short-circuit AND/OR patterns where a phi has a constant-false incoming and a proven-condition incoming, but the condition prover can't eliminate the infeasible false path.

**Root cause chain:**
1. `transfer.rs:745-753` — 9 of 10 ICmp variants return `Interval::new(0, 1, 1)` (always TOP). Only `ICmpSlt` calls `icmp_slt()` for a precise i1 result.
2. Because guards evaluate to `[0, 1]` (TOP), the condition prover's i1 interval lookup can never prove a guard is always true.
3. `evaluate_phi_condition_with_cfg` (condition_prover.rs:915-1091) cannot prove the false-path is unreachable, so the phi reports MayFail.

**Expected outcome:** 5-8 unsound cases fixed (ae_assert 9→4-7, ae_assert_fail 35→33-35).

## Agent Team Structure

Two parallel agents work on separate files, then main agent wires them together and verifies.

```
Agent 1 (interval.rs)  ──┐
                          ├──> Main agent: transfer.rs wiring + fmt/lint/test + PTABen
Agent 2 (condition_prover.rs)─┘
```

---

## Agent 1: ICmp Interval Methods

**File:** `crates/saf-analysis/src/absint/interval.rs` (ONLY this file)

**Task:** Add 9 `icmp_*` methods to the `Interval` struct, plus unit tests. These methods compute precise i1 (1-bit boolean) intervals for comparison results.

**Template — existing `icmp_slt` method (lines 627-638):**
```rust
pub fn icmp_slt(&self, other: &Self) -> Self {
    if self.bottom || other.bottom {
        return Self::make_bottom(1);
    }
    if self.hi < other.lo {
        Self::singleton(1, 1) // always true
    } else if self.lo >= other.hi {
        Self::singleton(0, 1) // always false
    } else {
        Self::new(0, 1, 1) // may be either
    }
}
```

**Insert after line 638, before the `refine_slt_true` method at line 640.**

**Methods to implement (each ~12 lines):**

| Method | Always True | Always False |
|--------|------------|-------------|
| `icmp_sle` | `self.hi <= other.lo` | `self.lo > other.hi` |
| `icmp_sgt` | `self.lo > other.hi` | `self.hi <= other.lo` |
| `icmp_sge` | `self.lo >= other.hi` | `self.hi < other.lo` |
| `icmp_eq` | both singletons AND `self.lo == other.lo` | `self.hi < other.lo \|\| other.hi < self.lo` |
| `icmp_ne` | `self.hi < other.lo \|\| other.hi < self.lo` | both singletons AND `self.lo == other.lo` |
| `icmp_ult` | if both `lo >= 0`: same as slt; else `[0,1]` |
| `icmp_ule` | if both `lo >= 0`: same as sle; else `[0,1]` |
| `icmp_ugt` | if both `lo >= 0`: same as sgt; else `[0,1]` |
| `icmp_uge` | if both `lo >= 0`: same as sge; else `[0,1]` |

**Doc comments:** Follow the pattern of `icmp_slt` — add `/// Compare self <op> other, returning a 1-bit interval.` and `#[must_use]`.

**Unit tests:** Add in the existing `#[cfg(test)] mod tests` section (starts ~line 870). Per method, add 3-4 tests:
- `_always_true`: disjoint intervals where result is guaranteed
- `_always_false`: disjoint intervals in opposite direction
- `_unknown`: overlapping intervals → `[0, 1]`
- `_bottom`: bottom input → bottom output

Example test names: `icmp_sge_always_true`, `icmp_eq_singletons_equal`, `icmp_ne_disjoint`, `icmp_ult_both_positive`, etc.

**Boundary condition cross-check:** The always-true/always-false conditions MUST match `evaluate_comparison` in `condition_prover.rs:1260-1335`. For reference:
- sge: always_true = `signed_ge(lhs.lo(), rhs.hi())` → `self.lo >= other.hi`
- sgt: always_true = `signed_gt(lhs.lo(), rhs.hi())` → `self.lo > other.hi`
- eq: always_true = `lhs.lo() == lhs.hi() && rhs.lo() == rhs.hi() && lhs.lo() == rhs.lo()`
- ne: always_false = same as eq always_true

---

## Agent 2: Enhanced Phi Guard Evaluation

**File:** `crates/saf-analysis/src/z3_utils/condition_prover.rs` (ONLY this file)

**Task:** Improve the phi short-circuit AND/OR guard evaluation to leverage precise i1 intervals from the fixpoint, and replace fragile string matching with semantic checks.

**Sub-task 2a: Extend `BlockTerminator` (lines 282-292)**

Add `cond_inst_id: Option<InstId>` field. This stores the CondBr instruction's ID for interval lookups at the branch point.

Current struct:
```rust
struct BlockTerminator {
    condition: Option<ValueId>,
    #[allow(dead_code)]
    then_target: Option<BlockId>,
    else_target: Option<BlockId>,
}
```

Add `cond_inst_id: Option<InstId>`. Update `build_block_terminator_map` (lines 295-335) to populate it — set `cond_inst_id: Some(inst.id)` in the `CondBr` arm (line 306-314), `cond_inst_id: None` in the other arm (line 320-326).

**Note:** Will need `use saf_core::ids::InstId;` if not already imported — check existing imports at top of file.

**Sub-task 2b: i1 Interval Fast Path (within lines 990-1028)**

In `evaluate_phi_condition_with_cfg`, inside the loop `for (false_pred_block, _, _, _) in &false_paths` (line 990), BEFORE the existing recursive guard evaluation (line 1001), add a fast path:

```rust
// Fast path: check the fixpoint's i1 interval for the guard condition
if let Some(cond_inst) = terminator.cond_inst_id {
    let guard_iv = best_interval(interval_source, cond_inst, cond_value);
    if guard_iv.lo() >= 1 {
        // Guard always true → else branch (false path) unreachable
        continue;
    }
}
```

The `best_interval` function (line 1168) already handles multi-width lookup. After the fixpoint computes precise i1 for all ICmp kinds, this fast path will find `[1, 1]` for provably-true guards.

**Sub-task 2c: Semantic False Path Detection (lines 964-976)**

Replace the string-based `desc.contains("constant false")` pattern matching with a semantic check using the constant map. Current code:

```rust
let false_paths: Vec<_> = path_results
    .iter()
    .filter(|(_, _, status, desc)| {
        *status == ConditionStatus::MayFail && desc.contains("constant false")
    })
    .collect();
```

Replace with:
```rust
let false_paths: Vec<_> = path_results
    .iter()
    .filter(|(_, value_id, status, _)| {
        if *status != ConditionStatus::MayFail {
            return false;
        }
        // Semantic check: is this value a constant zero?
        interval_source.constant_map().get(value_id)
            .map_or(false, |c| c.is_singleton() && c.lo() == 0)
    })
    .collect();
```

Also update `other_paths` filter (lines 971-976) to use the inverse check.

**Sub-task 2d: Short-Circuit OR Pattern (after line 1045)**

After the AND pattern check, add detection for OR patterns: `phi [true, pred_A], [result, pred_B]`.

```rust
// Check for short-circuit OR pattern:
// phi [true, pred_A], [result, pred_B]
// If all non-true paths are Proven, the phi is Proven.
if !all_guards_proven || false_paths.is_empty() {
    let true_paths: Vec<_> = path_results
        .iter()
        .filter(|(_, value_id, _, _)| {
            interval_source.constant_map().get(value_id)
                .map_or(false, |c| c.is_singleton() && c.lo() != 0)
        })
        .collect();

    if !true_paths.is_empty() {
        let non_true_paths: Vec<_> = path_results
            .iter()
            .filter(|(_, value_id, _, _)| {
                !interval_source.constant_map().get(value_id)
                    .map_or(false, |c| c.is_singleton() && c.lo() != 0)
            })
            .collect();

        let all_non_true_proven = non_true_paths
            .iter()
            .all(|(_, _, status, _)| *status == ConditionStatus::Proven);

        if all_non_true_proven {
            let descriptions: Vec<_> = path_results.iter()
                .map(|(_, _, _, desc)| desc.clone()).collect();
            return (
                ConditionStatus::Proven,
                format!("short-circuit OR proven: all non-true paths proven: [{}]",
                    descriptions.join(", ")),
                None,
            );
        }
    }
}
```

---

## Main Agent: Wiring + Verification (after both agents complete)

### Step 1: Wire ICmp methods in transfer.rs

**File:** `crates/saf-analysis/src/absint/transfer.rs`
**Lines 745-753:** Replace the 9-variant catch-all with individual dispatch:

```rust
// BEFORE:
BinaryOp::ICmpSle
| BinaryOp::ICmpSgt
| BinaryOp::ICmpSge
| BinaryOp::ICmpEq
| BinaryOp::ICmpNe
| BinaryOp::ICmpUlt
| BinaryOp::ICmpUle
| BinaryOp::ICmpUgt
| BinaryOp::ICmpUge => Interval::new(0, 1, 1),

// AFTER:
BinaryOp::ICmpSle => lhs.icmp_sle(&rhs),
BinaryOp::ICmpSgt => lhs.icmp_sgt(&rhs),
BinaryOp::ICmpSge => lhs.icmp_sge(&rhs),
BinaryOp::ICmpEq  => lhs.icmp_eq(&rhs),
BinaryOp::ICmpNe  => lhs.icmp_ne(&rhs),
BinaryOp::ICmpUlt => lhs.icmp_ult(&rhs),
BinaryOp::ICmpUle => lhs.icmp_ule(&rhs),
BinaryOp::ICmpUgt => lhs.icmp_ugt(&rhs),
BinaryOp::ICmpUge => lhs.icmp_uge(&rhs),
```

### Step 2: Format and lint
```bash
make fmt && make lint
```

### Step 3: Run tests
```bash
make test
```

### Step 4: Run PTABen benchmarks (background)
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'
```

### Regression Gate
- No unsound increase in ANY category
- ae_assert_fail unsound must not increase (would mean false precision)
- Expected: ae_assert 9→4-7, total unsound 80→73-78
