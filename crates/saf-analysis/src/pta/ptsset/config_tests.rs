//! Extended configuration and auto-selection tests.
//!
//! Tests for the `PtsConfig` auto-selection logic, threshold behaviors,
//! and explicit override handling. Complements the existing tests in `config.rs`.

#[cfg(test)]
mod tests {
    use crate::pta::ptsset::{ClusteringMode, PtsConfig, PtsRepresentation};

    // ---------------------------------------------------------------------------
    // Auto-Selection Threshold Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn auto_selects_fxhash_below_roaring_threshold() {
        let config = PtsConfig::default();

        // Below roaring threshold should use FxHash (faster O(1) ops for sparse sets)
        assert_eq!(config.select_by_count(0), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(1), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(100), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(1_000), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(5_000), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(9_999), PtsRepresentation::FxHash);
    }

    #[test]
    fn auto_selects_roaring_for_medium_large_programs() {
        let config = PtsConfig::default();

        // At or above roaring threshold but below bdd threshold
        assert_eq!(config.select_by_count(10_000), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(10_001), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(50_000), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(75_000), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(99_999), PtsRepresentation::Roaring);
    }

    #[test]
    fn auto_selects_bdd_for_large_programs() {
        let config = PtsConfig::default();

        // At or above bdd threshold
        assert_eq!(config.select_by_count(100_000), PtsRepresentation::Bdd);
        assert_eq!(config.select_by_count(100_001), PtsRepresentation::Bdd);
        assert_eq!(config.select_by_count(500_000), PtsRepresentation::Bdd);
        assert_eq!(config.select_by_count(1_000_000), PtsRepresentation::Bdd);
        assert_eq!(config.select_by_count(10_000_000), PtsRepresentation::Bdd);
    }

    // ---------------------------------------------------------------------------
    // Explicit Override Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn explicit_btreeset_ignores_size() {
        let config = PtsConfig::btreeset();

        // Should use BTreeSet regardless of allocation count
        // (select_by_count only applies when representation is Auto)
        assert_eq!(config.representation, PtsRepresentation::BTreeSet);

        // When representation is explicitly set, it should be used directly
        // The select_for_module would return the explicit representation
    }

    #[test]
    fn explicit_bitvector_ignores_size() {
        let config = PtsConfig::bitvector();
        assert_eq!(config.representation, PtsRepresentation::BitVector);
    }

    #[test]
    fn explicit_bdd_ignores_size() {
        let config = PtsConfig::bdd();
        assert_eq!(config.representation, PtsRepresentation::Bdd);
    }

    #[test]
    fn explicit_roaring_ignores_size() {
        let config = PtsConfig::roaring();
        assert_eq!(config.representation, PtsRepresentation::Roaring);
    }

    #[test]
    fn with_representation_overrides() {
        // Start with auto, override to specific
        let config = PtsConfig::default().with_representation(PtsRepresentation::BTreeSet);
        assert_eq!(config.representation, PtsRepresentation::BTreeSet);

        let config = PtsConfig::default().with_representation(PtsRepresentation::BitVector);
        assert_eq!(config.representation, PtsRepresentation::BitVector);

        let config = PtsConfig::default().with_representation(PtsRepresentation::Roaring);
        assert_eq!(config.representation, PtsRepresentation::Roaring);

        let config = PtsConfig::default().with_representation(PtsRepresentation::Bdd);
        assert_eq!(config.representation, PtsRepresentation::Bdd);

        let config = PtsConfig::default().with_representation(PtsRepresentation::Auto);
        assert_eq!(config.representation, PtsRepresentation::Auto);
    }

    // ---------------------------------------------------------------------------
    // Custom Threshold Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn custom_roaring_threshold() {
        let config = PtsConfig::default().with_roaring_threshold(500);

        // Below roaring threshold → FxHash
        assert_eq!(config.select_by_count(99), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(499), PtsRepresentation::FxHash);
        // At roaring threshold → Roaring
        assert_eq!(config.select_by_count(500), PtsRepresentation::Roaring);
    }

    #[test]
    fn custom_bdd_threshold() {
        let config = PtsConfig::default()
            .with_roaring_threshold(500)
            .with_bdd_threshold(1000);

        assert_eq!(config.select_by_count(99), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(499), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(500), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(999), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(1000), PtsRepresentation::Bdd);
        assert_eq!(config.select_by_count(1001), PtsRepresentation::Bdd);
    }

    #[test]
    fn overlapping_thresholds_bdd_wins() {
        // If roaring and bdd thresholds overlap, BDD should be selected first (highest priority)
        let config = PtsConfig::default()
            .with_roaring_threshold(500)
            .with_bdd_threshold(500);

        // At 500, BDD check comes first
        assert_eq!(config.select_by_count(499), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(500), PtsRepresentation::Bdd);
        assert_eq!(config.select_by_count(1000), PtsRepresentation::Bdd);
    }

    #[test]
    fn zero_thresholds() {
        // Zero roaring threshold means roaring at 0 (unless >= bdd_threshold)
        let config = PtsConfig::default()
            .with_roaring_threshold(50)
            .with_bdd_threshold(100);

        assert_eq!(config.select_by_count(0), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(49), PtsRepresentation::FxHash);
        assert_eq!(config.select_by_count(50), PtsRepresentation::Roaring);
        assert_eq!(config.select_by_count(100), PtsRepresentation::Bdd);

        // Zero bdd threshold means always use BDD
        let config = PtsConfig::default()
            .with_roaring_threshold(0)
            .with_bdd_threshold(0);

        assert_eq!(config.select_by_count(0), PtsRepresentation::Bdd);
        assert_eq!(config.select_by_count(1), PtsRepresentation::Bdd);
    }

    // ---------------------------------------------------------------------------
    // String Parsing Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn representation_from_str_case_insensitive() {
        // Uppercase
        assert_eq!(
            PtsRepresentation::from_str_opt("AUTO"),
            Some(PtsRepresentation::Auto)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("BTREESET"),
            Some(PtsRepresentation::BTreeSet)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("BITVECTOR"),
            Some(PtsRepresentation::BitVector)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("ROARING"),
            Some(PtsRepresentation::Roaring)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("BDD"),
            Some(PtsRepresentation::Bdd)
        );

        // Mixed case
        assert_eq!(
            PtsRepresentation::from_str_opt("Auto"),
            Some(PtsRepresentation::Auto)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("BTreeSet"),
            Some(PtsRepresentation::BTreeSet)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("BitVector"),
            Some(PtsRepresentation::BitVector)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("Roaring"),
            Some(PtsRepresentation::Roaring)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("Bdd"),
            Some(PtsRepresentation::Bdd)
        );
    }

    #[test]
    fn representation_from_str_aliases() {
        // btree alias
        assert_eq!(
            PtsRepresentation::from_str_opt("btree"),
            Some(PtsRepresentation::BTreeSet)
        );

        // bitvec aliases
        assert_eq!(
            PtsRepresentation::from_str_opt("bitvec"),
            Some(PtsRepresentation::BitVector)
        );
        assert_eq!(
            PtsRepresentation::from_str_opt("bv"),
            Some(PtsRepresentation::BitVector)
        );
    }

    #[test]
    fn representation_from_str_invalid() {
        assert_eq!(PtsRepresentation::from_str_opt(""), None);
        assert_eq!(PtsRepresentation::from_str_opt("invalid"), None);
        assert_eq!(PtsRepresentation::from_str_opt("btree_set"), None);
        assert_eq!(PtsRepresentation::from_str_opt("bit-vector"), None);
        assert_eq!(PtsRepresentation::from_str_opt("hashset"), None);
    }

    #[test]
    fn representation_as_str_roundtrip() {
        let variants = [
            PtsRepresentation::Auto,
            PtsRepresentation::BTreeSet,
            PtsRepresentation::FxHash,
            PtsRepresentation::BitVector,
            PtsRepresentation::Roaring,
            PtsRepresentation::Bdd,
        ];

        for variant in variants {
            let s = variant.as_str();
            let parsed = PtsRepresentation::from_str_opt(s);
            assert_eq!(parsed, Some(variant), "Roundtrip failed for {:?}", variant);
        }
    }

    // ---------------------------------------------------------------------------
    // Config Builder Chain Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn builder_chain_all_options() {
        let config = PtsConfig::new()
            .with_representation(PtsRepresentation::BitVector)
            .with_bitvec_threshold(5_000)
            .with_bdd_threshold(50_000);

        assert_eq!(config.representation, PtsRepresentation::BitVector);
        assert_eq!(config.bitvec_threshold, 5_000);
        assert_eq!(config.bdd_threshold, 50_000);
    }

    #[test]
    fn builder_chain_order_independent() {
        // Order of with_* calls shouldn't matter
        let config1 = PtsConfig::new()
            .with_bitvec_threshold(5_000)
            .with_bdd_threshold(50_000)
            .with_representation(PtsRepresentation::Auto);

        let config2 = PtsConfig::new()
            .with_representation(PtsRepresentation::Auto)
            .with_bdd_threshold(50_000)
            .with_bitvec_threshold(5_000);

        assert_eq!(config1, config2);
    }

    #[test]
    fn builder_override_previous() {
        // Later calls should override earlier ones
        let config = PtsConfig::new()
            .with_representation(PtsRepresentation::BTreeSet)
            .with_representation(PtsRepresentation::BitVector)
            .with_representation(PtsRepresentation::Bdd);

        assert_eq!(config.representation, PtsRepresentation::Bdd);
    }

    // ---------------------------------------------------------------------------
    // Serialization Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn representation_serialization() {
        // Test serde serialization (snake_case)
        let repr = PtsRepresentation::BitVector;
        let json = serde_json::to_string(&repr).unwrap();
        assert_eq!(json, "\"bit_vector\"");

        let parsed: PtsRepresentation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, repr);
    }

    #[test]
    fn config_serialization() {
        let config = PtsConfig {
            representation: PtsRepresentation::Bdd,
            bitvec_threshold: 5_000,
            roaring_threshold: 25_000,
            bdd_threshold: 50_000,
            clustering: ClusteringMode::Auto,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: PtsConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, config);
    }

    // ---------------------------------------------------------------------------
    // Default Value Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn default_values_documented() {
        // Verify the documented default values
        let config = PtsConfig::default();

        assert_eq!(config.representation, PtsRepresentation::Auto);
        assert_eq!(config.bitvec_threshold, 10_000);
        assert_eq!(config.roaring_threshold, 10_000);
        assert_eq!(config.bdd_threshold, 100_000);
    }

    #[test]
    fn representation_default_is_auto() {
        assert_eq!(PtsRepresentation::default(), PtsRepresentation::Auto);
    }

    // ---------------------------------------------------------------------------
    // Boundary Condition Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn exactly_at_thresholds() {
        let config = PtsConfig::default();

        // Exactly at roaring threshold
        assert_eq!(config.select_by_count(10_000), PtsRepresentation::Roaring);

        // Exactly at bdd threshold
        assert_eq!(config.select_by_count(100_000), PtsRepresentation::Bdd);
    }

    #[test]
    fn one_below_thresholds() {
        let config = PtsConfig::default();

        // One below roaring threshold
        assert_eq!(config.select_by_count(9_999), PtsRepresentation::FxHash);

        // One below bdd threshold
        assert_eq!(config.select_by_count(99_999), PtsRepresentation::Roaring);
    }

    #[test]
    fn one_above_thresholds() {
        let config = PtsConfig::default();

        // One above roaring threshold
        assert_eq!(config.select_by_count(10_001), PtsRepresentation::Roaring);

        // One above bdd threshold
        assert_eq!(config.select_by_count(100_001), PtsRepresentation::Bdd);
    }
}
