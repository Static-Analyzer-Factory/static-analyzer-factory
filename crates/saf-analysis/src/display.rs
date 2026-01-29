//! Human-readable display resolution for AIR entity IDs.
//!
//! Maps opaque `u128` hex IDs to human-readable labels containing names,
//! source locations, and contextual descriptions.
//!
//! # Resolution Tiers
//!
//! **Tier 1** operates purely on AIR structure (no analysis results needed):
//! functions, globals, blocks, instructions, values, source files.
//!
//! **Tier 2** uses analysis results to resolve analysis-derived IDs:
//! - `LocId` (PTA memory locations) via `PtaResult` — region-prefixed labels
//!   like `"heap@malloc:12"`, `"stack_p"`, `"global_stderr"`
//! - `SvfgNodeId` (SVFG nodes) via `Svfg` — `"svfg:%p"` or `"memphi:abcd"`
//!
//! # Architecture
//!
//! [`DisplayResolver`] wraps an [`AirLookupIndex`] (built once from an
//! [`AirModule`]) and caches resolved [`HumanLabel`]s. Resolution follows
//! a priority chain: functions -> globals -> blocks -> instructions ->
//! values -> source files -> locations -> SVFG nodes -> fallback short hex.
//!
//! # Example
//!
//! ```ignore
//! // Tier 1 only:
//! let resolver = DisplayResolver::from_module(&module);
//! let label = resolver.resolve(some_function_id);
//! println!("{} at {}", label.short_name, label.source_loc.unwrap_or_default());
//!
//! // Tier 1 + Tier 2:
//! let resolver = DisplayResolver::with_analysis(&module, Some(&pta), Some(&svfg));
//! let loc_label = resolver.resolve(some_loc_id);
//! ```

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, LocId, ValueId};
use saf_core::span::Span;
use serde::{Deserialize, Serialize};

use crate::pta::{MemoryRegion, PathStep, PtaResult};
use crate::svfg::{Svfg, SvfgNodeId};

// =============================================================================
// Public types
// =============================================================================

/// The kind of AIR entity an ID refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    /// A function definition or declaration.
    Function,
    /// A basic block.
    Block,
    /// An instruction.
    Instruction,
    /// An SSA value (parameter, instruction result, or global address).
    Value,
    /// A global variable or constant.
    Global,
    /// A global memory object (allocation backing a global).
    GlobalObj,
    /// A source file.
    SourceFile,
    /// A memory location from points-to analysis.
    Location,
    /// A node in the Sparse Value-Flow Graph.
    SvfgNode,
    /// Entity kind could not be determined.
    Unknown,
}

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Block => write!(f, "block"),
            Self::Instruction => write!(f, "instruction"),
            Self::Value => write!(f, "value"),
            Self::Global => write!(f, "global"),
            Self::GlobalObj => write!(f, "global_obj"),
            Self::SourceFile => write!(f, "source_file"),
            Self::Location => write!(f, "location"),
            Self::SvfgNode => write!(f, "svfg_node"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Source location information resolved from a [`Span`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLoc {
    /// File path (from `SourceFile.path`).
    pub file: String,
    /// Start line (1-based).
    pub line: u32,
    /// Start column (1-based).
    pub col: u32,
    /// End line (1-based), if different from start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// End column (1-based), if different from start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_col: Option<u32>,
}

impl fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.col)?;
        if let (Some(el), Some(ec)) = (self.end_line, self.end_col) {
            if el != self.line || ec != self.col {
                write!(f, "-{el}:{ec}")?;
            }
        }
        Ok(())
    }
}

/// A human-readable label for an AIR entity.
///
/// Contains both a short display name and optional context like source
/// location, full qualified name, and the containing function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanLabel {
    /// The kind of entity this label describes.
    pub kind: EntityKind,
    /// Short display name (e.g., `"main"`, `"call @printf"`, `"%p"`).
    pub short_name: String,
    /// Longer descriptive name with context (e.g., `"function main"`,
    /// `"alloca (p) in main"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_name: Option<String>,
    /// Source location, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_loc: Option<SourceLoc>,
    /// Name of the containing function, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub containing_function: Option<String>,
}

impl fmt::Display for HumanLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name)?;
        if let Some(loc) = &self.source_loc {
            write!(f, " at {loc}")?;
        }
        Ok(())
    }
}

// =============================================================================
// Internal index types
// =============================================================================

/// Describes how a `ValueId` was introduced into the program.
#[derive(Debug, Clone)]
enum ValueOrigin {
    /// Function parameter.
    Param { func_idx: usize, param_idx: usize },
    /// Destination of an instruction.
    InstructionDst {
        func_idx: usize,
        block_idx: usize,
        inst_idx: usize,
    },
    /// Address of a global variable.
    Global { global_idx: usize },
}

/// Pre-computed lookup tables for resolving AIR entity IDs to their
/// structural context within an `AirModule`.
///
/// Built once via [`AirLookupIndex::build`] and stored inside
/// [`DisplayResolver`]. Not intended for direct public use.
struct AirLookupIndex {
    /// Global value ID -> index into `AirModule.globals`.
    globals: BTreeMap<u128, usize>,
    /// Global object ID -> index into `AirModule.globals`.
    global_objs: BTreeMap<u128, usize>,
    /// Block ID -> (function index, block index within function).
    blocks: BTreeMap<u128, (usize, usize)>,
    /// Instruction ID -> (function index, block index, instruction index).
    instructions: BTreeMap<u128, (usize, usize, usize)>,
    /// Value ID -> how it was introduced.
    values: BTreeMap<u128, ValueOrigin>,
    /// File ID -> index into `AirModule.source_files`.
    files: BTreeMap<u128, usize>,
}

impl AirLookupIndex {
    /// Build lookup index from an `AirModule` in a single pass over all
    /// functions, blocks, instructions, parameters, globals, and source files.
    fn build(module: &AirModule) -> Self {
        let mut globals = BTreeMap::new();
        let mut global_objs = BTreeMap::new();
        let mut blocks = BTreeMap::new();
        let mut instructions = BTreeMap::new();
        let mut values = BTreeMap::new();
        let mut files = BTreeMap::new();

        // Index globals.
        for (gi, global) in module.globals.iter().enumerate() {
            globals.insert(global.id.raw(), gi);
            global_objs.insert(global.obj.raw(), gi);
            values.insert(global.id.raw(), ValueOrigin::Global { global_idx: gi });
        }

        // Index functions, blocks, instructions, and parameters.
        for (fi, func) in module.functions.iter().enumerate() {
            // Index parameters.
            for (pi, param) in func.params.iter().enumerate() {
                values.insert(
                    param.id.raw(),
                    ValueOrigin::Param {
                        func_idx: fi,
                        param_idx: pi,
                    },
                );
            }

            // Index blocks and their instructions.
            for (bi, block) in func.blocks.iter().enumerate() {
                blocks.insert(block.id.raw(), (fi, bi));

                for (ii, inst) in block.instructions.iter().enumerate() {
                    instructions.insert(inst.id.raw(), (fi, bi, ii));

                    if let Some(dst) = inst.dst {
                        values.insert(
                            dst.raw(),
                            ValueOrigin::InstructionDst {
                                func_idx: fi,
                                block_idx: bi,
                                inst_idx: ii,
                            },
                        );
                    }
                }
            }
        }

        // Index source files.
        for (si, sf) in module.source_files.iter().enumerate() {
            files.insert(sf.id.raw(), si);
        }

        Self {
            globals,
            global_objs,
            blocks,
            instructions,
            values,
            files,
        }
    }
}

