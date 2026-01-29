//! Lock set tracking for race detection.
//!
//! Computes the set of locks held at each program point using dataflow analysis.
//! This enables lock-sensitive race detection that can prove accesses protected
//! by common locks do not race.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirFunction, AirModule, Operation};
use saf_core::ids::{FunctionId, InstId, ValueId};

use crate::cfg::Cfg;
use crate::icfg::Icfg;

use super::MtaConfig;
use super::types::{AccessKind, MemoryAccess, MhpResult};

/// A set of locks held at a program point.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LockSet {
    /// Lock values (mutex pointers) currently held.
    locks: BTreeSet<ValueId>,
}

impl LockSet {
    /// Create an empty lock set.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a lock set from an iterator of lock values.
    pub fn collect_from<I: IntoIterator<Item = ValueId>>(iter: I) -> Self {
        Self {
            locks: iter.into_iter().collect(),
        }
    }

    /// Acquire a lock.
    pub fn acquire(&mut self, lock: ValueId) {
        self.locks.insert(lock);
    }

    /// Release a lock.
    pub fn release(&mut self, lock: ValueId) {
        self.locks.remove(&lock);
    }

    /// Check if a specific lock is held.
    #[must_use]
    pub fn holds(&self, lock: ValueId) -> bool {
        self.locks.contains(&lock)
    }

    /// Check if any lock is held.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.locks.is_empty()
    }

    /// Get the number of locks held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.locks.len()
    }

    /// Check if there is any common lock between two lock sets.
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        // Check if any lock appears in both sets
        self.locks.iter().any(|lock| other.locks.contains(lock))
    }

    /// Get the intersection of two lock sets.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            locks: self.locks.intersection(&other.locks).copied().collect(),
        }
    }

    /// Get the union of two lock sets.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            locks: self.locks.union(&other.locks).copied().collect(),
        }
    }

    /// Get all held locks.
    pub fn locks(&self) -> impl Iterator<Item = ValueId> + '_ {
        self.locks.iter().copied()
    }

    /// Convert to a `BTreeSet<ValueId>` for `MemoryAccess`.
    #[must_use]
    pub fn to_btreeset(&self) -> BTreeSet<ValueId> {
        self.locks.clone()
    }
}

/// Lock set analysis result.
#[derive(Clone, Debug, Default)]
pub struct LockSetResult {
    /// Lock set at each instruction (after instruction executes).
    locksets: BTreeMap<InstId, LockSet>,
}

impl LockSetResult {
    /// Create empty result.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the lock set at a specific instruction.
    #[must_use]
    pub fn lockset_at(&self, inst_id: InstId) -> LockSet {
        self.locksets
            .get(&inst_id)
            .cloned()
            .unwrap_or_else(LockSet::empty)
    }

    /// Set the lock set at a specific instruction.
    pub fn set_lockset(&mut self, inst_id: InstId, lockset: LockSet) {
        self.locksets.insert(inst_id, lockset);
    }

    /// Get all instructions with lock sets.
    pub fn instructions(&self) -> impl Iterator<Item = InstId> + '_ {
        self.locksets.keys().copied()
    }
}

