//! Memory access types for Memory SSA.
//!
//! Each memory-affecting instruction (store, load, call) gets a `MemoryAccess`
//! that places it in the memory def-use chain. Phi nodes merge memory versions
//! at control-flow join points.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use saf_core::id::{id_to_hex, make_id};
use saf_core::ids::{BlockId, FunctionId, InstId};

// ---------------------------------------------------------------------------
// MemAccessId
// ---------------------------------------------------------------------------

/// Unique identifier for a memory access in the Memory SSA form.
///
/// BLAKE3-derived from domain `"memaccess"` for determinism (NFR-DET-001).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemAccessId(pub u128);

impl MemAccessId {
    /// Create from raw `u128`.
    #[must_use]
    pub const fn new(id: u128) -> Self {
        Self(id)
    }

    /// Derive a deterministic ID from the given data bytes.
    #[must_use]
    pub fn derive(data: &[u8]) -> Self {
        Self(make_id("memaccess", data))
    }

    /// Get the raw `u128` value.
    #[must_use]
    pub const fn raw(self) -> u128 {
        self.0
    }

    /// Format as hex string with `0x` prefix.
    #[must_use]
    pub fn to_hex(self) -> String {
        id_to_hex(self.0)
    }
}

impl std::fmt::Debug for MemAccessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemAccessId({})", id_to_hex(self.0))
    }
}

impl std::fmt::Display for MemAccessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", id_to_hex(self.0))
    }
}

impl Serialize for MemAccessId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&id_to_hex(self.0))
    }
}

impl<'de> Deserialize<'de> for MemAccessId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let hex_str = s.strip_prefix("0x").unwrap_or(&s);
        u128::from_str_radix(hex_str, 16)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// MemoryAccess
// ---------------------------------------------------------------------------

/// A memory access in the Memory SSA form.
///
/// Every memory-affecting instruction gets exactly one `MemoryAccess`.
/// The `defining` field links each access to its reaching definition in the
/// skeleton chain (dominance order). The clobber walker later refines these
/// links on demand for specific memory locations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryAccess {
    /// Sentinel: memory state at function entry (before any instruction).
    LiveOnEntry {
        /// Unique access ID.
        id: MemAccessId,
        /// The function this sentinel belongs to.
        function: FunctionId,
    },

    /// A store or call that may modify memory.
    Def {
        /// Unique access ID.
        id: MemAccessId,
        /// The instruction that performs the store/call.
        inst: InstId,
        /// The block containing this instruction.
        block: BlockId,
        /// Previous def in the skeleton chain (dominance order).
        defining: MemAccessId,
    },

    /// A load or read-only memory access.
    Use {
        /// Unique access ID.
        id: MemAccessId,
        /// The instruction that performs the load.
        inst: InstId,
        /// The block containing this instruction.
        block: BlockId,
        /// Reaching def (skeleton; refined by clobber walker on demand).
        defining: MemAccessId,
    },

    /// Join point merging memory versions from predecessor blocks.
    Phi {
        /// Unique access ID.
        id: MemAccessId,
        /// The block where this phi is placed.
        block: BlockId,
        /// Predecessor block -> reaching def from that path.
        operands: BTreeMap<BlockId, MemAccessId>,
    },
}

impl MemoryAccess {
    /// Get the unique ID of this access.
    #[must_use]
    pub fn id(&self) -> MemAccessId {
        match self {
            Self::LiveOnEntry { id, .. }
            | Self::Def { id, .. }
            | Self::Use { id, .. }
            | Self::Phi { id, .. } => *id,
        }
    }

    /// Get the block this access belongs to, if any.
    ///
    /// `LiveOnEntry` has no block (it's a function-level sentinel).
    #[must_use]
    pub fn block(&self) -> Option<BlockId> {
        match self {
            Self::LiveOnEntry { .. } => None,
            Self::Def { block, .. } | Self::Use { block, .. } | Self::Phi { block, .. } => {
                Some(*block)
            }
        }
    }

