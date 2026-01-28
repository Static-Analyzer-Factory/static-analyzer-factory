//! Source span and symbol types for optional metadata.
//!
//! These types support source-level debugging and error reporting.
//! They are optional in AIR — frontends may or may not provide them.

use serde::{Deserialize, Serialize};

use crate::ids::FileId;

/// Source location span within a file.
///
/// Byte offsets are absolute within the file; line/column are 1-based.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Unique identifier for the source file.
    pub file_id: FileId,

    /// Byte offset of span start (0-based).
    pub byte_start: u32,

    /// Byte offset of span end (exclusive, 0-based).
    pub byte_end: u32,

    /// Line number of span start (1-based).
    pub line_start: u32,

    /// Column number of span start (1-based).
    pub col_start: u32,

    /// Line number of span end (1-based).
    pub line_end: u32,

    /// Column number of span end (1-based).
    pub col_end: u32,
}

impl Span {
    /// Create a new span.
    #[must_use]
    pub const fn new(
        file_id: FileId,
        byte_start: u32,
        byte_end: u32,
        line_start: u32,
        col_start: u32,
        line_end: u32,
        col_end: u32,
    ) -> Self {
        Self {
            file_id,
            byte_start,
            byte_end,
            line_start,
            col_start,
            line_end,
            col_end,
        }
    }

    /// Create a point span (single character).
    #[must_use]
    pub const fn point(file_id: FileId, byte_offset: u32, line: u32, col: u32) -> Self {
        Self {
            file_id,
            byte_start: byte_offset,
            byte_end: byte_offset + 1,
            line_start: line,
            col_start: col,
            line_end: line,
            col_end: col + 1,
        }
    }
}

/// Symbol information for named entities.
///
/// Provides human-readable names and optional mangled/qualified names
/// for debugging and demangling support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Human-readable display name (e.g., `main`, `MyClass::method`).
    pub display_name: String,

    /// Mangled/linkage name if different from display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mangled_name: Option<String>,

    /// Namespace path (e.g., `["std", "collections", "HashMap"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub namespace_path: Vec<String>,
}

impl Symbol {
    /// Create a symbol with just a display name.
    #[must_use]
    pub fn simple(name: impl Into<String>) -> Self {
        Self {
            display_name: name.into(),
            mangled_name: None,
            namespace_path: Vec::new(),
        }
    }

    /// Create a symbol with display and mangled names.
    #[must_use]
    pub fn with_mangled(display: impl Into<String>, mangled: impl Into<String>) -> Self {
        Self {
            display_name: display.into(),
            mangled_name: Some(mangled.into()),
            namespace_path: Vec::new(),
        }
    }

    /// Create a fully qualified symbol.
    #[must_use]
    pub fn qualified(
        display: impl Into<String>,
        mangled: Option<String>,
        namespace: Vec<String>,
    ) -> Self {
        Self {
            display_name: display.into(),
            mangled_name: mangled,
            namespace_path: namespace,
        }
    }
}

/// Source file metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    /// Unique identifier for this file.
    pub id: FileId,

    /// File path (may be absolute or relative).
    pub path: String,

    /// Optional checksum of file contents for verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

impl SourceFile {
    /// Create a source file entry.
    #[must_use]
    pub fn new(id: FileId, path: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            checksum: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_serialization_roundtrip() {
        let span = Span::new(FileId::new(42), 100, 150, 10, 5, 12, 20);
        let json = serde_json::to_string(&span).expect("serialize");
        let parsed: Span = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(span, parsed);
    }

    #[test]
    fn symbol_simple() {
        let sym = Symbol::simple("main");
        assert_eq!(sym.display_name, "main");
        assert!(sym.mangled_name.is_none());
        assert!(sym.namespace_path.is_empty());
    }

    #[test]
    fn symbol_with_mangled_serialization() {
        let sym = Symbol::with_mangled("MyClass::method", "_ZN7MyClass6methodEv");
        let json = serde_json::to_string(&sym).expect("serialize");
        assert!(json.contains("_ZN7MyClass6methodEv"));
        let parsed: Symbol = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(sym, parsed);
    }

    #[test]
    fn symbol_namespace_path() {
        let sym = Symbol::qualified(
            "HashMap",
            None,
            vec!["std".to_string(), "collections".to_string()],
        );
        assert_eq!(sym.namespace_path.len(), 2);
    }
}
