"""Tests for the SAF BenchExec tool-info module in ``benchexec/tools/saf.py``.

NOT PART OF THE UPSTREAM SUBMISSION. Only ``saf.py`` is proposed for
``github.com/sosy-lab/benchexec``; this file stays in SAF's own tree. Every
reason below was checked against benchexec ``main`` on 2026-09-13:

* ``benchexec/tools/`` holds 196 ``.py`` files and not one of them is a unit
  test for a single tool-info module. The three whose names look like tests are
  ``test.py`` (a generic test that loads *every* module in the directory) plus
  ``testcoca.py`` and ``testcov.py``, which are the tool-info modules of the
  tools TestCoca and TestCov.
* ``tools/test.py`` iterates ``os.listdir()`` and calls
  ``benchexec.model.load_tool_info(<basename>, ...)`` for every ``.py`` file
  except itself, so an upstream ``tools/test_saf.py`` would be loaded as if it
  were a tool-info module. Measured against benchexec 3.35, that raises
  ``SystemExit: Unsupported tool "test_saf" specified, class "Tool" is missing``
  which ``tools/test.py`` swallows into a ``logging.warning`` on every CI run.
* ``pyproject.toml`` has ``[tool.coverage.run] omit = ['benchexec/tools/*']``
  and ``[tool.pytype] exclude`` contains ``'benchexec/tools'``: upstream
  deliberately does not measure or type-check tool-info modules.
* ``[tool.pytest.ini_options] python_files`` includes ``test_*.py``, so this
  file would additionally be collected by upstream's own ``pytest`` run and
  make their CI depend on SAF's test expectations.

The module is loaded from its path, exactly as it sits in this repository, and
is exercised through the real ``BaseTool2.Task`` / ``BaseTool2.Run`` /
``BaseTool2.RunOutput`` / ``util.ProcessExitCode`` constructors rather than
hand-rolled stand-ins, so a change to the BenchExec API shows up here.

Requires ``benchexec`` to be importable; the whole module is skipped otherwise,
so the main ``pytest python/tests`` run stays independent of a benchexec
install. Note that SAF's ``make test`` target only runs ``python/tests/``, so
this file has to be invoked explicitly:

    pip install benchexec && pytest benchexec/tools/test_saf.py

SPDX-License-Identifier: Apache-2.0
"""

import importlib.util
import os
import stat

import pytest

pytest.importorskip("benchexec")

import benchexec.result as result  # noqa: E402
import benchexec.util as beutil  # noqa: E402
from benchexec.tooladapter import CURRENT_BASETOOL  # noqa: E402
from benchexec.tools.template import UnsupportedFeatureException  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
MODULE_PATH = os.path.join(HERE, "saf.py")

PROP = "properties/unreach-call.prp"
INPUT = "loops/array-1.i"

# Every FALSE verdict the module has to pass through unchanged, mapped to the
# benchexec constant it has to be equal to. This is deliberately a SUPERSET of
# what `saf verify` prints today (as of 2026-09-13 it emits only
# false(unreach-call), false(valid-deref), false(valid-free),
# false(no-overflow) and false(no-data-race)): determine_result must not have to
# change when a new SAF strategy starts reporting one of the others.
SAF_FALSE_VERDICTS = {
    "false(unreach-call)": result.RESULT_FALSE_REACH,
    "false(valid-deref)": result.RESULT_FALSE_DEREF,
    "false(valid-free)": result.RESULT_FALSE_FREE,
    "false(valid-memtrack)": result.RESULT_FALSE_MEMTRACK,
    "false(no-overflow)": result.RESULT_FALSE_OVERFLOW,
    "false(termination)": result.RESULT_FALSE_TERMINATION,
    "false(no-data-race)": result.RESULT_FALSE_DATARACE,
}


@pytest.fixture(scope="module")
def tool():
    """Load benchexec/tools/saf.py from this repository, not from site-packages."""
    spec = importlib.util.spec_from_file_location("saf_toolinfo", MODULE_PATH)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    instance = module.Tool()
    # The same adapter benchexec.model.load_tool_info() applies; it asserts that
    # the class really implements the current BaseTool2 API.
    from benchexec import tooladapter

    return tooladapter.adapt_to_current_version(instance)


def _task(data_model="ILP32", property_file=PROP, input_files=(INPUT,)):
    options = {"language": "C"}
    if data_model:
        options["data_model"] = data_model
    return CURRENT_BASETOOL.Task.with_files(
        list(input_files), property_file=property_file, options=options
    )


