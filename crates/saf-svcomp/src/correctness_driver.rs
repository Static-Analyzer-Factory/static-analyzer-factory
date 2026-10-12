//! Driver: read SAF's converged interval abstract-interpretation fixpoint and
//! build a correctness (`invariant_set`) witness over loop-head invariants —
//! honoring the plan-207 soundness gates. **Emits no verdict**; this is the
//! read-out + selection layer beneath the pure emitter in `correctness_witness`.
//!
//! Gates (all fail-closed): module-level abstain on OpenMP source text; abstain
//! unless the absint fixpoint CONVERGED; per-loop abstain on function-call guards
//! (`while(f())`) and on any variable that does not recover a C name; drop bounds
//! that are type-trivial or non-inductive (via `interval_to_c_expr`). Any bound
//! that survives is a *true* fact over the converged solution.

use std::collections::BTreeMap;

use saf_analysis::absint::{AbstractInterpConfig, detect_loop_headers, solve_abstract_interp};
use saf_analysis::cfg::Cfg;
use saf_core::air::{AirBlock, AirModule, Operation};
use saf_core::ids::ValueId;

use crate::unreach_prove::{HeaderStates, StateVar};

use crate::correctness_witness::{
    InvariantKind, InvariantSetWitness, SourceInvariant, interval_to_c_expr,
};
use crate::witness_lower::span_to_location;
use crate::witness_yaml::WitnessMeta;

/// Whether the source text uses OpenMP. SAF's frontend drops `#pragma omp`, so
/// the AIR silently loses parallelism — any TRUE over such a program could be
/// wrong. Detected from raw source (the AIR can't see it) → module-level abstain.
#[must_use]
pub fn source_has_openmp(source: &str) -> bool {
    source.lines().any(|line| {
        let t = line.trim_start();
        (t.starts_with("#pragma") && t.contains("omp")) || t.contains("_Pragma(\"omp")
    })
}

/// Build a correctness `invariant_set` witness from `module`'s converged interval
/// fixpoint, or `None` if any soundness gate abstains or no confirmable loop-head
/// invariant is found. `source` is the original C text (for the OpenMP gate and
/// the loop-keyword column); `meta` supplies task metadata.
#[must_use]
pub fn build_interval_invariant_witness(
    module: &AirModule,
    source: &str,
    meta: &WitnessMeta,
) -> Option<InvariantSetWitness> {
    let invariants = interval_invariants(module, source);
    if invariants.is_empty() {
        return None;
    }
    InvariantSetWitness::assemble(meta, &invariants).ok()
}

