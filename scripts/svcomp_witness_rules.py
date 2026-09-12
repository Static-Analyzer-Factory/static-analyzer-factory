#!/usr/bin/env python3
"""The SV-COMP **2027** witness-requirement rule, as data.

Scoring a verdict needs one question answered: *does this result have to be
accompanied by a confirmed witness to earn points, and in which format version?*
SV-COMP answers it per **base category** `C.<property>.<suffix>` — not per
property — so the same `termination` verdict can be free in one edition and
witness-gated in the next, and the same `unreach-call` verdict can be free in
`C.unreach-call.Heap` and gated in `C.unreach-call.Loops`.

Until 2026 the harness encoded the rule as two flat per-property sets. That was
correct for 2026 and is wrong for 2027, most expensively for `C.termination.*`,
which moved from "2.1 (demo mode)" (= free) to "2.1 or higher" (= required).

Two independent inputs, deliberately kept separate because they have different
authorities and different update cadences:

1. **WHICH base category a task is in** — §1, `BASE_CATEGORY_SETS`. Derived from
   the SV-COMP 2027 benchmark definitions. This is a *structural* question about
   how the competition groups tasks, so the bench-defs are the right source.
2. **WHAT that base category requires** — §2, `TRUE_WITNESS_RULE` /
   `FALSE_WITNESS_RULE`. Taken ONLY from the published rules page.

   ⚠️  Do **not** re-derive §2 from the bench-defs or from a validator's
   `<rundefinition>`s. That method gives the WRONG answer for `no-data-race`:
   several validators declare a `SV-COMP27_no-data-race` *correctness*
   rundefinition, yet the rules table says "not supported", so it is
   verdict-only on the TRUE side. The rules page is the authority.

Sources, both read 2026-09-12:
  §1  https://gitlab.com/sosy-lab/sv-comp/bench-defs
      `benchmark-defs/category-structure.yml` + `benchmark-defs/cpachecker.xml`
      (`competition: SV-COMP / year: 2027 / edition: 16th`)
  §2  https://sv-comp.sosy-lab.org/2027/rules.php
"""
from __future__ import annotations

import glob
import os
from pathlib import Path

