# Phase 0 + Phase 1: Multi-Module Infrastructure — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable SAF to ingest multiple `.ll`/`.bc`/`.air.json` files, link them into a unified `AirProgram`, and cache per-module AIR bundles — the foundation for all incremental analysis.

**Architecture:** Introduce `AirProgram` (a whole-program container of linked `AirModule`s) in `saf-core`, extend both frontends to support multi-file ingestion with per-file caching via `BundleCache`, and provide a `merged_view` flattened `AirModule` so all existing analysis passes work unchanged. Add `ProgramDiff` for change detection and single-file auto-split by source metadata.

**Tech Stack:** Rust, serde/serde_json, blake3, `BundleCache` (already implemented in `saf-core::cache`)

**Design doc:** `plans/2026-02-25-incremental-analysis-design.md` (Sections 3-4)

---

## Task 1: Add `ProgramId` to the ID system

**Files:**
- Modify: `crates/saf-core/src/ids.rs`

**Step 1: Add the new ID type**

After the `FileId` definition (~line 167), add:

```rust
define_id_type!(
    /// Unique identifier for a whole program (set of linked modules).
    ProgramId,
    "program"
);
```

**Step 2: Run tests to verify no breakage**

Run: `cargo test -p saf-core` (safe to run locally — no LLVM dependency)
Expected: All existing tests pass, new `ProgramId` type is available.

**Step 3: Commit**

```bash
git add crates/saf-core/src/ids.rs
git commit -m "feat(core): add ProgramId type for multi-module programs"
```

---

## Task 2: Define `LinkTable` and `AirProgram` types

**Files:**
- Create: `crates/saf-core/src/program.rs`
- Modify: `crates/saf-core/src/lib.rs` (add `pub mod program;`)

**Step 1: Write tests for the new types**

Create `crates/saf-core/src/program.rs` with types AND inline tests at the bottom:

