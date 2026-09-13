#!/usr/bin/env python3
"""Movement 5, part 5: fix the substantive defects the adversarial verifiers found.

Each fix here corresponds to a finding that was demonstrated, not argued:

  MAJOR  version() returned 'saf 0.1.0 (LLVM 18.1)' verbatim. fm-tools
         ci/check_archive.py errors when the version starts with the name the
         module reports; SAF passed only because name() is "SAF" and clap prints
         "saf" -- a case accident one rename away from a rejected archive. The
         module's comment also claimed the opposite of what the binary does.
  MAJOR  test_executable_found_from_archive_root passed VACUOUSLY: the verifier
         mutation-tested it by deleting subdir="bin" and planting a decoy `saf`
         on PATH, and it still passed.
  MAJOR  test_program_files_include_the_share_tree FAILED under that same PATH,
         so the suite was not green in the environment SAF's own harness creates.
  MAJOR  The archive README -- a file SV-COMP mandates -- advertised "GraphML 1.0
         for no-data-race". The 2027 rules support only YAML 2.0/2.1/2.2 and mark
         GraphML legacy, usable by inactive tools only.
  MINOR  'PLACEHOLDER-SAF-COPYRIGHT-HOLDER' passed all four upstream CI jobs, so
         nothing stopped it shipping to a public pull request.
  MINOR  smoketest.sh ran --version but asserted none of the properties that are
         a hard fm-tools failure.
"""
import sys
from pathlib import Path

TOOL = Path("benchexec/tools/saf.py")
TEST = Path("benchexec/tools/test_saf.py")
ARCHIVE = Path("scripts/make_svcomp_archive.sh")
SMOKE = Path("smoketest/smoketest.sh")

EDITS = [
    (TOOL, '''    def version(self, executable):
        # SAF prints its version as a single line that starts with the version
        # number. Guard against further lines anyway, because a version string
        # containing a line break is rejected by the tools that build the
        # result tables.
        version = self._version_from_tool(executable)
        return version.splitlines()[0].strip() if version else ""''',
     '''    def version(self, executable):
        # `saf --version` prints a single line, and clap prefixes it with the
        # binary name: "saf 0.1.0 (LLVM 18.1)". fm-tools' ci/check_archive.py
        # REJECTS an archive whose version string starts with the name this
        # module reports, so strip that leading token. The comparison is
        # deliberately case-insensitive: name() is "SAF" and clap prints "saf",
        # so today the check passes only by accident, and renaming either one
        # would silently fail the archive. Extra lines are dropped because a
        # version containing a newline is rejected by the same script.
        version = self._version_from_tool(executable)
        version = version.splitlines()[0].strip() if version else ""
        prefix = self.name().lower() + " "
        if version.lower().startswith(prefix):
            version = version[len(prefix):].strip()
        return version'''),

    (TEST, '''        ("0.1.0+svcomp27 (llvm18)\\n", "0.1.0+svcomp27 (llvm18)"),
        ("0.1.0+svcomp27 (llvm18)", "0.1.0+svcomp27 (llvm18)"),
        ("   0.1.0+svcomp27 (llvm18)   \\n", "0.1.0+svcomp27 (llvm18)"),
        ("0.1.0+svcomp27 (llvm18)\\n\\n\\n", "0.1.0+svcomp27 (llvm18)"),
        # A second line would make fm-tools ci/check_archive.py fail the archive.
        ("0.1.0+svcomp27 (llvm18)\\nbuilt from abcdef\\n", "0.1.0+svcomp27 (llvm18)"),
        ("", ""),''',
     '''        ("0.1.0+svcomp27 (llvm18)\\n", "0.1.0+svcomp27 (llvm18)"),
        ("0.1.0+svcomp27 (llvm18)", "0.1.0+svcomp27 (llvm18)"),
        ("   0.1.0+svcomp27 (llvm18)   \\n", "0.1.0+svcomp27 (llvm18)"),
        ("0.1.0+svcomp27 (llvm18)\\n\\n\\n", "0.1.0+svcomp27 (llvm18)"),
        # A second line would make fm-tools ci/check_archive.py fail the archive.
        ("0.1.0+svcomp27 (llvm18)\\nbuilt from abcdef\\n", "0.1.0+svcomp27 (llvm18)"),
        # THE STRING THE BINARY ACTUALLY PRINTS TODAY. clap renders
        # "{display_name} {version}", so the tool name leads. check_archive.py
        # errors on a version starting with the reported name; it slips through
        # only because that comparison is case-sensitive. version() strips it.
        ("saf 0.1.0 (LLVM 18.1)\\n", "0.1.0 (LLVM 18.1)"),
        ("SAF 0.1.0 (LLVM 18.1)\\n", "0.1.0 (LLVM 18.1)"),
        ("", ""),'''),

    (TEST, '''    version = tool.version(str(exe))
    assert version == expected
    assert "\\n" not in version
    assert len(version) <= 100
    # check_archive.py rejects a version that starts with the reported name.
    assert not version.startswith(tool.name())''',
     '''    version = tool.version(str(exe))
    assert version == expected
    assert "\\n" not in version
    assert len(version) <= 100
    # check_archive.py rejects a version that starts with the reported name. Its
    # own comparison is case-sensitive; assert the stronger case-insensitive
    # property so this cannot pass by the accident of "SAF" vs "saf".
    assert not version.lower().startswith(tool.name().lower())'''),

    (TEST, '''def test_executable_found_from_archive_root(tool, tmp_path, monkeypatch):
    # How fm-tools ci/check_archive.py runs it: chdir into the unpacked archive
    # root, then ToolLocator(use_path=True, use_current=True). That only finds
    # bin/saf because executable() passes subdir="bin".
    _fake_archive(tmp_path / "saf")
    monkeypatch.chdir(tmp_path / "saf")
    locator = CURRENT_BASETOOL.ToolLocator(use_path=True, use_current=True)
    executable = tool.executable(locator)
    assert os.path.isfile(executable)
    assert os.access(executable, os.X_OK)''',
     '''def test_executable_found_from_archive_root(tool, tmp_path, monkeypatch):
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
    )'''),

    (TEST, '''def test_program_files_include_the_share_tree(tool, tmp_path, monkeypatch):
    # The default program_files() would resolve REQUIRED_PATHS relative to bin/
    # and find nothing; the override uses parent_dir=True.
    _fake_archive(tmp_path / "saf")
    monkeypatch.chdir(tmp_path / "saf")''',
     '''def test_program_files_include_the_share_tree(tool, tmp_path, monkeypatch):
    # The default program_files() would resolve REQUIRED_PATHS relative to bin/
    # and find nothing; the override uses parent_dir=True.
    #
    # PATH is emptied for the same reason as above -- but here an ambient `saf`
    # made this test FAIL rather than pass vacuously, because the locator picked
    # the PATH binary and REQUIRED_PATHS resolved beside that instead.
    _fake_archive(tmp_path / "saf")
    monkeypatch.setenv("PATH", "")
    monkeypatch.chdir(tmp_path / "saf")'''),

    (ARCHIVE, '''A violation witness (YAML 2.0, or GraphML 1.0 for \\`no-data-race\\`) is written''',
     '''A violation witness (YAML, per the format version the task's base category
requires) is written'''),
]

