#!/usr/bin/env python3
"""Non-overfitting train/holdout splitter for the sv-benchmarks pool (plan 203).

The sv-benchmarks task set is FULLY PUBLIC and has NO hidden test set, so a tool
can silently overfit to it. This script carves the pool into an honest
train/holdout split so recall/score reported on the *holdout* is a real
generalization estimate rather than a re-measurement of tuned-on tasks.

Two split modes (the mode is chosen automatically):

  * edition holdout  (`--holdout-tag T2`, the MOST honest estimate) — train = every
    target-property task present at `--train-tag`; holdout = tasks whose `.yml` was
    *added* between the train tag and the holdout tag (`git diff --diff-filter=A`).
    The holdout is genuinely-new benchmarks the tool could not have been tuned on.
    Requires the sv-benchmarks working tree checked out at (or after) the holdout
    tag so the added files exist on disk.

  * origin-grouped   (no `--holdout-tag`, single edition) — partition whole ORIGIN
    families (the first `--group-depth` path components under `c/`, e.g.
    `Juliet_Test/CWE190_...`) into train/holdout, stratified so each property keeps
    ~`--holdout-frac` of its tasks in holdout. Splitting by origin — NOT per task —
    is essential: tasks in one directory are near-duplicates from one generator, so
    a per-task random split leaks the test set into training.

Outputs, under `--out-dir`, for each of {train, holdout}:
  * `<split>.jsonl`             — one task/line (yml, src, property, expected, ...),
                                  consumed by scripts/svcomp_split_eval.py.
  * `<split>.<property>.set`    — BenchExec task-set files (one per property), paths
                                  relative to the set file so `<includesfile>` works.
  * `<split>.bench.xml`         — a ready-to-run BenchExec benchmark definition at
                                  competition resource limits for that split.
  * `split_manifest.json`       — the audit summary: per-property/per-category counts,
                                  achieved holdout fractions, and the holdout group list.

Determinism: the group->side assignment uses SHA-256 of `"<seed>:<group>"` (stable
across machines/runs, unlike Python's salted `hash()`), and every listing is sorted.
Same (pool, seed, frac, depth) => byte-identical split.

Run from the workspace root (host or container — pure Python, no LLVM needed):
    python3 scripts/svcomp_split.py --holdout-tag svcomp26     # edition holdout
    python3 scripts/svcomp_split.py                            # origin-grouped 80/20
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

# ---- Task enumeration (regex parse, mirrors scripts/svcomp_verify_eval_all.py) ----
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")
PROP_RE = re.compile(r"([\w-]+)\.prp")
EXP_RE = re.compile(r"expected_verdict:\s*(true|false)")
SUB_RE = re.compile(r"subproperty:\s*([\w-]+)")

# SAF's wired/scored properties (valid-memcleanup is opt-in via --properties).
DEFAULT_PROPERTIES = (
    "unreach-call",
    "valid-memsafety",
    "no-overflow",
    "termination",
    "no-data-race",
)


# ============================ pure, unit-tested core ============================
def origin_group(rel_dir: str, depth: int) -> str:
    """Group key = the first `depth` path components of a task's directory
    (relative to `c/`). Whole families share a key so they never straddle the
    split. e.g. origin_group("Juliet_Test/CWE190_x/s01", 2) -> "Juliet_Test/CWE190_x";
    origin_group("array-examples", 2) -> "array-examples" (fewer than depth comps)."""
    parts = [p for p in rel_dir.split("/") if p]
    if depth <= 0 or not parts:
        return rel_dir or "."
    return "/".join(parts[:depth])


def stable_key(name: str, seed: int) -> str:
    """Deterministic per-group ordering key — SHA-256 of "<seed>:<name>". Stable
    across processes and machines (Python's built-in hash() is salted per run)."""
    return hashlib.sha256(f"{seed}:{name}".encode()).hexdigest()


def assign_groups(
    tasks: list[dict], holdout_frac: float, seed: int
) -> dict[str, str]:
    """Assign every origin group to "train" or "holdout" (whole groups only — the
    no-leakage invariant). Stratified by property: for each property, whole groups
    whose *dominant* property is that property are moved to holdout (in stable-key
    order) until ~holdout_frac of that property's tasks are held out.

    Returns {group: "train"|"holdout"}. Guarantees: (1) every group maps to exactly
    one side; (2) no group's tasks appear on both sides."""
    groups: dict[str, list[dict]] = defaultdict(list)
    for t in tasks:
        groups[t["group"]].append(t)

    prop_total: dict[str, int] = defaultdict(int)
    for t in tasks:
        prop_total[t["property"]] += 1

    def group_prop_counts(g: str) -> dict[str, int]:
        c: dict[str, int] = defaultdict(int)
        for t in groups[g]:
            c[t["property"]] += 1
        return c

    # Dominant property per group (tie broken by property name for determinism).
    dominant: dict[str, str] = {}
    for g in groups:
        counts = group_prop_counts(g)
        dominant[g] = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))[0][0]

    side: dict[str, str] = {}
    holdout_prop: dict[str, int] = defaultdict(int)
    # Process properties in a fixed order; a group is credited to every property it
    # contains once assigned, so later buckets account for already-held-out tasks.
    for p in sorted(prop_total):
        target = holdout_frac * prop_total[p]
        cands = sorted(
            (g for g in groups if dominant[g] == p and g not in side),
            key=lambda g: stable_key(g, seed),
        )
        for g in cands:
            if holdout_prop[p] >= target:
                break
            side[g] = "holdout"
            for q, n in group_prop_counts(g).items():
                holdout_prop[q] += n

    for g in groups:
        side.setdefault(g, "train")
    return side


