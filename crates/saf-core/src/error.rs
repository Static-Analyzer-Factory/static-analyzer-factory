//! Core error types for `saf-core` operations.

use thiserror::Error;

/// Errors from `saf-core` operations.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Schema version mismatch during deserialization.
    #[error("schema version mismatch: expected {expected}, found {found}")]
    SchemaMismatch {
        /// The expected schema version.
        expected: String,
        /// The schema version that was found.
        found: String,
    },

    /// Invalid ID format.
    #[error("invalid entity ID: {0}")]
    InvalidId(String),

    /// Spec registry error.
    #[error("spec error: {0}")]
    Spec(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
