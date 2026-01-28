//! Frontend API trait — the architectural cornerstone for extensibility
//! (NFR-EXT-001, NFR-EXT-002, FR-FE-002).
//!
//! All frontends implement this trait. The analysis engine operates only
//! on the AIR produced by frontends, never on frontend-specific types.

use std::collections::BTreeMap;
use std::path::Path;

use saf_core::air::AirBundle;
use saf_core::config::Config;

use crate::error::FrontendError;

/// Common contract that all SAF frontends must implement.
///
/// A frontend converts an input program representation (e.g., LLVM bitcode,
/// AIR JSON) into SAF's canonical AIR plus metadata for caching and schema
/// discoverability.
pub trait Frontend {
    /// Ingest inputs and produce an AIR bundle.
    ///
    /// The `inputs` slice contains paths to input files (e.g., `.bc`, `.air.json`).
    ///
    /// # Errors
    ///
    /// Returns `FrontendError` if ingestion fails (e.g., invalid input, I/O error).
    fn ingest(&self, inputs: &[&Path], config: &Config) -> Result<AirBundle, FrontendError>;

    /// Compute a deterministic fingerprint of the inputs.
    ///
    /// Used for cache key derivation (FR-FE-003). Must be path-normalized
    /// and must not include debug info by default (NFR-DET-004).
    ///
    /// # Errors
    ///
    /// Returns `FrontendError` if fingerprinting fails (e.g., I/O error).
    fn input_fingerprint_bytes(
        &self,
        inputs: &[&Path],
        config: &Config,
    ) -> Result<Vec<u8>, FrontendError>;

    /// Report which features this frontend supports for schema discoverability.
    fn supported_features(&self) -> BTreeMap<String, bool>;

    /// A stable string identifier for this frontend (e.g., `"llvm"`, `"air-json"`).
    fn frontend_id(&self) -> &'static str;

    /// Ingest multiple input files, returning a separate [`AirBundle`] per file.
    ///
    /// Uses `cache` (if provided) to skip re-ingestion for files whose
    /// fingerprint matches a cached bundle. Falls back to per-file `ingest()`.
    ///
    /// # Errors
    ///
    /// Returns [`FrontendError`] if any individual file fails to ingest.
    fn ingest_multi(
        &self,
        inputs: &[&Path],
        config: &Config,
        cache: Option<&saf_core::cache::BundleCache>,
    ) -> Result<Vec<AirBundle>, FrontendError> {
        let mut bundles = Vec::with_capacity(inputs.len());
        for input in inputs {
            // Check cache
            if let Some(cache) = cache {
                if let Ok(fp) = self.input_fingerprint_bytes(&[*input], config) {
                    if let Some(bundle) = cache.get(&fp) {
                        bundles.push(bundle);
                        continue;
                    }
                }
            }

            let bundle = self.ingest(&[*input], config)?;

            // Store in cache
            if let Some(cache) = cache {
                if let Ok(fp) = self.input_fingerprint_bytes(&[*input], config) {
                    let _ = cache.put(&fp, &bundle);
                }
            }

            bundles.push(bundle);
        }
        Ok(bundles)
    }
}
