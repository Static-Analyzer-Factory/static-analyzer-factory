//! Consolidated function property lookups.
//!
//! This module centralizes hardcoded knowledge about external/library function
//! behavior (purity, return aliasing, nullness, noreturn) and spec-aware
//! wrappers that consult a [`SpecRegistry`] before falling back to the
//! hardcoded lists.
//!
//! Previously these helpers were duplicated across `nullness.rs` and
//! `transfer.rs`. Having a single source of truth avoids divergence and
//! makes it easier to extend coverage.

use saf_core::spec::SpecRegistry;

use super::nullness::Nullness;

// ---------------------------------------------------------------------------
// Hardcoded property lookups (private)
// ---------------------------------------------------------------------------

/// Check if a function is known to be pure (no side effects on memory).
///
/// This includes LLVM intrinsics and common library functions that don't
/// modify user-visible memory state.
fn is_known_pure_function(name: &str) -> bool {
    // LLVM debug/lifetime intrinsics
    if name.starts_with("llvm.dbg.") || name.starts_with("llvm.lifetime.") {
        return true;
    }

    // Annotation intrinsics
    if name.starts_with("llvm.var.annotation")
        || name.starts_with("llvm.ptr.annotation")
        || name.starts_with("llvm.annotation")
    {
        return true;
    }

    // Assume intrinsic (doesn't modify memory)
    if name == "llvm.assume" {
        return true;
    }

    // Math intrinsics (pure computation)
    if name.starts_with("llvm.sin")
        || name.starts_with("llvm.cos")
        || name.starts_with("llvm.sqrt")
        || name.starts_with("llvm.pow")
        || name.starts_with("llvm.exp")
        || name.starts_with("llvm.log")
        || name.starts_with("llvm.fabs")
        || name.starts_with("llvm.floor")
        || name.starts_with("llvm.ceil")
        || name.starts_with("llvm.trunc")
        || name.starts_with("llvm.round")
        || name.starts_with("llvm.fma")
        || name.starts_with("llvm.minnum")
        || name.starts_with("llvm.maxnum")
        || name.starts_with("llvm.copysign")
    {
        return true;
    }

    // Bit manipulation intrinsics
    if name.starts_with("llvm.bswap")
        || name.starts_with("llvm.ctpop")
        || name.starts_with("llvm.ctlz")
        || name.starts_with("llvm.cttz")
        || name.starts_with("llvm.bitreverse")
        || name.starts_with("llvm.fshl")
        || name.starts_with("llvm.fshr")
    {
        return true;
    }

    // Overflow intrinsics (pure computation)
    if name.starts_with("llvm.sadd.with.overflow")
        || name.starts_with("llvm.uadd.with.overflow")
        || name.starts_with("llvm.ssub.with.overflow")
        || name.starts_with("llvm.usub.with.overflow")
        || name.starts_with("llvm.smul.with.overflow")
        || name.starts_with("llvm.umul.with.overflow")
    {
        return true;
    }

    // SVF assertion functions (don't modify memory)
    if name == "svf_assert" || name == "svf_print" || name.starts_with("svf_assert") {
        return true;
    }

    // PTABen oracle functions (don't modify memory)
    if name == "UNSAFE_LOAD"
        || name == "SAFE_LOAD"
        || name == "UNSAFE_BUFACCESS"
        || name == "SAFE_BUFACCESS"
        || name == "MUSTALIAS"
        || name == "MAYALIAS"
        || name == "NOALIAS"
        || name == "PARTIALALIAS"
        || name == "EXPECTEDFAIL_MAYALIAS"
        || name == "EXPECTEDFAIL_NOALIAS"
    {
        return true;
    }

    // External functions that return random/nondeterministic values
    // but don't modify caller's memory (no pointer arguments)
    if name == "nd" || name == "nd_int" || name == "rand" || name == "srand" || name == "time" {
        return true;
    }

    // I/O functions that read their pointer args but don't modify caller's memory
    if name == "printf" || name == "puts" || name == "putchar" || name == "putc" {
        return true;
    }

    false
}

/// Check if a function is `set_value(var, lb, ub)` which constrains
/// a variable's interval for testing/benchmarking purposes.
pub fn is_set_value_function(name: &str) -> bool {
    name == "set_value"
}

