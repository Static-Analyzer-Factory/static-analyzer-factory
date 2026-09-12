//! Driver: turn SAF's synthesized linear ranking functions into a YAML-**2.1**
//! correctness witness of `loop_transition_invariant` entries.
//!
//! The TRUE-side counterpart to [`crate::correctness_driver`], which emits 2.0
//! *state* invariants from the interval fixpoint. **Emits no verdict** — the
//! termination proof is `ranking::rank_loops` / `program_structurally_terminates`
//! and has already happened by the time this runs. This layer only renders it.
//!
//! ## Why a transition invariant
//!
//! Format 2.1 has no `ranking_function` key. A termination argument is instead
//! *encoded into* an invariant of `type: loop_transition_invariant`, whose value is
//! an `ext_c_expression` relating the current state to any previous state at the
//! same loop head via the `\at(x, AnyPrev)` keyword. So a ranking function
//! `f(x) = Σ cᵥ·v` becomes the relation
//!
//! ```text
//! c₁*\at(v₁, AnyPrev) + c₂*\at(v₂, AnyPrev) > c₁*v₁ + c₂*v₂
//! ```
//!
//! The additive constant cancels on both sides and is dropped. The spec requires
//! that every `\at(_, AnyPrev)` in one expression be substituted from the SAME
//! previous state, which is exactly the semantics a ranking function needs.
//!
//! ## Gates (all fail-closed)
//!
//! Every gate drops the WITNESS, never the verdict — an unconfirmed-but-correct
//! TRUE scores 0, never −32, so abstaining here is free and guessing is not.
//!
//! * `\at(x, AnyPrev)` requires `x` to be a **variable**, so every symbol carrying
//!   a non-zero coefficient must recover a C identifier. One anonymous SSA value
//!   drops the whole loop.
//! * A loop whose header performs a call (`while (f())`) has no matchable keyword
//!   column — abstain, as [`crate::correctness_driver`] already does.
//! * Lexicographic and disjunctive-SCC ranks are not single transition invariants;
//!   they arrive as [`Ranked::Opaque`] and are skipped.
//! * OpenMP source: the frontend drops `#pragma omp`, so the AIR silently loses
//!   parallelism — module-level abstain, same as the 2.0 driver.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use saf_analysis::cfg::Cfg;
use saf_core::air::AirModule;
use saf_core::ids::ValueId;

use crate::correctness_driver::{
    build_value_names, header_has_function_call, is_c_identifier, loop_head_location,
    source_has_openmp,
};
use crate::correctness_witness::{InvariantKind, InvariantSetWitness, SourceInvariant};
use crate::ranking::{Ranked, RankingFunction, rank_loops};
use crate::witness_yaml::WitnessMeta;

/// Render one side of the transition relation: `Σ cᵥ·name(v)`, with `\at(v,
/// AnyPrev)` substituted for each variable when `previous` is set.
///
/// Returns `None` if any coefficient-bearing symbol has no C name — the relation
/// would be unsound to state with a symbol missing, and `\at` needs a variable.
fn render_side(
    f: &RankingFunction,
    names: &BTreeMap<ValueId, String>,
    previous: bool,
) -> Option<String> {
    let mut out = String::new();
    for (v, &c) in &f.terms {
        let name = names.get(v).filter(|n| is_c_identifier(n))?;
        let term = if previous {
            format!("\\at({name}, AnyPrev)")
        } else {
            name.clone()
        };
        // Fold the sign into the joining operator so the expression reads like C
        // rather than accumulating `+ -3*x`.
        let (op, mag) = if c < 0 {
            (" - ", c.unsigned_abs())
        } else {
            (" + ", c.unsigned_abs())
        };
        if out.is_empty() {
            if c < 0 {
                out.push('-');
            }
        } else {
            out.push_str(op);
        }
        if mag == 1 {
            out.push_str(&term);
        } else {
            let _ = write!(out, "{mag}*{term}");
        }
    }
    (!out.is_empty()).then_some(out)
}

/// The `ext_c_expression` for a SCALAR ranking function: the previous state's rank
/// is strictly greater than the current one.
///
/// The constant is deliberately absent — it appears identically on both sides of
/// `>` and cancels, so including it would only widen the expression.
fn scalar_relation(f: &RankingFunction, names: &BTreeMap<ValueId, String>) -> Option<String> {
    let prev = render_side(f, names, true)?;
    let cur = render_side(f, names, false)?;
    Some(format!("{prev} > {cur}"))
}

