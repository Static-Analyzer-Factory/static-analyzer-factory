//! AIR → source lowering for SV-COMP violation witnesses.
//!
//! Turns the `unreach-call` evidence — a `must_reach_error` block chain or a
//! replay-confirmed [`FalseCandidate`] — into the flat, ordered
//! [`SourceWaypoint`] list that
//! [`crate::witness_yaml::ViolationWitness::assemble`] serializes. This is the
//! only witness code that touches AIR; the YAML model in `witness_yaml` stays
//! property-generic.
//!
//! Emits a single `target` waypoint at the `reach_error()` call site. This is
//! sound and — as validated in plan 194 Slice E — CPAchecker confirms it for
//! both unconditional and guarded/nondet violations. `branching`/`assumption`
//! enrichment was tried and dropped (it broke CPAchecker confirmation); see
//! [`lower_candidate`] for the details.

use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction, Operation};
use saf_core::ids::{BlockId, InstId};
use saf_core::span::Span;

use crate::property::{FalseCandidate, REACH_ERROR_NAMES};
use crate::witness_yaml::{Action, SourceWaypoint, WaypointKind};

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

/// Lower a replay-confirmed [`FalseCandidate`] to a target-only waypoint list.
///
/// Emits a single `target` waypoint at the `reach_error` call — which CPAchecker
/// confirms for both unconditional and guarded/nondet violations (its
/// analysis-based validation finds the feasible path to the target on its own).
///
/// Branching enrichment was implemented and then dropped after validation
/// (plan 194 Slice E): CPAchecker requires a `branching` waypoint at the
/// `if`/`while` KEYWORD, but the AIR `CondBr` span points into the condition
/// expression — CPAchecker mis-parses that as a ternary and REJECTS the whole
/// witness, making it strictly worse than target-only. `assumption`-value
/// waypoints are likewise infeasible (no reliable C variable name after
/// `mem2reg`). Correct keyword-location / name recovery is future work. Returns
/// `None` if the `reach_error` instruction or its span cannot be resolved.
#[must_use]
pub fn lower_candidate(module: &AirModule, cand: &FalseCandidate) -> Option<Vec<SourceWaypoint>> {
    let (func, inst) = find_inst(module, cand.reach_error_inst)?;
    let location = span_to_location(module, inst.span.as_ref()?)?;
    Some(vec![target_waypoint(location, Some(func.name.clone()))])
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
        let err_iid = InstId::new(33);
        let mut m = AirModule::new(ModuleId::new(1));
        m.source_files.push(SourceFile::new(FileId::new(1), "t.c"));
        let err_call = Instruction::new(err_iid, Operation::CallDirect { callee: err_id })
            .with_span(point_span(33, 7));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(bb, vec![err_call, ret(4)])],
            bb,
        ));
        m.functions.push(decl(err_id, "reach_error"));

        let cand = FalseCandidate {
            reach_error_inst: err_iid,
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
}
