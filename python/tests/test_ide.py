"""E2E tests for IDE typestate analysis via the Python SDK.

These tests load compiled LLVM IR fixtures and run typestate analysis
through the Python bindings.

Source files: tests/fixtures/sources/typestate_* (C, C++)
Compiled fixtures: tests/fixtures/llvm/e2e/typestate_*.ll
"""

from pathlib import Path

import pytest

FIXTURES_DIR = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "llvm" / "e2e"


@pytest.fixture
def double_close_ll() -> str:
    return str(FIXTURES_DIR / "typestate_double_close.ll")


class TestTypestateFinding:
    """Tests for TypestateFinding properties."""

    def test_finding_properties(self, double_close_ll: str) -> None:
        import saf

        proj = saf.Project.open(double_close_ll)
        result = proj.typestate("file_io")
        finding = result.findings()[0]

        assert isinstance(finding.resource, str)
        assert finding.resource.startswith("0x")
        assert isinstance(finding.state, str)
        assert isinstance(finding.inst, str)
        assert finding.inst.startswith("0x")
        assert isinstance(finding.kind, str)
        assert isinstance(finding.spec_name, str)



class TestTypestateSpec:
    """Tests for custom TypestateSpec creation."""

    def test_builtin_specs_list(self) -> None:
        import saf

        specs = saf.typestate_specs()
        assert "file_io" in specs
        assert "mutex_lock" in specs
        assert "memory_alloc" in specs

    def test_custom_spec_creation(self) -> None:
        import saf

        spec = saf.TypestateSpec(
            name="custom_resource",
            states=["uninit", "open", "closed", "error"],
            initial_state="open",
            error_states=["error"],
            accepting_states=["uninit", "closed"],
            transitions=[
                ("open", "close_resource", "closed"),
                ("closed", "close_resource", "error"),
            ],
            constructors=["create_resource"],
        )
        r = repr(spec)
        assert "custom_resource" in r

    def test_invalid_spec_raises(self) -> None:
        import saf

        with pytest.raises(ValueError, match="Invalid typestate spec"):
            saf.TypestateSpec(
                name="bad",
                states=["a", "b"],
                initial_state="nonexistent",
                error_states=[],
                accepting_states=["a"],
                transitions=[],
                constructors=["create"],
            )



class TestTypestateErrors:
    """Tests for error handling."""

    def test_unknown_spec_raises(self, double_close_ll: str) -> None:
        import saf

        proj = saf.Project.open(double_close_ll)
        with pytest.raises(ValueError, match="Unknown typestate spec"):
            proj.typestate("nonexistent_spec")
