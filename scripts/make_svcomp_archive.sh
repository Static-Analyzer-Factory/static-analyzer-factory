#!/usr/bin/env bash
#
# Build the reproducible SAF tool archive for SV-COMP.
#
# Output: dist/saf-verify.zip -- the name fm-tools' ci/check_archive.py derives
# for tool id "saf" on track "Verification"
# (ci/_ciutil.py:get_track_for_filename lowercases the track and rewrites
# "verification" -> "verify", and check_zipfile opens
# "<archives-root>/<tool>-<track>.zip").
#
# Archive layout -- NOT negotiable:
#
#   saf/
#     LICENSE            MIT, copied verbatim from the repo root
#     README.md          generated here so the version can never drift
#     smoketest.sh       mode 0755, from smoketest/smoketest.sh
#     bin/saf            mode 0755, the release binary
#     share/saf/...      stubs/ + specs/, the whole tree
#     smoketest/         the fixtures smoketest.sh runs
#
# The share/ tree must sit *beside* bin/ because SAF resolves its SV-COMP stub
# header with `resolve_svcomp_stub()` (crates/saf-cli/src/commands.rs), which
# walks `current_exe().ancestors()` joined with
# "share/saf/stubs/sv-comp-stubs.h". A binary shipped without that tree answers
# `unknown` on every task, silently. smoketest.sh check 3 is what catches it.
#
# SV-COMP's competition-scripts mkInstall.sh unzips the archive and then
# flattens the single top-level directory away, so nothing may depend on the
# top directory's *name* -- only on the paths below it.
#
# Everything is assembled from an explicit allow-list. The working tree of this
# repo contains stray junk (files literally named '((((((((((' and
# '!!!!!!!!!!', *.jsonl dumps, .p*spike/ scratch dirs, a root-owned unreadable
# directory), so any glob-the-repo approach produces a failing archive.
#
# Usage:
#   scripts/make_svcomp_archive.sh          # inside the dev container
#   make svcomp-archive                     # the same, from the host
#
# Environment overrides:
#   SAF_ARCHIVE_BIN          path to the release binary   (default target/release/saf)
#   SAF_ARCHIVE_OUT          output directory             (default dist/)
#   SAF_ARCHIVE_TOP          top-level directory name     (default saf)
#   SAF_ARCHIVE_ZIP_NAME     output file name             (default saf-verify.zip)
#   SAF_ARCHIVE_MAX_GLIBC    highest tolerated GLIBC_     (default 2.39)
#   SAF_ARCHIVE_TIMESTAMP    mtime stamped on every entry (default 2020-01-01T00:00:00Z)
#   SAF_ARCHIVE_SKIP_SMOKETEST=1  skip running the smoke test on the unpacked
#                            archive (only for hosts without clang-18; the
#                            archive is then NOT fully verified)
#
# SPDX-License-Identifier: MIT

set -euo pipefail

# Deterministic zip entries: both `zip` and Python's zipfile convert mtimes to
# local time before storing the DOS timestamp.
export TZ=UTC
export LC_ALL=C

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SAF_BIN="${SAF_ARCHIVE_BIN:-$REPO_ROOT/target/release/saf}"
OUT_DIR="${SAF_ARCHIVE_OUT:-$REPO_ROOT/dist}"
TOP_DIR="${SAF_ARCHIVE_TOP:-saf}"
ZIP_NAME="${SAF_ARCHIVE_ZIP_NAME:-saf-verify.zip}"
MAX_GLIBC="${SAF_ARCHIVE_MAX_GLIBC:-2.39}"
STAMP="${SAF_ARCHIVE_TIMESTAMP:-2020-01-01T00:00:00Z}"
OUT_ZIP="$OUT_DIR/$ZIP_NAME"

PROJECT_URL="https://github.com/Static-Analyzer-Factory/static-analyzer-factory"

# The complete allow-list of smoketest/ payload (smoketest.sh is handled
# separately -- it is installed at the archive ROOT, not in smoketest/).
SMOKETEST_FILES=(
    smoke_false_unreach.c
    smoke_unknown_unreach.c
    unreach-call.prp
    unreach-call.prp.license
)

