//! IFDS solver configuration.

/// Configuration for the IFDS solver.
#[derive(Debug, Clone)]
pub struct IfdsConfig {
    /// Maximum worklist iterations before aborting (default: `1_000_000`).
    pub max_iterations: usize,
    /// Maximum facts per program point (default: `10_000`).
    pub max_facts_per_point: usize,
}

impl Default for IfdsConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1_000_000,
            max_facts_per_point: 10_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_values() {
        let config = IfdsConfig::default();
        assert_eq!(config.max_iterations, 1_000_000);
        assert_eq!(config.max_facts_per_point, 10_000);
    }
}
