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
//! the `if`/`while` keyword (plan 195 Tier A). See [`lower_candidate`].

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

/// Lower a replay-confirmed [`FalseCandidate`] to a waypoint list: `branching`
/// waypoints for each `CondBr` decision along the confirmed path (plan 195
/// Tier A), followed by the `target` at the `reach_error` call.
///
/// Each branching waypoint is located by LINE ONLY (see [`branching_waypoint`]).
/// The derivation is **strictly additive**: any hop whose decision cannot be
/// unambiguously recovered — a non-`CondBr` terminator, `then == else`, a path
/// pair that is not a CFG edge, or an unresolved span — is skipped, degrading
/// toward the always-valid target-only witness (never an invalid constraint that
/// would make CPAchecker reject the whole witness). Returns `None` if the
/// `reach_error` instruction or its span cannot be resolved.
#[must_use]
pub fn lower_candidate(module: &AirModule, cand: &FalseCandidate) -> Option<Vec<SourceWaypoint>> {
    let (func, inst) = find_inst(module, cand.reach_error_inst)?;
    let target_location = span_to_location(module, inst.span.as_ref()?)?;

    let mut waypoints = Vec::new();
    for pair in cand.block_path.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let Some((bfunc, block)) = find_block_in_module(module, from) else {
            continue;
        };
        let Some(terminator) = block.terminator() else {
            continue;
        };
        let Operation::CondBr {
            then_target,
            else_target,
        } = &terminator.op
        else {
            continue;
        };
        // Ambiguous when both edges go to the same block.
        if then_target == else_target {
            continue;
        }
        let taken = if to == *then_target {
            true
        } else if to == *else_target {
            false
        } else {
            // Path pair is not an edge of this CondBr — do not guess.
            continue;
        };
        let Some((file_name, line, _col)) = terminator
            .span
            .as_ref()
            .and_then(|s| span_to_location(module, s))
        else {
            continue;
        };
        waypoints.push(branching_waypoint(
            file_name,
            line,
            Some(bfunc.name.clone()),
            taken,
        ));
    }
    waypoints.push(target_waypoint(target_location, Some(func.name.clone())));
    Some(waypoints)
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
}
