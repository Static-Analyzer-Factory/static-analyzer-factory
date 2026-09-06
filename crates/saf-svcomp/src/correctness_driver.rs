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

use crate::correctness_witness::{InvariantSetWitness, SourceInvariant, interval_to_c_expr};
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
    // Module-level OpenMP abstain (the AIR silently dropped any parallelism).
    if source_has_openmp(source) {
        return None;
    }
    // Convergence gate (fail-closed): never rest an invariant on a mid-ascent
    // widening state — only the converged solution.
    let result = solve_abstract_interp(module, &AbstractInterpConfig::default());
    if !result.diagnostics().converged {
        return None;
    }

    let value_names = build_value_names(module);
    let src_lines: Vec<&str> = source.lines().collect();

    let mut invariants: Vec<SourceInvariant> = Vec::new();
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
            });
        }
    }

    if invariants.is_empty() {
        return None;
    }
    InvariantSetWitness::assemble(meta, &invariants).ok()
}

/// Map each value-producing `ValueId` to its recovered C name, from debug
/// symbols only (post-mem2reg these sit on promoted phis + function params — the
/// `%`-free source names, unlike `DisplayResolver`, which `%`-prefixes and falls
/// back to hex for anonymous temporaries).
fn build_value_names(module: &AirModule) -> BTreeMap<ValueId, String> {
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
fn header_has_function_call(block: &AirBlock) -> bool {
    block.instructions.iter().any(|i| {
        matches!(
            i.op,
            Operation::CallDirect { .. } | Operation::CallIndirect { .. }
        )
    })
}

/// `(file_name, line, column)` for a loop header: the line comes from the first
/// spanned instruction in the header block (phis carry no span post-mem2reg; the
/// guard `icmp`/`br` sits on the loop-keyword line), and the column is the
/// leftmost non-ws char of that source line (NOT the guard's dbg column).
fn loop_head_location(
    module: &AirModule,
    block: &AirBlock,
    src_lines: &[&str],
) -> Option<(String, u32, u32)> {
    // Skip null spans (line 0 — the promoted phis carry these); the first real
    // span is the guard `icmp`/`br` on the loop-keyword line.
    let span = block
        .instructions
        .iter()
        .find_map(|i| i.span.as_ref().filter(|s| s.line_start > 0))?;
    let (file_name, line, _guard_col) = span_to_location(module, span)?;
    let column = leftmost_non_ws_column(src_lines, line)?;
    Some((file_name, line, column))
}

/// `true` iff `name` is a C identifier (rejects the `%<hex>` fallback the display
/// resolver returns for an un-named/anonymous SSA value).
fn is_c_identifier(name: &str) -> bool {
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
mod tests {
    use super::*;

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