// =============================================================================
// DisplayResolver
// =============================================================================

/// Resolves AIR entity IDs to human-readable labels.
///
/// Tier 1 resolution operates purely on the `AirModule` structure:
/// function names, block labels, instruction operations, parameter names,
/// global names, and source file paths.
///
/// Resolved labels are cached so that repeated lookups for the same ID
/// return the same result without redundant work.
///
/// # Construction
///
/// ```ignore
/// let resolver = DisplayResolver::from_module(&module);
/// let label = resolver.resolve(function_id.raw());
/// ```
pub struct DisplayResolver<'a> {
    /// Reference to the AIR module for data access.
    module: &'a AirModule,
    /// Pre-computed index for fast lookup.
    index: AirLookupIndex,
    /// Cache of already-resolved labels.
    cache: RefCell<BTreeMap<u128, HumanLabel>>,

    // ----- Tier 2: analysis-derived resolution -----
    /// Reverse map: `LocId` -> set of `ValueId`s whose points-to set includes it.
    /// Built from `PtaResult` when `with_analysis()` is used.
    loc_to_values: BTreeMap<u128, Vec<ValueId>>,
    /// PTA result for location metadata (region, field path).
    pta_result: Option<&'a PtaResult>,
    /// SVFG for node resolution.
    svfg: Option<&'a Svfg>,
}

impl<'a> DisplayResolver<'a> {
    /// Create a `DisplayResolver` from an `AirModule`.
    ///
    /// Builds the internal lookup index in a single pass over the module.
    /// This provides Tier 1 resolution only (AIR structure). For analysis-
    /// derived resolution (locations, SVFG nodes), use [`with_analysis`].
    #[must_use]
    pub fn from_module(module: &'a AirModule) -> Self {
        let index = AirLookupIndex::build(module);
        Self {
            module,
            index,
            cache: RefCell::new(BTreeMap::new()),
            loc_to_values: BTreeMap::new(),
            pta_result: None,
            svfg: None,
        }
    }

    /// Create a `DisplayResolver` with analysis results for Tier 2 resolution.
    ///
    /// Builds a reverse map from `LocId` to `ValueId`s by inverting the
    /// `PtaResult`'s points-to map: for each `(value_id, loc_set)`, each
    /// `LocId` in the set is mapped back to the value that "owns" it.
    ///
    /// This enables resolution of:
    /// - `LocId` → location description with region and variable context
    /// - `SvfgNodeId` → value or `MemPhi` description
    #[must_use]
    pub fn with_analysis(
        module: &'a AirModule,
        pta_result: Option<&'a PtaResult>,
        svfg: Option<&'a Svfg>,
    ) -> Self {
        let index = AirLookupIndex::build(module);
        let loc_to_values = Self::build_loc_to_values(pta_result);

        Self {
            module,
            index,
            cache: RefCell::new(BTreeMap::new()),
            loc_to_values,
            pta_result,
            svfg,
        }
    }

    /// Build the reverse map from `LocId` -> owning `ValueId`s.
    fn build_loc_to_values(pta_result: Option<&PtaResult>) -> BTreeMap<u128, Vec<ValueId>> {
        let mut map: BTreeMap<u128, Vec<ValueId>> = BTreeMap::new();
        let Some(pta) = pta_result else {
            return map;
        };
        for (value_id, loc_set) in pta.points_to_map() {
            for loc_id in loc_set {
                map.entry(loc_id.raw()).or_default().push(*value_id);
            }
        }
        map
    }

    /// Resolve a raw `u128` ID to a human-readable label.
    ///
    /// Resolution priority:
    /// 1. Functions (via `AirModule.function_index`)
    /// 2. Globals (by value ID)
    /// 3. Global objects (by object ID)
    /// 4. Blocks
    /// 5. Instructions
    /// 6. Values (params, instruction destinations)
    /// 7. Source files
    /// 8. Fallback: unknown entity with short hex
    #[must_use]
    pub fn resolve(&self, id: u128) -> HumanLabel {
        // Check cache first.
        if let Some(cached) = self.cache.borrow().get(&id) {
            return cached.clone();
        }

        let label = self.resolve_uncached(id);

        // Insert into cache.
        self.cache.borrow_mut().insert(id, label.clone());
        label
    }

    /// Resolve without checking or populating the cache.
    fn resolve_uncached(&self, id: u128) -> HumanLabel {
        // 1. Try function.
        if let Some(label) = self.try_resolve_function(id) {
            return label;
        }

        // 2. Try global (by value ID).
        if let Some(label) = self.try_resolve_global(id) {
            return label;
        }

        // 3. Try global object (by ObjId).
        if let Some(label) = self.try_resolve_global_obj(id) {
            return label;
        }

        // 4. Try block.
        if let Some(label) = self.try_resolve_block(id) {
            return label;
        }

        // 5. Try instruction.
        if let Some(label) = self.try_resolve_instruction(id) {
            return label;
        }

        // 6. Try value (param or instruction dst — only if not already
        //    resolved as instruction above by InstId).
        if let Some(label) = self.try_resolve_value(id) {
            return label;
        }

        // 7. Try source file.
        if let Some(label) = self.try_resolve_file(id) {
            return label;
        }

        // --- Tier 2: analysis-derived resolution ---

        // 8. Try PTA location (LocId).
        if let Some(label) = self.try_resolve_location(id) {
            return label;
        }

        // 9. Try SVFG node.
        if let Some(label) = self.try_resolve_svfg_node(id) {
            return label;
        }

        // 10. Fallback.
        HumanLabel {
            kind: EntityKind::Unknown,
            short_name: short_hex(id),
            long_name: None,
            source_loc: None,
            containing_function: None,
        }
    }

    // -------------------------------------------------------------------------
    // Resolution helpers
    // -------------------------------------------------------------------------

