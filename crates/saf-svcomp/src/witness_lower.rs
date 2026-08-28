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

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction, Operation};
use saf_core::ids::{BlockId, InstId};
use saf_core::span::Span;

use crate::property::{FalseCandidate, REACH_ERROR_NAMES};
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
/// Located by LINE ONLY (column omitted). A column-less `function_return`
/// waypoint is resolved by the validator to the right parenthesis of the (first
/// suitable) call on that line; for the overwhelmingly common one-nondet-per-line
/// shape (`x = __VERIFIER_nondet_int();`) that is exactly this call, and dropping
/// the column mirrors the branching waypoints (a column into the wrong token is a
/// rejection risk). Returns `None` if the call instruction or its span cannot be
/// resolved, degrading toward the always-valid target-only witness.
fn function_return_waypoint(
    module: &AirModule,
    call_inst: InstId,
    value: i64,
) -> Option<SourceWaypoint> {
    let (func, inst) = find_inst(module, call_inst)?;
    let (file_name, line, _col) = span_to_location(module, inst.span.as_ref()?)?;
    Some(SourceWaypoint {
        kind: WaypointKind::FunctionReturn,
        action: Action::Follow,
        file_name,
        line,
        column: None,
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

/// The `branching` waypoint for the `CondBr` decision on the `from -> to` edge,
/// or `None` when the decision cannot be unambiguously recovered (non-`CondBr`
/// terminator, `then == else`, a pair that is not an edge, or an unresolved
/// span). Factored out of [`lower_candidate`] so the block walk can interleave it
/// with the nondet `function_return` waypoints.
fn branching_for_edge(
    module: &AirModule,
    from_block: &AirBlock,
    from_func: &AirFunction,
    to: BlockId,
) -> Option<SourceWaypoint> {
    let terminator = from_block.terminator()?;
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
    let taken = if to == *then_target {
        true
    } else if to == *else_target {
        false
    } else {
        // Path pair is not an edge of this CondBr — do not guess.
        return None;
    };
    let (file_name, line, _col) = terminator
        .span
        .as_ref()
        .and_then(|s| span_to_location(module, s))?;
    Some(branching_waypoint(
        file_name,
        line,
        Some(from_func.name.clone()),
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
pub fn lower_candidate(module: &AirModule, cand: &FalseCandidate) -> Option<Vec<SourceWaypoint>> {
    let (func, inst) = find_inst(module, cand.reach_error_inst)?;
    let target_location = span_to_location(module, inst.span.as_ref()?)?;

    // Value to pin at each nondet call site (first occurrence wins; a Z3/interproc
    // path is simple so every scalar-int read appears exactly once).
    let mut nondet_value: BTreeMap<InstId, i64> = BTreeMap::new();
    for nc in &cand.nondet_sequence {
        if let Some(id) = nc.call_inst {
            nondet_value.entry(id).or_insert(nc.value);
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
        if inpath_insts.contains(&id) {
            continue; // emitted during the block walk below, in place
        }
        if let Some(wp) = function_return_waypoint(module, id, nc.value) {
            waypoints.push(wp);
        }
    }

    // 2. Walk the error function's path: each block's nondet reads (instruction
    //    order) before that block's branching decision.
    for (i, bid) in cand.block_path.iter().enumerate() {
        let Some((bfunc, block)) = find_block_in_module(module, *bid) else {
            continue;
        };
        for ins in &block.instructions {
            if let Some(&value) = nondet_value.get(&ins.id) {
                if let Some(wp) = function_return_waypoint(module, ins.id, value) {
                    waypoints.push(wp);
                }
            }
        }
        if let Some(&next) = cand.block_path.get(i + 1) {
            if let Some(wp) = branching_for_edge(module, block, bfunc, next) {
                waypoints.push(wp);
            }
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
        // function_return (read) -> branching (the if) -> target, in exec order.
        assert_eq!(wps.len(), 3, "fr + branching + target");
        assert_eq!(wps[0].kind, WaypointKind::FunctionReturn);
        assert_eq!(wps[0].line, 5);
        assert_eq!(wps[0].column, None, "function_return must be line-only");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
        assert_eq!(wps.len(), 2, "caller fr prefix + target");
        assert_eq!(wps[0].kind, WaypointKind::FunctionReturn);
        assert_eq!(wps[0].line, 3);
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
        let wps = lower_candidate(&m, &cand).expect("lowered");
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
