//! Selector API for specifying taint sources, sinks, and sanitizers.
//!
//! Selectors provide a user-friendly way to identify values of interest
//! in the value flow graph without dealing with raw `ValueId`s.

mod resolve;
mod sanitizers;
mod sinks;
mod sources;
mod spec_selectors;

pub use resolve::{ResolveError, SelectorResolver};
pub use sanitizers::SanitizerSelector;
pub use sinks::SinkSelector;
pub use sources::SourceSelector;
pub use spec_selectors::{
    sanitizers_from_specs, sinks_from_specs, sources_from_specs, taint_selectors_from_specs,
};

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use saf_core::air::AirModule;
use saf_core::ids::ValueId;

/// A selector for identifying values in the AIR module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selector {
    /// Select a specific value by ID.
    ValueId {
        /// The value ID (hex string).
        id: String,
    },
    /// Select function parameters by function name pattern.
    FunctionParam {
        /// Function name pattern (glob-style: `*` matches any).
        function: String,
        /// Parameter index (0-based), or None for all params.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
    },
    /// Select function return values by function name pattern.
    FunctionReturn {
        /// Function name pattern (glob-style).
        function: String,
    },
    /// Select all values in functions matching a pattern.
    FunctionAll {
        /// Function name pattern (glob-style).
        function: String,
    },
    /// Select global variables by name pattern.
    Global {
        /// Global name pattern (glob-style).
        name: String,
    },
    /// Select call results to specific functions.
    CallTo {
        /// Callee function name pattern.
        callee: String,
    },
    /// Select arguments passed to specific functions.
    ArgTo {
        /// Callee function name pattern.
        callee: String,
        /// Argument index (0-based), or None for all args.
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
    },
}

impl Selector {
    /// Create a selector for a specific value ID.
    #[must_use]
    pub fn value_id(id: impl Into<String>) -> Self {
        Self::ValueId { id: id.into() }
    }

    /// Create a selector for function parameters.
    #[must_use]
    pub fn function_param(function: impl Into<String>, index: Option<u32>) -> Self {
        Self::FunctionParam {
            function: function.into(),
            index,
        }
    }

    /// Create a selector for function return values.
    #[must_use]
    pub fn function_return(function: impl Into<String>) -> Self {
        Self::FunctionReturn {
            function: function.into(),
        }
    }

    /// Create a selector for all values in a function.
    #[must_use]
    pub fn function_all(function: impl Into<String>) -> Self {
        Self::FunctionAll {
            function: function.into(),
        }
    }

    /// Create a selector for global variables.
    #[must_use]
    pub fn global(name: impl Into<String>) -> Self {
        Self::Global { name: name.into() }
    }

    /// Create a selector for call results to a function.
    #[must_use]
    pub fn call_to(callee: impl Into<String>) -> Self {
        Self::CallTo {
            callee: callee.into(),
        }
    }

    /// Create a selector for arguments to a function.
    #[must_use]
    pub fn arg_to(callee: impl Into<String>, index: Option<u32>) -> Self {
        Self::ArgTo {
            callee: callee.into(),
            index,
        }
    }

    /// Resolve this selector to a set of value IDs.
    ///
    /// # Errors
    /// Returns an error if the selector cannot be resolved (e.g., invalid ID format).
    pub fn resolve(&self, module: &AirModule) -> Result<BTreeSet<ValueId>, ResolveError> {
        SelectorResolver::new(module).resolve(self)
    }
}

/// Resolve multiple selectors to a combined set of value IDs.
///
/// # Errors
/// Returns an error if any selector cannot be resolved.
pub fn resolve_selectors(
    selectors: &[Selector],
    module: &AirModule,
) -> Result<BTreeSet<ValueId>, ResolveError> {
    let resolver = SelectorResolver::new(module);
    let mut result = BTreeSet::new();
    for selector in selectors {
        result.extend(resolver.resolve(selector)?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_value_id_creation() {
        let sel = Selector::value_id("0x123");
        assert!(matches!(sel, Selector::ValueId { id } if id == "0x123"));
    }

    #[test]
    fn selector_function_param_creation() {
        let sel = Selector::function_param("main", Some(0));
        assert!(matches!(
            sel,
            Selector::FunctionParam { function, index } if function == "main" && index == Some(0)
        ));
    }

    #[test]
    fn selector_serialization_roundtrip() {
        let selectors = vec![
            Selector::value_id("0x123"),
            Selector::function_param("main", Some(0)),
            Selector::function_param("*", None),
            Selector::function_return("get_*"),
            Selector::global("g_*"),
            Selector::call_to("malloc"),
            Selector::arg_to("free", Some(0)),
        ];

        for sel in selectors {
            let json = serde_json::to_string(&sel).unwrap();
            let parsed: Selector = serde_json::from_str(&json).unwrap();
            assert_eq!(sel, parsed);
        }
    }

    #[test]
    fn selector_json_format() {
        let sel = Selector::function_param("main", Some(0));
        let json = serde_json::to_string(&sel).unwrap();
        assert!(json.contains("\"kind\":\"function_param\""));
        assert!(json.contains("\"function\":\"main\""));
    }
}