SMOKE_OLD = '''[ -n "$version" ] || fail "'$SAF --version' printed nothing"
case "$version" in
*"
"*)
    fail "'$SAF --version' printed more than one line: $version"
    ;;
esac
note "version: $version"'''

SMOKE_NEW = '''[ -n "$version" ] || fail "'$SAF --version' printed nothing"
case "$version" in
*"
"*)
    fail "'$SAF --version' printed more than one line: $version"
    ;;
esac
# fm-tools ci/check_archive.py rejects the archive when the version string is
# empty, spans lines, exceeds 100 characters, or starts with the name the
# BenchExec tool-info module reports. That last one currently passes only
# because the comparison is case-sensitive ("SAF" vs "saf"), so pin the
# stronger property here -- this is the cheapest place to catch a regression.
if [ "${#version}" -gt 100 ]; then
    fail "'$SAF --version' printed ${#version} characters; check_archive.py caps it at 100"
fi
case "$version" in
[0-9]*) ;;
*) fail "'$SAF --version' must start with a digit, got: $version" ;;
esac
note "version: $version"'''

PLACEHOLDER_TEST = '''

def test_no_placeholder_text_ships_to_upstream():
    """A placeholder in the SPDX header passes every upstream CI job.

    reuse, ruff, ruff-format and codespell were all run against this file with
    `PLACEHOLDER-SAF-COPYRIGHT-HOLDER` in place and all four returned 0, so
    nothing upstream would stop it reaching a public pull request. This is the
    only thing that will.
    """
    source = MODULE_PATH.read_text(encoding="utf-8")
    assert "PLACEHOLDER" not in source, (
        "benchexec/tools/saf.py still contains a PLACEHOLDER; replace the "
        "SPDX-FileCopyrightText holder before opening the BenchExec MR"
    )
'''


def main():
    tool_src = TOOL.read_text()
    if "case-insensitive: name() is" in tool_src:
        sys.exit("already applied -- refusing to run twice")

    for path, old, new in EDITS:
        text = path.read_text()
        if text.count(old) != 1:
            sys.exit(f"pattern appears {text.count(old)}x (want 1) in {path}:\n"
                     f"---\n{old[:240]}\n---")
        path.write_text(text.replace(old, new))
        print(f"  {path}: rewrote {old.splitlines()[0].strip()[:64]}")

    s = SMOKE.read_text()
    if s.count(SMOKE_OLD) != 1:
        sys.exit(f"smoketest version block not unique ({s.count(SMOKE_OLD)} hits)")
    SMOKE.write_text(s.replace(SMOKE_OLD, SMOKE_NEW))
    print(f"  {SMOKE}: --version now asserts digit-first and <=100 chars")

    t = TEST.read_text()
    if "MODULE_PATH" not in t:
        sys.exit("test_saf.py has no MODULE_PATH constant; cannot add the placeholder guard")
    TEST.write_text(t.rstrip() + "\n" + PLACEHOLDER_TEST)
    print(f"  {TEST}: added the PLACEHOLDER ship-guard test")


if __name__ == "__main__":
    main()