def _run(lines, *, value=0, signal=None, termination_reason=None):
    exit_code = beutil.ProcessExitCode.create(value=value, signal=signal)
    output = CURRENT_BASETOOL.RunOutput([line + "\n" for line in lines])
    return CURRENT_BASETOOL.Run(["saf"], exit_code, output, termination_reason)


# --- Tests that guard the upstream CI jobs (reuse, ruff check, ruff format) ---


def test_source_carries_the_upstream_spdx_header():
    # The `reuse` CI job (fsfe/reuse-action) fails on any file without one, and
    # upstream's REUSE.toml has no annotation covering benchexec/tools/*.py.
    with open(MODULE_PATH, encoding="utf-8") as f:
        head = [line.rstrip("\n") for line in f.readlines()[:7]]
    assert head[0] == (
        "# This file is part of BenchExec, a framework for reliable benchmarking:"
    )
    assert head[1] == "# https://github.com/sosy-lab/benchexec"
    assert head[2] == "#"
    assert any(line.startswith("# SPDX-FileCopyrightText: ") for line in head)
    assert "# SPDX-License-Identifier: Apache-2.0" in head


def test_source_lines_fit_the_ruff_format_width():
    # Upstream runs `ruff format --check`, whose default width is 88 columns.
    with open(MODULE_PATH, encoding="utf-8") as f:
        too_long = [
            (n, len(line.rstrip("\n")))
            for n, line in enumerate(f, start=1)
            if len(line.rstrip("\n")) > 88
        ]
    assert not too_long, f"lines over 88 columns: {too_long}"


def test_source_is_pure_ascii():
    # Upstream selects ALL ruff rules; RUF001/RUF002/RUF003 flag ambiguous
    # unicode (an em dash or a typographic quote, for example) in strings,
    # docstrings and comments. Staying ASCII sidesteps the whole family.
    with open(MODULE_PATH, "rb") as f:
        raw = f.read()
    raw.decode("ascii")  # raises UnicodeDecodeError if not


# --- Tests for the static metadata methods ---


def test_name(tool):
    assert tool.name() == "SAF"


def test_project_url(tool):
    assert tool.project_url().startswith("https://")


def test_required_paths_cover_the_archive(tool):
    # The archive is <root>/bin/saf + <root>/share/saf/{stubs,specs}. SAF
    # resolves its stub header relative to its own executable, so share/ has to
    # travel with bin/.
    assert set(tool.REQUIRED_PATHS) == {"bin", "share"}


def test_docstring_is_present(tool):
    # `python -m benchexec.test_tool_info saf` prints this first.
    doc = (type(tool).__doc__ or "").strip()
    assert doc
    # A reviewer has to be able to see the three things that are not obvious
    # from the code: which properties are covered, that a property file is
    # mandatory, and where the data files live.
    for expected in ("unreach-call", "property file", "share/saf"):
        assert expected in doc


# --- Tests for executable and program_files ---


def _fake_archive(root):
    (root / "bin").mkdir(parents=True)
    (root / "share" / "saf" / "stubs").mkdir(parents=True)
    (root / "share" / "saf" / "specs").mkdir(parents=True)
    exe = root / "bin" / "saf"
    exe.write_text("#!/bin/sh\nexit 0\n")
    exe.chmod(exe.stat().st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)
    (root / "share" / "saf" / "stubs" / "sv-comp-stubs.h").write_text("/* stub */\n")
    (root / "share" / "saf" / "specs" / "libc.json").write_text("{}\n")
    return exe


def test_executable_found_with_tool_directory(tool, tmp_path):
    # How SV-COMP runs it: benchexec ... --tool-directory .
    _fake_archive(tmp_path / "saf")
    locator = CURRENT_BASETOOL.ToolLocator(tool_directory=str(tmp_path / "saf"))
    assert tool.executable(locator).endswith(os.path.join("bin", "saf"))


def test_executable_found_from_archive_root(tool, tmp_path, monkeypatch):
    # How fm-tools ci/check_archive.py runs it: chdir into the unpacked archive
    # root, then ToolLocator(use_path=True, use_current=True). That only finds
    # bin/saf because executable() passes subdir="bin".
    #
    # PATH is emptied on purpose. With any `saf` on PATH -- which is exactly what
    # scripts/run_benchexec_svcomp.sh arranges -- use_path=True resolves first and
    # this test passes even if subdir="bin" is deleted, i.e. it stops testing the
    # one behaviour it exists to protect. Asserting the resolved location closes
    # the same hole from the other side.
    _fake_archive(tmp_path / "saf")
    monkeypatch.setenv("PATH", "")
    monkeypatch.chdir(tmp_path / "saf")
    locator = CURRENT_BASETOOL.ToolLocator(use_path=True, use_current=True)
    executable = tool.executable(locator)
    assert os.path.isfile(executable)
    assert os.access(executable, os.X_OK)
    assert os.path.realpath(executable) == os.path.realpath(
        tmp_path / "saf" / "bin" / "saf"
    )


