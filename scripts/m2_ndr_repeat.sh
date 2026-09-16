#!/usr/bin/env bash
# Is the no-data-race churn NONDETERMINISM or a real behaviour change?
# Re-run the SAME binary on the SAME tasks. If a comparable number of tasks flip
# bidirectionally again, the churn is inherent to the race search, not the refactor.
set -euo pipefail
cd ~/static-analyzer-factory
docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c "
  cd /workspace && python3 scripts/svcomp_split_eval.py \
    --manifest splits/train.jsonl --svb tests/benchmarks/sv-benchmarks \
    --property no-data-race --confirm-witness --timeout 60 --confirm-timeout 90 \
    --correctness-confirm-timeout 240 --max-rss-mb 15000 \
    --group-weight --weight-cap 1 --jobs 8 \
    --per-task /workspace/m2-oracle-ndr-repeat-pertask.jsonl -o /workspace/m2-oracle-ndr-repeat.json" 2>&1 | tail -4
python3 - <<PY
import json
def load(f):
    return {json.loads(l)["rel_yml"]: json.loads(l)["outcome"] for l in open(f)}
a = load("m2-oracle-no-data-race-pertask.jsonl")
b = load("m2-oracle-ndr-repeat-pertask.jsonl")
common = [k for k in b if k in a]
d = [(k, a[k], b[k]) for k in common if a[k] != b[k]]
print(f"SAME BINARY, SAME TASKS: compared={len(common)} changed={len(d)}")
gain = sum(1 for _,x,y in d if y.endswith("Correct"))
print(f"  gains={gain} losses={len(d)-gain}  (bidirectional => nondeterminism)")
for k,x,y in d[:12]: print(f"    {k:56s} {x} -> {y}")
PY