/// Check if a function returns its first argument.
///
/// Many C library functions (memcpy, strcpy, strcat, etc.) return their
/// destination pointer. If the first argument is non-null, the return is non-null.
pub(crate) fn returns_first_argument(name: &str) -> bool {
    matches!(
        name,
        // Memory functions
        "memcpy"
            | "memmove"
            | "memset"
            | "memchr"
            | "llvm.memcpy.p0.p0.i64"
            | "llvm.memcpy.p0.p0.i32"
            | "llvm.memmove.p0.p0.i64"
            | "llvm.memmove.p0.p0.i32"
            | "llvm.memset.p0.i64"
            | "llvm.memset.p0.i32"
            // String functions
            | "strcpy"
            | "strncpy"
            | "strcat"
            | "strncat"
            | "stpcpy"
            | "stpncpy"
            // Wide string functions
            | "wcscpy"
            | "wcsncpy"
            | "wcscat"
            | "wcsncat"
            | "wmemcpy"
            | "wmemmove"
            | "wmemset"
    )
}

/// Check if a function always returns a non-null pointer.
///
/// These functions either return a valid pointer or abort/exit.
pub(crate) fn returns_nonnull(name: &str) -> bool {
    matches!(
        name,
        // Allocation functions that abort on failure
        "xmalloc" | "xcalloc" | "xrealloc" | "xstrdup" |
        // String search functions that return pointer into input
        "strchr" | "strrchr" | "strstr" | "strpbrk" |
        // GNU extensions
        "rawmemchr"
    )
}

/// Check if a function is a known deallocation function (hardcoded list).
///
/// Covers C `free`, C++ `operator delete`/`operator delete[]` (with and
/// without size parameter), and common library deallocators.
fn is_known_deallocation_function(name: &str) -> bool {
    matches!(
        name,
        "free" | "_ZdlPv" | "_ZdaPv" | "_ZdlPvm" | "_ZdaPvm" | "cfree" | "g_free" | "safe_free"
    )
}

/// Check if a function is known to be noreturn (hardcoded list).
fn is_known_noreturn(name: &str) -> bool {
    matches!(
        name,
        "exit"
            | "_exit"
            | "_Exit"
            | "abort"
            | "__assert_fail"
            | "__assert_rtn"
            | "longjmp"
            | "siglongjmp"
            | "pthread_exit"
            | "thrd_exit"
    )
}

// ---------------------------------------------------------------------------
// Spec-aware public API
// ---------------------------------------------------------------------------

/// Check if a function is pure using specs, falling back to hardcoded list.
///
/// Returns true if:
/// - The spec for this function has `pure: true`, OR
/// - The spec indicates purity by having only `reads` params (no `modifies`, no `escapes`), OR
/// - The function is in the hardcoded list of known pure functions
///
/// Allocators and deallocators are never considered pure even if they don't have
/// `modifies` params, because they have system-level side effects.
#[must_use]
pub fn is_pure_function_with_specs(name: &str, specs: Option<&SpecRegistry>) -> bool {
    // Check specs first
    if let Some(specs) = specs {
        if let Some(spec) = specs.lookup(name) {
            // Explicit pure marking
            if spec.pure == Some(true) {
                return true;
            }

            // Exclude allocators/deallocators (they have side effects)
            if matches!(
                spec.role,
                Some(
                    saf_core::spec::Role::Allocator
                        | saf_core::spec::Role::Deallocator
                        | saf_core::spec::Role::Reallocator
                )
            ) {
                return false;
            }

            // Infer purity from params: if all params are read-only
            // (no modifies, no escapes), function is effectively pure
            if !spec.params.is_empty() {
                let has_modifies = spec.params.iter().any(|p| p.modifies == Some(true));
                let has_escapes = spec.params.iter().any(|p| p.escapes == Some(true));
                let has_reads = spec.params.iter().any(|p| p.reads == Some(true));

                // Pure if: has some read params, no modifies, no escapes
                if has_reads && !has_modifies && !has_escapes {
                    return true;
                }
            }
        }
    }
    // Fall back to hardcoded list
    is_known_pure_function(name)
}

/// Check if a function is pure using specs first, then hardcoded list.
///
/// Convenience wrapper around [`is_pure_function_with_specs`] that takes a
/// non-optional `&SpecRegistry`.
#[must_use]
pub fn is_pure_with_spec(name: &str, specs: &SpecRegistry) -> bool {
    is_pure_function_with_specs(name, Some(specs))
}