# The allow-list of share/ subtrees. Every file underneath is shipped; the
# contents are checked against ALLOWED_SHARE_EXT below.
SHARE_SUBTREES=(
    saf/stubs
    saf/specs
)
ALLOWED_SHARE_EXT='h|yaml|md'

STAGE=""
VERIFY_DIR=""
cleanup() {
    if [ -n "$STAGE" ]; then rm -rf "$STAGE"; fi
    if [ -n "$VERIFY_DIR" ]; then rm -rf "$VERIFY_DIR"; fi
}
trap cleanup EXIT

die() {
    printf '\n*** make_svcomp_archive: FATAL: %s\n\n' "$*" >&2
    exit 1
}

step() { printf '==> %s\n' "$*"; }
info() { printf '    %s\n' "$*"; }

need() {
    command -v "$1" >/dev/null 2>&1 || die "required tool '$1' is not on PATH"
}

# ---------------------------------------------------------------------------
# 0. Preflight: tools
# ---------------------------------------------------------------------------

step "preflight: host tools"
need python3
need objdump
need ldd
need install
need find
need sort
need awk
need touch

case "$TOP_DIR" in
*[!A-Za-z0-9._-]*) die "SAF_ARCHIVE_TOP='$TOP_DIR' contains characters unsafe for an archive path" ;;
"") die "SAF_ARCHIVE_TOP must not be empty" ;;
esac
if command -v zip >/dev/null 2>&1; then
    PACKER=zip
    info "packer: $(command -v zip) (preferred)"
else
    PACKER=python
    info "packer: python3 zipfile (no 'zip' on PATH)"
    info "        mode bits are written explicitly and re-checked in step 5;"
    info "        install the 'zip' package to use the canonical packer."
fi

# ---------------------------------------------------------------------------
# 1. Preflight: the binary
# ---------------------------------------------------------------------------

step "preflight: binary"

[ -e "$SAF_BIN" ] || die "release binary not found at $SAF_BIN
      This script does NOT build SAF. Build it first, e.g.
          docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev cargo build --release -p saf-cli
      or point SAF_ARCHIVE_BIN at an existing binary."
[ -f "$SAF_BIN" ] || die "$SAF_BIN is not a regular file"
[ -x "$SAF_BIN" ] || die "$SAF_BIN is not executable"
[ -s "$SAF_BIN" ] || die "$SAF_BIN is empty"

BIN_BYTES=$(wc -c <"$SAF_BIN")
info "binary: $SAF_BIN ($BIN_BYTES bytes)"

# ELF / architecture. `objdump -f` is available wherever objdump is; `file` is
# not guaranteed to be installed in the dev image.
ARCH_LINE="$(objdump -f "$SAF_BIN" 2>/dev/null | grep '^architecture:' || true)"
case "$ARCH_LINE" in
*i386:x86-64*) info "architecture: x86-64" ;;
"") die "objdump could not read $SAF_BIN -- is it an ELF object at all?" ;;
*) die "unexpected architecture for an SV-COMP submission: $ARCH_LINE (expected i386:x86-64)" ;;
esac

# Static linking of LLVM and Z3 is load-bearing: the competition machines have
# neither. This assertion exists so a build-config regression is loud.
LDD_OUT="$(ldd "$SAF_BIN" 2>&1 || true)"
if printf '%s\n' "$LDD_OUT" | grep -qiE 'libLLVM|libz3'; then
    printf '%s\n' "$LDD_OUT" >&2
    die "the binary links libLLVM and/or libz3 dynamically; both must be linked statically"
fi
info "dynamic deps:"
printf '%s\n' "$LDD_OUT" | sed 's/^/      /'

