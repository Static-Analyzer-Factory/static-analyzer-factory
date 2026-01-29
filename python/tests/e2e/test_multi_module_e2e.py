"""E2E tests for Plan 011: Multi-module and interprocedural patterns.

Tests validate SAF's ability to track taint flow across module boundaries,
through library wrappers, and across deep call chains using the Python SDK.
"""

import pytest
from pathlib import Path


class TestCrossModuleTaint:
    """Tainted module_a_get_input(argv) flows to module_b_execute → system()."""

    def test_cross_module_taint_flow(self, e2e_fixtures_dir: Path):
        from saf import Project, sources, sinks

        proj = Project.open(str(e2e_fixtures_dir / "cross_module_taint.ll"))
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.call("module_a_get_input"),
            sinks=sinks.call("system", arg_index=0),
        )

        assert len(findings) >= 1, "should detect cross-module taint flow"


class TestCallbackChain:
    """Tainted argv flows through step_a → step_b → step_c → system()."""

    def test_deep_chain_taint_flow(self, e2e_fixtures_dir: Path):
        from saf import Project, sources, sinks

        proj = Project.open(str(e2e_fixtures_dir / "callback_chain.ll"))
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("system", arg_index=0),
        )

        assert len(findings) >= 1, "should detect interprocedural taint flow through callback chain"
