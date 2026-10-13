//! AIR → source lowering for SV-COMP violation witnesses.
//!
//! Turns the `unreach-call` evidence — a `must_reach_error` block chain or a
//! replay-confirmed [`FalseCandidate`] — into the flat, ordered
//! [`SourceWaypoint`] list that
//! [`crate::witness_yaml::ViolationWitness::assemble`] serializes. This is the
//! only witness code that touches AIR; the YAML model in `witness_yaml` stays
//! property-generic.
//!
//! Emits a `target` waypoint at the `reach_error()` call site, plus — for a
//! replay-confirmed candidate — `branching` waypoints for the `CondBr` decisions
//! along the confirmed path, located by LINE ONLY so CPAchecker resolves them to
//! the `if`/`while` keyword (plan 195 Tier A), and `function_return` waypoints
//! pinning each `__VERIFIER_nondet_*` read to the exact value the replay used
//! (`\result == <value>`, an ACSL constraint at the call site). The nondet
//! waypoints are what let execution-based validators (`cpa-witness2test`)
//! reproduce a nondet-driven violation without re-solving the path constraints —
//! the confirmation-gap fix. See [`lower_candidate`].
//!
//! Two validator-facing rules, both measured against CPAchecker 4.2.2 on the
//! svcomp26 run (plan 221 §3 A3):
//!
//! * A `branching` waypoint's value is the value of the SOURCE controlling
//!   expression, not of the IR `CondBr` operand. clang compiles `if (!(c))` as
//!   `br (c != 0), if.end, if.then` — targets swapped — so the IR decision is
//!   the negation of the source one; with the IR value the validator follows
//!   the non-error edge and reports `TRUE` (every `__VERIFIER_assert`-idiom
//!   witness was rejected that way). The source value is recovered from clang's
//!   block names (`if.then`, `*.body`, `land.*`, …), which `saf verify` keeps
//!   with `-fno-discard-value-names`; a short-circuit chain (`a && b`) folds into
//!   ONE waypoint whose value is the side the statement finally took, as the
//!   format defines it. Without clang names the IR decision is used as before.
//! * A `function_return` waypoint MUST carry the column of the call's closing
//!   `)`: CPAchecker 4.2.2 throws (`NoSuchElementException`) on a column-less
//!   one, voiding the witness. The column comes from the source text; when the
//!   text is unavailable the waypoint is dropped (a target-only witness stays
//!   valid).

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction, Operation};
use saf_core::ids::{BlockId, InstId};
use saf_core::span::Span;

use crate::overflow::{OverflowHit, lower_overflow_hit_enriched};
use crate::property::{FalseCandidate, REACH_ERROR_NAMES, is_scalar_integer_nondet};
use crate::witness_yaml::{Action, Constraint, SourceWaypoint, WaypointKind};

/// Resolve a [`Span`] to `(file_name, line, column)` via `module.source_files`.
///
/// Returns `None` if the span's `file_id` is not among the module's source files.
#[must_use]
pub fn span_to_location(module: &AirModule, span: &Span) -> Option<(String, u32, u32)> {
    let source = module
        .source_files
        .iter()
        .find(|sf| sf.id == span.file_id)?;
    // Use the basename so the witness `file_name` matches the task's
    // `input_files` entry and the source file a validator is given, and is
    // machine-independent (CPAchecker emits basenames too).
    let file_name = std::path::Path::new(&source.path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(source.path.as_str())
        .to_string();
    Some((file_name, span.line_start, span.col_start))
}

/// Locate the `(function, block)` owning `block_id`.
///
/// Linear scan — robust for hand-built modules whose `block_index` may be empty.
fn find_block_in_module(
    module: &AirModule,
    block_id: BlockId,
) -> Option<(&AirFunction, &AirBlock)> {
    module
        .functions
        .iter()
        .find_map(|f| f.blocks.iter().find(|b| b.id == block_id).map(|b| (f, b)))
}

/// Locate the `(function, instruction)` for `inst_id`.
fn find_inst(module: &AirModule, inst_id: InstId) -> Option<(&AirFunction, &Instruction)> {
    module.functions.iter().find_map(|f| {
        f.blocks.iter().find_map(|b| {
            b.instructions
                .iter()
                .find(|i| i.id == inst_id)
                .map(|i| (f, i))
        })
    })
}

/// The first `reach_error`/`__VERIFIER_error` direct call in `block`, if any.
fn reach_error_inst_in_block<'a>(
    module: &AirModule,
    block: &'a AirBlock,
) -> Option<&'a Instruction> {
    block.instructions.iter().find(|inst| {
        if let Operation::CallDirect { callee } = &inst.op {
            let name = module.function(*callee).map_or("", |f| f.name.as_str());
            REACH_ERROR_NAMES.contains(&name)
        } else {
            false
        }
    })
}

/// Build a single `target` waypoint at a resolved source location.
fn target_waypoint(location: (String, u32, u32), function: Option<String>) -> SourceWaypoint {
    let (file_name, line, column) = location;
    SourceWaypoint {
        kind: WaypointKind::Target,
        action: Action::Follow,
        file_name,
        line,
        column: Some(column),
        function,
        constraint: None,
    }
}

/// Build a `branching` waypoint located by LINE ONLY (column omitted).
///
/// CPAchecker snaps a column-less `branching` waypoint to the `if`/`while`
/// keyword; a column pointing into the condition expression is mis-parsed as a
/// ternary and the whole witness is rejected (the plan-194 Slice-E regression —
/// fixed here by dropping the column, plan 195 Tier A). `taken` is which
/// successor the confirmed path followed (`true` = `then_target`).
fn branching_waypoint(
    file_name: String,
    line: u32,
    function: Option<String>,
    taken: bool,
) -> SourceWaypoint {
    SourceWaypoint {
        kind: WaypointKind::Branching,
        action: Action::Follow,
        file_name,
        line,
        column: None,
        function,
        constraint: Some(Constraint {
            format: None,
            value: if taken { "true" } else { "false" }.to_string(),
        }),
    }
}

