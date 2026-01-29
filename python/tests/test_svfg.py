"""Smoke test for SVFG (mirrors Rust svfg_e2e.rs)."""

from pathlib import Path

import pytest


FIXTURES_DIR = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "llvm" / "e2e"


@pytest.fixture
def store_load_disambig_ll() -> str:
    """Path to store/load disambiguation fixture."""
    return str(FIXTURES_DIR / "svfg_store_load_disambig.ll")


def test_svfg_build_smoke(store_load_disambig_ll: str):
    """Smoke test: SVFG builds and has nodes and edges."""
    from saf import Project

    proj = Project.open(store_load_disambig_ll)
    svfg = proj.svfg()
    assert svfg.node_count > 0
    assert svfg.edge_count > 0
