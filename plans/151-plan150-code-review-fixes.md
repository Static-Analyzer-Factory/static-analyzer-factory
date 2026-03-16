# Plan 151: Plan 150 Code Review Fixes

**Goal:** Address the 6 important issues and 7 suggestions identified by the 3-agent code review of Plan 150 (AnalyzedSpecRegistry with Computed Return Bounds).

**Context:** Plan 150 implemented a layered `AnalyzedSpecRegistry` unifying YAML specs with analysis-derived `DerivedSpec` overlays. The code review found no critical issues but identified API design gaps, performance concerns, missing derives, and documentation gaps.

---

## Issues Summary (from review)

### Important (I-1 through I-6)

| # | Issue | File | Severity |
|---|-------|------|----------|
| I-1 | `lookup()` silently drops derived-only entries; misleading doc | `analyzed.rs:46-54` | Important |
| I-2 | `find_argument_alloc_size` is O(M) whole-module scan with unbounded GEP recursion | `transfer.rs:2146-2173` | Important |
| I-3 | PTA fallback dropped from plan without documentation | `transfer.rs` | Important |
| I-4 | Computed bounds not wired into interprocedural path | `interprocedural.rs` | Important |
| I-5 | `wcslen` bound uses byte-level `AllocSizeMinusOne` (imprecise for wide chars) | `property.rs:381-390` | Important |
| I-6 | Missing `PartialEq, Eq` on `DerivedSpec` | `derived.rs:35` | Important |

### Suggestions (S-1 through S-7)

| # | Suggestion | File |
|---|-----------|------|
| S-1 | Add `Default` impl for `AnalyzedSpecRegistry` | `analyzed.rs` |
| S-2 | DRY: Loop over strlen-like function names instead of 4 identical blocks | `property.rs:352-405` |
| S-3 | Add negative test (strlen on function parameter -> TOP fallback) | test fixture + `smoke.rs` |
| S-4 | Test edge cases: `add_derived` overwrite, `derived_count()`, other `BoundMode` variants | `smoke.rs` |
| S-5 | Add `strlen` to YAML specs with wide `[0, SIZE_MAX-1]` fallback | `property.rs` |
| S-6 | Handle `Cast`/`Phi` instructions in `find_argument_alloc_size` | `transfer.rs` |
| S-7 | Register `fread`/`recv`/`read` computed bounds | `property.rs` |

---

## Task 1: Fix `AnalyzedSpecRegistry::lookup()` API (I-1)

**Files:**
- Modify: `crates/saf-core/src/spec/analyzed.rs`
- Modify: `crates/saf-core/tests/smoke.rs`

**What:** The `lookup()` method returns `Option<(&FunctionSpec, Option<&DerivedSpec>)>` which structurally cannot represent "derived spec but no YAML spec." The doc comment says "at least one must exist for a lookup to succeed," but derived-only entries return `None`.

**Fix:** Change the return type to use a `LookupResult` enum:

```rust
/// Result of looking up a function in the analyzed registry.
#[derive(Debug)]
pub enum LookupResult<'a> {
    /// YAML spec exists, with optional derived overlay.
    Yaml(&'a FunctionSpec, Option<&'a DerivedSpec>),
    /// Only analysis-derived spec exists (no YAML entry).
    DerivedOnly(&'a DerivedSpec),
}

impl<'a> LookupResult<'a> {
    /// Get the YAML spec if present.
    pub fn yaml(&self) -> Option<&'a FunctionSpec> { ... }
    /// Get the derived spec if present.
    pub fn derived(&self) -> Option<&'a DerivedSpec> { ... }
}
```

Update `lookup()` to return `Option<LookupResult>` and fix all callers (currently only tests use `lookup()` — production code uses `lookup_yaml()` and `lookup_derived()` separately).

Update the `analyzed_registry_derived_only` test to use `lookup()` instead of `lookup_derived()`, validating the `DerivedOnly` variant.

---

## Task 2: Add recursion depth guard to `find_argument_alloc_size` (I-2)

**Files:**
- Modify: `crates/saf-analysis/src/absint/transfer.rs`

**What:** `find_argument_alloc_size` scans the entire module (O(M) per call) and recurses through GEP chains without a depth limit. While practical GEP chains are short, pathological IR could cause deep recursion.

**Fix:**
1. Add a `depth` parameter (max 8) to prevent unbounded GEP chain recursion:
```rust
fn find_argument_alloc_size(
    value: ValueId,
    module: &AirModule,
    depth: u32,
) -> Option<i128> {
    if depth == 0 {
        return None;
    }
    // ... existing logic ...
    Operation::Gep { .. } => {
        if let Some(base) = inst.operands.first() {
            return find_argument_alloc_size(*base, module, depth - 1);
        }
    }
}
```
2. Call with `depth: 8` from `resolve_computed_bound`.
3. Add doc comment documenting the O(M) per-call cost and depth limit.

---

## Task 3: Document known limitations (I-3, I-4)

**Files:**
- Modify: `crates/saf-analysis/src/absint/transfer.rs` (I-3)
- Modify: `crates/saf-analysis/src/absint/interprocedural.rs` (I-4)

**What:** Two limitations exist but are undocumented:
- **I-3:** PTA-based allocation resolution was planned but dropped. Only syntactic instruction tracing is implemented, limiting computed bounds to local allocas in the same function.
- **I-4:** `InterproceduralContext` still uses `&SpecRegistry`, not `&AnalyzedSpecRegistry`. Computed bounds only apply via the direct fixpoint path (buffer overflow checker), not the interprocedural solver.

**Fix:** Add doc comments at the relevant locations:

