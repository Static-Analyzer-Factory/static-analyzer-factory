# Context-Sensitive SVFG Checker Traversal (CFL-Reachability)

## Problem

SAF's SVFG checker solvers (`may_reach`, `multi_reach`, `must_not_reach`) perform plain BFS reachability without tracking calling context. When value flow crosses function boundaries via `CallArg`/`Return` edges, the solvers allow interprocedurally unrealizable paths — entering a callee through call site A but returning to call site B.

### Concrete impact (verified with tests)

**`df_identity_fn.c`** — Two independent allocations pass through an identity function `id()`. Each is freed exactly once (no bug). SAF reports **2 false-positive double-frees** because the context-insensitive SVFG merges both allocations' flows through `id()`'s parameter node.

**`df_identity_real.c`** — One allocation passes through `id()` twice and is freed twice (real bug). SAF correctly detects it but also reports a spurious memory leak.

### Root cause

The BFS in `solver.rs` discards edge kind:

```rust
for (_, target) in succs {  // underscore ignores SvfgEdgeKind
```

No call-string context is tracked. No matched-parentheses validation on `CallArg`/`Return` pairs.

### SVF's approach

SVF builds the SVFG context-insensitively but performs CFL-reachability during checker traversal: it tracks a call-string context stack and only follows `Return` edges that match the corresponding `CallArg` entry point. SAF's own DDA module already implements this pattern (`dda/types.rs::CallString`, `dda/solver.rs::handle_call_entry`/`handle_return_exit`).

## Design

### 1. Annotate `CallArg`/`Return` edges with call-site `InstId`

Current:
```rust
pub enum SvfgEdgeKind {
    CallArg,
    Return,
    // ...
}
```

New:
```rust
pub enum SvfgEdgeKind {
    CallArg { call_site: InstId },
    Return { call_site: InstId },
    // ... (other variants unchanged)
}
```

The builder's `add_call_edges()` already has `call_site: InstId` as a parameter — pass it into the edge variants. This makes call-site information available at traversal time without expensive runtime recovery.

**Breaking changes:**
- Serde format for `CallArg`/`Return` changes from string to struct
- Pattern matches on `SvfgEdgeKind::CallArg` become `SvfgEdgeKind::CallArg { .. }`
- `is_direct()`, `name()` use `matches!` — still work with `{ .. }` patterns
- `SvfgEdgeKind` can no longer be `Copy` (contains `InstId`) — but it's already behind references in BTreeSet iterators, so impact is minimal
- Export: `PropertyGraph` uses `kind.name()` which still works; `SvfgExportEdge` serializes `SvfgEdgeKind` directly, now includes `call_site` field
- DDA solver: `classify_node()` currently recovers call-site via `find_call_site_for_param()` — can switch to reading from edge directly (simplification)

**Sites requiring update (~13):**
- `svfg/builder.rs`: 2 sites (construction)
- `svfg/mod.rs`: 4 sites (tests + `is_direct`/`name`)
- `svfg/optimize.rs`: 2 sites (tests)
- `svfg/export.rs`: 0 (uses `kind.name()`)
- `dda/solver.rs`: 3 sites (can simplify by reading call-site from edge)

### 2. Relocate `CallString` for shared use

Move `CallString` from `dda/types.rs` to `svfg/context.rs` (or a new `cfl.rs` module in the svfg directory). Both DDA and checker solvers import from the shared location.

`CallString` API (already exists, no changes needed):
- `empty()` — create empty context
- `push(call_site)` — enter callee (opening parenthesis)
- `pop()` — exit callee (closing parenthesis)
- `matches(return_site)` — check if top matches return site
- `depth()` — current call-string length

### 3. Add `max_context_depth` to `SolverConfig`

```rust
pub struct SolverConfig {
    pub max_depth: usize,            // existing: BFS depth limit
    pub max_context_depth: usize,    // new: k-limit for CFL context (default: 3)
}
```

### 4. CFL-aware BFS in all three solvers

#### `may_reach` changes

**BFS state**: Change from `(node, depth)` to `(node, depth, CallString)`.

**Visited set**: Change from `BTreeSet<SvfgNodeId>` to `BTreeSet<(SvfgNodeId, CallString)>`. The same SVFG node may be visited in different calling contexts.

