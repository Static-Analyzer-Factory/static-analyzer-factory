//! Checker specification types and built-in checker definitions.
//!
//! Checkers are defined declaratively as data (`CheckerSpec`), not code.
//! AI agents can author custom checkers by specifying sources, sinks,
//! sanitizers, and a reachability mode.

use serde::{Deserialize, Serialize};

use super::resource_table::ResourceRole;

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

/// Finding severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Informational finding.
    Info,
    /// Potential issue that should be reviewed.
    Warning,
    /// Likely bug or vulnerability.
    Error,
    /// Critical security vulnerability.
    Critical,
}

impl Severity {
    /// Get a human-readable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

// ---------------------------------------------------------------------------
// ReachabilityMode
// ---------------------------------------------------------------------------

/// How the checker evaluates reachability on the SVFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReachabilityMode {
    /// Report if source reaches sink on SOME path without sanitizer.
    ///
    /// Used for: UAF, null-deref, stack-escape, uninit-use.
    MayReach,
    /// Report if source does NOT reach sanitizer on ALL paths before exit.
    ///
    /// Used for: memory leak, file-descriptor leak, lock-not-released.
    MustNotReach,
    /// Report if source reaches 2+ distinct sink nodes.
    ///
    /// Used for: double-free (allocation reaching multiple deallocations).
    MultiReach,
    /// Report if source does NOT reach any sink on any path.
    ///
    /// Used for: memory leak (SVF-style — no sink reached = `NEVERFREE`).
    NeverReachSink,
}

// ---------------------------------------------------------------------------
// SitePattern
// ---------------------------------------------------------------------------

/// Pattern for matching SVFG sites (sources, sinks, sanitizers).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SitePattern {
    /// Match calls to functions with a specific resource role.
    Role {
        /// The resource role to match.
        role: ResourceRole,
        /// Whether to match the call's return value (true) or first argument (false).
        /// Allocators use return, deallocators use first arg.
        match_return: bool,
    },
    /// Match a specific function by name.
    FunctionName {
        /// Function name to match.
        name: String,
        /// Whether to match the return value (true) or first argument (false).
        match_return: bool,
    },
    /// Any function return/exit point.
    FunctionExit,
    /// Any use of value flowing from source (matches any SVFG successor).
    AnyUseOf,
    /// Stack allocation (`alloca` instruction).
    AllocaInst,
    /// Load instruction — dereferences the pointer operand.
    LoadDeref,
    /// Store instruction — dereferences the pointer operand.
    StoreDeref,
    /// `GEP` instruction — the base pointer must be valid (dereferenceable).
    /// In SV-COMP semantics, `GEP` on NULL is a `valid-deref` violation.
    GepDeref,
    /// Explicit NULL constant assignment (e.g., `%ptr = copy null`).
    /// Used as a source for null-deref detection.
    NullConstant,
    /// Direct null dereference — instructions where null is literally used as
    /// a pointer operand (Load/Store/GEP) in code NOT guarded by a null check.
    /// Used as a sink for definite null-deref detection.
    DirectNullDeref,
    /// Null-check branch — values guarded by a null check on the not-null path.
    /// Used as a sanitizer for null-deref checker.
    NullCheckBranch,
    /// Custom pattern defined by a predicate function name.
    /// The name is resolved at runtime by the checker runner.
    CustomPredicate {
        /// Predicate identifier string.
        name: String,
    },
}

// ---------------------------------------------------------------------------
// CheckerSpec
// ---------------------------------------------------------------------------

/// Declarative specification of a SVFG-reachability checker.
///
/// Defines what constitutes a source, sink, and sanitizer for a particular
/// bug pattern, plus how reachability is evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckerSpec {
    /// Unique checker name (e.g., "memory-leak", "use-after-free").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// CWE ID (if applicable).
    pub cwe: Option<u32>,
    /// Severity of findings from this checker.
    pub severity: Severity,
    /// How reachability is evaluated.
    pub mode: ReachabilityMode,
    /// Patterns for source sites.
    pub sources: Vec<SitePattern>,
    /// Patterns for sink sites.
    pub sinks: Vec<SitePattern>,
    /// Patterns for sanitizer sites.
    pub sanitizers: Vec<SitePattern>,
}

