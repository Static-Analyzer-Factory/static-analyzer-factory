#!/usr/bin/env bash
# Behaviour-preservation gate for the universe.rs extraction.
# Re-runs ONLY the two properties whose verdicts the gate can move, and diffs
# against m2-universe-baseline.jsonl (captured from the post-Movement-1 binary).
# These two callers hold FP=0 across 55,690 tasks: ANY outcome change is a regression.
set -euo pipefail
cd ~/static-analyzer-factory
AVAIL=$(df --output=avail -BG / | tail -1 | tr -dc 0-9)
echo "free disk: ${AVAIL}G"
[ "$AVAIL" -lt 100 ] && { echo "ABORT: need >=100G"; exit 1; }
echo "=== building release ==="
docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c "cargo build --release --bin saf" > /tmp/m2rb.log 2>&1
echo "release build=$?"
for P in no-data-race termination; do
  echo "=== $P ==="
  docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c "
    cd /workspace && python3 scripts/svcomp_split_eval.py \
      --manifest splits/train.jsonl --svb tests/benchmarks/sv-benchmarks \
      --property $P --confirm-witness --timeout 60 --confirm-timeout 90 \
      --correctness-confirm-timeout 240 --max-rss-mb 15000 \
      --group-weight --weight-cap 1 --jobs 8 \
      --per-task /workspace/m2-oracle-$P-pertask.jsonl -o /workspace/m2-oracle-$P.json" 2>&1 | tail -6
done
echo "=== DIFF vs baseline ==="
python3 scripts/m2_oracle_diff.py
