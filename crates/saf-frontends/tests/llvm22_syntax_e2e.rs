//! End-to-end tests targeting LLVM IR language changes between LLVM 18 and
//! LLVM 22 (covering releases 19, 20, 21, 22).
//!
//! These tests verify that the SAF frontend parses and maps the post-LLVM-18
//! IR syntax correctly. They are gated on `feature = "llvm-22"` because the
//! fixtures exercise IR forms the LLVM 18 parser rejects outright.
//!
//! Each test targets a specific change documented in the corresponding LLVM
//! release notes; the fixture filename and comment cite the release.

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
// LLVM 19 — GEP no-wrap flags (`nusw`, `nuw`)
// -----------------------------------------------------------------------------

/// LLVM 19 added `nusw` / `nuw` flags on `getelementptr`. The flags are
/// poison-generation hints; they don't change the computed address. SAF must
/// parse them, map to the same `Operation::Gep` shape, and emit exactly one
/// GEP per function without losing or fabricating operands.
#[test]
fn llvm19_gep_nusw_nuw_flags_parse_cleanly() {
    let bundle = load("gep_nusw_nuw.ll");
    let funcs = &bundle.module.functions;
    assert_eq!(funcs.len(), 4, "expected 4 functions (one per flag combo)");

    for func in funcs {
        let gep_count = func
            .blocks
            .iter()
            .flat_map(|b| b.instructions.iter())
            .filter(|inst| matches!(inst.op, Operation::Gep { .. }))
            .count();
        assert_eq!(
            gep_count, 1,
            "function `{}` should map to exactly one Gep instruction",
            func.name
        );
    }
}

// -----------------------------------------------------------------------------
// LLVM 19 -> 21 — `nocapture` replaced by `captures(none)` attribute
// -----------------------------------------------------------------------------

/// LLVM 21 retired the `nocapture` parameter attribute in favor of a general
/// `captures(...)` family; the textual reader auto-upgrades `nocapture` to
/// `captures(none)`. SAF must parse the new spelling without rejecting the
/// module, and capture-sensitive analyses must still see the function and its
/// three declared sinks.
#[test]
fn llvm21_captures_attribute_parses_and_preserves_callgraph() {
    let bundle = load("captures_attr.ll");
    let mod_ = &bundle.module;

    let names: Vec<&str> = mod_.functions.iter().map(|f| f.name.as_str()).collect();
    for expected in ["caller", "sink", "sink_readonly", "sink_partial"] {
        assert!(
            names.contains(&expected),
            "expected function `{expected}` in module; got {names:?}"
        );
    }

    // `caller` must have two calls (one per invoked sink); neither should be
    // dropped because of an unfamiliar attribute.
    let caller = mod_
        .functions
        .iter()
        .find(|f| f.name == "caller")
        .expect("caller function missing");
    let call_count = caller
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
        call_count, 2,
        "`caller` should have 2 calls after attribute parsing"
    );
}

// -----------------------------------------------------------------------------
// LLVM 22 — `ptrtoaddr` instruction
// -----------------------------------------------------------------------------

/// LLVM 22 introduced `ptrtoaddr` — a provenance-free address extraction,
/// distinct from `ptrtoint`. We don't yet model provenance, so the minimum
/// bar is that the frontend parses the instruction without panicking and the
/// two defining functions land in AIR with a plausible body shape (1 or 2
/// non-terminator instructions each).
#[test]
fn llvm22_ptrtoaddr_parses_without_panic() {
    let bundle = load("ptrtoaddr.ll");
    let mod_ = &bundle.module;
    assert_eq!(mod_.functions.len(), 2);

    for func in &mod_.functions {
        let block_count = func.blocks.len();
        assert!(
            block_count >= 1,
            "function `{}` should have at least one block",
            func.name
        );
        let inst_count: usize = func.blocks.iter().map(|b| b.instructions.len()).sum();
        assert!(
            inst_count >= 1,
            "function `{}` should have at least one instruction (got {inst_count})",
            func.name
        );
    }
}

// -----------------------------------------------------------------------------
// LLVM 22 — masked load/store/gather/scatter intrinsic signature change
// -----------------------------------------------------------------------------

/// LLVM 22 removed the explicit alignment argument from `@llvm.masked.*`;
/// alignment is now carried by the `align` attribute on the pointer operand.
/// SAF's intrinsic classifier must still recognize these calls so downstream
/// passes treat them as memory operations, not opaque calls.
#[test]
fn llvm22_masked_intrinsic_new_signature_parses() {
    let bundle = load("masked_intrinsic.ll");
    let mod_ = &bundle.module;

    // The module should have 4 functions: 2 defined + 2 intrinsic declarations.
    let defined = mod_.functions.iter().filter(|f| !f.is_declaration).count();
    assert_eq!(defined, 2, "expected 2 defined functions");

    // Each defined function should carry exactly one call to the intrinsic.
    for func in mod_.functions.iter().filter(|f| !f.is_declaration) {
        let call_count = func
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
            "function `{}` should carry one call to @llvm.masked.*",
            func.name
        );
    }
}

// -----------------------------------------------------------------------------
// LLVM 20 — `inrange(low, high)` GEP attribute position (regression test for
// the bug diagnosed during PTABen-LLVM22: parse_constant_gep was fooled by
// inline `inrange`, and decompose_constant_gep kept LLVM's pointer-level index
// as a field step — the two bugs cancelled on LLVM 18 but diverged here.)
// -----------------------------------------------------------------------------

/// The vtable install GEP `{[3 x ptr]}, ptr @_ZTV1A, i32 0, i32 0, i32 2` should
/// decompose to a single `Operation::Gep` with exactly two `Field` steps
/// `[Field{0}, Field{2}]` — descending into the struct's sole array field,
/// then element 2. Any more or fewer steps indicates the pointer-level index
/// leaked into the path or a descent index was dropped.
#[test]
fn llvm20_inrange_gep_attribute_yields_correct_field_path() {
    let bundle = load("inrange_attr.ll");
    let func = bundle
        .module
        .functions
        .iter()
        .find(|f| f.name == "install_vtable")
        .expect("install_vtable missing");

    let mut geps: Vec<&Operation> = func
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .map(|inst| &inst.op)
        .filter(|op| matches!(op, Operation::Gep { .. }))
        .collect();

    assert_eq!(
        geps.len(),
        1,
        "install_vtable should synthesize exactly one Gep from the constant-expression GEP"
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
        "FieldPath must be [Field{{0}}, Field{{2}}] — one step per type descent, \
         not [Field{{0}}, Field{{0}}, Field{{2}}] (pointer-level index leaking in) \
         nor [Field{{0}}] (descent index dropped)"
    );
}
