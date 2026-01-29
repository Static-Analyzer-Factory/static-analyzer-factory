"""Acceptance tests for the SAF Python SDK.

These tests verify the high-level user-facing API works as specified in the SRS.
"""

import pytest


class TestProjectOpen:
    """Tests for Project.open()."""

    def test_open_nonexistent_file_raises_frontend_error(self, tmp_path):
        """Project.open() should raise FrontendError for missing files."""
        from saf import Project
        from saf.exceptions import FrontendError

        with pytest.raises(FrontendError) as exc_info:
            Project.open(str(tmp_path / "nonexistent.air.json"))

        assert hasattr(exc_info.value, "code") or "not found" in str(exc_info.value).lower()

    def test_open_invalid_json_raises_frontend_error(self, tmp_path):
        """Project.open() should raise FrontendError for invalid JSON."""
        from saf import Project
        from saf.exceptions import FrontendError

        invalid_file = tmp_path / "invalid.air.json"
        invalid_file.write_text("{ not valid json")

        with pytest.raises(FrontendError):
            Project.open(str(invalid_file))


class TestTaintFlow:
    """Tests for taint_flow() query."""

    def test_simple_taint_flow(self, taint_simple_path: str):
        """taint_flow() should find flows from source to sink."""
        from saf import Project, sources, sinks

        proj = Project.open(taint_simple_path)
        q = proj.query()

        # Find flows from main's first parameter to dangerous_sink's first argument
        # We use arg_index=0 because we want to detect when tainted data
        # flows INTO the dangerous function, not the call result
        findings = q.taint_flow(
            sources=sources.function_param("main", 0),
            sinks=sinks.call("dangerous_sink", arg_index=0),
        )

        # Should find at least one flow
        assert len(findings) >= 1

    def test_finding_has_required_attributes(self, taint_simple_path: str):
        """Finding objects should have source_location, sink_location, and trace."""
        from saf import Project, sources, sinks

        proj = Project.open(taint_simple_path)
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.function_param("main", 0),
            sinks=sinks.call("dangerous_sink", arg_index=0),
        )

        assert len(findings) >= 1
        f = findings[0]

        # Check required attributes exist
        assert hasattr(f, "source_location")
        assert hasattr(f, "sink_location")
        assert hasattr(f, "trace")
        assert hasattr(f, "finding_id")

        # Check they have values
        assert f.source_location is not None
        assert f.sink_location is not None
        assert f.trace is not None
        assert f.finding_id is not None

    def test_trace_has_steps(self, taint_simple_path: str):
        """Trace should have steps showing the flow path."""
        from saf import Project, sources, sinks

        proj = Project.open(taint_simple_path)
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.function_param("main", 0),
            sinks=sinks.call("dangerous_sink", arg_index=0),
        )

        assert len(findings) >= 1
        trace = findings[0].trace

        # Trace should have steps
        assert hasattr(trace, "steps")
        assert len(trace.steps) >= 1


class TestSelectors:
    """Tests for selector functionality."""

    def test_selector_or_combines(self):
        """Selectors should combine with | operator."""
        from saf import sources

        s1 = sources.function_param("main", 0)
        s2 = sources.function_param("process", 0)
        combined = s1 | s2

        # Combined should be a SelectorSet
        assert hasattr(combined, "__len__")
        assert len(combined) == 2

    def test_selector_set_or_extends(self):
        """SelectorSet should extend when combined with |."""
        from saf import sources

        s1 = sources.function_param("main", 0)
        s2 = sources.function_param("process", 0)
        s3 = sources.function_param("read", 0)

        combined = s1 | s2 | s3
        assert len(combined) == 3


class TestSanitizers:
    """Tests for sanitizer functionality (Phase 2)."""

    def test_sanitizer_blocks_flow(self, fixtures_dir):
        """Sanitizers should block taint propagation."""
        from saf import Project, sources, sinks, sanitizers

        # Load fixture with sanitizer in the path
        # The fixture has: user_input -> copy to data_copy -> arg to both calls
        path = str(fixtures_dir / "taint_sanitizer.air.json")
        proj = Project.open(path)
        q = proj.query()

        # Without sanitizer: flow should exist
        findings_no_san = q.taint_flow(
            sources=sources.function_param("main", 0),
            sinks=sinks.call("dangerous_sink", arg_index=0),
        )

        # With sanitizer: flow should be blocked
        # The sanitizer's argument (same value that goes to sink) acts as a sanitizer point
        findings_with_san = q.taint_flow(
            sources=sources.function_param("main", 0),
            sinks=sinks.call("dangerous_sink", arg_index=0),
            sanitizers=sanitizers.arg_to("sanitize_input", 0),
        )

        # Without sanitizer should find flow
        assert len(findings_no_san) >= 1

        # With sanitizer, the intermediate value is blocked, so no flow to sink
        assert len(findings_with_san) == 0


