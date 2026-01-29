"""Shared pytest fixtures for SAF E2E tests."""

from pathlib import Path

import pytest


@pytest.fixture
def e2e_fixtures_dir() -> Path:
    """Return the path to the E2E test fixtures directory."""
    return Path(__file__).parent.parent.parent.parent / "tests" / "fixtures" / "llvm" / "e2e"
