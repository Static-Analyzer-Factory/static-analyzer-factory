#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR"
SIBLING="$SCRIPT_DIR/../saf-feature-dev"

echo "=== Syncing own references to Claude Code plugin ==="
mkdir -p "$ROOT/claude-code/skills/saf-checker-dev/references"
cp "$ROOT/core/references/"*.md "$ROOT/claude-code/skills/saf-checker-dev/references/"

echo "=== Copying shared references from saf-feature-dev ==="
for f in saf-invariants.md saf-log-guide.md e2e-testing-guide.md; do
    src="$SIBLING/core/references/$f"
    dst="$ROOT/claude-code/skills/saf-checker-dev/references/$f"
    if [ -f "$src" ]; then
        cp "$src" "$dst"
    else
        echo "WARNING: shared reference $f not found at $src"
    fi
done

echo "=== Checking Codex workflow exists ==="
if [ ! -f "$ROOT/codex/saf-checker-dev-workflow.md" ]; then
    echo "WARNING: codex/saf-checker-dev-workflow.md not found"
    exit 1
fi

CODEX_SIZE=$(wc -c < "$ROOT/codex/saf-checker-dev-workflow.md")
if [ "$CODEX_SIZE" -gt 15360 ]; then
    echo "WARNING: Codex workflow is ${CODEX_SIZE} bytes (limit: 15360)"
fi

echo "=== Checking for stale references ==="
STALE=0
for f in "$ROOT/core/references/"*.md; do
    base=$(basename "$f")
    cc="$ROOT/claude-code/skills/saf-checker-dev/references/$base"
    if [ ! -f "$cc" ]; then
        echo "MISSING: $base"
        STALE=1
    elif [ "$f" -nt "$cc" ]; then
        echo "STALE: $base"
        STALE=1
    fi
done

if [ $STALE -eq 1 ]; then
    echo "References synced."
else
    echo "All references up to date."
fi

echo "=== Done ==="