# --------------------------------------------------------------------------- §1
# Base category -> the `c/*.set` files that compose it, for SV-COMP 2027.
#
# The suffix is NOT simply the `.set` basename. Three shapes appear:
#   * 1:1            `C.unreach-call.Arrays`  <- Arrays.set
#   * union          `C.unreach-call.Heap`    <- Heap.set + LinkedLists.set
#                    (there is no `C.unreach-call.LinkedLists`)
#   * catch-all      `C.no-overflow.Main`, `C.termination.Other`,
#                    `C.valid-memsafety.Other`, `C.unreach-call.SoftwareSystems-Other`
# and the SAME `.set` maps to DIFFERENT suffixes per property: `Heap.set` is
# `unreach-call.Heap`, `no-overflow.Main`, `termination.Other`,
# `valid-memsafety.Heap` and `valid-memcleanup.Main`. So the key is
# (property, set-name), never the set name alone.
#
# A `.set` absent from every entry below is **out of competition** for that
# property — SV-COMP never runs it. `Unused_Juliet`,
# `Unused_DeviceDriversLinux64Regression`, `Hardness` (non-unreach properties),
# `Sanity` (non-unreach) and the `/todo` directories are all in this bucket.
BASE_CATEGORY_SETS: dict[str, dict[str, frozenset[str]]] = {
    "unreach-call": {
        "Arrays": frozenset({"Arrays"}),
        "BitVectors": frozenset({"BitVectors"}),
        "Combinations": frozenset({"Combinations"}),
        "Concurrency": frozenset({"Concurrency"}),
        "ControlFlow": frozenset({"ControlFlow", "Sanity"}),
        "ECA": frozenset({"ECA"}),
        "Floats": frozenset({"Floats"}),
        "Hardness": frozenset({"Hardness"}),
        "Hardware": frozenset({"Hardware"}),
        "Heap": frozenset({"Heap", "LinkedLists"}),
        "Huawei-Concurrency-Challenges": frozenset({"Huawei-Concurrency-Challenges"}),
        "Loops": frozenset({"Loops", "VerifyThis-Loops"}),
        "ProductLines": frozenset({"ProductLines"}),
        "Recursive": frozenset({"Recursive", "VerifyThis-Recursive"}),
        "Sequentialized": frozenset({"Sequentialized"}),
        "SoftwareSystems-AWS-C-Common": frozenset({"SoftwareSystems-AWS-C-Common"}),
        "SoftwareSystems-DeviceDriversLinux64": frozenset({"SoftwareSystems-DeviceDriversLinux64"}),
        "SoftwareSystems-DeviceDriversLinux64Large": frozenset({"SoftwareSystems-DeviceDriversLinux64Large"}),
        "SoftwareSystems-Intel-TDX-Module": frozenset({"SoftwareSystems-Intel-TDX-Module"}),
        "SoftwareSystems-Other": frozenset({"SoftwareSystems-BusyBox", "SoftwareSystems-OpenBSD",
                                            "SoftwareSystems-coreutils"}),
        "SoftwareSystems-uthash": frozenset({"SoftwareSystems-uthash"}),
        "XCSP": frozenset({"XCSP"}),
    },
    "no-overflow": {
        "Concurrency": frozenset({"Concurrency"}),
        "Huawei-Concurrency-Challenges": frozenset({"Huawei-Concurrency-Challenges"}),
        "Juliet": frozenset({"Juliet"}),
        "Main": frozenset({"Arrays", "BitVectors", "BitVectors-Termination", "ControlFlow",
                           "ControlFlow-Termination", "ECA", "Floats", "Heap", "Heap-Termination",
                           "LinkedLists", "Loops", "Recursive", "Sequentialized",
                           "SoftwareSystems-AWS-C-Common", "SoftwareSystems-DeviceDriversLinux64",
                           "VerifyThis-Loops", "VerifyThis-Recursive", "XCSP"}),
        "SoftwareSystems-BusyBox": frozenset({"SoftwareSystems-BusyBox"}),
        "SoftwareSystems-coreutils": frozenset({"SoftwareSystems-coreutils"}),
        "SoftwareSystems-uthash": frozenset({"SoftwareSystems-uthash"}),
    },
    "termination": {
        "BitVectors": frozenset({"BitVectors-Termination"}),
        "MainControlFlow": frozenset({"ControlFlow-Termination"}),
        "MainHeap": frozenset({"Heap-Termination"}),
        "Other": frozenset({"Arrays", "BitVectors", "ControlFlow", "ECA", "Floats", "Heap",
                            "Loops", "ProductLines", "Recursive", "Sequentialized"}),
        "SoftwareSystems-DeviceDriversLinux64": frozenset({"SoftwareSystems-DeviceDriversLinux64"}),
        "SoftwareSystems-uthash": frozenset({"SoftwareSystems-uthash"}),
    },
    "valid-memsafety": {
        "Arrays": frozenset({"Arrays", "Heap-Termination", "VerifyThis-Loops", "VerifyThis-Recursive"}),
        "Concurrency": frozenset({"Concurrency"}),
        "Heap": frozenset({"Heap"}),
        "Huawei-Concurrency-Challenges": frozenset({"Huawei-Concurrency-Challenges"}),
        "Juliet": frozenset({"Juliet"}),
        "LinkedLists": frozenset({"LinkedLists"}),
        "Other": frozenset({"ControlFlow", "ControlFlow-Termination", "Loops", "Recursive"}),
        "SoftwareSystems-DeviceDriversLinux64": frozenset({"SoftwareSystems-DeviceDriversLinux64"}),
        "SoftwareSystems-Other": frozenset({"SoftwareSystems-BusyBox", "SoftwareSystems-OpenBSD"}),
        "SoftwareSystems-coreutils": frozenset({"SoftwareSystems-coreutils"}),
        "SoftwareSystems-uthash": frozenset({"SoftwareSystems-uthash"}),
    },
    "valid-memcleanup": {
        "Main": frozenset({"Heap", "Juliet", "LinkedLists", "VerifyThis-Loops", "VerifyThis-Recursive"}),
        "SoftwareSystems-uthash": frozenset({"SoftwareSystems-uthash"}),
    },
    "no-data-race": {
        "Concurrency": frozenset({"Concurrency"}),
        "Huawei-Concurrency-Challenges": frozenset({"Huawei-Concurrency-Challenges"}),
    },
}

# `demo_categories` from category-structure.yml: scores here do NOT count toward
# the Overall score. Orthogonal to the "(demo mode)" WITNESS column in §2 — one
# is about the score, the other about the witness — but both happen to apply to
# the Huawei family, so keep them visibly distinct.
DEMO_CATEGORIES: frozenset[str] = frozenset({
    "C.Huawei-Concurrency-Challenges",
    "C.no-data-race.Huawei-Concurrency-Challenges",
    "C.no-overflow.Huawei-Concurrency-Challenges",
    "C.unreach-call.Huawei-Concurrency-Challenges",
    "C.valid-memsafety.Huawei-Concurrency-Challenges",
})

