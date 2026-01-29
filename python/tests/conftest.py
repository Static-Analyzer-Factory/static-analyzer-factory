"""Shared pytest fixtures for SAF tests."""

from pathlib import Path

import pytest


@pytest.fixture
def fixtures_dir() -> Path:
    """Return the path to the test fixtures directory."""
    return Path(__file__).parent / "fixtures"


@pytest.fixture
def taint_simple_path(fixtures_dir: Path) -> str:
    """Return the path to the simple taint test fixture."""
    return str(fixtures_dir / "taint_simple.air.json")
