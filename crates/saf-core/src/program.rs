//! Multi-module program linking.
//!
//! `AirProgram` represents a whole program composed of multiple linked
//! `AirModule`s. The `LinkTable` resolves cross-module references
//! (extern declarations matched to definitions).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::air::{AirBundle, AirFunction, AirModule};
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
    #[must_use]
    pub fn merged_view(&self) -> AirModule {
        let mut merged = AirModule::new(crate::ids::ModuleId::new(self.id.raw()));
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

        // Rewrite CallDirect callee IDs: replace declaration IDs with definition IDs
        if !self.link_table.function_resolutions.is_empty() {
            for func in &mut merged.functions {
                for block in &mut func.blocks {
                    for inst in &mut block.instructions {
                        if let crate::air::Operation::CallDirect { callee } = &mut inst.op {
                            if let Some(def_id) = self.link_table.function_resolutions.get(callee) {
                                *callee = *def_id;
                            }
                        }
                    }
                }
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
            merged.type_hierarchy.extend(module.type_hierarchy.clone());
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
        let mut func_index: BTreeMap<String, Vec<(ModuleId, FunctionId, bool)>> = BTreeMap::new();
        for module in modules {
            for func in &module.functions {
                func_index.entry(func.name.clone()).or_default().push((
                    module.id,
                    func.id,
                    func.is_declaration,
                ));
            }
        }

        // For each function name, find definitions and declarations
        for (name, entries) in &func_index {
            let definitions: Vec<_> = entries.iter().filter(|e| !e.2).collect();
            let declarations: Vec<_> = entries.iter().filter(|e| e.2).collect();

            if definitions.len() > 1 {
                table.conflicts.push(LinkConflict {
                    name: name.clone(),
                    defining_modules: definitions.iter().map(|d| d.0).collect(),
                });
            }

            if let Some(def) = definitions.first() {
                for decl in &declarations {
                    table.function_resolutions.insert(decl.1, def.1);
                }
            }
        }

        // Same for globals: name -> Vec<(ModuleId, ValueId, has_init)>
        let mut global_index: BTreeMap<String, Vec<(ModuleId, ValueId, bool)>> = BTreeMap::new();
        for module in modules {
            for global in &module.globals {
                global_index.entry(global.name.clone()).or_default().push((
                    module.id,
                    global.id,
                    global.init.is_some(),
                ));
            }
        }

        for entries in global_index.values() {
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

/// Split a monolithic module into per-source-file modules.
///
/// Groups functions by their `Span.file_id` field (which maps to a `SourceFile`
/// entry). Functions without span information go into an "unknown" module.
/// If no functions have span info, returns the original module unchanged
/// in a single-element Vec.
pub fn split_module(module: AirModule, strategy: SplitStrategy) -> Vec<AirModule> {
    match strategy {
        SplitStrategy::Monolithic => vec![module],
        SplitStrategy::ByFunction => split_by_function(module),
        SplitStrategy::BySourceFile => split_by_source_file(module),
        SplitStrategy::Auto => {
            // Check if any function has span info
            let has_spans = module.functions.iter().any(|f| f.span.is_some());
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
        let mod_id = ModuleId::new(make_id("module-split-fn", func.name.as_bytes()));
        let mut sub = AirModule::new(mod_id);
        sub.name = Some(func.name.clone());
        sub.target_pointer_width = module.target_pointer_width;
        // Copy types and constants needed by this function
        sub.types.clone_from(&module.types);
        sub.constants.clone_from(&module.constants);
        sub.add_function(func.clone());
        result.push(sub);
    }

    // Globals go into the first module
    if !module.globals.is_empty() {
        if let Some(first) = result.first_mut() {
            first.globals.clone_from(&module.globals);
        }
    }

    result
}

fn split_by_source_file(module: AirModule) -> Vec<AirModule> {
    // Build file_id -> source_file path mapping
    let file_paths: BTreeMap<crate::ids::FileId, String> = module
        .source_files
        .iter()
        .map(|sf| (sf.id, sf.path.clone()))
        .collect();

    // Group functions by their source file
    let mut groups: BTreeMap<String, Vec<AirFunction>> = BTreeMap::new();
    for func in &module.functions {
        let key = func
            .span
            .as_ref()
            .and_then(|s| file_paths.get(&s.file_id))
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
        let mod_id = ModuleId::new(make_id("module-split-src", path.as_bytes()));
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
            first.globals.clone_from(&module.globals);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::{AirBundle, AirFunction, AirGlobal, AirModule, Constant};
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
            g.init = Some(Constant::Int { value: 0, bits: 32 });
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
        let bundle = make_bundle(
            "main",
            vec![
                make_function("main", false),
                make_function("printf", true), // extern
            ],
        );
        let program = AirProgram::link(vec![bundle]);

        assert_eq!(program.modules.len(), 1);
        // No cross-module resolution possible with single module
        assert!(program.link_table.function_resolutions.is_empty());
    }

    #[test]
    fn link_resolves_extern_functions() {
        let main_bundle = make_bundle(
            "main",
            vec![
                make_function("main", false),
                make_function("helper", true), // extern declaration
            ],
        );
        let lib_bundle = make_bundle(
            "lib",
            vec![
                make_function("helper", false), // definition
            ],
        );

        let program = AirProgram::link(vec![main_bundle, lib_bundle]);

        assert_eq!(program.modules.len(), 2);
        // helper declaration in main -> helper definition in lib
        assert_eq!(program.link_table.function_resolutions.len(), 1);

        let decl_id = FunctionId::new(make_id("fn", b"helper"));
        assert!(
            program
                .link_table
                .function_resolutions
                .contains_key(&decl_id)
        );
    }

    #[test]
    fn link_unresolved_extern_stays_unresolved() {
        let bundle = make_bundle(
            "main",
            vec![
                make_function("main", false),
                make_function("printf", true), // libc — no definition anywhere
            ],
        );
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
        let main_bundle = make_bundle(
            "main",
            vec![
                make_function("main", false),
                make_function("helper", true), // extern
            ],
        );
        let lib_bundle = make_bundle(
            "lib",
            vec![
                make_function("helper", false), // definition
            ],
        );

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
        let bundle = make_bundle(
            "main",
            vec![
                make_function("main", false),
                make_function("printf", true), // extern, no definition
            ],
        );

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

    #[test]
    fn split_monolithic_returns_original() {
        let module = make_module(
            "test",
            vec![make_function("a", false), make_function("b", false)],
        );
        let result = split_module(module, SplitStrategy::Monolithic);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].functions.len(), 2);
    }

    #[test]
    fn split_by_function_creates_one_module_per_function() {
        let module = make_module(
            "test",
            vec![
                make_function("a", false),
                make_function("b", false),
                make_function("c", false),
            ],
        );
        let result = split_module(module, SplitStrategy::ByFunction);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].functions.len(), 1);
        assert_eq!(result[0].functions[0].name, "a");
    }

    #[test]
    fn split_auto_uses_by_function_when_no_spans() {
        let module = make_module(
            "test",
            vec![make_function("a", false), make_function("b", false)],
        );
        let result = split_module(module, SplitStrategy::Auto);
        // No spans → ByFunction → 2 modules
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn split_empty_module_returns_single() {
        let module = make_module("test", vec![]);
        let result = split_module(module, SplitStrategy::ByFunction);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn split_preserves_globals_in_first_module() {
        let mut module = make_module(
            "test",
            vec![make_function("a", false), make_function("b", false)],
        );
        module.globals.push(make_global("g", true));

        let result = split_module(module, SplitStrategy::ByFunction);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].globals.len(), 1);
        assert!(result[1].globals.is_empty());
    }
}
