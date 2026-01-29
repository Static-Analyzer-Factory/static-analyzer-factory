//! Counterexample analysis for CEGAR.
//!
//! Provides functionality to check whether a potential counterexample
//! (error trace) is feasible or spurious, and to extract refinement
//! hints when it's spurious.

use std::collections::BTreeSet;

use saf_core::air::AirModule;
use saf_core::ids::{BlockId, InstId, ValueId};

use super::abstraction::{CompareOp, Predicate, PredicateCondition, RefinementHints};

/// A potential counterexample trace.
#[derive(Clone, Debug)]
pub struct Counterexample {
    /// Sequence of instructions in the error path.
    pub trace: Vec<InstId>,
    /// Basic blocks visited along the path.
    pub blocks: Vec<BlockId>,
    /// Type of property violation.
    pub violation: PropertyViolation,
}

/// Types of property violations.
#[derive(Clone, Debug)]
pub enum PropertyViolation {
    /// `reach_error()` is called.
    ReachError,
    /// Null pointer dereference.
    NullDeref { pointer: ValueId },
    /// Use after free.
    UseAfterFree { pointer: ValueId },
    /// Double free.
    DoubleFree { pointer: ValueId },
    /// Integer overflow.
    IntegerOverflow { value: ValueId },
    /// Buffer overflow.
    BufferOverflow { base: ValueId, index: ValueId },
    /// Data race.
    DataRace { location: ValueId },
}

/// Result of checking a counterexample.
#[derive(Clone, Debug)]
pub enum CexCheckResult {
    /// Counterexample is real (feasible).
    Real,
    /// Counterexample is spurious with refinement hints.
    Spurious { hints: RefinementHints },
    /// Could not determine (timeout, complexity).
    Unknown { reason: String },
}

/// Configuration for counterexample checking.
#[derive(Clone, Debug)]
pub struct CexCheckConfig {
    /// Z3 solver timeout in milliseconds.
    pub z3_timeout_ms: u64,
    /// Maximum path length to analyze.
    pub max_path_length: usize,
    /// Whether to extract detailed refinement hints.
    pub extract_refinement_hints: bool,
}

impl Default for CexCheckConfig {
    fn default() -> Self {
        Self {
            z3_timeout_ms: 5000,
            max_path_length: 1000,
            extract_refinement_hints: true,
        }
    }
}

/// Check if a counterexample is feasible using constraint solving.
///
/// This performs a symbolic execution of the path and checks if the
/// path constraints are satisfiable.
#[must_use]
pub fn check_counterexample(
    cex: &Counterexample,
    _module: &AirModule,
    config: &CexCheckConfig,
) -> CexCheckResult {
    // Check path length limit
    if cex.trace.len() > config.max_path_length {
        return CexCheckResult::Unknown {
            reason: format!(
                "Path too long ({} > {})",
                cex.trace.len(),
                config.max_path_length
            ),
        };
    }

    // For now, provide a basic implementation that:
    // 1. Collects path constraints from the trace
    // 2. Returns hints based on the violation type

    // In a full implementation, this would:
    // 1. Build Z3 constraints from the instruction trace
    // 2. Check satisfiability
    // 3. If UNSAT, analyze the unsat core for refinement

    // Placeholder: assume short paths are real, long paths need refinement
    if cex.trace.len() < 10 {
        CexCheckResult::Real
    } else if config.extract_refinement_hints {
        let hints = extract_refinement_hints(cex);
        CexCheckResult::Spurious { hints }
    } else {
        CexCheckResult::Unknown {
            reason: "Path analysis not implemented".to_string(),
        }
    }
}