/// The `ext_c_expression` for a LEXICOGRAPHIC tuple `(f0 .. fk)`, most-significant
/// component first:
///
/// ```text
/// f0p > f0 || (f0p >= f0 && (f1p > f1 || (f1p >= f1 && f2p > f2)))
/// ```
///
/// `>=` on the higher components, **not** `==`. Each greedy round proves its `f`
/// only NON-INCREASING on the branches still remaining, so `==` would claim
/// something strictly stronger than the synthesis established. The disjunction
/// above states exactly what was proven, branch by branch: the branch retired at
/// round `i` satisfies `(AND over j<i of fjp >= fj) && fip > fi`.
///
/// Well-founded because every component is bounded below (requirement A of its own
/// round), so the tuple ranges over a product of naturals under a non-increasing
/// prefix.
fn lex_relation(fs: &[RankingFunction], names: &BTreeMap<ValueId, String>) -> Option<String> {
    let (last, head) = fs.split_last()?;
    // Build outward from the least-significant component.
    let mut acc = scalar_relation(last, names)?;
    for f in head.iter().rev() {
        let prev = render_side(f, names, true)?;
        let cur = render_side(f, names, false)?;
        acc = format!("{prev} > {cur} || ({prev} >= {cur} && ({acc}))");
    }
    Some(acc)
}

/// The `ext_c_expression` witnessing whatever ranked this loop, or `None` when the
/// proof shape is not expressible as a single transition invariant.
fn transition_invariant(r: &Ranked, names: &BTreeMap<ValueId, String>) -> Option<String> {
    match r.components()? {
        [] => None,
        [f] => scalar_relation(f, names),
        fs => lex_relation(fs, names),
    }
}

