# Playground Checker Examples + Python Analyzers + Differential Testing

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create example C programs and Python analyzers for all 9 query checkers in the playground, then verify consistency via backend differential tests and Playwright E2E tests.

**Architecture:** Add 8 new C examples to the playground examples dropdown (UAF already exists), add 9 Python analyzer templates that detect the same bugs via graph traversal, run a backend differential test (Python SDK `proj.check()` vs graph-traversal analyzers on compiled LLVM IR), then run Playwright E2E tests through the browser UI.

**Tech Stack:** TypeScript (playground), Python (analyzers + tests), Playwright (E2E), Docker (backend compilation + testing)

---

## Task 1: Add 8 New C Example Programs

**Files:**
- Modify: `playground/src/examples/index.ts`

**Step 1: Add the 8 new example programs**

Add these examples BEFORE the `export const examples` array, then include them in the array. Keep the existing 7 examples (pointer-alias, indirect-call, struct-field, taint-flow, complex-cfg, use-after-free, library-modeling) and append the 8 new ones.

```typescript
const memoryLeak: Example = {
  name: 'Memory Leak',
  slug: 'memory_leak',
  description: 'Heap allocation not freed — CWE-401 memory leak detection',
  source: `#include <stdlib.h>

void process(int flag) {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;

    if (flag) {
        free(p);
        return;
    }

    // BUG: p is not freed when flag == 0
}

int main() {
    process(0);
    return 0;
}`,
};

const doubleFree: Example = {
  name: 'Double Free',
  slug: 'double_free',
  description: 'Memory freed twice — CWE-415 double-free detection',
  source: `#include <stdlib.h>

void process() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;

    free(p);

    // BUG: p is freed again
    free(p);
}

int main() {
    process();
    return 0;
}`,
};

const nullDeref: Example = {
  name: 'Null Dereference',
  slug: 'null_deref',
  description: 'Pointer used without null check — CWE-476 null dereference detection',
  source: `#include <stdlib.h>

int process() {
    int *p = (int *)malloc(sizeof(int));
    // BUG: malloc can return NULL, but p is used without check
    *p = 42;
    int val = *p;
    free(p);
    return val;
}

int main() {
    return process();
}`,
};

const fileDescriptorLeak: Example = {
  name: 'File Descriptor Leak',
  slug: 'file_descriptor_leak',
  description: 'File opened but not closed — CWE-775 file descriptor leak detection',
  source: `#include <stdio.h>

void process(const char *path) {
    FILE *f = fopen(path, "r");
    if (!f) return;

    char buf[256];
    fgets(buf, sizeof(buf), f);

    // BUG: f is never closed
}

int main() {
    process("data.txt");
    return 0;
}`,
};

const lockNotReleased: Example = {
  name: 'Lock Not Released',
  slug: 'lock_not_released',
  description: 'Mutex locked but not unlocked on all paths — CWE-764 lock safety',
  source: `#include <pthread.h>

pthread_mutex_t mtx;
int shared_data = 0;

void process(int flag) {
    pthread_mutex_lock(&mtx);
    shared_data++;

    if (flag) {
        // BUG: lock not released on this path
        return;
    }

    pthread_mutex_unlock(&mtx);
}

int main() {
    pthread_mutex_init(&mtx, 0);
    process(1);
    pthread_mutex_destroy(&mtx);
    return 0;
}`,
};

const uninitUse: Example = {
  name: 'Uninitialized Use',
  slug: 'uninit_use',
  description: 'Heap memory read before initialization — CWE-908 uninitialized use',
  source: `#include <stdlib.h>

int process() {
    int *p = (int *)malloc(sizeof(int));

    // BUG: reading *p before writing to it
    int val = *p;

    free(p);
    return val;
}

int main() {
    return process();
}`,
};

const stackEscape: Example = {
  name: 'Stack Escape',
  slug: 'stack_escape',
  description: 'Local variable address returned — CWE-562 stack escape detection',
  source: `int *get_value() {
    int x = 42;
    // BUG: returning address of stack variable
    return &x;
}

int main() {
    int *p = get_value();
    return *p;
}`,
};

const genericResourceLeak: Example = {
  name: 'Generic Resource Leak',
  slug: 'generic_resource_leak',
  description: 'Custom resource acquired but not released — CWE-772 resource leak',
  source: `#include <stdlib.h>

// Simulate a custom resource (e.g., database connection)
typedef struct {
    int handle;
    char *name;
} Resource;

Resource *acquire_resource() {
    Resource *r = (Resource *)malloc(sizeof(Resource));
    r->handle = 1;
    r->name = (char *)malloc(64);
    return r;
}

void release_resource(Resource *r) {
    free(r->name);
    free(r);
}

void process() {
    Resource *r = acquire_resource();
    r->handle = 42;

    // BUG: r is never released via release_resource()
    // (inner mallocs are also leaked)
}

int main() {
    process();
    return 0;
}`,
};
```

Update the exports array to include all 15 examples:

```typescript
export const examples: Example[] = [
  pointerAlias,
  indirectCall,
  structField,
  taintFlow,
  complexCFG,
  useAfterFree,
  libraryModeling,
  memoryLeak,
  doubleFree,
  nullDeref,
  fileDescriptorLeak,
  lockNotReleased,
  uninitUse,
  stackEscape,
  genericResourceLeak,
];
```

**Step 2: Verify build**

```bash
cd playground && npm run build
```

Expected: Build succeeds with no errors.

**Step 3: Commit**

```bash
git add playground/src/examples/index.ts
git commit -m "feat(playground): add 8 checker example programs for all 9 query checkers"
```

---

## Task 2: Add 9 Python Analyzer Templates

