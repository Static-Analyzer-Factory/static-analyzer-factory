#!/usr/bin/env bash
# Convert CruxBC .bc corpus (from the ptaben submodule) into mem2reg-optimized
# .ll files that SAF's cruxbc benchmark harness consumes.
#
# Source:  tests/benchmarks/ptaben/test_cases_bc/crux-bc/*.bc
# Output:  tests/benchmarks/cruxbc/.compiled/<category>/<name>.ll
#
# Runs inside the saf-dev Docker container (LLVM 18): invoked via
#   docker compose run --rm dev ./scripts/prepare-cruxbc.sh
set -euo pipefail

SRC_DIR="tests/benchmarks/ptaben/test_cases_bc/crux-bc"
OUT_DIR="tests/benchmarks/cruxbc/.compiled"

OPT="$(command -v opt-18 || command -v opt)"

# Category assignment (matches historical benchmark layout):
#   small — utilities and libraries under ~60K IR instructions
#   big   — bash, libcurl
#   extra — tmux (largest, ~176K lines)
category_of() {
  case "$1" in
    bash|libcurl.so) echo big ;;
    tmux) echo extra ;;
    *) echo small ;;
  esac
}

mkdir -p "$OUT_DIR"

for bc in "$SRC_DIR"/*.bc; do
  name="$(basename "$bc" .bc)"
  cat="$(category_of "$name")"
  out="$OUT_DIR/$cat/$name.ll"
  mkdir -p "$OUT_DIR/$cat"
  if [ -f "$out" ] && [ "$out" -nt "$bc" ]; then
    echo "up-to-date: $cat/$name.ll"
    continue
  fi
  echo "mem2reg: $name.bc -> $cat/$name.ll"
  "$OPT" -passes=mem2reg -S "$bc" -o "$out"
done

echo "Done. Compiled corpus in $OUT_DIR"
