#!/usr/bin/env python3
"""Stamp the probe verdict onto plans/216 so nobody scopes work from a stale header.

plans/216 says "+8 weighted / Unblocked — depends on nothing". The §2 oracle probe it
itself mandates has now been run, and the answer is that the +8 cannot be collected in
2027. A plan whose header still advertises the points is exactly the trap plans/206 set.
"""
import sys
from pathlib import Path

P = Path("plans/216-movement4-false-witness-2.2.md")

OLD = """# Plan 216 — Movement 4: the FALSE-side witness upgrade (+8 weighted)

**Status:** DESIGNED, not started. **Unblocked** — depends on nothing."""

NEW = """# Plan 216 — Movement 4: the FALSE-side witness upgrade (+8 weighted)

> # 🛑 PROBE COMPLETE 2026-09-13 — DO NOT BUILD. The +8 is UNREACHABLE in 2027.
>
> The §2 oracle probe below was run (~20 min, no Rust written) and it says stop:
>
> 1. **`witnesslint` cannot confirm anything.** Upstream
>    `benchexec/tools/witnesslint.py::determine_result` returns `result.RESULT_DONE`,
>    never a `false(...)` status. Confirmation needs a validator reporting the SAME
>    STATUS as the verifier, so a linter is structurally incapable of it.
> 2. **The YAML (v2) violation track does not even COVER these targets.**
>    `witnesslint-validate-violation-witnesses-v2` declares no `no-data-race`
>    rundefinition at all, and zero `C.unreach-call.Concurrency` task blocks. The three
>    real violation validators (`cpachecker`, `dartagnan`, `uautomizer`) are all on the
>    v1 track and all consume `witness.graphml`.
> 3. **No in-flight 2027 branch fixes it** — all eight bench-defs branches, including
>    ones from 2026-09-01 and 2026-09-03, list only `witnesslint` on `C.Concurrency`.
>
> So a perfectly-formed 2.2 witness for either target has nowhere to be validated, and
> the emitter would produce artifacts no 2027 validator ever reads.
>
> **The corollary is worth raising with the organizers rather than engineering around:**
> SAF's EXISTING GraphML 1.0 `no-data-race` witnesses are exactly what the v1
> infrastructure validates. They score 0 only because the rules page marks GraphML
> legacy. The rules demand a format with no validator; the format with validators the
> rules forbid. **As configured, no tool can score a `no-data-race` FALSE in 2027.**
>
> Re-check before submission — if a v2 violation validator appears, or the v2
> rundefinitions are extended, this verdict flips and the plan is live again.
> Evidence and repro: memory `saf-movement4-concurrency-validator-gap`.

**Status:** PROBED 2026-09-13 → **BLOCKED on competition infrastructure, not on us.**
Was: "DESIGNED, not started. Unblocked — depends on nothing"."""


def main():
    if not P.exists():
        sys.exit(f"{P} not found")
    s = P.read_text()
    if "PROBE COMPLETE" in s:
        sys.exit("already stamped")
    if s.count(OLD) != 1:
        sys.exit(f"header not matched ({s.count(OLD)} hits)")
    P.write_text(s.replace(OLD, NEW))
    print(f"stamped {P}")


if __name__ == "__main__":
    main()
