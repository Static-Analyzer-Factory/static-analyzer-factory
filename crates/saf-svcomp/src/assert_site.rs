//! Detection of interprocedural **assert-wrapper** sites for the `unreach-call`
//! BMC engines ([`crate::bmc`] fixed-k and [`crate::bmc_incremental`]).
//!
//! # The gap this closes
//!
//! Both BMC engines (and the forward SE engine) root their search at the function
//! that *contains* the `reach_error` call. But the single most common SV-COMP
//! violation shape puts `reach_error` inside a tiny **assertion wrapper**:
//!
//! ```c
//! void __VERIFIER_assert(int cond) { if (!cond) { reach_error(); } }
//! int main(void) {
//!     unsigned x = __VERIFIER_nondet_uint();
//!     ... /* loop / arithmetic computing a condition */ ...
//!     __VERIFIER_assert(P);          // fails exactly when P == 0
//! }
//! ```
//!
//! Here the wrapper `__VERIFIER_assert` has *no* nondet input and *no* loop, so the
//! BMC/SE gates reject it; and the caller `main` — which owns the nondet input, the
//! loop, and the arithmetic that decides `P` — is never chosen as a root because it
//! contains no `reach_error` call of its own. So the whole class is missed by the
//! solver tail, and only a lucky blind-fuzz run over the raw nondet bytes can catch
//! it.
//!
//! # What a site is
//!
//! This module recognises a wrapper `W` that reaches `reach_error` **exactly when
//! its argument is zero/false** (the `if (!cond)` idiom, in either `icmp` polarity),
//! and reports each call `W(arg)` in a caller `C` as a [`AssertSite`]: rooting the
//! BMC search at `C`, reaching the call block, and additionally constraining
//! `arg == 0`, is equivalent to reaching `reach_error`. The caller's own nondet
//! reads / loop / arithmetic then drive the guard — exactly what the intraprocedural
//! encoders already know how to do, with a single extra constraint at the target.
//!
//! # Soundness
//!
//! The detection is a *heuristic that only proposes candidates*: a wrong wrapper
//! classification, or the wrong `arg == 0` direction, yields a model whose native
//! replay simply fails to reach `reach_error` → `unknown`. Native replay of the
//! ORIGINAL (unmodified) program remains the sole arbiter of FALSE (confirmer
//! contract R6), so nothing here can produce a wrong verdict — only recall.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirBlock, AirFunction, AirModule, BinaryOp, Constant, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};

use crate::property::REACH_ERROR_NAMES;

/// A virtual interprocedural error target: within `caller`, control reaches the
/// wrapper call in `call_block`, and passing `guard_arg == 0` makes the wrapper
/// fall into its `reach_error` branch. Rooting a BMC search at `caller`, reaching
/// `call_block`, and asserting `guard_arg == 0` is equivalent to reaching the
/// wrapper's `reach_error` (`reach_error_inst`, kept for the witness anchor).
#[derive(Debug, Clone)]
pub(crate) struct AssertSite {
    /// The calling function to root the BMC search in (e.g. `main`).
    pub caller: FunctionId,
    /// The block in `caller` that contains the wrapper call.
    pub call_block: BlockId,
    /// The caller SSA value passed as the wrapper's condition argument.
    pub guard_arg: ValueId,
    /// The `reach_error` / `__VERIFIER_error` call instruction inside the wrapper,
    /// used only to anchor the violation witness (it carries the source span).
    pub reach_error_inst: InstId,
}