/// The loop-head interval invariants of [`build_interval_invariant_witness`]
/// (empty when a gate abstains or nothing confirmable is found).
fn interval_invariants(module: &AirModule, source: &str) -> Vec<SourceInvariant> {
    // Module-level OpenMP abstain (the AIR silently dropped any parallelism).
    if source_has_openmp(source) {
        return Vec::new();
    }
    // Convergence gate (fail-closed): never rest an invariant on a mid-ascent
    // widening state — only the converged solution.
    let result = solve_abstract_interp(module, &AbstractInterpConfig::default());
    if !result.diagnostics().converged {
        return Vec::new();
    }

    let value_names = build_value_names(module);
    let src_lines: Vec<&str> = source.lines().collect();

    let mut invariants: Vec<SourceInvariant> = Vec::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        // Which SSA values may stand for a C variable at a loop head: the
        // header's OWN phis and the function's parameters — and only integer
        // ones. A same-named value from elsewhere (`j.2` after an inner loop,
        // or an inner `int j` that is out of scope at the outer header) is a
        // fact about a different program point and made the invariant false at
        // loop entry (`20 <= j` while `j == -100`); a pointer (`int *x` from
        // `alloca`) has no integer bound a validator accepts (`1 <= x`). Both
        // were CPAchecker "FALSE, invalid invariant" results (2026-10-10).
        let params: std::collections::BTreeSet<ValueId> = func
            .params
            .iter()
            .filter(|p| {
                !p.param_type
                    .is_some_and(|t| module.is_pointer_type(t) || module.is_float_type(t))
            })
            .map(|p| p.id)
            .collect();
        let pointer_values: std::collections::BTreeSet<ValueId> = func
            .blocks
            .iter()
            .flat_map(|b| b.instructions.iter())
            .filter(|i| {
                matches!(
                    i.op,
                    Operation::Alloca { .. } | Operation::HeapAlloc { .. } | Operation::Gep { .. }
                ) || i
                    .result_type
                    .is_some_and(|t| module.is_pointer_type(t) || module.is_float_type(t))
            })
            .filter_map(|i| i.dst)
            .collect();
        let cfg = Cfg::build(func);
        let headers = detect_loop_headers(&cfg);
        for block in &func.blocks {
            if !headers.contains(block.id) {
                continue;
            }
            let header_phis: std::collections::BTreeSet<ValueId> = block
                .instructions
                .iter()
                .filter(|i| matches!(i.op, Operation::Phi { .. }))
                .filter_map(|i| i.dst)
                .collect();
            // Function-call loop guard (`while(f())`): CPAchecker matches no
            // column and the omitted-column workaround crashes it (1a) — abstain.
            if header_has_function_call(block) {
                continue;
            }
            let Some((file_name, line, column)) = loop_head_location(module, block, &src_lines)
            else {
                continue;
            };
            // The loop-head invariant state lives in the inst-state after the
            // phis (the block-entry state is empty; phis are evaluated in-block).
            let Some(read_inst) = block
                .instructions
                .iter()
                .find(|i| !matches!(i.op, Operation::Phi { .. }))
            else {
                continue;
            };
            // Conjoin every named, non-trivial variable bound at the header. Each
            // kept bound is a true fact over the converged solution; unnamed
            // values and type-trivial/top bounds are dropped.
            let mut exprs: Vec<String> = Vec::new();
            for (vid, iv) in result.invariants_at_inst(read_inst.id) {
                let Some(name) = value_names.get(&vid) else {
                    continue;
                };
                if !is_c_identifier(name) {
                    continue;
                }
                // The variable at this header: its phi here, or a parameter.
                let represents_variable = (header_phis.contains(&vid) || params.contains(&vid))
                    && !pointer_values.contains(&vid);
                if !represents_variable {
                    continue;
                }
                if let Some(expr) = interval_to_c_expr(name, iv.lo(), iv.hi(), iv.bits()) {
                    exprs.push(expr);
                }
            }
            if exprs.is_empty() {
                continue; // self-validation: nothing confirmable here -> drop this loop.
            }
            invariants.push(SourceInvariant {
                file_name,
                line,
                column,
                function: Some(func.name.clone()),
                value: exprs.join(" && "),
                kind: InvariantKind::Loop,
            });
        }
    }

    invariants
}

/// The C literal for the bit pattern `v` of a `bits`-wide variable whose C
/// signedness the IR does not record. With the usual arithmetic conversions a
/// signed rendering compares equal for both signednesses at 32 and 64 bits
/// (`x == -1` holds for `unsigned x == UINT_MAX`); below 32 bits the operand is
/// promoted to `int` first, so both readings are offered when they differ.
fn c_value_literal(name: &str, v: u64, bits: u32) -> String {
    let signed: i128 = if bits >= 64 {
        i128::from(i64::from_ne_bytes(v.to_ne_bytes()))
    } else if v & (1u64 << (bits - 1)) != 0 {
        i128::from(v) - (1i128 << bits)
    } else {
        i128::from(v)
    };
    let suffix = if bits > 32 { "LL" } else { "" };
    let lit = |x: i128| {
        if x == i128::from(i64::MIN) {
            "(-9223372036854775807LL - 1)".to_string()
        } else if x == i128::from(i32::MIN) && bits <= 32 {
            "(-2147483647 - 1)".to_string()
        } else {
            format!("{x}{suffix}")
        }
    };
    if bits < 32 && signed < 0 {
        format!(
            "({name} == {} || {name} == {})",
            lit(signed),
            lit(i128::from(v))
        )
    } else {
        format!("{name} == {}", lit(signed))
    }
}

