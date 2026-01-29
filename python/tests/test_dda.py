"""Smoke test for demand-driven PTA (mirrors Rust dda_e2e.rs)."""

from pathlib import Path

import pytest


FIXTURES_DIR = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "llvm" / "e2e"


@pytest.fixture
def basic_query_ll() -> str:
    """Load the basic DDA query test fixture."""
    return str(FIXTURES_DIR / "dda_basic_query.ll")


def test_dda_diagnostics_smoke(basic_query_ll: str):
    """Smoke test: DDA builds and diagnostics dict has expected keys."""
    from saf import Project

    proj = Project.open(basic_query_ll)
    dda = proj.demand_pta()
    diag = dda.diagnostics()

    assert isinstance(diag, dict)
    assert "queries" in diag
    assert "cache_hits" in diag
    assert "total_steps" in diag
