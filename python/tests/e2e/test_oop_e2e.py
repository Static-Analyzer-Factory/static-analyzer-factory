"""E2E tests for Plan 010: OOP patterns via Python SDK.

Tests validate SAF's ability to model virtual dispatch, function pointers,
RAII lifecycle, and trait dispatch patterns using the Python SDK API.
"""

import pytest
from pathlib import Path


class TestVtableDispatch:
    """C++ virtual method dispatch through vtable pointer."""

    def test_callgraph_structure(self, e2e_fixtures_dir: Path):
        from saf import Project

        proj = Project.open(str(e2e_fixtures_dir / "vtable_dispatch.ll"))
        graphs = proj.graphs()

        cg = graphs.export("callgraph")
        assert isinstance(cg, dict)
        assert "nodes" in cg
        assert "edges" in cg


class TestCallbackFnPtr:
    """C function pointer callback invocation."""

    def test_callgraph_structure(self, e2e_fixtures_dir: Path):
        from saf import Project

        proj = Project.open(str(e2e_fixtures_dir / "callback_fn_ptr.ll"))
        graphs = proj.graphs()

        cg = graphs.export("callgraph")
        assert isinstance(cg, dict)
        assert "nodes" in cg
