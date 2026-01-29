"""Tests for ProgramDatabase JSON protocol integration."""

import json

import saf


FIXTURE = "tests/fixtures/llvm/e2e/null_deref.ll"


def test_project_request_schema():
    """Test schema request returns check catalog and graph types."""
    proj = saf.Project.open(FIXTURE)
    resp = proj.request('{"action": "schema"}')
    data = json.loads(resp)
    assert data["status"] == "ok"
    assert "checks" in data
    assert any(c["name"] == "use_after_free" for c in data["checks"])


def test_project_request_check():
    """Test running a named check via JSON protocol."""
    proj = saf.Project.open(FIXTURE)
    resp = proj.request('{"action": "check", "name": "null_deref"}')
    data = json.loads(resp)
    assert data["status"] == "ok"
    assert "findings" in data


def test_project_request_unknown_check():
    """Test error response for unknown check name."""
    proj = saf.Project.open(FIXTURE)
    resp = proj.request('{"action": "check", "name": "nonexistent"}')
    data = json.loads(resp)
    assert data["status"] == "error"
    assert data["error"]["code"] == "UNKNOWN_CHECK"


def test_project_request_check_all():
    """Test running all checks via JSON protocol."""
    proj = saf.Project.open(FIXTURE)
    resp = proj.request('{"action": "check_all"}')
    data = json.loads(resp)
    assert data["status"] == "ok"
    assert "findings" in data
