//! Site classifier: walks AIR instructions and matches call sites to
//! resource table roles, producing `ClassifiedSites`.
//!
//! Given a `ResourceTable` and an `AirModule`, the classifier scans all
//! instructions for `CallDirect` operations and looks up the callee name
//! in the resource table. Each matched call site is recorded with the
//! SVFG node IDs for the call's return value and arguments.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, BinaryOp, Operation};
use saf_core::ids::{BlockId, FunctionId, ValueId};
use saf_core::saf_log;

use crate::svfg::{Svfg, SvfgNodeId};

use super::resource_table::{ResourceRole, ResourceTable};

// ---------------------------------------------------------------------------
// ClassifiedSite
// ---------------------------------------------------------------------------

/// A call site classified by its resource role(s).
#[derive(Debug, Clone)]
pub struct ClassifiedSite {
    /// The SVFG node for the call's return value (if any).
    pub return_value: Option<SvfgNodeId>,
    /// The SVFG nodes for the call's arguments (index → node).
    pub arguments: Vec<SvfgNodeId>,
    /// The roles this call site fulfills.
    pub roles: BTreeSet<ResourceRole>,
    /// The callee function name.
    pub callee_name: String,
    /// The function containing this call site.
    pub containing_function: FunctionId,
}

// ---------------------------------------------------------------------------
// ClassifiedSites
// ---------------------------------------------------------------------------

/// Collection of all classified call sites from a module.
#[derive(Debug, Clone)]
pub struct ClassifiedSites {
    /// All classified sites.
    sites: Vec<ClassifiedSite>,
    /// Index: role → indices into `sites`.
    by_role: BTreeMap<ResourceRole, Vec<usize>>,
    /// Index: SVFG node (return value) → index into `sites`.
    _by_return_node: BTreeMap<SvfgNodeId, Vec<usize>>,
    /// All `Alloca` instruction destinations (for stack-escape checks).
    alloca_values: BTreeSet<SvfgNodeId>,
    /// All `Store` instruction destinations where the pointer operand may
    /// point to non-stack memory (for stack-escape checks).
    _store_to_nonstack: BTreeSet<SvfgNodeId>,
    /// All `Ret` instruction value nodes (for function-exit checks).
    ret_values: BTreeSet<SvfgNodeId>,
    /// All function exit nodes (return instructions that exist in SVFG).
    function_exits: BTreeSet<SvfgNodeId>,
    /// All `Load` instruction pointer operands (for dereference detection).
    load_deref_pointers: BTreeSet<SvfgNodeId>,
    /// All `Store` instruction pointer operands (for dereference detection).
    store_deref_pointers: BTreeSet<SvfgNodeId>,
    /// All `GEP` instruction base pointer operands (for dereference detection).
    gep_deref_pointers: BTreeSet<SvfgNodeId>,
    /// Values guarded by null checks on the not-null branch.
    /// Maps: SVFG node → true if this node is on the "not-null" path after a null check.
    null_check_guarded: BTreeSet<SvfgNodeId>,
    /// SVFG nodes representing explicit NULL constant values (null pointer assignments).
    null_source_values: BTreeSet<SvfgNodeId>,
    /// Instruction DST nodes where null is literally used as a dereference pointer
    /// (Load/Store/GEP with null operand) in a block that is NOT null-check-guarded.
    /// These represent definite null dereferences in reachable code.
    direct_null_derefs: BTreeSet<SvfgNodeId>,
}

impl ClassifiedSites {
    /// Get all classified sites.
    #[must_use]
    pub fn all(&self) -> &[ClassifiedSite] {
        &self.sites
    }

    /// Get sites with a specific role.
    #[must_use]
    pub fn with_role(&self, role: ResourceRole) -> Vec<&ClassifiedSite> {
        self.by_role
            .get(&role)
            .map(|indices| indices.iter().map(|&i| &self.sites[i]).collect())
            .unwrap_or_default()
    }

    /// Get SVFG nodes that are return values of calls with a given role.
    pub fn return_nodes_for_role(&self, role: ResourceRole) -> Vec<SvfgNodeId> {
        self.with_role(role)
            .iter()
            .filter_map(|site| site.return_value)
            .collect()
    }

    /// Get SVFG nodes that are arguments (operand 0) to calls with a given role.
    ///
    /// For deallocators like `free(ptr)`, this returns the pointer being freed.
    pub fn first_arg_nodes_for_role(&self, role: ResourceRole) -> Vec<SvfgNodeId> {
        self.with_role(role)
            .iter()
            .filter_map(|site| site.arguments.first().copied())
            .collect()
    }

