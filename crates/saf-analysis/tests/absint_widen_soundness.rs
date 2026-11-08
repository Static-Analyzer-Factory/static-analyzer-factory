//! Widening at a loop header must not copy a back-edge interval onto a value
//! defined outside the loop.
//!
//! Fixture `fixtures/absint/param_bound_back_edge.json` (`saf index` output) is the AIR of
//! `sv-benchmarks/c/ldv-memsafety/ArraysWithLenghtAtDeclaration.c` (ILP32, after
//! mem2reg):
//!
//! ```c
//! int foo(char b[32], int size) {
//!     char a[32]; int i;
//!     for (i = 0; i <= size; i++) a[i] = b[i];
//!     return i;
//! }
//! ```
//!
//! `main` calls `foo(mask, 0)` first, so `size == 0` at the loop's first entry.
//! The parameter enters the header as ⊤. The first widening dropped the ⊤, so
//! the second back edge looked like the first sight of `size` and copied its
//! `[1, MAX]` (`i <= size` after `i++`). The correctness witness then stated
//! `1 <= size` at the loop head, and ESBMC, `CPAchecker` and `UAutomizer` all
//! refuted it.

use saf_analysis::absint::{AbstractInterpConfig, solve_abstract_interp};
use saf_core::air::{AirBundle, Operation};

#[test]
fn parameter_keeps_entry_top_at_loop_header() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/absint/param_bound_back_edge.json"
    );
    let bundle: AirBundle =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read fixture"))
            .expect("parse fixture");
    let module = bundle.module;
    let result = solve_abstract_interp(&module, &AbstractInterpConfig::default());
    assert!(result.diagnostics().converged);

    let foo = module
        .functions
        .iter()
        .find(|f| f.name == "foo")
        .expect("fixture defines foo");
    let size = foo
        .params
        .iter()
        .find(|p| p.name.as_deref() == Some("size"))
        .expect("foo has a parameter `size`")
        .id;
    let header = foo
        .blocks
        .iter()
        .find(|b| b.label.as_deref() == Some("for.cond"))
        .expect("foo has the loop header for.cond");
    // The loop-head state the witness reads: before the first non-phi instruction.
    let read = header
        .instructions
        .iter()
        .find(|i| !matches!(i.op, Operation::Phi { .. }))
        .expect("header has a non-phi instruction");

    let states = result.invariants_at_inst(read.id);
    if let Some(iv) = states.get(&size) {
        assert!(
            iv.contains(0),
            "`size` is 0 on the first entry, but the header has [{}, {}]",
            iv.lo(),
            iv.hi()
        );
    }
}
