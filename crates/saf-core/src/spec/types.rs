//! Type definitions for function specifications.
//!
//! This module defines the types used to describe external/library function behavior
//! for use by various analyses (PTA, nullness, taint, etc.).

use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A function specification describing external/library function behavior.
///
/// All fields are optional except `name`. Analyses use safe defaults when fields
/// are missing. See the design doc for default behavior per field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionSpec {
    /// Function name or pattern. Use `glob:` or `regex:` prefix for pattern matching.
    pub name: String,

    /// High-level role of this function (e.g., allocator, source, sink).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,

    /// Function has no side effects; result depends only on inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pure: Option<bool>,

    /// Function never returns (e.g., exit, abort, longjmp).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noreturn: Option<bool>,

    /// Suppress an inherited spec from a lower-priority source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    /// Per-parameter specifications.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<ParamSpec>,

    /// Return value specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnSpec>,

    /// Taint propagation rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taint: Option<TaintSpec>,
}

impl FunctionSpec {
    /// Create a new minimal spec with just a name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            role: None,
            pure: None,
            noreturn: None,
            disabled: None,
            params: Vec::new(),
            returns: None,
            taint: None,
        }
    }

    /// Check if this spec is disabled.
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.disabled.unwrap_or(false)
    }

    /// Check if function is pure (no side effects).
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.pure.unwrap_or(false)
    }

    /// Check if function never returns.
    #[must_use]
    pub fn is_noreturn(&self) -> bool {
        self.noreturn.unwrap_or(false)
    }

    /// Get the spec for a specific parameter by index.
    #[must_use]
    pub fn param(&self, index: u32) -> Option<&ParamSpec> {
        self.params.iter().find(|p| p.index == index)
    }

    /// Merge another spec into this one (other overrides self for present fields).
    pub fn merge(&mut self, other: &FunctionSpec) {
        if other.role.is_some() {
            self.role.clone_from(&other.role);
        }
        if other.pure.is_some() {
            self.pure = other.pure;
        }
        if other.noreturn.is_some() {
            self.noreturn = other.noreturn;
        }
        if other.disabled.is_some() {
            self.disabled = other.disabled;
        }
        // Merge params: other's params override by index
        for other_param in &other.params {
            if let Some(existing) = self
                .params
                .iter_mut()
                .find(|p| p.index == other_param.index)
            {
                existing.merge(other_param);
            } else {
                self.params.push(other_param.clone());
            }
        }
        if other.returns.is_some() {
            match (&mut self.returns, &other.returns) {
                (Some(r), Some(o)) => r.merge(o),
                (None, Some(o)) => self.returns = Some(o.clone()),
                _ => {}
            }
        }
        if other.taint.is_some() {
            match (&mut self.taint, &other.taint) {
                (Some(t), Some(o)) => t.merge(o),
                (None, Some(o)) => self.taint = Some(o.clone()),
                _ => {}
            }
        }
    }
}

/// High-level role of a function for analysis purposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Memory allocator (e.g., malloc, new).
    Allocator,
    /// Memory reallocator (e.g., realloc).
    Reallocator,
    /// Memory deallocator (e.g., free, delete).
    Deallocator,
    /// Taint source (e.g., getenv, recv).
    Source,
    /// Taint sink (e.g., system, exec).
    Sink,
    /// Taint sanitizer (e.g., escape functions).
    Sanitizer,
    /// String operation (e.g., strlen, strcpy).
    StringOperation,
    /// I/O operation (e.g., read, write).
    Io,
    /// Custom role for domain-specific analyses.
    Custom(String),
}

/// Nullness requirements and guarantees for pointers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nullness {
    /// Pointer is guaranteed non-null.
    NotNull,
    /// Pointer may be null.
    MaybeNull,
    /// Pointer must be non-null (precondition for caller).
    RequiredNonnull,
    /// Pointer is explicitly allowed to be null.
    Nullable,
}

