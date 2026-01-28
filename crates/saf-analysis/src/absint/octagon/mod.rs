//! Octagon abstract domain for relational numeric analysis.
//!
//! The octagon domain tracks constraints of the form `±x ± y <= c` using
//! a Difference-Bound Matrix (DBM) representation. This provides more
//! precision than the interval domain for properties involving relationships
//! between variables.
//!
//! # Use Cases
//!
//! - Loop counter bounds: `i < n` for array access safety
//! - Relational invariants: `x + y <= MAX` for overflow prevention
//! - Range constraints: `low <= x && x <= high`
//!
//! # Complexity
//!
//! - Space: O(n^2) for n variables
//! - Closure (Floyd-Warshall): O(n^3)
//! - Join/Meet/Widen: O(n^2)
//!
//! For very large numbers of variables (> 100), consider using the interval
//! domain or selectively tracking only "interesting" variables in the octagon.
//!
//! # Example
//!
//! ```ignore
//! use saf_analysis::absint::octagon::OctagonDomain;
//! use saf_core::ids::ValueId;
//!
//! let mut oct = OctagonDomain::new();
//!
//! let i = ValueId::new(1);
//! let n = ValueId::new(2);
//!
//! // Set: 0 <= i <= 100, 0 <= n <= 50
//! oct.set_interval(i, 0, 100);
//! oct.set_interval(n, 0, 50);
//!
//! // Add relational constraint: i < n (i.e., i - n <= -1)
//! oct.assume_diff_leq(i, n, -1);
//!
//! // After closure, i's upper bound is refined to 49
//! oct.close();
//! let i_interval = oct.get_interval(i).unwrap();
//! assert!(i_interval.hi() <= 49);
//! ```

pub mod dbm;
pub mod domain;

pub use dbm::{Bound, Dbm, VarIndex};
pub use domain::OctagonDomain;

/// Configuration for octagon analysis.
#[derive(Clone, Debug)]
pub struct OctagonConfig {
    /// Maximum number of variables to track.
    ///
    /// Since octagon operations are O(n^3), tracking too many variables
    /// can become expensive. Variables beyond this limit are handled
    /// by projecting to interval bounds only.
    pub max_vars: usize,

    /// Number of iterations to delay widening.
    ///
    /// Delaying widening allows the analysis to reach a more precise
    /// fixpoint before jumping to infinity.
    pub widening_delay: usize,

    /// Whether to apply strong closure instead of standard closure.
    ///
    /// Strong closure applies additional octagon-specific tightening
    /// but is more expensive.
    pub use_strong_closure: bool,
}

impl Default for OctagonConfig {
    fn default() -> Self {
        Self {
            max_vars: 50,
            widening_delay: 3,
            use_strong_closure: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::absint::domain::AbstractDomain;

    #[test]
    fn test_octagon_config_defaults() {
        let config = OctagonConfig::default();
        assert_eq!(config.max_vars, 50);
        assert_eq!(config.widening_delay, 3);
        assert!(!config.use_strong_closure);
    }

    #[test]
    fn test_octagon_domain_is_abstract_domain() {
        // Verify OctagonDomain implements AbstractDomain
        fn check_abstract_domain<T: AbstractDomain>() {}
        check_abstract_domain::<OctagonDomain>();
    }
}
