//! Composable analysis pass framework.
//!
//! Provides [`AnalysisPass`], [`AnalysisContext`], and [`PassManager`] for
//! building modular analysis pipelines with automatic dependency resolution.

use std::any::Any;
use std::collections::BTreeMap;

use saf_core::air::AirModule;

use crate::error::AnalysisError;

/// Unique identifier for an analysis pass.
pub type PassId = &'static str;

// =============================================================================
// AnalysisContext
// =============================================================================

/// Accumulated results from prior analysis passes.
///
/// `AnalysisContext` acts as a type-erased result bag: each pass stores its
/// output under a [`PassId`] key and subsequent passes retrieve it via
/// [`get`](AnalysisContext::get) with the expected concrete type.
pub struct AnalysisContext {
    results: BTreeMap<PassId, Box<dyn Any + Send + Sync>>,
}

impl std::fmt::Debug for AnalysisContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisContext")
            .field("pass_ids", &self.results.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl AnalysisContext {
    /// Create an empty context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: BTreeMap::new(),
        }
    }

    /// Store a result for `pass_id`, replacing any previous value.
    pub fn insert<T: Any + Send + Sync>(&mut self, pass_id: PassId, result: T) {
        self.results.insert(pass_id, Box::new(result));
    }

    /// Retrieve a result by `pass_id`, downcasting to `T`.
    ///
    /// Returns `None` if no result is stored under `pass_id` or if the
    /// stored type does not match `T`.
    #[must_use]
    pub fn get<T: Any + Send + Sync>(&self, pass_id: PassId) -> Option<&T> {
        self.results
            .get(pass_id)
            .and_then(|v| v.downcast_ref::<T>())
    }

    /// Check whether a result exists for `pass_id`.
    #[must_use]
    pub fn has(&self, pass_id: PassId) -> bool {
        self.results.contains_key(pass_id)
    }
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// AnalysisPass trait
// =============================================================================

/// Trait for composable analysis passes.
///
/// Each pass declares a unique [`id`](AnalysisPass::id), optional
/// [`dependencies`](AnalysisPass::dependencies), and a [`run`](AnalysisPass::run)
/// method that reads from and writes to an [`AnalysisContext`].
pub trait AnalysisPass: Send + Sync {
    /// Unique identifier for this pass.
    fn id(&self) -> PassId;

    /// IDs of passes that must run before this one.
    fn dependencies(&self) -> &[PassId] {
        &[]
    }

    /// Execute the pass, reading from `ctx` and storing results back.
    fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError>;
}

// =============================================================================
// PassManager
// =============================================================================

/// Manages and executes analysis passes in dependency order.
///
/// Passes are registered via [`register`](PassManager::register) and executed
/// in topologically sorted order via [`run_all`](PassManager::run_all).
pub struct PassManager {
    passes: Vec<Box<dyn AnalysisPass>>,
}

impl PassManager {
    /// Create an empty `PassManager`.
    #[must_use]
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Register a pass. Passes are executed in dependency order regardless
    /// of registration order.
    pub fn register(&mut self, pass: Box<dyn AnalysisPass>) {
        self.passes.push(pass);
    }

    /// Topologically sort all registered passes and run them in order.
    ///
    /// Returns the [`AnalysisContext`] containing results from all passes,
    /// or an error if dependencies are missing or cyclic.
    pub fn run_all(&self, module: &AirModule) -> Result<AnalysisContext, AnalysisError> {
        let sorted = self.topological_sort()?;
        let mut ctx = AnalysisContext::new();
        for pass in sorted {
            pass.run(module, &mut ctx)?;
        }
        Ok(ctx)
    }