// ---------------------------------------------------------------------------
// Built-in checker specs
// ---------------------------------------------------------------------------

/// Get the memory leak checker spec.
///
/// Detects heap allocations that are never freed (SVF-style `NEVERFREE`).
/// CWE-401: Missing Release of Memory after Effective Lifetime.
///
/// Uses `NeverReachSink` mode: forward BFS from allocation, check if any
/// deallocation is reached. No deallocation found = leak. This naturally
/// handles dead-end value flows where the pointer is never returned,
/// stored, or freed.
#[must_use]
pub fn memory_leak() -> CheckerSpec {
    CheckerSpec {
        name: "memory-leak".to_string(),
        description: "Heap allocation never freed".to_string(),
        cwe: Some(401),
        severity: Severity::Warning,
        mode: ReachabilityMode::NeverReachSink,
        sources: vec![SitePattern::Role {
            role: ResourceRole::Allocator,
            match_return: true,
        }],
        sinks: vec![SitePattern::Role {
            role: ResourceRole::Deallocator,
            match_return: false,
        }],
        sanitizers: vec![],
    }
}

/// Get the use-after-free checker spec.
///
/// Detects use of a pointer after it has been freed.
/// CWE-416: Use After Free.
///
/// Sinks are explicit pointer dereferences (`Load`/`Store`) rather than
/// `AnyUseOf`, because the source (first arg of `free`) is the same SSA
/// value as the `malloc` return. `AnyUseOf` would collect ALL forward-
/// reachable nodes — including pre-free uses (`strcpy`) and the `free`
/// call itself — producing false positives.
#[must_use]
pub fn use_after_free() -> CheckerSpec {
    CheckerSpec {
        name: "use-after-free".to_string(),
        description: "Pointer used after being freed".to_string(),
        cwe: Some(416),
        severity: Severity::Critical,
        mode: ReachabilityMode::MayReach,
        sources: vec![SitePattern::Role {
            role: ResourceRole::Deallocator,
            match_return: false,
        }],
        sinks: vec![SitePattern::LoadDeref, SitePattern::StoreDeref],
        sanitizers: vec![],
    }
}

/// Get the double-free checker spec.
///
/// Detects double-free of the same allocation.
/// CWE-415: Double Free.
///
/// Uses `MultiReach` mode: source is the allocation (malloc's return),
/// sinks are deallocations (free's argument). Reports when an allocation
/// reaches 2+ distinct free calls, indicating double-free.
#[must_use]
pub fn double_free() -> CheckerSpec {
    CheckerSpec {
        name: "double-free".to_string(),
        description: "Memory freed more than once".to_string(),
        cwe: Some(415),
        severity: Severity::Critical,
        mode: ReachabilityMode::MultiReach,
        sources: vec![SitePattern::Role {
            role: ResourceRole::Allocator,
            match_return: true, // malloc's return value
        }],
        sinks: vec![SitePattern::Role {
            role: ResourceRole::Deallocator,
            match_return: false, // free's argument
        }],
        sanitizers: vec![], // No sanitizer for double-free
    }
}

/// Get the null dereference checker spec.
///
/// Detects dereference of a potentially null pointer.
/// CWE-476: NULL Pointer Dereference.
#[must_use]
pub fn null_deref() -> CheckerSpec {
    CheckerSpec {
        name: "null-deref".to_string(),
        description: "Potentially null pointer dereferenced without check".to_string(),
        cwe: Some(476),
        severity: Severity::Error,
        mode: ReachabilityMode::MayReach,
        sources: vec![
            SitePattern::Role {
                role: ResourceRole::NullSource,
                match_return: true,
            },
            SitePattern::NullConstant,
        ],
        sinks: vec![
            // Function calls that dereference arguments (memcpy, strlen, etc.)
            SitePattern::Role {
                role: ResourceRole::Dereference,
                match_return: false,
            },
            // Direct Load instruction dereferencing a pointer
            SitePattern::LoadDeref,
            // Direct Store instruction dereferencing a pointer
            SitePattern::StoreDeref,
            // GEP instruction — base pointer must be valid
            SitePattern::GepDeref,
        ],
        sanitizers: vec![SitePattern::NullCheckBranch],
    }
}