```rust
//! Multi-module program linking.
//!
//! `AirProgram` represents a whole program composed of multiple linked
//! `AirModule`s. The `LinkTable` resolves cross-module references
//! (extern declarations matched to definitions).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::air::{AirBundle, AirFunction, AirGlobal, AirModule};
use crate::id::make_id;
use crate::ids::{FunctionId, ModuleId, ProgramId, ValueId};

/// Cross-module symbol resolution.
///
/// Maps extern declarations in one module to their definitions in another.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LinkTable {
    /// Extern function declaration `FunctionId` -> defining function's `FunctionId`.
    pub function_resolutions: BTreeMap<FunctionId, FunctionId>,

    /// Extern global `ValueId` -> defining global's `ValueId`.
    pub global_resolutions: BTreeMap<ValueId, ValueId>,

    /// Functions with conflicting definitions across modules.
    pub conflicts: Vec<LinkConflict>,
}

/// A conflicting symbol found during linking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkConflict {
    /// The symbol name that has multiple definitions.
    pub name: String,

    /// Modules that define this symbol.
    pub defining_modules: Vec<ModuleId>,
}

/// Tracks what changed between two analysis runs of the same program.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProgramDiff {
    /// Modules present in the new run but not the previous.
    pub added_modules: Vec<ModuleId>,

    /// Modules present in the previous run but not the new.
    pub removed_modules: Vec<ModuleId>,

    /// Modules whose fingerprint changed between runs.
    pub changed_modules: Vec<ModuleId>,

    /// Modules whose fingerprint is identical to the previous run.
    pub unchanged_modules: Vec<ModuleId>,

    /// Functions that appear in added or changed modules but not in previous.
    pub added_functions: BTreeSet<FunctionId>,

    /// Functions that were in removed or changed modules but not in new.
    pub removed_functions: BTreeSet<FunctionId>,
}

/// How to split a single pre-linked input into logical modules.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitStrategy {
    /// Group functions by their `source_files` metadata.
    BySourceFile,

    /// Each function becomes its own module.
    ByFunction,

    /// Keep as a single monolithic module (today's behavior).
    Monolithic,

    /// `BySourceFile` if source metadata present, else `ByFunction`.
    #[default]
    Auto,
}

/// A whole program composed of multiple linked modules.
#[derive(Debug, Clone)]
pub struct AirProgram {
    /// Deterministic ID derived from the sorted set of module fingerprints.
    pub id: ProgramId,

    /// Individual compilation units (one per input file).
    pub modules: Vec<AirModule>,

    /// Cross-module symbol resolution.
    pub link_table: LinkTable,
}

impl AirProgram {
    /// Link multiple bundles into a unified program.
    ///
    /// Resolves extern function declarations to their definitions across modules.
    /// Produces a `LinkTable` mapping declaration IDs to definition IDs.
    pub fn link(bundles: Vec<AirBundle>) -> Self {
        let modules: Vec<AirModule> = bundles.into_iter().map(|b| b.module).collect();

        // Compute deterministic ProgramId from sorted module IDs
        let mut module_ids: Vec<u128> = modules.iter().map(|m| m.id.raw()).collect();
        module_ids.sort_unstable();
        let id_bytes: Vec<u8> = module_ids.iter().flat_map(|id| id.to_le_bytes()).collect();
        let id = ProgramId::new(make_id("program", &id_bytes));

        let link_table = Self::resolve_symbols(&modules);

        Self {
            id,
            modules,
            link_table,
        }
    }

    /// Produce a flattened `AirModule` for backward compatibility with
    /// the existing single-module analysis pipeline.
    ///
    /// Merges all functions and globals into one module, rewriting
    /// extern declarations that have definitions in other modules.
    pub fn merged_view(&self) -> AirModule {
        let mut merged = AirModule::new(
            crate::ids::ModuleId::new(self.id.raw()),
        );
        merged.name = Some("merged".to_string());

        // Collect all defined function names to skip duplicate declarations
        let mut defined_functions: BTreeSet<String> = BTreeSet::new();
        for module in &self.modules {
            for func in &module.functions {
                if !func.is_declaration {
                    defined_functions.insert(func.name.clone());
                }
            }
        }

        // Merge functions: include definitions and unresolved declarations only
        for module in &self.modules {
            for func in &module.functions {
                if func.is_declaration && defined_functions.contains(&func.name) {
                    // Skip this declaration — a definition exists in another module
                    continue;
                }
                merged.add_function(func.clone());
            }
        }

        // Merge globals: include definitions and unresolved declarations only
        let mut defined_globals: BTreeSet<String> = BTreeSet::new();
        for module in &self.modules {
            for global in &module.globals {
                if global.init.is_some() {
                    defined_globals.insert(global.name.clone());
                }
            }
        }
        for module in &self.modules {
            for global in &module.globals {
                if global.init.is_none() && defined_globals.contains(&global.name) {
                    continue;
                }
                merged.globals.push(global.clone());
            }
        }

        // Merge source files (deduplicated by path)
        let mut seen_paths: BTreeSet<String> = BTreeSet::new();
        for module in &self.modules {
            for sf in &module.source_files {
                if seen_paths.insert(sf.path.clone()) {
                    merged.source_files.push(sf.clone());
                }
            }
        }

        // Merge type hierarchies
        for module in &self.modules {
            merged
                .type_hierarchy
                .extend(module.type_hierarchy.clone());
        }

        // Merge type tables
        for module in &self.modules {
            for (tid, ty) in &module.types {
                merged.types.entry(*tid).or_insert_with(|| ty.clone());
            }
        }

        // Merge constants
        for module in &self.modules {
            for (vid, c) in &module.constants {
                merged.constants.entry(*vid).or_insert_with(|| c.clone());
            }
        }

        // Use largest target_pointer_width
        merged.target_pointer_width = self
            .modules
            .iter()
            .map(|m| m.target_pointer_width)
            .max()
            .unwrap_or(8);

        merged.rebuild_function_index();
        merged
    }

    /// Resolve extern declarations across modules.
    fn resolve_symbols(modules: &[AirModule]) -> LinkTable {
        let mut table = LinkTable::default();

        // Index: function name -> Vec<(ModuleId, FunctionId, is_declaration)>
        let mut func_index: BTreeMap<String, Vec<(ModuleId, FunctionId, bool)>> =
            BTreeMap::new();
        for module in modules {
            for func in &module.functions {
                func_index
                    .entry(func.name.clone())
                    .or_default()
                    .push((module.id, func.id, func.is_declaration));
            }
        }

        // For each function name, find definitions and declarations
        for (_name, entries) in &func_index {
            let definitions: Vec<_> = entries.iter().filter(|e| !e.2).collect();
            let declarations: Vec<_> = entries.iter().filter(|e| e.2).collect();

            if definitions.len() > 1 {
                table.conflicts.push(LinkConflict {
                    name: _name.clone(),
                    defining_modules: definitions.iter().map(|d| d.0).collect(),
                });
            }

            if let Some(def) = definitions.first() {
                for decl in &declarations {
                    table
                        .function_resolutions
                        .insert(decl.1, def.1);
                }
            }
        }

        // Same for globals: name -> Vec<(ModuleId, ValueId, has_init)>
        let mut global_index: BTreeMap<String, Vec<(ModuleId, ValueId, bool)>> =
            BTreeMap::new();
        for module in modules {
            for global in &module.globals {
                global_index
                    .entry(global.name.clone())
                    .or_default()
                    .push((module.id, global.id, global.init.is_some()));
            }
        }

        for (_name, entries) in &global_index {
            let definitions: Vec<_> = entries.iter().filter(|e| e.2).collect();
            let declarations: Vec<_> = entries.iter().filter(|e| !e.2).collect();

            if let Some(def) = definitions.first() {
                for decl in &declarations {
                    table.global_resolutions.insert(decl.1, def.1);
                }
            }
        }

        table
    }

    /// Compute what changed between two programs.
    ///
    /// Compares modules by `ModuleId`. Modules with the same ID but different
    /// content should have different `ModuleId`s (since IDs are content-derived).
    pub fn diff(previous: &[ModuleId], current: &[ModuleId]) -> ProgramDiff {
        let prev_set: BTreeSet<ModuleId> = previous.iter().copied().collect();
        let curr_set: BTreeSet<ModuleId> = current.iter().copied().collect();

        ProgramDiff {
            added_modules: curr_set.difference(&prev_set).copied().collect(),
            removed_modules: prev_set.difference(&curr_set).copied().collect(),
            changed_modules: Vec::new(), // fingerprint-based diff is done at a higher level
            unchanged_modules: curr_set.intersection(&prev_set).copied().collect(),
            added_functions: BTreeSet::new(),
            removed_functions: BTreeSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::{AirBundle, AirFunction, AirGlobal, AirModule};
    use crate::id::make_id;
    use crate::ids::{FunctionId, ModuleId, ObjId, ValueId};

    fn make_function(name: &str, is_declaration: bool) -> AirFunction {
        let id = FunctionId::new(make_id("fn", name.as_bytes()));
        let mut f = AirFunction::new(id, name.to_string());
        f.is_declaration = is_declaration;
        f
    }

    fn make_module(name: &str, functions: Vec<AirFunction>) -> AirModule {
        let id = ModuleId::new(make_id("module", name.as_bytes()));
        let mut m = AirModule::new(id);
        m.name = Some(name.to_string());
        for f in functions {
            m.add_function(f);
        }
        m
    }

    fn make_bundle(name: &str, functions: Vec<AirFunction>) -> AirBundle {
        AirBundle::new("test".to_string(), make_module(name, functions))
    }

    fn make_global(name: &str, has_init: bool) -> AirGlobal {
        let id = ValueId::new(make_id("val", name.as_bytes()));
        let obj = ObjId::new(make_id("obj", name.as_bytes()));
        let mut g = AirGlobal::new(id, obj, name.to_string());
        if has_init {
            g.init = Some(crate::air::Constant::Int(0));
        }
        g
    }

    #[test]
    fn link_empty_program() {
        let program = AirProgram::link(vec![]);
        assert!(program.modules.is_empty());
        assert!(program.link_table.function_resolutions.is_empty());
    }

    #[test]
    fn link_single_module() {
        let bundle = make_bundle("main", vec![
            make_function("main", false),
            make_function("printf", true), // extern
        ]);
        let program = AirProgram::link(vec![bundle]);

        assert_eq!(program.modules.len(), 1);
        // No cross-module resolution possible with single module
        assert!(program.link_table.function_resolutions.is_empty());
    }

    #[test]
    fn link_resolves_extern_functions() {
        let main_bundle = make_bundle("main", vec![
            make_function("main", false),
            make_function("helper", true), // extern declaration
        ]);
        let lib_bundle = make_bundle("lib", vec![
            make_function("helper", false), // definition
        ]);

        let program = AirProgram::link(vec![main_bundle, lib_bundle]);

        assert_eq!(program.modules.len(), 2);
        // helper declaration in main -> helper definition in lib
        assert_eq!(program.link_table.function_resolutions.len(), 1);

        let decl_id = FunctionId::new(make_id("fn", b"helper"));
        assert!(program.link_table.function_resolutions.contains_key(&decl_id));
    }

    #[test]
    fn link_unresolved_extern_stays_unresolved() {
        let bundle = make_bundle("main", vec![
            make_function("main", false),
            make_function("printf", true), // libc — no definition anywhere
        ]);
        let program = AirProgram::link(vec![bundle]);

        // printf has no definition, so no resolution
        assert!(program.link_table.function_resolutions.is_empty());
    }

    #[test]
    fn link_detects_conflicting_definitions() {
        let a = make_bundle("a", vec![make_function("foo", false)]);
        let b = make_bundle("b", vec![make_function("foo", false)]);

        let program = AirProgram::link(vec![a, b]);

        assert_eq!(program.link_table.conflicts.len(), 1);
        assert_eq!(program.link_table.conflicts[0].name, "foo");
        assert_eq!(program.link_table.conflicts[0].defining_modules.len(), 2);
    }

    #[test]
    fn merged_view_contains_all_definitions() {
        let main_bundle = make_bundle("main", vec![
            make_function("main", false),
            make_function("helper", true), // extern
        ]);
        let lib_bundle = make_bundle("lib", vec![
            make_function("helper", false), // definition
        ]);

        let program = AirProgram::link(vec![main_bundle, lib_bundle]);
        let merged = program.merged_view();

        // Should have: main (def) + helper (def), NOT helper (decl)
        assert_eq!(merged.functions.len(), 2);
        let names: BTreeSet<_> = merged.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains("main"));
        assert!(names.contains("helper"));
        // The helper should be the definition, not the declaration
        let helper = merged.function_by_name("helper").unwrap();
        assert!(!helper.is_declaration);
    }

    #[test]
    fn merged_view_keeps_unresolved_declarations() {
        let bundle = make_bundle("main", vec![
            make_function("main", false),
            make_function("printf", true), // extern, no definition
        ]);

        let program = AirProgram::link(vec![bundle]);
        let merged = program.merged_view();

        // printf stays as a declaration since no module defines it
        assert_eq!(merged.functions.len(), 2);
        let printf = merged.function_by_name("printf").unwrap();
        assert!(printf.is_declaration);
    }

    #[test]
    fn merged_view_resolves_globals() {
        let mut main_mod = make_module("main", vec![make_function("main", false)]);
        let decl_global = make_global("config", false); // declaration
        main_mod.globals.push(decl_global);

        let mut lib_mod = make_module("lib", vec![]);
        let def_global = make_global("config", true); // definition (has init)
        lib_mod.globals.push(def_global);

        let program = AirProgram::link(vec![
            AirBundle::new("test".to_string(), main_mod),
            AirBundle::new("test".to_string(), lib_mod),
        ]);
        let merged = program.merged_view();

        // Only the definition should appear, not the declaration
        let config_globals: Vec<_> = merged
            .globals
            .iter()
            .filter(|g| g.name == "config")
            .collect();
        assert_eq!(config_globals.len(), 1);
        assert!(config_globals[0].init.is_some());
    }

    #[test]
    fn program_id_is_deterministic() {
        let bundles1 = vec![
            make_bundle("a", vec![make_function("fa", false)]),
            make_bundle("b", vec![make_function("fb", false)]),
        ];
        let bundles2 = vec![
            make_bundle("b", vec![make_function("fb", false)]),
            make_bundle("a", vec![make_function("fa", false)]),
        ];

        let p1 = AirProgram::link(bundles1);
        let p2 = AirProgram::link(bundles2);

        // Same modules in different order should produce same ProgramId
        assert_eq!(p1.id, p2.id);
    }

    #[test]
    fn diff_identifies_added_modules() {
        let m1 = ModuleId::new(1);
        let m2 = ModuleId::new(2);
        let m3 = ModuleId::new(3);

        let diff = AirProgram::diff(&[m1, m2], &[m1, m2, m3]);

        assert_eq!(diff.added_modules, vec![m3]);
        assert!(diff.removed_modules.is_empty());
        assert_eq!(diff.unchanged_modules.len(), 2);
    }

    #[test]
    fn diff_identifies_removed_modules() {
        let m1 = ModuleId::new(1);
        let m2 = ModuleId::new(2);

        let diff = AirProgram::diff(&[m1, m2], &[m1]);

        assert!(diff.added_modules.is_empty());
        assert_eq!(diff.removed_modules, vec![m2]);
        assert_eq!(diff.unchanged_modules.len(), 1);
    }
}
```