    /// Get all `alloca` instruction values (stack allocations).
    #[must_use]
    pub fn alloca_values(&self) -> &BTreeSet<SvfgNodeId> {
        &self.alloca_values
    }

    /// Get all return instruction value nodes.
    #[must_use]
    pub fn ret_values(&self) -> &BTreeSet<SvfgNodeId> {
        &self.ret_values
    }

    /// Get all function exit nodes.
    #[must_use]
    pub fn function_exits(&self) -> &BTreeSet<SvfgNodeId> {
        &self.function_exits
    }

    /// Get all Load instruction pointer operands (dereference sinks).
    #[must_use]
    pub fn load_deref_pointers(&self) -> &BTreeSet<SvfgNodeId> {
        &self.load_deref_pointers
    }

    /// Get all Store instruction pointer operands (dereference sinks).
    #[must_use]
    pub fn store_deref_pointers(&self) -> &BTreeSet<SvfgNodeId> {
        &self.store_deref_pointers
    }

    /// Get all `GEP` instruction base pointer operands (dereference sinks).
    /// In SV-COMP, `GEP` on a NULL pointer is a `valid-deref` violation.
    #[must_use]
    pub fn gep_deref_pointers(&self) -> &BTreeSet<SvfgNodeId> {
        &self.gep_deref_pointers
    }

    /// Get all values guarded by null checks (on the not-null path).
    #[must_use]
    pub fn null_check_guarded(&self) -> &BTreeSet<SvfgNodeId> {
        &self.null_check_guarded
    }

    /// Get all SVFG nodes representing explicit NULL constant values (null pointer assignments).
    #[must_use]
    pub fn null_source_values(&self) -> &BTreeSet<SvfgNodeId> {
        &self.null_source_values
    }

    /// Get all instruction DST nodes where null is literally used as a dereference pointer
    /// in reachable (non-guarded) code. These are definite null-deref bugs.
    #[must_use]
    pub fn direct_null_derefs(&self) -> &BTreeSet<SvfgNodeId> {
        &self.direct_null_derefs
    }

    /// Get the number of classified sites.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sites.len()
    }

    /// Check if there are no classified sites.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }

    /// Find the `ClassifiedSite` that produced a given return-value SVFG node.
    ///
    /// Returns `None` if no site has this node as its return value.
    #[must_use]
    pub fn site_for_return_node(&self, node: SvfgNodeId) -> Option<&ClassifiedSite> {
        self.sites
            .iter()
            .find(|site| site.return_value == Some(node))
    }
}

// ---------------------------------------------------------------------------
// classify()
// ---------------------------------------------------------------------------