/// Get the file descriptor leak checker spec.
///
/// Detects file/socket descriptors not closed on all paths.
/// CWE-775: Missing Release of File Descriptor or Handle after Effective Lifetime.
#[must_use]
pub fn file_descriptor_leak() -> CheckerSpec {
    CheckerSpec {
        name: "file-descriptor-leak".to_string(),
        description: "File descriptor or handle not released on all paths".to_string(),
        cwe: Some(775),
        severity: Severity::Warning,
        mode: ReachabilityMode::MustNotReach,
        sources: vec![SitePattern::Role {
            role: ResourceRole::Acquire,
            match_return: true,
        }],
        sinks: vec![SitePattern::FunctionExit],
        sanitizers: vec![SitePattern::Role {
            role: ResourceRole::Release,
            match_return: false,
        }],
    }
}

/// Get the uninitialized use checker spec.
///
/// Detects use of a heap allocation before initialization.
/// CWE-908: Use of Uninitialized Resource.
///
/// Sinks are `LoadDeref` (reads through the pointer) rather than
/// `AnyUseOf`, because the source (allocator return) is the pointer
/// value itself. `AnyUseOf` would include initialization writes and
/// function calls, not just reads of uninitialized data.
/// Sanitizers are `StoreDeref` — writes through the pointer initialize
/// the memory, so any read reachable only through a store is safe.
#[must_use]
pub fn uninit_use() -> CheckerSpec {
    CheckerSpec {
        name: "uninit-use".to_string(),
        description: "Heap-allocated memory used before initialization".to_string(),
        cwe: Some(908),
        severity: Severity::Warning,
        mode: ReachabilityMode::MayReach,
        sources: vec![SitePattern::Role {
            role: ResourceRole::Allocator,
            match_return: true,
        }],
        sinks: vec![SitePattern::LoadDeref],
        sanitizers: vec![SitePattern::StoreDeref],
    }
}

/// Get the stack escape checker spec.
///
/// Detects stack-allocated memory escaping via return or store to heap/global.
/// CWE-562: Return of Stack Variable Address.
#[must_use]
pub fn stack_escape() -> CheckerSpec {
    CheckerSpec {
        name: "stack-escape".to_string(),
        description: "Stack-allocated memory address escapes function scope".to_string(),
        cwe: Some(562),
        severity: Severity::Error,
        mode: ReachabilityMode::MayReach,
        sources: vec![SitePattern::AllocaInst],
        sinks: vec![SitePattern::FunctionExit], // returning a stack address
        sanitizers: vec![],
    }
}

/// Get the lock-not-released checker spec.
///
/// Detects locks not released on all paths.
/// CWE-764: Multiple Locks of a Critical Resource.
/// CWE-765: Multiple Unlocks of a Critical Resource.
#[must_use]
pub fn lock_not_released() -> CheckerSpec {
    CheckerSpec {
        name: "lock-not-released".to_string(),
        description: "Lock not released on all paths to function exit".to_string(),
        cwe: Some(764),
        severity: Severity::Warning,
        mode: ReachabilityMode::MustNotReach,
        sources: vec![SitePattern::Role {
            role: ResourceRole::Lock,
            match_return: false, // lock functions take mutex as arg
        }],
        sinks: vec![SitePattern::FunctionExit],
        sanitizers: vec![SitePattern::Role {
            role: ResourceRole::Unlock,
            match_return: false,
        }],
    }
}