**Files:**
- Modify: `playground/src/components/AnalyzerPanel.tsx`

**Step 1: Add 9 checker-specific analyzer templates**

Add these to the `TEMPLATES` object in `AnalyzerPanel.tsx`, keeping the existing 4 templates (list_free, detect_uaf, explore_cfg, custom) and adding 9 new ones.

Each analyzer uses the playground's Python API: `saf.analyze()` → graph traversal → `saf.report()`.

```typescript
  detect_memory_leak: {
    label: 'Detect Memory Leak (CWE-401)',
    code: `import saf

result = saf.analyze()
cg = result.callgraph
vf = result.valueflow

# Find allocator functions (malloc, calloc, realloc)
alloc_names = {"malloc", "calloc", "realloc", "aligned_alloc"}
alloc_fns = [n for n in cg.nodes if n.properties.get("name") in alloc_names]

# Find deallocator functions (free)
free_fns = [n for n in cg.nodes if n.properties.get("name") == "free"]

if not alloc_fns:
    print("No allocator calls found in this program")
else:
    # Find value flow nodes belonging to allocator returns
    alloc_return_ids = set()
    for fn in alloc_fns:
        for n in vf.nodes:
            if n.properties.get("parent_function") == fn.properties.get("name"):
                alloc_return_ids.add(n.id)

    # Find value flow nodes belonging to free's parameter
    free_param_ids = set()
    for fn in free_fns:
        for n in vf.nodes:
            if n.properties.get("parent_function") == "free":
                free_param_ids.add(n.id)

    # For each alloc call site, check if its return value reaches free
    for node in vf.nodes:
        if node.id not in alloc_return_ids:
            continue
        outgoing = [e for e in vf.edges if e.src == node.id]
        reaches_free = any(e.dst in free_param_ids for e in outgoing
                          if e.edge_type == "CALLARG")
        if not reaches_free:
            line = saf.source_line(node.id)
            line_str = f" at line {line}" if line else ""
            saf.report(node.id, "medium",
                       f"Allocation{line_str} may not be freed on all paths")
            print(f"Potential leak: allocation{line_str}")

    if not any(True for n in vf.nodes if n.id in alloc_return_ids):
        print("No allocation return values found in value flow")
`,
  },
  detect_double_free: {
    label: 'Detect Double Free (CWE-415)',
    code: `import saf

result = saf.analyze()
cg = result.callgraph
vf = result.valueflow

# Find free() calls
free_fns = [n for n in cg.nodes if n.properties.get("name") == "free"]
if not free_fns:
    print("No free() calls found")
else:
    # Find value flow nodes for free's parameter
    free_param_ids = set()
    for n in vf.nodes:
        if n.properties.get("parent_function") == "free":
            free_param_ids.add(n.id)

    # Find values that flow to free() via CALLARG more than once
    # Group by source value
    from collections import defaultdict
    value_to_frees = defaultdict(list)

    for node in vf.nodes:
        outgoing = [e for e in vf.edges if e.src == node.id]
        free_args = [e for e in outgoing if e.edge_type == "CALLARG"
                     and e.dst in free_param_ids]
        if len(free_args) >= 2:
            line = saf.source_line(node.id)
            line_str = f" at line {line}" if line else ""
            saf.report(node.id, "high",
                       f"Value{line_str} passed to free() {len(free_args)} times — double free")
            print(f"Double free: value{line_str} freed {len(free_args)} times")

    # Also check if the same allocation reaches multiple free nodes
    alloc_names = {"malloc", "calloc", "realloc"}
    for node in vf.nodes:
        parent = node.properties.get("parent_function", "")
        if parent not in alloc_names:
            continue
        outgoing = [e for e in vf.edges if e.src == node.id]
        free_dsts = [e for e in outgoing if e.edge_type == "CALLARG"
                     and e.dst in free_param_ids]
        if len(free_dsts) >= 2:
            line = saf.source_line(node.id)
            line_str = f" at line {line}" if line else ""
            saf.report(node.id, "high",
                       f"Allocation{line_str} freed {len(free_dsts)} times")
            print(f"Double free from allocation{line_str}")
`,
  },
  detect_null_deref: {
    label: 'Detect Null Dereference (CWE-476)',
    code: `import saf

result = saf.analyze()
cg = result.callgraph
vf = result.valueflow

# Functions that can return NULL
nullable_fns = {"malloc", "calloc", "realloc", "fopen", "aligned_alloc"}

# Find nullable function nodes in callgraph
nullable_nodes = [n for n in cg.nodes
                  if n.properties.get("name") in nullable_fns]

if not nullable_nodes:
    print("No nullable function calls found")
else:
    # Find value flow nodes from nullable functions (return values)
    nullable_return_ids = set()
    for fn in nullable_nodes:
        fname = fn.properties.get("name")
        for n in vf.nodes:
            if n.properties.get("parent_function") == fname:
                nullable_return_ids.add(n.id)

    # Check if any nullable return flows to a dereference without guard
    for node in vf.nodes:
        if node.id not in nullable_return_ids:
            continue
        outgoing = [e for e in vf.edges if e.src == node.id]
        for e in outgoing:
            if e.edge_type in ("Direct", "Store", "Load"):
                dst_node = next((n for n in vf.nodes if n.id == e.dst), None)
                if dst_node:
                    line = saf.source_line(dst_node.id)
                    src_line = saf.source_line(node.id)
                    line_str = f" at line {line}" if line else ""
                    src_str = f" from line {src_line}" if src_line else ""
                    saf.report(node.id, "high",
                               f"Nullable pointer{src_str} used{line_str} without null check")
                    print(f"Potential null deref: pointer{src_str} dereferenced{line_str}")
                    break  # One report per nullable source
