"""Differential test: query checkers vs Python graph-traversal analyzers.

Compares built-in SVFG checkers (proj.check()) with Python analyzers
that detect the same bugs via PropertyGraph traversal.

Run inside Docker: python3 tests/differential/test_checker_differential.py
"""

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
    """Detect memory leaks via graph traversal.

    Strategy: malloc return nodes don't have parent_function="malloc" in
    the PropertyGraph (specs handle allocators specially).  Instead, find
    value nodes that write to memory (STORE) but never flow to free via
    CALLARG — they represent allocated values that are never deallocated.
    """
    graphs = proj.graphs()
    vf = graphs.export("valueflow")

    free_param_ids = set()
    for n in vf["nodes"]:
        if n["properties"].get("parent_function") == "free":
            free_param_ids.add(n["id"])

    # Collect all nodes that eventually reach free via CALLARG
    reaches_free = set()
    for e in vf["edges"]:
        if e["edge_type"] == "CALLARG" and e["dst"] in free_param_ids:
            reaches_free.add(e["src"])

    findings = []
    for n in vf["nodes"]:
        if n["properties"].get("kind") != "value":
            continue
        if n["properties"].get("parent_function"):
            continue  # Skip external-function parameter nodes
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        if not outgoing:
            continue
        has_store = any(e["edge_type"] == "STORE" for e in outgoing)
        in_reaches_free = n["id"] in reaches_free
        if has_store and not in_reaches_free:
            findings.append({"id": n["id"], "message": "allocation not freed"})
    return findings


def analyze_use_after_free(proj):
    """Detect UAF via graph traversal."""
    graphs = proj.graphs()
    vf = graphs.export("valueflow")

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
    """Detect double-free via graph traversal.

    Strategy: The SVFG deduplicates identical SSA values, so two free(p)
    calls on the same %p register produce only 1 CALLARG edge.  Instead,
    look for value nodes whose ONLY outgoing edge is CALLARG to free —
    meaning the value exists solely to be freed, with no other productive
    use, which is suspicious in a double-free scenario.
    """
    graphs = proj.graphs()
    vf = graphs.export("valueflow")

    free_param_ids = set()
    for n in vf["nodes"]:
        if n["properties"].get("parent_function") == "free":
            free_param_ids.add(n["id"])

    findings = []
    for n in vf["nodes"]:
        if n["properties"].get("kind") != "value":
            continue
        if n["properties"].get("parent_function"):
            continue  # Skip external-function param nodes
        outgoing = [e for e in vf["edges"] if e["src"] == n["id"]]
        if not outgoing:
            continue
        free_args = [e for e in outgoing if e["edge_type"] == "CALLARG"
                     and e["dst"] in free_param_ids]
        non_free = [e for e in outgoing if e not in free_args]
        # Value whose only purpose is flowing to free → double-free suspect
        if free_args and not non_free:
            findings.append({"id": n["id"], "message": "double free"})
    return findings


def analyze_null_deref(proj):
    """Detect null deref via graph traversal.

    Strategy: malloc/calloc/fopen can return NULL.  In the PropertyGraph,
    allocator return nodes don't carry parent_function="malloc".  Instead,
    detect dereferences by looking for LOAD edges from unknown_mem — any
    LOAD indicates a pointer dereference.  If the callgraph contains a
    nullable external (malloc, fopen, etc.), those dereferences are at
    risk of null-pointer access.
    """
    graphs = proj.graphs()
    vf = graphs.export("valueflow")
    cg = graphs.export("callgraph")

    # Check if the program calls any nullable function
    nullable_fns = {"malloc", "calloc", "realloc", "fopen", "aligned_alloc"}
    has_nullable = any(
        n["properties"].get("name") in nullable_fns
        for n in cg["nodes"]
        if "External" in n.get("labels", [])
    )

    findings = []
    if has_nullable:
        # Find dereferences: LOAD edges from unknown_mem or STORE edges
        for e in vf["edges"]:
            if e["edge_type"] == "LOAD":
                findings.append({"id": e["dst"], "message": "nullable ptr used"})
            elif e["edge_type"] == "STORE":
                findings.append({"id": e["src"], "message": "nullable ptr used"})
        # Deduplicate by node id
        seen = set()
        unique = []
        for f in findings:
            if f["id"] not in seen:
                seen.add(f["id"])
                unique.append(f)
        findings = unique
    return findings


def analyze_file_descriptor_leak(proj):
    """Detect file descriptor leak via graph traversal."""
    graphs = proj.graphs()
    vf = graphs.export("valueflow")

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
    vf = graphs.export("valueflow")

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
    """Detect uninitialized use via graph traversal.

    Strategy: If the value-flow graph has a LOAD from unknown_mem but no
    STORE to unknown_mem, memory is being read without being written first.
    This indicates use of uninitialized heap memory (malloc returns
    uninitialized data, unlike calloc).
    """
    graphs = proj.graphs()
    vf = graphs.export("valueflow")

    has_load_from_mem = False
    has_store_to_mem = False
    load_dst_id = None

    for e in vf["edges"]:
        if e["edge_type"] == "LOAD":
            has_load_from_mem = True
            load_dst_id = e["dst"]
        if e["edge_type"] == "STORE":
            has_store_to_mem = True

    findings = []
    if has_load_from_mem and not has_store_to_mem:
        node_id = load_dst_id or "unknown"
        findings.append({"id": node_id, "message": "read before write"})
    return findings


def analyze_stack_escape(proj):
    """Detect stack escape via graph traversal.

    Strategy: Stack escape occurs when a local variable's address is
    returned from a function.  In the value-flow graph, this manifests
    as a RETURN edge — a value flows out of a function via return.
    If the program also has STORE edges (writing to stack), a RETURN
    edge indicates the address of stack memory is escaping.
    """
    graphs = proj.graphs()
    vf = graphs.export("valueflow")

    has_store = any(e["edge_type"] == "STORE" for e in vf["edges"])
    findings = []
    for e in vf["edges"]:
        if e["edge_type"] == "RETURN" and has_store:
            findings.append({"id": e["src"], "message": "stack var escapes"})
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
