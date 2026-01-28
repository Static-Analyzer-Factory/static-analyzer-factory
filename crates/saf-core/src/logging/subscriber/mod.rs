//! Subscriber initialization for SAF structured debug logging.
//!
//! Provides `init()` which reads `SAF_LOG` and `SAF_LOG_FILE` environment
//! variables and returns a `SafLogLayer` for composition into a
//! `tracing_subscriber::Registry`.

pub mod filter;
pub mod layer;

use std::fs::File;

use self::filter::SafLogFilter;
use self::layer::SafLogLayer;

/// Initialize the SAF log layer from environment variables.
///
/// Reads:
/// - `SAF_LOG`: filter specification (if absent, returns a no-op layer)
/// - `SAF_LOG_FILE`: output file path (if absent, writes to stderr)
///
/// Returns a `SafLogLayer` to compose into a tracing subscriber:
///
/// ```ignore
/// use tracing_subscriber::prelude::*;
/// let saf_layer = saf_core::logging::subscriber::init();
/// tracing_subscriber::registry()
///     .with(tracing_subscriber::fmt::layer())
///     .with(saf_layer)
///     .init();
/// ```
///
/// # Panics
///
/// Panics if `SAF_LOG_FILE` is set but the file cannot be created.
pub fn init() -> SafLogLayer {
    let filter = match std::env::var("SAF_LOG") {
        Ok(spec) => SafLogFilter::parse(&spec),
        Err(_) => SafLogFilter::none(),
    };

    let writer: Box<dyn std::io::Write + Send> = match std::env::var("SAF_LOG_FILE") {
        Ok(path) => Box::new(
            File::create(&path)
                .unwrap_or_else(|e| panic!("SAF_LOG_FILE: failed to create {path}: {e}")),
        ),
        Err(_) => Box::new(std::io::stderr()),
    };

    SafLogLayer::new(filter, writer)
}