/// Build a `function_return` waypoint pinning a `__VERIFIER_nondet_*` read to
/// the concrete value the confirmed replay used: `\result == <value>` (an ACSL
/// constraint at the call site — the witness-2.0 mechanism for a nondet input).
///
/// Located at the call's closing `)` (line AND column), which is what the
/// format prescribes for `function_return` and what CPAchecker 4.2.2 requires:
/// a column-less `function_return` makes its parser throw and voids the whole
/// witness (measured; adding the column alone turned a rejected minimal case
/// into `FALSE`/confirmed). The column is found in `src_lines` from the call's
/// own span; returns `None` — dropping just this waypoint, toward the
/// always-valid target-only witness — when the call, its span, the source text
/// or the parenthesis cannot be resolved.
/// Lower an engine overflow counterexample (plan 222 §2n): one
/// `function_return` waypoint per pinned nondet / `rand` read on the path
/// (`\result == value`, in program order), then the overflow's `target` from
/// the UBSan report. Reads the validator cannot pin by result (`fscanf`) are
/// left out; the target alone is still a valid (weaker) witness.
#[must_use]
pub fn lower_overflow_candidate(
    module: &AirModule,
    cand: &FalseCandidate,
    hit: &OverflowHit,
    source: Option<&str>,
) -> Vec<SourceWaypoint> {
    let src_lines: Option<Vec<&str>> = source.map(|s| s.lines().collect());
    let src_lines = src_lines.as_deref();
    let mut waypoints = Vec::new();
    for nc in &cand.nondet_sequence {
        if !(is_scalar_integer_nondet(&nc.func_name)
            || nc.func_name == "rand"
            || crate::unreach_prove::is_pinned_extern(&nc.func_name))
        {
            continue;
        }
        let Some(id) = nc.call_inst else {
            continue;
        };
        if let Some(wp) = function_return_waypoint(module, id, nc.value, src_lines) {
            waypoints.push(wp);
        }
    }
    waypoints.extend(lower_overflow_hit_enriched(hit, &[]));
    waypoints
}

/// The violation witness of an engine counterexample for `valid-memsafety`,
/// confirmed by an AddressSanitizer replay: the pinned nondeterministic reads
/// as `function_return` assumptions (as for `no-overflow`), then the target at
/// the access ASan trapped on.
#[must_use]
pub fn lower_memsafety_candidate(
    module: &AirModule,
    cand: &FalseCandidate,
    hit: &crate::memsafety::AsanHit,
    source: Option<&str>,
) -> Vec<SourceWaypoint> {
    let src_lines: Option<Vec<&str>> = source.map(|s| s.lines().collect());
    let src_lines = src_lines.as_deref();
    let mut waypoints = Vec::new();
    for nc in &cand.nondet_sequence {
        if !(is_scalar_integer_nondet(&nc.func_name)
            || nc.func_name == "rand"
            || crate::unreach_prove::is_pinned_extern(&nc.func_name))
        {
            continue;
        }
        let Some(id) = nc.call_inst else {
            continue;
        };
        if let Some(wp) = function_return_waypoint(module, id, nc.value, src_lines) {
            waypoints.push(wp);
        }
    }
    waypoints.extend(crate::memsafety::lower_memsafety_hit(hit));
    waypoints
}

/// Developer aid: the `(file, line, column)` of an instruction, if it has a span.
#[must_use]
pub fn find_inst_span(module: &AirModule, inst_id: InstId) -> Option<(String, u32, u32)> {
    let (_, inst) = find_inst(module, inst_id)?;
    span_to_location(module, inst.span.as_ref()?)
}

fn function_return_waypoint(
    module: &AirModule,
    call_inst: InstId,
    value: i64,
    src_lines: Option<&[&str]>,
) -> Option<SourceWaypoint> {
    let (func, inst) = find_inst(module, call_inst)?;
    let (file_name, line, col) = span_to_location(module, inst.span.as_ref()?)?;
    let column = call_close_paren_column(src_lines?, line, col)?;
    Some(SourceWaypoint {
        kind: WaypointKind::FunctionReturn,
        action: Action::Follow,
        file_name,
        line,
        column: Some(column),
        function: Some(func.name.clone()),
        constraint: Some(Constraint {
            format: Some("acsl_expression".to_string()),
            // `\result <op> <const>` is the only function_return form the format
            // supports; a bare integer is a valid constant expression for every
            // integer return type (a negative value from a signed model still
            // compares equal to the unsigned wrap the replay produced).
            value: format!("\\result == {value}"),
        }),
    })
}

/// 1-based column of the `)` closing the call whose expression starts at
/// 1-based (`line`, `col`): the first `(` at or after `col` on that line and its
/// matching parenthesis. `None` if the line is out of range or unbalanced.
fn call_close_paren_column(src_lines: &[&str], line: u32, col: u32) -> Option<u32> {
    let text = src_lines.get(usize::try_from(line).ok()?.checked_sub(1)?)?;
    let start = usize::try_from(col).ok()?.checked_sub(1)?;
    let open = start + text.get(start..)?.find('(')?;
    let mut depth = 0usize;
    for (off, ch) in text.get(open..)?.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return u32::try_from(open + off + 1).ok();
                }
            }
            _ => {}
        }
    }
    None
}

