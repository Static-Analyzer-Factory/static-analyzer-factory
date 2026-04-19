#[cfg(all(feature = "llvm-18", feature = "llvm-22"))]
compile_error!(
    "saf-frontends: features `llvm-18` and `llvm-22` are mutually exclusive. \
     Both link against `llvm-sys`, and only one LLVM version can be linked at a time. \
     Use `--no-default-features --features llvm-22` to switch."
);

/// LLVM major.minor version this build links against. Surfaced through
/// `saf --version` and `saf schema` so users can tell at a glance which
/// image they have, and diagnose IR parse failures that trace back to a
/// clang / SAF LLVM-version mismatch.
#[cfg(all(feature = "llvm-18", not(feature = "llvm-22")))]
pub const LLVM_VERSION: &str = "18.1";
/// LLVM version (see doc above).
#[cfg(all(feature = "llvm-22", not(feature = "llvm-18")))]
pub const LLVM_VERSION: &str = "22.1";
/// LLVM version — stub when no LLVM feature is enabled.
#[cfg(not(any(feature = "llvm-18", feature = "llvm-22")))]
pub const LLVM_VERSION: &str = "none";

pub mod air_json;
pub mod air_json_schema;
pub mod api;
pub mod error;
pub mod llvm;
