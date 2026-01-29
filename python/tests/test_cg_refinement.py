"""E2E tests for call graph refinement via the Python SDK.

These tests load compiled LLVM IR fixtures and run the iterative
CHA + PTA call graph refinement through Python bindings.

Source files: tests/programs/c/cg_*, tests/programs/cpp/cg_*, tests/programs/rust/cg_*
Compiled fixtures: tests/fixtures/llvm/e2e/cg_*.ll
"""

from pathlib import Path

import pytest

FIXTURES_DIR = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "llvm" / "e2e"


@pytest.fixture
def fptr_callback_ll() -> str:
    return str(FIXTURES_DIR / "cg_fptr_callback.ll")


@pytest.fixture
def virtual_dispatch_ll() -> str:
    return str(FIXTURES_DIR / "cg_virtual_dispatch.ll")




class TestRefineCallGraph:
    """Tests for Project.refine_call_graph()."""

    def test_virtual_dispatch_has_cha(self, virtual_dispatch_ll: str):
        """C++ virtual dispatch produces a class hierarchy."""
        from saf import Project

        proj = Project.open(virtual_dispatch_ll)
        result = proj.refine_call_graph(entry_points="main")

        cha = result.class_hierarchy()
        assert cha is not None, "C++ program should produce a ClassHierarchy"

        hierarchy = cha.export()
        assert "classes" in hierarchy
        assert len(hierarchy["classes"]) > 0

    def test_resolved_sites_dict(self, fptr_callback_ll: str):
        """resolved_sites() returns a dict mapping site IDs to target lists."""
        from saf import Project

        proj = Project.open(fptr_callback_ll)
        result = proj.refine_call_graph(entry_points="main")

        sites = result.resolved_sites()
        assert isinstance(sites, dict)

    def test_deterministic(self, fptr_callback_ll: str):
        """Two refinement runs produce identical results."""
        from saf import Project

        proj = Project.open(fptr_callback_ll)
        r1 = proj.refine_call_graph(entry_points="main")
        r2 = proj.refine_call_graph(entry_points="main")

        assert r1.iterations == r2.iterations
        assert r1.call_graph_export() == r2.call_graph_export()
        assert r1.resolved_sites() == r2.resolved_sites()


class TestClassHierarchy:
    """Tests for ClassHierarchy Python API."""

    def test_subclasses_of(self, virtual_dispatch_ll: str):
        """subclasses_of returns a list of subclass names."""
        from saf import Project

        proj = Project.open(virtual_dispatch_ll)
        result = proj.refine_call_graph(entry_points="main")
        cha = result.class_hierarchy()
        assert cha is not None

        # Get all classes and find one that has subclasses
        hierarchy = cha.export()
        classes = hierarchy["classes"]
        assert len(classes) > 0

        # Every class should return a list (possibly empty) for subclasses_of
        for cls in classes:
            subs = cha.subclasses_of(cls)
            assert isinstance(subs, list)

    def test_resolve_virtual(self, virtual_dispatch_ll: str):
        """resolve_virtual returns function IDs for a given slot."""
        from saf import Project

        proj = Project.open(virtual_dispatch_ll)
        result = proj.refine_call_graph(entry_points="main")
        cha = result.class_hierarchy()
        assert cha is not None

        hierarchy = cha.export()
        vtables = hierarchy.get("vtables", {})

        # Find a class with vtable entries
        for class_name, slots in vtables.items():
            if slots:
                # Try resolving slot 0
                targets = cha.resolve_virtual(class_name, 0)
                assert isinstance(targets, list)