/// Enumerate every interprocedural assert-wrapper site in the module, in
/// deterministic module-walk order.
///
/// A wrapper is a defined function with exactly one integer parameter that reaches
/// a `reach_error` call **on the branch taken when that parameter is zero/false**
/// (the `assert(cond)` / `if(!cond) reach_error()` idiom). For each such wrapper,
/// every direct call site in another defined function becomes a site.
#[must_use]
pub(crate) fn virtual_assert_sites(module: &AirModule) -> Vec<AssertSite> {
    // Wrapper function id -> the reach_error inst inside it (for the witness).
    let mut wrappers: BTreeMap<FunctionId, InstId> = BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        if let Some(err_inst) = wrapper_fails_on_zero_arg(func, module) {
            wrappers.insert(func.id, err_inst);
        }
    }
    if wrappers.is_empty() {
        return Vec::new();
    }

    let mut sites = Vec::new();
    for caller in &module.functions {
        if caller.is_declaration {
            continue;
        }
        for block in &caller.blocks {
            for inst in &block.instructions {
                let Operation::CallDirect { callee } = &inst.op else {
                    continue;
                };
                // A wrapper never roots at itself; a self-call would be a recursive
                // wrapper we do not model.
                if *callee == caller.id {
                    continue;
                }
                let Some(&err_inst) = wrappers.get(callee) else {
                    continue;
                };
                let Some(&guard_arg) = inst.operands.first() else {
                    continue;
                };
                sites.push(AssertSite {
                    caller: caller.id,
                    call_block: block.id,
                    guard_arg,
                    reach_error_inst: err_inst,
                });
            }
        }
    }
    sites
}

/// If `func` is an assert-wrapper that reaches `reach_error` exactly on the
/// argument-is-zero branch, return the `reach_error` call instruction inside it.
///
/// Requirements (all structural, all deterministic):
/// - exactly one parameter `p`;
/// - a `reach_error` / `__VERIFIER_error` call somewhere in the body;
/// - a conditional branch whose condition is `p == 0`, `p != 0`, or `p`'s
///   truthiness (via `icmp`/`trunc`/`copy`), such that the successor taken when
///   `p == 0` can reach the `reach_error` block.
fn wrapper_fails_on_zero_arg(func: &AirFunction, module: &AirModule) -> Option<InstId> {
    if func.params.len() != 1 {
        return None;
    }
    let param = func.params[0].id;

    // The reach_error block + its call instruction.
    let (err_block, err_inst) = find_reach_error(func, module)?;

    // Index defs so we can inspect the branch condition's definition.
    let mut def_map: BTreeMap<ValueId, &Instruction> = BTreeMap::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                def_map.insert(dst, inst);
            }
        }
    }

    let succ = successor_map(func);
    for block in &func.blocks {
        let Some(term) = block.terminator() else {
            continue;
        };
        let Operation::CondBr {
            then_target,
            else_target,
        } = &term.op
        else {
            continue;
        };
        let Some(&cond) = term.operands.first() else {
            continue;
        };
        // Which edge is taken when the parameter is zero/false?
        let Some(zero_target) =
            zero_edge(cond, param, *then_target, *else_target, &def_map, module)
        else {
            continue;
        };
        // Reaching reach_error from that edge confirms "fails when arg == 0".
        if reaches(zero_target, err_block, &succ) {
            return Some(err_inst);
        }
    }
    None
}

/// The `reach_error` / `__VERIFIER_error` call `(block, inst)` in `func`, if any
/// (first in deterministic walk order).
fn find_reach_error(func: &AirFunction, module: &AirModule) -> Option<(BlockId, InstId)> {
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::CallDirect { callee } = &inst.op {
                if module
                    .function(*callee)
                    .is_some_and(|t| REACH_ERROR_NAMES.contains(&t.name.as_str()))
                {
                    return Some((block.id, inst.id));
                }
            }
        }
    }
    None
}