/// Get the generic resource leak checker spec (template for custom resources).
///
/// CWE-772: Missing Release of Resource after Effective Lifetime.
#[must_use]
pub fn generic_resource_leak() -> CheckerSpec {
    CheckerSpec {
        name: "generic-resource-leak".to_string(),
        description: "Resource acquired but not released on all paths".to_string(),
        cwe: Some(772),
        severity: Severity::Warning,
        mode: ReachabilityMode::MustNotReach,
        sources: vec![SitePattern::Role {
            role: ResourceRole::Allocator,
            match_return: true,
        }],
        sinks: vec![SitePattern::FunctionExit],
        sanitizers: vec![SitePattern::Role {
            role: ResourceRole::Deallocator,
            match_return: false,
        }],
    }
}

/// Get all built-in checker specs.
#[must_use]
pub fn builtin_checkers() -> Vec<CheckerSpec> {
    vec![
        memory_leak(),
        use_after_free(),
        double_free(),
        null_deref(),
        file_descriptor_leak(),
        uninit_use(),
        stack_escape(),
        lock_not_released(),
        generic_resource_leak(),
    ]
}

/// Look up a built-in checker by name.
#[must_use]
pub fn builtin_checker(name: &str) -> Option<CheckerSpec> {
    builtin_checkers().into_iter().find(|c| c.name == name)
}

