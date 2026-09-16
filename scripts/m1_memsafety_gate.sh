#!/usr/bin/env bash
# Movement 1: the valid-memsafety blast-radius gate.
# `analyze_memsafety` consumes `run_sccp_module` and prunes SVFG dead-PHI edges to
# REDUCE false positives. The SCCP fix shrinks `dead_blocks`, so less pruning =>
# possibly MORE false alarms. 20,570 tasks, -16/-32 each. Same flags as
# scripts/m5_full_rerun.sh so the rows splice into the baseline directly.
set -euo pipefail
cd ~/static-analyzer-factory
AVAIL=$(df --output=avail -BG / | tail -1 | tr -dc 0-9)
echo "free disk: ${AVAIL}G"
if [ "$AVAIL" -lt 100 ]; then echo "ABORT: need >=100G free (the last run died on ENOSPC)"; exit 1; fi
TAG="m1-memsafety-$(date -u +%Y%m%dT%H%M%SZ)"
echo "run tag: $TAG"; echo "started: $(date -u +%FT%TZ)"
docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c "
  cd /workspace
  python3 scripts/svcomp_split_eval.py \
      --manifest splits/train.jsonl \
      --svb tests/benchmarks/sv-benchmarks \
      --property valid-memsafety \
      --confirm-witness \
      --timeout 60 --confirm-timeout 90 --correctness-confirm-timeout 240 \
      --max-rss-mb 15000 \
      --group-weight --weight-cap 1 \
      --jobs 8 \
      --per-task /workspace/$TAG-pertask.jsonl \
      -o /workspace/$TAG.json
"
echo "finished: $(date -u +%FT%TZ)"