# The competition runs Ubuntu with glibc 2.39; a symbol from a newer glibc
# makes the binary refuse to start there.
GLIBC_MAX="$(objdump -T "$SAF_BIN" 2>/dev/null | grep -o 'GLIBC_2\.[0-9]*' | sort -uV | tail -1 || true)"
[ -n "$GLIBC_MAX" ] || die "could not read any GLIBC_ symbol version from $SAF_BIN"
GLIBC_HIGHEST="$(printf '%s\nGLIBC_%s\n' "$GLIBC_MAX" "$MAX_GLIBC" | sort -uV | tail -1)"
[ "$GLIBC_HIGHEST" = "GLIBC_$MAX_GLIBC" ] ||
    die "binary requires $GLIBC_MAX but the competition image provides at most GLIBC_$MAX_GLIBC
      Build against an older glibc (the LLVM-18 dev image) or raise SAF_ARCHIVE_MAX_GLIBC
      only after confirming the competition's Ubuntu release."
info "max glibc symbol: $GLIBC_MAX (limit GLIBC_$MAX_GLIBC)"

# Informational only -- libstdc++ is also resolved at runtime.
CXX_MAX="$(objdump -T "$SAF_BIN" 2>/dev/null | grep -oE 'GLIBCXX_[0-9.]+' | sort -uV | tail -1 || true)"
if [ -n "$CXX_MAX" ]; then
    info "max libstdc++ symbol: $CXX_MAX (informational)"
fi

# ---------------------------------------------------------------------------
# 2. Preflight: sources on the allow-list
# ---------------------------------------------------------------------------

step "preflight: allow-listed sources"

[ -f "$REPO_ROOT/LICENSE" ] || die "$REPO_ROOT/LICENSE not found (fm-tools requires a license file at the archive root)"
[ -f "$REPO_ROOT/smoketest/smoketest.sh" ] || die "$REPO_ROOT/smoketest/smoketest.sh not found"
[ -s "$REPO_ROOT/smoketest/smoketest.sh" ] || die "$REPO_ROOT/smoketest/smoketest.sh is empty (fm-weck rejects an empty smoke test)"
[ -f "$REPO_ROOT/share/saf/stubs/sv-comp-stubs.h" ] ||
    die "$REPO_ROOT/share/saf/stubs/sv-comp-stubs.h not found -- without it SAF answers 'unknown' on every task"

for f in "${SMOKETEST_FILES[@]}"; do
    [ -f "$REPO_ROOT/smoketest/$f" ] || die "allow-listed fixture smoketest/$f is missing"
done

# Fail if the fixture directory grew a file nobody added to the allow-list --
# better a loud failure than silently shipping (or silently dropping) it.
ACTUAL_FIXTURES="$(cd "$REPO_ROOT/smoketest" && find . -mindepth 1 -printf '%P\n' | sort)"
EXPECTED_FIXTURES="$(printf '%s\n' smoketest.sh "${SMOKETEST_FILES[@]}" | sort)"
if [ "$ACTUAL_FIXTURES" != "$EXPECTED_FIXTURES" ]; then
    printf 'on disk:\n%s\n\nallow-list:\n%s\n' "$ACTUAL_FIXTURES" "$EXPECTED_FIXTURES" >&2
    die "smoketest/ does not match the allow-list in this script (SMOKETEST_FILES)"
fi
info "smoketest fixtures: ${#SMOKETEST_FILES[@]} + smoketest.sh"

SHARE_FILES=()
for sub in "${SHARE_SUBTREES[@]}"; do
    [ -d "$REPO_ROOT/share/$sub" ] || die "allow-listed share subtree share/$sub is missing"
    while IFS= read -r rel; do
        [ -n "$rel" ] || continue
        case "$rel" in
        *[!A-Za-z0-9._/-]*) die "share/$sub/$rel contains characters unsafe for an archive path" ;;
        esac
        case "$rel" in
        *.*) ;;
        *) die "share/$sub/$rel has no extension; refusing to ship an unclassified file" ;;
        esac
        ext="${rel##*.}"
        if ! printf '%s' "$ext" | grep -qE "^($ALLOWED_SHARE_EXT)$"; then
            die "share/$sub/$rel has extension .$ext, not on the allow-list ($ALLOWED_SHARE_EXT)"
        fi
        SHARE_FILES+=("$sub/$rel")
    done < <(cd "$REPO_ROOT/share/$sub" && find . -mindepth 1 -type f -printf '%P\n' | sort)
done
[ "${#SHARE_FILES[@]}" -gt 0 ] || die "no files found under share/"
info "share/ files: ${#SHARE_FILES[@]}"

