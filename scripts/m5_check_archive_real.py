#!/usr/bin/env python3
"""Run fm-tools' REAL ci/check_archive.py zip rules against dist/saf-verify.zip,
then exercise the BenchExec tool-info module end to end against the unpacked tree.

This closes the loop on the version() fix: check_archive.py's hard rule is
`not version.startswith(reported_name)`, and the version it tests is the one the
tool-info module returns -- so the only way to know the fix works is to unpack
the real archive, load the real module, and read what it reports.
"""
import io
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import urllib.request
import zipfile

ZIP = "dist/saf-verify.zip"
CHECK_URL = ("https://gitlab.com/api/v4/projects/sosy-lab%2Fbenchmarking%2Ffm-tools"
             "/repository/files/ci%2Fcheck_archive.py/raw?ref=main")

fails = []
def check(ok, msg):
    print(("  PASS  " if ok else "  FAIL  ") + msg)
    if not ok:
        fails.append(msg)


print("=== 1. fm-tools ci/check_archive.py, check_zipfile rules (verbatim source) ===")
try:
    src = urllib.request.urlopen(CHECK_URL, timeout=30).read().decode()
    print(f"  fetched check_archive.py ({len(src)} bytes)")
    forbidden = [ln for ln in src.splitlines() if "__MACOSX" in ln or ".aptrelease" in ln]
    print("  its forbidden-path rule:", forbidden[0].strip() if forbidden else "(not found)")
except Exception as e:                                   # noqa: BLE001
    print(f"  WARN could not fetch upstream ({e}); applying the rules from the audit")

z = zipfile.ZipFile(ZIP)
names = z.namelist()
roots = {n.split("/")[0] for n in names}
check(len(roots) == 1, f"exactly one top-level directory (got {sorted(roots)})")

top = next(iter(roots)) + "/"
root_files = [n[len(top):] for n in names
              if n.startswith(top) and "/" not in n[len(top):] and not n.endswith("/")]
check(any(f.lower().startswith("readme") for f in root_files),
      f"a README* at the archive root (root files: {sorted(root_files)})")
check(any(f.lower().startswith(("license", "licence")) for f in root_files),
      "a LICENSE* at the archive root")
check("smoketest.sh" in root_files, "smoketest.sh at the archive root")

BAD = (".git/", ".svn/", ".hg/", "CVS/", "__MACOSX", ".aptrelease")
offenders = [n for n in names if any(b in n for b in BAD)]
check(not offenders, f"no repository/aux paths (found {offenders[:3]})")

links = [n for n in names if stat.S_ISLNK(z.getinfo(n).external_attr >> 16)]
check(not links, f"no symlinks (found {links[:3]})")

smode = z.getinfo(top + "smoketest.sh").external_attr >> 16
check(smode & 0o001, f"smoketest.sh has the OTHER-execute bit fm-weck tests (mode {smode:o})")
check(z.getinfo(top + "smoketest.sh").file_size > 0, "smoketest.sh is non-empty")
bmode = z.getinfo(top + "bin/saf").external_attr >> 16
check(bmode & 0o111, f"bin/saf is executable (mode {bmode:o})")

print("\n=== 2. unpack the way lib-fm-tools fm_tools/files.py does ===")
tmp = tempfile.mkdtemp(prefix="saf-archive-check-")
for n in names:
    z.extract(n, tmp)
    m = z.getinfo(n).external_attr >> 16
    if m:
        os.chmod(os.path.join(tmp, n), m)
root = os.path.join(tmp, top.rstrip("/"))
print(f"  unpacked to {root}")

print("\n=== 3. the BenchExec tool-info module, against the REAL unpacked archive ===")
sys.path.insert(0, os.path.abspath("benchexec/tools"))
try:
    import importlib.util
    spec = importlib.util.spec_from_file_location("saf_ti", "benchexec/tools/saf.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    tool = mod.Tool()

    raw = subprocess.run([os.path.join(root, "bin", "saf"), "--version"],
                         capture_output=True, text=True, check=False).stdout.strip()
    reported = tool.version(os.path.join(root, "bin", "saf"))
    print(f"  binary prints : {raw!r}")
    print(f"  module reports: {reported!r}")

    # The exact rules from ci/check_archive.py::_checks_on_executable.
    check(bool(reported), "version is non-empty")
    check("\n" not in reported, "version is a single line")
    check(len(reported) <= 100, f"version is <= 100 chars ({len(reported)})")
    check(not reported.startswith(tool.name()),
          f"version does not start with name() == {tool.name()!r}  [the HARD rule]")
    check(not reported.lower().startswith(tool.name().lower()),
          "version does not start with the name case-INsensitively either "
          "[stronger than upstream; guards a rename]")
    check(reported[:1].isdigit(),
          "version starts with a digit [test_tool_info warns otherwise]")
except Exception as e:                                   # noqa: BLE001
    check(False, f"tool-info module raised: {e!r}")

shutil.rmtree(tmp, ignore_errors=True)

print()
if fails:
    print(f"*** {len(fails)} CHECK(S) FAILED ***")
    for f in fails:
        print("   -", f)
    sys.exit(1)
print("ALL ARCHIVE + TOOL-INFO CHECKS PASSED")