/// Describes what a returned pointer points to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pointer {
    /// Returns freshly allocated heap memory.
    FreshHeap,
    /// Returns freshly allocated stack memory.
    FreshStack,
    /// Returns unknown pointer.
    Unknown,
    /// Returns pointer to a static singleton (all calls return the same location).
    ///
    /// Used for functions like `__errno_location`, `getenv`, `gmtime`, `localtime`
    /// that return pointers to internal static buffers. The abstract location is
    /// keyed by the callee function name, so multiple call sites to the same
    /// function share the same points-to target.
    StaticSingleton,
}

/// Per-parameter specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamSpec {
    /// Parameter position (0-indexed).
    pub index: u32,

    /// Optional human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Parameter's pointee is modified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modifies: Option<bool>,

    /// Parameter's pointee is read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reads: Option<bool>,

    /// Nullness requirement/guarantee.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nullness: Option<Nullness>,

    /// Pointer may escape function scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escapes: Option<bool>,

    /// This is a function pointer that will be called.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback: Option<bool>,

    /// Semantic meaning (e.g., `allocation_size`, `byte_count`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic: Option<String>,

    /// Size bound relation (e.g., "param.2" for buffer size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_from: Option<String>,

    /// Parameter becomes tainted after call (for sources).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tainted: Option<bool>,
}

impl ParamSpec {
    /// Create a new param spec for the given index.
    #[must_use]
    pub fn new(index: u32) -> Self {
        Self {
            index,
            name: None,
            modifies: None,
            reads: None,
            nullness: None,
            escapes: None,
            callback: None,
            semantic: None,
            size_from: None,
            tainted: None,
        }
    }

    /// Check if this parameter modifies its pointee.
    #[must_use]
    pub fn does_modify(&self) -> bool {
        self.modifies.unwrap_or(false)
    }

    /// Check if this parameter reads its pointee.
    #[must_use]
    pub fn does_read(&self) -> bool {
        self.reads.unwrap_or(false)
    }

    /// Check if this parameter may escape.
    #[must_use]
    pub fn may_escape(&self) -> bool {
        // Default to true (conservative) when unspecified
        self.escapes.unwrap_or(true)
    }

    /// Check if this parameter is a callback.
    #[must_use]
    pub fn is_callback(&self) -> bool {
        self.callback.unwrap_or(false)
    }

    /// Merge another param spec into this one.
    fn merge(&mut self, other: &ParamSpec) {
        if other.name.is_some() {
            self.name.clone_from(&other.name);
        }
        if other.modifies.is_some() {
            self.modifies = other.modifies;
        }
        if other.reads.is_some() {
            self.reads = other.reads;
        }
        if other.nullness.is_some() {
            self.nullness = other.nullness;
        }
        if other.escapes.is_some() {
            self.escapes = other.escapes;
        }
        if other.callback.is_some() {
            self.callback = other.callback;
        }
        if other.semantic.is_some() {
            self.semantic.clone_from(&other.semantic);
        }
        if other.size_from.is_some() {
            self.size_from.clone_from(&other.size_from);
        }
        if other.tainted.is_some() {
            self.tainted = other.tainted;
        }
    }
}

/// Return value specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnSpec {
    /// Nullness of return value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nullness: Option<Nullness>,

    /// What the returned pointer points to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<Pointer>,

    /// Return aliases a parameter (e.g., "param.0").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aliases: Option<String>,

    /// Numeric interval [min, max].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<(i64, i64)>,

    /// Return value is tainted (for sources).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tainted: Option<bool>,
}

impl ReturnSpec {
    /// Create a new empty return spec.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nullness: None,
            pointer: None,
            aliases: None,
            interval: None,
            tainted: None,
        }
    }

    /// Check if return value is tainted.
    #[must_use]
    pub fn is_tainted(&self) -> bool {
        self.tainted.unwrap_or(false)
    }

    /// Parse alias reference to get parameter index.
    /// Returns `Some(index)` for "param.N" format.
    #[must_use]
    pub fn alias_param_index(&self) -> Option<u32> {
        self.aliases.as_ref().and_then(|s| {
            s.strip_prefix("param.")
                .and_then(|idx| idx.parse::<u32>().ok())
        })
    }

    /// Merge another return spec into this one.
    fn merge(&mut self, other: &ReturnSpec) {
        if other.nullness.is_some() {
            self.nullness = other.nullness;
        }
        if other.pointer.is_some() {
            self.pointer.clone_from(&other.pointer);
        }
        if other.aliases.is_some() {
            self.aliases.clone_from(&other.aliases);
        }
        if other.interval.is_some() {
            self.interval = other.interval;
        }
        if other.tainted.is_some() {
            self.tainted = other.tainted;
        }
    }
}