/// What a clang-named basic block tells us about the branching statement it
/// belongs to (`-fno-discard-value-names`): the side a decision enters, or a
/// short-circuit intermediate (`land.lhs.true`, `land.rhs`, `lor.lhs.false`, …)
/// of an `&&`/`||` chain. `None` for an unnamed block or a non-clang name (a
/// CIL `goto` label, say).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockRole {
    /// `if.then*`, `while.body*`, `for.body*`, `do.body*`, `cond.true*`.
    TrueSide,
    /// `if.else*`, `if.end*`, `while.end*`, `for.end*`, `do.end*`, `cond.false*`.
    FalseSide,
    /// `land.*` / `lor.*`: still inside the controlling expression.
    ShortCircuit,
}

fn clang_block_role(label: Option<&str>) -> Option<BlockRole> {
    const TRUE_SIDE: [&str; 5] = ["if.then", "while.body", "for.body", "do.body", "cond.true"];
    const FALSE_SIDE: [&str; 6] = [
        "if.else",
        "if.end",
        "while.end",
        "for.end",
        "do.end",
        "cond.false",
    ];
    let l = label?;
    if l.starts_with("land.") || l.starts_with("lor.") {
        return Some(BlockRole::ShortCircuit);
    }
    if TRUE_SIDE.iter().any(|p| l.starts_with(p)) {
        return Some(BlockRole::TrueSide);
    }
    if FALSE_SIDE.iter().any(|p| l.starts_with(p)) {
        return Some(BlockRole::FalseSide);
    }
    None
}

/// The `branching` waypoint for the decision `path[i] -> path[i + 1]`, or
/// `None` when there is nothing to emit: a non-`CondBr` terminator,
/// `then == else`, a pair that is not an edge, an unresolved span — or a
/// decision INSIDE a short-circuit chain, whose statement already got its
/// waypoint at the chain's first decision.
///
/// The value is the SOURCE controlling expression's value (what the format
/// defines and the validator checks), recovered from clang's block names:
/// follow the path through any `land.*`/`lor.*` intermediates to the block that
/// marks the side taken (`if.then`/`*.body` ⇒ `true`, `if.else`/`if.end`/`*.end`
/// ⇒ `false`). That is what makes `if (!(c))` right — clang emits it as
/// `br (c != 0), if.end, if.then`, so the IR decision is the negation of the
/// source one. Without clang names the IR decision (`true` = `then_target`) is
/// used, exactly as before.
fn branching_for_path_step(
    module: &AirModule,
    path: &[BlockId],
    i: usize,
) -> Option<SourceWaypoint> {
    let from = *path.get(i)?;
    let to = *path.get(i + 1)?;
    let (func, block) = find_block_in_module(module, from)?;
    let terminator = block.terminator()?;
    let Operation::CondBr {
        then_target,
        else_target,
    } = &terminator.op
    else {
        return None;
    };
    // Ambiguous when both edges go to the same block.
    if then_target == else_target {
        return None;
    }
    // Path pair is not an edge of this CondBr — do not guess.
    if to != *then_target && to != *else_target {
        return None;
    }
    let label_of = |id: BlockId| {
        func.blocks
            .iter()
            .find(|b| b.id == id)
            .and_then(|b| b.label.as_deref())
    };
    if clang_block_role(block.label.as_deref()) == Some(BlockRole::ShortCircuit) {
        return None;
    }
    let mut exit = i + 1;
    while exit + 1 < path.len()
        && clang_block_role(label_of(path[exit])) == Some(BlockRole::ShortCircuit)
    {
        exit += 1;
    }
    let taken = match clang_block_role(label_of(path[exit])) {
        Some(BlockRole::TrueSide) => true,
        Some(BlockRole::FalseSide) => false,
        _ => {
            // No clang-named exit: the IR decision, undoing clang's target swap
            // for a negated condition when the successors' names reveal it.
            let swapped = clang_block_role(label_of(*then_target)) == Some(BlockRole::FalseSide)
                && clang_block_role(label_of(*else_target)) == Some(BlockRole::TrueSide);
            (to == *then_target) != swapped
        }
    };
    let (file_name, line, _col) = terminator
        .span
        .as_ref()
        .and_then(|s| span_to_location(module, s))?;
    Some(branching_waypoint(
        file_name,
        line,
        Some(func.name.clone()),
        taken,
    ))
}

/// Lower a `must_reach_error` block chain to a target-only waypoint list.
///
/// The `reach_error` call lives in the LAST block of the chain. Returns `None`
/// if the block, the call, or its span cannot be resolved (a sound `false` is
/// still emitted without a witness in that case).
#[must_use]
pub fn lower_must_reach(module: &AirModule, chain: &[BlockId]) -> Option<Vec<SourceWaypoint>> {
    let last = chain.last()?;
    let (func, block) = find_block_in_module(module, *last)?;
    let inst = reach_error_inst_in_block(module, block)?;
    let location = span_to_location(module, inst.span.as_ref()?)?;
    Some(vec![target_waypoint(location, Some(func.name.clone()))])
}

