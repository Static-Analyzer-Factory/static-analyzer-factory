//! Module/phase registry for compile-time validation of `saf_log!` call sites.
//!
//! The `saf_log_module!` macro generates a hidden module hierarchy that
//! `saf_log!` references to produce compile errors on typos.

/// Declares SAF log modules and their phases for compile-time validation.
///
/// Each module/phase pair becomes a unit struct in a hidden `__saf_log_registry`
/// module. The `saf_log!` macro references these structs so that invalid
/// `module::phase` combinations produce compile errors.
#[macro_export]
macro_rules! saf_log_module {
    ($($module:ident { $($phase:ident),* $(,)? }),* $(,)?) => {
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code, non_snake_case)]
        pub mod __saf_log_registry {
            $(
                pub mod $module {
                    $(pub struct $phase;)*
                }
            )*
        }
    };
}

// The actual registry is declared in lib.rs (crate root) so that
// $crate::__saf_log_registry resolves correctly in cross-crate usage.
