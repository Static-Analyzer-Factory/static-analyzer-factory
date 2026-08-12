#!/bin/sh
# Slice 0a — R5 ASan toolchain probe (measurement only; NO production code).
# Runs INSIDE the dev Docker image (clang-18 lives only there). Answers:
#   (1) does `-fsanitize=address` compile+trap on 4 memory-safety toys at -m64?
#   (2) does it work at -m32 (the ILP32 majority) or link-fail (missing i386 ASan runtime)?
#   (3) is libclang_rt.asan-{x86_64,i386} present in the clang resource dir?
# Confirm predicate: stderr contains "ERROR: AddressSanitizer: <class>".
set -u
CLANG="${SAF_CLANG:-clang-18}"
WORK="$(mktemp -d)"
cd "$WORK" || exit 1

echo "=== clang ==="
"$CLANG" --version 2>/dev/null | head -1
RTDIR="$("$CLANG" -print-runtime-dir 2>/dev/null)"
echo "runtime-dir: $RTDIR"
echo "--- asan runtimes on disk ---"
found=0
for d in "$RTDIR" /usr/lib/llvm-18/lib/clang/*/lib/linux /usr/lib/clang/*/lib/linux; do
  [ -d "$d" ] || continue
  ls -1 "$d" 2>/dev/null | grep -i 'asan' | sed "s|^|  $d/|" && found=1
done
[ "$found" = 0 ] && echo "  (no libclang_rt.asan* found)"
echo

cat > uaf.c <<'C'
#include <stdlib.h>
int main(void){int*p=malloc(sizeof(int));free(p);return *p;}
C
cat > oob.c <<'C'
#include <stdlib.h>
int main(void){int*a=malloc(4*sizeof(int));a[100]=1;return a[0];}
C
cat > dfree.c <<'C'
#include <stdlib.h>
int main(void){int*p=malloc(sizeof(int));free(p);free(p);return 0;}
C
cat > nullderef.c <<'C'
int main(void){volatile int*p=0;return *p;}
C

run_one() {
  name="$1"; src="$2"; bits="$3"
  bin="./${name}_${bits}"
  clog="$(mktemp)"
  if ! "$CLANG" -O0 -g -fsanitize=address -fno-sanitize-recover=address "-m${bits}" "$src" -o "$bin" >"$clog" 2>&1; then
    echo "[$name m$bits] COMPILE/LINK FAIL:"
    sed 's/^/    /' "$clog" | head -6
    return
  fi
  rlog="$(mktemp)"
  ASAN_OPTIONS=exitcode=1:abort_on_error=0:detect_leaks=0 "$bin" >"$rlog" 2>&1
  ec=$?
  line="$(grep -m1 'ERROR: AddressSanitizer' "$rlog" 2>/dev/null | sed 's/^[ =]*//' | cut -c1-100)"
  if [ -n "$line" ]; then
    echo "[$name m$bits] exit=$ec  $line"
  else
    echo "[$name m$bits] exit=$ec  NO ASan banner"
    sed 's/^/    /' "$rlog" | head -3
  fi
}

for bits in 64 32; do
  echo "=== -m$bits ==="
  run_one uaf uaf.c "$bits"
  run_one oob oob.c "$bits"
  run_one dfree dfree.c "$bits"
  run_one nullderef nullderef.c "$bits"
  echo
done
rm -rf "$WORK"