# ============================== enumeration / IO ===============================
def parse_task_yaml(text: str):
    """Yield (property_name, expected_bool, subproperty|None) per property block."""
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if "property_file:" not in line:
            continue
        m = PROP_RE.search(line)
        if not m:
            continue
        name = m.group(1)
        exp = None
        sub = None
        for j in range(i, min(i + 5, len(lines))):
            me = EXP_RE.search(lines[j])
            if me and exp is None:
                exp = me.group(1) == "true"
            ms = SUB_RE.search(lines[j])
            if ms and sub is None:
                sub = ms.group(1)
        yield name, exp, sub


def collect_tasks(c_dir: Path, properties: set[str], group_depth: int) -> list[dict]:
    """Walk `c/**/*.yml` and build one task record per (yml, target-property)."""
    c_root = c_dir.resolve()
    svb_root = c_root.parent
    tasks: list[dict] = []
    for yml in c_dir.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists():
            continue
        md = DM_RE.search(text)
        data_model = md.group(1) if md else "LP64"
        try:
            rel_yml = str(yml.resolve().relative_to(c_root))
        except ValueError:
            continue
        rel_dir = os.path.dirname(rel_yml)
        group = origin_group(rel_dir, group_depth)
        for name, exp, sub in parse_task_yaml(text):
            if name not in properties or exp is None:
                continue
            tasks.append(
                {
                    "yml": str(yml.resolve()),
                    "src": str(src),                              # abs (host convenience)
                    "rel_src": os.path.relpath(str(src), svb_root),  # portable: relative
                    "rel_yml": rel_yml,          # relative to c/
                    "rel_svb": f"c/{rel_yml}",   # relative to sv-benchmarks root
                    "property": name,
                    "expected": exp,
                    "subproperty": sub,
                    "data_model": data_model,
                    "group": group,
                }
            )
    return tasks


def git_added_between(svb: Path, train_tag: str, holdout_tag: str) -> set[str]:
    """Set of `.yml` paths (relative to the sv-benchmarks root, e.g. 'c/foo/bar.yml')
    that were ADDED between two tags — the genuinely-new edition-holdout tasks."""
    out = subprocess.run(
        ["git", "-C", str(svb), "diff", "--name-only", "--diff-filter=A",
         f"{train_tag}..{holdout_tag}", "--", "c"],
        capture_output=True, text=True, check=True,
    ).stdout
    return {ln.strip() for ln in out.splitlines() if ln.strip().endswith(".yml")}


