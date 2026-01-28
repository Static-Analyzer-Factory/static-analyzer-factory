//! Function summaries for compositional analysis.
//!
//! A [`FunctionSummary`] is a compact, serializable description of a function's
//! pointer and memory behavior. Summaries serve as the universal exchange format
//! between:
//!
//! - **YAML specs** (hand-written for external/library functions)
//! - **Analysis results** (computed by PTA, value-flow, etc.)
//! - **Compositional analysis** (bottom-up summary composition)
//!
//! Access paths ([`AccessPath`]) describe memory locations relative to function
//! parameters and globals, enabling field-sensitive and context-sensitive
//! summaries without concrete `ValueId` bindings.
//!
//! # Persistence
//!
//! Summaries support JSON serialization to disk via [`FunctionSummary::save`] and
//! [`FunctionSummary::load`], stored at `{cache_dir}/summaries/{function_id_hex}.json`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ids::{FunctionId, ValueId};
use crate::spec::{
    FunctionSpec, Nullness, ParamSpec, Pointer, ReturnSpec, Role, TaintLocation, TaintSpec,
};

// ---------------------------------------------------------------------------
// AccessPath
// ---------------------------------------------------------------------------

/// Describes a memory access location relative to function parameters or globals.
///
/// Access paths form a tree rooted at a base (`Param`, `Global`, or `Return`)
/// with `Deref` and `Field` steps descending into pointed-to memory.
///
/// # Examples
///
/// - `Param(0)` -- the first parameter itself
/// - `Deref(Param(0))` -- what the first parameter points to
/// - `Field(Deref(Param(0)), 2)` -- field 2 of the struct pointed to by param 0
/// - `Return` -- the return value
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AccessPath {
    /// A function parameter by zero-based index.
    Param(u32),
    /// A global variable identified by its `ValueId`.
    Global(ValueId),
    /// Dereference of an inner access path (one level of pointer indirection).
    Deref(Box<AccessPath>),
    /// Field access at a given byte offset within a struct.
    Field(Box<AccessPath>, u32),
    /// The function return value.
    Return,
}

/// Errors that can occur when parsing an [`AccessPath`] from a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessPathParseError {
    /// The input string was empty.
    Empty,
    /// An unrecognized base token was encountered.
    UnknownBase(String),
    /// A `param.N` token had a non-integer index.
    InvalidParamIndex(String),
    /// A `global(0x...)` token had an invalid hex ID.
    InvalidGlobalId(String),
    /// A `field(N)` step had a non-integer offset.
    InvalidFieldOffset(String),
    /// An unrecognized step after `->` was encountered.
    UnknownStep(String),
}

impl fmt::Display for AccessPathParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty access path string"),
            Self::UnknownBase(s) => write!(f, "unknown access path base: '{s}'"),
            Self::InvalidParamIndex(s) => write!(f, "invalid param index: '{s}'"),
            Self::InvalidGlobalId(s) => write!(f, "invalid global ID: '{s}'"),
            Self::InvalidFieldOffset(s) => write!(f, "invalid field offset: '{s}'"),
            Self::UnknownStep(s) => write!(f, "unknown access path step: '{s}'"),
        }
    }
}

impl std::error::Error for AccessPathParseError {}

impl AccessPath {
    /// Parse an access path from its string representation.
    ///
    /// Supported formats:
    /// - `"return"` -- [`AccessPath::Return`]
    /// - `"param.N"` -- [`AccessPath::Param`] with index N
    /// - `"global(0x...)"` -- [`AccessPath::Global`] with hex ID
    /// - `"param.0->deref"` -- [`AccessPath::Deref`] wrapping `Param(0)`
    /// - `"param.0->field(8)"` -- [`AccessPath::Field`] with offset 8
    /// - Steps can be chained: `"param.0->deref->field(4)->deref"`
    ///
    /// # Errors
    ///
    /// Returns [`AccessPathParseError`] if the string is empty, contains an
    /// unrecognized base or step token, or has invalid numeric indices.
    pub fn parse(s: &str) -> Result<Self, AccessPathParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(AccessPathParseError::Empty);
        }

        // Split into base and steps on "->"
        let mut parts = s.split("->");
        let base_str = parts.next().ok_or(AccessPathParseError::Empty)?.trim();

        let mut path = Self::parse_base(base_str)?;

        for step_str in parts {
            let step = step_str.trim();
            path = Self::parse_step(path, step)?;
        }

        Ok(path)
    }

    /// Parse the base token (before any `->` steps).
    fn parse_base(s: &str) -> Result<Self, AccessPathParseError> {
        if s == "return" {
            return Ok(AccessPath::Return);
        }

        if let Some(idx_str) = s.strip_prefix("param.") {
            let idx = idx_str
                .parse::<u32>()
                .map_err(|_| AccessPathParseError::InvalidParamIndex(idx_str.to_string()))?;
            return Ok(AccessPath::Param(idx));
        }

        if let Some(inner) = s.strip_prefix("global(").and_then(|r| r.strip_suffix(')')) {
            let hex_str = inner.strip_prefix("0x").unwrap_or(inner);
            let id = u128::from_str_radix(hex_str, 16)
                .map_err(|_| AccessPathParseError::InvalidGlobalId(inner.to_string()))?;
            return Ok(AccessPath::Global(ValueId::new(id)));
        }

        Err(AccessPathParseError::UnknownBase(s.to_string()))
    }

    /// Parse a single step token and wrap the current path.
    fn parse_step(base: Self, step: &str) -> Result<Self, AccessPathParseError> {
        if step == "deref" {
            return Ok(AccessPath::Deref(Box::new(base)));
        }

        if let Some(inner) = step
            .strip_prefix("field(")
            .and_then(|r| r.strip_suffix(')'))
        {
            let offset = inner
                .parse::<u32>()
                .map_err(|_| AccessPathParseError::InvalidFieldOffset(inner.to_string()))?;
            return Ok(AccessPath::Field(Box::new(base), offset));
        }

        Err(AccessPathParseError::UnknownStep(step.to_string()))
    }

    /// Compute the depth of this access path (number of `Deref`/`Field` steps).
    ///
    /// Base paths (`Param`, `Global`, `Return`) have depth 0.
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            AccessPath::Param(_) | AccessPath::Global(_) | AccessPath::Return => 0,
            AccessPath::Deref(inner) | AccessPath::Field(inner, _) => 1 + inner.depth(),
        }
    }

    /// Truncate this access path to at most `k` levels of `Deref`/`Field`.
    ///
    /// If the path is already within the limit, returns a clone. Otherwise,
    /// the deepest steps beyond `k` are dropped, and the path is terminated
    /// at the k-th level.
    #[must_use]
    pub fn truncate(&self, k: u32) -> Self {
        self.truncate_inner(k as usize)
    }

    /// Recursive truncation helper.
    fn truncate_inner(&self, remaining: usize) -> Self {
        match self {
            AccessPath::Param(_) | AccessPath::Global(_) | AccessPath::Return => self.clone(),
            AccessPath::Deref(inner) => {
                if remaining == 0 {
                    // At the limit -- return the inner base without this Deref
                    inner.truncate_inner(0)
                } else {
                    AccessPath::Deref(Box::new(inner.truncate_inner(remaining - 1)))
                }
            }
            AccessPath::Field(inner, offset) => {
                if remaining == 0 {
                    inner.truncate_inner(0)
                } else {
                    AccessPath::Field(Box::new(inner.truncate_inner(remaining - 1)), *offset)
                }
            }
        }
    }
}

