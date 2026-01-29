//! Core types for Multi-Threaded Analysis.

use saf_core::ids::{FunctionId, InstId, ValueId};
use std::collections::{BTreeMap, BTreeSet};

/// Unique identifier for a thread.
///
/// Thread IDs are assigned based on calling context:
/// - Thread 0 is always the main thread
/// - Spawned threads get IDs based on their creation call chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(pub u32);

impl ThreadId {
    /// The main thread (thread 0).
    pub const MAIN: ThreadId = ThreadId(0);

    /// Create a new thread ID.
    pub fn new(id: u32) -> Self {
        ThreadId(id)
    }

    /// Check if this is the main thread.
    pub fn is_main(&self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Display for ThreadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A callsite label in the thread context (e.g., "cs1.Call").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallsiteLabel {
    /// The label identifier (e.g., "cs1", "cs2").
    pub label: String,

    /// The function name being called.
    pub callee: String,

    /// The instruction ID of the call site.
    pub inst_id: InstId,
}

impl CallsiteLabel {
    /// Create a new callsite label.
    pub fn new(label: String, callee: String, inst_id: InstId) -> Self {
        Self {
            label,
            callee,
            inst_id,
        }
    }

    /// Format as "label.callee" (e.g., "cs1.Call").
    pub fn to_context_string(&self) -> String {
        format!("{}.{}", self.label, self.callee)
    }
}

/// Thread context representing the call chain leading to thread creation.
///
/// For example, if main calls `Call()` which then calls `pthread_create` to spawn `foo`,
/// the context would be `["cs1.Call", "cs2.foo"]`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadContext {
    /// Unique thread ID.
    pub id: ThreadId,

    /// Call chain from main to thread creation (empty for main thread).
    pub call_chain: Vec<CallsiteLabel>,

    /// The `pthread_create` call site that spawned this thread (None for main).
    pub creation_site: Option<InstId>,

    /// The thread's entry function.
    pub entry_function: FunctionId,

    /// Name of the entry function (for display).
    pub entry_function_name: String,

    /// Parent thread that spawned this thread.
    pub parent: Option<ThreadId>,
}

impl ThreadContext {
    /// Create context for the main thread.
    pub fn main(entry_function: FunctionId, entry_function_name: String) -> Self {
        Self {
            id: ThreadId::MAIN,
            call_chain: Vec::new(),
            creation_site: None,
            entry_function,
            entry_function_name,
            parent: None,
        }
    }

    /// Create context for a spawned thread.
    pub fn spawned(
        id: ThreadId,
        call_chain: Vec<CallsiteLabel>,
        creation_site: InstId,
        entry_function: FunctionId,
        entry_function_name: String,
        parent: ThreadId,
    ) -> Self {
        Self {
            id,
            call_chain,
            creation_site: Some(creation_site),
            entry_function,
            entry_function_name,
            parent: Some(parent),
        }
    }

    /// Format the context as a string (e.g., "cs1.Call,cs2.foo").
    pub fn to_context_string(&self) -> String {
        self.call_chain
            .iter()
            .map(CallsiteLabel::to_context_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Thread concurrency graph tracking thread relationships.
#[derive(Debug, Clone, Default)]
pub struct ThreadConcurrencyGraph {
    /// All discovered threads by ID.
    pub threads: BTreeMap<ThreadId, ThreadContext>,

    /// Thread pairs that may run concurrently.
    /// Key is (min_id, max_id) to ensure canonical ordering.
    concurrency: BTreeSet<(ThreadId, ThreadId)>,

    /// Join constraints: (joining_thread, joined_thread) -> join site.
    /// After a join, the threads are no longer concurrent.
    pub join_constraints: BTreeMap<(ThreadId, ThreadId), InstId>,

    /// Next thread ID to assign.
    next_id: u32,
}

impl ThreadConcurrencyGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add the main thread.
    pub fn add_main_thread(&mut self, entry_function: FunctionId, entry_function_name: String) {
        let ctx = ThreadContext::main(entry_function, entry_function_name);
        self.threads.insert(ThreadId::MAIN, ctx);
        self.next_id = 1;
    }

    /// Add a spawned thread and return its ID.
    pub fn add_thread(
        &mut self,
        call_chain: Vec<CallsiteLabel>,
        creation_site: InstId,
        entry_function: FunctionId,
        entry_function_name: String,
        parent: ThreadId,
    ) -> ThreadId {
        let id = ThreadId::new(self.next_id);
        self.next_id += 1;

        let ctx = ThreadContext::spawned(
            id,
            call_chain,
            creation_site,
            entry_function,
            entry_function_name,
            parent,
        );
        self.threads.insert(id, ctx);

        // Initially, the new thread is concurrent with its parent
        self.set_concurrent(parent, id);

        id
    }

    /// Mark two threads as potentially concurrent.
    pub fn set_concurrent(&mut self, t1: ThreadId, t2: ThreadId) {
        if t1 != t2 {
            let key = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            self.concurrency.insert(key);
        }
    }

    /// Check if two threads may run concurrently.
    pub fn may_run_concurrently(&self, t1: &ThreadId, t2: &ThreadId) -> bool {
        if t1 == t2 {
            return false;
        }
        let key = if t1 < t2 { (*t1, *t2) } else { (*t2, *t1) };
        self.concurrency.contains(&key)
    }

    /// Record a join constraint (waiting_thread waits for joined_thread).
    #[allow(clippy::similar_names)]
    pub fn add_join(
        &mut self,
        waiting_thread: ThreadId,
        joined_thread: ThreadId,
        join_site: InstId,
    ) {
        self.join_constraints
            .insert((waiting_thread, joined_thread), join_site);
    }

    /// Get thread by ID.
    pub fn get_thread(&self, id: ThreadId) -> Option<&ThreadContext> {
        self.threads.get(&id)
    }

    /// Get all thread IDs.
    pub fn thread_ids(&self) -> impl Iterator<Item = ThreadId> + '_ {
        self.threads.keys().copied()
    }

    /// Get threads that are concurrent with the given thread.
    pub fn concurrent_with(&self, id: ThreadId) -> BTreeSet<ThreadId> {
        let mut result = BTreeSet::new();
        for &(t1, t2) in &self.concurrency {
            if t1 == id {
                result.insert(t2);
            } else if t2 == id {
                result.insert(t1);
            }
        }
        result
    }

    /// Find thread by context string.
    pub fn find_by_context(&self, context: &str) -> Option<ThreadId> {
        for (id, ctx) in &self.threads {
            if ctx.to_context_string() == context {
                return Some(*id);
            }
        }
        None
    }
}

/// MHP (May-Happen-in-Parallel) result for each program point.
#[derive(Debug, Clone, Default)]
pub struct MhpResult {
    /// For each (thread_id, instruction), which threads may be concurrent.
    interleaving: BTreeMap<(ThreadId, InstId), BTreeSet<ThreadId>>,
}

impl MhpResult {
    /// Create a new empty MHP result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the concurrent threads for a program point.
    pub fn set_concurrent_at(
        &mut self,
        thread_id: ThreadId,
        inst_id: InstId,
        concurrent: BTreeSet<ThreadId>,
    ) {
        self.interleaving.insert((thread_id, inst_id), concurrent);
    }