def write_split(
    out_dir: Path, split: str, tasks: list[dict], properties: list[str]
) -> dict:
    """Write <split>.jsonl + per-property <split>.<prop>.set; return a count summary."""
    out_dir.mkdir(parents=True, exist_ok=True)
    tasks = sorted(tasks, key=lambda t: (t["property"], t["rel_yml"]))

    with (out_dir / f"{split}.jsonl").open("w") as f:
        for t in tasks:
            f.write(json.dumps({k: t[k] for k in (
                "yml", "src", "rel_src", "rel_yml", "property", "expected",
                "subproperty", "data_model", "group")}) + "\n")

    by_prop: dict[str, list[dict]] = defaultdict(list)
    for t in tasks:
        by_prop[t["property"]].append(t)

    props_present = []
    for prop in properties:
        rows = by_prop.get(prop, [])
        if not rows:
            continue
        props_present.append(prop)
        set_path = out_dir / f"{split}.{prop}.set"
        with set_path.open("w") as f:
            for t in rows:
                # Path relative to the .set file's own directory (BenchExec resolves
                # <includesfile> globs relative to the set file's location).
                f.write(os.path.relpath(t["yml"], out_dir) + "\n")

    return {
        "split": split,
        "total": len(tasks),
        "properties_present": props_present,
        "per_property": {p: len(by_prop.get(p, [])) for p in properties},
        "per_property_false": {
            p: sum(1 for t in by_prop.get(p, []) if not t["expected"])
            for p in properties
        },
    }


BENCH_DTD = ('<!DOCTYPE benchmark PUBLIC "+//IDN sosy-lab.org//DTD BenchExec '
             'benchmark 2.3//EN" "https://www.sosy-lab.org/benchexec/'
             'benchmark-2.3.dtd">')