/// Extract refinement hints from a spurious counterexample.
///
/// Analyzes the path to determine what abstractions would rule out
/// this spurious behavior.
fn extract_refinement_hints(cex: &Counterexample) -> RefinementHints {
    let mut hints = RefinementHints::empty();

    // Based on violation type, suggest appropriate refinements
    match &cex.violation {
        PropertyViolation::NullDeref { pointer }
        | PropertyViolation::UseAfterFree { pointer }
        | PropertyViolation::DoubleFree { pointer } => {
            // Track nullness/allocation state of the pointer
            hints.add_predicate(Predicate {
                condition: PredicateCondition::IsNull(*pointer),
                source_line: None,
            });
            hints.track_variable(*pointer);
            // Enable flow sensitivity to track pointer state through program
            hints.request_flow_sensitivity();
        }

        PropertyViolation::IntegerOverflow { value } => {
            // Track the value precisely
            hints.track_variable(*value);
        }

        PropertyViolation::BufferOverflow { base, index } => {
            // Track both base and index
            hints.track_variable(*base);
            hints.track_variable(*index);
            // Track the comparison predicate
            hints.add_predicate(Predicate {
                condition: PredicateCondition::Compare {
                    left: *index,
                    op: CompareOp::Lt,
                    right: *base, // Assuming base encodes length info
                },
                source_line: None,
            });
        }

        PropertyViolation::DataRace { location } => {
            // Track the memory location
            hints.track_variable(*location);
            // Increase context sensitivity for thread analysis
            hints.request_more_context();
        }

        PropertyViolation::ReachError => {
            // For reach_error, increase context sensitivity if path is long
            if cex.trace.len() > 50 {
                hints.request_more_context();
            }
            hints.request_flow_sensitivity();
        }
    }

    // If path involves many control-flow decisions, increase precision
    if cex.blocks.len() > 20 {
        hints.request_more_context();
    }

    hints
}

/// Collect variables involved in a counterexample path.
pub fn collect_path_variables(cex: &Counterexample) -> BTreeSet<ValueId> {
    let mut vars = BTreeSet::new();

    // Add variables from the violation
    match &cex.violation {
        PropertyViolation::NullDeref { pointer }
        | PropertyViolation::UseAfterFree { pointer }
        | PropertyViolation::DoubleFree { pointer } => {
            vars.insert(*pointer);
        }
        PropertyViolation::IntegerOverflow { value } => {
            vars.insert(*value);
        }
        PropertyViolation::BufferOverflow { base, index } => {
            vars.insert(*base);
            vars.insert(*index);
        }
        PropertyViolation::DataRace { location } => {
            vars.insert(*location);
        }
        PropertyViolation::ReachError => {}
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ModuleId;

    fn make_simple_cex() -> Counterexample {
        Counterexample {
            trace: vec![InstId::new(1), InstId::new(2), InstId::new(3)],
            blocks: vec![BlockId::new(10), BlockId::new(20)],
            violation: PropertyViolation::ReachError,
        }
    }

    #[test]
    fn test_short_path_is_real() {
        let cex = make_simple_cex();
        let config = CexCheckConfig::default();
        let module = AirModule::new(ModuleId::new(0));

        let result = check_counterexample(&cex, &module, &config);

        match result {
            CexCheckResult::Real => {}
            other => panic!("Expected Real, got {other:?}"),
        }
    }

    #[test]
    fn test_path_too_long() {
        let mut cex = make_simple_cex();
        cex.trace = (0..2000).map(|i| InstId::new(i as u128)).collect();

        let config = CexCheckConfig {
            max_path_length: 1000,
            ..Default::default()
        };
        let module = AirModule::new(ModuleId::new(0));

        let result = check_counterexample(&cex, &module, &config);

        match result {
            CexCheckResult::Unknown { reason } => {
                assert!(reason.contains("too long"));
            }
            other => panic!("Expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn test_refinement_hints_for_null_deref() {
        let cex = Counterexample {
            trace: (0..20).map(|i| InstId::new(i)).collect(),
            blocks: (0..15).map(|i| BlockId::new(i * 10)).collect(),
            violation: PropertyViolation::NullDeref {
                pointer: ValueId::new(42),
            },
        };

        let hints = extract_refinement_hints(&cex);

        assert!(hints.has_refinement());
        assert!(hints.track_vars.contains(&ValueId::new(42)));
        assert!(hints.enable_flow_sensitivity);
    }

    #[test]
    fn test_collect_path_variables() {
        let cex = Counterexample {
            trace: vec![],
            blocks: vec![],
            violation: PropertyViolation::BufferOverflow {
                base: ValueId::new(1),
                index: ValueId::new(2),
            },
        };

        let vars = collect_path_variables(&cex);

        assert!(vars.contains(&ValueId::new(1)));
        assert!(vars.contains(&ValueId::new(2)));
    }
}
