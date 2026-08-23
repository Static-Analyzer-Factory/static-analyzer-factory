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
exit 0