impl fmt::Display for AccessPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessPath::Return => write!(f, "return"),
            AccessPath::Param(idx) => write!(f, "param.{idx}"),
            AccessPath::Global(id) => write!(f, "global({id})"),
            AccessPath::Deref(inner) => write!(f, "{inner}->deref"),
            AccessPath::Field(inner, offset) => write!(f, "{inner}->field({offset})"),
        }
    }
}

// ---------------------------------------------------------------------------
// Supporting enums
// ---------------------------------------------------------------------------

/// How a function summary was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummarySource {
    /// From a YAML spec file (hand-authored or shipped).
    Spec,
    /// Computed by static analysis.
    Analysis,
    /// Default conservative assumption.
    Default,
}

/// Precision guarantee of a function summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryPrecision {
    /// Sound over-approximation (no false negatives).
    Sound,
    /// Best-effort approximation (may miss behaviors).
    BestEffort,
}

/// Nullness classification for a pointer value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryNullness {
    /// Guaranteed non-null.
    NonNull,
    /// May be null.
    MaybeNull,
    /// Always null.
    AlwaysNull,
}

/// High-level role of a function in the summary system.
///
/// Mirrors [`Role`] but is self-contained for summary serialization
/// without depending on the spec module's enum variants.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryRole {
    /// Memory allocator (e.g., `malloc`, `new`).
    Allocator,
    /// Memory reallocator (e.g., `realloc`).
    Reallocator,
    /// Memory deallocator (e.g., `free`, `delete`).
    Deallocator,
    /// Taint source (e.g., `getenv`, `recv`).
    Source,
    /// Taint sink (e.g., `system`, `exec`).
    Sink,
    /// Taint sanitizer.
    Sanitizer,
    /// String operation (e.g., `strlen`, `strcpy`).
    StringOperation,
    /// I/O operation (e.g., `read`, `write`).
    Io,
    /// Custom role for domain-specific analyses.
    Custom(String),
}

impl From<&Role> for SummaryRole {
    fn from(role: &Role) -> Self {
        match role {
            Role::Allocator => SummaryRole::Allocator,
            Role::Reallocator => SummaryRole::Reallocator,
            Role::Deallocator => SummaryRole::Deallocator,
            Role::Source => SummaryRole::Source,
            Role::Sink => SummaryRole::Sink,
            Role::Sanitizer => SummaryRole::Sanitizer,
            Role::StringOperation => SummaryRole::StringOperation,
            Role::Io => SummaryRole::Io,
            Role::Custom(s) => SummaryRole::Custom(s.clone()),
        }
    }
}

impl From<&SummaryRole> for Role {
    fn from(role: &SummaryRole) -> Self {
        match role {
            SummaryRole::Allocator => Role::Allocator,
            SummaryRole::Reallocator => Role::Reallocator,
            SummaryRole::Deallocator => Role::Deallocator,
            SummaryRole::Source => Role::Source,
            SummaryRole::Sink => Role::Sink,
            SummaryRole::Sanitizer => Role::Sanitizer,
            SummaryRole::StringOperation => Role::StringOperation,
            SummaryRole::Io => Role::Io,
            SummaryRole::Custom(s) => Role::Custom(s.clone()),
        }
    }
}

impl SummaryNullness {
    /// Convert from the spec system's [`Nullness`] to [`SummaryNullness`].
    ///
    /// The spec system has four variants (`NotNull`, `MaybeNull`,
    /// `RequiredNonnull`, `Nullable`). For summary purposes, `NotNull` and
    /// `RequiredNonnull` both map to `NonNull`, and `Nullable` maps to
    /// `MaybeNull`.
    #[must_use]
    pub fn from_spec_nullness(n: &Nullness) -> Self {
        match n {
            Nullness::NotNull | Nullness::RequiredNonnull => SummaryNullness::NonNull,
            Nullness::MaybeNull | Nullness::Nullable => SummaryNullness::MaybeNull,
        }
    }

