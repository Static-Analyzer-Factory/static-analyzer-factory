//! Spec registry for loading and querying function specifications.
//!
//! The registry loads specs from multiple directories with a defined precedence order:
//! 1. `./saf-specs/*.yaml` — project local (highest priority)
//! 2. `~/.saf/specs/*.yaml` — user global
//! 3. `<binary>/../share/saf/specs/*.yaml` — shipped defaults
//! 4. `$SAF_SPECS_PATH/*.yaml` — explicit override (if set)

use super::pattern::NamePattern;
use super::schema::{SchemaError, SpecFile};
use super::types::FunctionSpec;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, warn};

/// A registry of function specifications.
///
/// Provides lookup by function name with support for exact matches,
/// glob patterns, and regex patterns.
#[derive(Debug, Default)]
pub struct SpecRegistry {
    /// Merged specs indexed by exact name.
    exact_specs: BTreeMap<String, FunctionSpec>,

    /// Pattern-based specs (glob and regex).
    pattern_specs: Vec<(NamePattern, FunctionSpec)>,

    /// Paths that were loaded (for diagnostics).
    loaded_paths: Vec<PathBuf>,

    /// Warnings generated during loading.
    warnings: Vec<String>,
}

impl SpecRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load specs from default discovery paths.
    ///
    /// Discovery order (later overrides earlier per-function):
    /// 1. `<binary>/../share/saf/specs/*.yaml` — shipped defaults
    /// 2. `~/.saf/specs/*.yaml` — user global
    /// 3. `./saf-specs/*.yaml` — project local
    /// 4. `$SAF_SPECS_PATH/*.yaml` — explicit override (if set)
    ///
    /// # Errors
    /// Returns an error if any spec file fails to parse.
    pub fn load() -> Result<Self, RegistryError> {
        let mut registry = Self::new();
        let paths = Self::discovery_paths();

        for path in paths {
            if path.exists() {
                registry.load_directory(&path)?;
            }
        }

        Ok(registry)
    }

    /// Load specs from specific paths only.
    ///
    /// Each path can be a file or directory. Directories are scanned for `*.yaml` files.
    ///
    /// # Errors
    /// Returns an error if any path fails to load.
    pub fn load_from(paths: &[PathBuf]) -> Result<Self, RegistryError> {
        let mut registry = Self::new();

        for path in paths {
            if path.is_dir() {
                registry.load_directory(path)?;
            } else if path.is_file() {
                registry.load_file(path)?;
            } else {
                return Err(RegistryError::PathNotFound(path.display().to_string()));
            }
        }

        Ok(registry)
    }

    /// Load specs from a directory.
    fn load_directory(&mut self, dir: &Path) -> Result<(), RegistryError> {
        let pattern = dir.join("**/*.yaml");
        let pattern_str = pattern.display().to_string();

        let entries = glob::glob(&pattern_str).map_err(|e| RegistryError::GlobError {
            pattern: pattern_str.clone(),
            message: e.to_string(),
        })?;

        // Collect and sort for deterministic ordering
        let mut files: Vec<PathBuf> = entries.filter_map(Result::ok).collect();
        files.sort();

        for file in files {
            self.load_file(&file)?;
        }

        Ok(())
    }

    /// Load specs from a single file.
    fn load_file(&mut self, path: &Path) -> Result<(), RegistryError> {
        debug!("loading spec file: {}", path.display());

        let spec_file = SpecFile::load(path).map_err(RegistryError::Schema)?;

        for spec in spec_file.specs {
            self.insert(spec)?;
        }

        self.loaded_paths.push(path.to_path_buf());
        Ok(())
    }

    /// Insert a spec into the registry.
    ///
    /// For exact names: merges with existing spec (new overrides old).
    /// For patterns: appends to pattern list.
    fn insert(&mut self, spec: FunctionSpec) -> Result<(), RegistryError> {
        let pattern = NamePattern::parse(&spec.name).map_err(|e| RegistryError::PatternError {
            name: spec.name.clone(),
            message: e.to_string(),
        })?;

        if pattern.is_exact() {
            // Check for duplicate and warn
            if self.exact_specs.contains_key(&spec.name) {
                self.warnings.push(format!(
                    "duplicate spec for '{}', later definition overrides",
                    spec.name
                ));
            }

            // Merge with existing or insert
            self.exact_specs
                .entry(spec.name.clone())
                .and_modify(|existing| existing.merge(&spec))
                .or_insert(spec);
        } else {
            // Pattern specs are appended (last one wins during lookup)
            self.pattern_specs.push((pattern, spec));
        }

        Ok(())
    }

    /// Look up a function spec by name.
    ///
    /// Returns the first matching spec:
    /// 1. Exact match has highest priority
    /// 2. Pattern matches are checked in order (last loaded wins)
    ///
    /// Returns `None` for disabled specs.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&FunctionSpec> {
        // Try exact match first
        if let Some(spec) = self.exact_specs.get(name) {
            if spec.is_disabled() {
                return None;
            }
            return Some(spec);
        }

        // Try patterns (reverse order so last loaded wins).
        // A disabled pattern does not suppress lower-priority enabled matches;
        // we `continue` past it so the loop can find an enabled alternative.
        for (pattern, spec) in self.pattern_specs.iter().rev() {
            if pattern.matches(name) {
                if spec.is_disabled() {
                    continue;
                }
                return Some(spec);
            }
        }

        None
    }

    /// Iterate over all exact-match specs.
    pub fn iter(&self) -> impl Iterator<Item = &FunctionSpec> {
        self.exact_specs.values()
    }

    /// Iterate over all pattern-based specs.
    pub fn patterns(&self) -> impl Iterator<Item = &FunctionSpec> {
        self.pattern_specs.iter().map(|(_, s)| s)
    }

    /// Get the number of exact-match specs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.exact_specs.len()
    }

    /// Check if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.exact_specs.is_empty() && self.pattern_specs.is_empty()
    }

    /// Get paths that were loaded.
    #[must_use]
    pub fn loaded_paths(&self) -> &[PathBuf] {
        &self.loaded_paths
    }

    /// Get warnings generated during loading.
    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Get default discovery paths.
    fn discovery_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Shipped defaults (binary/../share/saf/specs)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent().and_then(|p| p.parent()) {
                let share_path = parent.join("share/saf/specs");
                paths.push(share_path);
            }
        }

        // 2. User global (~/.saf/specs)
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home).join(".saf/specs");
            paths.push(home_path);
        }

        // 3. Project local (./saf-specs)
        paths.push(PathBuf::from("./saf-specs"));

        // 4. Workspace share directory (./share/saf/specs)
        // This handles development builds where the exe is in target/release/
        // but specs are in the workspace root's share/ directory.
        paths.push(PathBuf::from("./share/saf/specs"));

        // 5. Explicit override ($SAF_SPECS_PATH)
        if let Ok(specs_path) = std::env::var("SAF_SPECS_PATH") {
            for path in specs_path.split(':') {
                if !path.is_empty() {
                    paths.push(PathBuf::from(path));
                }
            }
        }

        paths
    }

    /// Add a spec directly (for programmatic construction).
    ///
    /// # Errors
    ///
    /// Returns an error if the spec has a duplicate function name.
    pub fn add(&mut self, spec: FunctionSpec) -> Result<(), RegistryError> {
        self.insert(spec)
    }

    /// Create a registry from a YAML string (for testing and programmatic use).
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML is invalid.
    pub fn from_yaml(yaml: &str) -> Result<Self, RegistryError> {
        let file = SpecFile::parse(yaml)?;
        let mut registry = Self::new();
        for spec in file.specs {
            registry.insert(spec)?;
        }
        Ok(registry)
    }

    /// Build a `SpecRegistry` from multiple raw YAML strings.
    ///
    /// Used by the WASM frontend where filesystem access is unavailable.
    /// Each string should be a complete spec file (with `version` and `specs` keys).
    ///
    /// # Errors
    ///
    /// Returns an error if any YAML string fails to parse.
    pub fn from_yaml_strs(yamls: &[String]) -> Result<Self, RegistryError> {
        let mut registry = Self::new();
        for yaml_str in yamls {
            let file = SpecFile::parse(yaml_str)?;
            for spec in file.specs {
                registry.insert(spec)?;
            }
        }
        Ok(registry)
    }

    /// Report missing spec warning.
    pub fn warn_missing(&self, name: &str) {
        warn!("no spec for '{}', using conservative assumptions", name);
    }
}

