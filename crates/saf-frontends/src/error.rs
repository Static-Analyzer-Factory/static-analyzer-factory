/// Error types for SAF frontends.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrontendError {
    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),
}