**Step 2: Export the module from saf-core**

Add to `crates/saf-core/src/lib.rs`:

```rust
pub mod program;
```

**Step 3: Run tests**

Run: `cargo test -p saf-core` (safe locally)
Expected: All new tests pass. All existing tests pass.

**Step 4: Commit**

```bash
git add crates/saf-core/src/program.rs crates/saf-core/src/lib.rs
git commit -m "feat(core): add AirProgram, LinkTable, and ProgramDiff types"
```

---

## Task 3: Extend `LlvmFrontend` to support multi-file ingestion

**Files:**
- Modify: `crates/saf-frontends/src/api.rs` (add `ingest_multi` default method)
- Modify: `crates/saf-frontends/src/llvm/mod.rs` (implement `ingest_multi`)

**Step 1: Add `ingest_multi` to the `Frontend` trait with a default implementation**

In `crates/saf-frontends/src/api.rs`, add after the existing trait methods:

```rust
    /// Ingest multiple input files, returning a separate [`AirBundle`] per file.
    ///
    /// Uses `cache` (if provided) to skip re-ingestion for files whose
    /// fingerprint matches a cached bundle. Falls back to per-file `ingest()`.
    fn ingest_multi(
        &self,
        inputs: &[&Path],
        config: &Config,
        cache: Option<&saf_core::cache::BundleCache>,
    ) -> Result<Vec<AirBundle>, FrontendError> {
        let mut bundles = Vec::with_capacity(inputs.len());
        for input in inputs {
            // Check cache
            if let Some(cache) = cache {
                if let Ok(fp) = self.input_fingerprint_bytes(&[*input], config) {
                    if let Some(bundle) = cache.get(&fp) {
                        bundles.push(bundle);
                        continue;
                    }
                }
            }

            let bundle = self.ingest(&[*input], config)?;

            // Store in cache
            if let Some(cache) = cache {
                if let Ok(fp) = self.input_fingerprint_bytes(&[*input], config) {
                    let _ = cache.put(&fp, &bundle);
                }
            }

            bundles.push(bundle);
        }
        Ok(bundles)
    }
```

