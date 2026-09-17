#!/usr/bin/env bash
# Movement 2 blast-radius control on `unreach-call`.
#
# `convert_global` now interns the global's value type, which lands in
# `AirModule.types`. `absint::build_obj_type_map` builds a size -> struct-type
# index from that table and maps an alloca ONLY when exactly one struct shares its
# size -- so a new entry can DROP an existing mapping as well as add one. That is a
# precision change, and 17.9% of sampled `unreach-call` tasks have an anonymous
# struct global that can trigger it (vs 0% of `no-overflow`, 1% of memsafety).
#
# `unreach-call` is 22,631 tasks at a 21.4 s mean -- ~17 h for the full property.
# This runs a deterministic stride sample instead and diffs per-task outcomes
# against the same tasks in the 55,690-task baseline.
set -euo pipefail
cd ~/static-analyzer-factory
AVAIL=$(df --output=avail -BG / | tail -1 | tr -dc 0-9)
echo "free disk: ${AVAIL}G"
if [ "$AVAIL" -lt 100 ]; then echo "ABORT: need >=100G free"; exit 1; fi
TAG="m4-unreach-control-$(date -u +%Y%m%dT%H%M%SZ)"
echo "run tag: $TAG"; echo "started: $(date -u +%FT%TZ)"
docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c "
  cd /workspace
  python3 scripts/svcomp_split_eval.py \
      --manifest splits/train.jsonl \
      --svb tests/benchmarks/sv-benchmarks \
      --property unreach-call \
      --sample 1500 \
      --confirm-witness \
      --timeout 60 --confirm-timeout 90 --correctness-confirm-timeout 240 \
      --max-rss-mb 15000 \
      --group-weight --weight-cap 1 \
      --jobs 8 \
      --per-task /workspace/$TAG-pertask.jsonl \
      -o /workspace/$TAG.json
"
echo "finished: $(date -u +%FT%TZ)"
