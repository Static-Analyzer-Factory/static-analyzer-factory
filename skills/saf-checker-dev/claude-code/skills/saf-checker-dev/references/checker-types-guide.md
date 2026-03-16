# Checker Types Reference

Quick-reference for SAF checker classification, reachability modes, site patterns, resource roles, and all 9 built-in checkers.

---

## Reachability Mode Decision Tree

```
Does the bug require a value to REACH a bad location?
 |
 +-- YES --> Does it need to reach the SAME kind of sink TWICE?
 |            |
 |            +-- YES --> MultiReach
 |            |           (e.g., double-free: allocation reaches 2+ free() calls)
 |            |
 |            +-- NO  --> MayReach
 |                        (e.g., UAF, null-deref, stack-escape, uninit-use)
 |
 +-- NO  --> Does the bug occur when a value FAILS to reach cleanup?
              |
              +-- "Source reaches exit WITHOUT sanitizer on ALL paths"
              |    --> MustNotReach
              |        (e.g., fd-leak, lock-not-released, generic-resource-leak)
              |
              +-- "Source NEVER reaches ANY sink on any path"
                   --> NeverReachSink
                       (e.g., memory-leak SVF-style: no free() reached = NEVERFREE)
```

**MustNotReach vs NeverReachSink**: MustNotReach uses a three-role model (source/sanitizer/sink=exit) and reports when no sanitizer appears on all paths to exit. NeverReachSink uses a two-role model (source/sink) and reports when the source never reaches any sink at all.

---

## SitePattern Catalog

| Variant | What It Matches | Typical Role | Example Checker |
|---|---|---|---|
| `Role { role, match_return }` | Calls to functions with a `ResourceRole`. `match_return=true` selects return value; `false` selects first arg. | source, sink, or sanitizer | memory-leak (Allocator), use-after-free (Deallocator) |
| `FunctionName { name, match_return }` | Calls to a specific function by name. `match_return` selects return vs first arg. | source, sink, or sanitizer | (custom checkers) |
| `FunctionExit` | Any function return/exit point (`Ret` instruction). | sink | stack-escape, fd-leak, lock-not-released |
| `AnyUseOf` | Any use of a value flowing from source (any SVFG successor). | sink | (not used in built-ins; risks false positives) |
| `AllocaInst` | Stack allocation (`alloca` instruction). | source | stack-escape |
| `LoadDeref` | `Load` instruction -- dereferences the pointer operand. | sink or sanitizer | null-deref (sink), uninit-use (sink) |
| `StoreDeref` | `Store` instruction -- dereferences the pointer operand. | sink or sanitizer | null-deref (sink), uninit-use (sanitizer) |
| `GepDeref` | `GEP` instruction -- base pointer must be valid. In SV-COMP, GEP on NULL is a valid-deref violation. | sink | null-deref |
| `NullConstant` | Explicit NULL constant assignment (`%ptr = copy null`). | source | null-deref |
| `DirectNullDeref` | Instructions where null is literally the pointer operand (Load/Store/GEP) in code NOT guarded by a null check. | sink | (available for definite null-deref detection) |
| `NullCheckBranch` | Values guarded by a null check on the not-null path. | sanitizer | null-deref |
| `CustomPredicate { name }` | Runtime-resolved predicate function. | any | (Tier 3 custom checkers) |

---

## Built-in Checkers (9)

| # | Name | CWE | Severity | Mode | Sources | Sinks | Sanitizers | Description |
|---|---|---|---|---|---|---|---|---|
| 1 | `memory-leak` | 401 | Warning | NeverReachSink | `Role(Allocator, ret)` | `Role(Deallocator, arg)` | -- | Heap allocation never freed |
| 2 | `use-after-free` | 416 | Critical | MayReach | `Role(Deallocator, arg)` | `LoadDeref`, `StoreDeref` | -- | Pointer used after being freed |
| 3 | `double-free` | 415 | Critical | MultiReach | `Role(Allocator, ret)` | `Role(Deallocator, arg)` | -- | Memory freed more than once |
| 4 | `null-deref` | 476 | Error | MayReach | `Role(NullSource, ret)`, `NullConstant` | `Role(Dereference, arg)`, `LoadDeref`, `StoreDeref`, `GepDeref` | `NullCheckBranch` | Potentially null pointer dereferenced without check |
| 5 | `file-descriptor-leak` | 775 | Warning | MustNotReach | `Role(Acquire, ret)` | `FunctionExit` | `Role(Release, arg)` | File descriptor not released on all paths |
| 6 | `uninit-use` | 908 | Warning | MayReach | `Role(Allocator, ret)` | `LoadDeref` | `StoreDeref` | Heap memory used before initialization |
| 7 | `stack-escape` | 562 | Error | MayReach | `AllocaInst` | `FunctionExit` | -- | Stack address escapes function scope |
| 8 | `lock-not-released` | 764 | Warning | MustNotReach | `Role(Lock, arg)` | `FunctionExit` | `Role(Unlock, arg)` | Lock not released on all paths to exit |
| 9 | `generic-resource-leak` | 772 | Warning | MustNotReach | `Role(Allocator, ret)` | `FunctionExit` | `Role(Deallocator, arg)` | Resource acquired but not released on all paths |

