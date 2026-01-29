"""Smoke test for Memory SSA (mirrors Rust mssa_e2e.rs)."""

from pathlib import Path

import pytest


FIXTURES_DIR = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "llvm" / "e2e"


@pytest.fixture
def store_load_ll() -> str:
    """Path to basic store/load fixture."""
    return str(FIXTURES_DIR / "mssa_store_load_simple.ll")


def test_mssa_build_smoke(store_load_ll: str):
    """Smoke test: MSSA builds and has accesses."""
    from saf import Project

    proj = Project.open(store_load_ll)
    mssa = proj.memory_ssa()
    assert mssa.access_count > 0
