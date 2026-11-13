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
    build_value_names, c_value_literal, header_has_function_call, header_variable_names,
    is_c_identifier, leftmost_non_ws_column, loop_head_location, source_has_openmp,
};
use crate::correctness_witness::{InvariantKind, InvariantSetWitness, SourceInvariant};
use crate::ranking::{Ranked, RankingFunction, rank_loops};
use crate::unreach_prove::{Lasso, StateVar};
use crate::witness_yaml::{
    Action, Constraint, SourceWaypoint, ViolationWitness, WaypointKind, WitnessMeta,
};

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

/// A 2.1 termination witness: the synthesised ranking function's transition
/// invariant for every loop [`rank_loops`] ranked, and for every other loop a
/// candidate read off its exit guard — a loop staying while `a > b` (or `>=`)
/// is witnessed by `\at(a - b, AnyPrev) > a - b`, one staying while `a < b`
/// by `b - a` decreasing. The guard candidate is a validator HINT, not part of
/// any proof: the verdict never depends on it, and a candidate the validator
/// rejects costs what the empty `invariant_set` costs (an unconfirmed TRUE).
/// `None` when no loop yields an entry.
#[must_use]
pub fn build_termination_witness(
    module: &AirModule,
    source: &str,
    meta: &WitnessMeta,
) -> Option<InvariantSetWitness> {
    if source_has_openmp(source) {
        return None;
    }
    let names = build_value_names(module);
    let globals: BTreeMap<ValueId, &str> = module
        .globals
        .iter()
        .map(|g| (g.id, g.name.as_str()))
        .collect();
    let src_lines: Vec<&str> = source.lines().collect();
    let mut invariants: Vec<SourceInvariant> = Vec::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let cfg = Cfg::build(func);
        let mut ranked: BTreeMap<saf_core::ids::BlockId, String> = BTreeMap::new();
        if let Some(rankings) = rank_loops(func, module, &cfg) {
            for lr in rankings {
                if let Some(v) = lr
                    .ranked
                    .components()
                    .and(transition_invariant(&lr.ranked, &names))
                {
                    ranked.insert(lr.header, v);
                }
            }
        }
        let headers = saf_analysis::absint::detect_loop_headers(&cfg);
        for block in &func.blocks {
            if !headers.contains(block.id) || header_has_function_call(block) {
                continue;
            }
            let value = match ranked.get(&block.id) {
                Some(v) => Some(v.clone()),
                None => guard_relation(module, func, &cfg, block, &names, &globals),
            };
            let Some(value) = value else { continue };
            let Some((file_name, line, column)) = loop_head_location(module, block, &src_lines)
            else {
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

/// The guard candidate of [`build_termination_witness`] for the loop headed by
/// `block`, when the header branches on an integer comparison with exactly one
/// successor that leads back to the header and both operands render as C.
fn guard_relation(
    module: &AirModule,
    func: &saf_core::air::AirFunction,
    cfg: &Cfg,
    block: &saf_core::air::AirBlock,
    names: &BTreeMap<ValueId, String>,
    globals: &BTreeMap<ValueId, &str>,
) -> Option<String> {
    use saf_core::air::{BinaryOp, Operation};
    let t = block.instructions.last()?;
    let Operation::CondBr {
        then_target,
        else_target,
    } = t.op
    else {
        return None;
    };
    let reaches = |from: saf_core::ids::BlockId| {
        let mut seen = std::collections::BTreeSet::new();
        let mut stack = vec![from];
        while let Some(b) = stack.pop() {
            if b == block.id {
                return true;
            }
            if seen.insert(b) {
                if let Some(ss) = cfg.successors.get(&b) {
                    stack.extend(ss.iter().copied());
                }
            }
        }
        false
    };
    let stay_then = match (reaches(then_target), reaches(else_target)) {
        (true, false) => true,
        (false, true) => false,
        _ => return None,
    };
    let c = *t.operands.first()?;
    let defs: BTreeMap<ValueId, &saf_core::air::Instruction> = func
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter_map(|i| i.dst.map(|d| (d, i)))
        .collect();
    let ci = defs.get(&c)?;
    let Operation::BinaryOp { kind } = ci.op else {
        return None;
    };
    let [a, b] = ci.operands[..] else { return None };
    // The comparison that holds while the loop goes on.
    let kind = if stay_then {
        kind
    } else {
        match kind {
            BinaryOp::ICmpSlt => BinaryOp::ICmpSge,
            BinaryOp::ICmpSle => BinaryOp::ICmpSgt,
            BinaryOp::ICmpSgt => BinaryOp::ICmpSle,
            BinaryOp::ICmpSge => BinaryOp::ICmpSlt,
            _ => return None,
        }
    };
    let (hi, lo) = match kind {
        BinaryOp::ICmpSgt | BinaryOp::ICmpSge => (a, b),
        BinaryOp::ICmpSlt | BinaryOp::ICmpSle => (b, a),
        _ => return None,
    };
    let (hc, hp) = render_operand(module, &defs, names, globals, hi, 0)?;
    let (lc, lp) = render_operand(module, &defs, names, globals, lo, 0)?;
    // Constants cancel; the relation must mention at least one variable.
    if hc == hp && lc == lp {
        return None;
    }
    Some(format!("({hp}) - ({lp}) > ({hc}) - ({lc})"))
}

/// `(current, previous)` C renderings of an integer operand: a constant, a
/// named SSA value, a sign/zero extension of one, or a load of a named global.
fn render_operand(
    module: &AirModule,
    defs: &BTreeMap<ValueId, &saf_core::air::Instruction>,
    names: &BTreeMap<ValueId, String>,
    globals: &BTreeMap<ValueId, &str>,
    v: ValueId,
    depth: u32,
) -> Option<(String, String)> {
    use saf_core::air::{CastKind, Constant, Operation};
    if depth > 4 {
        return None;
    }
    if let Some(Constant::Int { value, .. }) = module.constants.get(&v) {
        let s = value.to_string();
        return Some((s.clone(), s));
    }
    if let Some(n) = names.get(&v).filter(|n| is_c_identifier(n)) {
        return Some((n.clone(), format!("\\at({n}, AnyPrev)")));
    }
    let d = defs.get(&v)?;
    match &d.op {
        Operation::Cast {
            kind: CastKind::SExt | CastKind::ZExt,
            ..
        } => render_operand(
            module,
            defs,
            names,
            globals,
            *d.operands.first()?,
            depth + 1,
        ),
        Operation::Load => {
            // A named global. (Not a cell through a pointer: CPAchecker 4.2.2's
            // termination validator throws on `\at(*p, AnyPrev)`.)
            let g = globals.get(d.operands.first()?)?;
            is_c_identifier(g).then(|| ((*g).to_string(), format!("\\at({g}, AnyPrev)")))
        }
        _ => None,
    }
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

    fn lasso_on(path: &[u128], start: usize) -> Lasso {
        use saf_core::ids::{BlockId, FunctionId};
        let f = FunctionId::new(1);
        Lasso {
            func: f,
            block: BlockId::new(10),
            state: Vec::new(),
            path: path.iter().map(|b| (f, BlockId::new(*b))).collect(),
            path_cycle_start: start,
            nondets: Vec::new(),
            cycle_start: 0,
            latches: Vec::new(),
        }
    }

    #[test]
    fn assumption_lasso_only_for_a_first_visit_one_round_cycle() {
        // entry, head, body, head: recurs at the first visit after one round.
        assert!(lasso_starts_at_first_visit_of_one_round(&lasso_on(
            &[1, 10, 11, 10],
            1
        )));
        // The recurring state is the head's second visit (`or-05`).
        assert!(!lasso_starts_at_first_visit_of_one_round(&lasso_on(
            &[1, 10, 11, 10, 11, 10],
            3
        )));
        // Two rounds (`Swingers`).
        assert!(!lasso_starts_at_first_visit_of_one_round(&lasso_on(
            &[1, 10, 11, 10, 11, 10],
            1
        )));
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

/// The format-2.1 `termination` violation witness of a [`Lasso`]: a `follow`
/// assumption at the loop head that pins the recurring state (the header's own
/// integer phis and parameters, and the scalar globals), then the same
/// assumption as the `cycle` — the loop head is passed in that state forever.
/// A variable without a C name is left out, which only weakens both
/// assumptions. `None` (no witness; the verdict stands) when the loop head
/// cannot be placed on its loop keyword (from the header's branch, or from a
/// back edge's when the header performs a call), or the program uses OpenMP.
#[must_use]
pub fn build_nontermination_witness(
    module: &AirModule,
    source: &str,
    meta: &WitnessMeta,
    lasso: &Lasso,
) -> Option<ViolationWitness> {
    if source_has_openmp(source) {
        return None;
    }
    let func = module.function(lasso.func)?;
    let block = func.blocks.iter().find(|b| b.id == lasso.block)?;
    let src_lines: Vec<&str> = source.lines().collect();
    // The loop statement's location: the header's own branch (`while (c)`), or
    // else a back edge's (`while (1)`: the header is the body's first block).
    let header_loc = (!header_has_function_call(block))
        .then(|| loop_head_location(module, block, &src_lines))
        .flatten();
    let (file_name, line, column) = header_loc
        .or_else(|| {
            lasso.latches.iter().find_map(|l| {
                let b = func.blocks.iter().find(|b| b.id == *l)?;
                loop_head_location(module, b, &src_lines)
            })
        })
        .or_else(|| header_statement_location(module, block, &src_lines))?;
    let value_names = build_value_names(module);
    let header_phis: std::collections::BTreeSet<ValueId> = block
        .instructions
        .iter()
        .filter(|i| matches!(i.op, saf_core::air::Operation::Phi { .. }))
        .filter_map(|i| i.dst)
        .collect();
    let scalar = |t: Option<saf_core::ids::TypeId>| {
        !t.is_some_and(|t| module.is_pointer_type(t) || module.is_float_type(t))
    };
    let params: std::collections::BTreeSet<ValueId> = func
        .params
        .iter()
        .filter(|p| scalar(p.param_type))
        .map(|p| p.id)
        .collect();
    let global_names: BTreeMap<ValueId, &str> = module
        .globals
        .iter()
        .filter(|g| scalar(g.value_type))
        .map(|g| (g.id, g.name.as_str()))
        .collect();
    // A reassigned parameter's entry value is not the C variable at the loop
    // head (its header phi is), and a local shadows a global of the same name.
    let variables = header_variable_names(&header_phis, &params, &value_names);
    let local_names: std::collections::BTreeSet<&str> = variables.values().copied().collect();
    let mut conj: Vec<String> = Vec::new();
    for (var, v, bits) in &lasso.state {
        let name = match var {
            StateVar::Value(id) => variables.get(id).copied(),
            StateVar::Global(g) => global_names
                .get(g)
                .copied()
                .filter(|n| !local_names.contains(n)),
        };
        let Some(name) = name.filter(|n| is_c_identifier(n)) else {
            continue;
        };
        if *bits == 0 || *bits > 64 {
            continue;
        }
        let lit = c_value_literal(name, *v, *bits);
        if !conj.contains(&lit) {
            conj.push(lit);
        }
    }
    let value = if conj.is_empty() {
        "1".to_string()
    } else {
        format!("({})", conj.join(" && "))
    };
    let at_head = |action: Action| SourceWaypoint {
        kind: WaypointKind::Assumption,
        action,
        file_name: file_name.clone(),
        line,
        column: Some(column),
        function: Some(func.name.clone()),
        constraint: Some(Constraint {
            format: Some("c_expression".to_string()),
            value: value.clone(),
        }),
    };
    if !lasso_starts_at_first_visit_of_one_round(lasso) {
        if let Some(w) = branching_lasso(module, meta, lasso) {
            return Some(w);
        }
    }
    ViolationWitness::assemble_lasso(meta, &[at_head(Action::Follow)], &[at_head(Action::Cycle)])
        .ok()
}

/// Does the lasso's recurring state sit at the header's FIRST visit, with a
/// cycle of one round? Then the loop-head assumption holds at every pass of
/// the head, which is what validators check of an assumption lasso. Otherwise
/// (`or-05`: the first pass has `y == 0`; `Swingers`: two alternating states)
/// they reject it, and the witness lists the branches taken instead.
fn lasso_starts_at_first_visit_of_one_round(lasso: &Lasso) -> bool {
    let head = (lasso.func, lasso.block);
    let first = lasso.path.iter().position(|p| *p == head);
    let rounds = lasso
        .path
        .get(lasso.path_cycle_start + 1..)
        .map_or(0, |c| c.iter().filter(|p| **p == head).count());
    first == Some(lasso.path_cycle_start) && rounds == 1
}

/// Most waypoints of a branching non-termination witness (a longer stem falls
/// back to the loop-head assumption).
const MAX_LASSO_BRANCHINGS: usize = 4_000;

/// The branching form of a non-termination witness (what CPAchecker emits and
/// validates): one `branching` waypoint per decision of the stem (`follow`),
/// then one per decision of one round of the cycle (`cycle`), in execution
/// order. `None` when the cycle has no decision to list.
fn branching_lasso(
    module: &AirModule,
    meta: &WitnessMeta,
    lasso: &Lasso,
) -> Option<ViolationWitness> {
    let split = lasso.path_cycle_start;
    let decisions = crate::witness_lower::lower_path_decisions(module, &lasso.path);
    let mut stem = Vec::new();
    let mut cycle = Vec::new();
    for (entered, mut wp) in decisions {
        if entered <= split {
            stem.push(wp);
        } else {
            wp.action = Action::Cycle;
            cycle.push(wp);
        }
    }
    if cycle.is_empty() || stem.len() + cycle.len() > MAX_LASSO_BRANCHINGS {
        return None;
    }
    ViolationWitness::assemble_lasso(meta, &stem, &cycle).ok()
}

/// A loop without a loop statement (`goto` loops of CIL-generated code): the
/// start of the header's first statement, when that statement begins its own
/// line — an identifier (assignment or call), not a brace, label or keyword.
fn header_statement_location(
    module: &AirModule,
    block: &saf_core::air::AirBlock,
    src_lines: &[&str],
) -> Option<(String, u32, u32)> {
    let span = block
        .instructions
        .iter()
        .filter(|i| !matches!(i.op, saf_core::air::Operation::Phi { .. }))
        .find_map(|i| i.span.as_ref().filter(|s| s.line_start > 0))?;
    let (file_name, line, _) = crate::witness_lower::span_to_location(module, span)?;
    let column = leftmost_non_ws_column(src_lines, line)?;
    let text = src_lines.get(usize::try_from(line).ok()?.checked_sub(1)?)?;
    let rest = text.get(usize::try_from(column).ok()?.checked_sub(1)?..)?;
    let ident: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    let after = rest[ident.len()..].trim_start();
    let keyword = matches!(
        ident.as_str(),
        "if" | "else"
            | "while"
            | "for"
            | "do"
            | "switch"
            | "case"
            | "default"
            | "goto"
            | "return"
            | "break"
            | "continue"
    );
    let starts_statement = is_c_identifier(&ident)
        && !keyword
        && !after.starts_with(':')
        && rest.trim_end().ends_with(';');
    starts_statement.then_some((file_name, line, column))
}