/// Compute lock sets at each program point via forward dataflow analysis.
///
/// # Arguments
///
/// * `module` - The AIR module
/// * `cfg` - Control flow graph for the function
/// * `func` - The function to analyze
/// * `config` - MTA configuration with lock/unlock function names
///
/// # Returns
///
/// A mapping from instruction ID to the set of locks held after that instruction.
#[must_use]
pub fn compute_locksets(
    module: &AirModule,
    cfg: &Cfg,
    func: &AirFunction,
    config: &MtaConfig,
) -> LockSetResult {
    let mut result = LockSetResult::new();

    // Simple intraprocedural forward analysis
    // For each block in topological order, propagate lock sets
    let mut block_entry: BTreeMap<_, LockSet> = BTreeMap::new();

    // Initialize entry block with empty lock set
    let entry_block = cfg.entry;
    block_entry.insert(entry_block, LockSet::empty());

    // Worklist algorithm
    let mut worklist: Vec<_> = vec![entry_block];
    let mut visited = BTreeSet::new();

    while let Some(block_id) = worklist.pop() {
        if visited.contains(&block_id) {
            continue;
        }
        visited.insert(block_id);

        // Get entry lock set for this block
        let mut current = block_entry
            .get(&block_id)
            .cloned()
            .unwrap_or_else(LockSet::empty);

        // Find the block in the function
        if let Some(block) = func.blocks.iter().find(|b| b.id == block_id) {
            for inst in &block.instructions {
                // Check for lock/unlock calls
                process_instruction(inst, &mut current, module, config);

                // Record the lock set after this instruction
                result.set_lockset(inst.id, current.clone());
            }
        }

        // Propagate to successors
        if let Some(successors) = cfg.successors_of(block_id) {
            for succ_id in successors {
                // Join lock sets at successor entry (intersection for must-analysis)
                let entry = block_entry
                    .entry(*succ_id)
                    .or_insert_with(|| current.clone());
                let new_entry = entry.intersection(&current);

                if &new_entry != entry {
                    *entry = new_entry;
                    worklist.push(*succ_id);
                } else if !visited.contains(succ_id) {
                    worklist.push(*succ_id);
                }
            }
        }
    }

    result
}

/// Process a single instruction for lock set effects.
fn process_instruction(
    inst: &saf_core::air::Instruction,
    lockset: &mut LockSet,
    module: &AirModule,
    config: &MtaConfig,
) {
    // Handle direct calls
    if let Operation::CallDirect { callee } = &inst.op {
        // Get callee name
        if let Some(callee_func) = module.function(*callee) {
            let callee_name = &callee_func.name;

            // Check for lock acquisition
            if config.lock_funcs.iter().any(|f| callee_name.contains(f)) {
                // First operand is typically the mutex pointer
                if let Some(mutex_ptr) = inst.operands.first() {
                    lockset.acquire(*mutex_ptr);
                }
            }

            // Check for lock release
            if config.unlock_funcs.iter().any(|f| callee_name.contains(f)) {
                // First operand is typically the mutex pointer
                if let Some(mutex_ptr) = inst.operands.first() {
                    lockset.release(*mutex_ptr);
                }
            }
        }
    }
}

/// Result of race check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RaceCheckResult {
    /// No race possible.
    NoRace,
    /// Potential data race detected.
    PotentialRace {
        /// The first access.
        access1: MemoryAccess,
        /// The second access.
        access2: MemoryAccess,
        /// Reason why this is a potential race.
        reason: String,
    },
}

/// Check for data race with lock sensitivity.
///
/// Two memory accesses race if:
/// 1. They access the same memory location (may alias)
/// 2. At least one is a write
/// 3. They may happen in parallel (MHP)
/// 4. They are NOT protected by a common lock
///
/// # Arguments
///
/// * `access1` - First memory access
/// * `access2` - Second memory access
/// * `locksets` - Lock sets at each instruction
/// * `mhp` - May-happen-in-parallel result
/// * `may_alias` - Function to check if two locations may alias
///
/// # Returns
///
/// `RaceCheckResult::NoRace` if the accesses cannot race,
/// `RaceCheckResult::PotentialRace` if they may race.
#[must_use]
pub fn check_race_with_locks<F>(
    access1: &MemoryAccess,
    access2: &MemoryAccess,
    locksets: &LockSetResult,
    mhp: &MhpResult,
    may_alias: F,
) -> RaceCheckResult
where
    F: Fn(ValueId, ValueId) -> bool,
{
    // Same instruction cannot race with itself
    if access1.inst_id == access2.inst_id {
        return RaceCheckResult::NoRace;
    }

    // Both reads -> no race
    if access1.kind == AccessKind::Read && access2.kind == AccessKind::Read {
        return RaceCheckResult::NoRace;
    }

    // Check MHP: threads must be able to run concurrently at these program points
    let concurrent_at_1 = mhp.concurrent_at(access1.thread_id, access1.inst_id);
    if !concurrent_at_1.contains(&access2.thread_id) {
        // Check the other direction too
        let concurrent_at_2 = mhp.concurrent_at(access2.thread_id, access2.inst_id);
        if !concurrent_at_2.contains(&access1.thread_id) {
            return RaceCheckResult::NoRace;
        }
    }

    // Check alias: must access same memory
    if !may_alias(access1.location, access2.location) {
        return RaceCheckResult::NoRace;
    }

    // Check if protected by common lock
    let locks1 = locksets.lockset_at(access1.inst_id);
    let locks2 = locksets.lockset_at(access2.inst_id);

    if locks1.intersects(&locks2) {
        return RaceCheckResult::NoRace;
    }

    // Potential race!
    RaceCheckResult::PotentialRace {
        access1: access1.clone(),
        access2: access2.clone(),
        reason: format!(
            "Concurrent {} and {} to same location without common lock",
            access_kind_str(access1.kind),
            access_kind_str(access2.kind)
        ),
    }
}

