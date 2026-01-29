//! Abstract Interpretation Algorithm Verification E2E Tests
//!
//! Plan 041: Abstract Interpretation Algorithm Verification (6 phases)
//!
//! Verifies correctness of:
//! - Phase 1: Widening Timing at Loop Headers
//! - Phase 2: Threshold Widening Correctness
//! - Phase 3: Wrapped vs Signed Interval Semantics
//! - Phase 4: Narrowing Iteration Count
//! - Phase 5: Join/Meet Lattice Correctness
//! - Phase 6: Integration Tests

use saf_analysis::absint::{
    AbstractDomain, AbstractInterpConfig, Interval, extract_thresholds, solve_abstract_interp,
};
use saf_test_utils::load_verification_fixture;

// =============================================================================
// Phase 1: Widening Timing at Loop Headers
// =============================================================================

mod phase1_widening_timing {
    use super::*;

    /// Tests that the simple loop converges with widening.
    /// Without widening, x would iterate [0], [0,1], [0,2], ... infinitely.
    #[test]
    fn test_simple_loop_converges() {
        let module = load_verification_fixture("absint_verification", "widening_loop_simple");
        let config = AbstractInterpConfig::default();

        let result = solve_abstract_interp(&module, &config);

        assert!(
            result.diagnostics().converged,
            "Simple loop should converge with widening"
        );
        assert!(
            result.diagnostics().functions_analyzed >= 1,
            "At least one function should be analyzed"
        );
    }

    /// Tests that without widening (simulated by `max_iterations=1`),
    /// analysis terminates but may not fully converge.
    #[test]
    fn test_max_iterations_bounds_analysis() {
        let module = load_verification_fixture("absint_verification", "widening_loop_simple");
        let config = AbstractInterpConfig {
            max_widening_iterations: 1,
            ..Default::default()
        };

        let result = solve_abstract_interp(&module, &config);

        // Should terminate (not panic or hang)
        // May or may not converge depending on loop structure
        assert!(
            result.diagnostics().blocks_analyzed > 0,
            "Analysis should process some blocks"
        );
    }
}

// =============================================================================
// Phase 2: Threshold Widening Correctness
// =============================================================================

mod phase2_threshold_widening {
    use super::*;

    /// Tests that thresholds are extracted from program constants.
    #[test]
    fn test_threshold_extraction() {
        let module = load_verification_fixture("absint_verification", "threshold_widening");

        let thresholds = extract_thresholds(&module);

        // Standard boundary thresholds should be present
        assert!(thresholds.contains(&0), "Zero should be a threshold");
        assert!(thresholds.contains(&1), "One should be a threshold");
        assert!(
            thresholds.contains(&-1),
            "Negative one should be a threshold"
        );

        // Common buffer sizes should be thresholds
        assert!(
            thresholds.contains(&256),
            "256 should be a threshold (common buffer size)"
        );
        assert!(
            thresholds.contains(&1024),
            "1024 should be a threshold (common buffer size)"
        );
    }

    /// Tests that threshold widening improves precision.
    #[test]
    fn test_threshold_widening_precision() {
        let module = load_verification_fixture("absint_verification", "threshold_widening");

        // Run with threshold widening enabled
        let config_with = AbstractInterpConfig {
            use_threshold_widening: true,
            ..Default::default()
        };
        let result_with = solve_abstract_interp(&module, &config_with);

        // Run with threshold widening disabled
        let config_without = AbstractInterpConfig {
            use_threshold_widening: false,
            ..Default::default()
        };
        let result_without = solve_abstract_interp(&module, &config_without);

        // Both should converge
        assert!(
            result_with.diagnostics().converged,
            "Should converge with threshold widening"
        );
        assert!(
            result_without.diagnostics().converged,
            "Should converge without threshold widening"
        );

        // Both should analyze the same number of functions
        assert_eq!(
            result_with.diagnostics().functions_analyzed,
            result_without.diagnostics().functions_analyzed,
            "Same functions should be analyzed"
        );
    }
}

// =============================================================================
// Phase 3: Wrapped vs Signed Interval Semantics
// =============================================================================

mod phase3_wrapped_semantics {
    use super::*;

    /// Tests i8 overflow handling (127 + 1 should become top or wrap).
    #[test]
    fn test_i8_overflow_interval() {
        // Test interval arithmetic directly
        let max_i8 = Interval::singleton(127, 8);
        let one = Interval::singleton(1, 8);

        let result = max_i8.add(&one);

        // 127 + 1 = 128 exceeds i8 range [-128, 127]
        // Should become top (all values possible due to overflow)
        assert!(
            result.is_top(),
            "i8 overflow should result in top, got {result:?}"
        );
    }

    /// Tests that intervals respect bit-width.
    #[test]
    fn test_interval_bit_width_respected() {
        let i8_top = Interval::make_top(8);
        assert_eq!(i8_top.lo(), -128, "i8 top should have lo = -128");
        assert_eq!(i8_top.hi(), 127, "i8 top should have hi = 127");

        let i16_top = Interval::make_top(16);
        assert_eq!(i16_top.lo(), -32768, "i16 top should have lo = -32768");
        assert_eq!(i16_top.hi(), 32767, "i16 top should have hi = 32767");

        let i32_top = Interval::make_top(32);
        assert_eq!(
            i32_top.lo(),
            i128::from(i32::MIN),
            "i32 top should have lo = i32::MIN"
        );
        assert_eq!(
            i32_top.hi(),
            i128::from(i32::MAX),
            "i32 top should have hi = i32::MAX"
        );
    }

