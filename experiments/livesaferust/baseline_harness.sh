#!/usr/bin/env bash
set -euo pipefail

# Baseline harness for the LiveSafeRust proposal.
#
# This script is intentionally not invoked by the Node POC. Run it on a machine
# with cargo, rust-analyzer, and hyperfine installed. It gives the external
# latency anchor that the paper still needs.

if [[ $# -lt 1 ]]; then
  echo "usage: $0 /path/to/rust/crate" >&2
  exit 2
fi

crate_dir="$1"
out_dir="${2:-$(pwd)/experiments/livesaferust/out/baseline}"
mkdir -p "$out_dir"

command -v cargo >/dev/null
command -v rust-analyzer >/dev/null

echo "crate,$crate_dir" > "$out_dir/baseline.csv"

if command -v hyperfine >/dev/null; then
  hyperfine --warmup 1 --export-json "$out_dir/cargo-check-cold.json" \
    "cd '$crate_dir' && cargo clean && cargo check"

  hyperfine --warmup 2 --export-json "$out_dir/cargo-check-warm.json" \
    "cd '$crate_dir' && cargo check"
else
  echo "hyperfine not found; using coarse wall-clock fallback" >&2
  (cd "$crate_dir" && cargo clean)
  /usr/bin/time -p sh -c "cd '$crate_dir' && cargo check" \
    2> "$out_dir/cargo-check-cold.time"
  /usr/bin/time -p sh -c "cd '$crate_dir' && cargo check" \
    2> "$out_dir/cargo-check-warm.time"
fi

cat > "$out_dir/README.md" <<'EOF'
# Baseline Output

Fill these values into `PAPER_OUTLINE.md` / the LaTeX draft:

| System | Scenario | Time |
| --- | --- | ---: |
| cargo check | cold | TODO |
| cargo check | warm after one-line edit | TODO |
| rust-analyzer | didChange to publishDiagnostics | TODO |
| Clippy | cold | TODO |

The rust-analyzer LSP latency still needs an editor/LSP client probe. This
harness only measures cargo-facing baselines.
EOF

echo "wrote $out_dir"