/// Check if a function returns its first argument (spec-first, then hardcoded fallback).
///
/// First checks `returns.aliases` in spec, then falls back to hardcoded list.
/// Returns `Some(param_index)` if the function returns one of its arguments.
#[must_use]
pub fn returns_alias_param_with_specs(name: &str, specs: Option<&SpecRegistry>) -> Option<u32> {
    // Check spec first
    if let Some(registry) = specs {
        if let Some(idx) = return_aliases_param_from_spec(name, registry) {
            return Some(idx);
        }
    }

    // Fallback to hardcoded list (always returns param 0)
    if returns_first_argument(name) {
        return Some(0);
    }

    None
}

/// Check if a function always returns a non-null pointer (spec-first, then hardcoded fallback).
///
/// First checks `returns.nullness == not_null` in spec, then falls back to hardcoded list.
#[must_use]
pub fn returns_nonnull_with_specs(name: &str, specs: Option<&SpecRegistry>) -> bool {
    // Check spec first
    if let Some(registry) = specs {
        if let Some(nullness) = return_nullness_from_spec(name, registry) {
            return nullness == Nullness::NotNull;
        }
    }

    // Fallback to hardcoded list
    returns_nonnull(name)
}

/// Check if a function is a deallocator (spec-first, then hardcoded fallback).
///
/// First checks for `Role::Deallocator` in spec, then falls back to hardcoded list.
#[must_use]
pub fn is_deallocation_with_specs(name: &str, specs: Option<&SpecRegistry>) -> bool {
    if let Some(registry) = specs {
        if let Some(spec) = registry.lookup(name) {
            if matches!(spec.role, Some(saf_core::spec::Role::Deallocator)) {
                return true;
            }
        }
    }
    is_known_deallocation_function(name)
}

/// Check if a function is noreturn (spec-first, then hardcoded fallback).
///
/// First checks `noreturn: true` in spec, then falls back to hardcoded list.
#[must_use]
pub fn is_noreturn_with_specs(name: &str, specs: Option<&SpecRegistry>) -> bool {
    // Check spec first
    if let Some(registry) = specs {
        if is_noreturn_from_spec(name, registry) {
            return true;
        }
    }

    // Fallback to hardcoded list
    is_known_noreturn(name)
}

// ---------------------------------------------------------------------------
// Spec-only helpers
// ---------------------------------------------------------------------------

/// Get return nullness from spec if available.
///
/// Returns `Some(Nullness)` if the spec defines return nullness, `None` otherwise.
#[must_use]
pub fn return_nullness_from_spec(name: &str, specs: &SpecRegistry) -> Option<Nullness> {
    let spec = specs.lookup(name)?;
    let returns = spec.returns.as_ref()?;
    let spec_nullness = returns.nullness.as_ref()?;

    Some(match spec_nullness {
        saf_core::spec::Nullness::NotNull => Nullness::NotNull,
        saf_core::spec::Nullness::MaybeNull => Nullness::MaybeNull,
        // RequiredNonnull and Nullable are for parameters, not returns
        saf_core::spec::Nullness::RequiredNonnull | saf_core::spec::Nullness::Nullable => {
            return None;
        }
    })
}

/// Check if return aliases a parameter from spec.
///
/// Returns `Some(param_index)` if the spec says return aliases param.N.
#[must_use]
pub fn return_aliases_param_from_spec(name: &str, specs: &SpecRegistry) -> Option<u32> {
    let spec = specs.lookup(name)?;
    let returns = spec.returns.as_ref()?;
    returns.alias_param_index()
}

/// Check if a parameter has `required_nonnull` from spec.
///
/// Returns true if the spec says this parameter must not be null.
#[must_use]
pub fn param_required_nonnull_from_spec(
    name: &str,
    param_index: u32,
    specs: &SpecRegistry,
) -> bool {
    specs
        .lookup(name)
        .and_then(|spec| spec.param(param_index))
        .and_then(|p| p.nullness)
        .is_some_and(|n| n == saf_core::spec::Nullness::RequiredNonnull)
}

/// Check if a function is noreturn from spec.
#[must_use]
pub fn is_noreturn_from_spec(name: &str, specs: &SpecRegistry) -> bool {
    specs
        .lookup(name)
        .is_some_and(saf_core::spec::FunctionSpec::is_noreturn)
}