    /// Topological sort using Kahn's algorithm (BFS-based).
    ///
    /// Errors on missing dependencies or cycles.
    fn topological_sort(&self) -> Result<Vec<&dyn AnalysisPass>, AnalysisError> {
        // Build index: PassId -> position in self.passes
        let mut id_to_idx: BTreeMap<PassId, usize> = BTreeMap::new();
        for (i, pass) in self.passes.iter().enumerate() {
            id_to_idx.insert(pass.id(), i);
        }

        let n = self.passes.len();

        // in_degree[i] = number of dependencies for pass i
        let mut in_degree = vec![0usize; n];
        // dependents[i] = list of pass indices that depend on pass i
        let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (i, pass) in self.passes.iter().enumerate() {
            for &dep in pass.dependencies() {
                let dep_idx = id_to_idx.get(dep).ok_or_else(|| {
                    AnalysisError::Config(format!(
                        "pass `{}` depends on `{}`, which is not registered",
                        pass.id(),
                        dep
                    ))
                })?;
                in_degree[i] += 1;
                dependents[*dep_idx].push(i);
            }
        }

        // Kahn's algorithm: seed queue with passes that have no dependencies.
        // Use a BTreeMap-based queue keyed by PassId for deterministic ordering.
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();

        // Collect zero-in-degree passes and sort by id for determinism.
        let mut zero_in: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        zero_in.sort_by_key(|&i| self.passes[i].id());
        for idx in zero_in {
            queue.push_back(idx);
        }

        let mut result: Vec<&dyn AnalysisPass> = Vec::with_capacity(n);

        while let Some(idx) = queue.pop_front() {
            result.push(self.passes[idx].as_ref());

            // Collect newly freed dependents, sort for determinism.
            let mut newly_free: Vec<usize> = Vec::new();
            for &dep_idx in &dependents[idx] {
                in_degree[dep_idx] -= 1;
                if in_degree[dep_idx] == 0 {
                    newly_free.push(dep_idx);
                }
            }
            newly_free.sort_by_key(|&i| self.passes[i].id());
            for free_idx in newly_free {
                queue.push_back(free_idx);
            }
        }

        if result.len() != n {
            // Some passes remain with nonzero in-degree — cycle detected.
            let cycle_members: Vec<PassId> = (0..n)
                .filter(|&i| in_degree[i] > 0)
                .map(|i| self.passes[i].id())
                .collect();
            return Err(AnalysisError::Config(format!(
                "dependency cycle detected among passes: {}",
                cycle_members.join(", ")
            )));
        }

        Ok(result)
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ModuleId;

    fn minimal_module() -> AirModule {
        AirModule::new(ModuleId::derive(b"test"))
    }

    // -- AnalysisContext tests ------------------------------------------------

    #[test]
    fn test_context_insert_and_get() {
        let mut ctx = AnalysisContext::new();
        ctx.insert("count", 42_usize);
        let val = ctx.get::<usize>("count");
        assert_eq!(val, Some(&42_usize));
    }

    #[test]
    fn test_context_get_wrong_type() {
        let mut ctx = AnalysisContext::new();
        ctx.insert("count", 42_usize);
        let val = ctx.get::<String>("count");
        assert!(val.is_none());
    }

    #[test]
    fn test_context_has() {
        let mut ctx = AnalysisContext::new();
        assert!(!ctx.has("count"));
        ctx.insert("count", 42_usize);
        assert!(ctx.has("count"));
        assert!(!ctx.has("other"));
    }

    // -- AnalysisPass tests ---------------------------------------------------

    /// A simple pass that counts functions in the module.
    struct CountPass;

    impl AnalysisPass for CountPass {
        fn id(&self) -> PassId {
            "count"
        }

        fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
            ctx.insert(self.id(), module.functions.len());
            Ok(())
        }
    }

    #[test]
    fn test_pass_stores_and_retrieves() {
        let module = minimal_module();
        let mut ctx = AnalysisContext::new();
        let pass = CountPass;
        pass.run(&module, &mut ctx).unwrap();
        let count = ctx.get::<usize>("count");
        assert_eq!(count, Some(&0));
    }

    // -- PassManager tests ----------------------------------------------------

    /// Pass A stores a marker value.
    struct PassA;

    impl AnalysisPass for PassA {
        fn id(&self) -> PassId {
            "a"
        }

        fn run(&self, _module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
            ctx.insert(self.id(), 10_usize);
            Ok(())
        }
    }

    /// Pass B depends on pass A and reads its result.
    struct PassB;

    impl AnalysisPass for PassB {
        fn id(&self) -> PassId {
            "b"
        }

        fn dependencies(&self) -> &[PassId] {
            &["a"]
        }

        fn run(&self, _module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
            let a_val = ctx.get::<usize>("a").copied().unwrap_or(0);
            ctx.insert(self.id(), a_val + 5);
            Ok(())
        }
    }

    #[test]
    fn test_pass_manager_respects_dependencies() {
        let module = minimal_module();
        let mut mgr = PassManager::new();
        // Register B before A to verify dependency ordering.
        mgr.register(Box::new(PassB));
        mgr.register(Box::new(PassA));
        let ctx = mgr.run_all(&module).unwrap();
        assert_eq!(ctx.get::<usize>("a"), Some(&10));
        assert_eq!(ctx.get::<usize>("b"), Some(&15));
    }

    #[test]
    fn test_pass_manager_detects_missing_dependency() {
        struct NeedsMissing;

        impl AnalysisPass for NeedsMissing {
            fn id(&self) -> PassId {
                "needs_missing"
            }

            fn dependencies(&self) -> &[PassId] {
                &["nonexistent"]
            }

            fn run(
                &self,
                _module: &AirModule,
                _ctx: &mut AnalysisContext,
            ) -> Result<(), AnalysisError> {
                Ok(())
            }
        }

        let module = minimal_module();
        let mut mgr = PassManager::new();
        mgr.register(Box::new(NeedsMissing));
        let result = mgr.run_all(&module);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("nonexistent"),
            "error should mention the missing dependency: {err_msg}"
        );
    }

    #[test]
    fn test_pass_manager_detects_cycle() {
        struct CycleA;
        struct CycleB;

        impl AnalysisPass for CycleA {
            fn id(&self) -> PassId {
                "cycle_a"
            }

            fn dependencies(&self) -> &[PassId] {
                &["cycle_b"]
            }

            fn run(
                &self,
                _module: &AirModule,
                _ctx: &mut AnalysisContext,
            ) -> Result<(), AnalysisError> {
                Ok(())
            }
        }

        impl AnalysisPass for CycleB {
            fn id(&self) -> PassId {
                "cycle_b"
            }

            fn dependencies(&self) -> &[PassId] {
                &["cycle_a"]
            }

            fn run(
                &self,
                _module: &AirModule,
                _ctx: &mut AnalysisContext,
            ) -> Result<(), AnalysisError> {
                Ok(())
            }
        }

        let module = minimal_module();
        let mut mgr = PassManager::new();
        mgr.register(Box::new(CycleA));
        mgr.register(Box::new(CycleB));
        let result = mgr.run_all(&module);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cycle"),
            "error should mention cycle: {err_msg}"
        );
    }
}