**Step 2: Remove the multi-input error from `LlvmFrontend::ingest`**

In `crates/saf-frontends/src/llvm/mod.rs`, the `ingest()` method errors on `inputs.len() > 1` (around line 107-111). Leave this as-is — `ingest()` stays single-file, `ingest_multi()` calls `ingest()` per-file via the default trait implementation. No changes needed to `LlvmFrontend` itself.

**Step 3: Run tests**

Run: `make test` (must be in Docker for LLVM frontends)
Expected: All existing tests pass. The new `ingest_multi` default works via the trait.

**Step 4: Commit**

```bash
git add crates/saf-frontends/src/api.rs
git commit -m "feat(frontends): add ingest_multi with per-file caching to Frontend trait"
```

---

## Task 4: Write multi-file AIR-JSON test fixtures

**Files:**
- Create: `tests/fixtures/incremental/two_module/main.air.json`
- Create: `tests/fixtures/incremental/two_module/lib.air.json`
- Create: `tests/fixtures/incremental/two_module/lib_v2.air.json` (changed version)

These fixtures represent a two-module program where `main` calls `helper` defined in `lib`.

**Step 1: Create main module fixture**

`tests/fixtures/incremental/two_module/main.air.json`:

```json
{
  "schema_version": "0.1.0",
  "frontend_id": "air-json",
  "module": {
    "id": "0x00000000000000000000000000000001",
    "name": "main",
    "functions": [
      {
        "id": "0x00000000000000000000000000000010",
        "name": "main",
        "params": [],
        "blocks": [
          {
            "id": "0x00000000000000000000000000000100",
            "name": "entry",
            "instructions": [
              {
                "id": "0x00000000000000000000000000001000",
                "result": "0x00000000000000000000000000001001",
                "operation": {
                  "type": "CallDirect",
                  "callee": "helper",
                  "args": []
                }
              },
              {
                "id": "0x00000000000000000000000000001002",
                "operation": {
                  "type": "Return",
                  "value": "0x00000000000000000000000000001001"
                }
              }
            ]
          }
        ],
        "entry_block": "0x00000000000000000000000000000100",
        "is_declaration": false
      },
      {
        "id": "0x00000000000000000000000000000011",
        "name": "helper",
        "params": [],
        "blocks": [],
        "is_declaration": true
      }
    ],
    "globals": []
  }
}
```

**Step 2: Create lib module fixture**

`tests/fixtures/incremental/two_module/lib.air.json`:

```json
{
  "schema_version": "0.1.0",
  "frontend_id": "air-json",
  "module": {
    "id": "0x00000000000000000000000000000002",
    "name": "lib",
    "functions": [
      {
        "id": "0x00000000000000000000000000000020",
        "name": "helper",
        "params": [],
        "blocks": [
          {
            "id": "0x00000000000000000000000000000200",
            "name": "entry",
            "instructions": [
              {
                "id": "0x00000000000000000000000000002000",
                "operation": {
                  "type": "Return"
                }
              }
            ]
          }
        ],
        "entry_block": "0x00000000000000000000000000000200",
        "is_declaration": false
      }
    ],
    "globals": []
  }
}
```

**Step 3: Create lib_v2 (changed version — helper has a different body)**

`tests/fixtures/incremental/two_module/lib_v2.air.json`:

Same as `lib.air.json` but with a different `ModuleId` and an extra instruction:

```json
{
  "schema_version": "0.1.0",
  "frontend_id": "air-json",
  "module": {
    "id": "0x00000000000000000000000000000003",
    "name": "lib",
    "functions": [
      {
        "id": "0x00000000000000000000000000000020",
        "name": "helper",
        "params": [],
        "blocks": [
          {
            "id": "0x00000000000000000000000000000200",
            "name": "entry",
            "instructions": [
              {
                "id": "0x00000000000000000000000000002001",
                "result": "0x00000000000000000000000000002002",
                "operation": {
                  "type": "Alloca"
                }
              },
              {
                "id": "0x00000000000000000000000000002003",
                "operation": {
                  "type": "Return",
                  "value": "0x00000000000000000000000000002002"
                }
              }
            ]
          }
        ],
        "entry_block": "0x00000000000000000000000000000200",
        "is_declaration": false
      }
    ],
    "globals": []
  }
}
```

**Step 4: Commit**

```bash
git add tests/fixtures/incremental/
git commit -m "test: add multi-module AIR-JSON fixtures for incremental analysis"
```

---

## Task 5: Integration test — multi-file ingestion + linking + merged view

**Files:**
- Create: `crates/saf-core/tests/program.rs`

**Step 1: Write integration tests for `AirProgram`**