def test_program_files_include_the_share_tree(tool, tmp_path, monkeypatch):
    # The default program_files() would resolve REQUIRED_PATHS relative to bin/
    # and find nothing; the override uses parent_dir=True.
    #
    # PATH is emptied for the same reason as above -- but here an ambient `saf`
    # made this test FAIL rather than pass vacuously, because the locator picked
    # the PATH binary and REQUIRED_PATHS resolved beside that instead.
    _fake_archive(tmp_path / "saf")
    monkeypatch.setenv("PATH", "")
    monkeypatch.chdir(tmp_path / "saf")
    locator = CURRENT_BASETOOL.ToolLocator(use_path=True, use_current=True)
    executable = tool.executable(locator)
    files = beutil.get_files(tool.program_files(executable))
    assert any(f.endswith(os.path.join("bin", "saf")) for f in files), files
    assert any(f.endswith("sv-comp-stubs.h") for f in files), files
    assert any(f.endswith("libc.json") for f in files), files


# --- Tests for the version string ---


@pytest.mark.parametrize(
    ("printed", "expected"),
    [
        ("0.1.0+svcomp27 (llvm18)\n", "0.1.0+svcomp27 (llvm18)"),
        ("0.1.0+svcomp27 (llvm18)", "0.1.0+svcomp27 (llvm18)"),
        ("   0.1.0+svcomp27 (llvm18)   \n", "0.1.0+svcomp27 (llvm18)"),
        ("0.1.0+svcomp27 (llvm18)\n\n\n", "0.1.0+svcomp27 (llvm18)"),
        # A second line would make fm-tools ci/check_archive.py fail the archive.
        ("0.1.0+svcomp27 (llvm18)\nbuilt from abcdef\n", "0.1.0+svcomp27 (llvm18)"),
        # THE STRING THE BINARY ACTUALLY PRINTS TODAY. clap renders
        # "{display_name} {version}", so the tool name leads. check_archive.py
        # errors on a version starting with the reported name; it slips through
        # only because that comparison is case-sensitive. version() strips it.
        ("saf 0.1.0 (LLVM 18.1)\n", "0.1.0 (LLVM 18.1)"),
        ("SAF 0.1.0 (LLVM 18.1)\n", "0.1.0 (LLVM 18.1)"),
        ("", ""),
    ],
)
def test_version_is_a_single_line(tool, tmp_path, printed, expected):
    payload = tmp_path / "payload.txt"
    payload.write_text(printed)
    exe = tmp_path / "saf"
    exe.write_text(f'#!/bin/sh\ncat "{payload}"\n')
    exe.chmod(exe.stat().st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)
    version = tool.version(str(exe))
    assert version == expected
    assert "\n" not in version
    assert len(version) <= 100
    # check_archive.py rejects a version that starts with the reported name. Its
    # own comparison is case-sensitive; assert the stronger case-insensitive
    # property so this cannot pass by the accident of "SAF" vs "saf".
    assert not version.lower().startswith(tool.name().lower())


# --- Tests for the command line ---


def test_cmdline_builds_expected_argv(tool):
    cmd = tool.cmdline("./bin/saf", ["--witness", "witness.yml"], _task(), None)
    assert cmd == [
        "./bin/saf",
        "verify",
        "--property",
        PROP,
        "--data-model",
        "ILP32",
        "--witness",
        "witness.yml",
        INPUT,
    ]


def test_cmdline_lp64(tool):
    cmd = tool.cmdline("saf", [], _task(data_model="LP64"), None)
    assert cmd == ["saf", "verify", "--property", PROP, "--data-model", "LP64", INPUT]


def test_cmdline_without_data_model(tool):
    cmd = tool.cmdline("saf", [], _task(data_model=None), None)
    assert cmd == ["saf", "verify", "--property", PROP, INPUT]


def test_cmdline_does_not_duplicate_an_explicit_data_model(tool):
    cmd = tool.cmdline("saf", ["--data-model", "LP64"], _task(), None)
    assert cmd.count("--data-model") == 1
    assert "ILP32" not in cmd


def test_cmdline_does_not_mutate_the_options_list(tool):
    options = ["--witness", "witness.yml"]
    tool.cmdline("saf", options, _task(), None)
    assert options == ["--witness", "witness.yml"]