`,
  },
  detect_file_leak: {
    label: 'Detect File Descriptor Leak (CWE-775)',
    code: `import saf

result = saf.analyze()
cg = result.callgraph
vf = result.valueflow

# Find fopen() and similar file-opening functions
open_fns = {"fopen", "fdopen", "freopen", "tmpfile", "open"}
close_fns = {"fclose", "close"}

open_nodes = [n for n in cg.nodes if n.properties.get("name") in open_fns]
close_nodes = [n for n in cg.nodes if n.properties.get("name") in close_fns]

if not open_nodes:
    print("No file-opening calls found")
else:
    # Find value flow nodes for file open returns
    open_return_ids = set()
    for fn in open_nodes:
        fname = fn.properties.get("name")
        for n in vf.nodes:
            if n.properties.get("parent_function") == fname:
                open_return_ids.add(n.id)

    # Find value flow nodes for close parameters
    close_param_ids = set()
    for fn in close_nodes:
        fname = fn.properties.get("name")
        for n in vf.nodes:
            if n.properties.get("parent_function") == fname:
                close_param_ids.add(n.id)

    # Check if file descriptors reach close
    for node in vf.nodes:
        if node.id not in open_return_ids:
            continue
        outgoing = [e for e in vf.edges if e.src == node.id]
        reaches_close = any(e.dst in close_param_ids for e in outgoing
                           if e.edge_type == "CALLARG")
        if not reaches_close:
            line = saf.source_line(node.id)
            line_str = f" at line {line}" if line else ""
            saf.report(node.id, "medium",
                       f"File opened{line_str} may not be closed on all paths")
            print(f"Potential file leak{line_str}")
`,
  },
  detect_lock_leak: {
    label: 'Detect Lock Not Released (CWE-764)',
    code: `import saf

result = saf.analyze()
cg = result.callgraph
vf = result.valueflow

# Find lock/unlock functions
lock_fns = {"pthread_mutex_lock", "mtx_lock"}
unlock_fns = {"pthread_mutex_unlock", "mtx_unlock"}

lock_nodes = [n for n in cg.nodes if n.properties.get("name") in lock_fns]
unlock_nodes = [n for n in cg.nodes if n.properties.get("name") in unlock_fns]

if not lock_nodes:
    print("No lock calls found")
else:
    # Find value flow nodes for lock arguments
    lock_arg_ids = set()
    for fn in lock_nodes:
        fname = fn.properties.get("name")
        for n in vf.nodes:
            if n.properties.get("parent_function") == fname:
                lock_arg_ids.add(n.id)

    # Find value flow nodes for unlock arguments
    unlock_arg_ids = set()
    for fn in unlock_nodes:
        fname = fn.properties.get("name")
        for n in vf.nodes:
            if n.properties.get("parent_function") == fname:
                unlock_arg_ids.add(n.id)

    # Check if lock arguments reach unlock
    for node in vf.nodes:
        if node.id not in lock_arg_ids:
            continue
        outgoing = [e for e in vf.edges if e.src == node.id]
        reaches_unlock = any(e.dst in unlock_arg_ids for e in outgoing
                            if e.edge_type == "CALLARG")
        if not reaches_unlock:
            line = saf.source_line(node.id)
            line_str = f" at line {line}" if line else ""
            saf.report(node.id, "medium",
                       f"Lock acquired{line_str} may not be released on all paths")
            print(f"Potential lock leak{line_str}")

    if not lock_nodes:
        print("No lock operations detected")
    elif not unlock_nodes:
        print("Warning: locks found but no unlock calls detected")
        for fn in lock_nodes:
            saf.report(fn.id, "medium",
                       f"{fn.properties.get('name')} called but no unlock found")
`,
  },
  detect_uninit_use: {
    label: 'Detect Uninitialized Use (CWE-908)',
    code: `import saf

result = saf.analyze()
cg = result.callgraph
vf = result.valueflow

# Find malloc (returns uninitialized memory, unlike calloc)
alloc_fns = {"malloc", "realloc", "aligned_alloc"}
init_alloc_fns = {"calloc"}  # These zero-initialize

alloc_nodes = [n for n in cg.nodes
               if n.properties.get("name") in alloc_fns]

if not alloc_nodes:
    print("No uninitialized allocator calls found")
else:
    # Find value flow nodes for allocator returns
    alloc_return_ids = set()
    for fn in alloc_nodes:
        fname = fn.properties.get("name")
        for n in vf.nodes:
            if n.properties.get("parent_function") == fname:
                alloc_return_ids.add(n.id)

    # Check if allocated memory is read (Load) before being written (Store)
    for node in vf.nodes:
        if node.id not in alloc_return_ids:
            continue
        outgoing = [e for e in vf.edges if e.src == node.id]

        has_load_before_store = False
        has_store = False

        for e in outgoing:
            if e.edge_type == "Store":
                has_store = True
            elif e.edge_type == "Load":
                if not has_store:
                    has_load_before_store = True

        if has_load_before_store:
            line = saf.source_line(node.id)
            line_str = f" at line {line}" if line else ""
            saf.report(node.id, "medium",
                       f"Allocated memory{line_str} may be read before initialization")
            print(f"Potential uninit use{line_str}")
`,
  },
  detect_stack_escape: {
    label: 'Detect Stack Escape (CWE-562)',
    code: `import saf

result = saf.analyze()
vf = result.valueflow
cg = result.callgraph

# Look for value flow nodes from alloca (stack allocations)
# In value flow, alloca nodes have kind "location" and specific labels
alloca_ids = set()
for node in vf.nodes:
    kind = node.properties.get("kind", "")
    label = node.properties.get("label", "")
    if "alloca" in label.lower() or kind == "location":
        alloca_ids.add(node.id)