```rust
//! Integration tests for multi-module AirProgram linking.

use std::path::PathBuf;

use saf_core::program::AirProgram;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/incremental/two_module")
        .join(name)
}

#[test]
fn ingest_and_link_two_modules() {
    let main_json = std::fs::read_to_string(fixture_path("main.air.json")).unwrap();
    let lib_json = std::fs::read_to_string(fixture_path("lib.air.json")).unwrap();

    let main_bundle: saf_core::air::AirBundle = serde_json::from_str(&main_json).unwrap();
    let lib_bundle: saf_core::air::AirBundle = serde_json::from_str(&lib_json).unwrap();

    let program = AirProgram::link(vec![main_bundle, lib_bundle]);

    assert_eq!(program.modules.len(), 2);

    // helper declaration in main should resolve to helper definition in lib
    assert_eq!(program.link_table.function_resolutions.len(), 1);
    assert!(program.link_table.conflicts.is_empty());
}

#[test]
fn merged_view_has_definition_not_declaration() {
    let main_json = std::fs::read_to_string(fixture_path("main.air.json")).unwrap();
    let lib_json = std::fs::read_to_string(fixture_path("lib.air.json")).unwrap();

    let main_bundle: saf_core::air::AirBundle = serde_json::from_str(&main_json).unwrap();
    let lib_bundle: saf_core::air::AirBundle = serde_json::from_str(&lib_json).unwrap();

    let program = AirProgram::link(vec![main_bundle, lib_bundle]);
    let merged = program.merged_view();

    // merged should have 2 functions: main (def) + helper (def)
    assert_eq!(merged.functions.len(), 2);

    let helper = merged.function_by_name("helper").unwrap();
    assert!(!helper.is_declaration, "helper should be a definition in merged view");
    assert!(!helper.blocks.is_empty(), "helper definition should have blocks");
}

#[test]
fn merged_view_produces_valid_module_for_analysis() {
    let main_json = std::fs::read_to_string(fixture_path("main.air.json")).unwrap();
    let lib_json = std::fs::read_to_string(fixture_path("lib.air.json")).unwrap();

    let main_bundle: saf_core::air::AirBundle = serde_json::from_str(&main_json).unwrap();
    let lib_bundle: saf_core::air::AirBundle = serde_json::from_str(&lib_json).unwrap();

    let program = AirProgram::link(vec![main_bundle, lib_bundle]);
    let merged = program.merged_view();

    // The merged module should be round-trippable through JSON
    let json = serde_json::to_string(&merged).unwrap();
    let deserialized: saf_core::air::AirModule = serde_json::from_str(&json).unwrap();
    assert_eq!(merged.functions.len(), deserialized.functions.len());

    // Function index should work
    assert!(merged.function_by_name("main").is_some());
    assert!(merged.function_by_name("helper").is_some());
}

#[test]
fn program_diff_detects_changed_module() {
    let main_json = std::fs::read_to_string(fixture_path("main.air.json")).unwrap();
    let lib_v1_json = std::fs::read_to_string(fixture_path("lib.air.json")).unwrap();
    let lib_v2_json = std::fs::read_to_string(fixture_path("lib_v2.air.json")).unwrap();

    let main_bundle: saf_core::air::AirBundle = serde_json::from_str(&main_json).unwrap();
    let lib_v1: saf_core::air::AirBundle = serde_json::from_str(&lib_v1_json).unwrap();
    let lib_v2: saf_core::air::AirBundle = serde_json::from_str(&lib_v2_json).unwrap();

    let prev_ids = vec![main_bundle.module.id, lib_v1.module.id];
    let curr_ids = vec![main_bundle.module.id, lib_v2.module.id];

    let diff = AirProgram::diff(&prev_ids, &curr_ids);

    // main unchanged, lib_v1 removed, lib_v2 added
    assert_eq!(diff.unchanged_modules.len(), 1);
    assert_eq!(diff.removed_modules.len(), 1);
    assert_eq!(diff.added_modules.len(), 1);
    assert_eq!(diff.removed_modules[0], lib_v1.module.id);
    assert_eq!(diff.added_modules[0], lib_v2.module.id);
}
```

**Step 2: Run tests**

Run: `cargo test -p saf-core` (safe locally)
Expected: All tests pass.

**Step 3: Commit**

```bash
git add crates/saf-core/tests/program.rs
git commit -m "test(core): integration tests for AirProgram linking and merged view"
```

---

## Task 6: Wire `BundleCache` into `AirJsonFrontend`

**Files:**
- Modify: `crates/saf-frontends/src/air_json.rs` (override `ingest_multi` for normalized fingerprinting)
- Create: `crates/saf-frontends/tests/multi_file.rs`

**Step 1: Write the test first**

Create `crates/saf-frontends/tests/multi_file.rs`:

```rust
//! Tests for multi-file ingestion with caching.

use std::path::PathBuf;

use saf_core::cache::BundleCache;
use saf_core::config::Config;
use saf_frontends::air_json::AirJsonFrontend;
use saf_frontends::api::Frontend;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/incremental/two_module")
        .join(name)
}

#[test]
fn ingest_multi_air_json_returns_two_bundles() {
    let frontend = AirJsonFrontend::new();
    let config = Config::default();

    let main_path = fixture_path("main.air.json");
    let lib_path = fixture_path("lib.air.json");
    let inputs: Vec<&std::path::Path> = vec![main_path.as_path(), lib_path.as_path()];

    let bundles = frontend.ingest_multi(&inputs, &config, None).unwrap();

    assert_eq!(bundles.len(), 2);
    assert_eq!(bundles[0].module.name.as_deref(), Some("main"));
    assert_eq!(bundles[1].module.name.as_deref(), Some("lib"));
}

#[test]
fn ingest_multi_with_cache_hits_on_second_call() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = BundleCache::new(tmp.path());
    let frontend = AirJsonFrontend::new();
    let config = Config::default();

    let main_path = fixture_path("main.air.json");
    let lib_path = fixture_path("lib.air.json");
    let inputs: Vec<&std::path::Path> = vec![main_path.as_path(), lib_path.as_path()];

    // First call: cache miss, populates cache
    let bundles1 = frontend.ingest_multi(&inputs, &config, Some(&cache)).unwrap();
    assert_eq!(bundles1.len(), 2);

    // Second call: cache hit
    let bundles2 = frontend.ingest_multi(&inputs, &config, Some(&cache)).unwrap();
    assert_eq!(bundles2.len(), 2);

    // Results should be equivalent
    assert_eq!(bundles1[0].module.name, bundles2[0].module.name);
    assert_eq!(bundles1[1].module.name, bundles2[1].module.name);
    assert_eq!(bundles1[0].module.functions.len(), bundles2[0].module.functions.len());
}

#[test]
fn ingest_multi_detects_changed_file() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = BundleCache::new(tmp.path());
    let frontend = AirJsonFrontend::new();
    let config = Config::default();

    let main_path = fixture_path("main.air.json");
    let lib_v1_path = fixture_path("lib.air.json");
    let lib_v2_path = fixture_path("lib_v2.air.json");

    // First run: main + lib_v1
    let inputs1: Vec<&std::path::Path> = vec![main_path.as_path(), lib_v1_path.as_path()];
    let bundles1 = frontend.ingest_multi(&inputs1, &config, Some(&cache)).unwrap();
    assert_eq!(bundles1.len(), 2);

    // Second run: main + lib_v2 (different file content)
    let inputs2: Vec<&std::path::Path> = vec![main_path.as_path(), lib_v2_path.as_path()];
    let bundles2 = frontend.ingest_multi(&inputs2, &config, Some(&cache)).unwrap();
    assert_eq!(bundles2.len(), 2);

    // main should be the same (cache hit), lib should differ
    assert_eq!(bundles1[0].module.id, bundles2[0].module.id); // main unchanged
    assert_ne!(bundles1[1].module.id, bundles2[1].module.id); // lib changed
}
```