def test_cmdline_never_emits_an_unexpanded_witness_placeholder(tool):
    # BenchExec does not substitute ${witness} in option values, it only warns
    # and passes the literal through. The module must therefore never introduce
    # the placeholder itself; SAF's own default of witness.yml covers the rule.
    cmd = tool.cmdline("saf", [], _task(), None)
    assert not any("${witness}" in arg for arg in cmd)
    assert "--witness" not in cmd


def test_cmdline_without_property_file_is_rejected(tool):
    with pytest.raises(UnsupportedFeatureException):
        tool.cmdline("saf", [], _task(property_file=None), None)


def test_cmdline_with_unknown_data_model_is_rejected(tool):
    with pytest.raises(UnsupportedFeatureException):
        tool.cmdline("saf", [], _task(data_model="ILP128"), None)


def test_cmdline_with_several_input_files_is_rejected(tool):
    with pytest.raises(UnsupportedFeatureException):
        tool.cmdline("saf", [], _task(input_files=("a.i", "b.i")), None)


def test_cmdline_without_input_files_is_rejected(tool):
    task = CURRENT_BASETOOL.Task.without_files("ident", property_file=PROP)
    with pytest.raises(UnsupportedFeatureException):
        tool.cmdline("saf", [], task, None)


# --- Tests for the verdict parser ---


@pytest.mark.parametrize("verdict", sorted(SAF_FALSE_VERDICTS))
def test_determine_result_false_verdicts(tool, verdict):
    assert tool.determine_result(_run([verdict])) == SAF_FALSE_VERDICTS[verdict]
    assert result.get_result_classification(verdict) == result.RESULT_CLASS_FALSE


def test_determine_result_true(tool):
    assert tool.determine_result(_run(["true"])) == result.RESULT_TRUE_PROP


def test_determine_result_unknown(tool):
    assert tool.determine_result(_run(["unknown"])) == result.RESULT_UNKNOWN


def test_determine_result_bare_false(tool):
    assert tool.determine_result(_run(["false"])) == result.RESULT_FALSE_PROP


def test_determine_result_ignores_diagnostics(tool):
    # stdout and stderr end up in the same log file, so the verdict can be
    # preceded by SAF's stderr chatter.
    run = _run(
        [
            "saf verify: input=array-1.i property=unreach-call",
            "saf verify: fuzzing 4 seeds",
            "false(valid-free)",
        ]
    )
    assert tool.determine_result(run) == result.RESULT_FALSE_FREE


def test_determine_result_no_output(tool):
    assert tool.determine_result(_run([])) == result.RESULT_UNKNOWN


def test_determine_result_no_verdict_line(tool):
    assert tool.determine_result(_run(["saf verify: giving up"])) == (
        result.RESULT_UNKNOWN
    )


@pytest.mark.parametrize("signal", [6, 9, 11, 15])
def test_determine_result_discards_a_verdict_printed_before_a_signal(tool, signal):
    # A verdict flushed just before the process died must not be reported. It
    # would be SCORED: benchexec.model._analyze_result only replaces results in
    # result.RESULT_LIST_OTHER by ABORTED / SEGMENTATION FAULT / KILLED, so a
    # false(...) survives the signal and counts. Returning unknown lets that
    # substitution happen.
    run = _run(["false(unreach-call)"], value=None, signal=signal)
    assert tool.determine_result(run) == result.RESULT_UNKNOWN


def test_determine_result_discards_a_true_printed_before_a_signal(tool):
    run = _run(["true"], value=None, signal=9)
    assert tool.determine_result(run) == result.RESULT_UNKNOWN


def test_determine_result_keeps_the_verdict_on_a_nonzero_exit_value(tool):
    # Exiting non-zero is not the same as crashing; BenchExec appends the code.
    run = _run(["false(no-overflow)"], value=3)
    assert tool.determine_result(run) == result.RESULT_FALSE_OVERFLOW


def test_no_placeholder_text_ships_to_upstream():
    """A placeholder in the SPDX header passes every upstream CI job.

    reuse, ruff, ruff-format and codespell were all run against this file with
    `PLACEHOLDER-SAF-COPYRIGHT-HOLDER` in place and all four returned 0, so
    nothing upstream would stop it reaching a public pull request. This is the
    only thing that will.
    """
    with open(MODULE_PATH, encoding="utf-8") as f:
        source = f.read()
    assert "PLACEHOLDER" not in source, (
        "benchexec/tools/saf.py still contains a PLACEHOLDER; replace the "
        "SPDX-FileCopyrightText holder before opening the BenchExec MR"
    )
