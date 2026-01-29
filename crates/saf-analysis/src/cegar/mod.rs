//! Counter-Example Guided Abstraction Refinement (CEGAR).
//!
//! CEGAR is an iterative verification technique that:
//! 1. Starts with a coarse abstraction
//! 2. Analyzes with the current abstraction
//! 3. If a counterexample is found, checks if it's real
//! 4. If spurious, refines the abstraction and repeats
//! 5. Terminates when property is proven, real bug found, or budget exhausted
//!
//! # Algorithm
//!
//! ```text
//! abstraction := initial_coarse_abstraction()
//! loop:
//!     result := analyze(program, abstraction)
//!     if result == SAFE:
//!         return VERIFIED
//!     else:
//!         counterexample := result.counterexample
//!         if is_feasible(counterexample):
//!             return BUG_FOUND
//!         else:
//!             hints := extract_refinement_hints(counterexample)
//!             abstraction := refine(abstraction, hints)
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use saf_analysis::cegar::{CegarConfig, CegarResult, run_cegar};
//!
//! let config = CegarConfig::default();
//! let result = run_cegar(&module, &property, &config);
//!
//! match result {
//!     CegarResult::Verified => println!("Property holds!"),
//!     CegarResult::Falsified { witness } => println!("Bug found: {witness:?}"),
//!     CegarResult::Unknown { reason } => println!("Could not determine: {reason}"),
//! }
//! ```

pub mod abstraction;
pub mod counterexample;

pub use abstraction::{Abstraction, CompareOp, Predicate, PredicateCondition, RefinementHints};
pub use counterexample::{
    CexCheckConfig, CexCheckResult, Counterexample, PropertyViolation, check_counterexample,
    collect_path_variables,
};

use saf_core::air::AirModule;
use saf_core::ids::BlockId;

/// Configuration for CEGAR loop.
#[derive(Clone, Debug)]
pub struct CegarConfig {
    /// Maximum number of refinement iterations.
    pub max_iterations: usize,

    /// Z3 timeout per counterexample check (milliseconds).
    pub z3_timeout_ms: u64,

    /// Maximum path length for counterexample checking.
    pub max_path_length: usize,

    /// Whether to produce detailed witness on failure.
    pub produce_witness: bool,
}

impl Default for CegarConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            z3_timeout_ms: 5000,
            max_path_length: 1000,
            produce_witness: true,
        }
    }
}

/// Result of CEGAR verification.
#[derive(Clone, Debug)]
pub enum CegarResult {
    /// Property is verified (no violations found).
    Verified {
        /// Final abstraction used.
        abstraction: Abstraction,
        /// Number of refinement iterations performed.
        iterations: usize,
    },

    /// Property is falsified (real bug found).
    Falsified {
        /// Counterexample trace.
        counterexample: Counterexample,
        /// Block path for witness generation.
        witness_path: Vec<BlockId>,
    },

    /// Could not determine (timeout, max iterations reached).
    Unknown {
        /// Reason verification could not complete.
        reason: String,
        /// Number of iterations performed.
        iterations: usize,
        /// Final abstraction reached.
        abstraction: Abstraction,
    },
}

impl CegarResult {
    /// Check if the result is verified.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        matches!(self, Self::Verified { .. })
    }

    /// Check if the result is falsified.
    #[must_use]
    pub const fn is_falsified(&self) -> bool {
        matches!(self, Self::Falsified { .. })
    }

    /// Check if the result is unknown.
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }

    /// Get the number of iterations performed.
    #[must_use]
    pub const fn iterations(&self) -> usize {
        match self {
            Self::Verified { iterations, .. } | Self::Unknown { iterations, .. } => *iterations,
            Self::Falsified { .. } => 0,
        }
    }
}

/// State of the CEGAR loop.
#[derive(Clone, Debug)]
pub struct CegarState {
    /// Current abstraction.
    pub abstraction: Abstraction,
    /// Number of iterations completed.
    pub iteration: usize,
    /// History of abstractions (for cycle detection).
    history: Vec<Abstraction>,
}

impl CegarState {
    /// Create initial CEGAR state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            abstraction: Abstraction::initial(),
            iteration: 0,
            history: vec![Abstraction::initial()],
        }
    }

    /// Refine the abstraction based on hints.
    pub fn refine(&mut self, hints: &RefinementHints) {
        self.abstraction = self.abstraction.refine(hints);
        self.history.push(self.abstraction.clone());
        self.iteration += 1;
    }

    /// Check if we've seen this abstraction before (cycle detection).
    #[must_use]
    pub fn is_cycling(&self) -> bool {
        // Simple check: if abstraction matches any previous (except last)
        self.history[..self.history.len().saturating_sub(1)]
            .iter()
            .any(|h| h == &self.abstraction)
    }

    /// Check if maximum precision has been reached.
    #[must_use]
    pub fn at_max_precision(&self) -> bool {
        self.abstraction.at_max_precision()
    }
}

impl Default for CegarState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single CEGAR iteration.
#[derive(Clone, Debug)]
pub enum IterationResult {
    /// Analysis found no violations with current abstraction.
    Safe,
    /// Analysis found a potential violation.
    PotentialViolation { counterexample: Counterexample },
    /// Analysis failed or timed out.
    AnalysisFailed { reason: String },
}