/// Errors that can occur when loading the registry.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Schema/parsing error.
    #[error("{0}")]
    Schema(#[from] SchemaError),

    /// Path not found.
    #[error("path not found: {0}")]
    PathNotFound(String),

    /// Glob pattern error.
    #[error("glob error for '{pattern}': {message}")]
    GlobError {
        /// Pattern that failed.
        pattern: String,
        /// Error message.
        message: String,
    },

    /// Pattern parsing error.
    #[error("invalid pattern in '{name}': {message}")]
    PatternError {
        /// Spec name with invalid pattern.
        name: String,
        /// Parse error message.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::types::{Nullness, Pointer, ReturnSpec, Role};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_spec_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_empty_registry() {
        let registry = SpecRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.lookup("malloc").is_none());
    }

    #[test]
    fn test_load_single_file() {
        let dir = TempDir::new().unwrap();
        create_spec_file(
            dir.path(),
            "alloc.yaml",
            r#"
version: "1.0"
specs:
  - name: malloc
    role: allocator
    returns:
      pointer: fresh_heap
"#,
        );

        let registry = SpecRegistry::load_from(&[dir.path().to_path_buf()]).unwrap();
        assert_eq!(registry.len(), 1);

        let spec = registry.lookup("malloc").unwrap();
        assert_eq!(spec.role, Some(Role::Allocator));
        assert_eq!(
            spec.returns.as_ref().unwrap().pointer,
            Some(Pointer::FreshHeap)
        );
    }

    #[test]
    fn test_load_multiple_files_merge() {
        let dir = TempDir::new().unwrap();

        // First file: base spec
        create_spec_file(
            dir.path(),
            "a_base.yaml",
            r#"
version: "1.0"
specs:
  - name: malloc
    role: allocator
"#,
        );

        // Second file: adds returns (loaded after a_base due to sort)
        create_spec_file(
            dir.path(),
            "b_extra.yaml",
            r#"
version: "1.0"
specs:
  - name: malloc
    returns:
      nullness: maybe_null
"#,
        );

        let registry = SpecRegistry::load_from(&[dir.path().to_path_buf()]).unwrap();

        let spec = registry.lookup("malloc").unwrap();
        // Role from first file
        assert_eq!(spec.role, Some(Role::Allocator));
        // Returns merged from second file
        assert_eq!(
            spec.returns.as_ref().unwrap().nullness,
            Some(Nullness::MaybeNull)
        );
    }

    #[test]
    fn test_pattern_lookup() {
        let dir = TempDir::new().unwrap();
        create_spec_file(
            dir.path(),
            "string.yaml",
            r#"
version: "1.0"
specs:
  - name: "glob:str*"
    role: string_operation
"#,
        );

        let registry = SpecRegistry::load_from(&[dir.path().to_path_buf()]).unwrap();

        // Pattern should match various str* functions
        let strlen = registry.lookup("strlen").unwrap();
        assert_eq!(strlen.role, Some(Role::StringOperation));

        let strcpy = registry.lookup("strcpy").unwrap();
        assert_eq!(strcpy.role, Some(Role::StringOperation));

        // Non-matching should return None
        assert!(registry.lookup("malloc").is_none());
    }

    #[test]
    fn test_exact_overrides_pattern() {
        let dir = TempDir::new().unwrap();
        create_spec_file(
            dir.path(),
            "specs.yaml",
            r#"
version: "1.0"
specs:
  - name: "glob:str*"
    role: string_operation
  - name: strlen
    pure: true
"#,
        );

        let registry = SpecRegistry::load_from(&[dir.path().to_path_buf()]).unwrap();

        // Exact match should override pattern
        let strlen = registry.lookup("strlen").unwrap();
        assert!(strlen.is_pure());
        // Role comes from exact match (which doesn't have it)
        assert!(strlen.role.is_none());

        // Pattern still works for others
        let strcpy = registry.lookup("strcpy").unwrap();
        assert_eq!(strcpy.role, Some(Role::StringOperation));
    }

    #[test]
    fn test_disabled_spec() {
        let dir = TempDir::new().unwrap();
        create_spec_file(
            dir.path(),
            "specs.yaml",
            r#"
version: "1.0"
specs:
  - name: malloc
    role: allocator
    disabled: true
"#,
        );

        let registry = SpecRegistry::load_from(&[dir.path().to_path_buf()]).unwrap();
        // Disabled spec should not be returned
        assert!(registry.lookup("malloc").is_none());
    }

    #[test]
    fn test_nested_directories() {
        let dir = TempDir::new().unwrap();

        create_spec_file(
            dir.path(),
            "libc/alloc.yaml",
            r#"
version: "1.0"
specs:
  - name: malloc
    role: allocator
"#,
        );

        create_spec_file(
            dir.path(),
            "libc/string.yaml",
            r#"
version: "1.0"
specs:
  - name: strlen
    pure: true
"#,
        );

        let registry = SpecRegistry::load_from(&[dir.path().to_path_buf()]).unwrap();
        assert!(registry.lookup("malloc").is_some());
        assert!(registry.lookup("strlen").is_some());
    }

    #[test]
    fn test_add_programmatic() {
        let mut registry = SpecRegistry::new();

        let mut spec = FunctionSpec::new("my_alloc");
        spec.role = Some(Role::Allocator);
        spec.returns = Some(ReturnSpec {
            pointer: Some(Pointer::FreshHeap),
            ..ReturnSpec::default()
        });

        registry.add(spec).unwrap();

        let found = registry.lookup("my_alloc").unwrap();
        assert_eq!(found.role, Some(Role::Allocator));
    }

    #[test]
    fn test_invalid_file() {
        let dir = TempDir::new().unwrap();
        create_spec_file(dir.path(), "bad.yaml", "not: valid: yaml:");

        let result = SpecRegistry::load_from(&[dir.path().to_path_buf()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_not_found() {
        let result = SpecRegistry::load_from(&[PathBuf::from("/nonexistent/path")]);
        assert!(matches!(
            result.unwrap_err(),
            RegistryError::PathNotFound(_)
        ));
    }
}
