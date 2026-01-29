"""Smoke test for flow-sensitive PTA (mirrors Rust fspta_e2e.rs)."""

from pathlib import Path

import pytest


FIXTURES_DIR = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "llvm" / "e2e"


@pytest.fixture
def strong_update_ll() -> str:
    return str(FIXTURES_DIR / "fspta_strong_update.ll")


def test_fspta_diagnostics_smoke(strong_update_ll: str):
    """Smoke test: FS-PTA builds and diagnostics dict has expected keys."""
    from saf import Project

    proj = Project.open(strong_update_ll)
    result = proj.flow_sensitive_pta()
    diag = result.diagnostics()

    assert isinstance(diag, dict)
    assert "iterations" in diag
    assert "iteration_limit_hit" in diag
    assert not diag["iteration_limit_hit"]
    assert "strong_updates" in diag
    assert "fs_svfg_nodes" in diag