/// Trait for analyses that can be used with CEGAR.
pub trait CegarAnalysis {
    /// Run analysis with the given abstraction.
    fn analyze(&self, module: &AirModule, abstraction: &Abstraction) -> IterationResult;
}

/// Run CEGAR loop with the given analysis.
pub fn run_cegar<A: CegarAnalysis>(
    analysis: &A,
    module: &AirModule,
    config: &CegarConfig,
) -> CegarResult {
    let mut state = CegarState::new();

    for _ in 0..config.max_iterations {
        // Run analysis with current abstraction
        let result = analysis.analyze(module, &state.abstraction);

        match result {
            IterationResult::Safe => {
                // Property verified with current abstraction
                return CegarResult::Verified {
                    abstraction: state.abstraction,
                    iterations: state.iteration,
                };
            }

            IterationResult::PotentialViolation { counterexample } => {
                // Check if counterexample is real
                let cex_config = CexCheckConfig {
                    z3_timeout_ms: config.z3_timeout_ms,
                    max_path_length: config.max_path_length,
                    extract_refinement_hints: true,
                };

                let check_result = check_counterexample(&counterexample, module, &cex_config);

                match check_result {
                    CexCheckResult::Real => {
                        // Real bug found
                        return CegarResult::Falsified {
                            witness_path: counterexample.blocks.clone(),
                            counterexample,
                        };
                    }

                    CexCheckResult::Spurious { hints } => {
                        // Spurious - refine and continue
                        if !hints.has_refinement() {
                            // No refinement possible
                            return CegarResult::Unknown {
                                reason: "No refinement available for spurious counterexample"
                                    .to_string(),
                                iterations: state.iteration,
                                abstraction: state.abstraction,
                            };
                        }

                        state.refine(&hints);

                        // Check for cycles
                        if state.is_cycling() {
                            return CegarResult::Unknown {
                                reason: "Refinement cycle detected".to_string(),
                                iterations: state.iteration,
                                abstraction: state.abstraction,
                            };
                        }

                        // Check for max precision
                        if state.at_max_precision() {
                            return CegarResult::Unknown {
                                reason: "Maximum precision reached".to_string(),
                                iterations: state.iteration,
                                abstraction: state.abstraction,
                            };
                        }
                    }

                    CexCheckResult::Unknown { reason } => {
                        // Could not determine feasibility
                        return CegarResult::Unknown {
                            reason: format!("Counterexample feasibility unknown: {reason}"),
                            iterations: state.iteration,
                            abstraction: state.abstraction,
                        };
                    }
                }
            }

            IterationResult::AnalysisFailed { reason } => {
                return CegarResult::Unknown {
                    reason: format!("Analysis failed: {reason}"),
                    iterations: state.iteration,
                    abstraction: state.abstraction,
                };
            }
        }
    }

    // Max iterations reached
    CegarResult::Unknown {
        reason: format!("Maximum iterations ({}) reached", config.max_iterations),
        iterations: state.iteration,
        abstraction: state.abstraction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ModuleId;

    struct AlwaysSafeAnalysis;

    impl CegarAnalysis for AlwaysSafeAnalysis {
        fn analyze(&self, _module: &AirModule, _abstraction: &Abstraction) -> IterationResult {
            IterationResult::Safe
        }
    }

    #[allow(dead_code)] // Test utility for future violation-path tests
    struct AlwaysViolationAnalysis;

    impl CegarAnalysis for AlwaysViolationAnalysis {
        fn analyze(&self, _module: &AirModule, _abstraction: &Abstraction) -> IterationResult {
            IterationResult::PotentialViolation {
                counterexample: Counterexample {
                    trace: vec![],
                    blocks: vec![],
                    violation: PropertyViolation::ReachError,
                },
            }
        }
    }

    #[test]
    fn test_cegar_safe() {
        let analysis = AlwaysSafeAnalysis;
        let module = AirModule::new(ModuleId::new(0));
        let config = CegarConfig::default();

        let result = run_cegar(&analysis, &module, &config);

        assert!(result.is_verified());
    }

    #[test]
    fn test_cegar_state_refinement() {
        let mut state = CegarState::new();
        assert_eq!(state.iteration, 0);

        let mut hints = RefinementHints::empty();
        hints.increase_context_sensitivity = true;

        state.refine(&hints);

        assert_eq!(state.iteration, 1);
        assert_eq!(state.abstraction.pta_k, 1);
    }

    #[test]
    fn test_cegar_config_defaults() {
        let config = CegarConfig::default();
        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.z3_timeout_ms, 5000);
    }

    #[test]
    fn test_cegar_result_queries() {
        let verified = CegarResult::Verified {
            abstraction: Abstraction::initial(),
            iterations: 3,
        };
        assert!(verified.is_verified());
        assert!(!verified.is_falsified());
        assert_eq!(verified.iterations(), 3);

        let falsified = CegarResult::Falsified {
            counterexample: Counterexample {
                trace: vec![],
                blocks: vec![],
                violation: PropertyViolation::ReachError,
            },
            witness_path: vec![],
        };
        assert!(falsified.is_falsified());
        assert!(!falsified.is_verified());

        let unknown = CegarResult::Unknown {
            reason: "test".to_string(),
            iterations: 5,
            abstraction: Abstraction::initial(),
        };
        assert!(unknown.is_unknown());
        assert_eq!(unknown.iterations(), 5);
    }
}
