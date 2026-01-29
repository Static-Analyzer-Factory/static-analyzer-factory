"""E2E tests for the checker framework Python bindings (Plan 028).

Reduced test suite covering core functionality:
- check_custom() with user-defined resource pairs
- checker_schema() returns expected structure
- resource_table() access and extension
- CheckerFinding properties
- Checker constants
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


class TestCheckCustom(unittest.TestCase):
    """Test check_custom() with user-defined checker specs."""

    def test_custom_leak_checker(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        findings = proj.check_custom(
            "custom-leak",
            mode="must_not_reach",
            source_role="allocator",
            source_match_return=True,
            sink_is_exit=True,
            sanitizer_role="deallocator",
            sanitizer_match_return=False,
            cwe=772,
            severity="warning",
        )
        self.assertIsInstance(findings, list)

    def test_custom_checker_invalid_role(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        with self.assertRaises(ValueError):
            proj.check_custom(
                "bad",
                source_role="nonexistent_role",
            )


class TestCheckerSchema(unittest.TestCase):
    """Test checker_schema() returns expected structure."""

    def test_schema_has_checkers(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        schema = proj.checker_schema()
        self.assertIn("checkers", schema)
        self.assertIn("count", schema)
        self.assertEqual(schema["count"], 9)

    def test_schema_checker_fields(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        schema = proj.checker_schema()
        for checker in schema["checkers"]:
            self.assertIn("name", checker)
            self.assertIn("description", checker)
            self.assertIn("mode", checker)
            self.assertIn("severity", checker)

    def test_schema_includes_memory_leak(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        schema = proj.checker_schema()
        names = [c["name"] for c in schema["checkers"]]
        self.assertIn("memory-leak", names)
        self.assertIn("use-after-free", names)
        self.assertIn("double-free", names)
        self.assertIn("null-deref", names)
        self.assertIn("file-descriptor-leak", names)


class TestCheckerDiagnostics(unittest.TestCase):
    """Test checker_diagnostics() returns stats."""

    def test_diagnostics_returns_dict(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        diag = proj.checker_diagnostics()
        self.assertIsInstance(diag, dict)
        self.assertIn("checkers_run", diag)
        self.assertIn("classified_sites", diag)
        self.assertIn("total_findings", diag)


class TestResourceTable(unittest.TestCase):
    """Test resource_table() access and extension."""

    def test_resource_table_has_entries(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        table = proj.resource_table()
        self.assertGreater(table.size, 0)

    def test_resource_table_has_malloc(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        table = proj.resource_table()
        self.assertTrue(table.has_role("malloc", "allocator"))
        self.assertTrue(table.has_role("free", "deallocator"))

    def test_resource_table_add_custom(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        table = proj.resource_table()
        table.add("pool_alloc", "allocator")
        table.add("pool_free", "deallocator")
        self.assertTrue(table.has_role("pool_alloc", "allocator"))
        self.assertTrue(table.has_role("pool_free", "deallocator"))

    def test_resource_table_function_names(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        table = proj.resource_table()
        names = table.function_names()
        self.assertIsInstance(names, list)
        self.assertIn("malloc", names)
        self.assertIn("free", names)

    def test_resource_table_export(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        table = proj.resource_table()
        exported = table.export()
        self.assertIsInstance(exported, list)
        self.assertGreater(len(exported), 0)

        # Each entry should have name and roles
        for entry in exported:
            self.assertIn("name", entry)
            self.assertIn("roles", entry)


class TestCheckerFinding(unittest.TestCase):
    """Test CheckerFinding properties."""

    def test_finding_properties(self) -> None:
        proj = saf.Project.open(fixture("checker_leak_simple"))
        findings = proj.check_all()
        # If findings exist, verify their properties
        for f in findings:
            self.assertIsInstance(f.checker, str)
            self.assertIsInstance(f.severity, str)
            self.assertIn(f.severity, ["info", "warning", "error", "critical"])
            self.assertIsInstance(f.message, str)
            self.assertIsInstance(f.source, str)
            self.assertIsInstance(f.sink, str)
            self.assertIsInstance(f.trace, list)
            # CWE is optional
            if f.cwe is not None:
                self.assertIsInstance(f.cwe, int)


class TestCheckerConstants(unittest.TestCase):
    """Test module-level checker constants."""

    def test_role_constants(self) -> None:
        self.assertEqual(saf.Allocator, "allocator")
        self.assertEqual(saf.Deallocator, "deallocator")
        self.assertEqual(saf.Acquire, "acquire")
        self.assertEqual(saf.Release, "release")


if __name__ == "__main__":
    unittest.main()