# --------------------------------------------------------------------------- §2
# https://sv-comp.sosy-lab.org/2027/rules.php, read 2026-09-12. Verbatim preamble:
#
#   "We currently support versions 2.0, 2.1, and 2.2 of this format. For each
#    base category, the following table specifies which format versions are
#    supported for violation and correctness witnesses [...]. If the table says
#    'not supported', then witnesses are not required (even not expected). If
#    the table says '(demo mode)', then witnesses are also not required, but if
#    they are produced, available validators will be executed to analyze them
#    (without affecting the score). In all other cases, a witness in a supported
#    format has to accompany every verification result."
#
# Encoding: value is the MINIMUM format version required, or `None` when the
# result scores on the verdict alone ("not supported" AND "(demo mode)" both map
# to None — the preamble makes them equivalent for scoring). `None` under the
# `_DEFAULT` key means every un-listed suffix of that property is free.
_DEFAULT = "*"

TRUE_WITNESS_RULE: dict[str, dict[str, str | None]] = {
    # C.unreach-call. | Arrays, Heap | not supported | Floats | 2.0+ (demo mode)
    #                 | Concurrency | 2.1+ | Huawei* | 2.1+ (demo mode) | others | 2.0+
    "unreach-call": {"Arrays": None, "Heap": None, "Floats": None,
                     "Concurrency": "2.1", "Huawei-Concurrency-Challenges": None,
                     _DEFAULT: "2.0"},
    # C.no-overflow. | Concurrency | 2.1+ | Huawei* | 2.1+ (demo mode) | others | 2.0+
    "no-overflow": {"Concurrency": "2.1", "Huawei-Concurrency-Challenges": None,
                    _DEFAULT: "2.0"},
    # C.termination. | all | 2.1 or higher      <-- NEW in 2027 (2026 was "2.1 (demo mode)")
    "termination": {_DEFAULT: "2.1"},
    # C.valid-memsafety. | all suffixes | not supported
    "valid-memsafety": {_DEFAULT: None},
    # C.valid-memcleanup. | all | not supported
    "valid-memcleanup": {_DEFAULT: None},
    # C.no-data-race. | all | not supported
    "no-data-race": {_DEFAULT: None},
}

FALSE_WITNESS_RULE: dict[str, dict[str, str | None]] = {
    # C.unreach-call. | Concurrency | 2.2 | Huawei* | 2.2 (demo mode) | others | 2.0+
    "unreach-call": {"Concurrency": "2.2", "Huawei-Concurrency-Challenges": None,
                     _DEFAULT: "2.0"},
    # C.no-overflow. | Concurrency | 2.2 | Huawei* | 2.2 (demo mode) | others | 2.0+
    "no-overflow": {"Concurrency": "2.2", "Huawei-Concurrency-Challenges": None,
                    _DEFAULT: "2.0"},
    # C.termination. | all | 2.1 or higher
    "termination": {_DEFAULT: "2.1"},
    # C.valid-memsafety. | Concurrency, Huawei* | 2.2# (demo mode) | others | 2.0+#
    "valid-memsafety": {"Concurrency": None, "Huawei-Concurrency-Challenges": None,
                        _DEFAULT: "2.0"},
    # C.valid-memcleanup. | all | not supported
    "valid-memcleanup": {_DEFAULT: None},
    # C.no-data-race. | all | 2.2
    # NOTE this REVERSES the pre-2027 harness, which exempted no-data-race FALSE on
    # the premise that "there is no agreed data-race witness format or validator".
    # Format 2.2 added exactly that support, and the 2026 page already required 1.0.
    "no-data-race": {_DEFAULT: "2.2"},
}

# Rules-page footnote `#`, verbatim: "All current format versions support only
# violation of the subproperties valid-deref and valid-free of valid-memsafety.
# If the violated subproperty is valid-memtrack, there is no supported witness
# format." No format => nothing to require => the verdict scores alone.
NO_WITNESS_FORMAT_SUBPROPERTIES: frozenset[str] = frozenset({"valid-memtrack"})

SUPPORTED_VERSIONS: tuple[str, ...] = ("2.0", "2.1", "2.2")


# --------------------------------------------------------------------------- API

def _version_key(v: str) -> tuple[int, ...]:
    try:
        return tuple(int(p) for p in v.split("."))
    except ValueError:
        return (0,)


def base_categories(prop: str, set_names) -> set[str]:
    """The 2027 base-category SUFFIXES a task falls in, given the `.set` files
    that contain it. Empty => the task is out of competition for this property:
    SV-COMP never runs it, so it has no base category and no witness rule."""
    table = BASE_CATEGORY_SETS.get(prop)
    if not table:
        return set()
    names = set(set_names)
    return {suffix for suffix, sets in table.items() if names & sets}