/// Classify all call sites in a module against a resource table.
///
/// Walks all instructions in all functions, matching `CallDirect` targets
/// against the resource table. Also collects `Alloca`, `Ret`, `Load`, `Store`,
/// and null-check sites needed by specialized checkers.
// NOTE: Site classification scans every instruction in the module once,
// collecting multiple orthogonal site categories in a single pass.
// Splitting would require multiple redundant module traversals.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn classify(module: &AirModule, table: &ResourceTable, svfg: &Svfg) -> ClassifiedSites {
    let mut sites = Vec::new();
    let mut by_role: BTreeMap<ResourceRole, Vec<usize>> = BTreeMap::new();
    let mut by_return_node: BTreeMap<SvfgNodeId, Vec<usize>> = BTreeMap::new();
    let mut alloca_values = BTreeSet::new();
    let mut ret_values = BTreeSet::new();
    let mut function_exits = BTreeSet::new();
    let mut load_deref_pointers = BTreeSet::new();
    let mut store_deref_pointers = BTreeSet::new();
    let mut gep_deref_pointers = BTreeSet::new();
    let mut null_source_values = BTreeSet::new();
    // For null-check detection: track ICmp results comparing to null
    let mut null_check_conditions: BTreeMap<ValueId, (BlockId, bool)> = BTreeMap::new(); // condition -> (block, is_eq)
    // Build function name lookup (FunctionId → name)
    let func_name_map: BTreeMap<FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    Operation::CallDirect { callee } => {
                        // Look up callee name
                        let callee_name = func_name_map.get(callee).copied().unwrap_or("<unknown>");

                        if let Some(roles) = table.lookup(callee_name) {
                            let return_node = inst.dst.map(SvfgNodeId::Value);
                            let arg_nodes: Vec<SvfgNodeId> = inst
                                .operands
                                .iter()
                                .map(|&v| SvfgNodeId::Value(v))
                                .collect();

                            let idx = sites.len();
                            sites.push(ClassifiedSite {
                                return_value: return_node,
                                arguments: arg_nodes,
                                roles: roles.clone(),
                                callee_name: callee_name.to_string(),
                                containing_function: func.id,
                            });

                            for role in roles {
                                by_role.entry(*role).or_default().push(idx);
                            }

                            if let Some(node) = return_node {
                                by_return_node.entry(node).or_default().push(idx);
                            }
                        }
                    }
                    Operation::HeapAlloc { kind } => {
                        // HeapAlloc operations are recognized heap allocations (malloc, calloc, etc.)
                        // Treat them as Allocator + NullSource role sites
                        if let Some(dst) = inst.dst {
                            let return_node = Some(SvfgNodeId::Value(dst));
                            let arg_nodes: Vec<SvfgNodeId> = inst
                                .operands
                                .iter()
                                .map(|&v| SvfgNodeId::Value(v))
                                .collect();

                            // HeapAlloc has Allocator and NullSource roles
                            let roles: BTreeSet<ResourceRole> =
                                [ResourceRole::Allocator, ResourceRole::NullSource]
                                    .into_iter()
                                    .collect();

                            let idx = sites.len();
                            sites.push(ClassifiedSite {
                                return_value: return_node,
                                arguments: arg_nodes,
                                roles: roles.clone(),
                                callee_name: kind.to_string(),
                                containing_function: func.id,
                            });

                            for role in &roles {
                                by_role.entry(*role).or_default().push(idx);
                            }

                            if let Some(node) = return_node {
                                by_return_node.entry(node).or_default().push(idx);
                            }
                        }
                    }
                    Operation::Alloca { .. } => {
                        if let Some(dst) = inst.dst {
                            let node = SvfgNodeId::Value(dst);
                            if svfg.contains_node(node) {
                                alloca_values.insert(node);
                            }
                        }
                    }
                    Operation::Ret => {
                        // Track return instructions — the value being returned (if any)
                        // and the instruction itself as a function exit.
                        if let Some(&ret_val) = inst.operands.first() {
                            let node = SvfgNodeId::Value(ret_val);
                            if svfg.contains_node(node) {
                                ret_values.insert(node);
                                function_exits.insert(node);
                            }
                        }
                        // Also track the Ret instruction dst as exit
                        if let Some(dst) = inst.dst {
                            let node = SvfgNodeId::Value(dst);
                            if svfg.contains_node(node) {
                                function_exits.insert(node);
                            }
                        }
                    }
                    Operation::Load => {
                        // Load instruction dereferences operands[0] (the pointer)
                        if let Some(&ptr_val) = inst.operands.first() {
                            let node = SvfgNodeId::Value(ptr_val);
                            if svfg.contains_node(node) {
                                load_deref_pointers.insert(node);
                            }
                        }
                    }
                    Operation::Store => {
                        // Store instruction dereferences operands[1] (the pointer)
                        // operands[0] is the value being stored
                        if let Some(&ptr_val) = inst.operands.get(1) {
                            let node = SvfgNodeId::Value(ptr_val);
                            if svfg.contains_node(node) {
                                store_deref_pointers.insert(node);
                            }
                        }
                    }
                    Operation::BinaryOp { kind } => {
                        // Detect null-check comparisons: icmp eq/ne ptr, null
                        if matches!(kind, BinaryOp::ICmpEq | BinaryOp::ICmpNe)
                            && inst.operands.len() >= 2
                        {
                            // Check if either operand is null (checking constants map)
                            let is_null_check =
                                is_null_operand(inst.operands[0], inst.operands[1], module);
                            if is_null_check {
                                if let Some(dst) = inst.dst {
                                    // Track this as a null check condition
                                    // is_eq = true for ICmpEq (then branch = null, else = not-null)
                                    // is_eq = false for ICmpNe (then branch = not-null, else = null)
                                    let is_eq = matches!(kind, BinaryOp::ICmpEq);
                                    null_check_conditions.insert(dst, (block.id, is_eq));
                                }
                            }
                        }
                    }
                    Operation::Gep { .. } => {
                        // GEP instruction: the base pointer (operands[0]) must be valid/dereferenceable
                        if let Some(&base_ptr) = inst.operands.first() {
                            let node = SvfgNodeId::Value(base_ptr);
                            if svfg.contains_node(node) {
                                gep_deref_pointers.insert(node);
                            }
                        }
                    }
                    Operation::Copy => {
                        // Detect explicit NULL assignment: %ptr = copy null
                        if let Some(dst) = inst.dst {
                            let is_null = inst.operands.first().is_some_and(|&op| {
                                matches!(
                                    module.constants.get(&op),
                                    Some(
                                        saf_core::air::Constant::Null
                                            | saf_core::air::Constant::Int { value: 0, .. }
                                    )
                                )
                            });
                            if is_null {
                                let node = SvfgNodeId::Value(dst);
                                if svfg.contains_node(node) {
                                    null_source_values.insert(node);
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // Check if the destination itself is a null constant (for any instruction type)
                if let Some(dst) = inst.dst {
                    let dst_is_null = matches!(
                        module.constants.get(&dst),
                        Some(
                            saf_core::air::Constant::Null
                                | saf_core::air::Constant::Int { value: 0, .. }
                        )
                    );
                    if dst_is_null {
                        let node = SvfgNodeId::Value(dst);
                        if svfg.contains_node(node) {
                            null_source_values.insert(node);
                        }
                    }
                }
            }
        }
    }

    // ---- Broad NULL source detection ----
    // Scan ALL SVFG Value nodes for null pointer constants. This catches null
    // regardless of instruction type (phi incoming, store operand, copy, etc.).
    // NOTE: Only match Constant::Null, not Int{0} — integer zeros in non-pointer
    // contexts (loop counters, array indices) would cause false positives.
    for node in svfg.nodes() {
        if let SvfgNodeId::Value(vid) = node {
            if matches!(
                module.constants.get(vid),
                Some(saf_core::air::Constant::Null)
            ) {
                null_source_values.insert(*node);
            }
        }
    }

    if !null_source_values.is_empty() {
        saf_log!(checker::pathsens, stats, "null source classification"; count=null_source_values.len());
    }

    // Process CondBr terminators to find null-check-guarded values
    let null_check_guarded = collect_null_check_guarded(module, &null_check_conditions, svfg);

    // Detect direct null dereferences: instructions where null is literally used
    // as the pointer operand (Load/Store/GEP) in code NOT guarded by a null check.
    let direct_null_derefs = collect_direct_null_derefs(module, &null_check_guarded, svfg);

    ClassifiedSites {
        sites,
        by_role,
        _by_return_node: by_return_node,
        alloca_values,
        _store_to_nonstack: BTreeSet::new(), // populated if PTA available
        ret_values,
        function_exits,
        load_deref_pointers,
        store_deref_pointers,
        gep_deref_pointers,
        null_check_guarded,
        null_source_values,
        direct_null_derefs,
    }
}