**Edge traversal**: Instead of `for (_, target) in succs`, match on edge kind:

```rust
for (kind, target) in succs {
    let new_ctx = match kind {
        SvfgEdgeKind::CallArg { call_site } => {
            if ctx.depth() >= config.max_context_depth {
                ctx.clone()  // k-limit exceeded: treat as wildcard
            } else {
                ctx.push(*call_site)
            }
        }
        SvfgEdgeKind::Return { call_site } => {
            if ctx.is_empty() {
                ctx.clone()  // empty context: allow (conservative)
            } else if ctx.matches(*call_site) {
                let (popped, _) = ctx.pop().unwrap();
                popped
            } else {
                continue;  // mismatched return: unrealizable path, skip
            }
        }
        _ => ctx.clone(),  // intraprocedural edges: context unchanged
    };
    // ... rest of BFS with new_ctx
}
```

**Key CFL rules:**
- `CallArg`: push call-site onto context (opening parenthesis)
- `Return` with matching top: pop (closing parenthesis)
- `Return` with non-empty mismatched top: skip (unrealizable path)
- `Return` with empty context: allow (conservative — we entered this function from an unknown caller)
- k-limit exceeded: stop pushing, treat as wildcard (over-approximate)

#### `multi_reach` changes

Same CFL-aware BFS as `may_reach`. The `reached_sinks` collection and 2+ threshold logic remain unchanged.

#### `must_not_reach` changes

Same CFL-aware BFS. Exit node detection and sanitizer checking remain unchanged.

#### `may_reach_guarded` changes

Same CFL-aware BFS layered on top of existing guard accumulation. Guards and context are orthogonal — guards track branch conditions, context tracks call strings.

### 5. Fallback behavior

When `max_context_depth` is 0: disable CFL matching entirely (existing behavior, for backward compatibility and benchmarking).

When k-limit is reached during `push`: stop growing the context but continue traversal. This is over-approximate (may still have false positives from deep call chains) but sound (never misses real bugs).

### 6. Test plan

1. **`df_identity_fn.c`**: Two independent allocations through `id()`, freed once each. After fix: 0 double-free findings (currently: 2 false positives).
2. **`df_identity_real.c`**: One allocation through `id()`, freed twice. After fix: 1 double-free finding (should still work).
3. **Unit test — mismatched return**: Build SVFG with `A→CallArg(site1)→param→Return(site2)→B`. CFL-aware BFS should NOT find A→B reachable.
4. **Unit test — matched return**: Build SVFG with `A→CallArg(site1)→param→Return(site1)→B`. CFL-aware BFS should find A→B reachable.
5. **Unit test — k-limit**: Deep call chain exceeding k=3. Solver should degrade to context-insensitive rather than stopping.
6. **Unit test — empty context return**: Return edge with empty context should be allowed (conservative).
7. **Regression**: Run existing checker tests to verify no regressions.

### 7. Files affected

| File | Change |
|------|--------|
| `svfg/mod.rs` | `CallArg`/`Return` enum variants get `{ call_site: InstId }` |
| `svfg/mod.rs` | Update `is_direct()`, `name()`, tests |
| `svfg/builder.rs` | Pass `call_site` into edge construction |
| `svfg/context.rs` (new) | `CallString` relocated from `dda/types.rs` |
| `svfg/optimize.rs` | Update test edge construction |
| `svfg/export.rs` | Verify serde still works |
| `checkers/solver.rs` | CFL-aware BFS in `may_reach`, `multi_reach`, `must_not_reach`, `may_reach_guarded` |
| `checkers/solver.rs` | `SolverConfig` gets `max_context_depth` |
| `dda/types.rs` | Re-export `CallString` from new shared location |
| `dda/solver.rs` | Simplify `classify_node` to read call-site from edge; update pattern matches |

### 8. Non-goals

- **Interprocedural memory flow** (store through pointer parameter visible to caller after return): This is Plan 061, orthogonal to CFL-matching. The `df_interproc_mem.c` test case requires interprocedural MSSA, not context-sensitive traversal.
- **Heap cloning**: Distinguishing heap allocations by calling context (e.g., `malloc` inside a wrapper called from different sites). This would require context-sensitive PTA, not just CFL-aware traversal.
