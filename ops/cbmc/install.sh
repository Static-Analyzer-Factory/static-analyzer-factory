#!/usr/bin/env bash
# Provision CBMC for the SAF eval's witness-confirmation panel (validate_witness.sh
# Stage 4). CBMC is an official SV-COMP validator that confirms memory-safety and
# reachability violations CPAchecker/cpa-witness2test miss. This stages the cbmc
# binary + libminisat.so.2 into .svtools/cbmc, which is bind-mounted into the dev
# container at /workspace/.svtools/cbmc (validate_witness.sh reads $SAF_CBMC there).
# Idempotent; run once per VM (e.g. after a rebuild). Host and dev image are both
# Ubuntu 24.04, so the host binary runs in-container as-is with LD_LIBRARY_PATH.
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEST="$REPO/.svtools/cbmc"

if [ -x "$DEST/cbmc" ]; then
    echo "cbmc already staged at $DEST ($("$DEST/cbmc" --version 2>/dev/null || echo '?'))"
    exit 0
fi
if ! command -v cbmc >/dev/null 2>&1; then
    echo "installing cbmc via apt..."
    sudo apt-get update -qq && sudo apt-get install -y cbmc
fi
mkdir -p "$DEST"
install -m 0755 "$(command -v cbmc)" "$DEST/cbmc"
# stage the one non-standard shared lib (the SAT backend); the rest are in the image
lib="$(ldd "$(command -v cbmc)" | awk '/libminisat/{print $3}')"
[ -n "${lib:-}" ] && [ -f "$lib" ] && install -m 0644 "$lib" "$DEST/"
echo "staged cbmc -> $DEST"
LD_LIBRARY_PATH="$DEST" "$DEST/cbmc" --version