/// Given a `CondBr(cond) ? then_target : else_target`, return the successor taken
/// when `param == 0` (false), if `cond` is a recognizable predicate over `param`.
fn zero_edge(
    cond: ValueId,
    param: ValueId,
    then_target: BlockId,
    else_target: BlockId,
    def_map: &BTreeMap<ValueId, &Instruction>,
    module: &AirModule,
) -> Option<BlockId> {
    // Peel identity casts/copies/freezes off the condition value.
    let cur = peel_identity(cond, def_map);

    // Case A: the condition IS the parameter's truthiness (`br i1 %p`): true when
    // p != 0, so the p == 0 edge is the else branch.
    if cur == param {
        return Some(else_target);
    }

    // Case B: the condition is an icmp against the parameter and zero.
    let inst = def_map.get(&cur)?;
    let Operation::BinaryOp { kind } = &inst.op else {
        return None;
    };
    if inst.operands.len() < 2 {
        return None;
    }
    let a = peel_identity(inst.operands[0], def_map);
    let b = peel_identity(inst.operands[1], def_map);
    let (param_is_a, param_is_b) = (a == param, b == param);
    if !(param_is_a || param_is_b) {
        return None;
    }
    let other = if param_is_a { b } else { a };
    if !const_is_zero(module, other) {
        return None;
    }
    match kind {
        // p == 0 : true edge (then) is the zero edge.
        BinaryOp::ICmpEq => Some(then_target),
        // p != 0 : false edge (else) is the zero edge.
        BinaryOp::ICmpNe => Some(else_target),
        _ => None,
    }
}

/// Peel identity casts/copies/freezes off a value, returning the underlying value.
fn peel_identity(mut vid: ValueId, def_map: &BTreeMap<ValueId, &Instruction>) -> ValueId {
    for _ in 0..8 {
        let Some(inst) = def_map.get(&vid) else { break };
        match &inst.op {
            Operation::Cast { .. } | Operation::Copy | Operation::Freeze => {
                match inst.operands.first() {
                    Some(&o) => vid = o,
                    None => break,
                }
            }
            _ => break,
        }
    }
    vid
}

/// Deterministic successor map for `func`.
fn successor_map(func: &AirFunction) -> BTreeMap<BlockId, Vec<BlockId>> {
    func.blocks
        .iter()
        .map(|b| (b.id, block_successors(b)))
        .collect()
}

/// The successor blocks named by `block`'s terminator (deduplicated, ordered).
fn block_successors(block: &AirBlock) -> Vec<BlockId> {
    match block.terminator().map(|t| &t.op) {
        Some(Operation::Br { target }) => vec![*target],
        Some(Operation::CondBr {
            then_target,
            else_target,
        }) => {
            if then_target == else_target {
                vec![*then_target]
            } else {
                vec![*then_target, *else_target]
            }
        }
        Some(Operation::Switch { default, cases }) => {
            let mut out: BTreeSet<BlockId> = BTreeSet::new();
            out.insert(*default);
            for (_, t) in cases {
                out.insert(*t);
            }
            out.into_iter().collect()
        }
        _ => Vec::new(),
    }
}

/// Simple forward reachability `from → target` over the successor map.
fn reaches(from: BlockId, target: BlockId, succ: &BTreeMap<BlockId, Vec<BlockId>>) -> bool {
    let mut seen: BTreeSet<BlockId> = BTreeSet::new();
    let mut stack = vec![from];
    while let Some(b) = stack.pop() {
        if b == target {
            return true;
        }
        if !seen.insert(b) {
            continue;
        }
        for &s in succ.get(&b).map_or(&[][..], Vec::as_slice) {
            stack.push(s);
        }
    }
    false
}

