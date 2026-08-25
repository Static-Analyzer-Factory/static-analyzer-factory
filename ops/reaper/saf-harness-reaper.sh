#!/usr/bin/env bash
# Reap leaked SAF child processes that outlive their intended budget (load-runaway guard).
# Covers:
#   - sanitizer replay harnesses (saf_*_harness), the `saf verify` driver, captest harnesses
#   - CPAchecker-family witness validators (org.sosy_lab.cpachecker.cmdline.CPAMain, which
#     also backs cpa-witness2test). scripts/validate_witness.sh runs them with an internal
#     `--timelimit 90s` the JVM does NOT hard-enforce, so they orphan to init when the parent
#     eval dies and peg a core forever. The harness process-group kill (commands.rs) does not
#     reach them. See saf-loop-harness-leak-incident (this is the same failure one layer up).
# Anything older than MAX_AGE seconds is definitively stuck (largest intended budget = 90s).
MAX_AGE="${SAF_REAPER_MAX_AGE:-300}"
PATTERN='saf_[a-z0-9]*_harness|release/saf verify|saf_captest|org\.sosy_lab\.cpachecker'
reaped=0
for pid in $(pgrep -f "$PATTERN" 2>/dev/null); do
  s=$(ps -o etimes= -p "$pid" 2>/dev/null | tr -d ' ')
  if [ -n "$s" ] && [ "$s" -gt "$MAX_AGE" ] 2>/dev/null; then
    pgid=$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')
    # kill the whole process group (launcher + JVM grandchildren) when it is a real,
    # isolated group; always fall back to the direct pid.
    if [ -n "$pgid" ] && [ "$pgid" -gt 1 ] 2>/dev/null; then kill -9 -"$pgid" 2>/dev/null || true; fi
    kill -9 "$pid" 2>/dev/null || true
    comm=$(ps -o comm= -p "$pid" 2>/dev/null | tr -d ' ')
    echo "reaped leaked pid=$pid pgid=$pgid comm=$comm age=${s}s"
    reaped=$((reaped+1))
  fi
done
[ "$reaped" -gt 0 ] && echo "saf-harness-reaper: reaped $reaped leaked process(es)"

# --- Reap ORPHANED eval containers -------------------------------------------
# The eval runs in a `docker compose run --rm dev` container. On a supervisor stop
# or OOM-crash the `docker compose run` PARENT dies but the container keeps running
# under dockerd (the --rm never fires). Because mem_limit is PER-container, an orphan
# (<=52g) coexisting with a fresh eval (<=52g) can exceed box RAM -> SYSTEM OOM. So:
# if NO `docker compose run ... dev` parent is alive, any saf-dev-run container is
# orphaned -> kill it. Live evals (the loop's or a manual one) keep a parent, so are
# left untouched. All docker calls are timeout-wrapped so a slow daemon cannot hang
# the reaper.
if command -v docker >/dev/null 2>&1; then
  cids=$(timeout 20 docker ps -q --filter name=static-analyzer-factory-dev-run 2>/dev/null)
  if [ -n "$cids" ] && ! pgrep -f 'compose.*run.*dev' >/dev/null 2>&1; then
    for cid in $cids; do
      timeout 30 docker kill "$cid" >/dev/null 2>&1 && echo "saf-harness-reaper: killed orphaned eval container $cid"
    done
  fi
fi
exit 0
