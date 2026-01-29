"""E2E tests for path-sensitive checker Python bindings (Plan 032).

Tests cover:
- check_path_sensitive() with single and multiple checkers
- check_all_path_sensitive() running all built-in checkers
- filter_infeasible() post-filtering existing findings
- PathSensitiveResult properties (feasible, infeasible, unknown, diagnostics)
- Configuration parameters (z3_timeout_ms, max_guards)
- Determinism: repeated runs produce identical results
"""

import os
import unittest

import saf

FIXTURE_DIR = os.path.join(
    os.path.dirname(__file__),
    "..",
    "..",
    "tests",
    "fixtures",
    "llvm",
    "e2e",
)


def fixture(name: str) -> str:
    return os.path.join(FIXTURE_DIR, f"{name}.ll")


class TestCheckPathSensitive(unittest.TestCase):
    """Test check_path_sensitive() with named checkers."""

    def test_result_properties(self) -> None:
        proj = saf.Project.open(fixture("ps_null_guard"))
        result = proj.check_path_sensitive("null-deref")
        # All properties should be accessible
        self.assertIsInstance(result.feasible, list)
        self.assertIsInstance(result.infeasible, list)
        self.assertIsInstance(result.unknown, list)
        self.assertIsInstance(result.diagnostics, dict)
        self.assertIsInstance(result.total, int)
        self.assertIsInstance(result.false_positives_filtered, int)

    def test_diagnostics_keys(self) -> None:
        proj = saf.Project.open(fixture("ps_null_guard"))
        result = proj.check_path_sensitive("null-deref")
        diag = result.diagnostics
        expected_keys = [
            "total_findings",
            "feasible_count",
            "infeasible_count",
            "unknown_count",
            "guards_extracted",
            "z3_calls",
            "z3_timeouts",
            "skipped_too_many_guards",
        ]
        for key in expected_keys:
            self.assertIn(key, diag, f"Missing key: {key}")


class TestCheckAllPathSensitive(unittest.TestCase):
    """Test check_all_path_sensitive() with all built-in checkers."""

    def test_custom_z3_timeout(self) -> None:
        proj = saf.Project.open(fixture("ps_null_guard"))
        result = proj.check_all_path_sensitive(z3_timeout_ms=500)
        self.assertIsInstance(result, saf.PathSensitiveResult)


class TestFilterInfeasible(unittest.TestCase):
    """Test filter_infeasible() post-filtering."""

    def test_filter_preserves_findings(self) -> None:
        proj = saf.Project.open(fixture("ps_true_positive"))
        # Get path-insensitive findings first
        findings = proj.check_all()
        # Filter them
        result = proj.filter_infeasible(findings)
        self.assertEqual(result.total, len(findings))


class TestDeterminism(unittest.TestCase):
    """Test that path-sensitive analysis produces deterministic results."""

    def test_repeated_runs_identical(self) -> None:
        proj = saf.Project.open(fixture("ps_true_positive"))
        r1 = proj.check_all_path_sensitive()
        r2 = proj.check_all_path_sensitive()
        self.assertEqual(len(r1.feasible), len(r2.feasible))
        self.assertEqual(len(r1.infeasible), len(r2.infeasible))
        self.assertEqual(len(r1.unknown), len(r2.unknown))
        self.assertEqual(r1.total, r2.total)


if __name__ == "__main__":
    unittest.main()
