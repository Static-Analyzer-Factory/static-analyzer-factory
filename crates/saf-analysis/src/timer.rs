//! WASM-compatible timer utilities.
//!
//! On native targets, delegates to `std::time::Instant`.
//! On `wasm32`, returns zero elapsed time (timing is not critical to correctness).

use std::time::Duration;

/// A timer that works on both native and `wasm32` targets.
#[derive(Debug, Clone)]
pub(crate) struct Timer {
    #[cfg(not(target_arch = "wasm32"))]
    start: std::time::Instant,
}

impl Timer {
    /// Start a new timer.
    #[inline]
    pub(crate) fn now() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start: std::time::Instant::now(),
        }
    }

    /// Elapsed time as `std::time::Duration`.
    #[inline]
    pub(crate) fn elapsed(&self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.start.elapsed()
        }
        #[cfg(target_arch = "wasm32")]
        {
            Duration::ZERO
        }
    }

    /// Elapsed time in seconds as `f64`.
    #[inline]
    pub(crate) fn elapsed_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }

    /// Elapsed time in milliseconds (saturating to `u64`).
    #[inline]
    pub(crate) fn elapsed_ms(&self) -> u64 {
        // INVARIANT: analysis latency never exceeds u64::MAX milliseconds.
        #[allow(clippy::cast_possible_truncation)]
        let ms = self.elapsed().as_millis() as u64;
        ms
    }
}
