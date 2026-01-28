//! YAML schema parsing and validation for function specifications.
//!
//! Handles loading spec files with version validation and actionable error messages.

use super::types::FunctionSpec;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// The current spec file format version.
pub const CURRENT_VERSION: &str = "1.0";

/// A spec file containing multiple function specifications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecFile {
    /// Schema version (must be "1.0").
    pub version: String,

    /// List of function specifications.
    #[serde(default)]
    pub specs: Vec<FunctionSpec>,
}

impl SpecFile {
    /// Create a new empty spec file with current version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: CURRENT_VERSION.to_string(),
            specs: Vec::new(),
        }
    }

    /// Parse a spec file from YAML string.
    ///
    /// # Errors
    /// Returns an error if YAML parsing fails or version is unsupported.
    pub fn parse(content: &str) -> Result<Self, SchemaError> {
        let file: Self =
            serde_yaml::from_str(content).map_err(|e| SchemaError::Yaml(e.to_string()))?;

        // Validate version
        if file.version != CURRENT_VERSION {
            return Err(SchemaError::UnsupportedVersion {
                found: file.version,
                expected: CURRENT_VERSION.to_string(),
            });
        }

        // Validate each spec
        for (i, spec) in file.specs.iter().enumerate() {
            validate_spec(spec, i)?;
        }

        Ok(file)
    }

    /// Load a spec file from a path.
    ///
    /// # Errors
    /// Returns an error if file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, SchemaError> {
        let content = std::fs::read_to_string(path).map_err(|e| SchemaError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;

        Self::parse(&content).map_err(|e| match e {
            SchemaError::Yaml(msg) => SchemaError::ParseError {
                path: path.display().to_string(),
                message: msg,
            },
            other => other,
        })
    }
}

impl Default for SpecFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a function specification.
fn validate_spec(spec: &FunctionSpec, index: usize) -> Result<(), SchemaError> {
    // Name is required and must not be empty
    if spec.name.is_empty() {
        return Err(SchemaError::Validation {
            field: format!("specs[{index}].name"),
            message: "function name cannot be empty".to_string(),
        });
    }

    // Validate param indices are unique
    let mut seen_indices = std::collections::BTreeSet::new();
    for param in &spec.params {
        if !seen_indices.insert(param.index) {
            return Err(SchemaError::Validation {
                field: format!("specs[{index}].params"),
                message: format!("duplicate parameter index {}", param.index),
            });
        }
    }

    // Validate alias references
    if let Some(returns) = &spec.returns {
        if let Some(alias) = &returns.aliases {
            if !alias.starts_with("param.") {
                return Err(SchemaError::Validation {
                    field: format!("specs[{index}].returns.aliases"),
                    message: format!("invalid alias format '{alias}', expected 'param.N'"),
                });
            }
        }
    }

    // Validate taint propagation references
    if let Some(taint) = &spec.taint {
        for (j, prop) in taint.propagates.iter().enumerate() {
            validate_taint_location(prop.from, index, j, "from")?;
            for to in &prop.to {
                validate_taint_location(*to, index, j, "to")?;
            }
        }
    }

    Ok(())
}

/// Validate a [`TaintLocation`] is not `Unknown`.
///
/// Returns an error if the location is [`TaintLocation::Unknown`], which indicates
/// an unrecognized string was used in the YAML spec.
fn validate_taint_location(
    location: super::TaintLocation,
    spec_idx: usize,
    prop_idx: usize,
    field: &str,
) -> Result<(), SchemaError> {
    if matches!(location, super::TaintLocation::Unknown) {
        return Err(SchemaError::Validation {
            field: format!("specs[{spec_idx}].taint.propagates[{prop_idx}].{field}"),
            message: "invalid taint location, expected 'param.N' or 'return'".to_string(),
        });
    }
    Ok(())
}

/// Errors that can occur when parsing spec files.
#[derive(Debug, Error)]
pub enum SchemaError {
    /// YAML parsing error.
    #[error("YAML parse error: {0}")]
    Yaml(String),

