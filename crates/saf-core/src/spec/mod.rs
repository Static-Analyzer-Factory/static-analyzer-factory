//! Function specification system for external/library function modeling.
//!
//! This module provides a pluggable system for describing the behavior of external
//! functions (libc, POSIX, user libraries) that cannot be analyzed directly. Specs
//! are used by various analyses:
//!
//! - **PTA**: Memory allocation, aliasing, pointer returns
//! - **Nullness**: Pre/post-conditions on pointer validity
//! - **Taint**: Sources, sinks, sanitizers, propagation rules
//! - **Interval**: Numeric return bounds
//!
//! # Spec Format
//!
//! Specs are defined in YAML files with version "1.0":
//!
//! ```yaml
//! version: "1.0"
//! specs:
//!   - name: malloc
//!     role: allocator
//!     returns:
//!       pointer: fresh_heap
//!       nullness: maybe_null
//!     params:
//!       - index: 0
//!         semantic: allocation_size
//! ```
//!
//! # Name Matching
//!
//! - **Exact**: `name: malloc`
//! - **Glob**: `name: "glob:str*"`
//! - **Regex**: `name: "regex:^mem(cpy|set|move)$"`
//!
//! # Discovery Order
//!
//! Specs are loaded from multiple paths (later overrides earlier per-function):
//! 1. `<binary>/../share/saf/specs/*.yaml` — shipped defaults
//! 2. `~/.saf/specs/*.yaml` — user global
//! 3. `./saf-specs/*.yaml` — project local
//! 4. `$SAF_SPECS_PATH/*.yaml` — explicit override
//!
//! # Example
//!
//! ```
//! use saf_core::spec::{SpecRegistry, FunctionSpec, Role};
//!
//! // Load from default paths
//! let registry = SpecRegistry::new(); // Empty for now
//!
//! // Programmatic construction
//! let mut registry = SpecRegistry::new();
//! let mut spec = FunctionSpec::new("my_alloc");
//! spec.role = Some(Role::Allocator);
//! registry.add(spec).unwrap();
//!
//! if let Some(spec) = registry.lookup("my_alloc") {
//!     assert_eq!(spec.role, Some(Role::Allocator));
//! }
//! ```

mod analyzed;
mod derived;
mod pattern;
mod registry;
mod schema;
mod types;

// Re-export main types
pub use analyzed::{AnalyzedSpecRegistry, LookupResult};
pub use derived::{BoundMode, ComputedBound, DerivedSpec};
pub use pattern::{NamePattern, PatternError};
pub use registry::{RegistryError, SpecRegistry};
pub use schema::{CURRENT_VERSION, SchemaError, SpecFile};
pub use types::{
    FunctionSpec, Nullness, ParamSpec, Pointer, ReturnSpec, Role, TaintLocation, TaintPropagation,
    TaintSpec,
};