impl Default for ReturnSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// Taint propagation specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaintSpec {
    /// List of propagation rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub propagates: Vec<TaintPropagation>,
}

impl TaintSpec {
    /// Create a new empty taint spec.
    #[must_use]
    pub fn new() -> Self {
        Self {
            propagates: Vec::new(),
        }
    }

    /// Merge another taint spec into this one.
    fn merge(&mut self, other: &TaintSpec) {
        // For taint propagation, we append rules rather than replace
        self.propagates.extend(other.propagates.clone());
    }
}

impl Default for TaintSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// A single taint propagation rule.
///
/// The `from` field specifies where taint originates (a parameter or return value),
/// and `to` specifies where it flows. Both use the [`TaintLocation`] enum, which
/// serializes as `"param.N"` or `"return"` for YAML/JSON compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaintPropagation {
    /// Source of taint (e.g., `TaintLocation::Param(0)`, `TaintLocation::Return`).
    pub from: TaintLocation,

    /// Destinations where taint flows.
    pub to: Vec<TaintLocation>,
}

/// A location in taint propagation rules.
///
/// Serializes as `"return"` for [`TaintLocation::Return`], `"param.N"` for
/// [`TaintLocation::Param`], and the original string for [`TaintLocation::Unknown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaintLocation {
    /// The return value.
    Return,
    /// A parameter by index.
    Param(u32),
    /// Unknown location (forward compatibility for unrecognized strings).
    Unknown,
}

impl Serialize for TaintLocation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TaintLocation::Return => serializer.serialize_str("return"),
            TaintLocation::Param(idx) => {
                let s = format!("param.{idx}");
                serializer.serialize_str(&s)
            }
            TaintLocation::Unknown => serializer.serialize_str("unknown"),
        }
    }
}

impl<'de> Deserialize<'de> for TaintLocation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TaintLocationVisitor;

        impl Visitor<'_> for TaintLocationVisitor {
            type Value = TaintLocation;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .write_str("\"return\", \"param.N\" (where N is an integer), or other string")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<TaintLocation, E> {
                if value == "return" {
                    Ok(TaintLocation::Return)
                } else if let Some(idx_str) = value.strip_prefix("param.") {
                    match idx_str.parse::<u32>() {
                        Ok(idx) => Ok(TaintLocation::Param(idx)),
                        Err(_) => Ok(TaintLocation::Unknown),
                    }
                } else {
                    Ok(TaintLocation::Unknown)
                }
            }
        }

        deserializer.deserialize_str(TaintLocationVisitor)
    }
}

