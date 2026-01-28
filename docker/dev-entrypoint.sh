#!/usr/bin/env bash
set -e

# Create venv if it doesn't exist
if [ ! -d /workspace/.venv ]; then
    python3 -m venv /workspace/.venv
fi

export VIRTUAL_ENV=/workspace/.venv
export PATH="/workspace/.venv/bin:$PATH"

# Install pytest if missing (check first to avoid pip overhead)
if ! python3 -c "import pytest" 2>/dev/null; then
    pip install -q "pytest>=8.0" "pytest-cov>=5.0"
fi

# Build the Python extension in a separate target dir so it doesn't
# invalidate the main cargo cache used by cargo build/test/nextest.
_saf_maturin_develop() {
    CARGO_TARGET_DIR=/workspace/target-maturin maturin develop
}

# --- Stale Python extension detection via source fingerprint ---
FINGERPRINT_FILE="/workspace/.venv/.saf-build-fingerprint"

# Compute fingerprint of all Rust source + build config files
CURRENT_FINGERPRINT=$(
    find /workspace/crates -name '*.rs' -o -name 'Cargo.toml' | \
    sort | \
    xargs sha256sum 2>/dev/null | \
    sha256sum | cut -d' ' -f1
)

STORED_FINGERPRINT=""
if [ -f "$FINGERPRINT_FILE" ]; then
    STORED_FINGERPRINT=$(cat "$FINGERPRINT_FILE")
fi

if [ "${SKIP_MATURIN_BUILD:-}" != "1" ]; then
    if [ "$CURRENT_FINGERPRINT" != "$STORED_FINGERPRINT" ]; then
        echo "Rust sources changed — rebuilding SAF Python extension..."
        _saf_maturin_develop
        echo "$CURRENT_FINGERPRINT" > "$FINGERPRINT_FILE"
        echo "SAF Python SDK ready."
    elif ! python3 -c "import saf" 2>/dev/null; then
        echo "SAF Python SDK not found — building..."
        _saf_maturin_develop
        echo "$CURRENT_FINGERPRINT" > "$FINGERPRINT_FILE"
        echo "SAF Python SDK ready."
    fi
fi

exec "$@"
