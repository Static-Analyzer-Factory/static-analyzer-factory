#!/usr/bin/env bash
# Provision the CBMC oracle used by SAF's unreach-call cbmc lever
# (crates/saf-cli resolve_cbmc) and the witness validator Stage 4
# (scripts/validate_witness.sh). Both look for $SAF_CBMC (default
# /workspace/.svtools/cbmc)/cbmc.
#
# The CPAchecker-bundled cbmc is 5.12, which lacks --no-standard-checks (the flag
# SAF's lever requires), so we fetch a modern release matching the container's
# Ubuntu 24.04 base. cbmc 6.11.0 is verified compatible with SAF's invocation
# (--ILP32/--LP64 --unwind --no-standard-checks --stop-on-fail --trace
# --drop-unused-functions) and produces a __VERIFIER_nondet_* counterexample.
#
# Idempotent. Run from the repo root on each eval box (host side; the container
# sees it via the .:/workspace bind mount). .svtools is a gitignored cache.
set -euo pipefail

CBMC_VERSION="${CBMC_VERSION:-6.11.0}"
DEB="ubuntu-24.04-cbmc-${CBMC_VERSION}-Linux.deb"
URL="https://github.com/diffblue/cbmc/releases/download/cbmc-${CBMC_VERSION}/${DEB}"
SVT="${SVT:-.svtools}"
DEST="${SVT}/cbmc"

if [ -x "${DEST}/cbmc.real" ] && "${DEST}/cbmc.real" --version 2>/dev/null | grep -q "${CBMC_VERSION}"; then
  echo "cbmc ${CBMC_VERSION} already provisioned at ${DEST}/cbmc"
  exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
echo "downloading cbmc ${CBMC_VERSION} ..."
curl -fsSL -o "${tmp}/cbmc.deb" "$URL"
dpkg-deb -x "${tmp}/cbmc.deb" "${tmp}/x"
mkdir -p "${DEST}"
cp "${tmp}/x/usr/bin/cbmc" "${DEST}/cbmc.real"
chmod +x "${DEST}/cbmc.real"
ln -sf cbmc.real "${DEST}/cbmc"
echo "provisioned $("${DEST}/cbmc" --version) -> ${DEST}/cbmc"
