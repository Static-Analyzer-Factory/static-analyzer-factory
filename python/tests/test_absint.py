"""E2E tests for abstract interpretation and numeric checkers (Plan 045 / E26).

Tests cover:
- abstract_interp() API (smoke test)
- check_numeric() with custom checkers (division_by_zero, shift_count)
- error handling for unknown checkers
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


class TestAbstractInterpResult(unittest.TestCase):
    """Test abstract_interp() returns a result object."""

    def test_abstract_interp_returns_result(self) -> None:
        proj = saf.Project.open(fixture("absint_loop_bounds"))
        result = proj.abstract_interp()
        self.assertIsNotNone(result)


class TestDivisionByZeroChecker(unittest.TestCase):
    """Test division_by_zero checker via Python bindings."""

    def test_division_by_zero_finds_definite_errors(self) -> None:
        proj = saf.Project.open(fixture("absint_div_by_zero"))
        findings = proj.check_numeric("division_by_zero")

        # Should find at least one error
        errors = [f for f in findings if f.severity == "error"]
        self.assertGreater(
            len(errors), 0,
            "Expected at least one Error severity finding for definite division by zero"
        )


class TestShiftCountChecker(unittest.TestCase):
    """Test shift_count checker via Python bindings."""

    def test_shift_count_finds_definite_errors(self) -> None:
        proj = saf.Project.open(fixture("absint_shift_count"))
        findings = proj.check_numeric("shift_count")

        # Should find at least one error (x << 32, x >> 64, x >> -1)
        errors = [f for f in findings if f.severity == "error"]
        self.assertGreater(
            len(errors), 0,
            "Expected at least one Error severity finding for invalid shift count"
        )


class TestUnknownCheckerError(unittest.TestCase):
    """Test error handling for unknown checker names."""

    def test_unknown_checker_raises_value_error(self) -> None:
        proj = saf.Project.open(fixture("absint_loop_bounds"))
        with self.assertRaises(ValueError) as ctx:
            proj.check_numeric("nonexistent_checker")

        self.assertIn("Unknown numeric checker", str(ctx.exception))
        # Error message should list available checkers
        self.assertIn("buffer_overflow", str(ctx.exception))
        self.assertIn("division_by_zero", str(ctx.exception))
        self.assertIn("shift_count", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
