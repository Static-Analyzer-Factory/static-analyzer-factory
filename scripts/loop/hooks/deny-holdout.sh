#!/usr/bin/env bash
# PreToolUse hook: PREVENTIVE holdout-read denylist (Addendum A6 — isolation = BOTH preventive + audit).
# Claude Code pipes the tool-call JSON on stdin. If the tool input references the held-out manifest,
# block it (exit 2 + a reason on stderr). This is the fail-closed first layer; verify_arm.py's
# transcript audit is the second. Forbidden substrings come from SAF_FORBIDDEN_READS (default below).
set -euo pipefail
FORBIDDEN="${SAF_FORBIDDEN_READS:-svcomp-splits/holdout}"
payload="$(cat)"
for f in $FORBIDDEN; do
  if printf '%s' "$payload" | grep -qF -- "$f"; then
    echo "BLOCKED: this loop arm must not access the held-out set ($f). Use train.jsonl only." >&2
    exit 2
  fi
done
exit 0