if not alloca_ids:
    print("No stack allocations found in value flow")
else:
    print(f"Found {len(alloca_ids)} potential stack allocation(s)")

    # Check if any stack allocation flows to a function return
    # or escapes via store to a global/heap location
    for node in vf.nodes:
        if node.id not in alloca_ids:
            continue
        outgoing = [e for e in vf.edges if e.src == node.id]

        for e in outgoing:
            dst_node = next((n for n in vf.nodes if n.id == e.dst), None)
            if not dst_node:
                continue
            # Check if destination is in a different function (escape)
            src_func = node.properties.get("parent_function", "")
            dst_func = dst_node.properties.get("parent_function", "")
            if src_func and dst_func and src_func != dst_func:
                line = saf.source_line(node.id)
                line_str = f" at line {line}" if line else ""
                saf.report(node.id, "high",
                           f"Stack variable{line_str} in {src_func}() escapes to {dst_func}()")
                print(f"Stack escape: {src_func}{line_str} -> {dst_func}")
`,
  },
  detect_uaf_full: {
    label: 'Detect Use-After-Free (CWE-416)',
    code: `import saf

result = saf.analyze()
cg = result.callgraph
vf = result.valueflow

# Find free() in the call graph
free_fns = [n for n in cg.nodes if n.properties.get("name") == "free"]
if not free_fns:
    print("No free() calls found — no UAF possible")
else:
    for fn in free_fns:
        callers = cg.predecessors(fn.id)
        for c in callers:
            print(f"free() called from {c.properties.get('name', '?')}()")

# Build set of free() parameter node IDs
free_param_ids = set()
for n in vf.nodes:
    if n.properties.get("parent_function") == "free":
        free_param_ids.add(n.id)

# Find values freed then used again
print(f"\\nScanning value flow ({len(vf.nodes)} nodes)...")

suspects = []
for node in vf.nodes:
    if node.properties.get("kind") != "value":
        continue
    outgoing = [e for e in vf.edges if e.src == node.id]
    free_args = [e for e in outgoing if e.edge_type == "CALLARG" and e.dst in free_param_ids]
    others = [e for e in outgoing if e.edge_type != "CALLARG"]

    if len(free_args) >= 1 and len(others) >= 1:
        suspects.append((node, free_args, others))

if not suspects:
    print("No use-after-free patterns detected")
else:
    print(f"\\nFound {len(suspects)} suspect value(s):")
    for node, free_args, others in suspects:
        line = saf.source_line(node.id)
        line_str = f" (line {line})" if line else ""

        use_lines = []
        for e in others:
            dst_line = saf.source_line(e.dst)
            if dst_line:
                use_lines.append(dst_line)

        free_lines = []
        for e in free_args:
            dst_line = saf.source_line(e.dst)
            if dst_line:
                free_lines.append(dst_line)

        msg = f"Value{line_str} passed to free()"
        if free_lines:
            msg += f" at line {', '.join(str(l) for l in sorted(set(free_lines)))}"
        if use_lines:
            msg += f", used after free at line {', '.join(str(l) for l in sorted(set(use_lines)))}"

        saf.report(node.id, "high", msg)
        print(f"  {msg}")
`,
  },
  detect_resource_leak: {
    label: 'Detect Generic Resource Leak (CWE-772)',
    code: `import saf

result = saf.analyze()
cg = result.callgraph
vf = result.valueflow

# Generic resource leak: any allocation (malloc, calloc, etc.) not freed
alloc_names = {"malloc", "calloc", "realloc", "aligned_alloc"}
dealloc_names = {"free"}

alloc_nodes = [n for n in cg.nodes if n.properties.get("name") in alloc_names]
dealloc_nodes = [n for n in cg.nodes if n.properties.get("name") in dealloc_names]

if not alloc_nodes:
    print("No resource acquisition calls found")
else:
    # Find value flow nodes for allocator returns
    alloc_return_ids = set()
    for fn in alloc_nodes:
        fname = fn.properties.get("name")
        for n in vf.nodes:
            if n.properties.get("parent_function") == fname:
                alloc_return_ids.add(n.id)

    # Find value flow nodes for deallocator params
    dealloc_param_ids = set()
    for fn in dealloc_nodes:
        fname = fn.properties.get("name")
        for n in vf.nodes:
            if n.properties.get("parent_function") == fname:
                dealloc_param_ids.add(n.id)

    leaked = 0
    for node in vf.nodes:
        if node.id not in alloc_return_ids:
            continue
        outgoing = [e for e in vf.edges if e.src == node.id]
        reaches_dealloc = any(e.dst in dealloc_param_ids for e in outgoing
                             if e.edge_type == "CALLARG")
        if not reaches_dealloc:
            line = saf.source_line(node.id)
            line_str = f" at line {line}" if line else ""
            saf.report(node.id, "medium",
                       f"Resource acquired{line_str} but never released")
            print(f"Resource leak{line_str}")
            leaked += 1

    if leaked == 0:
        print("No resource leaks detected")
    else:
        print(f"\\nTotal: {leaked} potential resource leak(s)")
`,
  },
```

**Step 2: Verify build**

```bash
cd playground && npm run build
```

Expected: Build succeeds.

**Step 3: Commit**

```bash
git add playground/src/components/AnalyzerPanel.tsx
git commit -m "feat(playground): add 9 Python analyzer templates for all query checkers"
```

---

## Task 3: Build and Quick-Verify Playground

**Files:**
- None modified (build + visual check)

**Step 1: Build WASM module**

```bash
make build-wasm
```

**Step 2: Build playground**

```bash
cd playground && npm run build
```

**Step 3: Start dev server and visually verify**

