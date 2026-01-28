//! Edge types for the value flow graph.

use serde::{Deserialize, Serialize};

/// Kind of edge in the value flow graph.
///
/// Each edge kind represents a different mechanism of data propagation
/// through the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// SSA def → use (including phi incoming values).
    DefUse,
    /// Binary/unary op operand → result.
    Transform,
    /// Actual argument → formal parameter.
    CallArg,
    /// Callee return value → caller result.
    Return,
    /// Value → memory location (store).
    Store,
    /// Memory location → value (load).
    Load,
}

impl EdgeKind {
    /// Get a human-readable name for the edge kind.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::DefUse => "def_use",
            Self::Transform => "transform",
            Self::CallArg => "call_arg",
            Self::Return => "return",
            Self::Store => "store",
            Self::Load => "load",
        }
    }

    /// Check if this is a memory-related edge.
    #[must_use]
    pub const fn is_memory(&self) -> bool {
        matches!(self, Self::Store | Self::Load)
    }

    /// Check if this is a call-related edge.
    #[must_use]
    pub const fn is_call(&self) -> bool {
        matches!(self, Self::CallArg | Self::Return)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_kind_name() {
        assert_eq!(EdgeKind::DefUse.name(), "def_use");
        assert_eq!(EdgeKind::Transform.name(), "transform");
        assert_eq!(EdgeKind::CallArg.name(), "call_arg");
        assert_eq!(EdgeKind::Return.name(), "return");
        assert_eq!(EdgeKind::Store.name(), "store");
        assert_eq!(EdgeKind::Load.name(), "load");
    }

    #[test]
    fn edge_kind_is_memory() {
        assert!(!EdgeKind::DefUse.is_memory());
        assert!(!EdgeKind::Transform.is_memory());
        assert!(!EdgeKind::CallArg.is_memory());
        assert!(!EdgeKind::Return.is_memory());
        assert!(EdgeKind::Store.is_memory());
        assert!(EdgeKind::Load.is_memory());
    }

    #[test]
    fn edge_kind_is_call() {
        assert!(!EdgeKind::DefUse.is_call());
        assert!(!EdgeKind::Transform.is_call());
        assert!(EdgeKind::CallArg.is_call());
        assert!(EdgeKind::Return.is_call());
        assert!(!EdgeKind::Store.is_call());
        assert!(!EdgeKind::Load.is_call());
    }

    #[test]
    fn edge_kind_ordering() {
        // Test that edge kinds are orderable
        let kinds = vec![
            EdgeKind::DefUse,
            EdgeKind::Transform,
            EdgeKind::CallArg,
            EdgeKind::Return,
            EdgeKind::Store,
            EdgeKind::Load,
        ];

        // Each should be distinct
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn edge_kind_serialization_roundtrip() {
        let kinds = vec![
            EdgeKind::DefUse,
            EdgeKind::Transform,
            EdgeKind::CallArg,
            EdgeKind::Return,
            EdgeKind::Store,
            EdgeKind::Load,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: EdgeKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, parsed);
        }
    }

    #[test]
    fn edge_kind_serialization_format() {
        // Check that the serialization uses snake_case
        assert_eq!(
            serde_json::to_string(&EdgeKind::DefUse).unwrap(),
            "\"def_use\""
        );
        assert_eq!(
            serde_json::to_string(&EdgeKind::CallArg).unwrap(),
            "\"call_arg\""
        );
    }
}
