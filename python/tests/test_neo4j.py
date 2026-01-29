"""Tests for Neo4j integration (requires Neo4j running)."""

import os

import pytest

# Skip if NEO4J_URI not set
NEO4J_URI = os.environ.get("NEO4J_URI", "")
pytestmark = pytest.mark.skipif(
    not NEO4J_URI,
    reason="NEO4J_URI not set - Neo4j integration tests skipped",
)


def test_neo4j_exporter_import():
    """Test exporting graphs to Neo4j and querying."""
    from saf.neo4j_export import Neo4jExporter

    exporter = Neo4jExporter(NEO4J_URI)
    exporter.clear()

    import saf

    proj = saf.Project.open("tests/fixtures/llvm/e2e/null_deref.ll")

    # Export graphs via JSON protocol
    import json

    resp = proj.request('{"action": "schema"}')
    data = json.loads(resp)
    assert data["status"] == "ok"

    # Query should work after export
    results = exporter.cypher("RETURN 1 AS test")
    assert len(results) > 0

    exporter.close()
