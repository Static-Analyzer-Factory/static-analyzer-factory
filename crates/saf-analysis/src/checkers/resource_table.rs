//! Resource table: maps function names to resource management roles.
//!
//! The `ResourceTable` provides a built-in mapping of common C/C++/POSIX
//! functions to their resource management roles (allocator, deallocator,
//! acquire, release, etc.). Users can extend the table with custom entries
//! via the Python API.
//!
//! A function can have multiple roles (e.g., `malloc` is both `Allocator`
//! and `NullSource`).

use std::collections::{BTreeMap, BTreeSet};

use saf_core::spec::{Nullness as SpecNullness, Role as SpecRole, SpecRegistry};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ResourceRole
// ---------------------------------------------------------------------------

/// The role a function plays in resource management.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceRole {
    /// Allocates heap memory (malloc, calloc, new, mmap).
    Allocator,
    /// Frees heap memory (free, delete, munmap).
    Deallocator,
    /// Reallocates memory — both deallocates old and allocates new (realloc).
    Reallocator,
    /// Acquires a non-memory resource (fopen, open, socket).
    Acquire,
    /// Releases a non-memory resource (fclose, close).
    Release,
    /// Acquires a lock (pthread_mutex_lock, pthread_rwlock_rdlock).
    Lock,
    /// Releases a lock (pthread_mutex_unlock, pthread_rwlock_unlock).
    Unlock,
    /// May return null (malloc, calloc, realloc, fopen, etc.).
    NullSource,
    /// Dereferences a pointer argument (memcpy, strlen, printf).
    Dereference,
}

impl ResourceRole {
    /// Get a human-readable name for this role.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Allocator => "allocator",
            Self::Deallocator => "deallocator",
            Self::Reallocator => "reallocator",
            Self::Acquire => "acquire",
            Self::Release => "release",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
            Self::NullSource => "null_source",
            Self::Dereference => "dereference",
        }
    }
}

// ---------------------------------------------------------------------------
// ResourceTable
// ---------------------------------------------------------------------------

/// Maps function names to their resource management roles.
///
/// Ships with a comprehensive built-in table covering C stdlib, C++ operators,
/// POSIX I/O, pthreads, and memory mapping. Users can add custom entries
/// via `add()`.
#[derive(Debug, Clone)]
pub struct ResourceTable {
    /// function name -> set of roles
    entries: BTreeMap<String, BTreeSet<ResourceRole>>,
}

impl ResourceTable {
    /// Create a new `ResourceTable` with built-in entries.
    #[must_use]
    pub fn new() -> Self {
        let mut table = Self {
            entries: BTreeMap::new(),
        };
        table.populate_builtins();
        table
    }

    /// Create an empty `ResourceTable` (no built-in entries).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Create a `ResourceTable` populated from function specs.
    ///
    /// First loads entries from specs, then adds built-in entries for functions
    /// not covered by specs. This allows specs to override built-in behavior.
    #[must_use]
    pub fn from_specs(specs: &SpecRegistry) -> Self {
        let mut table = Self {
            entries: BTreeMap::new(),
        };
        table.populate_from_specs(specs);
        table.populate_builtins(); // Add remaining hardcoded entries
        table
    }

    /// Populate entries from a spec registry.
    ///
    /// Maps spec roles to resource roles:
    /// - `Role::Allocator` → `Allocator` + maybe `NullSource`
    /// - `Role::Deallocator` → `Deallocator`
    /// - `Role::Reallocator` → `Reallocator` + maybe `NullSource`
    /// - `returns.nullness: maybe_null` → `NullSource`
    /// - `params[*].reads: true` → `Dereference`
    fn populate_from_specs(&mut self, specs: &SpecRegistry) {
        for spec in specs.iter() {
            let name = &spec.name;

            // Map role to ResourceRole
            if let Some(role) = &spec.role {
                match role {
                    SpecRole::Allocator => {
                        self.add(name, ResourceRole::Allocator);
                    }
                    SpecRole::Deallocator => {
                        self.add(name, ResourceRole::Deallocator);
                    }
                    SpecRole::Reallocator => {
                        self.add(name, ResourceRole::Reallocator);
                    }
                    // Other roles don't map to ResourceRole
                    _ => {}
                }
            }

            // Check for NullSource from return nullness
            if let Some(returns) = &spec.returns {
                if returns.nullness == Some(SpecNullness::MaybeNull) {
                    self.add(name, ResourceRole::NullSource);
                }
            }

            // Check for Dereference from params that read
            let has_read_param = spec.params.iter().any(|p| p.reads == Some(true));
            if has_read_param {
                self.add(name, ResourceRole::Dereference);
            }
        }
    }

