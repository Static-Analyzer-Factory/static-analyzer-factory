#!/usr/bin/env python3
"""Compare SAF vs SVF CruxBC benchmark results (time + peak memory).

Usage:
    python3 scripts/compare-cruxbc.py \
        --saf-results tests/benchmarks/cruxbc/saf-results.json \
        [--svf-results tests/benchmarks/cruxbc/svf-mem2reg-results.json] \
        [--llvm-cg-results tests/benchmarks/cruxbc/llvm-cg-results.json] \
        [--saf-prev tests/benchmarks/cruxbc/saf-results.prev.json]
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def load(path: str | None) -> dict | None:
    if not path or not Path(path).exists():
        return None
    with open(path) as f:
        return json.load(f)


def index_programs(results: dict | None) -> dict[str, dict]:
    if not results:
        return {}
    return {f"{p['category']}/{p['name']}": p for p in results.get("programs", [])}


def fmt_secs(v: float | None) -> str:
    return f"{v:.2f}s" if v is not None else "—"


def fmt_mb(v: int | None) -> str:
    return f"{v}MB" if v else "—"


def fmt_ratio(a: float | None, b: float | None) -> str:
    if a is None or b is None or b == 0:
        return "—"
    return f"{a / b:.1f}x"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--saf-results", required=True)
    ap.add_argument("--svf-results")
    ap.add_argument("--llvm-cg-results")
    ap.add_argument("--saf-prev")
    args = ap.parse_args()

    saf = index_programs(load(args.saf_results))
    svf = index_programs(load(args.svf_results))
    prev = index_programs(load(args.saf_prev))

    labels = sorted(saf.keys() | svf.keys())

    hdr = (
        f"{'Program':<20} {'SAF':>9} {'SVF':>9} {'SAF/SVF':>8} "
        f"{'SAF RSS':>9} {'SVF RSS':>9} {'RSSx':>6} {'Δprev':>8}"
    )
    print(hdr)
    print("─" * len(hdr))

    for label in labels:
        s = saf.get(label)
        v = svf.get(label)
        p = prev.get(label)

        s_time = s.get("total_secs") if s and not s.get("error") else None
        v_time = v.get("wall_secs") if v and v.get("exit_code", 0) == 0 else None
        s_rss = s.get("peak_rss_mb") if s else None
        v_rss = v.get("peak_rss_mb") if v else None

        delta = "—"
        if p and s_time is not None and p.get("total_secs"):
            d = s_time - p["total_secs"]
            delta = f"{d:+.2f}s"

        err = ""
        if s and s.get("error"):
            err = f"  SAF-ERR: {s['error'][:40]}"
        if s and (s.get("stats") or {}).get("fspta_limit_hit"):
            err += "  SAF-FS-TRUNCATED"
        if v and v.get("exit_code", 0) != 0:
            err += f"  SVF-EXIT: {v['exit_code']}"

        print(
            f"{label:<20} {fmt_secs(s_time):>9} {fmt_secs(v_time):>9} "
            f"{fmt_ratio(s_time, v_time):>8} {fmt_mb(s_rss):>9} {fmt_mb(v_rss):>9} "
            f"{fmt_ratio(s_rss, v_rss):>6} {delta:>8}{err}"
        )

    # Per-phase SAF breakdown for the slow targets
    print()
    print("SAF per-phase breakdown:")
    phase_keys = ["frontend", "cg_refine", "andersen", "mssa_svfg", "fs_pta"]
    hdr2 = f"{'Program':<20}" + "".join(f"{k:>11}" for k in phase_keys) + f"{'total':>9}"
    print(hdr2)
    print("─" * len(hdr2))
    for label in sorted(saf.keys()):
        s = saf[label]
        if s.get("error"):
            continue
        row = f"{label:<20}"
        for k in phase_keys:
            row += f"{fmt_secs(s['phases'].get(k)):>11}"
        row += f"{fmt_secs(s.get('total_secs')):>9}"
        print(row)


if __name__ == "__main__":
    main()
