//! Named check catalog — pre-built `AnalysisConfig` entries for common bug patterns.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::config::Severity;

/// A catalog entry describing a named check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Check name (e.g., `"use_after_free"`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// CWE ID (if applicable).
    pub cwe: Option<u32>,
    /// Severity level.
    pub severity: Severity,
    /// Category (e.g., `"memory_safety"`, `"resource_management"`).
    pub category: String,
}

/// The named check catalog.
///
/// Maps check names to their descriptions and metadata.
pub struct CheckCatalog {
    entries: BTreeMap<String, CatalogEntry>,
}

impl CheckCatalog {
    /// Create a new catalog with all built-in checks.
    #[must_use]
    pub fn new() -> Self {
        let mut entries = BTreeMap::new();

        let checks: &[(&str, &str, Option<u32>, Severity, &str)] = &[
            (
                "use_after_free",
                "Detects dereference of freed heap memory",
                Some(416),
                Severity::Error,
                "memory_safety",
            ),
            (
                "double_free",
                "Detects double-free of heap memory",
                Some(415),
                Severity::Error,
                "memory_safety",
            ),
            (
                "null_deref",
                "Detects null pointer dereference",
                Some(476),
                Severity::Error,
                "memory_safety",
            ),
            (
                "memory_leak",
                "Detects unreleased heap allocations",
                Some(401),
                Severity::Warning,
                "resource_management",
            ),
            (
                "file_descriptor_leak",
                "Detects unclosed file descriptors",
                Some(775),
                Severity::Warning,
                "resource_management",
            ),
            (
                "uninit_use",
                "Detects use of uninitialized memory",
                Some(908),
                Severity::Error,
                "memory_safety",
            ),
            (
                "stack_escape",
                "Detects stack pointer escaping function scope",
                Some(562),
                Severity::Error,
                "memory_safety",
            ),
            (
                "lock_not_released",
                "Detects unreleased mutex locks",
                Some(764),
                Severity::Warning,
                "concurrency",
            ),
            (
                "generic_resource_leak",
                "Detects unreleased generic resources",
                Some(772),
                Severity::Warning,
                "resource_management",
            ),
            (
                "buffer_overflow",
                "Detects array access beyond bounds",
                Some(120),
                Severity::Error,
                "memory_safety",
            ),
            (
                "integer_overflow",
                "Detects arithmetic overflow/underflow",
                Some(190),
                Severity::Warning,
                "numeric",
            ),
            (
                "division_by_zero",
                "Detects potential division by zero",
                Some(369),
                Severity::Error,
                "numeric",
            ),
        ];

        for &(name, desc, cwe, severity, category) in checks {
            entries.insert(
                name.to_string(),
                CatalogEntry {
                    name: name.to_string(),
                    description: desc.to_string(),
                    cwe,
                    severity,
                    category: category.to_string(),
                },
            );
        }

        Self { entries }
    }

    /// Get a catalog entry by name.
    pub fn get(&self, name: &str) -> Option<&CatalogEntry> {
        self.entries.get(name)
    }

    /// List all available check names (sorted).
    pub fn list(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// Get all entries.
    pub fn entries(&self) -> &BTreeMap<String, CatalogEntry> {
        &self.entries
    }

    /// Map a catalog check name to the corresponding built-in checker name.
    ///
    /// Some catalog names differ from the internal checker names
    /// (e.g., `"use_after_free"` maps to `"use-after-free"`).
    pub fn to_checker_name(catalog_name: &str) -> Option<&'static str> {
        match catalog_name {
            "use_after_free" | "use-after-free" => Some("use-after-free"),
            "double_free" | "double-free" => Some("double-free"),
            "null_deref" | "null-deref" => Some("null-deref"),
            "memory_leak" | "memory-leak" => Some("memory-leak"),
            "file_descriptor_leak" | "file-descriptor-leak" => Some("file-descriptor-leak"),
            "uninit_use" | "uninit-use" => Some("uninit-use"),
            "stack_escape" | "stack-escape" => Some("stack-escape"),
            "lock_not_released" | "lock-not-released" => Some("lock-not-released"),
            "generic_resource_leak" | "generic-resource-leak" => Some("generic-resource-leak"),
            _ => None,
        }
    }
}

impl Default for CheckCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_known_checks() {
        let catalog = CheckCatalog::new();
        assert!(catalog.get("use_after_free").is_some());
        assert!(catalog.get("null_deref").is_some());
        assert!(catalog.get("memory_leak").is_some());
        assert!(catalog.get("nonexistent").is_none());
    }

    #[test]
    fn catalog_lists_all_checks() {
        let catalog = CheckCatalog::new();
        let names = catalog.list();
        assert!(names.len() >= 9, "should have at least 9 built-in checks");
    }

    #[test]
    fn catalog_entries_serialize_to_json() {
        let catalog = CheckCatalog::new();
        let entry = catalog.get("use_after_free").unwrap();
        let json = serde_json::to_string(entry).unwrap();
        assert!(json.contains("use_after_free"));
    }
}