    /// Try to resolve as a function.
    fn try_resolve_function(&self, id: u128) -> Option<HumanLabel> {
        let func_id = FunctionId::new(id);
        let &fi = self.module.function_index.get(&func_id)?;
        let func = self.module.functions.get(fi)?;

        let short_name = func.name.clone();

        let long_name = if func.is_declaration {
            format!("extern function {}", func.name)
        } else {
            let block_count = func.blocks.len();
            let inst_count: usize = func.blocks.iter().map(|b| b.instructions.len()).sum();
            format!(
                "function {} ({} blocks, {} instructions)",
                func.name, block_count, inst_count
            )
        };

        let source_loc = func.span.as_ref().and_then(|s| self.resolve_span(s));

        Some(HumanLabel {
            kind: EntityKind::Function,
            short_name,
            long_name: Some(long_name),
            source_loc,
            containing_function: None,
        })
    }

    /// Try to resolve as a global variable (by `ValueId`).
    fn try_resolve_global(&self, id: u128) -> Option<HumanLabel> {
        let &gi = self.index.globals.get(&id)?;
        let global = self.module.globals.get(gi)?;

        let short_name = format!("@{}", global.name);
        let long_name = if global.is_constant {
            format!("global constant @{}", global.name)
        } else {
            format!("global variable @{}", global.name)
        };

        let source_loc = global.span.as_ref().and_then(|s| self.resolve_span(s));

        Some(HumanLabel {
            kind: EntityKind::Global,
            short_name,
            long_name: Some(long_name),
            source_loc,
            containing_function: None,
        })
    }

    /// Try to resolve as a global object (by `ObjId`).
    fn try_resolve_global_obj(&self, id: u128) -> Option<HumanLabel> {
        let &gi = self.index.global_objs.get(&id)?;
        let global = self.module.globals.get(gi)?;

        let short_name = format!("obj@{}", global.name);
        let long_name = format!("memory object for global @{}", global.name);

        let source_loc = global.span.as_ref().and_then(|s| self.resolve_span(s));

        Some(HumanLabel {
            kind: EntityKind::GlobalObj,
            short_name,
            long_name: Some(long_name),
            source_loc,
            containing_function: None,
        })
    }

    /// Try to resolve as a basic block.
    fn try_resolve_block(&self, id: u128) -> Option<HumanLabel> {
        let &(fi, bi) = self.index.blocks.get(&id)?;
        let func = self.module.functions.get(fi)?;
        let block = func.blocks.get(bi)?;

        let label_part = block.label.as_deref().unwrap_or("");

        let short_name = if label_part.is_empty() {
            format!("bb{bi}")
        } else {
            label_part.to_string()
        };

        let inst_count = block.instructions.len();
        let long_name = format!(
            "block {} in {} ({} instructions)",
            short_name, func.name, inst_count
        );

        // Use span of the first instruction in the block as the block's location.
        let source_loc = block
            .instructions
            .first()
            .and_then(|inst| inst.span.as_ref())
            .and_then(|s| self.resolve_span(s));

        Some(HumanLabel {
            kind: EntityKind::Block,
            short_name,
            long_name: Some(long_name),
            source_loc,
            containing_function: Some(func.name.clone()),
        })
    }

    /// Try to resolve as an instruction.
    fn try_resolve_instruction(&self, id: u128) -> Option<HumanLabel> {
        let &(fi, bi, ii) = self.index.instructions.get(&id)?;
        let func = self.module.functions.get(fi)?;
        let block = func.blocks.get(bi)?;
        let inst = block.instructions.get(ii)?;

        let op_str = format_operation(&inst.op, self.module);

        // If the instruction has a symbol (e.g., variable name from debug info),
        // include it in the label.
        let short_name = if let Some(sym) = &inst.symbol {
            format!("{} ({})", op_str, sym.display_name)
        } else {
            op_str
        };

        let block_label = block
            .label
            .as_deref()
            .map_or_else(|| format!("bb{bi}"), ToString::to_string);
        let long_name = format!("{} in {}:{}", short_name, func.name, block_label);

        let source_loc = inst.span.as_ref().and_then(|s| self.resolve_span(s));

        Some(HumanLabel {
            kind: EntityKind::Instruction,
            short_name,
            long_name: Some(long_name),
            source_loc,
            containing_function: Some(func.name.clone()),
        })
    }

    /// Try to resolve as a value (parameter, instruction destination, or global).
    fn try_resolve_value(&self, id: u128) -> Option<HumanLabel> {
        let origin = self.index.values.get(&id)?;

        match origin {
            ValueOrigin::Param {
                func_idx,
                param_idx,
            } => {
                let func = self.module.functions.get(*func_idx)?;
                let param = func.params.get(*param_idx)?;

                let short_name = param
                    .name
                    .as_deref()
                    .map_or_else(|| format!("%arg{}", param.index), |n| format!("%{n}"));

                let long_name = format!(
                    "parameter {} of {}",
                    param
                        .name
                        .as_deref()
                        .unwrap_or(&format!("#{}", param.index)),
                    func.name
                );

                Some(HumanLabel {
                    kind: EntityKind::Value,
                    short_name,
                    long_name: Some(long_name),
                    source_loc: func.span.as_ref().and_then(|s| self.resolve_span(s)),
                    containing_function: Some(func.name.clone()),
                })
            }

            ValueOrigin::InstructionDst {
                func_idx,
                block_idx,
                inst_idx,
            } => {
                let func = self.module.functions.get(*func_idx)?;
                let block = func.blocks.get(*block_idx)?;
                let inst = block.instructions.get(*inst_idx)?;

                let op_str = format_operation(&inst.op, self.module);

                // Prefer symbol (variable name) if available.
                let short_name = if let Some(sym) = &inst.symbol {
                    format!("%{}", sym.display_name)
                } else {
                    format!("%{}", short_hex_bare(id))
                };

                let long_name = format!("result of {} in {}", op_str, func.name);

                let source_loc = inst.span.as_ref().and_then(|s| self.resolve_span(s));

                Some(HumanLabel {
                    kind: EntityKind::Value,
                    short_name,
                    long_name: Some(long_name),
                    source_loc,
                    containing_function: Some(func.name.clone()),
                })
            }

            ValueOrigin::Global { global_idx } => {
                let global = self.module.globals.get(*global_idx)?;

                let short_name = format!("@{}", global.name);
                let long_name = format!("global @{}", global.name);
                let source_loc = global.span.as_ref().and_then(|s| self.resolve_span(s));

                Some(HumanLabel {
                    kind: EntityKind::Value,
                    short_name,
                    long_name: Some(long_name),
                    source_loc,
                    containing_function: None,
                })
            }
        }
    }

    /// Try to resolve as a source file.
    fn try_resolve_file(&self, id: u128) -> Option<HumanLabel> {
        let &si = self.index.files.get(&id)?;
        let sf = self.module.source_files.get(si)?;

        Some(HumanLabel {
            kind: EntityKind::SourceFile,
            short_name: sf.path.clone(),
            long_name: Some(format!("source file {}", sf.path)),
            source_loc: None,
            containing_function: None,
        })
    }

    // -------------------------------------------------------------------------
    // Tier 2: Analysis-derived resolution
    // -------------------------------------------------------------------------