def write_bench_xml(
    out_dir: Path, split: str, props_present: list[str], c_dir: Path,
    timelimit: str, memlimit: str, cpu_cores: str,
) -> None:
    """Emit a BenchExec benchmark definition for one split at competition limits.
    One rundefinition per property; tasks come from the split's per-property .set;
    saf.py appends `--witness witness.yml` and BenchExec collects it."""
    prp_dir = os.path.relpath((c_dir / "properties").resolve(), out_dir)
    rundefs = []
    for prop in props_present:
        rundefs.append(
            f'  <rundefinition name="SAF_{split}_{prop}">\n'
            f'    <tasks name="{prop}">\n'
            f'      <includesfile>{split}.{prop}.set</includesfile>\n'
            f'      <propertyfile>{prp_dir}/{prop}.prp</propertyfile>\n'
            f'    </tasks>\n'
            f'  </rundefinition>'
        )
    xml = (
        '<?xml version="1.0"?>\n'
        f'{BENCH_DTD}\n'
        '<!-- GENERATED by scripts/svcomp_split.py. Competition resource limits.\n'
        f'     Run:  scripts/run_benchexec_svcomp.sh <this-dir>/{split}.bench.xml\n'
        '     (it installs the vendored benchexec/tools/saf.py that provides '
        'tool="saf"). -->\n'
        f'<benchmark tool="saf" timelimit="{timelimit}" memlimit="{memlimit}" '
        f'cpuCores="{cpu_cores}">\n'
        '  <resultfiles>**/witness.yml</resultfiles>\n'
        '  <option name="--witness">witness.yml</option>\n\n'
        + "\n\n".join(rundefs) + "\n"
        '</benchmark>\n'
    )
    (out_dir / f"{split}.bench.xml").write_text(xml)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--svb", default="tests/benchmarks/sv-benchmarks",
                    help="sv-benchmarks repo root (default: %(default)s)")
    ap.add_argument("--out-dir", default="tests/benchmarks/svcomp-splits")
    ap.add_argument("--properties", default=",".join(DEFAULT_PROPERTIES),
                    help="comma-separated properties to include")
    ap.add_argument("--train-tag", default="svcomp25",
                    help="edition mode: the tag the tool is tuned on")
    ap.add_argument("--holdout-tag", default=None,
                    help="edition mode: hold out tasks ADDED between train-tag and "
                         "this tag. Omit for origin-grouped single-edition split.")
    ap.add_argument("--group-depth", type=int, default=2,
                    help="origin-group granularity (path comps under c/, default 2)")
    ap.add_argument("--holdout-frac", type=float, default=0.2,
                    help="grouped mode: target holdout fraction per property")
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--timelimit", default="900 s", help="BenchExec per-task CPU limit")
    ap.add_argument("--memlimit", default="15 GB")
    ap.add_argument("--cpu-cores", default="4")
    args = ap.parse_args()

    svb = Path(args.svb)
    c_dir = svb / "c"
    if not c_dir.is_dir():
        print(f"ERROR: sv-benchmarks not found at {c_dir}\n"
              f"  run: git submodule update --init {args.svb}", file=sys.stderr)
        return 1
    properties = [p.strip() for p in args.properties.split(",") if p.strip()]
    out_dir = Path(args.out_dir)

    print(f"enumerating tasks under {c_dir} for properties: {', '.join(properties)}")
    tasks = collect_tasks(c_dir, set(properties), args.group_depth)
    print(f"  {len(tasks)} (yml,property) task records; "
          f"{len({t['group'] for t in tasks})} origin groups "
          f"(depth={args.group_depth})")

    mode = "edition" if args.holdout_tag else "grouped"
    if mode == "edition":
        added = git_added_between(svb, args.train_tag, args.holdout_tag)
        print(f"edition holdout: {len(added)} .yml added between "
              f"{args.train_tag}..{args.holdout_tag}")
        holdout = [t for t in tasks if t["rel_svb"] in added]
        train = [t for t in tasks if t["rel_svb"] not in added]
        holdout_groups = sorted({t["group"] for t in holdout})
    else:
        side = assign_groups(tasks, args.holdout_frac, args.seed)
        train = [t for t in tasks if side[t["group"]] == "train"]
        holdout = [t for t in tasks if side[t["group"]] == "holdout"]
        holdout_groups = sorted(g for g, s in side.items() if s == "holdout")

    # No-straddle is a GROUPED-mode invariant (whole families to one side, so
    # near-duplicate generator tasks can't leak across the split). In EDITION mode
    # the honest boundary is git-addition: a task added in 2026 could not have been
    # tuned on regardless of whether its family has older members, so a family
    # spanning both editions is expected, not leakage.
    tg = {t["group"] for t in train}
    hg = {t["group"] for t in holdout}
    straddle = tg & hg
    if mode == "grouped" and straddle:
        print(f"FATAL: {len(straddle)} groups straddle train/holdout "
              f"(e.g. {sorted(straddle)[:3]}) — leakage!", file=sys.stderr)
        return 2
    if mode == "edition" and straddle:
        print(f"note: {len(straddle)} origin families span both editions "
              f"(older members in train, 2026-added members in holdout) — "
              f"expected for edition holdout, not leakage.")

    train_sum = write_split(out_dir, "train", train, properties)
    holdout_sum = write_split(out_dir, "holdout", holdout, properties)
    write_bench_xml(out_dir, "train", train_sum["properties_present"], c_dir,
                    args.timelimit, args.memlimit, args.cpu_cores)
    write_bench_xml(out_dir, "holdout", holdout_sum["properties_present"], c_dir,
                    args.timelimit, args.memlimit, args.cpu_cores)

    achieved = {}
    for p in properties:
        tot = train_sum["per_property"][p] + holdout_sum["per_property"][p]
        achieved[p] = (holdout_sum["per_property"][p] / tot) if tot else 0.0

    manifest = {
        "mode": mode,
        "seed": args.seed,
        "group_depth": args.group_depth,
        "holdout_frac": args.holdout_frac,
        "train_tag": args.train_tag,
        "holdout_tag": args.holdout_tag,
        "properties": properties,
        "train": train_sum,
        "holdout": holdout_sum,
        "achieved_holdout_fraction": achieved,
        "n_holdout_groups": len(holdout_groups),
        "holdout_groups": holdout_groups,
    }
    (out_dir / "split_manifest.json").write_text(json.dumps(manifest, indent=2))

    print(f"\n=== split ({mode}) written to {out_dir} ===")
    print(f"  {'property':<18} {'train':>8} {'holdout':>8} {'holdout%':>9} "
          f"{'holdoutFALSE':>13}")
    for p in properties:
        print(f"  {p:<18} {train_sum['per_property'][p]:>8} "
              f"{holdout_sum['per_property'][p]:>8} {achieved[p]*100:>8.1f}% "
              f"{holdout_sum['per_property_false'][p]:>13}")
    print(f"  holdout origin groups: {len(holdout_groups)}")
    print(f"\nnext: score each split with confirmed witnesses, e.g.\n"
          f"  python3 scripts/svcomp_split_eval.py --manifest {out_dir}/holdout.jsonl "
          f"--confirm-witness")
    return 0


if __name__ == "__main__":
    sys.exit(main())