fn access_kind_str(kind: AccessKind) -> &'static str {
    match kind {
        AccessKind::Read => "read",
        AccessKind::Write => "write",
    }
}

/// Compute lock sets for an entire module (all functions).
#[must_use]
pub fn compute_module_locksets(
    module: &AirModule,
    icfg: &Icfg,
    config: &MtaConfig,
) -> BTreeMap<FunctionId, LockSetResult> {
    let mut results = BTreeMap::new();

    for func in &module.functions {
        if let Some(cfg) = icfg.cfg(func.id) {
            let lockset_result = compute_locksets(module, cfg, func, config);
            results.insert(func.id, lockset_result);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::super::types::ThreadId;
    use super::*;

    #[test]
    fn test_lockset_empty() {
        let lockset = LockSet::empty();
        assert!(lockset.is_empty());
        assert_eq!(lockset.len(), 0);
    }

    #[test]
    fn test_lockset_acquire_release() {
        let mut lockset = LockSet::empty();
        let lock1 = ValueId::new(1);
        let lock2 = ValueId::new(2);

        lockset.acquire(lock1);
        assert!(lockset.holds(lock1));
        assert!(!lockset.holds(lock2));
        assert_eq!(lockset.len(), 1);

        lockset.acquire(lock2);
        assert!(lockset.holds(lock1));
        assert!(lockset.holds(lock2));
        assert_eq!(lockset.len(), 2);

        lockset.release(lock1);
        assert!(!lockset.holds(lock1));
        assert!(lockset.holds(lock2));
        assert_eq!(lockset.len(), 1);
    }

    #[test]
    fn test_lockset_intersects() {
        let lock1 = ValueId::new(1);
        let lock2 = ValueId::new(2);
        let lock3 = ValueId::new(3);

        let mut set1 = LockSet::empty();
        set1.acquire(lock1);
        set1.acquire(lock2);

        let mut set2 = LockSet::empty();
        set2.acquire(lock2);
        set2.acquire(lock3);

        // Sets share lock2
        assert!(set1.intersects(&set2));

        let mut set3 = LockSet::empty();
        set3.acquire(lock3);

        // set1 and set3 share no locks
        assert!(!set1.intersects(&set3));
    }

    #[test]
    fn test_lockset_intersection() {
        let lock1 = ValueId::new(1);
        let lock2 = ValueId::new(2);
        let lock3 = ValueId::new(3);

        let mut set1 = LockSet::empty();
        set1.acquire(lock1);
        set1.acquire(lock2);

        let mut set2 = LockSet::empty();
        set2.acquire(lock2);
        set2.acquire(lock3);

        let intersection = set1.intersection(&set2);
        assert!(intersection.holds(lock2));
        assert!(!intersection.holds(lock1));
        assert!(!intersection.holds(lock3));
        assert_eq!(intersection.len(), 1);
    }

    #[test]
    fn test_lockset_union() {
        let lock1 = ValueId::new(1);
        let lock2 = ValueId::new(2);

        let mut set1 = LockSet::empty();
        set1.acquire(lock1);

        let mut set2 = LockSet::empty();
        set2.acquire(lock2);

        let union = set1.union(&set2);
        assert!(union.holds(lock1));
        assert!(union.holds(lock2));
        assert_eq!(union.len(), 2);
    }

    #[test]
    fn test_race_check_both_reads() {
        let access1 = MemoryAccess::new(
            InstId::new(1),
            ValueId::new(100),
            ThreadId::new(0),
            AccessKind::Read,
        );
        let access2 = MemoryAccess::new(
            InstId::new(2),
            ValueId::new(100),
            ThreadId::new(1),
            AccessKind::Read,
        );

        let locksets = LockSetResult::new();
        let mhp = MhpResult::new();

        let result = check_race_with_locks(&access1, &access2, &locksets, &mhp, |_, _| true);

        // Both reads -> no race
        assert_eq!(result, RaceCheckResult::NoRace);
    }

    #[test]
    fn test_race_check_protected_by_lock() {
        let lock = ValueId::new(50);

        // Create accesses with locks
        let access1 = MemoryAccess::with_locks(
            InstId::new(1),
            ValueId::new(100),
            ThreadId::new(0),
            AccessKind::Write,
            [lock].into_iter().collect(),
        );
        let access2 = MemoryAccess::with_locks(
            InstId::new(2),
            ValueId::new(100),
            ThreadId::new(1),
            AccessKind::Write,
            [lock].into_iter().collect(),
        );

        // Set up locksets
        let mut locksets = LockSetResult::new();
        let mut lockset = LockSet::empty();
        lockset.acquire(lock);
        locksets.set_lockset(InstId::new(1), lockset.clone());
        locksets.set_lockset(InstId::new(2), lockset);

        // Set up MHP (threads are concurrent)
        let mut mhp = MhpResult::new();
        let mut concurrent = BTreeSet::new();
        concurrent.insert(ThreadId::new(1));
        mhp.set_concurrent_at(ThreadId::new(0), InstId::new(1), concurrent.clone());
        concurrent.clear();
        concurrent.insert(ThreadId::new(0));
        mhp.set_concurrent_at(ThreadId::new(1), InstId::new(2), concurrent);

        let result = check_race_with_locks(&access1, &access2, &locksets, &mhp, |_, _| true);

        // Protected by common lock -> no race
        assert_eq!(result, RaceCheckResult::NoRace);
    }

    #[test]
    fn test_race_check_no_common_lock() {
        let lock1 = ValueId::new(50);
        let lock2 = ValueId::new(51);

        // Set up locksets with different locks
        let mut locksets = LockSetResult::new();
        let mut lockset1 = LockSet::empty();
        lockset1.acquire(lock1);
        locksets.set_lockset(InstId::new(1), lockset1);

        let mut lockset2 = LockSet::empty();
        lockset2.acquire(lock2);
        locksets.set_lockset(InstId::new(2), lockset2);

        // Set up MHP (threads are concurrent)
        let mut mhp = MhpResult::new();
        let mut concurrent = BTreeSet::new();
        concurrent.insert(ThreadId::new(1));
        mhp.set_concurrent_at(ThreadId::new(0), InstId::new(1), concurrent.clone());
        concurrent.clear();
        concurrent.insert(ThreadId::new(0));
        mhp.set_concurrent_at(ThreadId::new(1), InstId::new(2), concurrent);

        let access1 = MemoryAccess::new(
            InstId::new(1),
            ValueId::new(100),
            ThreadId::new(0),
            AccessKind::Write,
        );
        let access2 = MemoryAccess::new(
            InstId::new(2),
            ValueId::new(100),
            ThreadId::new(1),
            AccessKind::Read,
        );

        let result = check_race_with_locks(&access1, &access2, &locksets, &mhp, |_, _| true);

        // Different locks -> potential race
        match result {
            RaceCheckResult::PotentialRace { .. } => {}
            RaceCheckResult::NoRace => panic!("Expected potential race"),
        }
    }
}
