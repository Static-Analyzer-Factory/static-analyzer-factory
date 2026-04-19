//! Regression tests for constant-expression GEP decomposition in the LLVM
//! frontend. Lives in saf-analysis/tests/ so it exercises the full
//! frontend → AIR pipeline via the shared `load_ll_bundle` helper.

use saf_core::air::Operation;
use saf_test_utils::load_ll_bundle;

/// A single-index constant GEP (`getelementptr T, ptr @g, i64 K`) has no
/// type descent — the index is purely pointer-level. The frontend must not
/// synthesize a `Gep` instruction for it, because SAF's `FieldPath` can't
/// represent a pointer-level offset and emitting a phantom one-step path
/// would lie about the semantics.
///
/// The fixture stores a single-index constant GEP into `%dst`; the
/// resulting `consume_single_gep` AIR must contain exactly one `Store`
/// and zero `Gep` instructions.
#[test]
fn single_index_constant_gep_does_not_synthesize_phantom_gep() {
    let bundle = load_ll_bundle("single_index_const_gep");
    let func = bundle
        .module
        .functions
        .iter()
        .find(|f| f.name == "consume_single_gep")
        .expect("consume_single_gep missing from module");

    let (gep_count, store_count) = func.blocks.iter().flat_map(|b| b.instructions.iter()).fold(
        (0usize, 0usize),
        |(g, s), inst| match inst.op {
            Operation::Gep { .. } => (g + 1, s),
            Operation::Store => (g, s + 1),
            _ => (g, s),
        },
    );

    assert_eq!(
        gep_count, 0,
        "single-index constant GEP must not synthesize a Gep instruction \
         (would imply a bogus type descent)"
    );
    assert_eq!(
        store_count, 1,
        "expected exactly one store in consume_single_gep"
    );
}
