"""E2E tests for Plan 008: Memory safety patterns via Python SDK.

Tests validate SAF's analysis of memory safety vulnerability patterns
using the Python SDK API with realistic AIR-JSON fixtures.
"""

import pytest
from pathlib import Path


class TestCWE416UseAfterFree:
    """Allocated memory flows to free, then is dereferenced."""

    def test_malloc_to_free_flow(self, e2e_fixtures_dir: Path):
        from saf import Project, sources, sinks

        proj = Project.open(str(e2e_fixtures_dir / "use_after_free.ll"))
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.call("malloc"),
            sinks=sinks.call("free", arg_index=0),
        )

        assert len(findings) >= 1, "should detect alloc flowing to free"


class TestCWE476NullDeref:
    """Malloc result used without null check."""

    def test_loads_and_builds(self, e2e_fixtures_dir: Path):
        from saf import Project

        proj = Project.open(str(e2e_fixtures_dir / "null_deref.ll"))
        graphs = proj.graphs()
        vf = graphs.export("valueflow")

        assert isinstance(vf, dict)
        assert len(vf["nodes"]) > 0
        assert len(vf["edges"]) > 0
