# Plan 037: Clippy Lint Cleanup

**Epic:** E22 — Clippy Lint Cleanup
**Status:** approved
**Scope:** Moderate (fix crate-level suppressions, improve function-level documentation)

## Overview

Reduce unnecessary clippy suppressions across the SAF codebase by:
1. Removing crate-level `#![allow(clippy::...)]` directives and fixing underlying code
2. Fixing clearly wrong function-level suppressions
3. Improving documentation on legitimate suppressions
4. Updating CLAUDE.md with generalizable patterns

## Current State

- `saf-analysis/src/lib.rs`: **27** crate-level suppressions (tagged TODO(E21))
- `saf-python/src/lib.rs`: **30** crate-level suppressions (tagged TODO(E21))
- **~35** function-level `#[allow(clippy::...)]` across crates

## Target State

- **~0-5** crate-level suppressions (only PyO3-specific that cannot be avoided)
- **~30-40** function-level suppressions with clear invariant documentation
- CLAUDE.md updated with lint handling patterns

---

## Phase 1: Function-Level Suppressions (saf-analysis)

**Scope:** 30-45 min | **Risk:** Low

### Tasks

- [ ] `selector/resolve.rs:60` — `unused_self` on `resolve_value_id`
  - **Fix:** Convert to standalone function (doesn't use `self`)
- [ ] `selector/resolve.rs:68,94,114,136,147,176` — `unnecessary_wraps`
  - **Keep:** API consistency requires `Result` return type
  - **Action:** Improve comment explaining API contract
- [ ] `svfg/builder.rs:284` — `too_many_lines`
  - **Keep:** Algorithm requires exhaustive pattern matching
  - **Action:** Add comment explaining algorithmic reason
- [ ] `valueflow/builder.rs:93` — `too_many_lines`
  - **Keep:** Instruction processing requires exhaustive matching
  - **Action:** Add comment
- [ ] `ifds/solver.rs:38,525` — `too_many_lines`, `too_many_arguments`
  - **Keep:** IFDS tabulation algorithm complexity
  - **Action:** Add comments explaining algorithm structure
- [ ] `ifds/ide_solver.rs:42,166,173,179,670` — `too_many_lines`, `type_complexity`, `too_many_arguments`
  - **Keep:** IDE solver extends IFDS with edge functions
  - **Action:** Add comments, consider type aliases for `type_complexity`

### Verification

```bash
make lint && make test
```

---

## Phase 2: Function-Level Suppressions (saf-frontends)

**Scope:** 30 min | **Risk:** Low

### Tasks

- [ ] `llvm/mapping.rs:125,132` — `unused_self`
  - **Review:** Comment says "may use self in future"
  - **Action:** Either use `self` or make static with clearer roadmap comment
- [ ] `llvm/mapping.rs:250,768,827` — `cast_possible_truncation`
  - **Keep:** Safe truncations with documented invariants
  - **Action:** Ensure comments document invariants (e.g., "LLVM limits params to < 2^32")
- [ ] `llvm/mapping.rs:508,550,570,642,691` — `unnecessary_wraps`
  - **Keep:** FFI error handling consistency
  - **Action:** Improve comment
- [ ] `llvm/mapping.rs:325` — `too_many_lines`
  - **Keep:** `convert_instruction` requires exhaustive pattern matching
- [ ] `llvm/mapping.rs:768` — `cast_sign_loss`
  - **Keep:** Safe bitwise reinterpretation
  - **Action:** Document invariant
- [ ] `air_json.rs:226,326` — `cast_possible_truncation`, `too_many_lines`
  - **Keep:** JSON parsing constraints
- [ ] `llvm/intrinsics.rs:49` — `too_many_lines`
  - **Keep:** Intrinsic classification table
- [ ] `pta/extract.rs:149,157` — `cast_possible_truncation`, `cast_sign_loss`
  - **Keep:** Array index and pointer encoding
  - **Action:** Document invariants

### Verification

```bash
make lint && make test
```

---

## Phase 3: Easy Crate-Level Lints (saf-analysis Part 1)

**Scope:** 45-60 min | **Risk:** Low

### Tasks

Remove from `saf-analysis/src/lib.rs` and fix violations:

- [ ] `uninlined_format_args` — `format!("{}", x)` → `format!("{x}")`
- [ ] `for_kv_map` — `for (_, v) in map` → `for v in map.values()`
- [ ] `items_after_statements` — move function/const items before `let` statements
- [ ] `redundant_closure_for_method_calls` — `|x| x.method()` → `T::method`
- [ ] `if_not_else` — `if !x { A } else { B }` → `if x { B } else { A }`
- [ ] `inefficient_to_string` — `"str".to_string()` → `"str".to_owned()`
- [ ] `needless_borrowed_reference` — `&ref x` → `x` in patterns
- [ ] `field_reassign_with_default` — use struct update syntax

### Process

1. Remove one `#![allow]` at a time from `lib.rs`
2. Run `cargo clippy -p saf-analysis -- -D warnings`
3. Fix all violations
4. Repeat for next lint
5. Run full `make lint && make test`

### Verification

```bash
make lint && make test
```

---

## Phase 4: Easy Crate-Level Lints (saf-analysis Part 2)

**Scope:** 45-60 min | **Risk:** Low

### Tasks

- [ ] `manual_let_else` — use `let Some(x) = expr else { return; };` pattern
- [ ] `match_same_arms` — combine identical arms with `|`
- [ ] `map_unwrap_or` — `.map(f).unwrap_or(d)` → `.map_or(d, f)`
- [ ] `unnecessary_map_or` — `.map_or(false, |x| cond)` → `.is_some_and(|x| cond)`
- [ ] `comparison_chain` — `if a < b {} else if a > b {}` → `match a.cmp(&b) {}`
- [ ] `unnecessary_get_then_check` — `.get(k).is_some()` → `.contains_key(k)`

### Verification

```bash
make lint && make test
```

---

## Phase 5: Map Pattern Lints (saf-analysis)

**Scope:** 45-60 min | **Risk:** Medium

### Tasks

- [ ] `map_entry` — Use Entry API where appropriate
  - **Caution:** Not all `if !contains { insert }` can use Entry (computation may depend on absence)
  - **Action:** Review each case individually
- [ ] `zero_sized_map_values` — `BTreeMap<K, ()>` → `BTreeSet<K>`
- [ ] `never_loop` — Fix loops that only iterate once (likely bugs or simplifiable)

### Verification

```bash
make lint && make test
```

---

## Phase 6: Type and Reference Lints (saf-analysis)

**Scope:** 60 min | **Risk:** Medium (API changes possible)

### Tasks

- [ ] `trivially_copy_pass_by_ref` — Pass `Copy` types by value (e.g., `&u64` → `u64`)
- [ ] `needless_pass_by_value` — Take `&T` when ownership not needed
- [ ] `type_complexity` — Create type aliases for complex generic types
  - Example: `BTreeMap<(F, InstId, F), BuiltinEdgeFn<V>>` → `type JumpFnTable<F, V> = ...`
- [ ] `return_self_not_must_use` — Add `#[must_use]` to builder methods returning `Self`

### Notes

These may change public API signatures. Verify all call sites in tests.

### Verification

```bash
make lint && make test
```

---

## Phase 7: Cast Safety Lints (saf-analysis)

**Scope:** 60 min | **Risk:** Medium (requires safety review)

### Tasks

Remove crate-level allows and add function-level with invariant documentation:

- [ ] `cast_possible_truncation`
  - Review each cast site
  - If safe: add function-level `#[allow]` with invariant comment
  - If unsafe: use `try_into()` with error handling
- [ ] `cast_sign_loss`
  - Similar review — document bitwise semantics
- [ ] `cast_possible_wrap`
  - Review i128↔u128 casts for correctness

### Invariant Documentation Pattern

```rust
#[allow(clippy::cast_possible_truncation)]
// INVARIANT: Array index from enumerate() fits in u32 — arrays cannot exceed 2^32 elements
let field_index = i as u32;
```

### Verification

```bash
make lint && make test
```

---

## Phase 8: saf-python Crate-Level Lints

**Scope:** 90 min | **Risk:** Medium

### Tasks

Apply patterns from Phases 3-7 to `saf-python/src/lib.rs`.

**PyO3-specific lints to KEEP at crate level:**
- `unnecessary_wraps` — PyO3 `#[pyfunction]` often requires `PyResult`
- `needless_pass_by_value` — PyO3 requires owned types for Python interop
- `unused_self` — `#[pymethods]` requires `&self` parameter

**Lints to FIX:**
- Same easy lints as Phases 3-4
- Same type/reference lints as Phase 6

**Process:**
1. Remove non-PyO3 lints one at a time
2. Fix violations
3. For PyO3-specific violations, move to function-level with explanation

### Verification

```bash
make lint && make test
```

---

## Phase 9: Documentation Lints

**Scope:** 60-90 min | **Risk:** Low

### Tasks

- [ ] `doc_markdown` — Add backticks to identifiers in doc comments
  - Focus on public API first
  - Pattern: `ValueId` → `` `ValueId` ``
- [ ] `missing_errors_doc` — Add `# Errors` sections to public functions returning `Result`
  - Focus on high-traffic APIs
- [ ] `too_many_arguments` — Move remaining violations to function-level with explanations
- [ ] `too_many_lines` — Move remaining violations to function-level with explanations

### Verification

```bash
make lint && make test && cargo doc --no-deps -p saf-analysis -p saf-python
```

---

## Phase 10: CLAUDE.md Updates and Final Verification

**Scope:** 30 min | **Risk:** None

### Tasks

- [ ] Update CLAUDE.md Rust Conventions section with:
  - Cast safety documentation pattern
  - PyO3-specific clippy patterns
  - Algorithm complexity documentation pattern
  - When to use function-level vs crate-level allows
- [ ] Update `plans/PROGRESS.md`:
  - Mark E22 complete
  - Update Session Log
- [ ] Run full verification:
  ```bash
  make lint && make test
  ```
- [ ] Commit changes

### CLAUDE.md Additions

```markdown
### Clippy Lint Handling

**Cast safety (`cast_possible_truncation`, `cast_sign_loss`, `cast_possible_wrap`):**
When allowing cast lints, document the invariant:
```rust
#[allow(clippy::cast_possible_truncation)]
// INVARIANT: LLVM limits function parameters to < 2^32
fn param_index(i: usize) -> u32 { i as u32 }
```

**PyO3-specific allows:**
- `unnecessary_wraps`: Required for `#[pyfunction]` returning `PyResult`
- `unused_self`: Required for `#[pymethods]` on unit structs
- `needless_pass_by_value`: PyO3 requires owned types for conversion

**Algorithm complexity (`too_many_lines`, `too_many_arguments`):**
Add comments explaining why refactoring is not beneficial:
```rust
#[allow(clippy::too_many_lines)]
// NOTE: This function implements the IFDS tabulation algorithm as a single
// cohesive unit. Splitting would obscure the algorithm structure.
fn solve_ifds() { ... }
```

**Prefer function-level over crate-level:**
- Crate-level `#![allow]` hides issues across the entire crate
- Function-level `#[allow]` with comment documents the specific reason
- Only use crate-level for pervasive PyO3 constraints that cannot be avoided
```

### Verification

Full test suite passes, all lints clean.

---

## Commit Strategy

**Commit after each phase** to maintain clean history and enable easy rollback:

```
Phase 1: git commit -m "E22 Phase 1: Improve function-level suppression comments (saf-analysis)"
Phase 2: git commit -m "E22 Phase 2: Improve function-level suppression comments (saf-frontends)"
Phase 3: git commit -m "E22 Phase 3: Fix easy clippy lints Part 1 (saf-analysis)"
Phase 4: git commit -m "E22 Phase 4: Fix easy clippy lints Part 2 (saf-analysis)"
Phase 5: git commit -m "E22 Phase 5: Fix map pattern lints (saf-analysis)"
Phase 6: git commit -m "E22 Phase 6: Fix type and reference lints (saf-analysis)"
Phase 7: git commit -m "E22 Phase 7: Document cast safety invariants (saf-analysis)"
Phase 8: git commit -m "E22 Phase 8: Fix clippy lints (saf-python)"
Phase 9: git commit -m "E22 Phase 9: Fix documentation lints"
Phase 10: git commit -m "E22 Phase 10: Update CLAUDE.md with lint patterns"
```

---

## Summary

| Phase | Description | Time | Risk |
|-------|-------------|------|------|
| 1 | Function-level (saf-analysis) | 30-45 min | Low |
| 2 | Function-level (saf-frontends) | 30 min | Low |
| 3 | Easy crate-level Part 1 | 45-60 min | Low |
| 4 | Easy crate-level Part 2 | 45-60 min | Low |
| 5 | Map pattern lints | 45-60 min | Medium |
| 6 | Type and reference lints | 60 min | Medium |
| 7 | Cast safety lints | 60 min | Medium |
| 8 | saf-python lints | 90 min | Medium |
| 9 | Documentation lints | 60-90 min | Low |
| 10 | CLAUDE.md + verification | 30 min | None |

**Total:** ~8-10 hours across 10 sessions

**Execution order:** 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10

Phases 1-4 are independent quick wins. Phases 5-7 require careful review. Phase 8 applies patterns from earlier phases. Phase 9-10 are finishing touches.

---

## Critical Files

- `crates/saf-analysis/src/lib.rs` — 27 crate-level suppressions
- `crates/saf-python/src/lib.rs` — 30 crate-level suppressions
- `crates/saf-analysis/src/selector/resolve.rs` — `unused_self`, `unnecessary_wraps`
- `crates/saf-frontends/src/llvm/mapping.rs` — 13 function-level suppressions
- `CLAUDE.md` — Document patterns
- `plans/PROGRESS.md` — Track progress
