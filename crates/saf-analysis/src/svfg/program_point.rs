//! Program point tracking for temporal ordering in SVFG.
//!
//! This module provides types for tracking where each `ValueId` is defined
//! in the program (function, block, instruction index) and for checking
//! temporal ordering between program points.
//!
//! The primary use case is filtering false-positive Use-After-Free (UAF)
//! findings where the "use" actually happens BEFORE the "free" in program
//! order, not after.

use std::collections::BTreeMap;

use saf_core::ids::{BlockId, FunctionId, ValueId};

use crate::cfg::Cfg;

/// A specific location in the program.
///
/// Program points track where values are defined, allowing temporal
/// ordering queries to determine if one program point can execute
/// after another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramPoint {
    /// The function containing this program point.
    pub function: FunctionId,
    /// The basic block containing this program point.
    pub block: BlockId,
    /// The instruction index within the block (0-based).
    pub inst_index: u32,
}

impl ProgramPoint {
    /// Create a new program point.
    #[must_use]
    pub const fn new(function: FunctionId, block: BlockId, inst_index: u32) -> Self {
        Self {
            function,
            block,
            inst_index,
        }
    }
}

/// Map from `ValueId` to its defining program point.
///
/// This map is built during SVFG construction and tracks where each
/// SSA value is defined, enabling temporal ordering queries.
#[derive(Debug, Clone, Default)]
pub struct ProgramPointMap {
    points: BTreeMap<ValueId, ProgramPoint>,
}

