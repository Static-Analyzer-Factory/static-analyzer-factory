/// Error types for SAF analysis passes.
use thiserror::Error;

/// Errors that can occur during SAF analysis passes.
#[derive(Debug, Error)]
pub enum AnalysisError {
    /// A feature is not yet implemented.
    #[error("not implemented: {0}")]
    NotImplemented(String),

    /// Selector resolution failed (pattern matching for sources/sinks).
    #[error("selector resolution failed: {0}")]
    SelectorResolution(String),

    /// Analysis did not converge within iteration limits.
    #[error("analysis did not converge after {0} iterations")]
    Convergence(usize),

    /// Invalid configuration provided.
    #[error("configuration error: {0}")]
    Config(String),

    /// A required function was not found in the module.
    #[error("missing function: {0}")]
    MissingFunction(String),

    /// A required block was not found in a function.
    #[error("missing block: {0}")]
    MissingBlock(String),

    /// An invalid ID was encountered.
    #[error("invalid ID: {0}")]
    InvalidId(String),
}
