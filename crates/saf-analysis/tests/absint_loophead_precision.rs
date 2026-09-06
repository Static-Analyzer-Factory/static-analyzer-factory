//! Regression tests for Reason-1 / item 1c: the interval absint must expose a
//! SOUND, non-trivial loop-head interval on the NAMED loop variable.
//!
//! Two bugs these lock in (see plans/208):
//!   * Bug A (precision): the loop body was analysed exactly once (the empty (=⊤)
//!     loop-header entry made the change-gate reject the back-edge refinement), so
//!     the counter never ascended and read its entry value [0,0].
//!   * Bug B (soundness): the phi read a reached-but-absent incoming as ⊥, so an
//!     overflowing accumulator collapsed to the entry constant [0,0] (an
//!     UNDER-approximation).

use std::collections::BTreeMap;

use saf_analysis::absint::{
    AbstractInterpConfig, Interval, detect_loop_headers, solve_abstract_interp,
};
use saf_analysis::cfg::Cfg;
use saf_core::air::{AirModule, Operation};
use saf_core::ids::ValueId;
use saf_test_utils::load_ll_fixture;

/// dst/param debug symbol -> recovered C name (mirrors the 1b driver).
fn value_names(module: &AirModule) -> BTreeMap<ValueId, String> {
    let mut names = BTreeMap::new();
    for func in &module.functions {
        for param in &func.params {
            if let Some(n) = &param.name {
                names.insert(param.id, n.clone());
            }
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let (Some(dst), Some(sym)) = (inst.dst, &inst.symbol) {
                    names.insert(dst, sym.display_name.clone());
                }
            }
        }
    }
    names
}

/// The interval of the loop-header PHI whose dst recovers the C name `var`, read
/// at the first non-phi instruction (exactly where the 1b driver reads it).
fn loop_head_phi_interval(fixture: &str, var: &str) -> Interval {
    let module = load_ll_fixture(fixture);
    let result = solve_abstract_interp(&module, &AbstractInterpConfig::default());
    assert!(
        result.diagnostics().converged,
        "{fixture}: absint must converge (convergence gate)"
    );
    let names = value_names(&module);
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let cfg = Cfg::build(func);
        let headers = detect_loop_headers(&cfg);
        for block in &func.blocks {
            if !headers.contains(block.id) {
                continue;
            }
            let Some(read) = block
                .instructions
                .iter()
                .find(|i| !matches!(i.op, Operation::Phi { .. }))
            else {
                continue;
            };
            let inv = result.invariants_at_inst(read.id);
            for inst in &block.instructions {
                if !matches!(inst.op, Operation::Phi { .. }) {
                    continue;
                }
                let Some(dst) = inst.dst else { continue };
                if names.get(&dst).map(String::as_str) == Some(var) {
                    return inv.get(&dst).cloned().unwrap_or_else(|| {
                        panic!("{fixture}: phi `{var}` absent from read state")
                    });
                }
            }
        }
    }
    panic!("{fixture}: no loop-header phi named `{var}`");
}

/// Slice 2 (Bug A / precision): a counted `while (i < 1000000) i++;` loop must
/// expose the TIGHT, sound loop-head interval on the named counter.
#[test]
fn spike_counter_loop_head_i_is_tight() {
    let i = loop_head_phi_interval("spike_counter", "i");
    assert_eq!(
        (i.lo(), i.hi()),
        (0, 1_000_000),
        "counter `i` must read the tight sound [0,1000000] at the loop head, got [{}, {}]",
        i.lo(),
        i.hi()
    );
}

/// Slice 2 (Bug A / precision): a `for (i=0; i<100; i++)` counter -> [0,100].
#[test]
fn dbgvalue_loop_head_i_is_tight() {
    let i = loop_head_phi_interval("dbgvalue_promoted_phi", "i");
    assert_eq!(
        (i.lo(), i.hi()),
        (0, 100),
        "counter `i` must read [0,100] at the loop head, got [{}, {}]",
        i.lo(),
        i.hi()
    );
}

/// Slice 1 (Bug B / soundness): the accumulator `s = sum(0..i)` reaches 4950 at
/// the head; the sound absint must OVER-approximate (⊤ without relational
/// reasoning), never under-approximate to the entry constant [0,0].
#[test]
fn dbgvalue_accumulator_s_is_sound_overapprox() {
    let s = loop_head_phi_interval("dbgvalue_promoted_phi", "s");
    assert!(
        s.lo() <= 0 && s.hi() >= 4950,
        "accumulator `s` must be a sound superset of its true range [0,4950] \
         (never the under-approximation [0,0]), got [{}, {}]",
        s.lo(),
        s.hi()
    );
}