    /// Add a role for a function name.
    ///
    /// A function can have multiple roles. Adding the same (name, role) pair
    /// again is a no-op.
    pub fn add(&mut self, name: &str, role: ResourceRole) {
        self.entries
            .entry(name.to_string())
            .or_default()
            .insert(role);
    }

    /// Look up all roles for a function name.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&BTreeSet<ResourceRole>> {
        self.entries.get(name)
    }

    /// Check if a function has a specific role.
    #[must_use]
    pub fn has_role(&self, name: &str, role: ResourceRole) -> bool {
        self.entries
            .get(name)
            .is_some_and(|roles| roles.contains(&role))
    }

    /// Get all function names in the table.
    #[must_use]
    pub fn function_names(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// Get the number of entries in the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the table is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Export the table as a list of (name, roles) pairs.
    #[must_use]
    pub fn export(&self) -> Vec<ResourceTableEntry> {
        self.entries
            .iter()
            .map(|(name, roles)| ResourceTableEntry {
                name: name.clone(),
                roles: roles.iter().copied().collect(),
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Built-in table population
    // -----------------------------------------------------------------------

    fn populate_builtins(&mut self) {
        self.populate_c_stdlib();
        self.populate_cpp_operators();
        self.populate_posix_io();
        self.populate_posix_threads();
        self.populate_memory_mapping();
        self.populate_common_wrappers();
        self.populate_dereference();
    }

    fn populate_c_stdlib(&mut self) {
        // Allocators (also NullSource — may return NULL)
        for name in &["malloc", "calloc", "strdup", "strndup", "aligned_alloc"] {
            self.add(name, ResourceRole::Allocator);
            self.add(name, ResourceRole::NullSource);
        }

        // realloc is special: both reallocator and null source
        self.add("realloc", ResourceRole::Reallocator);
        self.add("realloc", ResourceRole::NullSource);

        // Deallocators
        self.add("free", ResourceRole::Deallocator);
    }

    fn populate_cpp_operators(&mut self) {
        // C++ new variants — these are the mangled names LLVM emits
        for name in &[
            "_Znwm",               // operator new(size_t)
            "_Znam",               // operator new[](size_t)
            "_ZnwmRKSt9nothrow_t", // operator new(size_t, nothrow)
            "_ZnamRKSt9nothrow_t", // operator new[](size_t, nothrow)
        ] {
            self.add(name, ResourceRole::Allocator);
        }
        // nothrow variants may return null
        self.add("_ZnwmRKSt9nothrow_t", ResourceRole::NullSource);
        self.add("_ZnamRKSt9nothrow_t", ResourceRole::NullSource);

        // C++ delete variants
        for name in &[
            "_ZdlPv",  // operator delete(void*)
            "_ZdaPv",  // operator delete[](void*)
            "_ZdlPvm", // operator delete(void*, size_t)
            "_ZdaPvm", // operator delete[](void*, size_t)
        ] {
            self.add(name, ResourceRole::Deallocator);
        }
    }

    fn populate_posix_io(&mut self) {
        // File open (acquire) — may return NULL/error
        for name in &["fopen", "fdopen", "freopen", "tmpfile"] {
            self.add(name, ResourceRole::Acquire);
            self.add(name, ResourceRole::NullSource);
        }

        // File close (release)
        self.add("fclose", ResourceRole::Release);

        // File descriptor open (acquire) — return -1 on error
        for name in &[
            "open", "openat", "creat", "socket", "accept", "dup", "dup2", "pipe",
        ] {
            self.add(name, ResourceRole::Acquire);
        }

        // File descriptor close (release)
        self.add("close", ResourceRole::Release);
    }

    fn populate_posix_threads(&mut self) {
        // Mutex lock
        for name in &["pthread_mutex_lock", "pthread_mutex_trylock"] {
            self.add(name, ResourceRole::Lock);
        }

        // Mutex unlock
        self.add("pthread_mutex_unlock", ResourceRole::Unlock);

        // Read-write lock variants
        for name in &[
            "pthread_rwlock_rdlock",
            "pthread_rwlock_wrlock",
            "pthread_rwlock_tryrdlock",
            "pthread_rwlock_trywrlock",
        ] {
            self.add(name, ResourceRole::Lock);
        }

        self.add("pthread_rwlock_unlock", ResourceRole::Unlock);
    }

    fn populate_memory_mapping(&mut self) {
        self.add("mmap", ResourceRole::Allocator);
        self.add("mmap", ResourceRole::NullSource);
        self.add("munmap", ResourceRole::Deallocator);
    }

    fn populate_common_wrappers(&mut self) {
        // GLib wrappers
        for name in &["g_malloc", "g_malloc0", "g_new", "g_new0"] {
            self.add(name, ResourceRole::Allocator);
        }
        self.add("g_free", ResourceRole::Deallocator);

        // xmalloc (common in GNU projects — aborts on failure, never returns NULL)
        self.add("xmalloc", ResourceRole::Allocator);
        self.add("xcalloc", ResourceRole::Allocator);
        self.add("xrealloc", ResourceRole::Reallocator);
        self.add("xstrdup", ResourceRole::Allocator);
    }

    fn populate_dereference(&mut self) {
        // Functions that dereference pointer arguments
        for name in &[
            "memcpy", "memmove", "memset", "memcmp", "strlen", "strcpy", "strncpy", "strcat",
            "strncat", "strcmp", "strncmp", "printf", "fprintf", "sprintf", "snprintf", "puts",
            "fputs",
        ] {
            self.add(name, ResourceRole::Dereference);
        }
    }
}

impl Default for ResourceTable {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ResourceTableEntry (for export)
// ---------------------------------------------------------------------------

/// A single entry in the resource table export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTableEntry {
    /// Function name.
    pub name: String,
    /// Roles assigned to this function.
    pub roles: Vec<ResourceRole>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_table_not_empty() {
        let table = ResourceTable::new();
        assert!(!table.is_empty());
        assert!(table.len() > 20);
    }

    #[test]
    fn empty_table() {
        let table = ResourceTable::empty();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn malloc_is_allocator_and_null_source() {
        let table = ResourceTable::new();
        assert!(table.has_role("malloc", ResourceRole::Allocator));
        assert!(table.has_role("malloc", ResourceRole::NullSource));
        assert!(!table.has_role("malloc", ResourceRole::Deallocator));
    }

    #[test]
    fn free_is_deallocator() {
        let table = ResourceTable::new();
        assert!(table.has_role("free", ResourceRole::Deallocator));
        assert!(!table.has_role("free", ResourceRole::Allocator));
    }

    #[test]
    fn realloc_is_reallocator() {
        let table = ResourceTable::new();
        assert!(table.has_role("realloc", ResourceRole::Reallocator));
        assert!(table.has_role("realloc", ResourceRole::NullSource));
    }

    #[test]
    fn cpp_new_delete() {
        let table = ResourceTable::new();
        assert!(table.has_role("_Znwm", ResourceRole::Allocator));
        assert!(table.has_role("_ZdlPv", ResourceRole::Deallocator));
    }

    #[test]
    fn posix_io() {
        let table = ResourceTable::new();
        assert!(table.has_role("fopen", ResourceRole::Acquire));
        assert!(table.has_role("fopen", ResourceRole::NullSource));
        assert!(table.has_role("fclose", ResourceRole::Release));
        assert!(table.has_role("open", ResourceRole::Acquire));
        assert!(table.has_role("close", ResourceRole::Release));
    }

    #[test]
    fn posix_threads() {
        let table = ResourceTable::new();
        assert!(table.has_role("pthread_mutex_lock", ResourceRole::Lock));
        assert!(table.has_role("pthread_mutex_unlock", ResourceRole::Unlock));
    }

    #[test]
    fn mmap_munmap() {
        let table = ResourceTable::new();
        assert!(table.has_role("mmap", ResourceRole::Allocator));
        assert!(table.has_role("munmap", ResourceRole::Deallocator));
    }

    #[test]
    fn dereference_functions() {
        let table = ResourceTable::new();
        assert!(table.has_role("memcpy", ResourceRole::Dereference));
        assert!(table.has_role("strlen", ResourceRole::Dereference));
        assert!(table.has_role("printf", ResourceRole::Dereference));
    }

    #[test]
    fn add_custom_entry() {
        let mut table = ResourceTable::new();
        table.add("pool_alloc", ResourceRole::Allocator);
        table.add("pool_free", ResourceRole::Deallocator);

        assert!(table.has_role("pool_alloc", ResourceRole::Allocator));
        assert!(table.has_role("pool_free", ResourceRole::Deallocator));
    }

    #[test]
    fn add_multiple_roles_to_same_function() {
        let mut table = ResourceTable::empty();
        table.add("my_func", ResourceRole::Allocator);
        table.add("my_func", ResourceRole::NullSource);

        let roles = table.lookup("my_func").unwrap();
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&ResourceRole::Allocator));
        assert!(roles.contains(&ResourceRole::NullSource));
    }

    #[test]
    fn add_same_role_twice_is_noop() {
        let mut table = ResourceTable::empty();
        table.add("my_func", ResourceRole::Allocator);
        table.add("my_func", ResourceRole::Allocator);

        let roles = table.lookup("my_func").unwrap();
        assert_eq!(roles.len(), 1);
    }

    #[test]
    fn lookup_nonexistent() {
        let table = ResourceTable::new();
        assert!(table.lookup("nonexistent_function").is_none());
    }

    #[test]
    fn function_names_sorted() {
        let mut table = ResourceTable::empty();
        table.add("zebra", ResourceRole::Allocator);
        table.add("apple", ResourceRole::Deallocator);

        let names = table.function_names();
        assert_eq!(names, vec!["apple", "zebra"]);
    }

    #[test]
    fn export_round_trip() {
        let mut table = ResourceTable::empty();
        table.add("my_alloc", ResourceRole::Allocator);
        table.add("my_alloc", ResourceRole::NullSource);

        let exported = table.export();
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].name, "my_alloc");
        assert_eq!(exported[0].roles.len(), 2);

        // Verify JSON serialization
        let json = serde_json::to_string(&exported).unwrap();
        let parsed: Vec<ResourceTableEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "my_alloc");
    }

    #[test]
    fn role_serialization() {
        let role = ResourceRole::Allocator;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"allocator\"");

        let parsed: ResourceRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ResourceRole::Allocator);
    }

    #[test]
    fn all_roles_serialize() {
        let roles = [
            ResourceRole::Allocator,
            ResourceRole::Deallocator,
            ResourceRole::Reallocator,
            ResourceRole::Acquire,
            ResourceRole::Release,
            ResourceRole::Lock,
            ResourceRole::Unlock,
            ResourceRole::NullSource,
            ResourceRole::Dereference,
        ];

        for role in roles {
            let json = serde_json::to_string(&role).unwrap();
            let parsed: ResourceRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, parsed);
        }
    }

    #[test]
    fn common_wrappers() {
        let table = ResourceTable::new();
        assert!(table.has_role("xmalloc", ResourceRole::Allocator));
        assert!(table.has_role("g_malloc", ResourceRole::Allocator));
        assert!(table.has_role("g_free", ResourceRole::Deallocator));
    }
}
