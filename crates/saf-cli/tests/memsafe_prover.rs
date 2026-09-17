//! Movement 2 (`plans/214`) — CLI-level regressions for the Anchored-Object
//! `valid-memsafety` prover.
//!
//! Each negative fixture pins exactly one obligation and is a program the prover
//! would answer TRUE on if that obligation were removed. `valid-memsafety` TRUE is
//! verdict-only under the 2027 rules, so nothing downstream re-checks the answer —
//! a wrong PROVE goes straight to a wrong TRUE at −32 points, uncapped by the
//! per-cluster dedup. That is why these are end-to-end (clang → mem2reg → AIR →
//! prover) rather than unit tests on a hand-built module: two of the four
//! mechanisms below are invisible in a hand-built `AirModule` because they are
//! introduced *by ingestion*.
//!
//! These need clang-18/opt-18 (Docker only), so they are `#[ignore]`d like the rest
//! of the LLVM-dependent suite. Run them with `make test-ignored`.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

fn fixture(name: &str) -> String {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/saf-cli -> crates
    p.pop(); // crates -> workspace root
    p.join("tests/programs/c/svcomp")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// `saf memsafe-prove --data-model ILP32 <fixture>`, asserting exit 0.
///
/// ILP32 matches the SV-COMP corpus: every one of the 388 tasks in the measured
/// yield population declares `data_model: ILP32`.
fn memsafe_prove(name: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("saf")
        .args([
            "memsafe-prove",
            "--data-model",
            "ILP32",
            fixture(name).as_str(),
        ])
        .assert()
        .success()
}

// =============================================================================
// The positive case
// =============================================================================

/// Two threads, a mutex, four globals, every access a whole-object global access.
/// This is the shape the yielding clusters actually have, and it must prove —
/// including across the thread bodies.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn proves_threaded_whole_object_globals() {
    memsafe_prove("memsafe_prove_threaded_globals.c").stdout(predicate::str::starts_with("PROVE"));
}

/// GUARD against a VACUOUS proof. `main` is spotless here; the only unsafe access
/// is in a thread body, which is unreachable from `main` in the call graph. If
/// MTA's thread-entry discovery came back empty, the universe would be main's tree
/// alone and the prover would answer TRUE having never looked at the code that
/// runs. Pairs with [`proves_threaded_whole_object_globals`]: together they show
/// the concurrent answer is decided by the thread bodies, not despite them.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn thread_bodies_are_actually_scanned() {
    memsafe_prove("memsafe_thread_body_unsafe.c")
        .stdout(predicate::str::starts_with("ABSTAIN:store-out-of-bounds"));
}

// =============================================================================
// Obligation 3 — bounds
// =============================================================================

/// A 4-byte store into a 1-byte global. Under LLVM 18's opaque pointers the
/// `(int *)&g` cast leaves **no** instruction and **no** constant expression, so
/// the AIR is an ordinary anchored `Store` — the access WIDTH against the object
/// SIZE is the only thing that separates it from a safe whole-object store.
///
/// RED without `check_access`'s `width <= size` test, and RED without
/// `AirGlobal.value_type` being populated by the LLVM frontend (there would be no
/// size to compare against).
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn bounds_obligation_rejects_over_wide_store() {
    memsafe_prove("memsafe_bounds_oob_width.c")
        .stdout(predicate::str::starts_with("ABSTAIN:store-out-of-bounds"));
}

// =============================================================================
// Obligation 2 — non-inert libc
// =============================================================================

/// Every access this program makes through its own instructions is safe and
/// anchored; the out-of-bounds read happens inside `puts`, walking a `char[4]`
/// with no NUL. RED the moment an external that dereferences a caller-supplied
/// pointer is admitted — which is the one soundness escape the plan's Python
/// spike actually measured.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn non_inert_libc_obligation_rejects_puts() {
    memsafe_prove("memsafe_noninert_libc.c").stdout(predicate::str::starts_with(
        "ABSTAIN:non-inert-external:puts",
    ));
}