/// Get all built-in checker names.
#[must_use]
pub fn builtin_checker_names() -> Vec<&'static str> {
    vec![
        "memory-leak",
        "use-after-free",
        "double-free",
        "null-deref",
        "file-descriptor-leak",
        "uninit-use",
        "stack-escape",
        "lock-not-released",
        "generic-resource-leak",
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_builtin_checkers_have_names() {
        let checkers = builtin_checkers();
        assert_eq!(checkers.len(), 9);

        for checker in &checkers {
            assert!(!checker.name.is_empty());
            assert!(!checker.description.is_empty());
        }
    }

    #[test]
    fn all_builtin_checkers_have_unique_names() {
        let checkers = builtin_checkers();
        let mut names: Vec<&str> = checkers.iter().map(|c| c.name.as_str()).collect();
        let unique_count = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), unique_count);
    }

    #[test]
    fn all_memory_checkers_have_cwe() {
        let checkers = builtin_checkers();
        for checker in &checkers {
            assert!(
                checker.cwe.is_some(),
                "Checker {} missing CWE",
                checker.name
            );
        }
    }

    #[test]
    fn memory_leak_spec() {
        let spec = memory_leak();
        assert_eq!(spec.name, "memory-leak");
        assert_eq!(spec.cwe, Some(401));
        assert_eq!(spec.mode, ReachabilityMode::NeverReachSink);
        assert_eq!(spec.sources.len(), 1);
        assert_eq!(spec.sinks.len(), 1);
        assert_eq!(spec.sanitizers.len(), 0);
    }

    #[test]
    fn uaf_spec() {
        let spec = use_after_free();
        assert_eq!(spec.name, "use-after-free");
        assert_eq!(spec.cwe, Some(416));
        assert_eq!(spec.severity, Severity::Critical);
        assert_eq!(spec.mode, ReachabilityMode::MayReach);
    }

    #[test]
    fn double_free_spec() {
        let spec = double_free();
        assert_eq!(spec.name, "double-free");
        assert_eq!(spec.cwe, Some(415));
        assert_eq!(spec.mode, ReachabilityMode::MultiReach);
        // Source is allocator return (malloc's return value)
        assert_eq!(spec.sources.len(), 1);
        // Sink is deallocator argument (free's argument)
        assert_eq!(spec.sinks.len(), 1);
    }

    #[test]
    fn null_deref_spec() {
        let spec = null_deref();
        assert_eq!(spec.name, "null-deref");
        assert_eq!(spec.cwe, Some(476));
        assert_eq!(spec.mode, ReachabilityMode::MayReach);
        // Should have 4 sink patterns: Role(Dereference), LoadDeref, StoreDeref, GepDeref
        assert_eq!(spec.sinks.len(), 4);
        // Should have 1 sanitizer pattern: NullCheckBranch
        assert_eq!(spec.sanitizers.len(), 1);
        assert_eq!(spec.sanitizers[0], SitePattern::NullCheckBranch);
    }

    #[test]
    fn file_descriptor_leak_spec() {
        let spec = file_descriptor_leak();
        assert_eq!(spec.name, "file-descriptor-leak");
        assert_eq!(spec.cwe, Some(775));
        assert_eq!(spec.mode, ReachabilityMode::MustNotReach);
    }

    #[test]
    fn lock_not_released_spec() {
        let spec = lock_not_released();
        assert_eq!(spec.name, "lock-not-released");
        assert_eq!(spec.cwe, Some(764));
        assert_eq!(spec.mode, ReachabilityMode::MustNotReach);
    }

    #[test]
    fn builtin_checker_lookup() {
        assert!(builtin_checker("memory-leak").is_some());
        assert!(builtin_checker("use-after-free").is_some());
        assert!(builtin_checker("nonexistent").is_none());
    }

    #[test]
    fn builtin_checker_names_match() {
        let names = builtin_checker_names();
        let checkers = builtin_checkers();

        assert_eq!(names.len(), checkers.len());
        for name in &names {
            assert!(
                builtin_checker(name).is_some(),
                "Name {} not found in built-in checkers",
                name
            );
        }
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn severity_serialization() {
        let severity = Severity::Critical;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"critical\"");

        let parsed: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Severity::Critical);
    }

    #[test]
    fn reachability_mode_serialization() {
        let mode = ReachabilityMode::MayReach;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"may_reach\"");

        let mode2 = ReachabilityMode::MustNotReach;
        let json2 = serde_json::to_string(&mode2).unwrap();
        assert_eq!(json2, "\"must_not_reach\"");

        let mode3 = ReachabilityMode::MultiReach;
        let json3 = serde_json::to_string(&mode3).unwrap();
        assert_eq!(json3, "\"multi_reach\"");

        let mode4 = ReachabilityMode::NeverReachSink;
        let json4 = serde_json::to_string(&mode4).unwrap();
        assert_eq!(json4, "\"never_reach_sink\"");

        let parsed4: ReachabilityMode = serde_json::from_str(&json4).unwrap();
        assert_eq!(parsed4, ReachabilityMode::NeverReachSink);
    }

    #[test]
    fn checker_spec_serialization() {
        let spec = memory_leak();
        let json = serde_json::to_string_pretty(&spec).unwrap();
        let parsed: CheckerSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, spec.name);
        assert_eq!(parsed.cwe, spec.cwe);
        assert_eq!(parsed.mode, spec.mode);
    }

    #[test]
    fn may_reach_checkers() {
        // UAF, null-deref, uninit-use, stack-escape (double-free now uses MultiReach)
        let may_reach: Vec<_> = builtin_checkers()
            .into_iter()
            .filter(|c| c.mode == ReachabilityMode::MayReach)
            .collect();
        assert_eq!(may_reach.len(), 4);
    }

    #[test]
    fn multi_reach_checkers() {
        // double-free
        let multi_reach: Vec<_> = builtin_checkers()
            .into_iter()
            .filter(|c| c.mode == ReachabilityMode::MultiReach)
            .collect();
        assert_eq!(multi_reach.len(), 1);
        assert_eq!(multi_reach[0].name, "double-free");
    }

    #[test]
    fn must_not_reach_checkers() {
        // file-descriptor-leak, lock-not-released, generic-resource-leak
        let must_not: Vec<_> = builtin_checkers()
            .into_iter()
            .filter(|c| c.mode == ReachabilityMode::MustNotReach)
            .collect();
        assert_eq!(must_not.len(), 3);
    }

    #[test]
    fn never_reach_sink_checkers() {
        // memory-leak
        let sts: Vec<_> = builtin_checkers()
            .into_iter()
            .filter(|c| c.mode == ReachabilityMode::NeverReachSink)
            .collect();
        assert_eq!(sts.len(), 1);
        assert_eq!(sts[0].name, "memory-leak");
    }
}