```bash
cd playground && npm run dev
```

Verify:
- All 15 examples appear in the dropdown
- All 13 analyzer templates appear in the template selector
- Selecting a new example loads its C code
- Selecting a new template loads its Python code

**Step 4: Commit if any build fixes were needed**

---

## Task 4: Compile C Examples to LLVM IR for Backend Testing

**Files:**
- Create: `tests/programs/c/checker_memory_leak.c`
- Create: `tests/programs/c/checker_double_free.c`
- Create: `tests/programs/c/checker_null_deref.c`
- Create: `tests/programs/c/checker_file_descriptor_leak.c`
- Create: `tests/programs/c/checker_lock_not_released.c`
- Create: `tests/programs/c/checker_uninit_use.c`
- Create: `tests/programs/c/checker_stack_escape.c`
- Create: `tests/programs/c/checker_generic_resource_leak.c`
- Create: `tests/programs/c/checker_use_after_free.c` (copy of existing UAF example)

**Step 1: Create C source files**

Create each C file with the same source code as the playground examples. Use the exact same code from Task 1 (the `source` strings) plus the existing UAF example.

**Step 2: Compile all to LLVM IR inside Docker**

```bash
docker compose run --rm dev sh -c '
  for f in tests/programs/c/checker_*.c; do
    base=$(basename "$f" .c)
    clang -S -emit-llvm -g -O0 -Xclang -disable-O0-optnone \
      "$f" -o "tests/fixtures/llvm/e2e/${base}.ll" 2>&1 || echo "FAILED: $f"
  done
  echo "Done compiling checker examples"
  ls -la tests/fixtures/llvm/e2e/checker_*.ll
'
```

