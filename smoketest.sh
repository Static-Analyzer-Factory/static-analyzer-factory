#!/bin/sh
# SAF SV-COMP smoketest (plan 192, P0.2).
#
# Exits 0 iff `saf verify` runs and prints a well-formed SV-COMP verdict on
# stdout (one of: true / unknown / false(<subproperty>)). SV-COMP requires this
# script to pass (exit code 0) as a submission merge gate. It is self-contained
# and runs offline.
#
# Usage: smoketest.sh [path-to-saf-binary]   (default: $SAF, else `saf` on PATH)
set -eu

SAF="${1:-${SAF:-saf}}"

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

cat > "$workdir/trivial.c" <<'EOF'
extern void reach_error(void);
int main(void) {
    if (0) {
        reach_error();
    }
    return 0;
}
EOF

cat > "$workdir/unreach-call.prp" <<'EOF'
CHECK( init(main()), LTL(G ! call(reach_error())) )
EOF

verdict="$("$SAF" verify \
    --property "$workdir/unreach-call.prp" \
    --data-model LP64 \
    --witness "$workdir/witness.yml" \
    "$workdir/trivial.c")"

case "$verdict" in
    true | unknown | false\(*\))
        echo "smoketest: OK (saf verify -> '$verdict')"
        exit 0
        ;;
    *)
        echo "smoketest: FAIL (unexpected stdout: '$verdict')" >&2
        exit 1
        ;;
esac
