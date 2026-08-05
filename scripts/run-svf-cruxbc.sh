#!/usr/bin/env bash
# Run SVF (wpa -fspta) on the CruxBC corpus via the svftools/svf Docker image,
# recording wall-clock time and peak RSS per program.
#
# Usage: scripts/run-svf-cruxbc.sh [--compiled-dir DIR] [--filter SUBSTR] [-o OUT.json]
#
# Full per-program SVF stat output is saved to tests/benchmarks/cruxbc/svf-logs/.
set -euo pipefail

COMPILED_DIR="tests/benchmarks/cruxbc/.compiled"
FILTER=""
OUT="tests/benchmarks/cruxbc/svf-mem2reg-results.json"

while [ $# -gt 0 ]; do
  case "$1" in
    --compiled-dir) COMPILED_DIR="$2"; shift 2 ;;
    --filter) FILTER="$2"; shift 2 ;;
    -o) OUT="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

LOG_DIR="tests/benchmarks/cruxbc/svf-logs"
mkdir -p "$LOG_DIR"

WPA=/home/SVF-tools/SVF/Release-build/bin/wpa

results="["
first=1

for ll in $(find "$COMPILED_DIR" -name '*.ll' | sort); do
  rel="${ll#"$COMPILED_DIR"/}"
  cat="$(dirname "$rel")"
  name="$(basename "$rel" .ll)"
  label="$cat/$name"

  if [ -n "$FILTER" ]; then
    case "$label" in *"$FILTER"*) ;; *) continue ;; esac
  fi

  echo "=== SVF wpa -fspta: $label ===" >&2
  start=$(date +%s.%N)
  set +e
  docker run --rm -v "$PWD/$COMPILED_DIR:/data:ro" svftools/svf:latest sh -c "
    $WPA -fspta -stat /data/$rel > /tmp/out.txt 2>&1 &
    pid=\$!
    peak=0
    while kill -0 \$pid 2>/dev/null; do
      cur=\$(awk '/VmHWM/{print \$2}' /proc/\$pid/status 2>/dev/null)
      [ -n \"\$cur\" ] && peak=\$cur
      sleep 0.2
    done
    wait \$pid; code=\$?
    echo \"EXIT_CODE=\$code PEAK_RSS_KB=\$peak\" >> /tmp/out.txt
    cat /tmp/out.txt
    exit \$code
  " > "$LOG_DIR/$name.txt" 2>&1
  code=$?
  set -e
  end=$(date +%s.%N)
  wall=$(echo "$end $start" | awk '{printf "%.2f", $1-$2}')
  peak_kb=$(grep -oE 'PEAK_RSS_KB=[0-9]+' "$LOG_DIR/$name.txt" | tail -1 | cut -d= -f2)
  peak_kb=${peak_kb:-0}

  echo "  $label: ${wall}s, peak $((peak_kb / 1024))MB, exit $code" >&2

  [ $first -eq 1 ] || results="$results,"
  first=0
  results="$results{\"name\":\"$name\",\"category\":\"$cat\",\"wall_secs\":$wall,\"peak_rss_mb\":$((peak_kb / 1024)),\"exit_code\":$code}"
done

results="$results]"
echo "{\"suite\":\"cruxbc-svf\",\"tool\":\"wpa -fspta\",\"programs\":$results}" > "$OUT"
echo "Wrote $OUT" >&2