/// Is `vid` the integer constant zero (a `Constant::Int{0}` / `Null` / `ZeroInit`)?
fn const_is_zero(module: &AirModule, vid: ValueId) -> bool {
    matches!(
        module.constants.get(&vid),
        Some(Constant::Int { value: 0, .. } | Constant::Null | Constant::ZeroInit)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::AirParam;
    use saf_core::ids::{InstId, ModuleId};

    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }
    fn bid(n: u128) -> BlockId {
        BlockId::new(n)
    }
    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }
    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }

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

    fn blk(id: BlockId, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id,
            label: None,
            instructions,
        }
    }

    /// `void W(int cond){ if(!cond){reach_error();} }`, compiled with the
    /// `icmp ne cond,0 -> br (ok,error)` polarity, plus the given wrapper name.
    fn assert_wrapper(w_id: FunctionId, err_id: FunctionId, name: &str) -> AirFunction {
        let (entry, ok_bb, err_bb) = (bid(20), bid(21), bid(22));
        let cond = vid(200);
        let zero = vid(201);
        let tobool = vid(202);
        let entry_insts = vec![
            Instruction::new(
                iid(30),
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpNe,
                },
            )
            .with_operands(vec![cond, zero])
            .with_dst(tobool),
            Instruction::new(
                iid(31),
                Operation::CondBr {
                    then_target: ok_bb,
                    else_target: err_bb,
                },
            )
            .with_operands(vec![tobool]),
        ];
        AirFunction {
            id: w_id,
            name: name.into(),
            params: vec![AirParam::new(cond, 0)],
            blocks: vec![
                blk(entry, entry_insts),
                blk(ok_bb, vec![Instruction::new(iid(32), Operation::Ret)]),
                blk(
                    err_bb,
                    vec![
                        Instruction::new(iid(33), Operation::CallDirect { callee: err_id }),
                        Instruction::new(iid(34), Operation::Ret),
                    ],
                ),
            ],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn base_module(zero: ValueId) -> AirModule {
        let mut m = AirModule::new(ModuleId::new(1));
        m.constants
            .insert(zero, Constant::Int { value: 0, bits: 32 });
        m
    }

    #[test]
    fn detects_ne_polarity_wrapper_and_its_call_site() {
        let (main_id, w_id, err_id) = (fid(1), fid(2), fid(3));
        let call_block = bid(10);
        let arg = vid(100);
        let main = {
            let insts = vec![
                Instruction::new(iid(1), Operation::CallDirect { callee: w_id })
                    .with_operands(vec![arg]),
                Instruction::new(iid(2), Operation::Ret),
            ];
            AirFunction {
                id: main_id,
                name: "main".into(),
                params: vec![],
                blocks: vec![blk(call_block, insts)],
                entry_block: Some(call_block),
                is_declaration: false,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            }
        };
        let mut m = base_module(vid(201));
        m.functions.push(main);
        m.functions
            .push(assert_wrapper(w_id, err_id, "__VERIFIER_assert"));
        m.functions.push(decl(err_id, "reach_error"));

        let sites = virtual_assert_sites(&m);
        assert_eq!(sites.len(), 1, "one wrapper call site");
        assert_eq!(sites[0].caller, main_id);
        assert_eq!(sites[0].call_block, call_block);
        assert_eq!(sites[0].guard_arg, arg);
        assert_eq!(sites[0].reach_error_inst, iid(33));
    }

    #[test]
    fn detects_eq_polarity_wrapper() {
        // `if(cond==0) reach_error();` — icmp eq -> then=error edge.
        let (main_id, w_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, err_bb, ok_bb) = (bid(20), bid(21), bid(22));
        let cond = vid(200);
        let zero = vid(201);
        let iseq = vid(202);
        let w = AirFunction {
            id: w_id,
            name: "assert_cond".into(),
            params: vec![AirParam::new(cond, 0)],
            blocks: vec![
                blk(
                    entry,
                    vec![
                        Instruction::new(
                            iid(30),
                            Operation::BinaryOp {
                                kind: BinaryOp::ICmpEq,
                            },
                        )
                        .with_operands(vec![cond, zero])
                        .with_dst(iseq),
                        Instruction::new(
                            iid(31),
                            Operation::CondBr {
                                then_target: err_bb,
                                else_target: ok_bb,
                            },
                        )
                        .with_operands(vec![iseq]),
                    ],
                ),
                blk(
                    err_bb,
                    vec![Instruction::new(
                        iid(33),
                        Operation::CallDirect { callee: err_id },
                    )],
                ),
                blk(ok_bb, vec![Instruction::new(iid(34), Operation::Ret)]),
            ],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let main = AirFunction {
            id: main_id,
            name: "main".into(),
            params: vec![],
            blocks: vec![blk(
                bid(10),
                vec![
                    Instruction::new(iid(1), Operation::CallDirect { callee: w_id })
                        .with_operands(vec![vid(100)]),
                    Instruction::new(iid(2), Operation::Ret),
                ],
            )],
            entry_block: Some(bid(10)),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut m = base_module(zero);
        m.functions.push(main);
        m.functions.push(w);
        m.functions.push(decl(err_id, "reach_error"));
        let sites = virtual_assert_sites(&m);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].guard_arg, vid(100));
    }

    #[test]
    fn two_param_wrapper_is_rejected() {
        // A wrapper with 2 params is not the recognized single-condition idiom.
        let (main_id, w_id, err_id) = (fid(1), fid(2), fid(3));
        let (entry, err_bb, ok_bb) = (bid(20), bid(21), bid(22));
        let (p0, p1, zero, tobool) = (vid(200), vid(210), vid(201), vid(202));
        let w = AirFunction {
            id: w_id,
            name: "two".into(),
            params: vec![AirParam::new(p0, 0), AirParam::new(p1, 1)],
            blocks: vec![
                blk(
                    entry,
                    vec![
                        Instruction::new(
                            iid(30),
                            Operation::BinaryOp {
                                kind: BinaryOp::ICmpNe,
                            },
                        )
                        .with_operands(vec![p0, zero])
                        .with_dst(tobool),
                        Instruction::new(
                            iid(31),
                            Operation::CondBr {
                                then_target: ok_bb,
                                else_target: err_bb,
                            },
                        )
                        .with_operands(vec![tobool]),
                    ],
                ),
                blk(ok_bb, vec![Instruction::new(iid(32), Operation::Ret)]),
                blk(
                    err_bb,
                    vec![Instruction::new(
                        iid(33),
                        Operation::CallDirect { callee: err_id },
                    )],
                ),
            ],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let main = AirFunction {
            id: main_id,
            name: "main".into(),
            params: vec![],
            blocks: vec![blk(
                bid(10),
                vec![
                    Instruction::new(iid(1), Operation::CallDirect { callee: w_id })
                        .with_operands(vec![vid(100), vid(101)]),
                    Instruction::new(iid(2), Operation::Ret),
                ],
            )],
            entry_block: Some(bid(10)),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut m = base_module(zero);
        m.functions.push(main);
        m.functions.push(w);
        m.functions.push(decl(err_id, "reach_error"));
        assert!(virtual_assert_sites(&m).is_empty());
    }

    #[test]
    fn plain_function_without_param_guard_is_not_a_wrapper() {
        // A function that unconditionally calls reach_error is a must-reach case,
        // not a conditional assert wrapper -> no zero-guard edge -> rejected.
        let (main_id, w_id, err_id) = (fid(1), fid(2), fid(3));
        let w = AirFunction {
            id: w_id,
            name: "boom".into(),
            params: vec![AirParam::new(vid(200), 0)],
            blocks: vec![blk(
                bid(20),
                vec![
                    Instruction::new(iid(30), Operation::CallDirect { callee: err_id }),
                    Instruction::new(iid(31), Operation::Ret),
                ],
            )],
            entry_block: Some(bid(20)),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let main = AirFunction {
            id: main_id,
            name: "main".into(),
            params: vec![],
            blocks: vec![blk(
                bid(10),
                vec![
                    Instruction::new(iid(1), Operation::CallDirect { callee: w_id })
                        .with_operands(vec![vid(100)]),
                    Instruction::new(iid(2), Operation::Ret),
                ],
            )],
            entry_block: Some(bid(10)),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(main);
        m.functions.push(w);
        m.functions.push(decl(err_id, "reach_error"));
        assert!(virtual_assert_sites(&m).is_empty());
    }
}
