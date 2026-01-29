"""E2E tests for Plan 007: Taint flow analysis via Python SDK.

Tests validate SAF's static analysis vulnerability detection capabilities
using the Python SDK API with realistic AIR-JSON fixtures.
"""

import pytest
from pathlib import Path


class TestCWE78ArgvToSystem:
    """Tainted argv parameter flows to dangerous sink."""

    def test_finds_taint_flow(self, e2e_fixtures_dir: Path):
        from saf import Project, sources, sinks

        proj = Project.open(str(e2e_fixtures_dir / "command_injection.ll"))
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("system", arg_index=0),
        )

        assert len(findings) >= 1, "should detect tainted data reaching sink"

    def test_finding_to_dict(self, e2e_fixtures_dir: Path):
        from saf import Project, sources, sinks

        proj = Project.open(str(e2e_fixtures_dir / "command_injection.ll"))
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("system", arg_index=0),
        )

        assert len(findings) >= 1
        d = findings[0].to_dict()

        assert isinstance(d, dict)
        assert "finding_id" in d
        assert "source_location" in d
        assert "sink_location" in d
        assert "trace" in d
        assert isinstance(d["trace"], list)


class TestSanitizerBlocksFlow:
    """Sanitizer taint_sanitized.ll fixture."""

    def test_sanitizer_blocks_flow(self, e2e_fixtures_dir: Path):
        from saf import Project, sources, sinks, sanitizers

        proj = Project.open(str(e2e_fixtures_dir / "taint_sanitized.ll"))
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("system", arg_index=0),
            sanitizers=sanitizers.arg_to("sanitize_input", 0),
        )

        assert len(findings) == 0, "with sanitizer, flow should be blocked"


class TestDeterminism:
    """Taint analysis should produce identical results across runs."""

    def test_deterministic_findings(self, e2e_fixtures_dir: Path):
        from saf import Project, sources, sinks

        path = str(e2e_fixtures_dir / "command_injection.ll")

        proj1 = Project.open(path)
        q1 = proj1.query()
        findings1 = q1.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("system", arg_index=0),
        )

        proj2 = Project.open(path)
        q2 = proj2.query()
        findings2 = q2.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("system", arg_index=0),
        )

        assert len(findings1) == len(findings2), "deterministic finding count"
        for f1, f2 in zip(findings1, findings2):
            assert f1.finding_id == f2.finding_id, "deterministic finding IDs"
