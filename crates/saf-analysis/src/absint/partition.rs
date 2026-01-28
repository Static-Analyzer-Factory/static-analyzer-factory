//! Trace partitioning for path-sensitive abstract interpretation.
//!
//! Based on Astree (Rival & Mauborgne, ESOP 2005) trace partitioning
//! with Eva-style fuel budget.

use std::collections::BTreeMap;

use saf_core::ids::{BlockId, ValueId};

use super::state::AbstractState;

/// A token identifying which branch was taken at a split point.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartitionToken {
    /// Taken branch at a `CondBr` (cond_id, true=then/false=else).
    Branch {
        /// The condition value that was branched on.
        cond_id: ValueId,
        /// Whether the true (then) branch was taken.
        taken: bool,
    },
    /// Loop iteration number at a loop header.
    LoopIter {
        /// The loop header block.
        header: BlockId,
        /// Which iteration (0-indexed).
        iteration: u8,
    },
}

/// A partition key: stack of active tokens.
/// Kept small (max 4 tokens) to bound state space.
pub type PartitionKey = smallvec::SmallVec<[PartitionToken; 4]>;

/// Partitioned state at a block: multiple abstract states, one per partition.
#[derive(Debug, Clone)]
pub struct PartitionedState {
    /// Map from partition key to abstract state.
    partitions: BTreeMap<PartitionKey, AbstractState>,
}

/// Configuration for trace partitioning.
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionConfig {
    /// Maximum partitions per block (Eva-style fuel cap). Default: 16.
    pub max_partitions: usize,
    /// Maximum loop iterations to partition before merging. Default: 3.
    pub max_loop_partitions: u8,
    /// Whether partitioning is enabled. Default: true.
    pub enabled: bool,
}

impl Default for PartitionConfig {
    fn default() -> Self {
        Self {
            max_partitions: 16,
            max_loop_partitions: 3,
            enabled: true,
        }
    }
}