Note: `-Xclang -disable-O0-optnone` is needed so mem2reg promotes stack allocas properly (discovered in session #S355).

Expected: All 9 `.ll` files created without errors.

**Step 3: Commit**

```bash
git add tests/programs/c/checker_*.c tests/fixtures/llvm/e2e/checker_*.ll
git commit -m "test: add 9 checker example C programs and compiled LLVM IR"
```

---

## Task 5: Backend Differential Test Script

**Files:**
- Create: `tests/differential/test_checker_differential.py`

**Step 1: Write the differential test script**

This script runs inside Docker and for each checker example:
1. Opens the LLVM IR with `saf.Project.open()`
2. Runs `proj.check(checker_name)` to get SVFG-based findings
3. Runs the Python graph-traversal analyzer on the same project
4. Compares: both should find >= 1 bug

```python
"""Differential test: query checkers vs Python graph-traversal analyzers.

Compares built-in SVFG checkers (proj.check()) with Python analyzers
that detect the same bugs via PropertyGraph traversal.

Run inside Docker: python3 tests/differential/test_checker_differential.py
"""

import json
import sys
import os

import saf

# Map checker name -> (example .ll file, expected minimum findings)
CHECKER_EXAMPLES = {
    "memory-leak": ("checker_memory_leak.ll", 1),
    "use-after-free": ("checker_use_after_free.ll", 1),
    "double-free": ("checker_double_free.ll", 1),
    "null-deref": ("checker_null_deref.ll", 1),
    "file-descriptor-leak": ("checker_file_descriptor_leak.ll", 1),
    "lock-not-released": ("checker_lock_not_released.ll", 1),
    "uninit-use": ("checker_uninit_use.ll", 1),
    "stack-escape": ("checker_stack_escape.ll", 1),
    "generic-resource-leak": ("checker_generic_resource_leak.ll", 1),
}

FIXTURE_DIR = "tests/fixtures/llvm/e2e"


# ---- Python graph-traversal analyzers (mirror playground templates) ----

def analyze_memory_leak(proj):
    """Detect memory leaks via graph traversal."""
    graphs = proj.graphs()
    cg = json.loads(graphs.export("callgraph"))
    vf = json.loads(graphs.export("valueflow"))

    alloc_names = {"malloc", "calloc", "realloc", "aligned_alloc"}
    alloc_return_ids = set()
    free_param_ids = set()

    for n in vf["nodes"]:
        parent = n["properties"].get("parent_function", "")
        if parent in alloc_names:
            alloc_return_ids.add(n["id"])
        if parent == "free":
            free_param_ids.add(n["id"])

    findings = []
    for n in vf["nodes"]:
        if n["id"] not in alloc_return_ids:
            continue
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        reaches_free = any(
            e["dst"] in free_param_ids
            for e in outgoing
            if e["edge_type"] == "CALLARG"
        )
        if not reaches_free:
            findings.append({"id": n["id"], "message": "allocation not freed"})
    return findings


def analyze_use_after_free(proj):
    """Detect UAF via graph traversal."""
    graphs = proj.graphs()
    vf = json.loads(graphs.export("valueflow"))

    free_param_ids = set()
    for n in vf["nodes"]:
        if n["properties"].get("parent_function") == "free":
            free_param_ids.add(n["id"])

    findings = []
    for n in vf["nodes"]:
        if n["properties"].get("kind") != "value":
            continue
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        free_args = [e for e in outgoing if e["edge_type"] == "CALLARG"
                     and e["dst"] in free_param_ids]
        others = [e for e in outgoing if e["edge_type"] != "CALLARG"]
        if len(free_args) >= 1 and len(others) >= 1:
            findings.append({"id": n["id"], "message": "use after free"})
    return findings


def analyze_double_free(proj):
    """Detect double-free via graph traversal."""
    graphs = proj.graphs()
    vf = json.loads(graphs.export("valueflow"))

    free_param_ids = set()
    for n in vf["nodes"]:
        if n["properties"].get("parent_function") == "free":
            free_param_ids.add(n["id"])

    findings = []
    for n in vf["nodes"]:
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        free_args = [e for e in outgoing if e["edge_type"] == "CALLARG"
                     and e["dst"] in free_param_ids]
        if len(free_args) >= 2:
            findings.append({"id": n["id"], "message": "double free"})
    return findings


def analyze_null_deref(proj):
    """Detect null deref via graph traversal."""
    graphs = proj.graphs()
    vf = json.loads(graphs.export("valueflow"))

    nullable_fns = {"malloc", "calloc", "realloc", "fopen", "aligned_alloc"}
    nullable_return_ids = set()
    for n in vf["nodes"]:
        if n["properties"].get("parent_function", "") in nullable_fns:
            nullable_return_ids.add(n["id"])

    findings = []
    for n in vf["nodes"]:
        if n["id"] not in nullable_return_ids:
            continue
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        for e in outgoing:
            if e["edge_type"] in ("Direct", "Store", "Load"):
                findings.append({"id": n["id"], "message": "nullable ptr used"})
                break
    return findings


def analyze_file_descriptor_leak(proj):
    """Detect file descriptor leak via graph traversal."""
    graphs = proj.graphs()
    vf = json.loads(graphs.export("valueflow"))

    open_fns = {"fopen", "fdopen", "freopen", "tmpfile", "open"}
    close_fns = {"fclose", "close"}

    open_return_ids = set()
    close_param_ids = set()
    for n in vf["nodes"]:
        parent = n["properties"].get("parent_function", "")
        if parent in open_fns:
            open_return_ids.add(n["id"])
        if parent in close_fns:
            close_param_ids.add(n["id"])

    findings = []
    for n in vf["nodes"]:
        if n["id"] not in open_return_ids:
            continue
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        reaches_close = any(
            e["dst"] in close_param_ids
            for e in outgoing
            if e["edge_type"] == "CALLARG"
        )
        if not reaches_close:
            findings.append({"id": n["id"], "message": "file not closed"})
    return findings


def analyze_lock_not_released(proj):
    """Detect unreleased lock via graph traversal."""
    graphs = proj.graphs()
    vf = json.loads(graphs.export("valueflow"))

    lock_fns = {"pthread_mutex_lock", "mtx_lock"}
    unlock_fns = {"pthread_mutex_unlock", "mtx_unlock"}

    lock_arg_ids = set()
    unlock_arg_ids = set()
    for n in vf["nodes"]:
        parent = n["properties"].get("parent_function", "")
        if parent in lock_fns:
            lock_arg_ids.add(n["id"])
        if parent in unlock_fns:
            unlock_arg_ids.add(n["id"])

    findings = []
    for n in vf["nodes"]:
        if n["id"] not in lock_arg_ids:
            continue
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        reaches_unlock = any(
            e["dst"] in unlock_arg_ids
            for e in outgoing
            if e["edge_type"] == "CALLARG"
        )
        if not reaches_unlock:
            findings.append({"id": n["id"], "message": "lock not released"})
    return findings


def analyze_uninit_use(proj):
    """Detect uninitialized use via graph traversal."""
    graphs = proj.graphs()
    vf = json.loads(graphs.export("valueflow"))

    alloc_fns = {"malloc", "realloc", "aligned_alloc"}
    alloc_return_ids = set()
    for n in vf["nodes"]:
        if n["properties"].get("parent_function", "") in alloc_fns:
            alloc_return_ids.add(n["id"])

    findings = []
    for n in vf["nodes"]:
        if n["id"] not in alloc_return_ids:
            continue
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        has_load = any(e["edge_type"] == "Load" for e in outgoing)
        has_store = any(e["edge_type"] == "Store" for e in outgoing)
        if has_load and not has_store:
            findings.append({"id": n["id"], "message": "read before write"})
    return findings


def analyze_stack_escape(proj):
    """Detect stack escape via graph traversal."""
    graphs = proj.graphs()
    vf = json.loads(graphs.export("valueflow"))

    findings = []
    for n in vf["nodes"]:
        label = n["properties"].get("label", "")
        kind = n["properties"].get("kind", "")
        if "alloca" not in label.lower() and kind != "location":
            continue
        src_func = n["properties"].get("parent_function", "")
        if not src_func:
            continue
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        for e in outgoing:
            dst = next((d for d in vf["nodes"] if d["id"] == e["dst"]), None)
            if dst:
                dst_func = dst["properties"].get("parent_function", "")
                if dst_func and dst_func != src_func:
                    findings.append({"id": n["id"], "message": "stack var escapes"})
                    break
    return findings


def analyze_generic_resource_leak(proj):
    """Detect generic resource leak (same as memory leak)."""
    return analyze_memory_leak(proj)


ANALYZERS = {
    "memory-leak": analyze_memory_leak,
    "use-after-free": analyze_use_after_free,
    "double-free": analyze_double_free,
    "null-deref": analyze_null_deref,
    "file-descriptor-leak": analyze_file_descriptor_leak,
    "lock-not-released": analyze_lock_not_released,
    "uninit-use": analyze_uninit_use,
    "stack-escape": analyze_stack_escape,
    "generic-resource-leak": analyze_generic_resource_leak,
}


def run_differential_test():
    """Run all differential tests and report results."""
    results = []
    passed = 0
    failed = 0

    for checker_name, (ll_file, min_expected) in CHECKER_EXAMPLES.items():
        ll_path = os.path.join(FIXTURE_DIR, ll_file)
        if not os.path.exists(ll_path):
            print(f"SKIP {checker_name}: {ll_path} not found")
            results.append({"checker": checker_name, "status": "skip"})
            continue

        print(f"\n{'='*60}")
        print(f"Testing: {checker_name}")
        print(f"{'='*60}")

        proj = saf.Project.open(ll_path)

        # Run built-in checker
        try:
            query_findings = proj.check(checker_name)
            query_count = len(query_findings)
            print(f"  Query checker: {query_count} finding(s)")
            for f in query_findings:
                print(f"    - [{f.severity}] {f.message}")
        except Exception as e:
            print(f"  Query checker ERROR: {e}")
            query_count = -1

        # Run Python graph-traversal analyzer
        analyzer = ANALYZERS.get(checker_name)
        if analyzer:
            try:
                py_findings = analyzer(proj)
                py_count = len(py_findings)
                print(f"  Python analyzer: {py_count} finding(s)")
                for f in py_findings:
                    print(f"    - {f['message']}")
            except Exception as e:
                print(f"  Python analyzer ERROR: {e}")
                py_count = -1
        else:
            print(f"  Python analyzer: NOT IMPLEMENTED")
            py_count = -1

        # Compare
        query_found = query_count >= min_expected
        py_found = py_count >= min_expected

        if query_found and py_found:
            status = "PASS"
            passed += 1
        elif query_found and not py_found:
            status = "FAIL (Python analyzer missed bugs)"
            failed += 1
        elif not query_found and py_found:
            status = "FAIL (Query checker missed bugs)"
            failed += 1
        else:
            status = "FAIL (Both missed bugs)"
            failed += 1

        print(f"  Result: {status}")
        print(f"  Query: {query_count} findings, Python: {py_count} findings, "
              f"Expected >= {min_expected}")

        results.append({
            "checker": checker_name,
            "status": status,
            "query_count": query_count,
            "py_count": py_count,
            "min_expected": min_expected,
        })

    # Summary
    print(f"\n{'='*60}")
    print(f"SUMMARY: {passed} passed, {failed} failed, "
          f"{len(results) - passed - failed} skipped")
    print(f"{'='*60}")

    for r in results:
        icon = "✓" if "PASS" in r["status"] else "✗" if "FAIL" in r["status"] else "○"
        print(f"  {icon} {r['checker']}: {r['status']}")

    return failed == 0


if __name__ == "__main__":
    success = run_differential_test()
    sys.exit(0 if success else 1)
```

**Step 2: Run the differential test inside Docker**

```bash
docker compose run --rm dev sh -c 'python3 tests/differential/test_checker_differential.py'
```

Expected: All 9 checkers pass. If any fail, proceed to Task 6.

**Step 3: Commit**

```bash
git add tests/differential/test_checker_differential.py
git commit -m "test: add backend differential test for query checkers vs Python analyzers"
```

---

## Task 6: Fix Backend Differential Test Failures

**Files:** Various (depends on failures found)

For each failure:

**Step 1: Analyze the failure**

Read the differential test output. Common failure modes:
- Python analyzer doesn't find the bug → analyzer logic needs adjustment
- Query checker doesn't find the bug → possible checker or LLVM IR issue
- Both miss the bug → C example doesn't trigger the pattern correctly

**Step 2: Fix the issue**

- If the Python analyzer misses: adjust the graph traversal logic in both `tests/differential/test_checker_differential.py` AND `playground/src/components/AnalyzerPanel.tsx`
- If the C example is wrong: fix in both `playground/src/examples/index.ts` AND `tests/programs/c/checker_*.c`, then recompile LLVM IR

**Step 3: Re-run differential test**

```bash
docker compose run --rm dev sh -c 'python3 tests/differential/test_checker_differential.py'
```

**Step 4: Commit fixes**

```bash
git add -A
git commit -m "fix: resolve backend differential test failures"
```

---

## Task 7: Set Up Playwright for Playground

**Files:**
- Modify: `playground/package.json` (add playwright devDep)
- Create: `playground/playwright.config.ts`
- Create: `playground/e2e/differential.spec.ts`

**Step 1: Install Playwright**

```bash
cd playground && npm install -D @playwright/test
npx playwright install chromium
```

**Step 2: Create Playwright config**

Create `playground/playwright.config.ts`:

```typescript
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  timeout: 120_000,
  retries: 1,
  use: {
    baseURL: 'http://localhost:5173',
    headless: true,
  },
  webServer: {
    command: 'npm run dev',
    port: 5173,
    reuseExistingServer: true,
    timeout: 30_000,
  },
});
```

**Step 3: Commit**

```bash
git add playground/package.json playground/playwright.config.ts
git commit -m "chore(playground): add Playwright E2E test infrastructure"
```

---

## Task 8: Playwright Differential Test

**Files:**
- Create: `playground/e2e/differential.spec.ts`

**Step 1: Write the Playwright differential test**

This test for each checker example:
1. Selects the example from the dropdown
2. Compiles and runs analysis
3. Runs the query checker via the Query tab
4. Runs the matching Python analyzer via the Analyzer tab
5. Compares: both should find >= 1 finding

```typescript
import { test, expect } from '@playwright/test';

// Map example slug -> { checker name, analyzer template key }
const CHECKER_EXAMPLES = [
  { slug: 'use_after_free', checker: 'use-after-free', template: 'detect_uaf_full' },
  { slug: 'memory_leak', checker: 'memory-leak', template: 'detect_memory_leak' },
  { slug: 'double_free', checker: 'double-free', template: 'detect_double_free' },
  { slug: 'null_deref', checker: 'null-deref', template: 'detect_null_deref' },
  { slug: 'file_descriptor_leak', checker: 'file-descriptor-leak', template: 'detect_file_leak' },
  { slug: 'lock_not_released', checker: 'lock-not-released', template: 'detect_lock_leak' },
  { slug: 'uninit_use', checker: 'uninit-use', template: 'detect_uninit_use' },
  { slug: 'stack_escape', checker: 'stack-escape', template: 'detect_stack_escape' },
  { slug: 'generic_resource_leak', checker: 'generic-resource-leak', template: 'detect_resource_leak' },
];

test.describe('Differential testing: Query checkers vs Python analyzers', () => {
  for (const { slug, checker, template } of CHECKER_EXAMPLES) {
    test(`${checker} example: query and python analyzer both find bugs`, async ({ page }) => {
      // Navigate to playground
      await page.goto('/');

      // Select the example from the dropdown
      const exampleSelect = page.locator('select').first();
      await exampleSelect.selectOption(slug);

      // Click Compile & Analyze button
      const compileBtn = page.getByRole('button', { name: /compile|analyze/i });
      await compileBtn.click();

      // Wait for analysis to complete (status shows "ready" or results appear)
      await page.waitForSelector('[class*="status"]', { timeout: 60_000 });
      // Wait a bit for analysis to finish
      await page.waitForTimeout(5000);

      // --- Query checker path ---
      // Click on the Query tab
      const queryTab = page.getByRole('tab', { name: /query/i }).or(
        page.locator('button').filter({ hasText: /query/i })
      );
      await queryTab.click();

      // Select the specific checker from dropdown
      const checkerSelect = page.locator('select').filter({ hasText: checker }).or(
        page.locator(`option[value="${checker}"]`).locator('..')
      );
      // Wait for checks to load
      await page.waitForTimeout(2000);

      // Find and select the checker
      const queryDropdown = page.locator('.query-panel select, .checks-panel select').first();
      if (await queryDropdown.isVisible()) {
        await queryDropdown.selectOption(checker);
      }

      // Click Run Check
      const runCheckBtn = page.getByRole('button', { name: /run check/i });
      if (await runCheckBtn.isVisible()) {
        await runCheckBtn.click();
        await page.waitForTimeout(3000);
      }

      // Count query findings
      const queryFindings = page.locator('.finding-card, .query-finding, [class*="finding"]');
      const queryCount = await queryFindings.count();
      console.log(`${checker}: Query found ${queryCount} finding(s)`);

      // --- Python analyzer path ---
      // Click on the Python Analyzer tab
      const analyzerTab = page.getByRole('tab', { name: /python|analyzer/i }).or(
        page.locator('button').filter({ hasText: /python|analyzer/i })
      );
      await analyzerTab.click();

      // Select the matching template
      const templateSelect = page.locator('.analyzer-panel select, .analyzer-toolbar select').first();
      if (await templateSelect.isVisible()) {
        await templateSelect.selectOption(template);
        await page.waitForTimeout(500);
      }

      // Click Run
      const runBtn = page.locator('.analyzer-panel .btn-run, .analyzer-toolbar .btn-run').first();
      if (await runBtn.isVisible()) {
        await runBtn.click();
        // Wait for Pyodide + analysis + script execution
        await page.waitForTimeout(15000);
      }

      // Switch to Findings tab in analyzer output
      const findingsTab = page.locator('.analyzer-output-tab').filter({ hasText: /findings/i });
      if (await findingsTab.isVisible()) {
        await findingsTab.click();
      }

      // Count Python analyzer findings
      const pyFindings = page.locator('.finding-card, [class*="finding"]');
      const pyCount = await pyFindings.count();
      console.log(`${checker}: Python analyzer found ${pyCount} finding(s)`);

      // Both should find at least 1 bug
      expect(queryCount, `Query checker ${checker} should find >= 1 bug`).toBeGreaterThanOrEqual(1);
      expect(pyCount, `Python analyzer for ${checker} should find >= 1 bug`).toBeGreaterThanOrEqual(1);
    });
  }
});
```

**Step 2: Run the Playwright tests**

```bash
cd playground && npx playwright test e2e/differential.spec.ts --reporter=list
```

Expected: All 9 tests pass. If any fail, proceed to Task 9.

**Step 3: Commit**

```bash
git add playground/e2e/differential.spec.ts
git commit -m "test(playground): add Playwright differential test for all 9 checkers"
```

---

## Task 9: Fix Playwright Test Failures

**Files:** Various (depends on failures)

Common Playwright-specific failure modes:
1. **Selector mismatches**: The HTML structure doesn't match expected selectors → inspect the actual DOM and update selectors
2. **Timing**: Analysis takes longer than expected → increase timeouts
3. **Compilation fails**: Example C code can't compile via Godbolt → simplify the example
4. **Python analyzer errors**: Pyodide-specific issues → debug via console output
5. **Template not in dropdown**: Template key doesn't match → verify key in TEMPLATES object

**Step 1: Debug failing tests**

Run with headed mode and screenshots:
```bash
cd playground && npx playwright test --headed --reporter=list
```

**Step 2: Fix issues and re-run**

**Step 3: Final commit**

```bash
git add -A
git commit -m "fix(playground): resolve Playwright differential test failures"
```

---

## Task 10: Update PROGRESS.md

**Files:**
- Modify: `plans/PROGRESS.md`

**Step 1: Update plan index**

Add entry:
```
| 156 | playground-checker-examples-differential-testing | playground | done |
```

**Step 2: Update session log**

Add:
```
### 2026-02-23 — Plan 156: Playground checker examples + differential testing
- Added 8 new C example programs (one per checker, UAF already existed)
- Added 9 Python analyzer templates for all query checkers
- Created backend differential test (Python SDK vs graph-traversal analyzers)
- Created Playwright E2E differential test
- Fixed N bugs found by differential testing
```

**Step 3: Commit**

```bash
git add plans/PROGRESS.md plans/156-playground-checker-examples-differential-testing.md
git commit -m "docs: update PROGRESS.md with Plan 156 completion"
```
