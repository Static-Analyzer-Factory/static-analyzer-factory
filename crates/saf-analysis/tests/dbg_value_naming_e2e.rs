//! E2E: promoted loop-head phis recover their C names via `llvm.dbg.value`
//! (plan 207 / 1b.0).
//!
//! Fixture source: `tests/programs/c/dbgvalue_promoted_phi.c`, compiled with the
//! `saf verify` recipe (mem2reg'd, `-g`). After promotion the loop variables
//! `s`/`i` exist only as SSA phis; their C names live in `llvm.dbg.value` +
//! `!DILocalVariable`, which the frontend now parses (previously only
//! `dbg.declare`, which mem2reg deletes).

use saf_core::air::Operation;
use saf_test_utils::load_ll_fixture;

#[test]
fn promoted_loop_phis_are_named_from_dbg_value() {
    let module = load_ll_fixture("dbgvalue_promoted_phi");

    let phi_names: Vec<String> = module
        .functions
        .iter()
        .flat_map(|f| f.blocks.iter())
        .flat_map(|b| b.instructions.iter())
        .filter(|i| matches!(i.op, Operation::Phi { .. }))
        .filter_map(|i| i.symbol.as_ref().map(|s| s.display_name.clone()))
        .collect();

    assert!(
        phi_names.iter().any(|n| n == "s"),
        "promoted loop-carried phi for `s` should be named from dbg.value; named phis = {phi_names:?}"
    );
    assert!(
        phi_names.iter().any(|n| n == "i"),
        "promoted loop-counter phi for `i` should be named from dbg.value; named phis = {phi_names:?}"
    );
}