    /// Try to resolve as a PTA memory location (`LocId`).
    ///
    /// Resolution chain: `LocId` -> `LocationFactory` for region/field info,
    /// then `loc_to_values` reverse map -> `ValueId` -> Tier 1 value index
    /// for context (variable name, containing function).
    ///
    /// Format examples:
    /// - `"heap@malloc:12"` — heap allocation at source line 12
    /// - `"stack_p"` — stack alloca for variable `p`
    /// - `"global_stderr"` — global variable location
    /// - `"loc_%a3f2"` — fallback when untraceable
    fn try_resolve_location(&self, id: u128) -> Option<HumanLabel> {
        let pta = self.pta_result?;
        let loc_id = LocId::new(id);

        // Check if this LocId exists in the PTA location factory.
        let location = pta.location(loc_id)?;
        let region = pta.region(loc_id);

        // Build a field path suffix if non-empty.
        let field_suffix = if location.path.steps.is_empty() {
            String::new()
        } else {
            location
                .path
                .steps
                .iter()
                .map(|step| match step {
                    PathStep::Field { index } => format!(".f{index}"),
                    PathStep::Index(_) => "[i]".to_string(),
                })
                .collect::<String>()
        };

        // Try to find owning values via the reverse map for context.
        let owning_values = self.loc_to_values.get(&id);

        // Try to resolve the first owning value to get a name/location.
        let (var_context, source_loc, containing_fn) = owning_values
            .and_then(|vals| vals.first())
            .and_then(|vid| {
                // Resolve the owning ValueId through Tier 1.
                let val_label = self.try_resolve_value(vid.raw())?;
                // Extract the variable/name part: strip leading % if present.
                let name = val_label
                    .short_name
                    .strip_prefix('%')
                    .unwrap_or(&val_label.short_name);
                Some((
                    name.to_string(),
                    val_label.source_loc,
                    val_label.containing_function,
                ))
            })
            .unwrap_or_default();

        let (short_name, long_name) = match region {
            MemoryRegion::Heap => {
                let src_suffix = source_loc
                    .as_ref()
                    .map_or(String::new(), |loc| format!(":{}", loc.line));
                if var_context.is_empty() {
                    let short = format!("heap@alloc{src_suffix}{field_suffix}");
                    let long = format!("heap location{src_suffix}{field_suffix}");
                    (short, long)
                } else {
                    let short = format!("heap@{var_context}{src_suffix}{field_suffix}");
                    let long = format!("heap location via {var_context}{src_suffix}{field_suffix}");
                    (short, long)
                }
            }
            MemoryRegion::Stack => {
                if var_context.is_empty() {
                    let short = format!("stack_{}{field_suffix}", short_hex_bare(id));
                    let long = format!("stack location {}{field_suffix}", short_hex_bare(id));
                    (short, long)
                } else {
                    let short = format!("stack_{var_context}{field_suffix}");
                    let long = format!("stack location {var_context}{field_suffix}");
                    (short, long)
                }
            }
            MemoryRegion::Global => {
                if var_context.is_empty() {
                    let short = format!("global_{}{field_suffix}", short_hex_bare(id));
                    let long = format!("global location {}{field_suffix}", short_hex_bare(id));
                    (short, long)
                } else {
                    let short = format!("global_{var_context}{field_suffix}");
                    let long = format!("global location {var_context}{field_suffix}");
                    (short, long)
                }
            }
            MemoryRegion::Unknown => {
                if var_context.is_empty() {
                    let short = format!("loc_{}{field_suffix}", short_hex_bare(id));
                    let long = format!("location {}{field_suffix}", short_hex_bare(id));
                    (short, long)
                } else {
                    let short = format!("loc_{var_context}{field_suffix}");
                    let long = format!("location via {var_context}{field_suffix}");
                    (short, long)
                }
            }
        };

        Some(HumanLabel {
            kind: EntityKind::Location,
            short_name,
            long_name: Some(long_name),
            source_loc,
            containing_function: containing_fn,
        })
    }

    /// Try to resolve as an SVFG node (`SvfgNodeId`).
    ///
    /// Checks if the raw ID matches any node in the SVFG. Resolves
    /// `Value(vid)` nodes by delegating to Tier 1 value resolution, and
    /// `MemPhi(access_id)` nodes with a descriptive `MemPhi` label.
    fn try_resolve_svfg_node(&self, id: u128) -> Option<HumanLabel> {
        let svfg = self.svfg?;

        // Check Value variant first: SvfgNodeId::Value(ValueId(id)).
        let value_node = SvfgNodeId::Value(ValueId::new(id));
        if svfg.contains_node(value_node) {
            // Delegate to Tier 1 value resolution for context.
            if let Some(val_label) = self.try_resolve_value(id) {
                return Some(HumanLabel {
                    kind: EntityKind::SvfgNode,
                    short_name: format!("svfg:{}", val_label.short_name),
                    long_name: Some(format!(
                        "SVFG value node {}",
                        val_label
                            .long_name
                            .as_deref()
                            .unwrap_or(&val_label.short_name)
                    )),
                    source_loc: val_label.source_loc,
                    containing_function: val_label.containing_function,
                });
            }
            // Value exists in SVFG but not in Tier 1 (e.g., global address).
            return Some(HumanLabel {
                kind: EntityKind::SvfgNode,
                short_name: format!("svfg:{}", short_hex(id)),
                long_name: Some(format!("SVFG value node {}", short_hex(id))),
                source_loc: None,
                containing_function: None,
            });
        }

        // Check MemPhi variant: SvfgNodeId::MemPhi(MemAccessId(id)).
        let mem_phi_node = SvfgNodeId::MemPhi(crate::mssa::MemAccessId::new(id));
        if svfg.contains_node(mem_phi_node) {
            return Some(HumanLabel {
                kind: EntityKind::SvfgNode,
                short_name: format!("memphi:{}", short_hex_bare(id)),
                long_name: Some(format!("SVFG MemPhi node {}", short_hex_bare(id))),
                source_loc: None,
                containing_function: None,
            });
        }

        None
    }

    // -------------------------------------------------------------------------
    // Span resolution
    // -------------------------------------------------------------------------

    /// Resolve a `Span` to a `SourceLoc` using the module's source file list.
    fn resolve_span(&self, span: &Span) -> Option<SourceLoc> {
        // Look up the source file path from FileId.
        let file_path = match self
            .index
            .files
            .get(&span.file_id.raw())
            .and_then(|&si| self.module.source_files.get(si))
        {
            Some(sf) => sf.path.clone(),
            None => format!("<file {}>", short_hex_bare(span.file_id.raw())),
        };

        // Don't produce a SourceLoc with line 0 (indicates missing info).
        if span.line_start == 0 {
            return None;
        }

        let end_line = (span.line_end != span.line_start).then_some(span.line_end);

        let end_col =
            (span.col_end != span.col_start || end_line.is_some()).then_some(span.col_end);

        Some(SourceLoc {
            file: file_path,
            line: span.line_start,
            col: span.col_start,
            end_line,
            end_col,
        })
    }
}

