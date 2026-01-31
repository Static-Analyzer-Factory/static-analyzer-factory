//! SV-COMP task definition parser.
//!
//! Parses the YAML task definition files used by SV-COMP (format version 2.0+).
//! Each benchmark program has an associated `.yml` file specifying:
//! - Input files (source code)
//! - Properties to verify
//! - Expected verdicts
//! - Compilation options (data model, language)

use anyhow::{Context, Result, bail};
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

/// SV-COMP property types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Property {
    /// `unreach-call`: `reach_error()` is never called.
    UnreachCall,

    /// `valid-memsafety`: No memory safety violations (valid-free, valid-deref, valid-memtrack).
    ValidMemsafety,

    /// `valid-memcleanup`: All memory is freed (no leaks).
    ValidMemcleanup,

    /// `no-overflow`: No signed integer overflows.
    NoOverflow,

    /// `no-data-race`: No data races in concurrent programs.
    NoDataRace,

    /// `termination`: Program terminates.
    Termination,

    /// Coverage property (not a verification property).
    Coverage,

    /// Unknown/unsupported property.
    Unknown,
}

impl Property {
    /// Parse a property from its file path or content.
    pub fn from_property_file(path: &Path) -> Self {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        if filename.contains("unreach-call") {
            Self::UnreachCall
        } else if filename.contains("valid-memsafety") {
            Self::ValidMemsafety
        } else if filename.contains("valid-memcleanup") {
            Self::ValidMemcleanup
        } else if filename.contains("no-overflow") {
            Self::NoOverflow
        } else if filename.contains("no-data-race") {
            Self::NoDataRace
        } else if filename.contains("termination") {
            Self::Termination
        } else if filename.contains("coverage") {
            Self::Coverage
        } else {
            Self::Unknown
        }
    }

    /// Human-readable name of the property.
    pub fn name(&self) -> &'static str {
        match self {
            Self::UnreachCall => "unreach-call",
            Self::ValidMemsafety => "valid-memsafety",
            Self::ValidMemcleanup => "valid-memcleanup",
            Self::NoOverflow => "no-overflow",
            Self::NoDataRace => "no-data-race",
            Self::Termination => "termination",
            Self::Coverage => "coverage",
            Self::Unknown => "unknown",
        }
    }

    /// Returns true if SAF supports this property.
    pub fn is_supported(&self) -> bool {
        matches!(
            self,
            Self::UnreachCall
                | Self::ValidMemsafety
                | Self::ValidMemcleanup
                | Self::NoOverflow
                | Self::NoDataRace
        )
    }
}

/// Programming language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    C,
    Cpp,
}

/// Data model (affects pointer and integer sizes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DataModel {
    /// 32-bit pointers, 32-bit int, 32-bit long
    ILP32,

    /// 64-bit pointers, 32-bit int, 64-bit long
    #[default]
    LP64,
}

impl DataModel {
    /// Returns the clang flag for this data model.
    pub fn clang_flag(&self) -> &'static str {
        match self {
            Self::ILP32 => "-m32",
            Self::LP64 => "-m64",
        }
    }
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