impl ProgramPointMap {
    /// Create a new empty program point map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            points: BTreeMap::new(),
        }
    }

    /// Insert a program point for a value.
    ///
    /// If the value already has a program point, this does nothing.
    /// The first definition location is preserved.
    pub fn insert(&mut self, value: ValueId, point: ProgramPoint) {
        self.points.entry(value).or_insert(point);
    }

    /// Get the program point for a value.
    #[must_use]
    pub fn get(&self, value: ValueId) -> Option<ProgramPoint> {
        self.points.get(&value).copied()
    }

    /// Check if `later` can execute after `earlier` in program order.
    ///
    /// This is a conservative check for temporal ordering:
    /// - Different functions: returns `true` (conservatively assume reachable)
    /// - Same block: compares instruction indices
    /// - Different blocks: uses CFG reachability
    ///
    /// # Returns
    ///
    /// - `true` if `later` can potentially execute after `earlier`
    /// - `false` if `later` definitely cannot execute after `earlier`
    #[must_use]
    pub fn can_happen_after(
        &self,
        earlier: ProgramPoint,
        later: ProgramPoint,
        cfgs: &BTreeMap<FunctionId, Cfg>,
    ) -> bool {
        // Different functions: conservatively return true
        // (interprocedural temporal ordering is complex)
        if earlier.function != later.function {
            return true;
        }

        // Same block: compare instruction indices
        if earlier.block == later.block {
            return later.inst_index > earlier.inst_index;
        }

        // Different blocks: check CFG reachability
        // If the later block is reachable from the earlier block,
        // then later can happen after earlier
        if let Some(cfg) = cfgs.get(&earlier.function) {
            return cfg.is_reachable(earlier.block, later.block);
        }

        // No CFG available: conservative fallback
        true
    }

    /// Get the number of tracked values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction, Operation};
    use saf_core::ids::InstId;

    fn make_function(id: u128, blocks: Vec<AirBlock>) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: format!("test_{id}"),
            params: Vec::new(),
            blocks,
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_block(id: u128, terminator: Operation) -> AirBlock {
        AirBlock {
            id: BlockId::new(id),
            label: None,
            instructions: vec![Instruction::new(InstId::new(id * 100), terminator)],
        }
    }

    #[test]
    fn program_point_same_block_ordering() {
        let func = FunctionId::new(1);
        let block = BlockId::new(10);

        let pp1 = ProgramPoint::new(func, block, 0);
        let pp2 = ProgramPoint::new(func, block, 5);
        let pp3 = ProgramPoint::new(func, block, 3);

        let map = ProgramPointMap::new();
        let cfgs = BTreeMap::new();

        // pp2 (inst 5) can happen after pp1 (inst 0)
        assert!(map.can_happen_after(pp1, pp2, &cfgs));

        // pp1 (inst 0) cannot happen after pp2 (inst 5)
        assert!(!map.can_happen_after(pp2, pp1, &cfgs));

        // pp3 (inst 3) can happen after pp1 (inst 0)
        assert!(map.can_happen_after(pp1, pp3, &cfgs));

        // pp1 cannot happen after itself (same index)
        assert!(!map.can_happen_after(pp1, pp1, &cfgs));
    }

    #[test]
    fn program_point_different_functions() {
        let func1 = FunctionId::new(1);
        let func2 = FunctionId::new(2);
        let block = BlockId::new(10);

        let pp1 = ProgramPoint::new(func1, block, 0);
        let pp2 = ProgramPoint::new(func2, block, 0);

        let map = ProgramPointMap::new();
        let cfgs = BTreeMap::new();

        // Different functions: conservatively return true
        assert!(map.can_happen_after(pp1, pp2, &cfgs));
        assert!(map.can_happen_after(pp2, pp1, &cfgs));
    }

    #[test]
    fn program_point_different_blocks_reachable() {
        // Create: b1 -> b2 -> b3
        let b1 = make_block(
            1,
            Operation::Br {
                target: BlockId::new(2),
            },
        );
        let b2 = make_block(
            2,
            Operation::Br {
                target: BlockId::new(3),
            },
        );
        let b3 = make_block(3, Operation::Ret);

        let func = make_function(1, vec![b1, b2, b3]);
        let cfg = Cfg::build(&func);

        let mut cfgs = BTreeMap::new();
        cfgs.insert(func.id, cfg);

        let pp1 = ProgramPoint::new(func.id, BlockId::new(1), 0);
        let pp2 = ProgramPoint::new(func.id, BlockId::new(2), 0);
        let pp3 = ProgramPoint::new(func.id, BlockId::new(3), 0);

        let map = ProgramPointMap::new();

        // b2 is reachable from b1
        assert!(map.can_happen_after(pp1, pp2, &cfgs));
        // b3 is reachable from b1
        assert!(map.can_happen_after(pp1, pp3, &cfgs));
        // b3 is reachable from b2
        assert!(map.can_happen_after(pp2, pp3, &cfgs));

        // b1 is NOT reachable from b2 (no back edge)
        assert!(!map.can_happen_after(pp2, pp1, &cfgs));
        // b1 is NOT reachable from b3
        assert!(!map.can_happen_after(pp3, pp1, &cfgs));
    }

    #[test]
    fn program_point_different_blocks_loop() {
        // Create: b1 -> b2 -> b3 -> b2 (back edge)
        let b1 = make_block(
            1,
            Operation::Br {
                target: BlockId::new(2),
            },
        );
        let b2 = make_block(
            2,
            Operation::CondBr {
                then_target: BlockId::new(3),
                else_target: BlockId::new(4),
            },
        );
        let b3 = make_block(
            3,
            Operation::Br {
                target: BlockId::new(2),
            },
        ); // back edge to b2
        let b4 = make_block(4, Operation::Ret);

        let func = make_function(1, vec![b1, b2, b3, b4]);
        let cfg = Cfg::build(&func);

        let mut cfgs = BTreeMap::new();
        cfgs.insert(func.id, cfg);

        let pp2 = ProgramPoint::new(func.id, BlockId::new(2), 0);
        let pp3 = ProgramPoint::new(func.id, BlockId::new(3), 0);

        let map = ProgramPointMap::new();

        // b3 is reachable from b2
        assert!(map.can_happen_after(pp2, pp3, &cfgs));
        // b2 is reachable from b3 (via back edge)
        assert!(map.can_happen_after(pp3, pp2, &cfgs));
    }

    #[test]
    fn program_point_map_insert_and_get() {
        let mut map = ProgramPointMap::new();

        let val = ValueId::new(100);
        let pp = ProgramPoint::new(FunctionId::new(1), BlockId::new(10), 5);

        assert!(map.get(val).is_none());

        map.insert(val, pp);
        assert_eq!(map.get(val), Some(pp));
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
    }

    #[test]
    fn program_point_map_first_wins() {
        let mut map = ProgramPointMap::new();

        let val = ValueId::new(100);
        let pp1 = ProgramPoint::new(FunctionId::new(1), BlockId::new(10), 5);
        let pp2 = ProgramPoint::new(FunctionId::new(1), BlockId::new(10), 10);

        map.insert(val, pp1);
        map.insert(val, pp2); // This should NOT overwrite

        assert_eq!(map.get(val), Some(pp1)); // First insert wins
    }

    #[test]
    fn cfg_is_reachable_same_block() {
        let b1 = make_block(1, Operation::Ret);
        let func = make_function(1, vec![b1]);
        let cfg = Cfg::build(&func);

        assert!(cfg.is_reachable(BlockId::new(1), BlockId::new(1)));
    }

    #[test]
    fn cfg_is_reachable_linear() {
        let b1 = make_block(
            1,
            Operation::Br {
                target: BlockId::new(2),
            },
        );
        let b2 = make_block(
            2,
            Operation::Br {
                target: BlockId::new(3),
            },
        );
        let b3 = make_block(3, Operation::Ret);

        let func = make_function(1, vec![b1, b2, b3]);
        let cfg = Cfg::build(&func);

        assert!(cfg.is_reachable(BlockId::new(1), BlockId::new(3)));
        assert!(!cfg.is_reachable(BlockId::new(3), BlockId::new(1)));
    }

    #[test]
    fn cfg_is_reachable_diamond() {
        // Diamond: b1 -> {b2, b3} -> b4
        let b1 = make_block(
            1,
            Operation::CondBr {
                then_target: BlockId::new(2),
                else_target: BlockId::new(3),
            },
        );
        let b2 = make_block(
            2,
            Operation::Br {
                target: BlockId::new(4),
            },
        );
        let b3 = make_block(
            3,
            Operation::Br {
                target: BlockId::new(4),
            },
        );
        let b4 = make_block(4, Operation::Ret);

        let func = make_function(1, vec![b1, b2, b3, b4]);
        let cfg = Cfg::build(&func);

        // All forward paths work
        assert!(cfg.is_reachable(BlockId::new(1), BlockId::new(4)));
        assert!(cfg.is_reachable(BlockId::new(2), BlockId::new(4)));
        assert!(cfg.is_reachable(BlockId::new(3), BlockId::new(4)));

        // b2 and b3 are not reachable from each other (parallel branches)
        assert!(!cfg.is_reachable(BlockId::new(2), BlockId::new(3)));
        assert!(!cfg.is_reachable(BlockId::new(3), BlockId::new(2)));

        // Backward not reachable
        assert!(!cfg.is_reachable(BlockId::new(4), BlockId::new(1)));
    }
}
