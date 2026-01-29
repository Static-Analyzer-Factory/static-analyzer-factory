"""Tests for DisplayResolver, enriched Finding, and TraceStep gap-filling."""

import saf


FIXTURE = "python/tests/fixtures/taint_simple.air.json"


class TestPyDisplayResolver:
    """Tests for the DisplayResolver Python wrapper."""

    def test_display_resolver_from_project(self):
        """Project.display_resolver() returns a DisplayResolver instance."""
        proj = saf.Project.open(FIXTURE)
        resolver = proj.display_resolver()
        assert resolver is not None
        assert "DisplayResolver" in repr(resolver)

    def test_resolve_function_by_id(self):
        """resolve() resolves a function ID to its name."""
        proj = saf.Project.open(FIXTURE)
        resolver = proj.display_resolver()

        # The function "dangerous_sink" has ID 0x00...0100
        label = resolver.resolve("0x00000000000000000000000000000100")
        assert label is not None
        assert label.short_name == "dangerous_sink"
        assert label.kind == "function"

    def test_resolve_value_id(self):
        """resolve() resolves a value ID (function parameter) to its name."""
        proj = saf.Project.open(FIXTURE)
        resolver = proj.display_resolver()

        # The parameter "input" in main has ID 0x00...0001
        label = resolver.resolve("0x00000000000000000000000000000001")
        assert label is not None
        assert label.short_name is not None
        assert len(label.short_name) > 0

    def test_resolve_unknown_id(self):
        """resolve() returns a fallback label for unknown IDs."""
        proj = saf.Project.open(FIXTURE)
        resolver = proj.display_resolver()

        label = resolver.resolve("0xdeadbeefdeadbeefdeadbeefdeadbeef")
        assert label is not None
        assert label.kind == "unknown"
        assert len(label.short_name) > 0

    def test_resolve_batch(self):
        """resolve_batch() resolves multiple IDs at once."""
        proj = saf.Project.open(FIXTURE)
        resolver = proj.display_resolver()

        ids = [
            "0x00000000000000000000000000000100",  # dangerous_sink function
            "0x00000000000000000000000000000001",  # main param "input"
        ]
        labels = resolver.resolve_batch(ids)
        assert len(labels) == 2
        assert labels[0].short_name == "dangerous_sink"
        assert labels[0].kind == "function"

    def test_human_label_to_dict(self):
        """HumanLabel.to_dict() returns a dictionary with all fields."""
        proj = saf.Project.open(FIXTURE)
        resolver = proj.display_resolver()

        label = resolver.resolve("0x00000000000000000000000000000100")
        d = label.to_dict()
        assert isinstance(d, dict)
        assert "kind" in d
        assert "short_name" in d
        assert "long_name" in d
        assert "source_loc" in d
        assert "containing_function" in d
        assert d["short_name"] == "dangerous_sink"
        assert d["kind"] == "function"

    def test_human_label_repr(self):
        """HumanLabel.__repr__() returns a readable string."""
        proj = saf.Project.open(FIXTURE)
        resolver = proj.display_resolver()

        label = resolver.resolve("0x00000000000000000000000000000100")
        r = repr(label)
        assert "HumanLabel" in r
        assert "dangerous_sink" in r

    def test_resolve_block_id(self):
        """resolve() resolves a block ID."""
        proj = saf.Project.open(FIXTURE)
        resolver = proj.display_resolver()

        # Block "entry" has ID 0x00...1000
        label = resolver.resolve("0x00000000000000000000000000001000")
        assert label is not None
        assert label.kind == "block"


class TestEnrichedFinding:
    """Tests for enriched PyFinding with source_name/sink_name."""

    def test_finding_has_source_name(self):
        """Finding should have a source_name attribute."""
        proj = saf.Project.open(FIXTURE)
        q = proj.query()

        findings = q.taint_flow(
            sources=saf.sources.function_param("main", 0),
            sinks=saf.sinks.call("dangerous_sink", arg_index=0),
        )

        assert len(findings) >= 1
        f = findings[0]
        assert hasattr(f, "source_name")
        assert f.source_name is not None
        assert len(f.source_name) > 0

    def test_finding_has_sink_name(self):
        """Finding should have a sink_name attribute."""
        proj = saf.Project.open(FIXTURE)
        q = proj.query()

        findings = q.taint_flow(
            sources=saf.sources.function_param("main", 0),
            sinks=saf.sinks.call("dangerous_sink", arg_index=0),
        )

        assert len(findings) >= 1
        f = findings[0]
        assert hasattr(f, "sink_name")
        assert f.sink_name is not None
        assert len(f.sink_name) > 0

    def test_finding_to_dict_includes_names(self):
        """Finding.to_dict() should include source_name and sink_name."""
        proj = saf.Project.open(FIXTURE)
        q = proj.query()

        findings = q.taint_flow(
            sources=saf.sources.function_param("main", 0),
            sinks=saf.sinks.call("dangerous_sink", arg_index=0),
        )

        assert len(findings) >= 1
        d = findings[0].to_dict()
        assert "source_name" in d
        assert "sink_name" in d
        assert d["source_name"] is not None
        assert d["sink_name"] is not None


class TestTraceStepGapFilling:
    """Tests for DisplayResolver-based gap filling in trace steps."""

    def test_trace_steps_have_symbols(self):
        """Trace steps should have from_symbol/to_symbol filled via resolver."""
        proj = saf.Project.open(FIXTURE)
        q = proj.query()

        findings = q.taint_flow(
            sources=saf.sources.function_param("main", 0),
            sinks=saf.sinks.call("dangerous_sink", arg_index=0),
        )

        assert len(findings) >= 1
        trace = findings[0].trace
        assert len(trace.steps) >= 1

        # At least some steps should have symbols filled in
        has_from_symbol = any(s.from_symbol is not None for s in trace.steps)
        has_to_symbol = any(s.to_symbol is not None for s in trace.steps)
        assert has_from_symbol, "Expected at least one step with from_symbol"
        assert has_to_symbol, "Expected at least one step with to_symbol"

    def test_flows_also_enriched(self):
        """flows() should also produce enriched findings."""
        proj = saf.Project.open(FIXTURE)
        q = proj.query()

        findings = q.flows(
            sources=saf.sources.function_param("main", 0),
            sinks=saf.sinks.call("dangerous_sink", arg_index=0),
        )

        if len(findings) >= 1:
            f = findings[0]
            assert f.source_name is not None
            assert f.sink_name is not None