/// Collect SVFG nodes that are guarded by null checks.
///
/// A value is "guarded" if it's in a block that is only reachable when
/// a null check passed (i.e., the pointer is not-null).
fn collect_null_check_guarded(
    module: &AirModule,
    null_check_conditions: &BTreeMap<ValueId, (BlockId, bool)>,
    svfg: &Svfg,
) -> BTreeSet<SvfgNodeId> {
    let mut guarded = BTreeSet::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            if let Some(term) = block.terminator() {
                if let Operation::CondBr {
                    then_target,
                    else_target,
                } = &term.op
                {
                    if let Some(&cond_val) = term.operands.first() {
                        if let Some(&(_cond_block, is_eq)) = null_check_conditions.get(&cond_val) {
                            // For ICmpEq (ptr == null): else branch is not-null
                            // For ICmpNe (ptr != null): then branch is not-null
                            let not_null_block = if is_eq { *else_target } else { *then_target };

                            // Mark all values in the not-null block as guarded
                            for target_func in &module.functions {
                                for target_block in &target_func.blocks {
                                    if target_block.id == not_null_block {
                                        for target_inst in &target_block.instructions {
                                            if let Some(dst) = target_inst.dst {
                                                let node = SvfgNodeId::Value(dst);
                                                if svfg.contains_node(node) {
                                                    guarded.insert(node);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    guarded
}

/// Detect instructions that literally dereference null (Load/Store/GEP with null pointer)
/// in code that is NOT guarded by a null check. Returns DST nodes of such instructions.
fn collect_direct_null_derefs(
    module: &AirModule,
    null_check_guarded: &BTreeSet<SvfgNodeId>,
    svfg: &Svfg,
) -> BTreeSet<SvfgNodeId> {
    let mut derefs = BTreeSet::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                let ptr_is_null = match &inst.op {
                    // Load: operands[0] is the pointer
                    Operation::Load => inst
                        .operands
                        .first()
                        .is_some_and(|&op| is_null_constant(op, module)),
                    // Store: operands[1] is the pointer
                    Operation::Store => inst
                        .operands
                        .get(1)
                        .is_some_and(|&op| is_null_constant(op, module)),
                    // GEP: operands[0] is the base pointer
                    Operation::Gep { .. } => inst
                        .operands
                        .first()
                        .is_some_and(|&op| is_null_constant(op, module)),
                    _ => false,
                };

                if ptr_is_null {
                    if let Some(dst) = inst.dst {
                        let node = SvfgNodeId::Value(dst);
                        // Only report if the instruction is NOT in a guarded block
                        // (i.e., not behind a null check that makes it unreachable)
                        if svfg.contains_node(node) && !null_check_guarded.contains(&node) {
                            derefs.insert(node);
                        }
                    }
                }
            }
        }
    }

    derefs
}

/// Check if a value is a null pointer constant.
fn is_null_constant(value: ValueId, module: &AirModule) -> bool {
    matches!(
        module.constants.get(&value),
        Some(saf_core::air::Constant::Null)
    )
}

/// Check if one of the operands is a null pointer constant.
///
/// Only matches `Constant::Null` (actual pointer null), NOT integer zero.
/// Integer zero comparisons (`icmp ne i32 %x, 0`) are boolean/integer checks
/// (e.g., `if(staticTrue)`), not null pointer checks. Treating them as null
/// checks falsely marks their successor blocks as "guarded," causing both
/// false negatives (unguarded null derefs missed) and false positives
/// (non-null-check branches treated as sanitizers).
fn is_null_operand(lhs: ValueId, rhs: ValueId, module: &AirModule) -> bool {
    use saf_core::air::Constant;

    matches!(module.constants.get(&lhs), Some(Constant::Null))
        || matches!(module.constants.get(&rhs), Some(Constant::Null))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    fn make_test_module() -> AirModule {
        // Create a simple module with:
        // - main() calls malloc() and free()
        // - malloc is a declaration
        // - free is a declaration
        let malloc_id = FunctionId::new(100);
        let free_id = FunctionId::new(200);
        let main_id = FunctionId::new(300);

        let malloc_fn = {
            let mut f = AirFunction::new(malloc_id, "malloc");
            f.is_declaration = true;
            f
        };
        let free_fn = {
            let mut f = AirFunction::new(free_id, "free");
            f.is_declaration = true;
            f
        };

        // main() { ptr = malloc(10); free(ptr); return; }
        let ptr_val = ValueId::new(1);
        let size_val = ValueId::new(2);

        let malloc_call =
            Instruction::new(InstId::new(10), Operation::CallDirect { callee: malloc_id })
                .with_operands(vec![size_val])
                .with_dst(ptr_val);

        let free_call =
            Instruction::new(InstId::new(20), Operation::CallDirect { callee: free_id })
                .with_operands(vec![ptr_val]);

        let ret = Instruction::new(InstId::new(30), Operation::Ret);

        let entry_block = {
            let mut b = AirBlock::new(BlockId::new(1));
            b.instructions = vec![malloc_call, free_call, ret];
            b
        };

        let main_fn = {
            let mut f = AirFunction::new(main_id, "main");
            f.blocks = vec![entry_block];
            f
        };

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions = vec![malloc_fn, free_fn, main_fn];
        module
    }

    #[test]
    fn classify_finds_malloc_and_free() {
        let module = make_test_module();
        let table = ResourceTable::new();
        let svfg = Svfg::new(); // empty SVFG for unit test

        let classified = classify(&module, &table, &svfg);

        // Should find 2 classified sites: malloc + free
        assert_eq!(classified.len(), 2);

        let allocators = classified.with_role(ResourceRole::Allocator);
        assert_eq!(allocators.len(), 1);
        assert_eq!(allocators[0].callee_name, "malloc");

        let deallocators = classified.with_role(ResourceRole::Deallocator);
        assert_eq!(deallocators.len(), 1);
        assert_eq!(deallocators[0].callee_name, "free");
    }

    #[test]
    fn classify_returns_empty_for_no_matches() {
        let mut module = AirModule::new(ModuleId::new(1));
        let unknown_fn = FunctionId::new(999);

        let call = Instruction::new(
            InstId::new(10),
            Operation::CallDirect { callee: unknown_fn },
        );
        let ret = Instruction::new(InstId::new(20), Operation::Ret);

        let mut block = AirBlock::new(BlockId::new(1));
        block.instructions = vec![call, ret];

        let mut func = AirFunction::new(FunctionId::new(1), "main");
        func.blocks = vec![block];
        module.functions = vec![
            {
                let mut f = AirFunction::new(unknown_fn, "my_custom_func");
                f.is_declaration = true;
                f
            },
            func,
        ];

        let table = ResourceTable::new();
        let svfg = Svfg::new();

        let classified = classify(&module, &table, &svfg);
        assert!(classified.is_empty());
    }

    #[test]
    fn classify_tracks_return_values() {
        let module = make_test_module();
        let table = ResourceTable::new();
        let svfg = Svfg::new();

        let classified = classify(&module, &table, &svfg);

        let alloc_sites = classified.with_role(ResourceRole::Allocator);
        assert_eq!(alloc_sites.len(), 1);
        // malloc has a return value
        assert!(alloc_sites[0].return_value.is_some());
    }

    #[test]
    fn classify_tracks_arguments() {
        let module = make_test_module();
        let table = ResourceTable::new();
        let svfg = Svfg::new();

        let classified = classify(&module, &table, &svfg);

        let dealloc_sites = classified.with_role(ResourceRole::Deallocator);
        assert_eq!(dealloc_sites.len(), 1);
        // free(ptr) has one argument
        assert_eq!(dealloc_sites[0].arguments.len(), 1);
    }

    #[test]
    fn classify_multiple_roles_per_site() {
        let module = make_test_module();
        let table = ResourceTable::new();
        let svfg = Svfg::new();

        let classified = classify(&module, &table, &svfg);

        // malloc is both Allocator and NullSource
        let null_sources = classified.with_role(ResourceRole::NullSource);
        assert_eq!(null_sources.len(), 1);
        assert_eq!(null_sources[0].callee_name, "malloc");
    }

    #[test]
    fn return_nodes_for_role() {
        let module = make_test_module();
        let table = ResourceTable::new();
        let svfg = Svfg::new();

        let classified = classify(&module, &table, &svfg);

        let alloc_returns = classified.return_nodes_for_role(ResourceRole::Allocator);
        assert_eq!(alloc_returns.len(), 1);
    }

    #[test]
    fn first_arg_nodes_for_role() {
        let module = make_test_module();
        let table = ResourceTable::new();
        let svfg = Svfg::new();

        let classified = classify(&module, &table, &svfg);

        let dealloc_args = classified.first_arg_nodes_for_role(ResourceRole::Deallocator);
        assert_eq!(dealloc_args.len(), 1);
    }

    #[test]
    fn classify_tracks_null_source_values() {
        use saf_core::air::Constant;

        let mut module = AirModule::new(ModuleId::new(1));
        let null_val = ValueId::new(10);
        let ptr_val = ValueId::new(11);

        // ptr = copy null
        let copy_null = Instruction::new(InstId::new(100), Operation::Copy)
            .with_operands(vec![null_val])
            .with_dst(ptr_val);
        let ret = Instruction::new(InstId::new(101), Operation::Ret);

        let mut block = AirBlock::new(BlockId::new(1));
        block.instructions = vec![copy_null, ret];

        let mut func = AirFunction::new(FunctionId::new(1), "test_func");
        func.blocks = vec![block];
        module.functions = vec![func];

        // Register null_val as a Null constant
        module.constants.insert(null_val, Constant::Null);

        let table = ResourceTable::new();
        let mut svfg = Svfg::new();
        svfg.add_node(SvfgNodeId::Value(ptr_val));

        let classified = classify(&module, &table, &svfg);
        assert!(
            classified
                .null_source_values()
                .contains(&SvfgNodeId::Value(ptr_val))
        );
    }

    #[test]
    fn classify_skips_declarations() {
        // Declarations have no body — they should not be scanned
        let mut module = AirModule::new(ModuleId::new(1));
        let malloc_id = FunctionId::new(100);
        let mut malloc_fn = AirFunction::new(malloc_id, "malloc");
        malloc_fn.is_declaration = true;
        module.functions = vec![malloc_fn];

        let table = ResourceTable::new();
        let svfg = Svfg::new();

        let classified = classify(&module, &table, &svfg);
        assert!(classified.is_empty());
    }
}