class TestSchema:
    """Tests for schema() functionality (Phase 2)."""

    def test_schema_returns_dict(self, taint_simple_path: str):
        """schema() should return a dictionary."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        schema = proj.schema()

        assert isinstance(schema, dict)

    def test_schema_has_required_keys(self, taint_simple_path: str):
        """schema() should have required top-level keys."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        schema = proj.schema()

        required_keys = [
            "tool_version",
            "schema_version",
            "frontends",
            "graphs",
            "queries",
            "selectors",
            "config",
        ]

        for key in required_keys:
            assert key in schema, f"Missing key: {key}"

    def test_schema_selectors_has_categories(self, taint_simple_path: str):
        """schema()['selectors'] should have sources, sinks, sanitizers."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        schema = proj.schema()

        assert "sources" in schema["selectors"]
        assert "sinks" in schema["selectors"]
        assert "sanitizers" in schema["selectors"]

    def test_schema_queries_has_taint_flow(self, taint_simple_path: str):
        """schema()['queries'] should describe taint_flow."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        schema = proj.schema()

        assert "taint_flow" in schema["queries"]
        taint_flow = schema["queries"]["taint_flow"]

        assert "description" in taint_flow
        assert "parameters" in taint_flow
        assert "example" in taint_flow


class TestVersion:
    """Tests for version function."""

    def test_version_returns_string(self):
        """version() should return a version string."""
        from saf import version

        v = version()
        assert isinstance(v, str)
        assert len(v) > 0
        # Should be semver-ish
        assert "." in v


class TestAirModule:
    """Tests for AIR module access (Phase 3)."""

    def test_air_module_has_functions(self, taint_simple_path: str):
        """AirModule should provide access to functions."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        air = proj.air()

        # Should have function_count property
        assert hasattr(air, "function_count")
        assert air.function_count >= 1

        # Should have function_names method
        names = air.function_names()
        assert isinstance(names, list)
        assert "main" in names

    def test_air_get_function(self, taint_simple_path: str):
        """AirModule.get_function() should return function details."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        air = proj.air()

        func = air.get_function("main")
        assert func is not None
        assert func.name == "main"
        assert hasattr(func, "param_count")
        assert hasattr(func, "block_count")


class TestGraphStore:
    """Tests for graph store access (Phase 3)."""

    def test_graphs_available_lists_graph_types(self, taint_simple_path: str):
        """GraphStore.available() should list available graph types."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        graphs = proj.graphs()

        available = graphs.available()
        assert isinstance(available, list)
        assert "cfg" in available
        assert "callgraph" in available
        assert "defuse" in available
        assert "valueflow" in available

    def test_graphs_export_callgraph(self, taint_simple_path: str):
        """GraphStore.export('callgraph') should export call graph data."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        graphs = proj.graphs()

        cg = graphs.export("callgraph")
        assert isinstance(cg, dict)
        # The export includes nodes and edges lists
        assert "nodes" in cg
        assert "edges" in cg

    def test_graphs_export_valueflow(self, taint_simple_path: str):
        """GraphStore.export('valueflow') should export value flow data."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        graphs = proj.graphs()

        vf = graphs.export("valueflow")
        assert isinstance(vf, dict)
        assert "nodes" in vf
        assert "edges" in vf


class TestQueryMidLevel:
    """Tests for Query mid-level methods (Phase 3)."""

    def test_query_export_graph_invalid_raises(self, taint_simple_path: str):
        """query.export_graph() with invalid name should raise ValueError."""
        from saf import Project

        proj = Project.open(taint_simple_path)
        q = proj.query()

        with pytest.raises(ValueError):
            q.export_graph("invalid")


class TestFindingExport:
    """Tests for Finding export functionality (Phase 4)."""

    def test_finding_to_dict(self, taint_simple_path: str):
        """Finding.to_dict() should return a dictionary representation."""
        from saf import Project, sources, sinks

        proj = Project.open(taint_simple_path)
        q = proj.query()

        findings = q.taint_flow(
            sources=sources.function_param("main", 0),
            sinks=sinks.call("dangerous_sink", arg_index=0),
        )

        assert len(findings) >= 1
        f = findings[0]

        d = f.to_dict()
        assert isinstance(d, dict)
        assert "finding_id" in d
        assert "source_location" in d
        assert "sink_location" in d
        assert "source_id" in d
        assert "sink_id" in d
        assert "trace" in d
        assert isinstance(d["trace"], list)
        # Verify trace has steps
        assert len(d["trace"]) >= 1
        step = d["trace"][0]
        assert "from_id" in step
        assert "edge" in step
        assert "to_id" in step
