//! Newtype wrappers for SAF entity IDs.
//!
//! All IDs are `u128` values derived from BLAKE3 hashes, ensuring deterministic
//! identification across runs (FR-AIR-002). Newtypes provide type safety and
//! prevent accidental mixing of different ID kinds.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::id::{id_to_hex, make_id};

/// Common interface for all SAF entity ID types.
///
/// Provides a uniform way to access the raw `u128` value and hex formatting
/// across all ID newtypes (`FunctionId`, `BlockId`, `ValueId`, etc.).
pub trait EntityId:
    Copy + Eq + Ord + std::hash::Hash + std::fmt::Display + std::fmt::Debug
{
    /// Get the raw `u128` value.
    fn raw(self) -> u128;

    /// Format as hex string with `0x` prefix.
    fn to_hex(self) -> String {
        crate::id::id_to_hex(self.raw())
    }
}

/// Helper macro to define ID newtypes with common implementations.
macro_rules! define_id_type {
    ($(#[$meta:meta])* $name:ident, $domain:literal) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub u128);

        impl $name {
            /// Create a new ID from raw `u128` value.
            #[must_use]
            pub const fn new(id: u128) -> Self {
                Self(id)
            }

            /// Derive a deterministic ID from the given data bytes.
            #[must_use]
            pub fn derive(data: &[u8]) -> Self {
                Self(make_id($domain, data))
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

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), id_to_hex(self.0))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", id_to_hex(self.0))
            }
        }

        impl From<u128> for $name {
            fn from(id: u128) -> Self {
                Self(id)
            }
        }

        impl From<$name> for u128 {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&id_to_hex(self.0))
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = String::deserialize(deserializer)?;
                parse_hex_id(&s)
                    .map(Self)
                    .map_err(serde::de::Error::custom)
            }
        }

        impl $crate::ids::EntityId for $name {
            fn raw(self) -> u128 {
                self.0
            }
        }
    };
}

/// Parse a hex string (with or without `0x` prefix) into a `u128`.
fn parse_hex_id(s: &str) -> Result<u128, String> {
    let hex_str = s.strip_prefix("0x").unwrap_or(s);
    u128::from_str_radix(hex_str, 16).map_err(|e| format!("invalid hex ID '{s}': {e}"))
}

define_id_type!(
    /// Unique identifier for an AIR module.
    ModuleId,
    "module"
);

define_id_type!(
    /// Unique identifier for a function within an AIR module.
    FunctionId,
    "function"
);

define_id_type!(
    /// Unique identifier for a basic block within a function.
    BlockId,
    "block"
);

define_id_type!(
    /// Unique identifier for an instruction.
    InstId,
    "inst"
);

define_id_type!(
    /// Unique identifier for a value (instruction result, parameter, constant).
    ValueId,
    "value"
);

define_id_type!(
    /// Unique identifier for a memory object (allocation site).
    ObjId,
    "obj"
);

define_id_type!(
    /// Unique identifier for a source location.
    LocId,
    "loc"
);

define_id_type!(
    /// Unique identifier for a type in the type table.
    TypeId,
    "type"
);

define_id_type!(
    /// Unique identifier for a source file referenced by spans.
    FileId,
    "file"
);

define_id_type!(
    /// Unique identifier for a whole program (set of linked modules).
    ProgramId,
    "program"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_derive_is_deterministic() {
        let a = FunctionId::derive(b"main");
        let b = FunctionId::derive(b"main");
        assert_eq!(a, b);
    }

    #[test]
    fn different_domains_produce_different_ids() {
        // Same data, different ID types (which use different domains)
        let fn_id = FunctionId::derive(b"main");
        let block_id = BlockId::derive(b"main");
        assert_ne!(fn_id.raw(), block_id.raw());
    }

    #[test]
    fn id_display_format() {
        let id = FunctionId::new(0x1234_5678_9abc_def0_1234_5678_9abc_def0);
        let hex = id.to_string();
        assert!(hex.starts_with("0x"));
        assert_eq!(hex.len(), 34); // 0x + 32 hex chars
    }

    #[test]
    fn id_serialization_roundtrip() {
        let original = FunctionId::derive(b"test_function");
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: FunctionId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, parsed);
    }

    #[test]
    fn id_parse_with_and_without_prefix() {
        let id = FunctionId::new(0x123);

        // With prefix
        let json_with_prefix = "\"0x00000000000000000000000000000123\"";
        let parsed: FunctionId = serde_json::from_str(json_with_prefix).expect("parse with prefix");
        assert_eq!(parsed, id);

        // Without prefix
        let json_without_prefix = "\"00000000000000000000000000000123\"";
        let parsed: FunctionId =
            serde_json::from_str(json_without_prefix).expect("parse without prefix");
        assert_eq!(parsed, id);
    }

    #[test]
    fn id_ordering() {
        let a = FunctionId::new(1);
        let b = FunctionId::new(2);
        let c = FunctionId::new(1);
        assert!(a < b);
        assert_eq!(a, c);
    }

    #[test]
    fn type_id_derive_is_deterministic() {
        let a = TypeId::derive(b"integer:32");
        let b = TypeId::derive(b"integer:32");
        assert_eq!(a, b);
    }

    #[test]
    fn type_id_different_from_other_domains() {
        let type_id = TypeId::derive(b"test");
        let value_id = ValueId::derive(b"test");
        assert_ne!(type_id.raw(), value_id.raw());
    }

    #[test]
    fn entity_id_trait_works() {
        let func_id = FunctionId::derive(b"test");
        // Test via EntityId trait
        let raw: u128 = EntityId::raw(func_id);
        assert_eq!(raw, func_id.0);
        let hex: String = EntityId::to_hex(func_id);
        assert!(hex.starts_with("0x"));
        assert_eq!(hex.len(), 34);
    }

    #[test]
    fn entity_id_trait_is_generic() {
        // Verify that EntityId works as a generic bound
        fn check_id<T: EntityId>(id: T) -> u128 {
            id.raw()
        }
        let fid = FunctionId::new(42);
        let bid = BlockId::new(42);
        // Same raw value, different types
        assert_eq!(check_id(fid), check_id(bid));
    }
}
