"""E2E tests for IFDS taint analysis via the Python SDK.

These tests load compiled LLVM IR fixtures and run the IFDS taint analysis
through the Python bindings.

Source files: tests/fixtures/sources/ifds_* (C, C++, Rust)
Compiled fixtures: tests/fixtures/llvm/e2e/ifds_*.ll
"""

from pathlib import Path

import pytest


FIXTURES_DIR = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "llvm" / "e2e"


@pytest.fixture
def simple_taint_ll() -> str:
    """Path to simple taint .ll fixture (getenv → system)."""
    return str(FIXTURES_DIR / "ifds_simple_taint.ll")


class TestIfdsTaint:
    """Tests for Project.ifds_taint() IFDS analysis."""

    def test_simple_taint_reaches_sink(self, simple_taint_ll: str):
        """getenv() → system(): taint should reach the sink."""
        from saf import Project, sources, sinks

        proj = Project.open(simple_taint_ll)
        result = proj.ifds_taint(
            sources=sources.call("getenv"),
            sinks=sinks.call("system", arg_index=0),
        )

        assert result.has_taint_at_sink(sinks.call("system", arg_index=0))


class TestIfdsResult:
    """Tests for IfdsResult object methods."""

    def test_diagnostics_returns_dict(self, simple_taint_ll: str):
        """diagnostics() should return a dict with solver stats."""
        from saf import Project, sources, sinks

        proj = Project.open(simple_taint_ll)
        result = proj.ifds_taint(
            sources=sources.call("getenv"),
            sinks=sinks.call("system", arg_index=0),
        )

        diag = result.diagnostics()
        assert isinstance(diag, dict)
        assert "iterations" in diag
        assert "path_edges_explored" in diag
        assert "summary_edges_created" in diag
        assert "facts_at_peak" in diag
        assert "reached_limit" in diag
        assert diag["iterations"] > 0

    def test_export_returns_dict(self, simple_taint_ll: str):
        """export() should return a serializable dict."""
        from saf import Project, sources, sinks

        proj = Project.open(simple_taint_ll)
        result = proj.ifds_taint(
            sources=sources.call("getenv"),
            sinks=sinks.call("system", arg_index=0),
        )

        export = result.export()
        assert isinstance(export, dict)
        assert "facts" in export
        assert "summaries" in export
        assert "diagnostics" in export
