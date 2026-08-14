#!/usr/bin/env bash
# run_tests.sh — all Stage-0 gate/orchestration tests for the SAF loop (no Docker/Claude needed).
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
rc=0
for t in test_gates.py test_ratelimit.py test_verify_arm.py; do
  echo "== $t =="
  python3 "$HERE/$t" || rc=1
done
echo "== smoke_supervisor.sh =="
bash "$HERE/smoke_supervisor.sh" || rc=1
echo
[ "$rc" -eq 0 ] && echo "ALL LOOP TESTS GREEN" || echo "SOME LOOP TESTS FAILED"
exit "$rc"
