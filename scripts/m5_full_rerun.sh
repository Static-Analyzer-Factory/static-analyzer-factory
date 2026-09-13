#!/usr/bin/env bash
# The authoritative 55,690-task re-run at HEAD, with both SV-COMP-participant
# tools removed. Produces a MEASURED number rather than the re-score that gave 123.
#
# Flag choices, each deliberate:
#   --manifest splits/train.jsonl   the authoritative population (exactly 55,690 rows)
#   --timeout 60                    UNCHANGED from the baseline. SV-COMP grants 900 s
#                                   CPU; 60 s is a practical necessity at this scale and
#                                   is CONSERVATIVE, so the result is a lower bound.
#   --confirm-timeout 90            CHANGED from the baseline's 150. SV-COMP allows only
#                                   90 s for VIOLATION-witness validation, so 150 counted
#                                   confirmations the competition would never grant. The
#                                   witness audit named this "the single highest-risk
#                                   unquantified gap in the number"; this run closes it.
#   --correctness-confirm-timeout 240   unchanged; SV-COMP allows 300, so conservative.
#   --max-rss-mb 15000              SV-COMP's real memory limit. Also an OOM regression
#                                   detector: peak RSS at HEAD is unverified since the
#                                   d2ee9313 fix (19 GB -> 244 MB).
#   --group-weight --weight-cap 1   dedup-weighted by (group, property), the honest metric.
#
# The delta against 123 is attributable to ONE thing: the tightened confirmation budget.
set -euo pipefail
cd ~/static-analyzer-factory

TAG="saf-alone-$(date -u +%Y%m%dT%H%M%SZ)"
echo "run tag: $TAG"
echo "HEAD:    $(git rev-parse --short HEAD) on $(git rev-parse --abbrev-ref HEAD)"
echo "started: $(date -u +%FT%TZ)"

docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c "
  set -e
  cd /workspace
  cargo build --release -p saf-cli 2>&1 | tail -3
  python3 scripts/svcomp_split_eval.py \
      --manifest splits/train.jsonl \
      --svb tests/benchmarks/sv-benchmarks \
      --confirm-witness \
      --timeout 60 \
      --confirm-timeout 90 \
      --correctness-confirm-timeout 240 \
      --max-rss-mb 15000 \
      --group-weight --weight-cap 1 \
      --jobs 8 \
      --per-task /workspace/$TAG-pertask.jsonl \
      -o /workspace/$TAG.json
"
echo "finished: $(date -u +%FT%TZ)"
echo "artifacts: $TAG.json  $TAG-pertask.jsonl"
