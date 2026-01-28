//! LLVM frontend-specific error types.

use thiserror::Error;

/// Errors that can occur during LLVM bitcode ingestion.
#[derive(Debug, Error)]
pub enum LlvmError {
    /// Failed to read the input file.
    #[error("failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse LLVM bitcode or IR.
    #[error("failed to parse LLVM IR: {0}")]
    Parse(String),
}

impl LlvmError {
    /// Create a file read error.
    pub fn file_read(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::FileRead {
            path: path.into(),
            source,
        }
    }

    /// Create a parse error.
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }
}

impl From<LlvmError> for crate::error::FrontendError {
    fn from(err: LlvmError) -> Self {
        match err {
            LlvmError::FileRead { source, .. } => crate::error::FrontendError::Io(source),
            LlvmError::Parse(msg) => crate::error::FrontendError::Parse(msg),
        }
    }
}