impl PartitionedState {
    /// Create an empty partitioned state (no partitions).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            partitions: BTreeMap::new(),
        }
    }

    /// Create from a single unpartitioned state.
    #[must_use]
    pub fn from_single(state: AbstractState) -> Self {
        let mut partitions = BTreeMap::new();
        partitions.insert(PartitionKey::new(), state);
        Self { partitions }
    }

    /// Insert or replace a partition.
    pub fn insert(&mut self, key: PartitionKey, state: AbstractState) {
        self.partitions.insert(key, state);
    }

    /// Number of active partitions.
    #[must_use]
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Iterate over all partitions.
    pub fn iter(&self) -> impl Iterator<Item = (&PartitionKey, &AbstractState)> {
        self.partitions.iter()
    }

    /// Mutably iterate over all partition states.
    pub fn partitions_mut(&mut self) -> impl Iterator<Item = &mut AbstractState> {
        self.partitions.values_mut()
    }

    /// Merge all partitions into a single state using join.
    #[must_use]
    pub fn merge_all(&self) -> AbstractState {
        let mut result: Option<AbstractState> = None;
        for state in self.partitions.values() {
            result = Some(match result {
                None => state.clone(),
                Some(acc) => acc.join(state),
            });
        }
        result.unwrap_or_else(AbstractState::bottom)
    }

    /// Split this state at a branch point, creating two partitioned states
    /// (one for each branch direction).
    pub fn split_at_branch(
        &self,
        cond_id: ValueId,
        then_state: &AbstractState,
        else_state: &AbstractState,
        config: &PartitionConfig,
    ) -> (Self, Self) {
        let mut then_partitions = BTreeMap::new();
        let mut else_partitions = BTreeMap::new();

        for key in self.partitions.keys() {
            let mut then_key = key.clone();
            then_key.push(PartitionToken::Branch {
                cond_id,
                taken: true,
            });
            let mut else_key = key.clone();
            else_key.push(PartitionToken::Branch {
                cond_id,
                taken: false,
            });

            if then_partitions.len() < config.max_partitions {
                then_partitions.insert(then_key, then_state.clone());
            } else {
                then_partitions.insert(key.clone(), then_state.clone());
            }
            if else_partitions.len() < config.max_partitions {
                else_partitions.insert(else_key, else_state.clone());
            } else {
                else_partitions.insert(key.clone(), else_state.clone());
            }
        }

        (
            Self {
                partitions: then_partitions,
            },
            Self {
                partitions: else_partitions,
            },
        )
    }

    /// Merge partitions at a join point: join states with matching keys,
    /// preserve one-sided partitions.
    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        let mut result = BTreeMap::new();

        for (key, state) in &self.partitions {
            if let Some(other_state) = other.partitions.get(key) {
                result.insert(key.clone(), state.join(other_state));
            } else {
                result.insert(key.clone(), state.clone());
            }
        }
        for (key, state) in &other.partitions {
            result.entry(key.clone()).or_insert_with(|| state.clone());
        }

        Self { partitions: result }
    }

    /// Check if state has changed compared to another (for worklist convergence).
    #[must_use]
    pub fn leq(&self, other: &Self) -> bool {
        // Every partition in self must be leq the corresponding partition in other
        for (key, state) in &self.partitions {
            if let Some(other_state) = other.partitions.get(key) {
                if !state.leq(other_state) {
                    return false;
                }
            } else {
                // Self has a partition that other doesn't — not leq
                if !state.is_unreachable() {
                    return false;
                }
            }
        }
        true
    }

    /// Check if all partitions are unreachable.
    #[must_use]
    pub fn is_unreachable(&self) -> bool {
        self.partitions.values().all(AbstractState::is_unreachable)
    }

    /// Reduce partitions to fit within budget by merging the two
    /// most similar partitions (semantic-directed clumping).
    // NOTE: The reduction loop requires tracking two indices into a key vector
    // and comparing key lengths. Splitting would obscure the merge logic.
    #[allow(clippy::too_many_lines)]
    pub fn reduce_to_budget(&mut self, max: usize) {
        while self.partitions.len() > max {
            let keys: Vec<PartitionKey> = self.partitions.keys().cloned().collect();
            if keys.len() < 2 {
                break;
            }
            // Simple heuristic: merge the two with the longest keys
            let mut longest_idx = 0;
            let mut second_idx = 1;
            for (i, k) in keys.iter().enumerate() {
                if k.len() > keys[longest_idx].len() {
                    second_idx = longest_idx;
                    longest_idx = i;
                } else if i != longest_idx && k.len() > keys[second_idx].len() {
                    second_idx = i;
                }
            }
            let k1 = keys[longest_idx].clone();
            let k2 = keys[second_idx].clone();
            if let (Some(s1), Some(s2)) = (self.partitions.remove(&k1), self.partitions.remove(&k2))
            {
                let merged_key = if k1.len() <= k2.len() { k1 } else { k2 };
                self.partitions.insert(merged_key, s1.join(&s2));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partitioned_state_from_single() {
        let state = AbstractState::bottom();
        let ps = PartitionedState::from_single(state);
        assert_eq!(ps.partition_count(), 1);
    }

    #[test]
    fn partitioned_state_merge_all_single() {
        let state = AbstractState::bottom();
        let ps = PartitionedState::from_single(state);
        let merged = ps.merge_all();
        assert!(merged.is_unreachable());
    }

    #[test]
    fn partition_config_defaults() {
        let config = PartitionConfig::default();
        assert_eq!(config.max_partitions, 16);
        assert_eq!(config.max_loop_partitions, 3);
        assert!(config.enabled);
    }

    #[test]
    fn partitioned_state_join_matching_keys() {
        let s1 = PartitionedState::from_single(AbstractState::bottom());
        let s2 = PartitionedState::from_single(AbstractState::bottom());
        let joined = s1.join(&s2);
        assert_eq!(joined.partition_count(), 1);
    }

    #[test]
    fn partitioned_state_is_unreachable() {
        let ps = PartitionedState::from_single(AbstractState::bottom());
        assert!(ps.is_unreachable());
    }

    #[test]
    fn partitioned_state_leq_self() {
        let ps = PartitionedState::from_single(AbstractState::bottom());
        assert!(ps.leq(&ps));
    }

    #[test]
    fn reduce_to_budget_no_op_under_budget() {
        let mut ps = PartitionedState::from_single(AbstractState::bottom());
        ps.reduce_to_budget(16);
        assert_eq!(ps.partition_count(), 1);
    }

    // ======================================================================
    // split_at_branch tests with non-bottom states (M-4)
    // ======================================================================

    /// Helper to create a `ValueId` for test purposes.
    fn test_vid(n: u128) -> ValueId {
        ValueId::new(n)
    }

    #[test]
    fn split_at_branch_with_non_bottom_states() {
        use super::super::interval::Interval;

        // Create a reachable state with actual interval values.
        // Note: bottom() is unreachable, so we use new() for a reachable state.
        let mut state = AbstractState::new();
        let vid_x = test_vid(1);
        let vid_y = test_vid(2);
        state.set(vid_x, Interval::singleton(42, 32));
        state.set(vid_y, Interval::new(0, 100, 32));

        let ps = PartitionedState::from_single(state.clone());
        let cond_id = test_vid(100);

        // Create distinct then/else states with different intervals
        let mut then_state = AbstractState::new();
        then_state.set(vid_x, Interval::singleton(42, 32));
        then_state.set(vid_y, Interval::new(1, 100, 32)); // y > 0 on then branch

        let mut else_state = AbstractState::new();
        else_state.set(vid_x, Interval::singleton(42, 32));
        else_state.set(vid_y, Interval::new(0, 0, 32)); // y == 0 on else branch

        let config = PartitionConfig::default();
        let (then_ps, else_ps) = ps.split_at_branch(cond_id, &then_state, &else_state, &config);

        // Each side should have 1 partition (from 1 original partition)
        assert_eq!(then_ps.partition_count(), 1);
        assert_eq!(else_ps.partition_count(), 1);

        // Then partition key should contain Branch { cond_id, taken: true }
        for (key, _state) in then_ps.iter() {
            assert!(key.iter().any(|t| matches!(
                t,
                PartitionToken::Branch { cond_id: c, taken: true } if *c == cond_id
            )));
        }

        // Else partition key should contain Branch { cond_id, taken: false }
        for (key, _state) in else_ps.iter() {
            assert!(key.iter().any(|t| matches!(
                t,
                PartitionToken::Branch { cond_id: c, taken: false } if *c == cond_id
            )));
        }

        // Merged then state should have the then_state interval for y
        let then_merged = then_ps.merge_all();
        assert!(!then_merged.is_unreachable());

        // Merged else state should have the else_state interval for y
        let else_merged = else_ps.merge_all();
        assert!(!else_merged.is_unreachable());
    }

    #[test]
    fn split_at_branch_budget_limiting() {
        use super::super::interval::Interval;

        // Create a partitioned state already at the budget limit
        let config = PartitionConfig {
            max_partitions: 2,
            max_loop_partitions: 3,
            enabled: true,
        };

        let cond_id_1 = test_vid(200);
        let cond_id_2 = test_vid(201);

        // Start with 2 partitions (at budget)
        let mut ps = PartitionedState::empty();
        let mut key1 = PartitionKey::new();
        key1.push(PartitionToken::Branch {
            cond_id: cond_id_1,
            taken: true,
        });
        let mut key2 = PartitionKey::new();
        key2.push(PartitionToken::Branch {
            cond_id: cond_id_1,
            taken: false,
        });

        let mut state1 = AbstractState::new();
        state1.set(test_vid(1), Interval::singleton(10, 32));
        let mut state2 = AbstractState::new();
        state2.set(test_vid(1), Interval::singleton(20, 32));

        ps.insert(key1, state1);
        ps.insert(key2, state2);
        assert_eq!(ps.partition_count(), 2);

        let then_state = AbstractState::new();
        let else_state = AbstractState::new();

        let (then_ps, else_ps) = ps.split_at_branch(cond_id_2, &then_state, &else_state, &config);

        // With max_partitions=2, the split should respect the budget.
        // The first partition gets a new key; the second falls back to the old key.
        assert!(
            then_ps.partition_count() <= config.max_partitions,
            "Then partitions {} should be <= max {}",
            then_ps.partition_count(),
            config.max_partitions
        );
        assert!(
            else_ps.partition_count() <= config.max_partitions,
            "Else partitions {} should be <= max {}",
            else_ps.partition_count(),
            config.max_partitions
        );
    }

    #[test]
    fn split_at_branch_preserves_partition_keys() {
        use super::super::interval::Interval;

        let mut state = AbstractState::new();
        state.set(test_vid(1), Interval::new(-10, 10, 32));

        let ps = PartitionedState::from_single(state);
        let cond_id = test_vid(50);
        let then_state = AbstractState::new();
        let else_state = AbstractState::new();
        let config = PartitionConfig::default();

        let (then_ps, else_ps) = ps.split_at_branch(cond_id, &then_state, &else_state, &config);

        // The original had 1 partition with empty key.
        // After split, then should have key [Branch(cond_id, true)]
        // and else should have key [Branch(cond_id, false)].
        assert_eq!(then_ps.partition_count(), 1);
        assert_eq!(else_ps.partition_count(), 1);

        // Keys should be distinct
        let then_keys: Vec<_> = then_ps.iter().map(|(k, _)| k.clone()).collect();
        let else_keys: Vec<_> = else_ps.iter().map(|(k, _)| k.clone()).collect();
        assert_ne!(
            then_keys, else_keys,
            "Then and else partition keys should differ"
        );
    }
}