# ---------------------------------------------------------------------------
# 3. Stage
# ---------------------------------------------------------------------------

step "stage"

STAGE="$(mktemp -d "${TMPDIR:-/tmp}/saf-archive-stage.XXXXXX")"
ROOT="$STAGE/$TOP_DIR"
mkdir -p "$ROOT/bin" "$ROOT/smoketest"

install -m 0755 "$SAF_BIN" "$ROOT/bin/saf"
install -m 0644 "$REPO_ROOT/LICENSE" "$ROOT/LICENSE"
install -m 0755 "$REPO_ROOT/smoketest/smoketest.sh" "$ROOT/smoketest.sh"

for f in "${SMOKETEST_FILES[@]}"; do
    install -m 0644 "$REPO_ROOT/smoketest/$f" "$ROOT/smoketest/$f"
done

for rel in "${SHARE_FILES[@]}"; do
    install -D -m 0644 "$REPO_ROOT/share/$rel" "$ROOT/share/$rel"
done

SAF_VERSION="$("$ROOT/bin/saf" --version 2>/dev/null || true)"
[ -n "$SAF_VERSION" ] || die "'bin/saf --version' produced no output from the staged tree"
info "version string: $SAF_VERSION"

cat >"$ROOT/README.md" <<EOF
# SAF — Static Analyzer Factory (SV-COMP submission archive)

    $SAF_VERSION