    /// Get the instruction associated with this access, if any.
    ///
    /// Phi nodes and `LiveOnEntry` have no instruction.
    #[must_use]
    pub fn inst(&self) -> Option<InstId> {
        match self {
            Self::Def { inst, .. } | Self::Use { inst, .. } => Some(*inst),
            Self::LiveOnEntry { .. } | Self::Phi { .. } => None,
        }
    }

    /// Get the defining (reaching def) access ID, if applicable.
    #[must_use]
    pub fn defining(&self) -> Option<MemAccessId> {
        match self {
            Self::Def { defining, .. } | Self::Use { defining, .. } => Some(*defining),
            Self::LiveOnEntry { .. } | Self::Phi { .. } => None,
        }
    }

    /// Check if this is a Def.
    #[must_use]
    pub fn is_def(&self) -> bool {
        matches!(self, Self::Def { .. })
    }

    /// Check if this is a Use.
    #[must_use]
    pub fn is_use(&self) -> bool {
        matches!(self, Self::Use { .. })
    }

    /// Check if this is a Phi.
    #[must_use]
    pub fn is_phi(&self) -> bool {
        matches!(self, Self::Phi { .. })
    }

    /// Check if this is a LiveOnEntry.
    #[must_use]
    pub fn is_live_on_entry(&self) -> bool {
        matches!(self, Self::LiveOnEntry { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_access_id_derive_deterministic() {
        let a = MemAccessId::derive(b"test_access");
        let b = MemAccessId::derive(b"test_access");
        assert_eq!(a, b);
    }

    #[test]
    fn mem_access_id_different_data_different_id() {
        let a = MemAccessId::derive(b"access_1");
        let b = MemAccessId::derive(b"access_2");
        assert_ne!(a, b);
    }

    #[test]
    fn mem_access_id_hex_format() {
        let id = MemAccessId::new(0x1234);
        let hex = id.to_hex();
        assert!(hex.starts_with("0x"));
        assert_eq!(hex.len(), 34);
    }

    #[test]
    fn mem_access_id_serialization_roundtrip() {
        let original = MemAccessId::derive(b"roundtrip");
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: MemAccessId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, parsed);
    }

    #[test]
    fn live_on_entry_has_no_block_or_inst() {
        let acc = MemoryAccess::LiveOnEntry {
            id: MemAccessId::new(1),
            function: FunctionId::new(100),
        };
        assert!(acc.is_live_on_entry());
        assert_eq!(acc.block(), None);
        assert_eq!(acc.inst(), None);
        assert_eq!(acc.defining(), None);
    }

    #[test]
    fn def_access_fields() {
        let acc = MemoryAccess::Def {
            id: MemAccessId::new(2),
            inst: InstId::new(200),
            block: BlockId::new(300),
            defining: MemAccessId::new(1),
        };
        assert!(acc.is_def());
        assert_eq!(acc.id(), MemAccessId::new(2));
        assert_eq!(acc.block(), Some(BlockId::new(300)));
        assert_eq!(acc.inst(), Some(InstId::new(200)));
        assert_eq!(acc.defining(), Some(MemAccessId::new(1)));
    }

    #[test]
    fn use_access_fields() {
        let acc = MemoryAccess::Use {
            id: MemAccessId::new(3),
            inst: InstId::new(201),
            block: BlockId::new(300),
            defining: MemAccessId::new(2),
        };
        assert!(acc.is_use());
        assert_eq!(acc.id(), MemAccessId::new(3));
        assert_eq!(acc.inst(), Some(InstId::new(201)));
        assert_eq!(acc.defining(), Some(MemAccessId::new(2)));
    }

    #[test]
    fn phi_access_fields() {
        let mut operands = BTreeMap::new();
        operands.insert(BlockId::new(10), MemAccessId::new(1));
        operands.insert(BlockId::new(20), MemAccessId::new(2));

        let acc = MemoryAccess::Phi {
            id: MemAccessId::new(4),
            block: BlockId::new(30),
            operands,
        };
        assert!(acc.is_phi());
        assert_eq!(acc.block(), Some(BlockId::new(30)));
        assert_eq!(acc.inst(), None);
        assert_eq!(acc.defining(), None);
    }
}
