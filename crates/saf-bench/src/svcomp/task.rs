//! SV-COMP task definition parser.
//!
//! Parses the YAML task definition files used by SV-COMP (format version 2.0+).
//! Each benchmark program has an associated `.yml` file specifying:
//! - Input files (source code)
//! - Properties to verify
//! - Expected verdicts
//! - Compilation options (data model, language)
//!
//! The property/data-model/language enums live in the shared `saf-svcomp` engine
//! crate ([`saf_svcomp::property_kind`]); this module keeps the benchmark-only
//! task model (`SvCompTask`, `PropertySpec` incl. `expected_verdict`) and its
//! YAML parsing.

use anyhow::{Context, Result, bail};
use saf_svcomp::{DataModel, Language, Property};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A parsed SV-COMP task definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvCompTask {
    /// Path to the YAML task definition file.
    pub path: PathBuf,

    /// Path to the compiled bitcode file.
    pub bitcode_path: PathBuf,

    /// Input source files (relative to task directory).
    pub input_files: Vec<PathBuf>,

    /// Properties to verify with expected verdicts.
    pub properties: Vec<PropertySpec>,

    /// Programming language.
    pub language: Language,

    /// Data model (pointer/integer sizes).
    pub data_model: DataModel,

    /// Category (directory name, e.g., "array-examples").
    pub category: String,
}

/// A property specification from the task definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySpec {
    /// Path to the property file (relative).
    pub property_file: PathBuf,

    /// The property type parsed from the property file.
    pub property: Property,

    /// Expected verdict: true = property holds, false = property violated.
    /// None means no verdict expected (coverage properties).
    pub expected_verdict: Option<bool>,

    /// Subproperty for composite properties (e.g., "valid-free" for memsafety).
    pub subproperty: Option<String>,
}

/// Raw YAML structure for deserialization.
#[derive(Debug, Deserialize)]
struct RawTaskDef {
    format_version: Option<String>,
    input_files: InputFiles,
    properties: Option<Vec<RawPropertySpec>>,
    options: Option<RawOptions>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum InputFiles {
    Single(String),
    Multiple(Vec<String>),
}

impl InputFiles {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s],
            Self::Multiple(v) => v,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawPropertySpec {
    property_file: String,
    expected_verdict: Option<bool>,
    subproperty: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawOptions {
    language: Option<String>,
    data_model: Option<String>,
}

impl SvCompTask {
    /// Parse a task definition from a YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_yaml_file(yml_path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(yml_path)
            .with_context(|| format!("Failed to read task file: {}", yml_path.display()))?;

        Self::from_yaml_str(&content, yml_path)
    }

    /// Parse a task definition from YAML content.
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML is invalid or uses unsupported format version.
    pub fn from_yaml_str(content: &str, yml_path: &Path) -> Result<Self> {
        let raw: RawTaskDef = serde_yaml::from_str(content)
            .with_context(|| format!("Failed to parse YAML: {}", yml_path.display()))?;

        // Check format version
        if let Some(version) = &raw.format_version {
            let major: u32 = version
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            if major < 2 {
                bail!("Unsupported task format version {version} (requires 2.0+)");
            }
        }

        let yml_dir = yml_path.parent().unwrap_or(Path::new("."));

        // Parse input files
        let input_files: Vec<PathBuf> = raw
            .input_files
            .into_vec()
            .into_iter()
            .map(PathBuf::from)
            .collect();

        // Parse properties
        let properties: Vec<PropertySpec> = raw
            .properties
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                let property_path = PathBuf::from(&p.property_file);
                PropertySpec {
                    property: Property::from_property_file(&property_path),
                    property_file: property_path,
                    expected_verdict: p.expected_verdict,
                    subproperty: p.subproperty,
                }
            })
            .collect();

        // Parse options
        let language = raw
            .options
            .as_ref()
            .and_then(|o| o.language.as_ref())
            .map_or(Language::C, |l| match l.to_lowercase().as_str() {
                "c++" | "cpp" => Language::Cpp,
                _ => Language::C,
            });

