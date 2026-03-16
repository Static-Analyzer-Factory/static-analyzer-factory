#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR"

echo "=== Syncing core references to Claude Code plugin ==="
mkdir -p "$ROOT/claude-code/skills/saf-feature-dev/references"
cp "$ROOT/core/references/"*.md "$ROOT/claude-code/skills/saf-feature-dev/references/"

echo "=== Checking Codex workflow exists ==="
if [ ! -f "$ROOT/codex/saf-feature-dev-workflow.md" ]; then
    echo "WARNING: codex/saf-feature-dev-workflow.md not found"
    exit 1
fi

CODEX_SIZE=$(wc -c < "$ROOT/codex/saf-feature-dev-workflow.md")
if [ "$CODEX_SIZE" -gt 15360 ]; then
    echo "WARNING: Codex workflow is ${CODEX_SIZE} bytes (limit: 15360)"
    echo "  The SAF repo AGENTS.md is ~13 KiB; combined must be under 32 KiB."
fi

echo "=== Checking for stale references ==="
STALE=0
for f in "$ROOT/core/references/"*.md; do
    base=$(basename "$f")
    cc="$ROOT/claude-code/skills/saf-feature-dev/references/$base"
    if [ ! -f "$cc" ]; then
        echo "MISSING: $base not in claude-code/skills/saf-feature-dev/references/"
        STALE=1
    elif [ "$f" -nt "$cc" ]; then
        echo "STALE: $base (core is newer than claude-code copy)"
        STALE=1
    fi
done

if [ $STALE -eq 1 ]; then
    echo "References synced. Review Codex file for manual sync if core content changed."
else
    echo "All references up to date."
fi

echo "=== Done ==="