// =============================================================================
// Obligation 1 — heap-free
// =============================================================================

/// `valid-free` and `valid-memtrack` are discharged vacuously by proving only
/// programs that never allocate. This one leaks and writes through a heap pointer.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn heap_obligation_rejects_malloc() {
    memsafe_prove("memsafe_heap_alloc.c").stdout(predicate::str::starts_with("ABSTAIN:"));
}

// =============================================================================
// Ingestion fidelity — the mechanism that is invisible in the instruction stream
// =============================================================================

/// A store 2000 bytes past a 400-byte array. Measured at `5f274d13`: this program
/// and the same one with `arr[5]` produce **structurally identical AIR**, because
/// ingestion resolves the constant-expression `getelementptr` to `arr`'s bare
/// `ValueId` and `FieldPath` cannot express a byte offset.
///
/// No obligation over the instruction stream can catch this — the prover sees an
/// anchored, offset-0, 4-byte store into a 400-byte object and every check passes.
/// RED without `IngestFidelity::collapsed_const_ptr_expr`, and RED in the worst
/// possible direction: a confident TRUE on a ground-truth-FALSE program.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn fidelity_gate_rejects_collapsed_constant_expression() {
    memsafe_prove("memsafe_collapsed_constexpr.c").stdout(predicate::str::starts_with(
        "ABSTAIN:ingest-collapsed-const-ptr-expr",
    ));
}

// =============================================================================
// The anchoring rule itself
// =============================================================================

/// A pointer read out of MEMORY never anchors — the rule that makes the domain
/// immune to dangling pointers, aliasing and interference.
///
/// Asserts only that it abstains, deliberately. The reason reported is
/// `load-width-unknown`, because `exact_size_of` declines pointer widths (the
/// AIR's `target_pointer_width` is hardcoded to 8 while these tasks are ILP32) and
/// the scan stops at the pointer load before reaching the dereference. Pinning the
/// exact string would pin the wrong rule.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn a_load_result_never_anchors() {
    memsafe_prove("memsafe_unanchored_load.c").stdout(predicate::str::starts_with("ABSTAIN:"));
}

/// Stack anchoring must not leak across a C scope. Allowing an alloca to anchor is
/// sound only because a DIRECT access to the slot can occur solely inside the
/// owning frame; here the address escapes its block through a pointer variable,
/// and clang leaves nothing in the IR to say the scope ended. Rejected twice over
/// — see the fixture. Abstain-only for the same reason as above.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn stack_anchoring_does_not_leak_across_scope() {
    memsafe_prove("memsafe_use_after_scope.c").stdout(predicate::str::starts_with("ABSTAIN:"));
}

/// The stack mirror of [`bounds_obligation_rejects_over_wide_store`]: a 4-byte
/// store into a 1-byte LOCAL. RED without `ALLOCA_EXACT_SIZE_KEY` — the older
/// `Operation::Alloca { size_bytes }` reports 8 bytes for every float and pointer,
/// and an over-stated object size makes this check pass.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn bounds_obligation_rejects_over_wide_stack_store() {
    memsafe_prove("memsafe_bounds_oob_stack.c")
        .stdout(predicate::str::starts_with("ABSTAIN:store-out-of-bounds"));
}

/// Anchoring is intraprocedural: a global passed to a helper arrives as a
/// parameter and is no longer anchored. The access here is genuinely SAFE, so
/// this pins deliberate recall loss, not a soundness rule — kept because it is
/// the single largest source of attrition and should fail loudly if someone adds
/// an interprocedural anchor summary without arguing its soundness.
#[test]
#[ignore = "requires clang-18/opt-18 (Docker); run via `make test-ignored`"]
fn anchoring_does_not_cross_a_call() {
    memsafe_prove("memsafe_unanchored_param.c")
        .stdout(predicate::str::starts_with("ABSTAIN:store-unanchored"));
}