        let data_model = raw
            .options
            .as_ref()
            .and_then(|o| o.data_model.as_ref())
            .map_or(DataModel::LP64, |d| match d.to_uppercase().as_str() {
                "ILP32" => DataModel::ILP32,
                _ => DataModel::LP64,
            });

        // Compute category from directory structure
        // e.g., "sv-benchmarks/c/array-examples/test.yml" -> "array-examples"
        let category = yml_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Compute bitcode path (same directory, .bc extension)
        let base_name = yml_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let bitcode_path = PathBuf::from(format!("{base_name}.bc"));

        Ok(Self {
            path: yml_path.to_path_buf(),
            bitcode_path,
            input_files,
            properties,
            language,
            data_model,
            category,
        })
    }

    /// Get the verification properties (excluding coverage).
    pub fn verification_properties(&self) -> impl Iterator<Item = &PropertySpec> {
        self.properties
            .iter()
            .filter(|p| p.property != Property::Coverage && p.property != Property::Unknown)
    }

    /// Get the primary property (first verification property).
    pub fn primary_property(&self) -> Option<&PropertySpec> {
        self.verification_properties().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_task() {
        let yaml = r"
format_version: '2.0'
input_files: 'test.c'
properties:
  - property_file: ../properties/unreach-call.prp
    expected_verdict: true
options:
  language: C
  data_model: ILP32
";

        let task = SvCompTask::from_yaml_str(yaml, Path::new("/sv-benchmarks/c/test-dir/test.yml"))
            .unwrap();

        assert_eq!(task.input_files.len(), 1);
        assert_eq!(task.input_files[0], PathBuf::from("test.c"));
        assert_eq!(task.language, Language::C);
        assert_eq!(task.data_model, DataModel::ILP32);
        assert_eq!(task.properties.len(), 1);
        assert_eq!(task.properties[0].property, Property::UnreachCall);
        assert_eq!(task.properties[0].expected_verdict, Some(true));
        assert_eq!(task.category, "test-dir");
    }

    #[test]
    fn test_parse_memsafety_task() {
        let yaml = r"
format_version: '2.0'
input_files: 'complex.i'
properties:
  - property_file: ../properties/valid-memsafety.prp
    expected_verdict: false
    subproperty: valid-memtrack
  - property_file: ../properties/coverage-branches.prp
options:
  language: C
  data_model: ILP32
";

        let task = SvCompTask::from_yaml_str(yaml, Path::new("/test.yml")).unwrap();

        assert_eq!(task.properties.len(), 2);
        assert_eq!(task.properties[0].property, Property::ValidMemsafety);
        assert_eq!(task.properties[0].expected_verdict, Some(false));
        assert_eq!(
            task.properties[0].subproperty,
            Some("valid-memtrack".to_string())
        );
        assert_eq!(task.properties[1].property, Property::Coverage);

        // Only one verification property
        assert_eq!(task.verification_properties().count(), 1);
    }

    #[test]
    fn test_property_parsing() {
        assert_eq!(
            Property::from_property_file(Path::new("unreach-call.prp")),
            Property::UnreachCall
        );
        assert_eq!(
            Property::from_property_file(Path::new("valid-memsafety.prp")),
            Property::ValidMemsafety
        );
        assert_eq!(
            Property::from_property_file(Path::new("no-overflow.prp")),
            Property::NoOverflow
        );
        assert_eq!(
            Property::from_property_file(Path::new("coverage-branches.prp")),
            Property::Coverage
        );
    }

    #[test]
    fn test_multiple_input_files() {
        let yaml = r"
format_version: '2.0'
input_files:
  - file1.c
  - file2.c
properties:
  - property_file: ../properties/unreach-call.prp
    expected_verdict: true
";

        let task = SvCompTask::from_yaml_str(yaml, Path::new("/test.yml")).unwrap();
        assert_eq!(task.input_files.len(), 2);
    }
}