def _requirement(rule: dict[str, str | None], suffixes) -> tuple[bool, str | None]:
    """Aggregate one property's rule over every base category the task is in.

    CONSERVATIVE: a witness is required if ANY containing base category requires
    one, and then at the HIGHEST version any of them demands. A task really does
    sit in several base categories at once (e.g. `Concurrency` and
    `Huawei-Concurrency-Challenges`), and SV-COMP scores it once per category;
    collapsing that to a single row means choosing, and 0A's whole purpose is to
    stop over-reporting, so round the requirement UP. Measured cost on the
    current corpus: 0 points — no scoring task straddles a disagreeing pair.
    """
    # An empty suffix set means the task is OUT OF COMPETITION — SV-COMP runs no
    # base category containing it, so the rules table has nothing to say. Falling
    # back to the property's generic cell is deliberate: treating "no category" as
    # "no requirement" would hand free points to ~19.9k tasks the competition never
    # runs, which is precisely the over-reporting this module exists to end. Use
    # `base_categories(...) == set()` to drop or segregate them instead.
    keys = list(suffixes) or [_DEFAULT]
    required = [rule.get(s, rule.get(_DEFAULT)) for s in keys]
    needed = [v for v in required if v is not None]
    if not needed:
        return False, None
    return True, max(needed, key=_version_key)


def true_witness_requirement(prop: str, suffixes) -> tuple[bool, str | None]:
    """(is a confirmed CORRECTNESS witness required, minimum format version)."""
    rule = TRUE_WITNESS_RULE.get(prop)
    if rule is None:
        return False, None
    return _requirement(rule, suffixes)


def false_witness_requirement(prop: str, suffixes,
                              violated_subproperty: str | None = None) -> tuple[bool, str | None]:
    """(is a confirmed VIOLATION witness required, minimum format version)."""
    rule = FALSE_WITNESS_RULE.get(prop)
    if rule is None:
        return False, None
    if violated_subproperty in NO_WITNESS_FORMAT_SUBPROPERTIES:
        return False, None  # footnote #: no supported format exists
    return _requirement(rule, suffixes)


def version_satisfies(emitted: str | None, minimum: str | None) -> bool:
    """Does an emitted witness meet a base category's format floor?

    `emitted is None` means the harness did not record a format — every dump
    written before this field existed. Treat that as "cannot tell" and do NOT
    penalise, so old dumps re-score identically under the version-BLIND rule.
    A recorded non-YAML format (GraphML) never satisfies a YAML floor.
    """
    if minimum is None:
        return True
    if emitted is None:
        return True
    if not emitted or not emitted[0].isdigit():
        return False  # e.g. "graphml-1.0"
    return _version_key(emitted) >= _version_key(minimum)


def sniff_witness_format(path: str | None) -> str | None:
    """Classify an emitted witness file: a YAML-format version string ("2.0"),
    `"graphml-1.0"`, or None when absent/unreadable."""
    if not path or not os.path.exists(path):
        return None
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            head = f.read(8192)
    except OSError:
        return None
    if "<graphml" in head or "<graphml" in head.replace(" ", ""):
        return "graphml-1.0"
    for line in head.splitlines():
        s = line.strip().lstrip("- ").strip()
        if s.startswith("format_version:"):
            return s.split(":", 1)[1].strip().strip("'\"")
    return None


# --------------------------------------------------------------- .set membership

_MEMBERSHIP_CACHE: dict[str, dict[str, frozenset[str]]] = {}


def load_set_membership(svb: str | Path) -> dict[str, frozenset[str]]:
    """`rel_yml -> {.set basenames}` for every task reachable from a `c/*.set`.

    Keys are paths relative to `<svb>/c`, matching the `rel_yml` the manifest
    carries, so the join is direct — no `.i`/`.c` normalisation (the `.set`
    patterns glob `.yml` files, and so does the manifest).

    Expanding 37 `.set` files is ~50k globs, so it is cached per root.
    """
    root = str(Path(svb) / "c")
    hit = _MEMBERSHIP_CACHE.get(root)
    if hit is not None:
        return hit
    acc: dict[str, set[str]] = {}
    for path in sorted(glob.glob(os.path.join(root, "*.set"))):
        name = os.path.basename(path)[:-4]
        try:
            patterns = Path(path).read_text().splitlines()
        except OSError:
            continue
        for pat in patterns:
            pat = pat.strip()
            if not pat or pat.startswith("#"):
                continue
            for match in glob.glob(os.path.join(root, pat)):
                acc.setdefault(os.path.relpath(match, root), set()).add(name)
    out = {k: frozenset(v) for k, v in acc.items()}
    _MEMBERSHIP_CACHE[root] = out
    return out
