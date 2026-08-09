"""Focused tests for the SAF BenchExec tool-info module (plan 192, P0.2).

Requires ``benchexec`` to be importable; skipped otherwise (so it does not couple
the main ``pytest python/tests`` run to a benchexec install).

Run with: ``pytest benchexec/tools/test_saf.py`` after ``pip install benchexec``.
"""

import importlib.util
import os

import pytest

pytest.importorskip("benchexec")

import benchexec.result as result  # noqa: E402


def _load_tool():
    here = os.path.dirname(__file__)
    spec = importlib.util.spec_from_file_location(
        "saf_toolinfo", os.path.join(here, "saf.py")
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.Tool()


class _FakeTask:
    def __init__(self, property_file, data_model, input_file):
        self.property_file = property_file
        self.options = {"data_model": data_model} if data_model else {}
        self.single_input_file = input_file


class _FakeRun:
    def __init__(self, output):
        self.output = output


def test_name():
    assert _load_tool().name() == "SAF"


def test_cmdline_builds_expected_argv():
    tool = _load_tool()
    task = _FakeTask("p.prp", "ILP32", "prog.i")
    cmd = tool.cmdline("saf", ["--witness", "w.yml"], task, None)
    assert cmd == [
        "saf",
        "verify",
        "--property",
        "p.prp",
        "--data-model",
        "ILP32",
        "--witness",
        "w.yml",
        "prog.i",
    ]


@pytest.mark.parametrize(
    "output,expected",
    [
        (["false(unreach-call)"], "false(unreach-call)"),
        (["true"], "true"),
        (["unknown"], "unknown"),
        # A stderr-style diagnostic before the verdict must be ignored:
        (
            ["saf verify: input=x (slice-0 skeleton)", "false(valid-free)"],
            "false(valid-free)",
        ),
        # No verdict at all (crash/timeout) -> UNKNOWN, never a negative guess:
        (["Segmentation fault"], "unknown"),
        ([], "unknown"),
    ],
)
def test_determine_result(output, expected):
    assert _load_tool().determine_result(_FakeRun(output)) == expected
    # Cross-check the FALSE strings really are the benchexec constants.
    if expected.startswith("false("):
        assert expected in {
            result.RESULT_FALSE_REACH,
            result.RESULT_FALSE_FREE,
            result.RESULT_FALSE_DEREF,
            result.RESULT_FALSE_MEMTRACK,
            result.RESULT_FALSE_MEMCLEANUP,
            result.RESULT_FALSE_OVERFLOW,
            result.RESULT_FALSE_TERMINATION,
            result.RESULT_FALSE_DATARACE,
        }