    /// Get threads that may be concurrent at a program point.
    pub fn concurrent_at(&self, thread_id: ThreadId, inst_id: InstId) -> BTreeSet<ThreadId> {
        self.interleaving
            .get(&(thread_id, inst_id))
            .cloned()
            .unwrap_or_default()
    }

    /// Get all recorded program points.
    pub fn program_points(&self) -> impl Iterator<Item = (ThreadId, InstId)> + '_ {
        self.interleaving.keys().copied()
    }
}

/// Kind of memory access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessKind {
    /// Read access (load).
    Read,
    /// Write access (store).
    Write,
}

/// A memory access at a program point.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryAccess {
    /// The instruction performing the access.
    pub inst_id: InstId,

    /// The memory location being accessed (value ID of the pointer).
    pub location: ValueId,

    /// Thread performing the access.
    pub thread_id: ThreadId,

    /// Read or write.
    pub kind: AccessKind,

    /// Locks held at this access point (for race detection).
    pub locks_held: BTreeSet<ValueId>,
}

impl MemoryAccess {
    /// Create a new memory access.
    pub fn new(inst_id: InstId, location: ValueId, thread_id: ThreadId, kind: AccessKind) -> Self {
        Self {
            inst_id,
            location,
            thread_id,
            kind,
            locks_held: BTreeSet::new(),
        }
    }

    /// Create with lock set.
    pub fn with_locks(
        inst_id: InstId,
        location: ValueId,
        thread_id: ThreadId,
        kind: AccessKind,
        locks_held: BTreeSet<ValueId>,
    ) -> Self {
        Self {
            inst_id,
            location,
            thread_id,
            kind,
            locks_held,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::id::make_id;
    use saf_core::ids::{FunctionId, InstId};

    fn test_inst_id(name: &str) -> InstId {
        InstId(make_id("test_inst", name.as_bytes()))
    }

    fn test_func_id(name: &str) -> FunctionId {
        FunctionId(make_id("test_func", name.as_bytes()))
    }

    #[test]
    fn test_thread_id() {
        assert!(ThreadId::MAIN.is_main());
        assert!(!ThreadId::new(1).is_main());
    }

    #[test]
    fn test_thread_context_string() {
        let chain = vec![
            CallsiteLabel::new("cs1".to_string(), "Call".to_string(), test_inst_id("inst1")),
            CallsiteLabel::new("cs2".to_string(), "foo".to_string(), test_inst_id("inst2")),
        ];
        let ctx = ThreadContext::spawned(
            ThreadId::new(1),
            chain,
            test_inst_id("create"),
            test_func_id("foo_fn"),
            "foo".to_string(),
            ThreadId::MAIN,
        );
        assert_eq!(ctx.to_context_string(), "cs1.Call,cs2.foo");
    }

    #[test]
    fn test_concurrency_graph() {
        let mut graph = ThreadConcurrencyGraph::new();
        graph.add_main_thread(test_func_id("main"), "main".to_string());

        let chain = vec![
            CallsiteLabel::new("cs1".to_string(), "Call".to_string(), test_inst_id("inst1")),
            CallsiteLabel::new("cs2".to_string(), "foo".to_string(), test_inst_id("inst2")),
        ];
        let t1 = graph.add_thread(
            chain,
            test_inst_id("create"),
            test_func_id("foo_fn"),
            "foo".to_string(),
            ThreadId::MAIN,
        );

        assert!(graph.may_run_concurrently(&ThreadId::MAIN, &t1));
        assert!(!graph.may_run_concurrently(&t1, &t1));
    }
}