    /// File I/O error.
    #[error("cannot read '{path}': {message}")]
    Io {
        /// Path that failed to read.
        path: String,
        /// I/O error message.
        message: String,
    },

    /// Parse error with file context.
    #[error("error in '{path}': {message}")]
    ParseError {
        /// Path of the file.
        path: String,
        /// Parse error message.
        message: String,
    },

    /// Unsupported spec version.
    #[error("unsupported version '{found}', expected '{expected}'")]
    UnsupportedVersion {
        /// Version found in file.
        found: String,
        /// Expected version.
        expected: String,
    },

    /// Validation error.
    #[error("{field}: {message}")]
    Validation {
        /// Field that failed validation.
        field: String,
        /// Validation error message.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_spec() {
        let yaml = r#"
version: "1.0"
specs:
  - name: malloc
    role: allocator
"#;
        let file = SpecFile::parse(yaml).unwrap();
        assert_eq!(file.version, "1.0");
        assert_eq!(file.specs.len(), 1);
        assert_eq!(file.specs[0].name, "malloc");
    }

    #[test]
    fn test_parse_full_spec() {
        let yaml = r#"
version: "1.0"
specs:
  - name: strcpy
    params:
      - index: 0
        name: dst
        modifies: true
        nullness: required_nonnull
      - index: 1
        name: src
        reads: true
        nullness: required_nonnull
    returns:
      aliases: param.0
      nullness: not_null
    taint:
      propagates:
        - from: param.1
          to: [param.0, return]
"#;
        let file = SpecFile::parse(yaml).unwrap();
        assert_eq!(file.specs.len(), 1);
        let spec = &file.specs[0];
        assert_eq!(spec.name, "strcpy");
        assert_eq!(spec.params.len(), 2);
        assert!(spec.params[0].modifies.unwrap());
        assert_eq!(
            spec.returns.as_ref().unwrap().aliases,
            Some("param.0".to_string())
        );
    }

    #[test]
    fn test_parse_empty_specs() {
        let yaml = r#"
version: "1.0"
specs: []
"#;
        let file = SpecFile::parse(yaml).unwrap();
        assert!(file.specs.is_empty());
    }

    #[test]
    fn test_unsupported_version() {
        let yaml = r#"
version: "2.0"
specs: []
"#;
        let err = SpecFile::parse(yaml).unwrap_err();
        assert!(matches!(err, SchemaError::UnsupportedVersion { .. }));
    }

    #[test]
    fn test_empty_name_error() {
        let yaml = r#"
version: "1.0"
specs:
  - name: ""
"#;
        let err = SpecFile::parse(yaml).unwrap_err();
        assert!(matches!(err, SchemaError::Validation { .. }));
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_duplicate_param_index_error() {
        let yaml = r#"
version: "1.0"
specs:
  - name: foo
    params:
      - index: 0
      - index: 0
"#;
        let err = SpecFile::parse(yaml).unwrap_err();
        assert!(matches!(err, SchemaError::Validation { .. }));
        assert!(err.to_string().contains("duplicate parameter index"));
    }

    #[test]
    fn test_invalid_alias_error() {
        let yaml = r#"
version: "1.0"
specs:
  - name: foo
    returns:
      aliases: invalid
"#;
        let err = SpecFile::parse(yaml).unwrap_err();
        assert!(matches!(err, SchemaError::Validation { .. }));
        assert!(err.to_string().contains("invalid alias format"));
    }

    #[test]
    fn test_invalid_taint_ref_error() {
        let yaml = r#"
version: "1.0"
specs:
  - name: foo
    taint:
      propagates:
        - from: invalid
          to: [return]
"#;
        let err = SpecFile::parse(yaml).unwrap_err();
        assert!(matches!(err, SchemaError::Validation { .. }));
        assert!(err.to_string().contains("invalid taint location"));
    }

    #[test]
    fn test_yaml_parse_error() {
        let yaml = "this is not: valid: yaml: :";
        let err = SpecFile::parse(yaml).unwrap_err();
        assert!(matches!(err, SchemaError::Yaml(_)));
    }
}
