#!/usr/bin/env python3
"""Movement 5: remove every SV-COMP-participant tool from SAF's verify path.

User decision 2026-09-13: SAF ships no tool that is itself an SV-COMP participant.
That is exactly two -- CPAchecker (`cpachecker`) and CBMC (`cbmc`, `cbmc-path`).
clang/opt stay (compilers), Z3 stays (linked library), the sanitizers stay (runtime).

Measured cost, `scripts/m5_saf_alone.py`: 137 -> 123. CPAchecker -12, CBMC -2,
`false_alarms == 0` and `wrong_true == 0` preserved in every arm.

Boundaries are re-derived by function NAME, never hardcoded, so the script survives
the file shifting under it. It refuses to run twice.
"""
import re
import sys
from pathlib import Path

CLI = Path("crates/saf-cli/src/commands.rs")
PORTFOLIO = Path("crates/saf-svcomp/src/portfolio.rs")
LIB = Path("crates/saf-svcomp/src/lib.rs")

# Functions that exist only to drive a competitor tool.
DELETE_FNS = [
    "cbmc_confirm", "cbmc_unwind", "cbmc_timeout", "resolve_cbmc", "cbmc_confirm_false",
    "cpachecker_timelimit", "resolve_cpachecker",
    "cpachecker_confirms_no_overflow", "cpachecker_confirms_unreach",
]

OVERFLOW_STUB = '''/// Rank-2 `no-overflow` TRUE: DISABLED, deliberately.
///
/// The sound-interval sentinel ([`saf_analysis::absint::prove_no_signed_overflow`])
/// PROPOSES a proof, but on its own it is not wrong-TRUE-safe: a sweep over the full
/// expected-FALSE population measured 7 wrong TRUEs from SAF's sentinels. The proposal
/// therefore required an independent confirmer, and the only one to hand was a bundled
/// `CPAchecker`.
///
/// SAF ships no tool that is itself an SV-COMP participant (user decision 2026-09-13),
/// so that confirmer is gone and nothing is left to make the proposal sound. SAF
/// ABSTAINS rather than emit an unvalidated TRUE -- a missed TRUE scores 0, a wrong one
/// scores -32. Measured price of this decision: -12 dedup-weighted.
///
/// Movement 1 (`plans/213`) fixes the two absint bug classes behind those wrong TRUEs;
/// when it lands this becomes a SAF-native proof needing no external gate.
fn try_overflow_true(_ctx: &VerifyCtx) -> Option<VerdictOutcome> {
    None
}'''

UNREACH_STUB = '''/// Rank-3 `unreach-call` TRUE: DISABLED, deliberately.
///
/// Same reasoning as [`try_overflow_true`]: the interval error-block-bottom sentinel
/// proposes, but only a bundled `CPAchecker` could confirm, and SAF ships no SV-COMP
/// participant. SAF abstains. Measured price: 3 tasks, folded into the -12 above.
///
/// Re-enabled by Movement 1 (`plans/213`) as a SAF-native proof.
fn try_unreach_true(_ctx: &VerifyCtx) -> Option<VerdictOutcome> {
    None
}'''

REPLACE_FNS = {"try_overflow_true": OVERFLOW_STUB, "try_unreach_true": UNREACH_STUB}


def span(lines, name):
    """(start, end) 0-based inclusive for `fn name`, doc comment and attrs included."""
    pat = re.compile(r"^(?:pub )?fn " + re.escape(name) + r"\b")
    for i, l in enumerate(lines):
        if pat.match(l):
            s = i
            while s > 0 and (lines[s - 1].lstrip().startswith("///")
                             or lines[s - 1].lstrip().startswith("#[")):
                s -= 1
            e = i
            while e < len(lines) - 1:
                e += 1
                if lines[e] == "}":
                    return s, e
    return None


