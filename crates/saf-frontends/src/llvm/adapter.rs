//! Version-agnostic LLVM adapter trait.
//!
//! This trait abstracts over LLVM version differences, allowing the frontend
//! to work with LLVM 17, 18, or future versions through a common interface.

#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
use inkwell::context::Context;
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
use inkwell::module::Module;

#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
use super::error::LlvmError;

/// Version-agnostic interface to LLVM operations.
///
/// Each supported LLVM version implements this trait with version-specific
/// handling where needed.
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
#[allow(dead_code)] // Trait contract: methods implemented by version-specific adapters via impl_llvm_adapter! macro
pub trait LlvmAdapter: Send + Sync {
    /// Get the LLVM version string (e.g., "17.0", "18.0").
    fn version(&self) -> &'static str;

    /// Create a new LLVM context.
    fn create_context(&self) -> Context;

    /// Parse LLVM bitcode from bytes.
    fn parse_bitcode<'ctx>(
        &self,
        context: &'ctx Context,
        bytes: &[u8],
        name: &str,
    ) -> Result<Module<'ctx>, LlvmError>;

    /// Parse LLVM IR text.
    fn parse_ir<'ctx>(
        &self,
        context: &'ctx Context,
        ir: &str,
        name: &str,
    ) -> Result<Module<'ctx>, LlvmError>;
}

/// Stub adapter trait when no LLVM feature is enabled.
#[cfg(not(any(feature = "llvm-17", feature = "llvm-18")))]
#[allow(dead_code)] // Stub trait for compilation when no LLVM feature is enabled
pub trait LlvmAdapter: Send + Sync {
    /// Get the LLVM version string.
    fn version(&self) -> &'static str;
}

/// Generate an `LlvmAdapter` implementation for a specific LLVM version.
///
/// All LLVM versions share the same implementation — only the struct name
/// and version string differ. This macro eliminates the duplication.
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
macro_rules! impl_llvm_adapter {
    ($name:ident, $version:expr) => {
        /// Adapter for a specific LLVM version.
        pub struct $name;

        impl $crate::llvm::adapter::LlvmAdapter for $name {
            fn version(&self) -> &'static str {
                $version
            }

            fn create_context(&self) -> inkwell::context::Context {
                inkwell::context::Context::create()
            }

            fn parse_bitcode<'ctx>(
                &self,
                context: &'ctx inkwell::context::Context,
                bytes: &[u8],
                name: &str,
            ) -> Result<inkwell::module::Module<'ctx>, $crate::llvm::LlvmError> {
                let buffer =
                    inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(bytes, name);
                inkwell::module::Module::parse_bitcode_from_buffer(&buffer, context)
                    .map_err(|e| $crate::llvm::LlvmError::parse(e.to_string()))
            }

            fn parse_ir<'ctx>(
                &self,
                context: &'ctx inkwell::context::Context,
                ir: &str,
                name: &str,
            ) -> Result<inkwell::module::Module<'ctx>, $crate::llvm::LlvmError> {
                // Use create_from_memory_range_copy to ensure null termination —
                // LLVM's IR parser may read past the buffer without it.
                let buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range_copy(
                    ir.as_bytes(),
                    name,
                );
                context
                    .create_module_from_ir(buffer)
                    .map_err(|e| $crate::llvm::LlvmError::parse(e.to_string()))
            }
        }
    };
}

/// Create the appropriate LLVM adapter for the enabled feature.
#[cfg(feature = "llvm-18")]
pub fn create_adapter() -> Box<dyn LlvmAdapter> {
    Box::new(super::llvm18::Llvm18Adapter)
}

#[cfg(all(feature = "llvm-17", not(feature = "llvm-18")))]
pub fn create_adapter() -> Box<dyn LlvmAdapter> {
    Box::new(super::llvm17::Llvm17Adapter)
}

#[cfg(not(any(feature = "llvm-17", feature = "llvm-18")))]
#[allow(dead_code)] // Stub for compilation when no LLVM feature is enabled
pub fn create_adapter() -> Box<dyn LlvmAdapter> {
    panic!("LLVM frontend requires either llvm-17 or llvm-18 feature to be enabled")
}
