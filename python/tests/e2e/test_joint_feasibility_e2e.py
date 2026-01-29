"""E2E tests for Plan 112: Joint path-feasibility filtering of MultiReach findings.

Tests validate that the joint feasibility filter correctly distinguishes
mutually exclusive double-frees (false positives) from real double-frees
(true positives) via the Python SDK.
"""

from pathlib import Path


class TestExclusiveDoubleFree:
    """Mutually exclusive frees on if/else branches — NOT a double-free."""

    def test_path_sensitive_runs(self, e2e_fixtures_dir: Path):
        from saf import Project

        proj = Project.open(str(e2e_fixtures_dir / "checker_double_free_exclusive.ll"))
        result = proj.check_all_path_sensitive()

        diag = result.diagnostics
        assert diag["total_findings"] == (
            diag["feasible_count"] + diag["infeasible_count"] + diag["unknown_count"]
        ), "Diagnostics should be consistent"

    def test_exclusive_frees_filtered(self, e2e_fixtures_dir: Path):
        from saf import Project

        proj = Project.open(str(e2e_fixtures_dir / "checker_double_free_exclusive.ll"))

        # Path-insensitive: may detect double-free
        pi_findings = proj.check("double-free")

        # Path-sensitive: joint feasibility should filter exclusive frees
        result = proj.check_all_path_sensitive()
        df_feasible = [f for f in result.feasible if f.checker == "double-free"]
        df_infeasible = [f for f in result.infeasible if f.checker == "double-free"]

        if len(pi_findings) > 0:
            # Exclusive frees should not remain in feasible without also being
            # partially classified as infeasible
            assert len(df_feasible) == 0 or len(df_infeasible) > 0, (
                f"Exclusive double-free should be filtered: "
                f"feasible={len(df_feasible)}, infeasible={len(df_infeasible)}"
            )


class TestRealDoubleFree:
    """Sequential frees on the same path — a REAL double-free."""

    def test_real_double_free_survives(self, e2e_fixtures_dir: Path):
        from saf import Project

        proj = Project.open(str(e2e_fixtures_dir / "checker_double_free.ll"))

        # Path-insensitive: should detect double-free
        pi_findings = proj.check("double-free")

        # Path-sensitive: should NOT filter a real double-free
        result = proj.check_all_path_sensitive()
        df_surviving = [
            f
            for f in list(result.feasible) + list(result.unknown)
            if f.checker == "double-free"
        ]

        if len(pi_findings) > 0:
            assert len(df_surviving) > 0, (
                f"Real double-free should survive filtering: "
                f"pi={len(pi_findings)}, surviving={len(df_surviving)}"
            )


class TestSinkTracesProperty:
    """Verify sink_traces getter on CheckerFinding."""

    def test_sink_traces_is_list(self, e2e_fixtures_dir: Path):
        from saf import Project

        proj = Project.open(str(e2e_fixtures_dir / "checker_double_free.ll"))
        findings = proj.check("double-free")
        for f in findings:
            st = f.sink_traces
            assert isinstance(st, list), "sink_traces should be a list"

    def test_sink_traces_in_to_dict(self, e2e_fixtures_dir: Path):
        from saf import Project

        proj = Project.open(str(e2e_fixtures_dir / "checker_double_free.ll"))
        findings = proj.check("double-free")
        for f in findings:
            d = f.to_dict()
            assert "sink_traces" in d, "to_dict() should include sink_traces"
            assert isinstance(d["sink_traces"], list)
