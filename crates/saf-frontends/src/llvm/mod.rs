//! LLVM Bitcode frontend — ingests `.bc` (and optionally `.ll`) files (FR-FE-004).
//!
//! This is the MVP frontend for production use, converting LLVM IR to AIR
//! for analysis by SAF's engine.
//!
//! # Feature Flags
//!
//! - `llvm-17`: Enable LLVM 17 support
//! - `llvm-18`: Enable LLVM 18 support (default)
//!
//! At least one LLVM version feature must be enabled to use this frontend.

#[macro_use]
mod adapter;
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
pub(crate) mod cha_extract;
mod debug_info;
mod error;
mod intrinsics;
#[cfg(feature = "llvm-17")]
mod llvm17;
#[cfg(feature = "llvm-18")]
mod llvm18;
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
mod mapping;
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
mod type_intern;

pub use error::LlvmError;
pub use intrinsics::{IntrinsicMapping, IntrinsicOp, classify_intrinsic};

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use saf_core::air::AirBundle;
use saf_core::config::Config;

use crate::api::Frontend;
use crate::error::FrontendError;

/// Frontend that ingests LLVM bitcode (`.bc`) and LLVM IR (`.ll`) files.
///
/// Converts LLVM IR to AIR (Analysis Intermediate Representation) for
/// analysis by SAF's engine.
///
/// # Example
///
/// ```ignore
/// use saf_frontends::llvm::LlvmFrontend;
/// use saf_frontends::api::Frontend;
/// use saf_core::config::Config;
/// use std::path::Path;
///
/// let frontend = LlvmFrontend::new();
/// let inputs = [Path::new("program.bc")];
/// let config = Config::default();
/// let bundle = frontend.ingest(&inputs, &config)?;
/// ```
pub struct LlvmFrontend {
    #[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
    adapter: Box<dyn adapter::LlvmAdapter>,
}

impl LlvmFrontend {
    /// Create a new LLVM frontend with the default adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            #[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
            adapter: adapter::create_adapter(),
        }
    }

    /// Get the LLVM version this frontend is using.
    #[must_use]
    #[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
    pub fn llvm_version(&self) -> &'static str {
        self.adapter.version()
    }

    /// Get the LLVM version this frontend is using.
    #[must_use]
    #[cfg(not(any(feature = "llvm-17", feature = "llvm-18")))]
    pub fn llvm_version(&self) -> &'static str {
        "none"
    }
}

impl Default for LlvmFrontend {
    fn default() -> Self {
        Self::new()
    }
}

impl Frontend for LlvmFrontend {
    #[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
    fn ingest(&self, inputs: &[&Path], _config: &Config) -> Result<AirBundle, FrontendError> {
        use blake3::Hasher;

        if inputs.is_empty() {
            return Err(FrontendError::Parse("no input files provided".to_string()));
        }

        // For now, support single input file
        // Multi-file linking will be added later
        if inputs.len() > 1 {
            return Err(FrontendError::Parse(
                "multiple input files not yet supported".to_string(),
            ));
        }

        let path = inputs[0];
        let path_str = path.display().to_string();

        // Read the file
        let bytes = fs::read(path).map_err(|e| LlvmError::file_read(&path_str, e))?;

        // Determine if it's bitcode or text IR
        let is_bitcode =
            bytes.starts_with(b"BC\xc0\xde") || bytes.starts_with(&[0xde, 0xc0, 0x17, 0x0b]);

        // Create LLVM context and parse
        let context = self.adapter.create_context();

        let module = if is_bitcode {
            self.adapter.parse_bitcode(&context, &bytes, &path_str)?
        } else {
            // Assume text IR
            let ir_text = std::str::from_utf8(&bytes)
                .map_err(|e| LlvmError::parse(format!("invalid UTF-8 in IR file: {e}")))?;
            self.adapter.parse_ir(&context, ir_text, &path_str)?
        };

        // For fingerprinting, we need the raw bytes
        // For .ll files, hash the text content
        let fingerprint_bytes = if is_bitcode {
            bytes.clone()
        } else {
            // Hash the normalized IR text
            let mut hasher = Hasher::new();
            hasher.update(&bytes);
            hasher.finalize().as_bytes().to_vec()
        };

        // Convert to AIR
        mapping::convert_module(&module, &fingerprint_bytes, self.adapter.as_ref())
            .map_err(FrontendError::from)
    }

    #[cfg(not(any(feature = "llvm-17", feature = "llvm-18")))]
    fn ingest(&self, _inputs: &[&Path], _config: &Config) -> Result<AirBundle, FrontendError> {
        Err(FrontendError::NotImplemented(
            "LLVM frontend requires llvm-17 or llvm-18 feature to be enabled".to_string(),
        ))
    }

    fn input_fingerprint_bytes(
        &self,
        inputs: &[&Path],
        _config: &Config,
    ) -> Result<Vec<u8>, FrontendError> {
        use blake3::Hasher;

        let mut hasher = Hasher::new();
        hasher.update(b"llvm-frontend-v1");

        for path in inputs {
            let bytes = fs::read(path).map_err(|e| {
                FrontendError::Io(std::io::Error::new(
                    e.kind(),
                    format!("failed to read {}: {e}", path.display()),
                ))
            })?;
            hasher.update(&bytes);
        }

        Ok(hasher.finalize().as_bytes().to_vec())
    }

    fn supported_features(&self) -> BTreeMap<String, bool> {
        let mut features = BTreeMap::new();
        features.insert("llvm-17".to_string(), cfg!(feature = "llvm-17"));
        features.insert("llvm-18".to_string(), cfg!(feature = "llvm-18"));
        features.insert("bitcode".to_string(), true);
        features.insert("text-ir".to_string(), true);
        features.insert("debug-info".to_string(), true);
        features
    }

    fn frontend_id(&self) -> &'static str {
        "llvm"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_id_is_llvm() {
        let frontend = LlvmFrontend::new();
        assert_eq!(frontend.frontend_id(), "llvm");
    }

    #[test]
    fn supported_features_includes_version() {
        let frontend = LlvmFrontend::new();
        let features = frontend.supported_features();

        // At least one LLVM version should be enabled in tests
        #[cfg(feature = "llvm-17")]
        assert!(features.get("llvm-17") == Some(&true));

        #[cfg(feature = "llvm-18")]
        assert!(features.get("llvm-18") == Some(&true));
    }

    #[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
    #[test]
    fn llvm_version_matches_feature() {
        let frontend = LlvmFrontend::new();
        let version = frontend.llvm_version();

        #[cfg(feature = "llvm-18")]
        assert!(version.starts_with("18"));

        #[cfg(all(feature = "llvm-17", not(feature = "llvm-18")))]
        assert!(version.starts_with("17"));
    }
}