    /// Convert to the spec system's [`Nullness`].
    ///
    /// This is lossy: `AlwaysNull` has no direct spec equivalent and maps to
    /// `MaybeNull` (the conservative choice).
    #[must_use]
    pub fn to_spec_nullness(self) -> Nullness {
        match self {
            SummaryNullness::NonNull => Nullness::NotNull,
            SummaryNullness::MaybeNull | SummaryNullness::AlwaysNull => Nullness::MaybeNull,
        }
    }
}

// ---------------------------------------------------------------------------
// Effect types
// ---------------------------------------------------------------------------

/// Describes the effect of a function on its return value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReturnEffect {
    /// What the return value aliases (e.g., `Param(0)` means "returns param 0").
    pub aliases: Option<AccessPath>,
    /// Whether the return value is freshly allocated.
    pub fresh_allocation: bool,
}

/// Describes a memory side-effect of a function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoryEffect {
    /// The access path being read or written.
    pub path: AccessPath,
    /// Whether this effect is a read.
    pub reads: bool,
    /// Whether this effect is a write.
    pub writes: bool,
}

/// Describes an allocation performed by a function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AllocationEffect {
    /// Where the allocated pointer flows to (usually `Return` or a parameter).
    pub target: AccessPath,
    /// Whether the allocation is heap-based (vs. stack/static).
    pub heap: bool,
}

/// Reference to a callee from within a function body.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CalleeRef {
    /// Direct call to a known function.
    Direct(FunctionId),
    /// Indirect call through a function pointer at an access path.
    Indirect(AccessPath),
}

/// Taint propagation rule within a function summary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SummaryTaintPropagation {
    /// Where taint originates (e.g., `Param(0)`).
    pub from: AccessPath,
    /// Where taint flows to (e.g., `Return`).
    pub to: AccessPath,
}

/// How a return value is bounded by a parameter property.
///
/// Mirrors [`crate::spec::derived::BoundMode`] with serde support for
/// summary persistence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryBoundMode {
    /// Return in `[0, alloc_size(param) - 1]`.
    AllocSizeMinusOne,
    /// Return in `[0, alloc_size(param)]`.
    AllocSize,
    /// Return in `[-1, param_value - 1]`.
    ParamValueMinusOne,
}

/// A computed return bound: return interval depends on a parameter property.
///
/// Mirrors [`crate::spec::derived::ComputedBound`] with serde support for
/// summary persistence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SummaryComputedBound {
    /// Which parameter's property bounds the return value.
    pub param_index: u32,
    /// How the return is bounded.
    pub mode: SummaryBoundMode,
}

// ---------------------------------------------------------------------------
// FunctionSummary
// ---------------------------------------------------------------------------

/// A compact, serializable description of a function's pointer and memory behavior.
///
/// This is the universal exchange format for function behavior information.
/// Summaries can originate from YAML specs, static analysis, or default
/// conservative assumptions, and are used by incremental and compositional
/// analyses to avoid re-analyzing unchanged functions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSummary {
    /// The function this summary describes.
    pub function_id: FunctionId,

    /// Monotonically increasing version for cache invalidation.
    pub version: u64,

    /// Effects on the return value (aliasing, fresh allocation).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub return_effects: Vec<ReturnEffect>,

    /// Memory read/write side effects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_effects: Vec<MemoryEffect>,

    /// Allocation effects (heap/stack objects created by this function).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allocation_effects: Vec<AllocationEffect>,

    /// Known callees (direct and indirect).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub callees: BTreeSet<CalleeRef>,

    /// High-level role (allocator, source, sink, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<SummaryRole>,

    /// Whether the function is pure (no side effects, result depends only on inputs).
    #[serde(default)]
    pub pure: bool,

    /// Whether the function never returns (e.g., `exit`, `abort`).
    #[serde(default)]
    pub noreturn: bool,

    /// Per-parameter nullness classification.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub param_nullness: BTreeMap<u32, SummaryNullness>,

    /// Return value nullness classification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_nullness: Option<SummaryNullness>,

    /// Taint propagation rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub taint_propagation: Vec<SummaryTaintPropagation>,

    /// Computed return value bound relative to a parameter property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_bound: Option<SummaryComputedBound>,

    /// Per-parameter: whether the callee frees this parameter.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub param_freed: BTreeMap<u32, bool>,

    /// Per-parameter: whether the callee dereferences this parameter.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub param_dereferenced: BTreeMap<u32, bool>,

    /// How this summary was produced.
    pub source: SummarySource,

    /// Precision guarantee.
    pub precision: SummaryPrecision,
}

impl FunctionSummary {
    /// Create a new default (conservative) summary for the given function.
    #[must_use]
    pub fn default_for(function_id: FunctionId) -> Self {
        Self {
            function_id,
            version: 0,
            return_effects: Vec::new(),
            memory_effects: Vec::new(),
            allocation_effects: Vec::new(),
            callees: BTreeSet::new(),
            role: None,
            pure: false,
            noreturn: false,
            param_nullness: BTreeMap::new(),
            return_nullness: None,
            taint_propagation: Vec::new(),
            return_bound: None,
            param_freed: BTreeMap::new(),
            param_dereferenced: BTreeMap::new(),
            source: SummarySource::Default,
            precision: SummaryPrecision::Sound,
        }
    }