**Step 2: Add `tempfile` dev-dependency if not already present**

Check `crates/saf-frontends/Cargo.toml` for `[dev-dependencies]`. Add `tempfile = "3"` if missing.

**Step 3: Run tests**

Run: `cargo test -p saf-frontends --test multi_file` (safe locally for AIR-JSON, no LLVM)
Expected: All 3 tests pass using the default trait implementation of `ingest_multi`.

**Step 4: Commit**

```bash
git add crates/saf-frontends/tests/multi_file.rs crates/saf-frontends/Cargo.toml
git commit -m "test(frontends): multi-file ingestion tests with BundleCache"
```

---

## Task 7: Add `IncrementalConfig` to core config

**Files:**
- Modify: `crates/saf-core/src/config.rs`

**Step 1: Add `IncrementalConfig` struct**

Add after the existing config types in `crates/saf-core/src/config.rs`:

```rust
/// Configuration for incremental analysis (Plan 165).
///
/// When `enabled` is false (default), the system behaves identically to
/// non-incremental analysis. All fields are ignored unless `enabled` is true.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IncrementalConfig {
    /// Master switch for incremental analysis.
    pub enabled: bool,

    /// Cache directory for per-module AIR bundles and analysis products.
    pub cache_dir: std::path::PathBuf,

    /// How to split a single pre-linked input into logical modules.
    pub split_strategy: crate::program::SplitStrategy,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_dir: std::path::PathBuf::from(".saf-cache"),
            split_strategy: crate::program::SplitStrategy::Auto,
        }
    }
}
```

Add the field to `Config`:

```rust
pub struct Config {
    pub frontend: Frontend,
    pub analysis: AnalysisConfig,
    pub external_side_effects: ExternalSideEffects,
    pub paths: PathsConfig,
    pub rust: RustConfig,
    #[serde(default)]
    pub incremental: IncrementalConfig,
}
```

**Step 2: Run tests**

Run: `cargo test -p saf-core` (safe locally)
Expected: All tests pass. `Config::default()` still works (new field has `Default`).

**Step 3: Commit**

```bash
git add crates/saf-core/src/config.rs
git commit -m "feat(core): add IncrementalConfig to Config"
```

---

## Task 8: End-to-end test — multi-file ingestion through analysis pipeline

This is the critical validation: link two AIR-JSON modules, produce a merged view, and run it through the existing `run_pipeline()`. Verify identical results to a single pre-merged module.

**Files:**
- Create: `crates/saf-analysis/tests/multi_module_e2e.rs`
- Create: `tests/fixtures/incremental/two_module/merged.air.json` (pre-merged single-file equivalent)

**Step 1: Create the pre-merged fixture**

`tests/fixtures/incremental/two_module/merged.air.json` — a single module containing both `main` and `helper` as definitions:

```json
{
  "schema_version": "0.1.0",
  "frontend_id": "air-json",
  "module": {
    "id": "0x00000000000000000000000000000099",
    "name": "merged",
    "functions": [
      {
        "id": "0x00000000000000000000000000000010",
        "name": "main",
        "params": [],
        "blocks": [
          {
            "id": "0x00000000000000000000000000000100",
            "name": "entry",
            "instructions": [
              {
                "id": "0x00000000000000000000000000001000",
                "result": "0x00000000000000000000000000001001",
                "operation": {
                  "type": "CallDirect",
                  "callee": "helper",
                  "args": []
                }
              },
              {
                "id": "0x00000000000000000000000000001002",
                "operation": {
                  "type": "Return",
                  "value": "0x00000000000000000000000000001001"
                }
              }
            ]
          }
        ],
        "entry_block": "0x00000000000000000000000000000100",
        "is_declaration": false
      },
      {
        "id": "0x00000000000000000000000000000020",
        "name": "helper",
        "params": [],
        "blocks": [
          {
            "id": "0x00000000000000000000000000000200",
            "name": "entry",
            "instructions": [
              {
                "id": "0x00000000000000000000000000002000",
                "operation": {
                  "type": "Return"
                }
              }
            ]
          }
        ],
        "entry_block": "0x00000000000000000000000000000200",
        "is_declaration": false
      }
    ],
    "globals": []
  }
}
```

**Step 2: Write the E2E test**

Create `crates/saf-analysis/tests/multi_module_e2e.rs`:

```rust
//! End-to-end test: multi-module AirProgram merged view produces
//! equivalent analysis results to a pre-merged single module.

use std::path::PathBuf;

use saf_core::air::AirBundle;
use saf_core::program::AirProgram;
use saf_analysis::pipeline::{PipelineConfig, run_pipeline};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/incremental/two_module")
        .join(name)
}

fn load_bundle(name: &str) -> AirBundle {
    let json = std::fs::read_to_string(fixture_path(name)).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn merged_view_produces_same_callgraph_as_single_module() {
    // Multi-module path
    let main_bundle = load_bundle("main.air.json");
    let lib_bundle = load_bundle("lib.air.json");
    let program = AirProgram::link(vec![main_bundle, lib_bundle]);
    let merged = program.merged_view();

    let config = PipelineConfig::default();
    let multi_result = run_pipeline(&merged, &config);

    // Single-module path (pre-merged)
    let single_bundle = load_bundle("merged.air.json");
    let single_result = run_pipeline(&single_bundle.module, &config);

    // Call graph should have the same edges
    let multi_cg = &multi_result.call_graph;
    let single_cg = &single_result.call_graph;

    // Both should discover main -> helper call edge
    let multi_edges: Vec<_> = multi_cg
        .edges()
        .map(|(src, dst)| {
            (
                multi_result.call_graph.node_name(src).unwrap_or_default(),
                multi_result.call_graph.node_name(dst).unwrap_or_default(),
            )
        })
        .collect();
    let single_edges: Vec<_> = single_cg
        .edges()
        .map(|(src, dst)| {
            (
                single_result.call_graph.node_name(src).unwrap_or_default(),
                single_result.call_graph.node_name(dst).unwrap_or_default(),
            )
        })
        .collect();

    // Verify the main->helper edge exists in both
    assert!(
        multi_edges.iter().any(|(s, d)| s == "main" && d == "helper"),
        "multi-module callgraph missing main->helper edge: {multi_edges:?}"
    );
    assert!(
        single_edges.iter().any(|(s, d)| s == "main" && d == "helper"),
        "single-module callgraph missing main->helper edge: {single_edges:?}"
    );
}
```

**Step 3: Run tests**