/// Build a 2.1 `invariant_set` of `loop_transition_invariant` entries from the
/// ranking functions `module`'s loops were proven with, or `None` if no loop
/// yields a renderable one.
///
/// `source` is the original C text (for the OpenMP gate and the loop-keyword
/// column); `meta` supplies task metadata. A `None` here is not a proof failure —
/// the caller should fall back to an empty 2.1 `invariant_set`.
#[must_use]
pub fn build_ranking_witness(
    module: &AirModule,
    source: &str,
    meta: &WitnessMeta,
) -> Option<InvariantSetWitness> {
    // Module-level OpenMP abstain (the AIR silently dropped any parallelism).
    if source_has_openmp(source) {
        return None;
    }

    let names = build_value_names(module);
    let src_lines: Vec<&str> = source.lines().collect();

    let mut invariants: Vec<SourceInvariant> = Vec::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let cfg = Cfg::build(func);
        // Only ask for rankings where a loop exists; a loop-free function has
        // nothing to witness and `rank_loops` would return an empty vector.
        let Some(rankings) = rank_loops(func, module, &cfg) else {
            continue;
        };
        for lr in rankings {
            // The disjunctive-SCC and recursion routes are not expressible as a
            // single transition invariant and arrive opaque.
            if lr.ranked.components().is_none() {
                continue;
            }
            let Some(block) = func.blocks.iter().find(|b| b.id == lr.header) else {
                continue;
            };
            // Function-call loop guard (`while (f())`): CPAchecker matches no
            // column and the omitted-column workaround crashes it — abstain.
            if header_has_function_call(block) {
                continue;
            }
            let Some((file_name, line, column)) = loop_head_location(module, block, &src_lines)
            else {
                continue;
            };
            let Some(value) = transition_invariant(&lr.ranked, &names) else {
                continue;
            };
            invariants.push(SourceInvariant {
                file_name,
                line,
                column,
                function: Some(func.name.clone()),
                value,
                kind: InvariantKind::LoopTransition,
            });
        }
    }

    if invariants.is_empty() {
        return None;
    }
    InvariantSetWitness::assemble(meta, &invariants).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(pairs: &[(u128, &str)]) -> BTreeMap<ValueId, String> {
        pairs
            .iter()
            .map(|&(id, n)| (ValueId::new(id), n.to_string()))
            .collect()
    }

    fn rank(terms: &[(u128, i64)], constant: i64) -> RankingFunction {
        RankingFunction {
            terms: terms.iter().map(|&(id, c)| (ValueId::new(id), c)).collect(),
            constant,
        }
    }

    #[test]
    fn single_variable_unit_coefficient_matches_the_spec_example() {
        // The sv-witnesses reference termination witness is exactly this shape.
        let f = rank(&[(1, 1)], 0);
        assert_eq!(
            transition_invariant(&Ranked::With(f.clone()), &names(&[(1, "i")])).unwrap(),
            "\\at(i, AnyPrev) > i"
        );
    }

    #[test]
    fn coefficient_greater_than_one_is_multiplied_out() {
        let f = rank(&[(1, 3)], 0);
        assert_eq!(
            transition_invariant(&Ranked::With(f.clone()), &names(&[(1, "n")])).unwrap(),
            "3*\\at(n, AnyPrev) > 3*n"
        );
    }

    #[test]
    fn negative_coefficient_renders_as_subtraction() {
        // f = x - y: the leading sign is inlined, the second term folds into `-`.
        let f = rank(&[(1, 1), (2, -1)], 0);
        let got =
            transition_invariant(&Ranked::With(f.clone()), &names(&[(1, "x"), (2, "y")])).unwrap();
        assert_eq!(got, "\\at(x, AnyPrev) - \\at(y, AnyPrev) > x - y");
    }

    #[test]
    fn leading_negative_coefficient_keeps_its_sign() {
        let f = rank(&[(1, -2)], 0);
        assert_eq!(
            transition_invariant(&Ranked::With(f.clone()), &names(&[(1, "k")])).unwrap(),
            "-2*\\at(k, AnyPrev) > -2*k"
        );
    }

    #[test]
    fn the_constant_is_dropped_because_it_cancels() {
        let a =
            transition_invariant(&Ranked::With(rank(&[(1, 1)], 0)), &names(&[(1, "i")])).unwrap();
        let b =
            transition_invariant(&Ranked::With(rank(&[(1, 1)], 99)), &names(&[(1, "i")])).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn an_anonymous_symbol_abstains_the_whole_loop() {
        // `\at(x, AnyPrev)` needs a variable; a `%3f`-style temporary is not one,
        // and silently omitting its term would state a DIFFERENT relation.
        let f = rank(&[(1, 1), (2, 1)], 0);
        assert!(transition_invariant(&Ranked::With(f.clone()), &names(&[(1, "i")])).is_none());
        assert!(
            transition_invariant(&Ranked::With(f.clone()), &names(&[(1, "i"), (2, "%3f")]))
                .is_none()
        );
    }

    #[test]
    fn a_symbolless_ranking_function_renders_nothing() {
        assert!(transition_invariant(&Ranked::With(rank(&[], 5)), &names(&[])).is_none());
    }

    #[test]
    fn two_component_lex_tuple_nests_one_level() {
        let fs = vec![rank(&[(1, 1)], 0), rank(&[(2, 1)], 0)];
        let got = transition_invariant(&Ranked::Lex(fs), &names(&[(1, "x"), (2, "y")])).unwrap();
        assert_eq!(
            got,
            "\\at(x, AnyPrev) > x || (\\at(x, AnyPrev) >= x && (\\at(y, AnyPrev) > y))"
        );
    }

    #[test]
    fn three_component_lex_tuple_nests_twice_most_significant_first() {
        let fs = vec![rank(&[(1, 1)], 0), rank(&[(2, 1)], 0), rank(&[(3, 1)], 0)];
        let got = transition_invariant(&Ranked::Lex(fs), &names(&[(1, "a"), (2, "b"), (3, "c")]))
            .unwrap();
        // `a` is the outermost (most significant) component, `c` the innermost.
        assert_eq!(
            got,
            "\\at(a, AnyPrev) > a || (\\at(a, AnyPrev) >= a \
             && (\\at(b, AnyPrev) > b || (\\at(b, AnyPrev) >= b && (\\at(c, AnyPrev) > c))))"
        );
    }

    #[test]
    fn lex_uses_non_strict_on_the_prefix_because_that_is_what_was_proven() {
        // A greedy round proves its f NON-INCREASING on the remaining branches.
        // Rendering `==` would overclaim.
        let fs = vec![rank(&[(1, 1)], 0), rank(&[(2, 1)], 0)];
        let got = transition_invariant(&Ranked::Lex(fs), &names(&[(1, "x"), (2, "y")])).unwrap();
        assert!(got.contains(">="), "{got}");
        assert!(!got.contains("=="), "{got}");
    }

    #[test]
    fn a_single_component_lex_tuple_is_just_the_scalar_relation() {
        let one = transition_invariant(&Ranked::Lex(vec![rank(&[(1, 1)], 0)]), &names(&[(1, "i")]));
        let scalar = transition_invariant(&Ranked::With(rank(&[(1, 1)], 0)), &names(&[(1, "i")]));
        assert_eq!(one, scalar);
    }

    #[test]
    fn an_opaque_ranking_renders_nothing() {
        assert!(transition_invariant(&Ranked::Opaque, &names(&[(1, "i")])).is_none());
    }

    #[test]
    fn one_anonymous_symbol_anywhere_abstains_the_whole_lex_tuple() {
        // A tuple missing a component states a DIFFERENT relation, so a single
        // unnameable symbol must drop the entry rather than the component.
        let fs = vec![rank(&[(1, 1)], 0), rank(&[(2, 1)], 0)];
        assert!(transition_invariant(&Ranked::Lex(fs), &names(&[(1, "x")])).is_none());
    }

    #[test]
    fn multi_variable_sides_stay_in_sync() {
        let f = rank(&[(1, 1), (2, 2)], 0);
        let got =
            transition_invariant(&Ranked::With(f.clone()), &names(&[(1, "a"), (2, "b")])).unwrap();
        let (prev, cur) = got.split_once(" > ").unwrap();
        // Same shape on both sides, differing only by the \at wrapper.
        assert_eq!(prev.matches("\\at(").count(), 2);
        assert!(!cur.contains("\\at("));
        assert_eq!(prev.matches('*').count(), cur.matches('*').count());
    }
}
