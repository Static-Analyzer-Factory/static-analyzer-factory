#!/usr/bin/env bash
# report.sh — human score view for the SAF loop. Read-only; safe to run anytime.
set -euo pipefail
LOOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SAF_REPO_ROOT:-$(cd "$LOOP_DIR/../.." && pwd)}"
STATE_DIR="${SAF_LOOP_STATE:-$REPO_ROOT/.loop-state}"

echo "== SAF loop report =="
echo "repo:   $REPO_ROOT   branch: $(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo ?)"
echo "state:  $STATE_DIR"
echo
if [ -f "$STATE_DIR/baseline.json" ]; then
  python3 - "$STATE_DIR/baseline.json" <<'PY'
import json,sys; d=json.load(open(sys.argv[1]))
print(f"baseline TRAIN: confirmed={d['confirmed_score']} raw={d.get('raw_score','?')} "
      f"FP={d['false_alarms']} wrongTRUE={d['wrong_true']}")
PY
fi
echo
echo "-- kept arms (auto/loop-*, cap-*) --"
git -C "$REPO_ROOT" for-each-ref --format='  %(refname:short)  %(objectname:short)  %(contents:subject)' \
  'refs/heads/auto/loop-*' 'refs/heads/cap-*' 2>/dev/null || true
echo
echo "-- held-out checks --"
ls -1 "$STATE_DIR"/heldout-*.json 2>/dev/null | while read -r f; do
  python3 - "$f" <<'PY'
import json,sys; d=json.load(open(sys.argv[1]))
print(f"  {sys.argv[1]}: confirmed={d['confirmed_score']} FP={d['false_alarms']} wrongTRUE={d['wrong_true']}")
PY
done
echo
echo "-- journal (last 25) --"
tail -25 "$STATE_DIR/journal.md" 2>/dev/null || tail -25 "$LOOP_DIR/progress-journal.md" 2>/dev/null || true
for a in ALERT_REJECT_TAMPER ALERT_REJECT_HOLDOUT; do
  [ -e "$STATE_DIR/$a" ] && echo && echo "!! SECURITY ALERT: $a present ($STATE_DIR/$a) — investigate"
done
