//! Structured debug logging system for SAF.
//!
//! Provides the `saf_log!` macro for emitting structured debug output
//! in a DSL format designed for AI coding agents:
//!
//! ```text
//! [module::phase][tag] narrative | key=value key=value
//! ```
//!
//! Controlled at runtime via the `SAF_LOG` environment variable.
//! No output when `SAF_LOG` is unset. Zero recompilation needed.
//!
//! # Usage
//!
//! ```ignore
//! use saf_core::saf_log;
//!
//! // Full form: narrative + key-value pairs (use ; to separate)
//! saf_log!(pta::solve, worklist, "pts grew"; val=node_id, delta=&added);
//!
//! // Narrative only
//! saf_log!(pta::solve, convergence, "fixpoint reached");
//!
//! // Keys only (no narrative)
//! saf_log!(pta::solve, stats; iter=12, worklist=342);
//! ```

pub mod formatter;
pub mod registry;
pub mod value;

#[cfg(feature = "logging-subscriber")]
pub mod subscriber;

pub use value::{PtsDelta, SafLogValue, SafPair, SafRatio};

/// Emit a structured SAF debug log event.
///
/// The macro validates `module::phase` at compile time via the registry,
/// serializes key-value pairs via [`SafLogValue`], and emits a tracing event
/// at `TRACE` level with target `"saf_debug"`.
///
/// # Forms
///
/// ```ignore
/// // Full: narrative + key-values (semicolon separates narrative from keys)
/// saf_log!(module::phase, tag, "narrative"; key=expr, key2=expr2);
///
/// // Narrative only (no key-values)
/// saf_log!(module::phase, tag, "narrative");
///
/// // Keys only (no narrative)
/// saf_log!(module::phase, tag; key=expr, key2=expr2);
/// ```
///
/// The output DSL uses `|` as separator, but the macro uses `;` because
/// Rust macro rules do not allow `|` after `expr` fragments.
#[macro_export]
macro_rules! saf_log {
    // Form 1: narrative + key-value pairs
    ($module:ident :: $phase:ident, $tag:ident, $narrative:expr; $($key:ident = $val:expr),+ $(,)?) => {{
        // Compile-time validation: reference the registry struct
        #[allow(unused, clippy::no_effect)]
        const _: () = {
            fn _validate() {
                let _ = core::mem::size_of::<$crate::__saf_log_registry::$module::$phase>();
            }
        };

        // Only do work if tracing would deliver the event
        if tracing::event_enabled!(target: "saf_debug", tracing::Level::TRACE) {
            use $crate::logging::value::SafLogValue as _;
            let mut _kv_buf = String::new();
            $(
                if !_kv_buf.is_empty() {
                    _kv_buf.push(' ');
                }
                _kv_buf.push_str(concat!(stringify!($key), "="));
                ($val).fmt_saf_log(&mut _kv_buf);
            )+

            tracing::event!(
                target: "saf_debug",
                tracing::Level::TRACE,
                saf_module = stringify!($module),
                saf_phase = stringify!($phase),
                saf_tag = stringify!($tag),
                saf_narrative = $narrative,
                saf_kv = _kv_buf.as_str(),
            );
        }
    }};

    // Form 2: narrative only (no key-value pairs)
    ($module:ident :: $phase:ident, $tag:ident, $narrative:expr) => {{
        #[allow(unused, clippy::no_effect)]
        const _: () = {
            fn _validate() {
                let _ = core::mem::size_of::<$crate::__saf_log_registry::$module::$phase>();
            }
        };

        if tracing::event_enabled!(target: "saf_debug", tracing::Level::TRACE) {
            tracing::event!(
                target: "saf_debug",
                tracing::Level::TRACE,
                saf_module = stringify!($module),
                saf_phase = stringify!($phase),
                saf_tag = stringify!($tag),
                saf_narrative = $narrative,
                saf_kv = "",
            );
        }
    }};

    // Form 3: keys only (no narrative)
    ($module:ident :: $phase:ident, $tag:ident; $($key:ident = $val:expr),+ $(,)?) => {{
        #[allow(unused, clippy::no_effect)]
        const _: () = {
            fn _validate() {
                let _ = core::mem::size_of::<$crate::__saf_log_registry::$module::$phase>();
            }
        };

        if tracing::event_enabled!(target: "saf_debug", tracing::Level::TRACE) {
            use $crate::logging::value::SafLogValue as _;
            let mut _kv_buf = String::new();
            $(
                if !_kv_buf.is_empty() {
                    _kv_buf.push(' ');
                }
                _kv_buf.push_str(concat!(stringify!($key), "="));
                ($val).fmt_saf_log(&mut _kv_buf);
            )+

            tracing::event!(
                target: "saf_debug",
                tracing::Level::TRACE,
                saf_module = stringify!($module),
                saf_phase = stringify!($phase),
                saf_tag = stringify!($tag),
                saf_narrative = "",
                saf_kv = _kv_buf.as_str(),
            );
        }
    }};
}

#[cfg(test)]
mod macro_tests {
    #[test]
    fn test_saf_log_form2() {
        crate::saf_log!(pta::solve, worklist, "test message");
    }

    #[test]
    fn test_saf_log_form1() {
        crate::saf_log!(pta::solve, worklist, "test"; val=42_u32);
    }

    #[test]
    fn test_saf_log_form3() {
        crate::saf_log!(pta::solve, worklist; val=42_u32);
    }
}
