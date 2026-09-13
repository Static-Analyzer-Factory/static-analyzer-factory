#!/bin/sh
# Movement 5: SAF's stdout+stderr vs SV-COMP's 2 MB cap -- take 2.
#
# Take 1 was unrepresentative: four of six globs matched nothing, so it only sampled
# busybox and intel-tdx. The risk being tested is the per-instruction
# `tracing::warn!("Unsupported LLVM instruction: ...")` in the LLVM frontend, which
# scales with PROGRAM SIZE. So sample the largest sources in the corpus -- the true
# worst case -- rather than a convenience set of directories.
set -eu
cd /workspace

SVB=tests/benchmarks/sv-benchmarks/c
OUT=/workspace/m5-output-budget.tsv
printf 'src_bytes\tout\terr\ttotal\tpct_of_2MB\tverdict\ttask\n' > "$OUT"

probe() {
    prp="$1"; src="$2"
    o=$(mktemp); e=$(mktemp)
    timeout 150 ./target/release/saf verify \
        --property "$SVB/properties/$prp" --data-model ILP32 \
        --timeout 100 "$src" > "$o" 2> "$e" || true
    sb=$(wc -c < "$src"); bo=$(wc -c < "$o"); be=$(wc -c < "$e"); tot=$((bo+be))
    printf '%s\t%s\t%s\t%s\t%.3f\t%s\t%s\n' \
        "$sb" "$bo" "$be" "$tot" \
        "$(awk -v t=$tot 'BEGIN{printf "%.3f", 100*t/2097152}')" \
        "$(head -1 "$o")" "$(basename "$src")" >> "$OUT"
    rm -f "$o" "$e"
}

echo "=== the 10 largest unreach-call sources in the corpus ==="
find "$SVB" -name '*.i' -o -name '*.c' 2>/dev/null \
    | grep -v '/properties/' \
    | xargs -r ls -S 2>/dev/null | head -10 > /tmp/biggest.txt
while read -r f; do
    printf '  %10d  %s\n' "$(wc -c < "$f")" "$f"
    probe unreach-call.prp "$f"
done < /tmp/biggest.txt

echo
echo "=== a spread across the biggest FAMILIES (4 each) ==="
for fam in eca-rers2012 ldv-linux-3.4-simple product-lines hardness neural-networks hardware-verification-bv; do
    n=0
    for f in "$SVB/$fam"/*; do
        case "$f" in *.i|*.c) ;; *) continue ;; esac
        [ -f "$f" ] || continue
        probe unreach-call.prp "$f"
        n=$((n+1)); [ "$n" -ge 4 ] && break
    done
    echo "  $fam: $n sampled"
done

echo
echo "=== Juliet under valid-memsafety (49,633 tasks, the largest set) ==="
n=0
for f in $(find "$SVB/Juliet_Test" -name '*.c' 2>/dev/null | head -6); do
    probe valid-memsafety.prp "$f"
    n=$((n+1))
done
echo "  Juliet: $n sampled"

echo
echo "=== results, worst first ==="
{ head -1 "$OUT"; tail -n +2 "$OUT" | sort -t"$(printf '\t')" -k4 -rn; } \
    | column -t -s "$(printf '\t')" 2>/dev/null || cat "$OUT"

echo
awk -F'\t' 'NR>1 {n++; if ($4+0>max){max=$4+0; who=$7}; sum+=$4} END {
    printf "=== %d tasks sampled ===\n", n;
    printf "  mean total = %d bytes\n", sum/n;
    printf "  MAX total  = %d bytes (%.3f%% of the 2 MB cap)\n", max, 100*max/2097152;
    printf "  worst      = %s\n", who;
    print (max > 2097152) ? "  *** OVER THE CAP -- must bound the frontend warnings ***" \
                          : "  WITHIN CAP by a factor of " int(2097152/(max?max:1));
}' "$OUT"