def main():
    text = CLI.read_text()
    if "ships no tool that is itself an SV-COMP participant" in text:
        sys.exit("already applied -- refusing to run twice")

    lines = text.splitlines()

    # Collect every edit as (start, end, replacement-lines), then apply bottom-up.
    edits = []
    for name in DELETE_FNS:
        sp = span(lines, name)
        if sp is None:
            sys.exit(f"could not locate fn {name}")
        edits.append((sp[0], sp[1], []))
        print(f"  delete  {name:32s} lines {sp[0]+1}..{sp[1]+1}")

    for name, stub in REPLACE_FNS.items():
        sp = span(lines, name)
        if sp is None:
            sys.exit(f"could not locate fn {name}")
        edits.append((sp[0], sp[1], stub.splitlines()))
        print(f"  stub    {name:32s} lines {sp[0]+1}..{sp[1]+1}")

    # The lever dispatch arm.
    for i, l in enumerate(lines):
        if "Lever::Cbmc => cbmc_confirm(ctx)" in l:
            edits.append((i, i, []))
            print(f"  delete  lever dispatch arm            line  {i+1}")
            break
    else:
        sys.exit("could not locate the Lever::Cbmc dispatch arm")

    # The CPACHECKER_TIMELIMIT_SECS constant and its doc comment.
    for i, l in enumerate(lines):
        if l.startswith("const CPACHECKER_TIMELIMIT_SECS"):
            s = i
            while s > 0 and lines[s - 1].lstrip().startswith("///"):
                s -= 1
            edits.append((s, i, []))
            print(f"  delete  CPACHECKER_TIMELIMIT_SECS     lines {s+1}..{i+1}")
            break

    # The cpachecker_timelimit unit tests (a contiguous block under one banner).
    for i, l in enumerate(lines):
        if "--- CPAchecker gate budget clamp" in l:
            e = i
            while e < len(lines) - 1 and "--- overflow loop-free boundary" not in lines[e + 1]:
                e += 1
            edits.append((i, e, []))
            print(f"  delete  cpachecker_timelimit tests    lines {i+1}..{e+1}")
            break

    # Overlap guard: a bad span would silently eat unrelated code.
    edits.sort(key=lambda t: t[0])
    for (s1, e1, _), (s2, _, _) in zip(edits, edits[1:]):
        if e1 >= s2:
            sys.exit(f"overlapping edits {s1}..{e1} and {s2}.. -- aborting")

    for s, e, repl in reversed(edits):
        lines[s:e + 1] = repl

    CLI.write_text("\n".join(lines) + "\n")
    print(f"  -> {CLI}: {len(text.splitlines())} -> {len(lines)} lines")

    # --- portfolio.rs: drop the Cbmc lever ------------------------------------
    p = PORTFOLIO.read_text()
    before = len(p.splitlines())
    p = p.replace("    /// Bit-precise CBMC oracle (`crate::cbmc`).\n    Cbmc,\n", "")
    p = p.replace("[Lever::Fuzz, Lever::Bmc, Lever::Se, Lever::Cbmc]",
                  "[Lever::Fuzz, Lever::Bmc, Lever::Se]")
    p = p.replace("[Lever::Bmc, Lever::Se, Lever::Fuzz, Lever::Cbmc]",
                  "[Lever::Bmc, Lever::Se, Lever::Fuzz]")
    p = p.replace("vec![Lever::Bmc, Lever::Se, Lever::Fuzz, Lever::Cbmc]",
                  "vec![Lever::Bmc, Lever::Se, Lever::Fuzz]")
    p = p.replace("vec![Lever::Fuzz, Lever::Bmc, Lever::Se, Lever::Cbmc]",
                  "vec![Lever::Fuzz, Lever::Bmc, Lever::Se]")
    p = p.replace("assert_eq!(plan_unreach(f), vec![Lever::Bmc, Lever::Fuzz, Lever::Cbmc]);",
                  "assert_eq!(plan_unreach(f), vec![Lever::Bmc, Lever::Fuzz]);")
    p = p.replace("assert_eq!(plan_unreach(f), vec![Lever::Fuzz, Lever::Bmc, Lever::Cbmc]);",
                  "assert_eq!(plan_unreach(f), vec![Lever::Fuzz, Lever::Bmc]);")
    p = p.replace("            Lever::Bmc | Lever::Cbmc => f.fuzzable_nondet,",
                  "            Lever::Bmc => f.fuzzable_nondet,")
    PORTFOLIO.write_text(p)
    print(f"  -> {PORTFOLIO}: {before} -> {len(p.splitlines())} lines"
          f"  (remaining 'Cbmc' mentions: {p.count('Cbmc')})")

    # --- lib.rs: stop re-exporting the CBMC-only helpers -----------------------
    ll = LIB.read_text()
    ll = ll.replace(
        "pub use cbmc::{DEFAULT_UNWIND, NondetSite, cbmc_precheck, nondet_line_map, parse_cbmc_trace};",
        "pub use cbmc::{NondetSite, nondet_line_map};")
    LIB.write_text(ll)
    print(f"  -> {LIB}: re-export narrowed to the nondet helpers still in use")


if __name__ == "__main__":
    main()