SAF is a Rust static-analysis framework for C programs. It lowers C to LLVM IR,
builds pointer-analysis / value-flow / CFG graphs over an IR-agnostic
representation, and — for SV-COMP — refutes safety properties by solving for a
concrete input and confirming it, so a \`false\` verdict always comes with a
witness it actually produced.

Project and source: <$PROJECT_URL>

## Running it

    ./bin/saf verify --property <property>.prp --data-model LP64|ILP32 <program>.c

Options:

| option | meaning |
| --- | --- |
| \`--property <file.prp>\` | required; the SV-COMP property, parsed from the file's \`CHECK(...)\` text, never from its name |
| \`--data-model LP64\|ILP32\` | selects clang \`-m64\`/\`-m32\` and the LLVM target; default \`LP64\` |
| \`--witness <file>\` | where to write the witness; default \`witness.yml\` |
| \`--timeout <seconds>\` | graceful budget; on expiry SAF prints \`unknown\` and exits 0; default 850 |

Exactly one line goes to stdout — \`true\`, \`false(<subproperty>)\` or
\`unknown\`, the BenchExec \`RESULT_*\` spellings. Everything else is stderr.
The exit status is 0 for all three; a non-zero exit means SAF itself failed.

A violation witness (YAML, per the format version the task's base category
requires) is written
only for a \`false\` verdict; \`unknown\` writes nothing.

## Layout

    bin/saf            the verifier (single binary)
    share/saf/stubs/   the SV-COMP stub header, resolved relative to bin/saf
    share/saf/specs/   library models (libc, POSIX, taint)
    smoketest.sh       self-check; run it from this directory
    smoketest/         the fixtures smoketest.sh uses

\`share/\` must stay beside \`bin/\`: SAF locates its stub header by walking the
ancestors of its own executable path. Moving the binary away from \`share/\`
makes every task answer \`unknown\`.

## Requirements

x86-64 Linux, glibc ≥ 2.39 (the highest \`GLIBC_\` symbol version the binary
references; asserted at archive-build time). LLVM and Z3 are linked statically.
At run time SAF spawns \`clang-18\` and \`opt-18\` (override with \`\$SAF_CLANG\` /
\`\$SAF_OPT\`) to compile the task and its replay harnesses; the sanitizer runtimes
(\`libclang-rt-18-dev\`) are needed for the memory-safety and overflow
confirmers. Remaining shared-library dependencies: libffi.so.8, libm, libz,
libzstd.so.1, libtinfo.so.6, libstdc++, libgcc_s, libc.

SAF bundles **no** SV-COMP participant — no other verifier is shipped or
invoked. clang/opt are compilers and the sanitizers are runtime libraries;
neither competes in SV-COMP.

## License

MIT — see \`LICENSE\`. The \`.prp\` files under \`smoketest/\` come from the
SV-Benchmarks collection and are Apache-2.0; see the accompanying
\`.license\` files.
EOF
chmod 0644 "$ROOT/README.md"

# ---------------------------------------------------------------------------
# 4. Sanitize the staged tree
# ---------------------------------------------------------------------------

step "sanitize"

# fm-tools ci/check_archive.py rejects any member matching
#   .*(/\.git/|/\.svn/|/\.hg/|/CVS/|/__MACOSX|/\.aptrelease).*
FORBIDDEN="$(find "$ROOT" \( -name '.git' -o -name '.svn' -o -name '.hg' -o -name 'CVS' -o -name '__MACOSX' -o -name '.aptrelease' \) -print)"
[ -z "$FORBIDDEN" ] || die "forbidden paths in the staged tree:
$FORBIDDEN"

# Symlinks must resolve inside the archive; shipping real files sidesteps the
# question entirely, so refuse any symlink at all.
SYMLINKS="$(find "$ROOT" -type l -print)"
[ -z "$SYMLINKS" ] || die "symlinks in the staged tree (ship real files instead):
$SYMLINKS"

ODDITIES="$(find "$ROOT" ! -type f ! -type d -print)"
[ -z "$ODDITIES" ] || die "non-regular, non-directory entries in the staged tree:
$ODDITIES"

BADNAMES="$(find "$ROOT" -mindepth 1 -printf "$TOP_DIR/%P\n" | grep -vE '^[A-Za-z0-9._/-]+$' || true)"
[ -z "$BADNAMES" ] || die "archive paths with characters unsafe for a zip:
$BADNAMES"

# Reproducibility: one fixed mtime on every entry, so two runs from the same
# binary produce byte-identical archives.
find "$ROOT" -exec touch -h -d "$STAMP" {} +

STAGED_FILES=$(find "$ROOT" -type f | wc -l)
info "staged: $STAGED_FILES files under $TOP_DIR/"

# ---------------------------------------------------------------------------
# 5. Pack, then verify the zip the way the competition tooling reads it
# ---------------------------------------------------------------------------

KIT="$STAGE/zipkit.py"
cat >"$KIT" <<'PYKIT'
#!/usr/bin/env python3
"""Pack and verify the SAF SV-COMP archive.

`pack` writes a zip whose Unix mode bits survive, because fm-tools re-applies
them on extraction:

    fm_tools/files.py (unzip):
        extracted_file = zipfile.extract(member, target_dir.parent)
        attr = member.external_attr >> 16
        if attr != 0:
            os.chmod(extracted_file, attr)

`verify` re-implements, against the finished zip, every structural rule the
competition tooling applies:

  * fm-tools ci/check_archive.py check_zipfile(): one common root directory,
    a root-level file whose lowercased name starts with "readme", one starting
    with "license" or "licence", no member matching the forbidden pattern, and
    every symlink resolving inside the archive.
  * fm-tools lib-fm-tools fm_tools/files.py: exactly one top-level directory
    among the members with at most one "/", and the chmod above.
  * fm-weck fm_weck/smoke_test_mode.py locate_and_check_smoke_test_file():
    <root>/smoketest.sh exists, is non-empty, and `st_mode & os.X_OK` -- note
    os.X_OK == 0o001, so the OTHER-execute bit specifically must be set.

`extract` unpacks exactly as fm_tools/files.py does, so the smoke test then
runs against a tree produced by the real extraction path.
"""

import os
import re
import shutil
import stat
import sys
import zipfile

FORBIDDEN = re.compile(r".*(/\.git/|/\.svn/|/\.hg/|/CVS/|/__MACOSX|/\.aptrelease).*")


def fail(msg):
    print(f"*** zipkit: FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def _mode_for(path):
    st = os.lstat(path)
    if stat.S_ISDIR(st.st_mode):
        return stat.S_IFDIR | 0o755
    return stat.S_IFREG | (0o755 if st.st_mode & 0o111 else 0o644)


def pack(stage, top, out):
    entries = []
    root = os.path.join(stage, top)
    entries.append((root, top + "/"))
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames.sort()
        for name in sorted(dirnames):
            p = os.path.join(dirpath, name)
            entries.append((p, os.path.relpath(p, stage) + "/"))
        for name in sorted(filenames):
            p = os.path.join(dirpath, name)
            entries.append((p, os.path.relpath(p, stage)))
    # Directories first, then files, each group sorted: extraction order then
    # never creates a file before its parent directory.
    entries.sort(key=lambda e: (not e[1].endswith("/"), e[1]))

    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
        for path, arcname in entries:
            info = zipfile.ZipInfo.from_file(path, arcname)
            mode = _mode_for(path)
            info.external_attr = mode << 16
            if arcname.endswith("/"):
                info.external_attr |= 0x10  # MS-DOS directory bit
                info.compress_type = zipfile.ZIP_STORED
                zf.writestr(info, b"")
            else:
                info.compress_type = zipfile.ZIP_DEFLATED
                # ZipFile.open(zinfo, "w") reads the level off the ZipInfo, not
                # off the ZipFile, so the constructor's compresslevel=9 would
                # otherwise be ignored. "_compresslevel" is in ZipInfo.__slots__
                # for every Python >= 3.7.
                info._compresslevel = 9
                with open(path, "rb") as src, zf.open(info, "w") as dst:
                    shutil.copyfileobj(src, dst, 1024 * 1024)
    print(f"    packed {len(entries)} entries")


def verify(zip_path, expected_top):
    with zipfile.ZipFile(zip_path) as zf:
        bad = zf.testzip()
        if bad is not None:
            fail(f"corrupt member: {bad}")
        names = zf.namelist()
        if not names:
            fail("archive is empty")

        # check_archive.py: root taken from the FIRST member.
        root = names[0].split("/")[0] + "/"
        if root != expected_top + "/":
            fail(f"first member implies root {root!r}, expected {expected_top + '/'!r}")
        outside = [n for n in names if not n.startswith(root)]
        if outside:
            fail(f"members outside the common root {root!r}: {outside[:5]}")

        # fm_tools/files.py: exactly one top-level directory.
        tops = {n.split("/")[0] for n in names if n.count("/") <= 1}
        if len(tops) != 1:
            fail(f"expected exactly one top-level directory, found {sorted(tops)}")

        def root_file_starting_with(prefix):
            return [
                n
                for n in names
                if n.lower().startswith((root + prefix).lower()) and n.count("/") == 1
            ]

        if not root_file_starting_with("readme"):
            fail(f"no readme* at the archive root {root!r}")
        if not (root_file_starting_with("license") or root_file_starting_with("licence")):
            fail(f"no license*/licence* at the archive root {root!r}")

        offenders = [n for n in names if FORBIDDEN.match(n)]
        if offenders:
            fail(f"members matching the forbidden pattern: {offenders[:5]}")

        directories = {os.path.dirname(f) for f in names}
        for info in zf.infolist():
            attr = info.external_attr >> 16
            if stat.S_ISLNK(attr):
                target = os.path.normpath(
                    os.path.join(
                        os.path.dirname(info.filename),
                        zf.read(info).decode(),
                    )
                )
                if target not in directories and target not in names:
                    fail(f"symlink {info.filename!r} points outside the archive: {target!r}")

        def mode_of(member):
            try:
                return zf.getinfo(member).external_attr >> 16
            except KeyError:
                fail(f"missing member: {member}")

        smoketest = root + "smoketest.sh"
        mode = mode_of(smoketest)
        if mode == 0:
            fail(f"{smoketest} carries no Unix mode bits (fm-tools would extract it non-executable)")
        if not mode & 0o001:
            fail(
                f"{smoketest} mode is {oct(mode & 0o7777)}; fm-weck tests `st_mode & os.X_OK`"
                " (0o001), so the other-execute bit must be set -- use 0755"
            )
        if zf.getinfo(smoketest).file_size == 0:
            fail(f"{smoketest} is empty; fm-weck raises SmokeTestFileIsEmptyError")

        binary = root + "bin/saf"
        mode = mode_of(binary)
        if not mode & 0o111:
            fail(f"{binary} mode is {oct(mode & 0o7777)}; it must be executable")
        if not mode & 0o001:
            fail(f"{binary} mode is {oct(mode & 0o7777)}; the other-execute bit must be set")

        stub = root + "share/saf/stubs/sv-comp-stubs.h"
        if stub not in names:
            fail(f"missing {stub}: SAF would answer 'unknown' on every task")

        print(f"    verified {len(names)} members, root {root!r}")
        print(f"    smoketest.sh mode {oct(mode_of(smoketest) & 0o7777)},"
              f" bin/saf mode {oct(mode_of(binary) & 0o7777)}")


def extract(zip_path, dest):
    """Unpack exactly as fm_tools/files.py does, chmod included."""
    with zipfile.ZipFile(zip_path) as zf:
        for member in zf.namelist():
            info = zf.getinfo(member)
            path = zf.extract(info, dest)
            attr = info.external_attr >> 16
            if attr != 0:
                os.chmod(path, attr)
    print(f"    extracted to {dest}")


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "pack":
        pack(sys.argv[2], sys.argv[3], sys.argv[4])
    elif cmd == "verify":
        verify(sys.argv[2], sys.argv[3])
    elif cmd == "extract":
        extract(sys.argv[2], sys.argv[3])
    else:
        fail(f"unknown command {cmd!r}")
PYKIT

step "pack"

mkdir -p "$OUT_DIR"
rm -f "$OUT_ZIP"

if [ "$PACKER" = zip ]; then
    (cd "$STAGE" && zip -q -r -9 "$OUT_ZIP" "$TOP_DIR")
else
    python3 "$KIT" pack "$STAGE" "$TOP_DIR" "$OUT_ZIP"
fi
[ -s "$OUT_ZIP" ] || die "no archive was produced at $OUT_ZIP"

step "verify the archive"
python3 "$KIT" verify "$OUT_ZIP" "$TOP_DIR"

VERIFY_DIR="$(mktemp -d "${TMPDIR:-/tmp}/saf-archive-verify.XXXXXX")"
python3 "$KIT" extract "$OUT_ZIP" "$VERIFY_DIR"

UNPACKED="$VERIFY_DIR/$TOP_DIR"
[ -x "$UNPACKED/smoketest.sh" ] || die "smoketest.sh is not executable after a faithful extraction"
[ -x "$UNPACKED/bin/saf" ] || die "bin/saf is not executable after a faithful extraction"
info "post-extraction: smoketest.sh and bin/saf are executable"

if [ "${SAF_ARCHIVE_SKIP_SMOKETEST:-0}" = "1" ]; then
    printf '    !! SAF_ARCHIVE_SKIP_SMOKETEST=1 -- the archive has NOT been smoke-tested\n'
else
    step "run smoketest.sh on the unpacked archive"
    (cd "$UNPACKED" && ./smoketest.sh) ||
        die "smoketest.sh failed on the unpacked archive.
      Run it inside the dev container (clang-18/opt-18 must be on PATH), or set
      SAF_ARCHIVE_SKIP_SMOKETEST=1 to build an explicitly unverified archive."
fi

# ---------------------------------------------------------------------------
# 6. Report
# ---------------------------------------------------------------------------

ZIP_BYTES=$(wc -c <"$OUT_ZIP")
ZIP_MIB=$(awk -v b="$ZIP_BYTES" 'BEGIN { printf "%.1f", b / 1048576 }')
if command -v sha256sum >/dev/null 2>&1; then
    (cd "$OUT_DIR" && sha256sum "$ZIP_NAME" >"$ZIP_NAME.sha256")
    SHA="$(cut -d' ' -f1 <"$OUT_ZIP.sha256")"
else
    SHA="(sha256sum unavailable)"
fi

printf '\n'
step "archive ready"
info "path:     $OUT_ZIP"
info "size:     $ZIP_BYTES bytes (${ZIP_MIB} MiB)"
info "sha256:   $SHA"
info "top dir:  $TOP_DIR/  (SV-COMP's mkInstall.sh flattens this away)"
info "version:  $SAF_VERSION"
printf '\n'