    /// Save this summary to `{cache_dir}/summaries/{function_id_hex}.json`.
    ///
    /// Creates the `summaries/` subdirectory if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the directory cannot be created or the file
    /// cannot be written.
    pub fn save(&self, cache_dir: &Path) -> Result<(), std::io::Error> {
        let dir = cache_dir.join("summaries");
        std::fs::create_dir_all(&dir)?;

        let filename = format!("{}.json", self.function_id.to_hex());
        let path = dir.join(filename);

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load a summary from `{cache_dir}/summaries/{function_id_hex}.json`.
    ///
    /// Returns `Ok(None)` if the file does not exist.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the file exists but cannot be read or parsed.
    pub fn load(
        cache_dir: &Path,
        function_id: &FunctionId,
    ) -> Result<Option<Self>, std::io::Error> {
        let filename = format!("{}.json", function_id.to_hex());
        let path = cache_dir.join("summaries").join(filename);

        if !path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&path)?;
        let summary: Self = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(summary))
    }

    // -- Spec conversion --

    /// Convert a [`FunctionSpec`] into a [`FunctionSummary`].
    ///
    /// This conversion is lossless: every spec field maps to a corresponding
    /// summary field. The resulting summary has `source = Spec` and
    /// `precision = Sound` (specs are assumed to be authoritative).
    #[must_use]
    pub fn from_spec(spec: &FunctionSpec, function_id: FunctionId) -> Self {
        let mut summary = Self::default_for(function_id);
        summary.source = SummarySource::Spec;

        // Role
        summary.role = spec.role.as_ref().map(SummaryRole::from);

        // Flags
        summary.pure = spec.pure.unwrap_or(false);
        summary.noreturn = spec.noreturn.unwrap_or(false);

        // Per-parameter effects
        for param in &spec.params {
            // Memory effects from reads/modifies
            let reads = param.reads.unwrap_or(false);
            let writes = param.modifies.unwrap_or(false);
            if reads || writes {
                summary.memory_effects.push(MemoryEffect {
                    path: AccessPath::Deref(Box::new(AccessPath::Param(param.index))),
                    reads,
                    writes,
                });
            }

            // Nullness
            if let Some(ref nullness) = param.nullness {
                summary
                    .param_nullness
                    .insert(param.index, SummaryNullness::from_spec_nullness(nullness));
            }

            // Callback → indirect callee
            if param.callback.unwrap_or(false) {
                summary
                    .callees
                    .insert(CalleeRef::Indirect(AccessPath::Param(param.index)));
            }
        }

        // Return effects
        if let Some(ref ret) = spec.returns {
            Self::convert_return_spec(ret, &mut summary);
        }

        // Taint propagation
        if let Some(ref taint) = spec.taint {
            Self::convert_taint_spec(taint, &mut summary);
        }

        summary
    }

    /// Helper: convert a [`ReturnSpec`] into summary return/allocation effects.
    fn convert_return_spec(ret: &ReturnSpec, summary: &mut Self) {
        // Nullness
        if let Some(ref nullness) = ret.nullness {
            summary.return_nullness = Some(SummaryNullness::from_spec_nullness(nullness));
        }

        // Fresh allocation
        let is_fresh = matches!(ret.pointer, Some(Pointer::FreshHeap | Pointer::FreshStack));
        let is_heap = matches!(ret.pointer, Some(Pointer::FreshHeap));

        if is_fresh {
            summary.allocation_effects.push(AllocationEffect {
                target: AccessPath::Return,
                heap: is_heap,
            });
            summary.return_effects.push(ReturnEffect {
                aliases: None,
                fresh_allocation: true,
            });
        }

        // Alias
        if let Some(param_idx) = ret.alias_param_index() {
            summary.return_effects.push(ReturnEffect {
                aliases: Some(AccessPath::Param(param_idx)),
                fresh_allocation: false,
            });
        }
    }

    /// Helper: convert a [`TaintSpec`] into summary taint propagation rules.
    fn convert_taint_spec(taint: &TaintSpec, summary: &mut Self) {
        for prop in &taint.propagates {
            let from = Self::taint_location_to_access_path(prop.from);
            let Some(from) = from else {
                continue;
            };
            for to_loc in &prop.to {
                let Some(to) = Self::taint_location_to_access_path(*to_loc) else {
                    continue;
                };
                summary.taint_propagation.push(SummaryTaintPropagation {
                    from: from.clone(),
                    to,
                });
            }
        }
    }

    /// Convert a [`TaintLocation`] to an [`AccessPath`].
    ///
    /// Returns `None` for `TaintLocation::Unknown`.
    fn taint_location_to_access_path(loc: TaintLocation) -> Option<AccessPath> {
        match loc {
            TaintLocation::Return => Some(AccessPath::Return),
            TaintLocation::Param(idx) => Some(AccessPath::Param(idx)),
            TaintLocation::Unknown => None,
        }
    }

    /// Convert this summary to a simple [`FunctionSpec`] (lossy).
    ///
    /// Access paths are collapsed to parameter-level reads/modifies. Complex
    /// effects (field-sensitive access, deep dereferences) are simplified.
    /// The resulting spec preserves role, pure, noreturn, nullness, taint,
    /// and parameter-level read/write information.
    #[must_use]
    pub fn to_simple_spec(&self, name: &str) -> FunctionSpec {
        let mut spec = FunctionSpec::new(name);

        // Role
        spec.role = self.role.as_ref().map(Role::from);

        // Flags
        if self.pure {
            spec.pure = Some(true);
        }
        if self.noreturn {
            spec.noreturn = Some(true);
        }

        // Collect per-parameter info from memory effects
        let mut param_reads: BTreeMap<u32, bool> = BTreeMap::new();
        let mut param_writes: BTreeMap<u32, bool> = BTreeMap::new();

        for effect in &self.memory_effects {
            if let Some(idx) = Self::access_path_root_param(&effect.path) {
                if effect.reads {
                    param_reads.insert(idx, true);
                }
                if effect.writes {
                    param_writes.insert(idx, true);
                }
            }
        }

        // Build ParamSpecs from all param indices we know about
        let mut param_indices: BTreeSet<u32> = BTreeSet::new();
        param_indices.extend(param_reads.keys());
        param_indices.extend(param_writes.keys());
        param_indices.extend(self.param_nullness.keys());

        for idx in param_indices {
            let mut ps = ParamSpec::new(idx);
            if param_reads.contains_key(&idx) {
                ps.reads = Some(true);
            }
            if param_writes.contains_key(&idx) {
                ps.modifies = Some(true);
            }
            if let Some(nullness) = self.param_nullness.get(&idx) {
                ps.nullness = Some(nullness.to_spec_nullness());
            }
            spec.params.push(ps);
        }

        // Return spec
        let has_return_info = self.return_nullness.is_some()
            || !self.return_effects.is_empty()
            || !self.allocation_effects.is_empty();

        if has_return_info {
            let mut ret = ReturnSpec::new();

            if let Some(nullness) = self.return_nullness {
                ret.nullness = Some(nullness.to_spec_nullness());
            }

            // Check for fresh allocation targeting Return
            for alloc in &self.allocation_effects {
                if alloc.target == AccessPath::Return {
                    ret.pointer = Some(if alloc.heap {
                        Pointer::FreshHeap
                    } else {
                        Pointer::FreshStack
                    });
                    break;
                }
            }

            // Check for return alias
            for re in &self.return_effects {
                if let Some(AccessPath::Param(idx)) = &re.aliases {
                    ret.aliases = Some(format!("param.{idx}"));
                    break;
                }
            }

            spec.returns = Some(ret);
        }

        // Taint propagation
        if !self.taint_propagation.is_empty() {
            let mut propagates = Vec::new();
            for tp in &self.taint_propagation {
                let Some(from) = Self::access_path_to_taint_location(&tp.from) else {
                    continue;
                };
                let Some(to) = Self::access_path_to_taint_location(&tp.to) else {
                    continue;
                };
                // Group by `from` to produce multi-target rules
                if let Some(existing) = propagates
                    .iter_mut()
                    .find(|p: &&mut crate::spec::TaintPropagation| p.from == from)
                {
                    existing.to.push(to);
                } else {
                    propagates.push(crate::spec::TaintPropagation { from, to: vec![to] });
                }
            }
            if !propagates.is_empty() {
                spec.taint = Some(TaintSpec { propagates });
            }
        }

        spec
    }

    /// Extract the root parameter index from an access path, if it is
    /// rooted at a `Param`.
    fn access_path_root_param(path: &AccessPath) -> Option<u32> {
        match path {
            AccessPath::Param(idx) => Some(*idx),
            AccessPath::Deref(inner) | AccessPath::Field(inner, _) => {
                Self::access_path_root_param(inner)
            }
            AccessPath::Global(_) | AccessPath::Return => None,
        }
    }

    /// Convert an [`AccessPath`] to a [`TaintLocation`] (lossy).
    ///
    /// Only `Param` and `Return` bases are representable; other paths
    /// return `None`.
    fn access_path_to_taint_location(path: &AccessPath) -> Option<TaintLocation> {
        match path {
            AccessPath::Return => Some(TaintLocation::Return),
            AccessPath::Param(idx) => Some(TaintLocation::Param(*idx)),
            // Complex paths (Deref, Field, Global) cannot be represented
            // in the simple TaintLocation system.
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{TaintPropagation as SpecTaintPropagation, TaintSpec};

    // -- AccessPath parsing --

    #[test]
    fn parse_return() {
        let path = AccessPath::parse("return").expect("parse return");
        assert_eq!(path, AccessPath::Return);
    }

    #[test]
    fn parse_param() {
        let path = AccessPath::parse("param.0").expect("parse param.0");
        assert_eq!(path, AccessPath::Param(0));

        let path = AccessPath::parse("param.42").expect("parse param.42");
        assert_eq!(path, AccessPath::Param(42));
    }

    #[test]
    fn parse_global() {
        let path =
            AccessPath::parse("global(0x00000000000000000000000000000abc)").expect("parse global");
        assert_eq!(path, AccessPath::Global(ValueId::new(0xabc)));
    }

    #[test]
    fn parse_deref() {
        let path = AccessPath::parse("param.0->deref").expect("parse deref");
        assert_eq!(path, AccessPath::Deref(Box::new(AccessPath::Param(0))));
    }

    #[test]
    fn parse_field() {
        let path = AccessPath::parse("param.0->field(8)").expect("parse field");
        assert_eq!(path, AccessPath::Field(Box::new(AccessPath::Param(0)), 8));
    }

    #[test]
    fn parse_chained_steps() {
        let path =
            AccessPath::parse("param.1->deref->field(4)->deref").expect("parse chained steps");
        let expected = AccessPath::Deref(Box::new(AccessPath::Field(
            Box::new(AccessPath::Deref(Box::new(AccessPath::Param(1)))),
            4,
        )));
        assert_eq!(path, expected);
    }

    #[test]
    fn parse_display_roundtrip() {
        let cases = [
            "return",
            "param.0",
            "param.3->deref",
            "param.1->field(8)",
            "param.0->deref->field(4)->deref",
        ];
        for input in &cases {
            let parsed = AccessPath::parse(input).expect("parse");
            let displayed = parsed.to_string();
            let reparsed = AccessPath::parse(&displayed).expect("reparse");
            assert_eq!(parsed, reparsed, "roundtrip failed for '{input}'");
        }
    }

    #[test]
    fn parse_errors() {
        assert!(AccessPath::parse("").is_err());
        assert!(AccessPath::parse("unknown").is_err());
        assert!(AccessPath::parse("param.abc").is_err());
        assert!(AccessPath::parse("param.0->bogus").is_err());
        assert!(AccessPath::parse("param.0->field(abc)").is_err());
    }

    // -- AccessPath depth --

    #[test]
    fn depth_base_paths() {
        assert_eq!(AccessPath::Return.depth(), 0);
        assert_eq!(AccessPath::Param(0).depth(), 0);
        assert_eq!(AccessPath::Global(ValueId::new(1)).depth(), 0);
    }

    #[test]
    fn depth_nested() {
        let one = AccessPath::Deref(Box::new(AccessPath::Param(0)));
        assert_eq!(one.depth(), 1);

        let two = AccessPath::Field(Box::new(one.clone()), 4);
        assert_eq!(two.depth(), 2);

        let three = AccessPath::Deref(Box::new(two));
        assert_eq!(three.depth(), 3);
    }

    // -- AccessPath truncation --

    #[test]
    fn truncate_within_limit() {
        let path = AccessPath::Deref(Box::new(AccessPath::Param(0)));
        let truncated = path.truncate(5);
        assert_eq!(truncated, path);
    }

    #[test]
    fn truncate_at_zero() {
        let path = AccessPath::Deref(Box::new(AccessPath::Field(
            Box::new(AccessPath::Param(0)),
            8,
        )));
        let truncated = path.truncate(0);
        assert_eq!(truncated, AccessPath::Param(0));
        assert_eq!(truncated.depth(), 0);
    }

    #[test]
    fn truncate_at_one() {
        // param.0->deref->field(4) (depth 2) truncated to k=1
        // The outermost step (Field) is kept, but the inner Deref is stripped,
        // resulting in Field(Param(0), 4) with depth 1.
        let path = AccessPath::Field(
            Box::new(AccessPath::Deref(Box::new(AccessPath::Param(0)))),
            4,
        );
        let truncated = path.truncate(1);
        assert_eq!(truncated.depth(), 1);
        assert_eq!(
            truncated,
            AccessPath::Field(Box::new(AccessPath::Param(0)), 4)
        );
    }

    #[test]
    fn truncate_deep_chain() {
        // param.0->deref->field(4)->deref (depth 3) truncated to k=2
        let path = AccessPath::Deref(Box::new(AccessPath::Field(
            Box::new(AccessPath::Deref(Box::new(AccessPath::Param(0)))),
            4,
        )));
        assert_eq!(path.depth(), 3);
        let truncated = path.truncate(2);
        assert_eq!(truncated.depth(), 2);
    }

    // -- FunctionSummary serialization --

    #[test]
    fn summary_serde_roundtrip() {
        let fid = FunctionId::derive(b"test_function");
        let mut summary = FunctionSummary::default_for(fid);
        summary.version = 3;
        summary.pure = true;
        summary.role = Some(SummaryRole::Allocator);
        summary.return_nullness = Some(SummaryNullness::MaybeNull);
        summary.param_nullness.insert(0, SummaryNullness::NonNull);
        summary.return_effects.push(ReturnEffect {
            aliases: Some(AccessPath::Param(0)),
            fresh_allocation: false,
        });
        summary.memory_effects.push(MemoryEffect {
            path: AccessPath::Deref(Box::new(AccessPath::Param(1))),
            reads: true,
            writes: false,
        });
        summary.allocation_effects.push(AllocationEffect {
            target: AccessPath::Return,
            heap: true,
        });
        summary
            .callees
            .insert(CalleeRef::Direct(FunctionId::derive(b"helper")));
        summary
            .callees
            .insert(CalleeRef::Indirect(AccessPath::Param(2)));
        summary.taint_propagation.push(SummaryTaintPropagation {
            from: AccessPath::Param(0),
            to: AccessPath::Return,
        });
        summary.return_bound = Some(SummaryComputedBound {
            param_index: 0,
            mode: SummaryBoundMode::AllocSizeMinusOne,
        });
        summary.param_freed.insert(0, true);
        summary.param_dereferenced.insert(1, true);
        summary.source = SummarySource::Analysis;
        summary.precision = SummaryPrecision::BestEffort;

        let json = serde_json::to_string_pretty(&summary).expect("serialize");
        let deserialized: FunctionSummary = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.function_id, fid);
        assert_eq!(deserialized.version, 3);
        assert!(deserialized.pure);
        assert_eq!(deserialized.role, Some(SummaryRole::Allocator));
        assert_eq!(
            deserialized.return_nullness,
            Some(SummaryNullness::MaybeNull)
        );
        assert_eq!(deserialized.return_effects.len(), 1);
        assert_eq!(deserialized.memory_effects.len(), 1);
        assert_eq!(deserialized.allocation_effects.len(), 1);
        assert_eq!(deserialized.callees.len(), 2);
        assert_eq!(deserialized.taint_propagation.len(), 1);
        assert!(deserialized.return_bound.is_some());
        assert_eq!(deserialized.param_freed.get(&0), Some(&true));
        assert_eq!(deserialized.param_dereferenced.get(&1), Some(&true));
        assert_eq!(deserialized.source, SummarySource::Analysis);
        assert_eq!(deserialized.precision, SummaryPrecision::BestEffort);
    }

    #[test]
    fn summary_default_for_is_conservative() {
        let fid = FunctionId::derive(b"unknown");
        let summary = FunctionSummary::default_for(fid);

        assert_eq!(summary.function_id, fid);
        assert_eq!(summary.version, 0);
        assert!(!summary.pure);
        assert!(!summary.noreturn);
        assert!(summary.return_effects.is_empty());
        assert!(summary.memory_effects.is_empty());
        assert!(summary.allocation_effects.is_empty());
        assert!(summary.callees.is_empty());
        assert!(summary.role.is_none());
        assert!(summary.return_nullness.is_none());
        assert_eq!(summary.source, SummarySource::Default);
        assert_eq!(summary.precision, SummaryPrecision::Sound);
    }

    // -- Disk persistence --

    #[test]
    fn save_load_roundtrip() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let fid = FunctionId::derive(b"save_load_test");
        let mut summary = FunctionSummary::default_for(fid);
        summary.version = 7;
        summary.pure = true;
        summary.noreturn = true;
        summary.role = Some(SummaryRole::Deallocator);
        summary.source = SummarySource::Spec;

        summary.save(tmp_dir.path()).expect("save");

        let loaded = FunctionSummary::load(tmp_dir.path(), &fid).expect("load");
        let loaded = loaded.expect("should exist");

        assert_eq!(loaded.function_id, fid);
        assert_eq!(loaded.version, 7);
        assert!(loaded.pure);
        assert!(loaded.noreturn);
        assert_eq!(loaded.role, Some(SummaryRole::Deallocator));
        assert_eq!(loaded.source, SummarySource::Spec);
    }

    #[test]
    fn load_missing_returns_none() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let fid = FunctionId::derive(b"nonexistent");

        let result = FunctionSummary::load(tmp_dir.path(), &fid).expect("load should succeed");
        assert!(result.is_none());
    }

    // -- Supporting type serde --

    #[test]
    fn summary_source_serde() {
        let json = serde_json::to_string(&SummarySource::Analysis).expect("serialize");
        assert_eq!(json, "\"analysis\"");

        let parsed: SummarySource = serde_json::from_str("\"spec\"").expect("deserialize");
        assert_eq!(parsed, SummarySource::Spec);
    }

    #[test]
    fn summary_precision_serde() {
        let json = serde_json::to_string(&SummaryPrecision::Sound).expect("serialize");
        assert_eq!(json, "\"sound\"");

        let parsed: SummaryPrecision =
            serde_json::from_str("\"best_effort\"").expect("deserialize");
        assert_eq!(parsed, SummaryPrecision::BestEffort);
    }

    #[test]
    fn summary_nullness_serde() {
        let json = serde_json::to_string(&SummaryNullness::AlwaysNull).expect("serialize");
        assert_eq!(json, "\"always_null\"");
    }

    #[test]
    fn summary_role_serde() {
        let json = serde_json::to_string(&SummaryRole::Allocator).expect("serialize");
        assert_eq!(json, "\"allocator\"");

        let custom = SummaryRole::Custom("validator".to_string());
        let json = serde_json::to_string(&custom).expect("serialize custom");
        let parsed: SummaryRole = serde_json::from_str(&json).expect("deserialize custom");
        assert_eq!(parsed, custom);
    }

    #[test]
    fn callee_ref_serde() {
        let direct = CalleeRef::Direct(FunctionId::derive(b"target"));
        let json = serde_json::to_string(&direct).expect("serialize");
        let parsed: CalleeRef = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, direct);

        let indirect = CalleeRef::Indirect(AccessPath::Param(0));
        let json = serde_json::to_string(&indirect).expect("serialize");
        let parsed: CalleeRef = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, indirect);
    }

    // -- Role conversion --

    #[test]
    fn role_conversion_roundtrip() {
        let roles = [
            Role::Allocator,
            Role::Reallocator,
            Role::Deallocator,
            Role::Source,
            Role::Sink,
            Role::Sanitizer,
            Role::StringOperation,
            Role::Io,
            Role::Custom("validator".to_string()),
        ];
        for role in &roles {
            let summary_role = SummaryRole::from(role);
            let back = Role::from(&summary_role);
            assert_eq!(&back, role);
        }
    }

    // -- Nullness conversion --

    #[test]
    fn nullness_conversion() {
        assert_eq!(
            SummaryNullness::from_spec_nullness(&Nullness::NotNull),
            SummaryNullness::NonNull
        );
        assert_eq!(
            SummaryNullness::from_spec_nullness(&Nullness::RequiredNonnull),
            SummaryNullness::NonNull
        );
        assert_eq!(
            SummaryNullness::from_spec_nullness(&Nullness::MaybeNull),
            SummaryNullness::MaybeNull
        );
        assert_eq!(
            SummaryNullness::from_spec_nullness(&Nullness::Nullable),
            SummaryNullness::MaybeNull
        );
        // Roundtrip (lossy for AlwaysNull)
        assert_eq!(
            SummaryNullness::NonNull.to_spec_nullness(),
            Nullness::NotNull
        );
        assert_eq!(
            SummaryNullness::MaybeNull.to_spec_nullness(),
            Nullness::MaybeNull
        );
        assert_eq!(
            SummaryNullness::AlwaysNull.to_spec_nullness(),
            Nullness::MaybeNull
        );
    }

    // -- from_spec --

    #[test]
    fn from_spec_minimal() {
        let spec = FunctionSpec::new("test_func");
        let fid = FunctionId::derive(b"test_func");
        let summary = FunctionSummary::from_spec(&spec, fid);

        assert_eq!(summary.function_id, fid);
        assert_eq!(summary.source, SummarySource::Spec);
        assert_eq!(summary.precision, SummaryPrecision::Sound);
        assert!(!summary.pure);
        assert!(!summary.noreturn);
        assert!(summary.role.is_none());
    }

    #[test]
    fn from_spec_allocator() {
        let mut spec = FunctionSpec::new("malloc");
        spec.role = Some(Role::Allocator);
        spec.params.push({
            let mut p = ParamSpec::new(0);
            p.semantic = Some("allocation_size".to_string());
            p
        });
        spec.returns = Some(ReturnSpec {
            pointer: Some(Pointer::FreshHeap),
            nullness: Some(Nullness::MaybeNull),
            ..ReturnSpec::default()
        });

        let fid = FunctionId::derive(b"malloc");
        let summary = FunctionSummary::from_spec(&spec, fid);

        assert_eq!(summary.role, Some(SummaryRole::Allocator));
        assert_eq!(summary.return_nullness, Some(SummaryNullness::MaybeNull));
        assert_eq!(summary.allocation_effects.len(), 1);
        assert!(summary.allocation_effects[0].heap);
        assert_eq!(summary.allocation_effects[0].target, AccessPath::Return);
        assert_eq!(summary.return_effects.len(), 1);
        assert!(summary.return_effects[0].fresh_allocation);
    }

    #[test]
    fn from_spec_with_params() {
        let mut spec = FunctionSpec::new("memcpy");
        spec.params.push({
            let mut p = ParamSpec::new(0);
            p.modifies = Some(true);
            p.nullness = Some(Nullness::RequiredNonnull);
            p
        });
        spec.params.push({
            let mut p = ParamSpec::new(1);
            p.reads = Some(true);
            p.nullness = Some(Nullness::RequiredNonnull);
            p
        });
        spec.returns = Some(ReturnSpec {
            aliases: Some("param.0".to_string()),
            ..ReturnSpec::default()
        });

        let fid = FunctionId::derive(b"memcpy");
        let summary = FunctionSummary::from_spec(&spec, fid);

        // Memory effects
        assert_eq!(summary.memory_effects.len(), 2);
        // Param 0 writes
        assert!(summary.memory_effects.iter().any(|e| {
            e.path == AccessPath::Deref(Box::new(AccessPath::Param(0))) && e.writes && !e.reads
        }));
        // Param 1 reads
        assert!(summary.memory_effects.iter().any(|e| {
            e.path == AccessPath::Deref(Box::new(AccessPath::Param(1))) && e.reads && !e.writes
        }));
        // Nullness
        assert_eq!(
            summary.param_nullness.get(&0),
            Some(&SummaryNullness::NonNull)
        );
        assert_eq!(
            summary.param_nullness.get(&1),
            Some(&SummaryNullness::NonNull)
        );
        // Return alias
        assert!(
            summary
                .return_effects
                .iter()
                .any(|e| e.aliases == Some(AccessPath::Param(0)) && !e.fresh_allocation)
        );
    }

    #[test]
    fn from_spec_with_taint() {
        let mut spec = FunctionSpec::new("getenv");
        spec.role = Some(Role::Source);
        spec.taint = Some(TaintSpec {
            propagates: vec![SpecTaintPropagation {
                from: TaintLocation::Param(0),
                to: vec![TaintLocation::Return],
            }],
        });

        let fid = FunctionId::derive(b"getenv");
        let summary = FunctionSummary::from_spec(&spec, fid);

        assert_eq!(summary.role, Some(SummaryRole::Source));
        assert_eq!(summary.taint_propagation.len(), 1);
        assert_eq!(summary.taint_propagation[0].from, AccessPath::Param(0));
        assert_eq!(summary.taint_propagation[0].to, AccessPath::Return);
    }

    #[test]
    fn from_spec_callback_becomes_indirect_callee() {
        let mut spec = FunctionSpec::new("qsort");
        spec.params.push({
            let mut p = ParamSpec::new(3);
            p.callback = Some(true);
            p
        });

        let fid = FunctionId::derive(b"qsort");
        let summary = FunctionSummary::from_spec(&spec, fid);

        assert!(
            summary
                .callees
                .contains(&CalleeRef::Indirect(AccessPath::Param(3)))
        );
    }

    // -- to_simple_spec --

    #[test]
    fn to_simple_spec_roundtrip_preserves_key_fields() {
        let mut spec = FunctionSpec::new("malloc");
        spec.role = Some(Role::Allocator);
        spec.pure = Some(true);
        spec.returns = Some(ReturnSpec {
            pointer: Some(Pointer::FreshHeap),
            nullness: Some(Nullness::MaybeNull),
            ..ReturnSpec::default()
        });

        let fid = FunctionId::derive(b"malloc");
        let summary = FunctionSummary::from_spec(&spec, fid);
        let back = summary.to_simple_spec("malloc");

        assert_eq!(back.name, "malloc");
        assert_eq!(back.role, Some(Role::Allocator));
        assert_eq!(back.pure, Some(true));
        let ret = back.returns.as_ref().expect("returns");
        assert_eq!(ret.pointer, Some(Pointer::FreshHeap));
        assert_eq!(ret.nullness, Some(Nullness::MaybeNull));
    }

    #[test]
    fn to_simple_spec_preserves_param_effects() {
        let mut spec = FunctionSpec::new("memcpy");
        spec.params.push({
            let mut p = ParamSpec::new(0);
            p.modifies = Some(true);
            p.nullness = Some(Nullness::NotNull);
            p
        });
        spec.params.push({
            let mut p = ParamSpec::new(1);
            p.reads = Some(true);
            p
        });

        let fid = FunctionId::derive(b"memcpy");
        let summary = FunctionSummary::from_spec(&spec, fid);
        let back = summary.to_simple_spec("memcpy");

        let p0 = back.param(0).expect("param 0");
        assert_eq!(p0.modifies, Some(true));
        assert_eq!(p0.nullness, Some(Nullness::NotNull));

        let p1 = back.param(1).expect("param 1");
        assert_eq!(p1.reads, Some(true));
    }

    #[test]
    fn to_simple_spec_preserves_taint() {
        let mut spec = FunctionSpec::new("getenv");
        spec.taint = Some(TaintSpec {
            propagates: vec![SpecTaintPropagation {
                from: TaintLocation::Param(0),
                to: vec![TaintLocation::Return, TaintLocation::Param(1)],
            }],
        });

        let fid = FunctionId::derive(b"getenv");
        let summary = FunctionSummary::from_spec(&spec, fid);
        let back = summary.to_simple_spec("getenv");

        let taint = back.taint.as_ref().expect("taint");
        assert_eq!(taint.propagates.len(), 1);
        assert_eq!(taint.propagates[0].from, TaintLocation::Param(0));
        assert_eq!(taint.propagates[0].to.len(), 2);
        assert!(taint.propagates[0].to.contains(&TaintLocation::Return));
        assert!(taint.propagates[0].to.contains(&TaintLocation::Param(1)));
    }

    #[test]
    fn to_simple_spec_alias_roundtrip() {
        let mut spec = FunctionSpec::new("realloc");
        spec.returns = Some(ReturnSpec {
            aliases: Some("param.0".to_string()),
            ..ReturnSpec::default()
        });

        let fid = FunctionId::derive(b"realloc");
        let summary = FunctionSummary::from_spec(&spec, fid);
        let back = summary.to_simple_spec("realloc");

        let ret = back.returns.as_ref().expect("returns");
        assert_eq!(ret.aliases, Some("param.0".to_string()));
    }
}