Run: `make test` (must be in Docker — saf-analysis depends on LLVM transitively)
Expected: E2E test passes, proving merged view works with the full pipeline.

**Step 4: Commit**

```bash
git add tests/fixtures/incremental/two_module/merged.air.json crates/saf-analysis/tests/multi_module_e2e.rs
git commit -m "test(analysis): E2E test proving multi-module merged view matches single-module analysis"
```

---

## Task 9: Single-file auto-split by source metadata

**Files:**
- Modify: `crates/saf-core/src/program.rs` (add `split_module` function)

**Step 1: Add split function and tests**

Add the following to `crates/saf-core/src/program.rs`:

```rust
/// Split a monolithic module into per-source-file modules.
///
/// Groups functions by their `Span.file` field (which maps to a `SourceFile`
/// entry). Functions without span information go into a "unknown" module.
/// If no functions have span info, returns the original module unchanged
/// in a single-element Vec.
pub fn split_module(module: AirModule, strategy: SplitStrategy) -> Vec<AirModule> {
    match strategy {
        SplitStrategy::Monolithic => vec![module],
        SplitStrategy::ByFunction => split_by_function(module),
        SplitStrategy::BySourceFile => split_by_source_file(module),
        SplitStrategy::Auto => {
            // Check if any function has span info
            let has_spans = module
                .functions
                .iter()
                .any(|f| f.span.is_some());
            if has_spans {
                split_by_source_file(module)
            } else {
                split_by_function(module)
            }
        }
    }
}

fn split_by_function(module: AirModule) -> Vec<AirModule> {
    if module.functions.is_empty() {
        return vec![module];
    }

    let mut result = Vec::new();
    for func in &module.functions {
        let mod_id = ModuleId::new(make_id(
            "module-split-fn",
            func.name.as_bytes(),
        ));
        let mut sub = AirModule::new(mod_id);
        sub.name = Some(func.name.clone());
        sub.target_pointer_width = module.target_pointer_width;
        // Copy types and constants needed by this function
        sub.types.clone_from(&module.types);
        sub.constants.clone_from(&module.constants);
        sub.add_function(func.clone());
        result.push(sub);
    }

    // Globals go into the first module (or a synthetic "globals" module)
    if !module.globals.is_empty() {
        if let Some(first) = result.first_mut() {
            first.globals = module.globals.clone();
        }
    }

    result
}

fn split_by_source_file(module: AirModule) -> Vec<AirModule> {
    use std::collections::BTreeMap as Map;

    // Build file_id -> source_file path mapping
    let file_paths: Map<crate::ids::FileId, String> = module
        .source_files
        .iter()
        .map(|sf| (sf.id, sf.path.clone()))
        .collect();

    // Group functions by their source file
    let mut groups: Map<String, Vec<AirFunction>> = Map::new();
    for func in &module.functions {
        let key = func
            .span
            .as_ref()
            .and_then(|s| file_paths.get(&s.file))
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string());
        groups.entry(key).or_default().push(func.clone());
    }

    if groups.len() <= 1 {
        // All functions from same file (or no span info) — don't split
        return vec![module];
    }

    let mut result = Vec::new();
    for (path, functions) in &groups {
        let mod_id = ModuleId::new(make_id(
            "module-split-src",
            path.as_bytes(),
        ));
        let mut sub = AirModule::new(mod_id);
        sub.name = Some(path.clone());
        sub.target_pointer_width = module.target_pointer_width;
        sub.types.clone_from(&module.types);
        sub.constants.clone_from(&module.constants);
        for f in functions {
            sub.add_function(f.clone());
        }
        result.push(sub);
    }

    // Distribute globals: for now, all globals go into the first module
    if !module.globals.is_empty() {
        if let Some(first) = result.first_mut() {
            first.globals = module.globals.clone();
        }
    }

    result
}
```

**Step 2: Add tests for split functions**

Add to the `#[cfg(test)] mod tests` in the same file:

```rust
    #[test]
    fn split_monolithic_returns_original() {
        let module = make_module("test", vec![
            make_function("a", false),
            make_function("b", false),
        ]);
        let result = super::split_module(module, SplitStrategy::Monolithic);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].functions.len(), 2);
    }

    #[test]
    fn split_by_function_creates_one_module_per_function() {
        let module = make_module("test", vec![
            make_function("a", false),
            make_function("b", false),
            make_function("c", false),
        ]);
        let result = super::split_module(module, SplitStrategy::ByFunction);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].functions.len(), 1);
        assert_eq!(result[0].functions[0].name, "a");
    }

    #[test]
    fn split_auto_uses_by_function_when_no_spans() {
        let module = make_module("test", vec![
            make_function("a", false),
            make_function("b", false),
        ]);
        let result = super::split_module(module, SplitStrategy::Auto);
        // No spans → ByFunction → 2 modules
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn split_empty_module_returns_single() {
        let module = make_module("test", vec![]);
        let result = super::split_module(module, SplitStrategy::ByFunction);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn split_preserves_globals_in_first_module() {
        let mut module = make_module("test", vec![
            make_function("a", false),
            make_function("b", false),
        ]);
        module.globals.push(make_global("g", true));

        let result = super::split_module(module, SplitStrategy::ByFunction);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].globals.len(), 1);
        assert!(result[1].globals.is_empty());
    }
```

**Step 3: Run tests**

Run: `cargo test -p saf-core` (safe locally)
Expected: All tests pass.

**Step 4: Commit**

```bash
git add crates/saf-core/src/program.rs
git commit -m "feat(core): add split_module for single-file auto-split"
```

---

## Task 10: Cache manifest for cross-run change detection

**Files:**
- Create: `crates/saf-core/src/manifest.rs`
- Modify: `crates/saf-core/src/lib.rs` (add `pub mod manifest;`)

**Step 1: Implement manifest**