In `resolve_computed_bound()`:
```rust
/// # Known Limitations
///
/// - Only resolves allocation sizes via syntactic instruction tracing
///   (alloca -> GEP chain). Does not use PTA to resolve through
///   points-to sets, function parameters, or phi nodes. When the
///   argument is not a local alloca, returns TOP (sound fallback).
/// - Future: integrate PTA-based resolution for heap allocations
///   and cross-function pointer flows.
```

In `interprocedural.rs` near the `specs: None` assignments:
```rust
// NOTE: InterproceduralContext uses &SpecRegistry (not AnalyzedSpecRegistry).
// Computed return bounds from AnalyzedSpecRegistry are NOT applied here.
// This is acceptable because:
// 1. Return intervals from YAML specs already flow through summary_from_spec()
// 2. The buffer overflow checker uses the direct fixpoint path where
//    AnalyzedSpecRegistry IS wired in
// Future: wire AnalyzedSpecRegistry into interprocedural solver for
// computed bounds in cross-function temporal analysis.
```

---

## Task 4: Fix `wcslen` imprecision + add `PartialEq` to `DerivedSpec` (I-5, I-6)

**Files:**
- Modify: `crates/saf-core/src/spec/derived.rs` (I-6)
- Modify: `crates/saf-bench/src/svcomp/property.rs` (I-5)

**What:**
- **I-5:** `wcslen` returns character count, not byte count. `AllocSizeMinusOne` gives byte-level bounds (e.g., [0, 43] for `wchar_t buf[11]` instead of [0, 10]).
- **I-6:** `DerivedSpec` is missing `PartialEq, Eq` derives that consumers need.

**Fix for I-6:** Add `PartialEq, Eq` to `DerivedSpec`:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedSpec { ... }
```

**Fix for I-5:** Add a comment documenting the `wcslen` imprecision and remove the registration until a proper `AllocElementCountMinusOne` mode is added:
```rust
// NOTE: wcslen returns wide-character count, not byte count.
// AllocSizeMinusOne gives byte-level bounds which over-approximates.
// Deferred: add AllocElementCountMinusOne mode that divides by sizeof(wchar_t).
// For now, wcslen falls back to the YAML interval (sound, less precise).
```

Alternatively, if keeping `wcslen` registered (it is still sound), add the explanatory comment inline.

---

## Task 5: Quality-of-life improvements (S-1, S-2, S-4)

**Files:**
- Modify: `crates/saf-core/src/spec/analyzed.rs` (S-1)
- Modify: `crates/saf-bench/src/svcomp/property.rs` (S-2)
- Modify: `crates/saf-core/tests/smoke.rs` (S-4)

**S-1: Add `Default` for `AnalyzedSpecRegistry`:**
```rust
impl Default for AnalyzedSpecRegistry {
    fn default() -> Self {
        Self::new(SpecRegistry::default())
    }
}
```

**S-2: DRY the strlen-like registrations:**
```rust
for name in ["strlen", "strnlen", "wcslen", "ldv_strlen"] {
    analyzed.add_derived(name, DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSizeMinusOne,
        }),
        ..DerivedSpec::empty()
    });
}
```

**S-4: Add missing test coverage:**
- Test `add_derived` overwrite semantics (second call replaces first)
- Test `derived_count()` and `iter_derived()`
- Test `BoundMode::AllocSize` and `BoundMode::ParamValueMinusOne` variants
- Test `DerivedSpec::from_effects()` with `lookup()` returning `DerivedOnly` variant (existing test at `smoke.rs:144` already covers `from_effects()` via `lookup_derived()`, but new `LookupResult` path needs coverage)

---

## Task 6: Add negative test for TOP fallback (S-3)

**Files:**
- Create: `tests/fixtures/llvm/e2e/strlen_param_buffer.ll`
- Modify: `crates/saf-core/tests/smoke.rs` or `crates/saf-analysis/tests/absint_e2e.rs`

**What:** Add a test fixture where strlen is called on a function parameter (not a local alloca) to verify the graceful fallback to TOP.

**Fixture:**
```llvm
declare i64 @strlen(ptr)

define i64 @test_strlen_param(ptr %buf) {
entry:
  %len = call i64 @strlen(ptr %buf)
  ret i64 %len
}
```

**Test:** Verify that `resolve_computed_bound` returns TOP for this case (the argument is a parameter, not traceable to an alloca).

---

## Deferred (Not in This Plan)

These items are valid but deferred to future work:

| # | Item | Reason |
|---|------|--------|
| S-5 | Tighten `strlen` YAML interval from `[0, i64_max]` to `[0, SIZE_MAX-1]` | `share/saf/specs/libc/string.yaml` already has strlen with `[0, 9223372036854775807]`; tightening to `SIZE_MAX-1` is a minor precision improvement deferred to broader spec audit |
| S-6 | Handle `Cast`/`Phi` in `find_argument_alloc_size` | Requires PTA integration (larger scope) |
| S-7 | Register `fread`/`recv`/`read` computed bounds | Needs `ParamValueMinusOne` testing + new fixtures |
| I-5 full | `AllocElementCountMinusOne` mode for wide-char functions | Needs element size info from AIR type system (Plan 139 dependency) |

---

## Expected Outcome

- Cleaner API: `lookup()` correctly represents all three cases (yaml-only, derived-only, both)
- Safer code: bounded recursion in `find_argument_alloc_size`
- Better docs: known limitations documented at point of use
- Better ergonomics: `Default`, `PartialEq`, DRY registrations
- Better coverage: negative tests, edge case tests, all `BoundMode` variants tested
- No functional regressions: all changes are API improvements or documentation
