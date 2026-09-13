#!/usr/bin/env python3
"""Movement 5, part 2: retire the CBMC module.

`crates/saf-svcomp/src/cbmc.rs` existed to drive the CBMC oracle. With the lever gone
(part 1) only two of its items are still used -- `NondetSite` and `nondet_line_map`,
which parse `__VERIFIER_nondet_*` reads out of C source and have nothing to do with
CBMC. Keep those, drop the rest, and rename the module to `nondet` so no file in the
tree is named after a competitor.
"""
import re
import sys
from pathlib import Path

OLD = Path("crates/saf-svcomp/src/cbmc.rs")
NEW = Path("crates/saf-svcomp/src/nondet.rs")
LIB = Path("crates/saf-svcomp/src/lib.rs")
PORTFOLIO = Path("crates/saf-svcomp/src/portfolio.rs")

DELETE = ["DEFAULT_UNWIND", "cbmc_precheck", "references_unsupported_nondet",
          "parse_cbmc_trace", "parse_state_line", "parse_assignment_line",
          "parse_trace_value"]

NEW_DOC = '''//! Source-level `__VERIFIER_nondet_*` read recognition.
//!
//! This module is **pure** (no I/O, no subprocess): it maps 1-based source lines to
//! the scalar-integer nondet read on that line ([`nondet_line_map`] / [`NondetSite`]),
//! which the replay driver uses to pin concrete inputs to the right call sites.
//!
//! It was once `cbmc.rs` and also held a pre-filter and trace parser for the CBMC
//! oracle. SAF no longer ships or invokes CBMC -- CBMC is itself an SV-COMP
//! participant, and SAF bundles no competitor (user decision 2026-09-13) -- so those
//! items are gone and only the source-scanning half, which was never CBMC-specific,
//! remains.'''


def span(lines, name):
    """(start, end) 0-based inclusive for a top-level item, docs and attrs included."""
    pat = re.compile(r"^(?:pub )?(?:fn|const|struct) " + re.escape(name) + r"\b")
    for i, l in enumerate(lines):
        if pat.match(l):
            s = i
            while s > 0 and (lines[s - 1].lstrip().startswith("///")
                             or lines[s - 1].lstrip().startswith("#[")):
                s -= 1
            if lines[i].rstrip().endswith(";"):          # a const: one line
                return s, i
            e = i
            while e < len(lines) - 1:
                e += 1
                if lines[e] == "}":
                    return s, e
    return None


def main():
    if NEW.exists():
        sys.exit("already applied -- refusing to run twice")
    lines = OLD.read_text().splitlines()
    before = len(lines)

    edits = []
    for name in DELETE:
        sp = span(lines, name)
        if sp is None:
            sys.exit(f"could not locate {name}")
        edits.append(sp)
        print(f"  delete  {name:32s} lines {sp[0]+1}..{sp[1]+1}")

    # The CBMC-trace tests: from their banner to just before `mod tests`'s closing brace.
    for i, l in enumerate(lines):
        if l.strip() == "// --- parse_cbmc_trace ---":
            e = len(lines) - 1
            while e > i and lines[e] != "}":            # file-final brace closes mod tests
                e -= 1
            edits.append((i, e - 1))                    # keep that brace
            print(f"  delete  parse_cbmc_trace tests        lines {i+1}..{e}")
            break
    else:
        sys.exit("could not locate the parse_cbmc_trace test banner")

    edits.sort()
    for (s1, e1), (s2, _) in zip(edits, edits[1:]):
        if e1 >= s2:
            sys.exit(f"overlapping edits {s1}..{e1} and {s2}.. -- aborting")
    for s, e in reversed(edits):
        del lines[s:e + 1]

    # Swap the module doc (the contiguous `//!` block at the top).
    end = 0
    while end < len(lines) and (lines[end].startswith("//!") or not lines[end].strip()):
        if lines[end].strip() and not lines[end].startswith("//!"):
            break
        end += 1
        if end < len(lines) and not lines[end].startswith("//!") and lines[end].strip():
            break
    lines[:end] = NEW_DOC.splitlines()

    NEW.write_text("\n".join(lines).rstrip() + "\n")
    OLD.unlink()
    print(f"  -> {NEW}: {before} -> {len(lines)} lines (cbmc.rs deleted)")

    ll = LIB.read_text()
    ll = ll.replace("pub mod cbmc;", "pub mod nondet;")
    ll = ll.replace("pub use cbmc::{NondetSite, nondet_line_map};",
                    "pub use nondet::{NondetSite, nondet_line_map};")
    LIB.write_text(ll)
    print(f"  -> {LIB}: module + re-export renamed (residual 'cbmc': {ll.count('cbmc')})")

    p = PORTFOLIO.read_text()
    p = p.replace("//! and the bit-precise CBMC oracle. Historically these ran in ONE hard-coded order",
                  "//! lever. Historically these ran in ONE hard-coded order")
    p = p.replace("//! order `[Bmc, Se, Fuzz, Cbmc]`, so the common case is a zero-behaviour-change",
                  "//! order `[Bmc, Se, Fuzz]`, so the common case is a zero-behaviour-change")
    p = p.replace("""    /// - `Bmc` / `Fuzz` / `Cbmc`: all need a fuzzable nondet input to have anything
    ///   to pin/drive (BMC's per-site gate, the fuzzer's gate and
    ///   [`crate::cbmc::cbmc_precheck`] all require it).""",
                  """    /// - `Bmc` / `Fuzz`: both need a fuzzable nondet input to have anything to
    ///   pin/drive (BMC's per-site gate and the fuzzer's gate both require it).""")
    p = re.sub(r"\n    /// `Cbmc` is deliberately NOT gated on a loop.*?latency only on tasks nothing else solved\.\n",
               "\n", p, flags=re.S)
    p = p.replace("            // whenever ANY fuzzable nondet is present. BMC/SE/CBMC are integer",
                  "            // whenever ANY fuzzable nondet is present. BMC/SE are integer")
    p = p.replace("/// The base order is the historical `[Bmc, Se, Fuzz, Cbmc]`; a non-linear nondet",
                  "/// The base order is the historical `[Bmc, Se, Fuzz]`; a non-linear nondet")
    p = p.replace("fn loop_free_prunes_se_but_keeps_cbmc()", "fn loop_free_prunes_se()")
    p = p.replace("""        // SE needs a CFG cycle -> dropped. CBMC is a whole-program bit-precise""",
                  """        // SE needs a CFG cycle -> dropped. The remaining levers are""")
    p = p.replace("        // Fuzz promoted, then BMC; SE pruned (loop-free), CBMC retained last.",
                  "        // Fuzz promoted, then BMC; SE pruned (loop-free).")
    PORTFOLIO.write_text(p)
    print(f"  -> {PORTFOLIO}: docs updated (residual 'bmc'/'Cbmc' case-insensitive: "
          f"{len(re.findall('cbmc', p, re.I))})")


if __name__ == "__main__":
    main()