/// Lower a replay-confirmed [`FalseCandidate`] to a waypoint list in execution
/// order: `function_return` waypoints pinning each `__VERIFIER_nondet_*` read to
/// the confirmed value (`\result == <value>`), interleaved with `branching`
/// waypoints for each `CondBr` decision along the confirmed path (plan 195 Tier
/// A), ending in the `target` at the `reach_error` call.
///
/// **Ordering (soundness of the witness, not the verdict):** nondet reads that
/// execute *before* the error site's own function — the caller reads in an
/// interprocedural candidate, whose `block_path` covers only the error function —
/// are emitted first, in `nondet_sequence` (whole-program execution) order. Then
/// the error function's `block_path` is walked block by block: each block's own
/// nondet reads (in instruction order) are emitted before that block's branching
/// decision, so a `read; branch-on-it` pair keeps its real order.
///
/// The derivation is **strictly additive**: any nondet read without an AIR
/// identity (`call_inst == None`, e.g. a fuzz/CBMC-derived sequence) or any hop
/// whose decision cannot be unambiguously recovered is skipped, degrading toward
/// the always-valid target-only witness — never an invalid waypoint that would
/// make a validator reject the whole witness. Returns `None` if the `reach_error`
/// instruction or its span cannot be resolved.
#[must_use]
pub fn lower_candidate(
    module: &AirModule,
    cand: &FalseCandidate,
    source: Option<&str>,
) -> Option<Vec<SourceWaypoint>> {
    let src_lines: Option<Vec<&str>> = source.map(|s| s.lines().collect());
    let src_lines = src_lines.as_deref();
    let (func, inst) = find_inst(module, cand.reach_error_inst)?;
    let target_location = span_to_location(module, inst.span.as_ref()?)?;

    // Value to pin at each nondet call site (first occurrence wins; a Z3/interproc
    // path is simple so every scalar-int read appears exactly once).
    // Only reads whose RESULT is the pinned value get a `\result == v`
    // waypoint: the nondet functions, `rand` and the pinned externs (`atoi`,
    // `recv`, …). A pinned `fscanf` conversion (plan 222 §2n) is written through
    // a pointer (its result is the count), and `fgets` is pinned to "returned
    // its buffer": neither gets a waypoint.
    let pinned_result = |name: &str| {
        is_scalar_integer_nondet(name)
            || name == "rand"
            || crate::unreach_prove::is_pinned_extern(name)
    };
    let mut nondet_value: BTreeMap<InstId, i64> = BTreeMap::new();
    for nc in &cand.nondet_sequence {
        if let Some(id) = nc.call_inst {
            if pinned_result(&nc.func_name) {
                nondet_value.entry(id).or_insert(nc.value);
            }
        }
    }

    // Instruction ids that live inside the error function's local `block_path`;
    // a nondet read NOT in this set executed in a caller (interprocedural prefix).
    let mut inpath_insts: BTreeSet<InstId> = BTreeSet::new();
    for bid in &cand.block_path {
        if let Some((_f, block)) = find_block_in_module(module, *bid) {
            for i in &block.instructions {
                inpath_insts.insert(i.id);
            }
        }
    }

    let mut waypoints = Vec::new();

    // 1. Interprocedural prefix: caller reads (execute before the error function),
    //    in whole-program execution order.
    for nc in &cand.nondet_sequence {
        let Some(id) = nc.call_inst else {
            continue;
        };
        if !pinned_result(&nc.func_name) || inpath_insts.contains(&id) {
            continue; // emitted during the block walk below, in place
        }
        if let Some(wp) = function_return_waypoint(module, id, nc.value, src_lines) {
            waypoints.push(wp);
        }
    }

    // 2. Walk the error function's path: each block's nondet reads (instruction
    //    order) before that block's branching decision.
    for (i, bid) in cand.block_path.iter().enumerate() {
        let Some((_bfunc, block)) = find_block_in_module(module, *bid) else {
            continue;
        };
        for ins in &block.instructions {
            if let Some(&value) = nondet_value.get(&ins.id) {
                if let Some(wp) = function_return_waypoint(module, ins.id, value, src_lines) {
                    waypoints.push(wp);
                }
            }
        }
        if let Some(wp) = branching_for_path_step(module, &cand.block_path, i) {
            waypoints.push(wp);
        }
    }

    waypoints.push(target_waypoint(target_location, Some(func.name.clone())));
    Some(waypoints)
}