---

## ResourceRole Catalog

| Role | Description | Example Functions |
|---|---|---|
| `Allocator` | Allocates heap memory | `malloc`, `calloc`, `strdup`, `mmap`, `_Znwm` (C++ new), `g_malloc`, `xmalloc` |
| `Deallocator` | Frees heap memory | `free`, `munmap`, `_ZdlPv` (C++ delete), `g_free` |
| `Reallocator` | Deallocates old + allocates new | `realloc`, `xrealloc` |
| `Acquire` | Acquires a non-memory resource | `fopen`, `open`, `socket`, `accept`, `dup`, `pipe` |
| `Release` | Releases a non-memory resource | `fclose`, `close` |
| `Lock` | Acquires a lock | `pthread_mutex_lock`, `pthread_mutex_trylock`, `pthread_rwlock_rdlock`, `pthread_rwlock_wrlock` |
| `Unlock` | Releases a lock | `pthread_mutex_unlock`, `pthread_rwlock_unlock` |
| `NullSource` | May return null | `malloc`, `calloc`, `realloc`, `fopen`, `mmap`, nothrow C++ new |
| `Dereference` | Dereferences a pointer argument | `memcpy`, `strlen`, `strcpy`, `printf`, `strcmp` |

A function can have multiple roles (e.g., `malloc` is both `Allocator` and `NullSource`). Custom entries can be added via `ResourceTable::add()`.

---

## Tier Classification Decision Tree

```
Can the checker be expressed entirely with existing SitePattern variants?
 |
 +-- YES --> Tier 1 (Declarative)
 |           Write a CheckerSpec with sources/sinks/sanitizers/mode.
 |           No Rust code needed. All 9 built-in checkers are Tier 1.
 |
 +-- NO  --> Does the bug involve ordered state transitions (open->use->close)?
              |
              +-- YES --> Tier 2 (Typestate)
              |           Requires a state machine over SitePattern events.
              |           Example: file opened, read/written, must be closed
              |           before any path that reopens it.
              |
              +-- NO  --> Tier 3 (Custom)
                          Need a new SitePattern variant or CustomPredicate.
                          Requires Rust code changes:
                          1. Add variant to SitePattern enum in spec.rs
                          2. Add classification logic in site_classifier.rs
                          3. Add matching logic in the SVFG solver
```

### Quick Tier Checklist

| Question | If YES |
|---|---|
| Source/sink/sanitizer are all function calls identifiable by name or role? | Tier 1 |
| Source/sink/sanitizer include instruction-level patterns (Load, Store, GEP, Alloca)? | Tier 1 (patterns exist) |
| Need null-pointer-specific matching? | Tier 1 (NullConstant, NullCheckBranch, DirectNullDeref exist) |
| Need to track ordered state transitions across multiple operations? | Tier 2 |
| Need a predicate that examines instruction operands, types, or context? | Tier 3 (CustomPredicate or new variant) |

---

## Key Caveats

- **SSA node deduplication**: `BTreeSet<SvfgNodeId>` deduplicates same-SSA call sites. Two calls to `free(p)` on the same SSA value collapse to one SVFG node.
- **ResourceRole is a flat namespace**: Roles shared across categories (memory/file/lock) can cause checker cross-contamination. Use specific roles (`Lock`/`Unlock`) not generic ones (`Allocator`/`Deallocator`) for non-memory resources.
- **`AnyUseOf` risks false positives**: The source SSA value may flow to pre-bug uses. Prefer specific deref patterns (`LoadDeref`, `StoreDeref`) over `AnyUseOf`.
- **`target != source` guard**: The SVFG solver skips zero-length flows where SSA collapses source and sink to the same node.

Source: `crates/saf-analysis/src/checkers/spec.rs`, `resource_table.rs`, `site_classifier.rs`