/// Most states a header may have before its exact state set is too large to
/// be a useful (or parseable) invariant.
const STATE_SET_MAX: usize = 64;

/// Loop invariants from the EXACT header state sets of an exhaustive proof
/// (`HeaderStates`): the disjunction, over the states seen at the header, of
/// the conjunction of `var == value` over the header's own integer phis,
/// parameters and scalar globals. Every state is one the proof explored, and
/// the fixpoint enumerated all of them, so the disjunction holds at the loop
/// head; dropping a variable (unnamed, pointer, out of scope) only weakens it.
fn state_set_invariants(
    module: &AirModule,
    src_lines: &[&str],
    value_names: &BTreeMap<ValueId, String>,
    states: &[HeaderStates],
) -> Vec<SourceInvariant> {
    let mut out = Vec::new();
    let global_names: BTreeMap<ValueId, &str> = module
        .globals
        .iter()
        .filter(|g| {
            !g.value_type
                .is_some_and(|t| module.is_pointer_type(t) || module.is_float_type(t))
        })
        .map(|g| (g.id, g.name.as_str()))
        .collect();
    for hs in states {
        if hs.states.is_empty() || hs.states.len() > STATE_SET_MAX {
            continue;
        }
        let Some(func) = module.function(hs.func) else {
            continue;
        };
        let Some(block) = func.blocks.iter().find(|b| b.id == hs.block) else {
            continue;
        };
        if header_has_function_call(block) {
            continue;
        }
        let Some((file_name, line, column)) = loop_head_location(module, block, src_lines) else {
            continue;
        };
        let header_phis: std::collections::BTreeSet<ValueId> = block
            .instructions
            .iter()
            .filter(|i| matches!(i.op, Operation::Phi { .. }))
            .filter_map(|i| i.dst)
            .collect();
        let params: std::collections::BTreeSet<ValueId> = func
            .params
            .iter()
            .filter(|p| {
                !p.param_type
                    .is_some_and(|t| module.is_pointer_type(t) || module.is_float_type(t))
            })
            .map(|p| p.id)
            .collect();
        let name_of = |var: &StateVar| -> Option<&str> {
            match var {
                StateVar::Value(v) if header_phis.contains(v) || params.contains(v) => {
                    value_names.get(v).map(String::as_str)
                }
                StateVar::Value(_) => None,
                StateVar::Global(g) => global_names.get(g).copied(),
            }
        };
        let mut disjuncts: Vec<String> = Vec::new();
        for st in &hs.states {
            let mut conj: Vec<String> = Vec::new();
            for (var, v, bits) in st {
                let Some(name) = name_of(var) else {
                    continue;
                };
                if !is_c_identifier(name) || *bits == 0 || *bits > 64 {
                    continue;
                }
                conj.push(c_value_literal(name, *v, *bits));
            }
            if conj.is_empty() {
                disjuncts.clear();
                break; // a state with nothing to say makes the disjunction `1`.
            }
            let d = format!("({})", conj.join(" && "));
            if !disjuncts.contains(&d) {
                disjuncts.push(d);
            }
        }
        if disjuncts.is_empty() {
            continue;
        }
        out.push(SourceInvariant {
            file_name,
            line,
            column,
            function: Some(func.name.clone()),
            value: disjuncts.join(" || "),
            kind: InvariantKind::Loop,
        });
    }
    out
}