/// Lower the module's first `reach_error` / `__VERIFIER_error` call site (in
/// deterministic module order) to a single-`target` waypoint list — a minimal,
/// always-valid YAML-2.0 violation-witness anchor for a **concurrency**
/// unreach-call FALSE.
///
/// The concurrency confirmers prove the violation by forced-schedule native
/// replay, not a sequential must-reach chain, so there is no block path to lower;
/// the reproducing interleaving is carried by the companion `GraphML` witness. But
/// an execution / re-verification validator (`CBMC`, `cpa-witness2test`, `CPAchecker`)
/// re-derives the schedule itself and only needs the violation *location* to
/// confirm — a target-only witness at the `reach_error` call gives exactly that,
/// and passes `WitnessLint`'s 2.0 violation-witness schema (the `GraphML` witness
/// does not, so a validator panel with no concurrency member scores it 0). This is
/// a witness *artifact* only: the FALSE verdict itself is already sound (native
/// replay confirmed), so the anchor can never turn a right verdict into a wrong
/// one. Returns `None` if no `reach_error` call carries a source span (the FALSE is
/// still emitted, just witnessless).
#[must_use]
pub fn lower_reach_error_target(module: &AirModule) -> Option<Vec<SourceWaypoint>> {
    for inst_id in crate::property::reach_error_call_sites(module) {
        let Some((func, inst)) = find_inst(module, inst_id) else {
            continue;
        };
        let Some(span) = inst.span.as_ref() else {
            continue;
        };
        let Some(location) = span_to_location(module, span) else {
            continue;
        };
        return Some(vec![target_waypoint(location, Some(func.name.clone()))]);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::{FileId, FunctionId, ModuleId};
    use saf_core::span::SourceFile;
    use std::collections::BTreeMap;

    fn decl(id: FunctionId, name: &str) -> AirFunction {
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn func(id: FunctionId, name: &str, blocks: Vec<AirBlock>, entry: BlockId) -> AirFunction {
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn blk(id: BlockId, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id,
            label: None,
            instructions,
        }
    }

    fn ret(iid: u128) -> Instruction {
        Instruction::new(InstId::new(iid), Operation::Ret)
    }

    fn point_span(line: u32, col: u32) -> Span {
        Span::new(FileId::new(1), 0, 1, line, col, line, col + 1)
    }

    #[test]
    fn span_to_location_resolves_via_source_files() {
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        assert_eq!(
            span_to_location(&m, &point_span(12, 5)),
            Some(("t.c".to_string(), 12, 5))
        );
        let unknown = Span::new(FileId::new(99), 0, 1, 1, 1, 1, 1);
        assert_eq!(span_to_location(&m, &unknown), None);
    }

    #[test]
    fn lower_must_reach_emits_single_target_at_last_block() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (bb0, bb1) = (BlockId::new(10), BlockId::new(11));
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        let err_call = Instruction::new(InstId::new(3), Operation::CallDirect { callee: err_id })
            .with_span(point_span(20, 3));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(bb0, vec![]), blk(bb1, vec![err_call, ret(4)])],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let wps = lower_must_reach(&m, &[bb0, bb1]).expect("lowered");
        assert_eq!(wps.len(), 1);
        assert_eq!(wps[0].kind, WaypointKind::Target);
        assert_eq!(wps[0].action, Action::Follow);
        assert_eq!(wps[0].line, 20);
        assert_eq!(wps[0].file_name, "t.c");
        assert_eq!(wps[0].function.as_deref(), Some("main"));
    }

    #[test]
    fn lower_must_reach_none_when_span_missing() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let bb = BlockId::new(10);
        let mut m = AirModule::new(ModuleId::new(1));
        // No source_files, and the reach_error call carries no span.
        let err_call = Instruction::new(InstId::new(3), Operation::CallDirect { callee: err_id });
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(bb, vec![err_call, ret(4)])],
            bb,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        assert!(lower_must_reach(&m, &[bb]).is_none());
    }

    #[test]
    fn lower_candidate_emits_target_at_reach_error_inst() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let bb = BlockId::new(10);
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        let err_call = Instruction::new(reach_iid, Operation::CallDirect { callee: err_id })
            .with_span(point_span(33, 7));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(bb, vec![err_call, ret(4)])],
            bb,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert_eq!(wps.len(), 1);
        assert_eq!(wps[0].kind, WaypointKind::Target);
        assert_eq!(wps[0].line, 33);
        assert_eq!(wps[0].file_name, "t.c");
    }

    #[test]
    fn lower_reach_error_target_emits_target_at_first_error_call() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (bb0, bb1) = (BlockId::new(10), BlockId::new(11));
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        let err_call = Instruction::new(InstId::new(3), Operation::CallDirect { callee: err_id })
            .with_span(point_span(41, 9));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(bb0, vec![]), blk(bb1, vec![err_call, ret(4)])],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let wps = lower_reach_error_target(&m).expect("lowered");
        assert_eq!(wps.len(), 1);
        assert_eq!(wps[0].kind, WaypointKind::Target);
        assert_eq!(wps[0].action, Action::Follow);
        assert_eq!(wps[0].line, 41);
        assert_eq!(wps[0].file_name, "t.c");
        assert_eq!(wps[0].function.as_deref(), Some("main"));
    }

    #[test]
    fn lower_reach_error_target_none_when_no_error_span() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let bb = BlockId::new(10);
        let mut m = AirModule::new(ModuleId::new(1));
        // reach_error call carries no span, and there are no source files.
        let err_call = Instruction::new(InstId::new(3), Operation::CallDirect { callee: err_id });
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(bb, vec![err_call, ret(4)])],
            bb,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        assert!(lower_reach_error_target(&m).is_none());
    }

    #[test]
    fn lower_reach_error_target_none_when_no_error_call() {
        let main_id = FunctionId::new(1);
        let bb = BlockId::new(10);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions
            .push(func(main_id, "main", vec![blk(bb, vec![ret(4)])], bb));
        assert!(lower_reach_error_target(&m).is_none());
    }

    // ---- Slice A (plan 195): Tier A branching waypoints (no column) ----

    fn cond_br(
        iid: u128,
        then_target: BlockId,
        else_target: BlockId,
        line: u32,
        col: u32,
    ) -> Instruction {
        Instruction::new(
            InstId::new(iid),
            Operation::CondBr {
                then_target,
                else_target,
            },
        )
        .with_span(point_span(line, col))
    }

    fn uncond_br(iid: u128, target: BlockId) -> Instruction {
        Instruction::new(InstId::new(iid), Operation::Br { target })
    }

    fn err_at(iid: InstId, err_id: FunctionId, line: u32, col: u32) -> Instruction {
        Instruction::new(iid, Operation::CallDirect { callee: err_id })
            .with_span(point_span(line, col))
    }

    #[test]
    fn lower_candidate_emits_branching_no_column_when_then_taken() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (bb0, bb1, bb2) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                // The CondBr span column (9) points INTO the condition and MUST be
                // discarded: CPAchecker snaps a column-less branching waypoint to
                // the if/while keyword; a column-into-condition is rejected as a
                // ternary (the plan-194 Slice-E regression).
                blk(bb0, vec![cond_br(2, bb1, bb2, 10, 9)]),
                blk(bb1, vec![err_at(reach_iid, err_id, 12, 7), ret(4)]),
                blk(bb2, vec![ret(5)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb1],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert_eq!(wps.len(), 2, "one branching + target");
        assert_eq!(wps[0].kind, WaypointKind::Branching);
        assert_eq!(wps[0].action, Action::Follow);
        assert_eq!(wps[0].line, 10);
        assert_eq!(wps[0].column, None, "branching column MUST be omitted");
        assert_eq!(
            wps[0].constraint,
            Some(crate::witness_yaml::Constraint {
                format: None,
                value: "true".into(),
            })
        );
        assert_eq!(wps[1].kind, WaypointKind::Target);
        assert_eq!(wps[1].line, 12);
    }

    #[test]
    fn lower_candidate_emits_branching_false_when_else_taken() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (bb0, bb1, bb2) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(bb0, vec![cond_br(2, bb1, bb2, 10, 9)]),
                blk(bb1, vec![ret(4)]),
                blk(bb2, vec![err_at(reach_iid, err_id, 14, 5), ret(5)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb2],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert_eq!(wps.len(), 2);
        assert_eq!(wps[0].kind, WaypointKind::Branching);
        assert_eq!(wps[0].column, None);
        assert_eq!(
            wps[0].constraint,
            Some(crate::witness_yaml::Constraint {
                format: None,
                value: "false".into(),
            })
        );
        assert_eq!(wps[1].kind, WaypointKind::Target);
        assert_eq!(wps[1].line, 14);
    }

    #[test]
    fn lower_candidate_no_branching_when_then_equals_else() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (bb0, bb1) = (BlockId::new(10), BlockId::new(11));
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                // Both edges go to bb1 → the branch value is ambiguous.
                blk(bb0, vec![cond_br(2, bb1, bb1, 10, 9)]),
                blk(bb1, vec![err_at(reach_iid, err_id, 12, 7), ret(4)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb1],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert_eq!(wps.len(), 1, "ambiguous then==else → target-only");
        assert_eq!(wps[0].kind, WaypointKind::Target);
    }

    #[test]
    fn lower_candidate_no_branching_through_unconditional_br() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (bb0, bb1) = (BlockId::new(10), BlockId::new(11));
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(bb0, vec![uncond_br(2, bb1)]),
                blk(bb1, vec![err_at(reach_iid, err_id, 12, 7), ret(4)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb1],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert_eq!(
            wps.len(),
            1,
            "unconditional Br has no decision → target-only"
        );
        assert_eq!(wps[0].kind, WaypointKind::Target);
    }

    #[test]
    fn lower_candidate_emits_branching_per_condbr_in_path_order() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (bb0, bb1, bb2, bx, by) = (
            BlockId::new(10),
            BlockId::new(11),
            BlockId::new(12),
            BlockId::new(20),
            BlockId::new(21),
        );
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(bb0, vec![cond_br(2, bb1, bx, 10, 9)]), // then taken → true
                blk(bb1, vec![cond_br(3, by, bb2, 14, 4)]), // else taken → false
                blk(bb2, vec![err_at(reach_iid, err_id, 16, 7), ret(4)]),
                blk(bx, vec![ret(5)]),
                blk(by, vec![ret(6)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb1, bb2],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert_eq!(wps.len(), 3, "two branching + target, in path order");
        assert_eq!(wps[0].kind, WaypointKind::Branching);
        assert_eq!(wps[0].line, 10);
        assert_eq!(wps[0].constraint.as_ref().unwrap().value, "true");
        assert_eq!(wps[1].kind, WaypointKind::Branching);
        assert_eq!(wps[1].line, 14);
        assert_eq!(wps[1].constraint.as_ref().unwrap().value, "false");
        assert_eq!(wps[2].kind, WaypointKind::Target);
        assert_eq!(wps[2].line, 16);
    }

    #[test]
    fn lower_candidate_branching_witness_is_byte_stable() {
        use crate::property_kind::{DataModel, Language};
        use crate::witness_yaml::{ViolationWitness, WitnessMeta};

        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (bb0, bb1, bb2) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(bb0, vec![cond_br(2, bb1, bb2, 10, 9)]),
                blk(bb1, vec![err_at(reach_iid, err_id, 12, 7), ret(4)]),
                blk(bb2, vec![ret(5)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb1],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        let meta = WitnessMeta {
            producer_version: "0.1.0".to_string(),
            specification: "CHECK( init(main()), LTL(G ! call(reach_error())) )".to_string(),
            data_model: DataModel::ILP32,
            language: Language::C,
            input_file: std::path::PathBuf::from("/nonexistent/t.c"),
        };
        let serialize = || {
            ViolationWitness::assemble(&meta, &wps)
                .expect("assemble")
                .to_yaml_string()
                .expect("serialize")
        };
        let (y1, y2) = (serialize(), serialize());
        assert_eq!(y1, y2, "enriched witness must be byte-stable");
        assert_eq!(
            y1.matches("type: branching").count(),
            1,
            "exactly one branching waypoint\n{y1}"
        );
        assert!(y1.contains("type: target"), "{y1}");
    }

    fn blk_named(id: BlockId, label: &str, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id,
            label: Some(label.to_string()),
            instructions,
        }
    }

    /// clang compiles `if (!(cond))` as `br (cond != 0), if.end, if.then`: the
    /// error block is the ELSE target of the IR decision, yet the source
    /// controlling expression `!(cond)` is TRUE on that edge. Measured on
    /// `loop-acceleration/array_1-1.c` & co.: the IR value (`false`) sent the
    /// validator down the non-error edge and it answered `TRUE` (rejected).
    #[test]
    fn branching_value_is_the_source_value_when_clang_swapped_the_targets() {
        let (f_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (entry, if_then, if_end) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            f_id,
            "__VERIFIER_assert",
            vec![
                blk_named(entry, "entry", vec![cond_br(2, if_end, if_then, 6, 7)]),
                blk_named(
                    if_then,
                    "if.then",
                    vec![err_at(reach_iid, err_id, 7, 13), ret(4)],
                ),
                blk_named(if_end, "if.end", vec![ret(5)]),
            ],
            entry,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![entry, if_then],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert_eq!(wps.len(), 2);
        assert_eq!(wps[0].kind, WaypointKind::Branching);
        assert_eq!(wps[0].constraint.as_ref().unwrap().value, "true");
    }

    /// `if (a || b)`: two IR decisions on one statement line. The format wants
    /// ONE branching waypoint whose value is that of the whole controlling
    /// expression — here `a` false, `b` true ⇒ the statement takes `if.then`
    /// ⇒ `true` — not one `false` and one `true`.
    #[test]
    fn short_circuit_chain_folds_into_one_waypoint_with_the_statement_value() {
        let (f_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (b0, lor, if_then, if_end) = (
            BlockId::new(10),
            BlockId::new(11),
            BlockId::new(12),
            BlockId::new(13),
        );
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            f_id,
            "main",
            vec![
                blk_named(b0, "while.end", vec![cond_br(2, if_then, lor, 9, 7)]),
                blk_named(
                    lor,
                    "lor.lhs.false",
                    vec![cond_br(3, if_then, if_end, 9, 12)],
                ),
                blk_named(
                    if_then,
                    "if.then",
                    vec![err_at(reach_iid, err_id, 10, 5), ret(4)],
                ),
                blk_named(if_end, "if.end", vec![ret(5)]),
            ],
            b0,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![b0, lor, if_then],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        let branchings: Vec<_> = wps
            .iter()
            .filter(|w| w.kind == WaypointKind::Branching)
            .collect();
        assert_eq!(
            branchings.len(),
            1,
            "one waypoint per branching statement\n{wps:?}"
        );
        assert_eq!(branchings[0].line, 9);
        assert_eq!(branchings[0].constraint.as_ref().unwrap().value, "true");
        assert_eq!(wps.last().unwrap().kind, WaypointKind::Target);
    }

    #[test]
    fn call_close_paren_column_finds_the_matching_paren() {
        let lines = [
            "int main() {",
            "  x = f(g(1), 2);",
            "  int x = __VERIFIER_nondet_int();",
        ];
        // `f(` starts at column 7; its `)` closes after the nested `g(1)`.
        assert_eq!(call_close_paren_column(&lines, 2, 7), Some(16));
        assert_eq!(call_close_paren_column(&lines, 3, 11), Some(33));
        assert_eq!(call_close_paren_column(&lines, 99, 1), None);
        assert_eq!(
            call_close_paren_column(&["x = f(1;"], 1, 5),
            None,
            "unbalanced"
        );
    }

    /// A column-less `function_return` crashes CPAchecker 4.2.2's parser and
    /// voids the witness, so without source text the read is NOT pinned and the
    /// witness degrades to branching + target.
    #[test]
    fn function_return_is_dropped_when_no_source_text_gives_its_column() {
        let (main_id, err_id, nd_id) = (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let (bb0, bb1, bb2) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(
                    bb0,
                    vec![nondet_at(50, nd_id, 5, 11), cond_br(2, bb1, bb2, 6, 9)],
                ),
                blk(bb1, vec![err_at(reach_iid, err_id, 7, 5), ret(4)]),
                blk(bb2, vec![ret(5)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        m.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));
        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb1],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![nc("__VERIFIER_nondet_int", 42, 50)],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert!(
            wps.iter().all(|w| w.kind != WaypointKind::FunctionReturn),
            "{wps:?}"
        );
        assert_eq!(wps.len(), 2, "branching + target");
    }

    // ---- function_return waypoints pinning nondet inputs (confirmation gap) ----

    /// A `CallDirect` (to `callee`) with a source span — a nondet read whose
    /// `function_return` waypoint the lowering emits.
    fn nondet_at(iid: u128, callee: FunctionId, line: u32, col: u32) -> Instruction {
        Instruction::new(InstId::new(iid), Operation::CallDirect { callee })
            .with_span(point_span(line, col))
    }

    fn nc(name: &str, value: i64, call_inst: u128) -> crate::property::NondetCall {
        crate::property::NondetCall {
            func_name: name.to_string(),
            value,
            call_inst: Some(InstId::new(call_inst)),
        }
    }

    #[test]
    fn lower_candidate_emits_function_return_for_path_nondet_before_branch() {
        // main: x = nondet(); if (x == 42) reach_error();
        let (main_id, err_id, nd_id) = (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let (bb0, bb1, bb2) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let reach_iid = InstId::new(33);
        let nd_iid = 50u128;
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(
                    bb0,
                    vec![nondet_at(nd_iid, nd_id, 5, 11), cond_br(2, bb1, bb2, 6, 9)],
                ),
                blk(bb1, vec![err_at(reach_iid, err_id, 7, 5), ret(4)]),
                blk(bb2, vec![ret(5)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        m.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb1],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![nc("__VERIFIER_nondet_int", 42, nd_iid)],
        };
        const SRC: &str = "int main(void) {\n  int x;\n  int y;\n  x = 0;\n  int x = __VERIFIER_nondet_int();\n  if (x == 42) {\n    reach_error();\n  }\n}\n";
        let wps = lower_candidate(&m, &cand, Some(SRC)).expect("lowered");
        // function_return (read) -> branching (the if) -> target, in exec order.
        assert_eq!(wps.len(), 3, "fr + branching + target");
        assert_eq!(wps[0].kind, WaypointKind::FunctionReturn);
        assert_eq!(wps[0].line, 5);
        assert_eq!(
            wps[0].column,
            Some(33),
            "function_return sits at the closing paren"
        );
        assert_eq!(
            wps[0].constraint,
            Some(Constraint {
                format: Some("acsl_expression".into()),
                value: "\\result == 42".into(),
            })
        );
        assert_eq!(wps[1].kind, WaypointKind::Branching);
        assert_eq!(wps[1].line, 6);
        assert_eq!(wps[2].kind, WaypointKind::Target);
        assert_eq!(wps[2].line, 7);
    }

    #[test]
    fn lower_candidate_emits_interproc_caller_nondet_as_prefix() {
        // main: y = nondet(); f();     f: reach_error();
        // block_path covers only f (the error function); the caller read must
        // still be pinned, emitted as a prefix in whole-program order.
        let (main_id, f_id, err_id, nd_id) = (
            FunctionId::new(1),
            FunctionId::new(2),
            FunctionId::new(3),
            FunctionId::new(4),
        );
        let (bb_main, bb_f) = (BlockId::new(10), BlockId::new(20));
        let (reach_iid, nd_iid, call_iid) = (InstId::new(33), 60u128, 61u128);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(
                bb_main,
                vec![
                    nondet_at(nd_iid, nd_id, 3, 13),
                    Instruction::new(
                        InstId::new(call_iid),
                        Operation::CallDirect { callee: f_id },
                    ),
                    ret(2),
                ],
            )],
            bb_main,
        ));
        m.functions.push(func(
            f_id,
            "f",
            vec![blk(bb_f, vec![err_at(reach_iid, err_id, 9, 3), ret(5)])],
            bb_f,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        m.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb_f],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![nc("__VERIFIER_nondet_int", 7, nd_iid)],
        };
        const SRC: &str =
            "int main(void) {\n  int y;\n    int y = __VERIFIER_nondet_int();\n  f();\n}\n";
        let wps = lower_candidate(&m, &cand, Some(SRC)).expect("lowered");
        assert_eq!(wps.len(), 2, "caller fr prefix + target");
        assert_eq!(wps[0].kind, WaypointKind::FunctionReturn);
        assert_eq!(wps[0].line, 3);
        assert_eq!(wps[0].column, Some(35));
        assert_eq!(wps[0].constraint.as_ref().unwrap().value, "\\result == 7");
        assert_eq!(wps[1].kind, WaypointKind::Target);
        assert_eq!(wps[1].line, 9);
    }

    #[test]
    fn lower_candidate_skips_nondet_without_call_inst() {
        // A fuzz/CBMC-derived sequence (call_inst == None) yields no
        // function_return waypoint — degrade to the target-only witness.
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let bb = BlockId::new(10);
        let reach_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(bb, vec![err_at(reach_iid, err_id, 12, 7), ret(4)])],
            bb,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![crate::property::NondetCall {
                func_name: "__VERIFIER_nondet_int".into(),
                value: 1,
                call_inst: None,
            }],
        };
        let wps = lower_candidate(&m, &cand, None).expect("lowered");
        assert_eq!(wps.len(), 1, "no fr without an AIR identity");
        assert_eq!(wps[0].kind, WaypointKind::Target);
    }

    #[test]
    fn lower_candidate_function_return_witness_is_byte_stable_and_lints_shape() {
        use crate::property_kind::{DataModel, Language};
        use crate::witness_yaml::{ViolationWitness, WitnessMeta};

        let (main_id, err_id, nd_id) = (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let (bb0, bb1, bb2) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let reach_iid = InstId::new(33);
        let nd_iid = 50u128;
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(
                    bb0,
                    vec![nondet_at(nd_iid, nd_id, 5, 11), cond_br(2, bb1, bb2, 6, 9)],
                ),
                blk(bb1, vec![err_at(reach_iid, err_id, 7, 5), ret(4)]),
                blk(bb2, vec![ret(5)]),
            ],
            bb0,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        m.functions.push(decl(nd_id, "__VERIFIER_nondet_int"));
        let cand = FalseCandidate {
            reach_error_inst: reach_iid,
            block_path: vec![bb0, bb1],
            assignments: BTreeMap::new(),
            nondet_sequence: vec![nc("__VERIFIER_nondet_int", 42, nd_iid)],
        };
        const SRC: &str = "int main(void) {\n  int x;\n  int y;\n  x = 0;\n  int x = __VERIFIER_nondet_int();\n  if (x == 42) {\n    reach_error();\n  }\n}\n";
        let wps = lower_candidate(&m, &cand, Some(SRC)).expect("lowered");
        let meta = WitnessMeta {
            producer_version: "0.1.0".to_string(),
            specification: "CHECK( init(main()), LTL(G ! call(reach_error())) )".to_string(),
            data_model: DataModel::ILP32,
            language: Language::C,
            input_file: std::path::PathBuf::from("/nonexistent/t.c"),
        };
        let serialize = || {
            ViolationWitness::assemble(&meta, &wps)
                .expect("assemble")
                .to_yaml_string()
                .expect("serialize")
        };
        let (y1, y2) = (serialize(), serialize());
        assert_eq!(y1, y2, "enriched witness must be byte-stable");
        assert_eq!(
            y1.matches("type: function_return").count(),
            1,
            "exactly one function_return waypoint\n{y1}"
        );
        assert!(y1.contains("format: acsl_expression"), "{y1}");
        assert!(y1.contains("\\result == 42"), "{y1}");
        assert!(y1.contains("type: target"), "{y1}");
    }
}