```rust
//! Cache manifest for tracking module fingerprints across analysis runs.
//!
//! The manifest records which modules were analyzed in a previous run
//! and their content fingerprints, enabling fast change detection on
//! subsequent runs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ids::{ModuleId, ProgramId};

/// Per-module entry in the manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// The module's content-derived ID.
    pub module_id: ModuleId,

    /// BLAKE3 fingerprint of the input file (hex-encoded).
    pub fingerprint: String,

    /// Original input file path (for display/debugging).
    pub input_path: String,
}

/// Persisted manifest recording the state of a previous analysis run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CacheManifest {
    /// Program ID from the previous run.
    pub program_id: Option<ProgramId>,

    /// Per-module entries keyed by input file path.
    pub modules: BTreeMap<String, ManifestEntry>,
}

impl CacheManifest {
    /// Load manifest from a cache directory. Returns default if not found.
    pub fn load(cache_dir: &Path) -> Self {
        let path = Self::manifest_path(cache_dir);
        match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save manifest to a cache directory.
    pub fn save(&self, cache_dir: &Path) -> Result<(), std::io::Error> {
        let path = Self::manifest_path(cache_dir);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&path, json)
    }

    fn manifest_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join("manifest.json")
    }

    /// Compare this manifest (previous run) against current fingerprints.
    ///
    /// Returns lists of unchanged, changed, added, and removed input paths.
    pub fn diff(
        &self,
        current: &BTreeMap<String, String>,
    ) -> ManifestDiff {
        let mut unchanged = Vec::new();
        let mut changed = Vec::new();
        let mut added = Vec::new();
        let mut removed = Vec::new();

        // Check current against previous
        for (path, fingerprint) in current {
            match self.modules.get(path) {
                Some(entry) if entry.fingerprint == *fingerprint => {
                    unchanged.push(path.clone());
                }
                Some(_) => {
                    changed.push(path.clone());
                }
                None => {
                    added.push(path.clone());
                }
            }
        }

        // Check for removed files
        for path in self.modules.keys() {
            if !current.contains_key(path) {
                removed.push(path.clone());
            }
        }

        ManifestDiff {
            unchanged,
            changed,
            added,
            removed,
        }
    }
}

/// Result of comparing previous manifest against current fingerprints.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ManifestDiff {
    /// Files whose fingerprint matches the previous run.
    pub unchanged: Vec<String>,

    /// Files whose fingerprint changed since the previous run.
    pub changed: Vec<String>,

    /// Files present now but not in the previous run.
    pub added: Vec<String>,

    /// Files in the previous run but not present now.
    pub removed: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manifest_diff_shows_all_added() {
        let manifest = CacheManifest::default();
        let mut current = BTreeMap::new();
        current.insert("a.ll".to_string(), "aaa".to_string());
        current.insert("b.ll".to_string(), "bbb".to_string());

        let diff = manifest.diff(&current);
        assert_eq!(diff.added.len(), 2);
        assert!(diff.unchanged.is_empty());
        assert!(diff.changed.is_empty());
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn same_fingerprints_show_unchanged() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "aaa".to_string(),
                input_path: "a.ll".to_string(),
            },
        );

        let mut current = BTreeMap::new();
        current.insert("a.ll".to_string(), "aaa".to_string());

        let diff = manifest.diff(&current);
        assert_eq!(diff.unchanged, vec!["a.ll"]);
        assert!(diff.changed.is_empty());
    }

    #[test]
    fn changed_fingerprint_detected() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "old".to_string(),
                input_path: "a.ll".to_string(),
            },
        );

        let mut current = BTreeMap::new();
        current.insert("a.ll".to_string(), "new".to_string());

        let diff = manifest.diff(&current);
        assert_eq!(diff.changed, vec!["a.ll"]);
        assert!(diff.unchanged.is_empty());
    }

    #[test]
    fn removed_file_detected() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "deleted.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "xxx".to_string(),
                input_path: "deleted.ll".to_string(),
            },
        );

        let current = BTreeMap::new(); // empty — file was deleted

        let diff = manifest.diff(&current);
        assert_eq!(diff.removed, vec!["deleted.ll"]);
    }

    #[test]
    fn manifest_roundtrip_through_filesystem() {
        let tmp = tempfile::tempdir().unwrap();
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "test.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(42),
                fingerprint: "deadbeef".to_string(),
                input_path: "test.ll".to_string(),
            },
        );

        manifest.save(tmp.path()).unwrap();
        let loaded = CacheManifest::load(tmp.path());
        assert_eq!(manifest, loaded);
    }
}
```

**Step 2: Export from lib.rs**

Add `pub mod manifest;` to `crates/saf-core/src/lib.rs`.

**Step 3: Add `tempfile` dev-dependency to saf-core if not present**

Check `crates/saf-core/Cargo.toml`. Add `tempfile = "3"` under `[dev-dependencies]` if missing.

**Step 4: Run tests**

Run: `cargo test -p saf-core` (safe locally)
Expected: All tests pass.

**Step 5: Commit**

```bash
git add crates/saf-core/src/manifest.rs crates/saf-core/src/lib.rs crates/saf-core/Cargo.toml
git commit -m "feat(core): add CacheManifest for cross-run change detection"
```

---

## Task 11: Validation — run full test suite

**Step 1: Run complete test suite in Docker**

Run: `make fmt && make lint` (format + lint)
Run: `make test` (full Rust + Python tests)

Expected: All existing tests pass. No regressions. New tests pass.

**Step 2: Verify test counts**

Grep output for `Summary` line. Total should be previous count + new tests from Tasks 1-10.

**Step 3: Final commit if any lint fixes needed**

```bash
git add -A && git commit -m "chore: lint fixes for Phase 0 + Phase 1"
```

---

## Summary

| Task | What | Files | Test count |
|------|------|-------|------------|
| 1 | `ProgramId` type | `ids.rs` | 0 (macro-generated) |
| 2 | `AirProgram`, `LinkTable`, `ProgramDiff` | `program.rs`, `lib.rs` | 10 unit tests |
| 3 | `ingest_multi` on Frontend trait | `api.rs` | 0 (tested in Task 6) |
| 4 | Multi-file AIR-JSON fixtures | `tests/fixtures/incremental/` | 0 (data) |
| 5 | Integration tests for linking | `tests/program.rs` | 4 integration tests |
| 6 | Multi-file ingestion + caching tests | `tests/multi_file.rs` | 3 integration tests |
| 7 | `IncrementalConfig` in Config | `config.rs` | 0 (tested via existing Default tests) |
| 8 | E2E: merged view through pipeline | `tests/multi_module_e2e.rs` | 1 E2E test |
| 9 | Single-file auto-split | `program.rs` | 5 unit tests |
| 10 | Cache manifest | `manifest.rs`, `lib.rs` | 5 unit tests |
| 11 | Full validation | — | All tests green |

**Total new tests:** ~28
**Total new files:** 7 (2 source, 5 test/fixture)
**Total modified files:** 4 (`ids.rs`, `lib.rs`, `api.rs`, `config.rs`)