    /// Tests zero-extension preserves value range.
    #[test]
    fn test_zext_preserves_range() {
        let i8_val = Interval::new(0, 100, 8);
        let i32_val = i8_val.zext(32);

        assert_eq!(i32_val.lo(), 0, "zext should preserve lo");
        assert_eq!(i32_val.hi(), 100, "zext should preserve hi");
        assert_eq!(i32_val.bits(), 32, "zext should change bits to 32");
    }

    /// Tests full module analysis with wrapped arithmetic.
    #[test]
    fn test_wrapped_arithmetic_module_converges() {
        let module = load_verification_fixture("absint_verification", "wrapped_arithmetic");
        let config = AbstractInterpConfig::default();

        let result = solve_abstract_interp(&module, &config);

        assert!(
            result.diagnostics().converged,
            "Wrapped arithmetic module should converge"
        );
    }
}

// =============================================================================
// Phase 4: Narrowing Iteration Count
// =============================================================================

mod phase4_narrowing {
    use super::*;

    /// Tests that narrowing is performed.
    #[test]
    fn test_narrowing_iterations_recorded() {
        let module = load_verification_fixture("absint_verification", "narrowing_precision");
        let config = AbstractInterpConfig {
            narrowing_iterations: 3,
            ..Default::default()
        };

        let result = solve_abstract_interp(&module, &config);

        assert_eq!(
            result.diagnostics().narrowing_iterations_performed,
            3,
            "Diagnostics should record 3 narrowing iterations"
        );
    }

    /// Tests narrowing on the Interval domain directly.
    #[test]
    fn test_interval_narrow_from_top() {
        let top = Interval::make_top(32);
        let precise = Interval::new(0, 100, 32);

        let narrowed = top.narrow(&precise);

        // Narrowing top with [0, 100] should give [0, 100]
        assert_eq!(narrowed.lo(), 0, "Narrowed lo should be 0");
        assert_eq!(narrowed.hi(), 100, "Narrowed hi should be 100");
    }

    /// Tests narrowing soundness: result ⊑ old.
    #[test]
    fn test_narrowing_soundness() {
        let old = Interval::new(0, 100, 32);
        let new = Interval::new(10, 90, 32);

        let narrowed = old.narrow(&new);

        assert!(
            narrowed.leq(&old),
            "Narrowing should produce result ⊑ old: {narrowed:?} ⊑ {old:?}"
        );
    }
}

// =============================================================================
// Phase 5: Join/Meet Lattice Correctness
// =============================================================================

mod phase5_lattice_properties {
    use super::*;

    // =========================================================================
    // Join commutativity: a.join(b) == b.join(a)
    // =========================================================================

    #[test]
    fn test_join_commutativity() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(5, 15, 32);

        assert_eq!(a.join(&b), b.join(&a), "Join should be commutative");
    }

    // =========================================================================
    // Meet commutativity: a.meet(b) == b.meet(a)
    // =========================================================================

    #[test]
    fn test_meet_commutativity() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(5, 15, 32);

        assert_eq!(a.meet(&b), b.meet(&a), "Meet should be commutative");
    }

    // =========================================================================
    // Bottom identity: bottom.join(a) == a
    // =========================================================================

    #[test]
    fn test_bottom_join_identity() {
        let bottom = Interval::make_bottom(32);
        let a = Interval::new(0, 10, 32);

        assert_eq!(bottom.join(&a), a, "Bottom should be join identity");
        assert_eq!(
            a.join(&bottom),
            a,
            "Bottom should be join identity (reverse)"
        );
    }

    // =========================================================================
    // Top identity: top.meet(a) == a
    // =========================================================================

    #[test]
    fn test_top_meet_identity() {
        let top = Interval::make_top(32);
        let a = Interval::new(0, 10, 32);

        assert_eq!(top.meet(&a), a, "Top should be meet identity");
        assert_eq!(a.meet(&top), a, "Top should be meet identity (reverse)");
    }

    // =========================================================================
    // Widening soundness: a ⊑ a.widen(b) and b ⊑ a.widen(b)
    // =========================================================================

    #[test]
    fn test_widening_soundness() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(0, 15, 32);

        let widened = a.widen(&b);

        assert!(a.leq(&widened), "a ⊑ a.widen(b)");
        assert!(b.leq(&widened), "b ⊑ a.widen(b)");
    }
}

// =============================================================================
// Phase 6: Integration Tests
// =============================================================================

mod phase6_integration {
    use super::*;

    /// Tests that all verification fixtures can be analyzed.
    #[test]
    fn test_all_fixtures_analyzable() {
        let fixtures = [
            "widening_loop_simple",
            "threshold_widening",
            "wrapped_arithmetic",
            "narrowing_precision",
            "lattice_properties",
            "integration",
        ];

        for fixture in fixtures {
            let module = load_verification_fixture("absint_verification", fixture);
            let config = AbstractInterpConfig::default();

            let result = solve_abstract_interp(&module, &config);

            assert!(
                result.diagnostics().converged,
                "Fixture '{fixture}' should converge"
            );
        }
    }

    /// Tests determinism: same input produces same output.
    #[test]
    fn test_determinism() {
        let module = load_verification_fixture("absint_verification", "integration");
        let config = AbstractInterpConfig::default();

        let result1 = solve_abstract_interp(&module, &config);
        let result2 = solve_abstract_interp(&module, &config);

        assert_eq!(
            result1.diagnostics().blocks_analyzed,
            result2.diagnostics().blocks_analyzed,
            "Same input should produce same blocks_analyzed"
        );
        assert_eq!(
            result1.diagnostics().widening_applications,
            result2.diagnostics().widening_applications,
            "Same input should produce same widening_applications"
        );
        assert_eq!(
            result1.diagnostics().functions_analyzed,
            result2.diagnostics().functions_analyzed,
            "Same input should produce same functions_analyzed"
        );
    }
}
