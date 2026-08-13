#!/bin/sh
# Slice 0a/0b — R6 UBSan toolchain + confirm-signal probe (measurement only; NO production code).
# Runs INSIDE the dev Docker image (clang-18 lives only there). Mirrors
# scripts/r5_asan_toolchain_probe.sh, swapping the arbiter to UBSan signed-overflow.
# Answers, at BOTH -m64 and -m32:
#   (1) does `-fsanitize=signed-integer-overflow` compile+LINK (ubsan_standalone runtime present)?
#   (2) does each signed-overflow toy (+,*,unary-,INT_MIN/-1) TRAP with a located report?
#   (3) SIGNED-ONLY discipline: does UNSIGNED wrap and signed LEFT-SHIFT NOT trap under this flag?
#   (4) confirm-signal: exact stderr line + a byte-stable, non-coredumping exit under UBSAN_OPTIONS.
#   (5) R1-rejection feasibility: does the located "file:line:col" point at a HELPER when the
#       overflow is inside one, and does print_stacktrace=1 (needs llvm-symbolizer) yield a FRAME?
# Confirm predicate under test: stderr contains "runtime error: signed integer overflow:".
set -u
CLANG="${SAF_CLANG:-clang-18}"
SANFLAGS="-fsanitize=signed-integer-overflow -fno-sanitize-recover=signed-integer-overflow"
# Deterministic, non-coredumping exit; a stack trace when the symbolizer is available.
UBOPTS="halt_on_error=1:abort_on_error=0:print_stacktrace=1"
WORK="$(mktemp -d)"
cd "$WORK" || exit 1

echo "=== clang ==="
"$CLANG" --version 2>/dev/null | head -1
RTDIR="$("$CLANG" -print-runtime-dir 2>/dev/null)"
echo "runtime-dir: $RTDIR"
echo "--- ubsan runtimes on disk ---"
found=0
for d in "$RTDIR" /usr/lib/llvm-18/lib/clang/*/lib/linux /usr/lib/clang/*/lib/linux; do
  [ -d "$d" ] || continue
  ls -1 "$d" 2>/dev/null | grep -i 'ubsan' | sed "s|^|  $d/|" && found=1
done
[ "$found" = 0 ] && echo "  (no libclang_rt.ubsan* found)"
echo "--- symbolizer ---"
command -v llvm-symbolizer llvm-symbolizer-18 2>/dev/null || echo "  (no llvm-symbolizer on PATH)"
echo

# Runtime-opaque operands (volatile) so clang cannot constant-fold the op away — the
# overflow must be a genuine RUNTIME operation for UBSan to instrument + trap.
cat > add.c <<'C'
#include <limits.h>
int main(void){ volatile int x = INT_MAX; return x + 1; }
C
cat > mul.c <<'C'
#include <limits.h>
int main(void){ volatile int x = 100000, y = 100000; return x * y; }
C
cat > neg.c <<'C'
#include <limits.h>
int main(void){ volatile int x = INT_MIN; return -x; }
C
cat > divmin.c <<'C'
#include <limits.h>
int main(void){ volatile int x = INT_MIN, y = -1; return x / y; }
C
# SIGNED-ONLY discipline: these must NOT trap under -fsanitize=signed-integer-overflow.
cat > uwrap.c <<'C'
#include <limits.h>
int main(void){ volatile unsigned x = UINT_MAX; return (int)(x + 1u); }
C
cat > lshift.c <<'C'
int main(void){ volatile int x = 1; return x << 31; }
C
# R1-rejection feasibility: the overflow happens INSIDE a helper `h`, called from main.
# Does the located line point at the helper (bad for line-only rejection), and does
# print_stacktrace=1 attribute a frame `h` (good for frame-based rejection)?
cat > helper.c <<'C'
#include <limits.h>
__attribute__((noinline)) int h(int a){ return a + 1; }
int main(void){ volatile int x = INT_MAX; return h(x); }
C

run_one() {
  name="$1"; bits="$2"; expect="$3"   # expect=trap|clean
  bin="./${name}_${bits}"
  clog="$(mktemp)"
  if ! "$CLANG" -O0 -g $SANFLAGS "-m${bits}" "${name}.c" -o "$bin" >"$clog" 2>&1; then
    echo "[$name m$bits] COMPILE/LINK FAIL:"
    sed 's/^/    /' "$clog" | head -6
    return
  fi
  rlog="$(mktemp)"
  UBSAN_OPTIONS="$UBOPTS" "$bin" >"$rlog" 2>&1
  ec=$?
  line="$(grep -m1 'runtime error:' "$rlog" 2>/dev/null | sed 's/^[ =]*//' | cut -c1-120)"
  frame="$(grep -m1 -E '^ *#[0-9]+ ' "$rlog" 2>/dev/null | sed 's/^ *//' | cut -c1-90)"
  if [ -n "$line" ]; then
    echo "[$name m$bits expect=$expect] exit=$ec  TRAP: $line"
    [ -n "$frame" ] && echo "        frame0: $frame"
  else
    echo "[$name m$bits expect=$expect] exit=$ec  no-trap"
  fi
}

for bits in 64 32; do
  echo "=== -m$bits (signed overflow -> MUST trap) ==="
  run_one add     "$bits" trap
  run_one mul     "$bits" trap
  run_one neg     "$bits" trap
  run_one divmin  "$bits" trap
  echo "--- -m$bits (defined behavior -> MUST NOT trap) ---"
  run_one uwrap   "$bits" clean
  run_one lshift  "$bits" clean
  echo "--- -m$bits (overflow inside helper h() — R1 attribution) ---"
  run_one helper  "$bits" trap
  echo
done
rm -rf "$WORK"
