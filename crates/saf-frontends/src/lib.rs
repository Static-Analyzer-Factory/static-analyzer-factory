#[cfg(all(feature = "llvm-18", feature = "llvm-22"))]
compile_error!(
    "saf-frontends: features `llvm-18` and `llvm-22` are mutually exclusive. \
     Both link against `llvm-sys`, and only one LLVM version can be linked at a time. \
     Use `--no-default-features --features llvm-22` to switch."
);

pub mod air_json;
pub mod air_json_schema;
pub mod api;
pub mod error;
pub mod llvm;