impl fmt::Display for TaintLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaintLocation::Return => write!(f, "return"),
            TaintLocation::Param(idx) => write!(f, "param.{idx}"),
            TaintLocation::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_spec_new() {
        let spec = FunctionSpec::new("malloc");
        assert_eq!(spec.name, "malloc");
        assert!(spec.role.is_none());
        assert!(!spec.is_pure());
        assert!(!spec.is_noreturn());
    }

    #[test]
    fn test_role_serde() {
        let role = Role::Allocator;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"allocator\"");

        let parsed: Role = serde_json::from_str("\"source\"").unwrap();
        assert_eq!(parsed, Role::Source);
    }

    #[test]
    fn test_custom_role_serde() {
        let role = Role::Custom("validator".to_string());
        let json = serde_json::to_string(&role).unwrap();
        let parsed: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, role);
    }

    #[test]
    fn test_nullness_serde() {
        let nullness = Nullness::RequiredNonnull;
        let json = serde_json::to_string(&nullness).unwrap();
        assert_eq!(json, "\"required_nonnull\"");
    }

    #[test]
    fn test_return_spec_alias_parsing() {
        let ret = ReturnSpec {
            aliases: Some("param.0".to_string()),
            ..ReturnSpec::default()
        };
        assert_eq!(ret.alias_param_index(), Some(0));

        let ret2 = ReturnSpec {
            aliases: Some("param.3".to_string()),
            ..ReturnSpec::default()
        };
        assert_eq!(ret2.alias_param_index(), Some(3));

        let ret3 = ReturnSpec::default();
        assert_eq!(ret3.alias_param_index(), None);
    }

    #[test]
    fn test_taint_propagation_fields() {
        let prop = TaintPropagation {
            from: TaintLocation::Param(1),
            to: vec![TaintLocation::Param(0), TaintLocation::Return],
        };
        assert_eq!(prop.from, TaintLocation::Param(1));
        assert!(!matches!(prop.from, TaintLocation::Return));
        assert_eq!(prop.to.len(), 2);
        assert_eq!(prop.to[0], TaintLocation::Param(0));
        assert_eq!(prop.to[1], TaintLocation::Return);
    }

    #[test]
    fn test_taint_location_serde_roundtrip() {
        // Test serialization
        let loc_return = TaintLocation::Return;
        let json = serde_json::to_string(&loc_return).expect("serialize Return");
        assert_eq!(json, "\"return\"");

        let loc_param = TaintLocation::Param(3);
        let json = serde_json::to_string(&loc_param).expect("serialize Param(3)");
        assert_eq!(json, "\"param.3\"");

        let loc_unknown = TaintLocation::Unknown;
        let json = serde_json::to_string(&loc_unknown).expect("serialize Unknown");
        assert_eq!(json, "\"unknown\"");

        // Test deserialization
        let parsed: TaintLocation = serde_json::from_str("\"return\"").expect("deserialize return");
        assert_eq!(parsed, TaintLocation::Return);

        let parsed: TaintLocation =
            serde_json::from_str("\"param.5\"").expect("deserialize param.5");
        assert_eq!(parsed, TaintLocation::Param(5));

        let parsed: TaintLocation =
            serde_json::from_str("\"something_else\"").expect("deserialize unknown");
        assert_eq!(parsed, TaintLocation::Unknown);
    }

    #[test]
    fn test_taint_propagation_yaml_compat() {
        // Verify backward compatibility: YAML `from: param.1` and `to: [param.0, return]`
        let yaml = "from: param.1\nto:\n  - param.0\n  - return\n";
        let prop: TaintPropagation =
            serde_yaml::from_str(yaml).expect("deserialize taint propagation from YAML");
        assert_eq!(prop.from, TaintLocation::Param(1));
        assert_eq!(
            prop.to,
            vec![TaintLocation::Param(0), TaintLocation::Return]
        );
    }

    #[test]
    fn test_taint_location_display() {
        assert_eq!(TaintLocation::Return.to_string(), "return");
        assert_eq!(TaintLocation::Param(2).to_string(), "param.2");
        assert_eq!(TaintLocation::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_function_spec_merge() {
        let mut base = FunctionSpec::new("malloc");
        base.role = Some(Role::Allocator);
        base.returns = Some(ReturnSpec {
            nullness: Some(Nullness::MaybeNull),
            ..ReturnSpec::default()
        });

        let override_spec = FunctionSpec {
            name: "malloc".to_string(),
            role: None,
            pure: Some(false),
            noreturn: None,
            disabled: None,
            params: vec![ParamSpec {
                index: 0,
                semantic: Some("allocation_size".to_string()),
                ..ParamSpec::new(0)
            }],
            returns: Some(ReturnSpec {
                pointer: Some(Pointer::FreshHeap),
                ..ReturnSpec::default()
            }),
            taint: None,
        };

        base.merge(&override_spec);

        // Role preserved from base
        assert_eq!(base.role, Some(Role::Allocator));
        // Pure added from override
        assert_eq!(base.pure, Some(false));
        // Returns merged
        let ret = base.returns.as_ref().unwrap();
        assert_eq!(ret.nullness, Some(Nullness::MaybeNull)); // from base
        assert_eq!(ret.pointer, Some(Pointer::FreshHeap)); // from override
        // Params added
        assert_eq!(base.params.len(), 1);
        assert_eq!(base.params[0].semantic, Some("allocation_size".to_string()));
    }
}