impl fmt::Debug for DisplayResolver<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DisplayResolver")
            .field("module_id", &self.module.id)
            .field("cached_entries", &self.cache.borrow().len())
            .finish()
    }
}

// =============================================================================
// Free functions
// =============================================================================

/// Format a `u128` ID as a short hex string with `%` prefix.
///
/// Strips leading zeros from the hex representation, e.g.,
/// `0x000000000000000000000000000012ab` becomes `"%12ab"`.
/// For zero, returns `"%0"`.
#[must_use]
pub fn short_hex(id: u128) -> String {
    format!("%{}", short_hex_bare(id))
}

/// Format a `u128` ID as a short hex string without any prefix.
///
/// Strips leading zeros, e.g. `0x00...12ab` becomes `"12ab"`.
/// For zero, returns `"0"`.
#[must_use]
fn short_hex_bare(id: u128) -> String {
    if id == 0 {
        "0".to_string()
    } else {
        format!("{id:x}")
    }
}

/// Format an `Operation` as a short human-readable string.
///
/// Mirrors the style of the existing `format_inst_short` in `saf-wasm`
/// but with slight improvements for clarity.
fn format_operation(op: &Operation, module: &AirModule) -> String {
    match op {
        Operation::CallDirect { callee } => {
            let name = module
                .function_index
                .get(callee)
                .and_then(|&fi| module.functions.get(fi))
                .map_or("?", |f| f.name.as_str());
            format!("call @{name}")
        }
        Operation::CallIndirect { .. } => "call indirect".to_string(),
        Operation::Alloca { size_bytes } => {
            if let Some(sz) = size_bytes {
                format!("alloca ({sz}B)")
            } else {
                "alloca".to_string()
            }
        }
        Operation::HeapAlloc { kind } => format!("{kind}"),
        Operation::Gep { field_path } => {
            if field_path.steps.is_empty() {
                "gep".to_string()
            } else {
                let path_str: String = field_path
                    .steps
                    .iter()
                    .map(|step| match step {
                        saf_core::air::FieldStep::Field { index } => format!(".f{index}"),
                        saf_core::air::FieldStep::Index => "[i]".to_string(),
                    })
                    .collect();
                format!("gep{path_str}")
            }
        }
        Operation::Load => "load".to_string(),
        Operation::Store => "store".to_string(),
        Operation::Phi { .. } => "phi".to_string(),
        Operation::BinaryOp { kind } => format!("{kind:?}"),
        Operation::Cast { kind, .. } => format!("{kind:?}"),
        Operation::Select => "select".to_string(),
        Operation::Ret => "ret".to_string(),
        Operation::Br { .. } => "br".to_string(),
        Operation::CondBr { .. } => "condbr".to_string(),
        Operation::Switch { .. } => "switch".to_string(),
        Operation::Unreachable => "unreachable".to_string(),
        Operation::Copy => "copy".to_string(),
        Operation::Freeze => "freeze".to_string(),
        Operation::Memcpy => "memcpy".to_string(),
        Operation::Memset => "memset".to_string(),
        Operation::Global { .. } => "global".to_string(),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirGlobal, AirParam, Instruction, Operation};
    use saf_core::ids::{BlockId, FileId, FunctionId, InstId, ModuleId, ObjId, ValueId};
    use saf_core::span::{SourceFile, Span, Symbol};

    /// Build a minimal test module with one function, one global, and one source file.
    fn build_test_module() -> AirModule {
        let file_id = FileId::new(100);
        let func_id = FunctionId::new(1);
        let block_id = BlockId::new(10);
        let block2_id = BlockId::new(11);
        let inst_alloca_id = InstId::new(20);
        let inst_load_id = InstId::new(21);
        let inst_call_id = InstId::new(22);
        let inst_ret_id = InstId::new(23);
        let param_id = ValueId::new(30);
        let alloca_dst = ValueId::new(31);
        let load_dst = ValueId::new(32);
        let call_dst = ValueId::new(33);
        let global_val_id = ValueId::new(40);
        let global_obj_id = ObjId::new(41);
        let callee_id = FunctionId::new(2);

        // Source file.
        let source_file = SourceFile::new(file_id, "main.c");

        // Span for the function.
        let func_span = Span::new(file_id, 0, 100, 1, 1, 10, 1);

        // Build instructions.
        let inst_alloca = Instruction::new(
            inst_alloca_id,
            Operation::Alloca {
                size_bytes: Some(8),
            },
        )
        .with_dst(alloca_dst)
        .with_span(Span::new(file_id, 10, 20, 3, 5, 3, 20))
        .with_symbol(Symbol::simple("p"));

        let inst_load = Instruction::new(inst_load_id, Operation::Load)
            .with_dst(load_dst)
            .with_span(Span::new(file_id, 20, 30, 4, 5, 4, 15));

        let inst_call = Instruction::new(inst_call_id, Operation::CallDirect { callee: callee_id })
            .with_dst(call_dst)
            .with_span(Span::new(file_id, 30, 40, 5, 5, 5, 25));

        let inst_ret = Instruction::new(inst_ret_id, Operation::Ret);

        // Build blocks.
        let mut block = AirBlock::with_label(block_id, "entry");
        block.instructions = vec![inst_alloca, inst_load, inst_call];

        let mut block2 = AirBlock::new(block2_id);
        block2.instructions = vec![inst_ret];

        // Build function with parameter.
        let param = AirParam::named(param_id, 0, "argc");
        let mut func = AirFunction::new(func_id, "main");
        func.params = vec![param];
        func.span = Some(func_span);
        func.symbol = Some(Symbol::simple("main"));
        func.blocks = vec![block, block2];
        func.rebuild_block_index();

        // Build callee (declaration only).
        let mut callee = AirFunction::new(callee_id, "printf");
        callee.is_declaration = true;

        // Build global.
        let global = AirGlobal::new(global_val_id, global_obj_id, "stderr");

        // Assemble module.
        let mut module = AirModule::new(ModuleId::new(999));
        module.source_files = vec![source_file];
        module.globals = vec![global];
        module.functions = vec![func, callee];
        module.rebuild_function_index();
        module
    }

    // =========================================================================
    // Function resolution
    // =========================================================================

    #[test]
    fn resolve_function_by_id() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(1); // FunctionId::new(1) = main
        assert_eq!(label.kind, EntityKind::Function);
        assert_eq!(label.short_name, "main");
        assert!(label.long_name.as_ref().unwrap().contains("function main"));
        assert!(label.long_name.as_ref().unwrap().contains("2 blocks"));
        assert!(label.source_loc.is_some());
        let loc = label.source_loc.unwrap();
        assert_eq!(loc.file, "main.c");
        assert_eq!(loc.line, 1);
    }

    #[test]
    fn resolve_declaration_function() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(2); // FunctionId::new(2) = printf
        assert_eq!(label.kind, EntityKind::Function);
        assert_eq!(label.short_name, "printf");
        assert!(
            label
                .long_name
                .as_ref()
                .unwrap()
                .contains("extern function")
        );
    }

    // =========================================================================
    // Block resolution
    // =========================================================================

    #[test]
    fn resolve_block_with_label() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(10); // BlockId::new(10) = "entry"
        assert_eq!(label.kind, EntityKind::Block);
        assert_eq!(label.short_name, "entry");
        assert!(label.long_name.as_ref().unwrap().contains("block entry"));
        assert!(label.long_name.as_ref().unwrap().contains("in main"));
        assert!(label.long_name.as_ref().unwrap().contains("3 instructions"));
        assert_eq!(label.containing_function.as_deref(), Some("main"));
        // Source loc should come from first instruction.
        let loc = label.source_loc.unwrap();
        assert_eq!(loc.line, 3);
    }

    #[test]
    fn resolve_block_without_label() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(11); // BlockId::new(11) = no label
        assert_eq!(label.kind, EntityKind::Block);
        assert_eq!(label.short_name, "bb1");
        assert!(label.long_name.as_ref().unwrap().contains("block bb1"));
    }

    // =========================================================================
    // Instruction resolution
    // =========================================================================

    #[test]
    fn resolve_instruction_alloca_with_symbol() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(20); // InstId for alloca
        assert_eq!(label.kind, EntityKind::Instruction);
        assert!(label.short_name.contains("alloca"));
        assert!(label.short_name.contains("(p)"));
        assert!(label.long_name.as_ref().unwrap().contains("in main"));
        assert_eq!(label.containing_function.as_deref(), Some("main"));
        let loc = label.source_loc.unwrap();
        assert_eq!(loc.line, 3);
        assert_eq!(loc.col, 5);
    }

    #[test]
    fn resolve_instruction_load() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(21); // InstId for load
        assert_eq!(label.kind, EntityKind::Instruction);
        assert_eq!(label.short_name, "load");
    }

    #[test]
    fn resolve_instruction_call_direct() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(22); // InstId for call @printf
        assert_eq!(label.kind, EntityKind::Instruction);
        assert!(label.short_name.contains("call @printf"));
    }

    #[test]
    fn resolve_instruction_ret() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(23); // InstId for ret
        assert_eq!(label.kind, EntityKind::Instruction);
        assert_eq!(label.short_name, "ret");
    }

    // =========================================================================
    // Value resolution
    // =========================================================================

    #[test]
    fn resolve_value_param_with_name() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(30); // ValueId for param "argc"
        assert_eq!(label.kind, EntityKind::Value);
        assert_eq!(label.short_name, "%argc");
        assert!(label.long_name.as_ref().unwrap().contains("parameter argc"));
        assert!(label.long_name.as_ref().unwrap().contains("of main"));
        assert_eq!(label.containing_function.as_deref(), Some("main"));
    }

    #[test]
    fn resolve_value_instruction_dst_with_symbol() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(31); // ValueId for alloca dst (has symbol "p")
        assert_eq!(label.kind, EntityKind::Value);
        assert_eq!(label.short_name, "%p");
        assert!(
            label
                .long_name
                .as_ref()
                .unwrap()
                .contains("result of alloca")
        );
    }

    #[test]
    fn resolve_value_instruction_dst_without_symbol() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(32); // ValueId for load dst (no symbol)
        assert_eq!(label.kind, EntityKind::Value);
        // Should have short hex fallback.
        assert!(label.short_name.starts_with('%'));
        assert!(label.long_name.as_ref().unwrap().contains("result of load"));
    }

    #[test]
    fn resolve_value_global() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(40); // ValueId for global "stderr"
        // Globals are resolved first as EntityKind::Global by try_resolve_global,
        // since they appear in both the globals index and values index.
        assert_eq!(label.kind, EntityKind::Global);
        assert_eq!(label.short_name, "@stderr");
    }

    // =========================================================================
    // Global object resolution
    // =========================================================================

    #[test]
    fn resolve_global_obj() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(41); // ObjId for global
        assert_eq!(label.kind, EntityKind::GlobalObj);
        assert!(label.short_name.contains("obj@stderr"));
        assert!(
            label
                .long_name
                .as_ref()
                .unwrap()
                .contains("memory object for global")
        );
    }

    // =========================================================================
    // Source file resolution
    // =========================================================================

    #[test]
    fn resolve_source_file() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(100); // FileId for "main.c"
        assert_eq!(label.kind, EntityKind::SourceFile);
        assert_eq!(label.short_name, "main.c");
    }

    // =========================================================================
    // Fallback and unknown IDs
    // =========================================================================

    #[test]
    fn resolve_unknown_id() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(0xDEAD_BEEF);
        assert_eq!(label.kind, EntityKind::Unknown);
        assert_eq!(label.short_name, "%deadbeef");
        assert!(label.source_loc.is_none());
        assert!(label.containing_function.is_none());
    }

    #[test]
    fn resolve_zero_id() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label = resolver.resolve(0);
        assert_eq!(label.kind, EntityKind::Unknown);
        assert_eq!(label.short_name, "%0");
    }

    // =========================================================================
    // Cache behavior
    // =========================================================================

    #[test]
    fn resolve_caches_results() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let label1 = resolver.resolve(1);
        let label2 = resolver.resolve(1);
        assert_eq!(label1, label2);
        // Verify cache has exactly one entry for this ID.
        assert!(resolver.cache.borrow().contains_key(&1));
    }

    #[test]
    fn cache_populated_incrementally() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        assert_eq!(resolver.cache.borrow().len(), 0);
        resolver.resolve(1);
        assert_eq!(resolver.cache.borrow().len(), 1);
        resolver.resolve(10);
        assert_eq!(resolver.cache.borrow().len(), 2);
        // Resolving again should not increase cache size.
        resolver.resolve(1);
        assert_eq!(resolver.cache.borrow().len(), 2);
    }

    // =========================================================================
    // Short hex formatting
    // =========================================================================

    #[test]
    fn short_hex_strips_leading_zeros() {
        assert_eq!(short_hex(0x1234), "%1234");
        assert_eq!(short_hex(0xABCD_EF01), "%abcdef01");
        assert_eq!(short_hex(0), "%0");
        assert_eq!(short_hex(1), "%1");
    }

    // =========================================================================
    // SourceLoc display
    // =========================================================================

    #[test]
    fn source_loc_display_single_line() {
        let loc = SourceLoc {
            file: "main.c".to_string(),
            line: 42,
            col: 5,
            end_line: None,
            end_col: None,
        };
        assert_eq!(format!("{loc}"), "main.c:42:5");
    }

    #[test]
    fn source_loc_display_multi_line() {
        let loc = SourceLoc {
            file: "main.c".to_string(),
            line: 10,
            col: 1,
            end_line: Some(15),
            end_col: Some(3),
        };
        assert_eq!(format!("{loc}"), "main.c:10:1-15:3");
    }

    // =========================================================================
    // HumanLabel display
    // =========================================================================

    #[test]
    fn human_label_display_with_source() {
        let label = HumanLabel {
            kind: EntityKind::Function,
            short_name: "main".to_string(),
            long_name: None,
            source_loc: Some(SourceLoc {
                file: "main.c".to_string(),
                line: 1,
                col: 1,
                end_line: None,
                end_col: None,
            }),
            containing_function: None,
        };
        assert_eq!(format!("{label}"), "main at main.c:1:1");
    }

    #[test]
    fn human_label_display_without_source() {
        let label = HumanLabel {
            kind: EntityKind::Unknown,
            short_name: "%deadbeef".to_string(),
            long_name: None,
            source_loc: None,
            containing_function: None,
        };
        assert_eq!(format!("{label}"), "%deadbeef");
    }

    // =========================================================================
    // EntityKind display
    // =========================================================================

    #[test]
    fn entity_kind_display() {
        assert_eq!(format!("{}", EntityKind::Function), "function");
        assert_eq!(format!("{}", EntityKind::Block), "block");
        assert_eq!(format!("{}", EntityKind::Instruction), "instruction");
        assert_eq!(format!("{}", EntityKind::Value), "value");
        assert_eq!(format!("{}", EntityKind::Global), "global");
        assert_eq!(format!("{}", EntityKind::GlobalObj), "global_obj");
        assert_eq!(format!("{}", EntityKind::SourceFile), "source_file");
        assert_eq!(format!("{}", EntityKind::Location), "location");
        assert_eq!(format!("{}", EntityKind::SvfgNode), "svfg_node");
        assert_eq!(format!("{}", EntityKind::Unknown), "unknown");
    }

    // =========================================================================
    // Span resolution edge cases
    // =========================================================================

    #[test]
    fn resolve_span_with_zero_line_returns_none() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let span = Span::new(FileId::new(100), 0, 10, 0, 0, 0, 0);
        assert!(resolver.resolve_span(&span).is_none());
    }

    #[test]
    fn resolve_span_unknown_file() {
        let module = build_test_module();
        let resolver = DisplayResolver::from_module(&module);

        let span = Span::new(FileId::new(999), 0, 10, 5, 1, 5, 10);
        let loc = resolver.resolve_span(&span).unwrap();
        // Unknown file ID falls back to <file XXXX>.
        assert!(loc.file.starts_with("<file "));
        assert_eq!(loc.line, 5);
    }

    // =========================================================================
    // Operation formatting
    // =========================================================================

    #[test]
    fn format_operation_coverage() {
        let module = AirModule::new(ModuleId::new(0));
        // Just test that all operation variants produce non-empty strings.
        let ops: Vec<Operation> = vec![
            Operation::Alloca { size_bytes: None },
            Operation::Alloca {
                size_bytes: Some(16),
            },
            Operation::Load,
            Operation::Store,
            Operation::Gep {
                field_path: saf_core::air::FieldPath::empty(),
            },
            Operation::Gep {
                field_path: saf_core::air::FieldPath::field(2),
            },
            Operation::Memcpy,
            Operation::Memset,
            Operation::Phi { incoming: vec![] },
            Operation::Select,
            Operation::Ret,
            Operation::Br {
                target: BlockId::new(0),
            },
            Operation::CondBr {
                then_target: BlockId::new(0),
                else_target: BlockId::new(1),
            },
            Operation::Switch {
                default: BlockId::new(0),
                cases: vec![],
            },
            Operation::Unreachable,
            Operation::Copy,
            Operation::Freeze,
            Operation::CallIndirect {
                expected_signature: None,
            },
            Operation::Global { obj: ObjId::new(0) },
        ];
        for op in &ops {
            let s = format_operation(op, &module);
            assert!(!s.is_empty(), "empty format for {op:?}");
        }
    }

    #[test]
    fn format_operation_gep_with_path() {
        let module = AirModule::new(ModuleId::new(0));
        let op = Operation::Gep {
            field_path: saf_core::air::FieldPath {
                steps: vec![
                    saf_core::air::FieldStep::Field { index: 0 },
                    saf_core::air::FieldStep::Index,
                ],
            },
        };
        let s = format_operation(&op, &module);
        assert_eq!(s, "gep.f0[i]");
    }

    // =========================================================================
    // Param without name
    // =========================================================================

    #[test]
    fn resolve_param_without_name() {
        let file_id = FileId::new(200);
        let func_id = FunctionId::new(50);
        let param_id = ValueId::new(60);

        let param = AirParam::new(param_id, 0);
        let mut func = AirFunction::new(func_id, "foo");
        func.params = vec![param];

        let mut module = AirModule::new(ModuleId::new(999));
        module.source_files = vec![SourceFile::new(file_id, "test.c")];
        module.functions = vec![func];
        module.rebuild_function_index();

        let resolver = DisplayResolver::from_module(&module);
        let label = resolver.resolve(60);
        assert_eq!(label.kind, EntityKind::Value);
        assert_eq!(label.short_name, "%arg0");
        assert!(label.long_name.as_ref().unwrap().contains("parameter #0"));
    }

    // =========================================================================
    // Tier 2: Location resolution (LocId via PtaResult)
    // =========================================================================

    /// Build a minimal PtaResult with one value -> one location mapping.
    fn build_pta_with_location(
        value_id: ValueId,
        obj_id: ObjId,
        region: crate::pta::MemoryRegion,
    ) -> (crate::pta::PtaResult, LocId) {
        use crate::pta::PointsToMap;
        use crate::pta::PtaDiagnostics;
        use crate::pta::{FieldPath, LocationFactory};
        use std::collections::BTreeSet;
        use std::sync::Arc;

        let mut factory =
            LocationFactory::new(crate::pta::FieldSensitivity::StructFields { max_depth: 2 });
        let loc_id = factory.get_or_create(obj_id, FieldPath::empty());
        factory.set_region(obj_id, region);

        let mut pts_map = PointsToMap::new();
        let mut loc_set = BTreeSet::new();
        loc_set.insert(loc_id);
        pts_map.insert(value_id, loc_set);

        let pta = crate::pta::PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());
        (pta, loc_id)
    }

    #[test]
    fn resolve_location_stack_with_variable_name() {
        let module = build_test_module();
        // alloca_dst (ValueId 31) has symbol "p" in the test module.
        let (pta, loc_id) = build_pta_with_location(
            ValueId::new(31),
            ObjId::new(500),
            crate::pta::MemoryRegion::Stack,
        );

        let resolver = DisplayResolver::with_analysis(&module, Some(&pta), None);
        let label = resolver.resolve(loc_id.raw());

        assert_eq!(label.kind, EntityKind::Location);
        assert_eq!(label.short_name, "stack_p");
        assert!(
            label
                .long_name
                .as_ref()
                .unwrap()
                .contains("stack location p")
        );
    }

    #[test]
    fn resolve_location_heap() {
        let module = build_test_module();
        // call_dst (ValueId 33) = result of call @printf. Use it as heap owner.
        let (pta, loc_id) = build_pta_with_location(
            ValueId::new(33),
            ObjId::new(600),
            crate::pta::MemoryRegion::Heap,
        );

        let resolver = DisplayResolver::with_analysis(&module, Some(&pta), None);
        let label = resolver.resolve(loc_id.raw());

        assert_eq!(label.kind, EntityKind::Location);
        // The owning value is call_dst with no symbol, so it uses short hex.
        // The source loc from line 5 yields ":5".
        assert!(label.short_name.starts_with("heap@"));
        assert!(label.short_name.contains(":5"));
    }

    #[test]
    fn resolve_location_global() {
        let module = build_test_module();
        // global_val_id (ValueId 40) = @stderr.
        let (pta, loc_id) = build_pta_with_location(
            ValueId::new(40),
            ObjId::new(700),
            crate::pta::MemoryRegion::Global,
        );

        let resolver = DisplayResolver::with_analysis(&module, Some(&pta), None);
        let label = resolver.resolve(loc_id.raw());

        assert_eq!(label.kind, EntityKind::Location);
        assert_eq!(label.short_name, "global_@stderr");
        assert!(
            label
                .long_name
                .as_ref()
                .unwrap()
                .contains("global location @stderr")
        );
    }

    #[test]
    fn resolve_location_unknown_region_no_owner() {
        let module = build_test_module();

        // Create a PTA with a location but no owning value in the points-to map.
        use crate::pta::PointsToMap;
        use crate::pta::PtaDiagnostics;
        use crate::pta::{FieldPath, LocationFactory};
        use std::sync::Arc;

        let mut factory =
            LocationFactory::new(crate::pta::FieldSensitivity::StructFields { max_depth: 2 });
        let loc_id = factory.get_or_create(ObjId::new(800), FieldPath::empty());
        // No region set → defaults to Unknown.

        let pta = crate::pta::PtaResult::new(
            PointsToMap::new(), // empty — no owning value
            Arc::new(factory),
            PtaDiagnostics::default(),
        );

        let resolver = DisplayResolver::with_analysis(&module, Some(&pta), None);
        let label = resolver.resolve(loc_id.raw());

        assert_eq!(label.kind, EntityKind::Location);
        // Fallback format: "loc_<hex>"
        assert!(label.short_name.starts_with("loc_"));
    }

    #[test]
    fn resolve_location_not_in_pta_falls_through() {
        let module = build_test_module();

        // Create a PTA with no locations at all.
        use crate::pta::LocationFactory;
        use crate::pta::PointsToMap;
        use crate::pta::PtaDiagnostics;
        use std::sync::Arc;

        let factory =
            LocationFactory::new(crate::pta::FieldSensitivity::StructFields { max_depth: 2 });
        let pta = crate::pta::PtaResult::new(
            PointsToMap::new(),
            Arc::new(factory),
            PtaDiagnostics::default(),
        );

        let resolver = DisplayResolver::with_analysis(&module, Some(&pta), None);
        // Use an ID that is not a LocId in the factory.
        let label = resolver.resolve(0xCAFE_BABE);
        assert_eq!(label.kind, EntityKind::Unknown);
    }

    // =========================================================================
    // Tier 2: SVFG node resolution
    // =========================================================================

    #[test]
    fn resolve_svfg_value_node_unknown_to_tier1() {
        let module = build_test_module();

        let mut svfg = crate::svfg::Svfg::new();
        // Use an ID not in Tier 1 so SVFG resolution is reached.
        svfg.add_node(crate::svfg::SvfgNodeId::Value(ValueId::new(0xBEEF)));

        let resolver = DisplayResolver::with_analysis(&module, None, Some(&svfg));
        let label = resolver.resolve(0xBEEF);

        // SVFG value node with no Tier 1 context uses hex fallback.
        assert_eq!(label.kind, EntityKind::SvfgNode);
        assert!(label.short_name.starts_with("svfg:"));
        assert!(label.short_name.contains("%beef"));
        assert!(
            label
                .long_name
                .as_ref()
                .unwrap()
                .contains("SVFG value node")
        );
    }

    #[test]
    fn resolve_svfg_memphi_node() {
        let module = build_test_module();

        let mut svfg = crate::svfg::Svfg::new();
        let access_id = crate::mssa::MemAccessId::new(0xABCD);
        svfg.add_node(crate::svfg::SvfgNodeId::MemPhi(access_id));

        let resolver = DisplayResolver::with_analysis(&module, None, Some(&svfg));
        let label = resolver.resolve(0xABCD);

        assert_eq!(label.kind, EntityKind::SvfgNode);
        assert!(label.short_name.starts_with("memphi:"));
        assert!(label.short_name.contains("abcd"));
        assert!(label.long_name.as_ref().unwrap().contains("MemPhi"));
    }

    #[test]
    fn resolve_svfg_node_not_in_graph_falls_through() {
        let module = build_test_module();

        let svfg = crate::svfg::Svfg::new(); // empty graph
        let resolver = DisplayResolver::with_analysis(&module, None, Some(&svfg));

        // Unknown ID should fall through SVFG resolution.
        let label = resolver.resolve(0xDEAD_BEEF);
        assert_eq!(label.kind, EntityKind::Unknown);
    }

    // =========================================================================
    // Tier 1 still works via with_analysis
    // =========================================================================

    #[test]
    fn with_analysis_preserves_tier1_resolution() {
        let module = build_test_module();

        // Even with analysis, Tier 1 IDs resolve correctly.
        let resolver = DisplayResolver::with_analysis(&module, None, None);
        let label = resolver.resolve(1); // FunctionId::new(1) = main
        assert_eq!(label.kind, EntityKind::Function);
        assert_eq!(label.short_name, "main");
    }

    // =========================================================================
    // Tier 1 takes priority over Tier 2
    // =========================================================================

    #[test]
    fn tier1_takes_priority_over_svfg() {
        let module = build_test_module();

        let mut svfg = crate::svfg::Svfg::new();
        // ValueId 31 (alloca dst "p") exists in both Tier 1 and SVFG.
        svfg.add_node(crate::svfg::SvfgNodeId::Value(ValueId::new(31)));

        let resolver = DisplayResolver::with_analysis(&module, None, Some(&svfg));
        let label = resolver.resolve(31);

        // Tier 1 value resolution should win (EntityKind::Value, not SvfgNode)
        // because Tier 1 is checked first.
        assert_eq!(label.kind, EntityKind::Value);
        assert_eq!(label.short_name, "%p");
    }
}
