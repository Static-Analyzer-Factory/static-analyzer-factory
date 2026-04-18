//! End-to-end tests targeting LLVM IR language changes between LLVM 18 and
//! LLVM 22. Each fixture is compiled from a C or C++ source in
//! `tests/programs/{c,cpp}/llvm22_syntax/` using clang-22 inside the
//! `dev-llvm22` image via `make compile-llvm22-syntax-fixtures`. The sources
//! are chosen to force clang into emitting the specific post-LLVM-18 IR
//! construct under test.
//!
//! This file is gated on `feature = "llvm-22"` — the compiled fixtures use
//! IR syntax the LLVM 18 parser rejects outright, so tests are only wired in
//! when SAF is built against LLVM 22.
//!
//! Changes from LLVM 19–22 that are *not* surfaced by normal C/C++ compilation
//! (e.g. `ptrtoaddr`, GEP `nusw`/`nuw` flags) are intentionally not tested
//! here — they would require hand-written IR or fragile opt-pipeline coaxing.

#![cfg(feature = "llvm-22")]

use std::path::PathBuf;

use saf_core::air::{AirBundle, FieldStep, Operation};
use saf_core::config::Config;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

/// Resolve a fixture path under `tests/fixtures/llvm/llvm22_syntax/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("llvm")
        .join("llvm22_syntax")
        .join(name)
}

fn load(name: &str) -> AirBundle {
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let path = fixture(name);
    frontend
        .ingest(&[path.as_path()], &config)
        .unwrap_or_else(|e| panic!("Failed to load LLVM 22 fixture '{name}': {e}"))
}

// -----------------------------------------------------------------------------
// LLVM 19 -> 21 — `nocapture` replaced by `captures(none)` attribute
// -----------------------------------------------------------------------------

/// Compiled from `tests/programs/c/llvm22_syntax/captures_attr.c`. clang-22
/// infers `captures(none)` on `read_only`'s pointer parameter (pointer is
/// only read, never stored) and emits it as a direct attribute in IR. SAF's
/// frontend must parse the new attribute spelling without dropping the
/// function or the call site in `pass_through`.
#[test]
fn llvm21_captures_attribute_parses_and_preserves_callgraph() {
    let bundle = load("captures_attr.ll");
    let mod_ = &bundle.module;

    let names: Vec<&str> = mod_.functions.iter().map(|f| f.name.as_str()).collect();
    for expected in ["read_only", "pass_through", "sink_opaque"] {
        assert!(
            names.contains(&expected),
            "expected function `{expected}` in module; got {names:?}"
        );
    }

    // `pass_through` calls the declared opaque sink exactly once — that call
    // must survive the attribute parsing.
    let pass_through = mod_
        .functions
        .iter()
        .find(|f| f.name == "pass_through")
        .expect("pass_through function missing");
    let call_count = pass_through
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter(|inst| {
            matches!(
                inst.op,
                Operation::CallDirect { .. } | Operation::CallIndirect { .. }
            )
        })
        .count();
    assert_eq!(
        call_count, 1,
        "`pass_through` should retain its single call to sink_opaque"
    );
}

// -----------------------------------------------------------------------------
// LLVM 22 — masked load/store/gather/scatter intrinsic signature change
// -----------------------------------------------------------------------------

/// Compiled from `tests/programs/c/llvm22_syntax/masked_intrinsic.c` with
/// `-O2 -mavx2`. The loop vectorizer emits `@llvm.masked.store.v8f32.p0`
/// whose LLVM 22 signature takes no explicit alignment operand — alignment
/// now rides on the pointer argument's `align` attribute. SAF must parse the
/// new shape so downstream passes treat the call like any other memory
/// operation rather than silently skipping it.
#[test]
fn llvm22_masked_intrinsic_new_signature_parses() {
    let bundle = load("masked_intrinsic.ll");
    let mod_ = &bundle.module;

    let cs = mod_
        .functions
        .iter()
        .find(|f| f.name == "conditional_store")
        .expect("conditional_store function missing");

    // The vectorized loop emits at least one `llvm.masked.store` call; a
    // silent parse failure would produce zero calls in the function body.
    let call_count = cs
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter(|inst| {
            matches!(
                inst.op,
                Operation::CallDirect { .. } | Operation::CallIndirect { .. }
            )
        })
        .count();
    assert!(
        call_count >= 1,
        "`conditional_store` should carry at least one call to @llvm.masked.* \
         (got {call_count}) — a parse regression would produce zero"
    );
}

// -----------------------------------------------------------------------------
// LLVM 20 — `inrange(low, high)` GEP attribute position (regression test for
// the bug diagnosed during PTABen-LLVM22: parse_constant_gep was fooled by
// inline `inrange`, and decompose_constant_gep kept LLVM's pointer-level
// index as a field step — the two bugs cancelled on LLVM 18 but diverged
// here.)
// -----------------------------------------------------------------------------

/// Compiled from `tests/programs/cpp/llvm22_syntax/inrange_attr.cpp`.
/// clang-22 lowers `A::A()`'s implicit vtable-install into a store of the
/// constant-expression GEP
///   `getelementptr inbounds inrange(-16, 8) ({ [3 x ptr] }, ptr @_ZTV1A,
///    i32 0, i32 0, i32 2)`
/// inside the constructor `_ZN1AC2Ev`. The GEP must decompose to a single
/// `Operation::Gep` with FieldPath `[Field{0}, Field{2}]` — the struct
/// field 0 (the [3 x ptr] array), then array element 2. Anything else means
/// either the pointer-level index leaked in (3 steps) or a descent index
/// was dropped (1 step).
#[test]
fn llvm20_inrange_gep_attribute_yields_correct_field_path() {
    let bundle = load("inrange_attr.ll");
    let ctor = bundle
        .module
        .functions
        .iter()
        .find(|f| f.name == "_ZN1AC2Ev")
        .expect("A::A() (`_ZN1AC2Ev`) missing from module");

    let mut geps: Vec<&Operation> = ctor
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .map(|inst| &inst.op)
        .filter(|op| matches!(op, Operation::Gep { .. }))
        .collect();

    assert_eq!(
        geps.len(),
        1,
        "A::A() should synthesize exactly one Gep from the vtable-install \
         constant-expression GEP"
    );
    let Operation::Gep { field_path } = geps.pop().unwrap() else {
        unreachable!()
    };

    let steps: Vec<u32> = field_path
        .steps
        .iter()
        .filter_map(|s| match s {
            FieldStep::Field { index } => Some(*index),
            FieldStep::Index => None,
        })
        .collect();
    assert_eq!(
        steps,
        vec![0u32, 2u32],
        "FieldPath must be [Field{{0}}, Field{{2}}] — one step per type \
         descent, not [Field{{0}}, Field{{0}}, Field{{2}}] (pointer-level \
         index leaking in) nor [Field{{0}}] (descent index dropped)"
    );
}
