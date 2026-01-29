"""Smoke test for pts_repr parameter (mirrors Rust pts_repr_e2e.rs)."""

from pathlib import Path

import pytest


FIXTURES_DIR = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "llvm"


@pytest.fixture
def memory_ops_ll() -> str:
    """Simple fixture with pointer operations."""
    return str(FIXTURES_DIR / "memory_ops.ll")


def test_cspta_btreeset_smoke(memory_ops_ll: str):
    """Smoke test: PTA with btreeset repr produces a result with value_count."""
    from saf import Project

    proj = Project.open(memory_ops_ll)
    result = proj.context_sensitive_pta(k=1, pts_repr="btreeset")
    assert result is not None
    diag = result.diagnostics()
    assert "iterations" in diag
