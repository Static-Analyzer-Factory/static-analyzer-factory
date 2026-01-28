//! Node types for the value flow graph.

use serde::{Deserialize, Serialize};

use saf_core::ids::{LocId, ValueId};

/// A node in the value flow graph.
///
/// Nodes represent either SSA values or memory locations. This hybrid
/// representation allows tracking data flow through both SSA and memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeId {
    /// An SSA value (instruction result, parameter, global address).
    Value {
        /// The value ID.
        id: ValueId,
    },
    /// A memory location (from PTA).
    Location {
        /// The location ID.
        id: LocId,
    },
    /// Unknown memory location (used in fast mode).
    UnknownMem,
}

impl NodeId {
    /// Create a value node.
    #[must_use]
    pub const fn value(id: ValueId) -> Self {
        Self::Value { id }
    }

    /// Create a location node.
    #[must_use]
    pub const fn location(id: LocId) -> Self {
        Self::Location { id }
    }

    /// Create the unknown memory node.
    #[must_use]
    pub const fn unknown_mem() -> Self {
        Self::UnknownMem
    }

    /// Check if this is a value node.
    #[must_use]
    pub const fn is_value(&self) -> bool {
        matches!(self, Self::Value { .. })
    }

    /// Check if this is a location node.
    #[must_use]
    pub const fn is_location(&self) -> bool {
        matches!(self, Self::Location { .. })
    }

    /// Check if this is the unknown memory node.
    #[must_use]
    pub const fn is_unknown_mem(&self) -> bool {
        matches!(self, Self::UnknownMem)
    }

    /// Get the value ID if this is a value node.
    #[must_use]
    pub const fn as_value(&self) -> Option<ValueId> {
        match self {
            Self::Value { id } => Some(*id),
            _ => None,
        }
    }

    /// Get the location ID if this is a location node.
    #[must_use]
    pub const fn as_location(&self) -> Option<LocId> {
        match self {
            Self::Location { id } => Some(*id),
            _ => None,
        }
    }
}

impl From<ValueId> for NodeId {
    fn from(id: ValueId) -> Self {
        Self::value(id)
    }
}

impl From<LocId> for NodeId {
    fn from(id: LocId) -> Self {
        Self::location(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_value_creation() {
        let id = ValueId::new(42);
        let node = NodeId::value(id);
        assert!(node.is_value());
        assert!(!node.is_location());
        assert!(!node.is_unknown_mem());
        assert_eq!(node.as_value(), Some(id));
        assert_eq!(node.as_location(), None);
    }

    #[test]
    fn node_location_creation() {
        let id = LocId::new(42);
        let node = NodeId::location(id);
        assert!(!node.is_value());
        assert!(node.is_location());
        assert!(!node.is_unknown_mem());
        assert_eq!(node.as_value(), None);
        assert_eq!(node.as_location(), Some(id));
    }

    #[test]
    fn node_unknown_mem() {
        let node = NodeId::unknown_mem();
        assert!(!node.is_value());
        assert!(!node.is_location());
        assert!(node.is_unknown_mem());
        assert_eq!(node.as_value(), None);
        assert_eq!(node.as_location(), None);
    }

    #[test]
    fn node_from_value_id() {
        let id = ValueId::new(1);
        let node: NodeId = id.into();
        assert_eq!(node, NodeId::value(id));
    }

    #[test]
    fn node_from_loc_id() {
        let id = LocId::new(1);
        let node: NodeId = id.into();
        assert_eq!(node, NodeId::location(id));
    }

    #[test]
    fn node_ordering() {
        // Test that nodes are orderable (for BTreeSet/BTreeMap)
        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        let l1 = NodeId::location(LocId::new(1));
        let um = NodeId::unknown_mem();

        // We just care that they can be compared, not the specific order
        assert!(v1 != v2);
        assert!(v1 != l1);
        assert!(v1 != um);
    }

    #[test]
    fn node_serialization_value() {
        let node = NodeId::value(ValueId::new(42));
        let json = serde_json::to_string(&node).unwrap();
        let parsed: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(node, parsed);
    }

    #[test]
    fn node_serialization_location() {
        let node = NodeId::location(LocId::new(42));
        let json = serde_json::to_string(&node).unwrap();
        let parsed: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(node, parsed);
    }

    #[test]
    fn node_serialization_unknown_mem() {
        let node = NodeId::unknown_mem();
        let json = serde_json::to_string(&node).unwrap();
        let parsed: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(node, parsed);
    }
}