/// The correctness witness for an exhaustive-engine proof: the exact
/// state-set invariants for the loop heads the fixpoint enumerated
/// (`states`), the converged interval invariants for the other loop heads, or
/// an empty `invariant_set` when neither says anything.
#[must_use]
pub fn build_correctness_witness(
    module: &AirModule,
    source: &str,
    meta: &WitnessMeta,
    states: &[HeaderStates],
) -> InvariantSetWitness {
    let src_lines: Vec<&str> = source.lines().collect();
    let value_names = build_value_names(module);
    let mut invariants = state_set_invariants(module, &src_lines, &value_names, states);
    let covered: std::collections::BTreeSet<(String, u32)> = invariants
        .iter()
        .map(|i| (i.file_name.clone(), i.line))
        .collect();
    for inv in interval_invariants(module, source) {
        if !covered.contains(&(inv.file_name.clone(), inv.line)) {
            invariants.push(inv);
        }
    }
    if invariants.is_empty() {
        return InvariantSetWitness::empty(meta);
    }
    InvariantSetWitness::assemble(meta, &invariants)
        .unwrap_or_else(|_| InvariantSetWitness::empty(meta))
}

/// Map each value-producing `ValueId` to its recovered C name, from debug
/// symbols only (post-mem2reg these sit on promoted phis + function params — the
/// `%`-free source names, unlike `DisplayResolver`, which `%`-prefixes and falls
/// back to hex for anonymous temporaries).
pub(crate) fn build_value_names(module: &AirModule) -> BTreeMap<ValueId, String> {
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

/// A loop whose header block itself performs a call re-evaluates a function-call
/// guard each iteration (`while(f())`); its keyword column is unmatchable.
pub(crate) fn header_has_function_call(block: &AirBlock) -> bool {
    block.instructions.iter().any(|i| {
        matches!(
            i.op,
            Operation::CallDirect { .. } | Operation::CallIndirect { .. }
        )
    })
}

/// `(file_name, line, column)` for a loop header, or `None` when it cannot be
/// placed on the loop statement's own line.
///
/// The line comes from the header block's LAST spanned non-phi instruction —
/// normally the terminator (`CondBr`/`Br`), which clang places at the
/// `while`/`for` keyword (measured: `while (i < 10)` → `br i1` at 11:5). It must
/// NOT come from the first spanned instruction: post-mem2reg a promoted phi can
/// carry the span of a reaching store BEFORE the loop (`Nested-2.c`: `j = 3;`,
/// line 9), and a witness entry on that line makes CPAchecker's witness parser
/// throw (`getIterationStructureFollowingColumnAtTheSameLine(..).orElseThrow()`),
/// which voids the WHOLE witness — 201 of the 451 rejected termination witnesses
/// in the svcomp26 run (plan 221 §3 A1) had exactly this shape.
///
/// The column is the leftmost non-ws char of that line (NOT the guard's dbg
/// column), and the line must START with a loop keyword there — the validator
/// resolves the entry to the iteration statement that begins at (line, column),
/// so `} while (c);`, `else while (c)` or a labelled loop cannot be matched and
/// the loop is skipped instead of risking the parser.
pub(crate) fn loop_head_location(
    module: &AirModule,
    block: &AirBlock,
    src_lines: &[&str],
) -> Option<(String, u32, u32)> {
    let span = block
        .instructions
        .iter()
        .rev()
        .filter(|i| !matches!(i.op, Operation::Phi { .. }))
        .find_map(|i| i.span.as_ref().filter(|s| s.line_start > 0))?;
    let (file_name, line, _guard_col) = span_to_location(module, span)?;
    let column = leftmost_non_ws_column(src_lines, line)?;
    if !line_starts_with_loop_keyword(src_lines, line, column) {
        return None;
    }
    Some((file_name, line, column))
}

/// `true` iff the text of 1-based `line` has a C loop keyword (`while`, `for`,
/// `do`) as the token starting at 1-based `column`.
fn line_starts_with_loop_keyword(src_lines: &[&str], line: u32, column: u32) -> bool {
    let Some(text) = usize::try_from(line)
        .ok()
        .and_then(|l| l.checked_sub(1))
        .and_then(|idx| src_lines.get(idx))
    else {
        return false;
    };
    let Some(rest) = usize::try_from(column)
        .ok()
        .and_then(|c| c.checked_sub(1))
        .and_then(|off| text.get(off..))
    else {
        return false;
    };
    ["while", "for", "do"].iter().any(|kw| {
        rest.starts_with(kw)
            && !rest[kw.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

/// `true` iff `name` is a C identifier (rejects the `%<hex>` fallback the display
/// resolver returns for an un-named/anonymous SSA value).
pub(crate) fn is_c_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 1-based column of the first non-whitespace char on 1-based `line`, or `None`.
/// This is the loop-keyword column SV-COMP witnesses want (never the guard's
/// column — see plan 207 / 1a gotchas).
fn leftmost_non_ws_column(src_lines: &[&str], line: u32) -> Option<u32> {
    let idx = usize::try_from(line).ok()?.checked_sub(1)?;
    let text = src_lines.get(idx)?;
    let byte_pos = text.find(|c: char| !c.is_whitespace())?;
    // Leading whitespace is ASCII in C source, so byte position == char column.
    u32::try_from(byte_pos + 1).ok()
}

#[cfg(test)]
mod state_set_tests {
    use super::c_value_literal;

    #[test]
    fn literals_read_the_same_for_either_signedness() {
        assert_eq!(c_value_literal("x", 5, 32), "x == 5");
        assert_eq!(c_value_literal("x", 0xFFFF_FFFF, 32), "x == -1");
        assert_eq!(
            c_value_literal("x", 0x8000_0000, 32),
            "x == (-2147483647 - 1)"
        );
        assert_eq!(c_value_literal("x", u64::MAX, 64), "x == -1LL");
        assert_eq!(c_value_literal("c", 0xFF, 8), "(c == -1 || c == 255)");
        assert_eq!(c_value_literal("b", 1, 1), "(b == -1 || b == 1)");
        assert_eq!(c_value_literal("b", 0, 1), "b == 0");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::Instruction;
    use saf_core::ids::{FileId, InstId, ModuleId};
    use saf_core::span::{SourceFile, Span};

    fn point(line: u32, col: u32) -> Span {
        Span::new(FileId::new(1), 0, 0, line, col, line, col)
    }

    fn inst(id: u128, op: Operation, span: Option<Span>) -> Instruction {
        let mut i = Instruction::new(InstId::new(id), op);
        i.span = span;
        i
    }

    fn module_with_t_c() -> AirModule {
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m
    }

    /// Measured on `termination-restricted-15/Nested-2.c`: post-mem2reg the
    /// outer loop header's phi carries the span of the reaching store BEFORE
    /// the loop (`j = 3;`, line 9) while the header's `CondBr` sits on the
    /// `while` line (11). CPAchecker's 2.1 parser looks the iteration statement
    /// up by that line and crashes (`NoSuchElementException`) when it is wrong,
    /// which rejected every entry of the witness.
    #[test]
    fn loop_head_location_takes_the_terminator_line_not_the_phi_line() {
        let m = module_with_t_c();
        let mut block = AirBlock::new(saf_core::ids::BlockId::new(1));
        block.instructions.push(inst(
            1,
            Operation::Phi { incoming: vec![] },
            Some(point(9, 7)),
        ));
        block
            .instructions
            .push(inst(2, Operation::Select, Some(point(11, 14))));
        block.instructions.push(inst(
            3,
            Operation::CondBr {
                then_target: saf_core::ids::BlockId::new(2),
                else_target: saf_core::ids::BlockId::new(3),
            },
            Some(point(11, 5)),
        ));
        let src: Vec<&str> = vec![
            "int main() {",
            "    int i;",
            "    int j;",
            "    i = 0;",
            "",
            "",
            "",
            "",
            "    j = 3;",
            "",
            "    while (i < 10) {",
            "        while (j < 12) {",
        ];
        assert_eq!(
            loop_head_location(&m, &block, &src),
            Some(("t.c".to_string(), 11, 5))
        );
    }

    /// A header whose terminator does not sit on a line that STARTS a loop
    /// statement (`} while (k > 0);` of a do-while, a labelled loop, …) cannot
    /// be matched by the validator's (line, column) lookup — abstain rather
    /// than emit an entry that makes the parser throw and void the witness.
    #[test]
    fn loop_head_location_abstains_unless_the_line_starts_with_a_loop_keyword() {
        let m = module_with_t_c();
        let mut block = AirBlock::new(saf_core::ids::BlockId::new(1));
        block
            .instructions
            .push(inst(1, Operation::Phi { incoming: vec![] }, None));
        block.instructions.push(inst(
            2,
            Operation::Br {
                target: saf_core::ids::BlockId::new(2),
            },
            Some(point(3, 5)),
        ));
        let src: Vec<&str> = vec!["int main() {", "    do {", "    } while (k > 0);"];
        assert_eq!(loop_head_location(&m, &block, &src), None);
        // `dowhile_count` is an identifier, not the keyword `do`.
        let src2: Vec<&str> = vec!["int main() {", "    do {", "    dowhile_count++;"];
        assert_eq!(loop_head_location(&m, &block, &src2), None);
    }

    #[test]
    fn loop_head_location_accepts_for_and_cil_style_while_one() {
        let m = module_with_t_c();
        let mut block = AirBlock::new(saf_core::ids::BlockId::new(1));
        block.instructions.push(inst(
            1,
            Operation::Br {
                target: saf_core::ids::BlockId::new(2),
            },
            Some(point(2, 3)),
        ));
        let src: Vec<&str> = vec!["{", "  while (1) {", "    for (i = 0; i < n; i++) {"];
        assert_eq!(
            loop_head_location(&m, &block, &src),
            Some(("t.c".to_string(), 2, 3))
        );
        block.instructions[0].span = Some(point(3, 5));
        assert_eq!(
            loop_head_location(&m, &block, &src),
            Some(("t.c".to_string(), 3, 5))
        );
    }

    #[test]
    fn openmp_pragma_detected() {
        assert!(source_has_openmp(
            "int main(){\n#pragma omp parallel for\nfor(;;);\n}"
        ));
        assert!(source_has_openmp("  #pragma omp atomic\n"));
    }

    #[test]
    fn no_openmp_not_detected() {
        assert!(!source_has_openmp(
            "int main(){ return 0; }\n#pragma once\n// not omp\n"
        ));
    }

    #[test]
    fn c_identifier_accepts_names_rejects_hex_and_temps() {
        assert!(is_c_identifier("s"));
        assert!(is_c_identifier("my_var"));
        assert!(is_c_identifier("_x0"));
        assert!(!is_c_identifier("%5cc0"));
        assert!(!is_c_identifier(""));
        assert!(!is_c_identifier("1abc"));
        assert!(!is_c_identifier("a b"));
    }

    #[test]
    fn leftmost_col_is_one_based_first_non_ws() {
        let lines = [
            "int main() {",
            "    for (int i = 0; i < 100; i++) {",
            "\t\twhile (x) {",
        ];
        assert_eq!(leftmost_non_ws_column(&lines, 2), Some(5)); // 4 spaces -> col 5
        assert_eq!(leftmost_non_ws_column(&lines, 1), Some(1)); // col 1
        assert_eq!(leftmost_non_ws_column(&lines, 3), Some(3)); // 2 tabs -> col 3
        assert_eq!(leftmost_non_ws_column(&lines, 99), None); // out of range
    }
}
