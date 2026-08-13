#!/usr/bin/env python3
"""R8 Slice-0 de-risk: THROWAWAY VM-only patch that wires a temporary
`no_data_race_strategy` arm into `saf verify` so the REAL pipeline can be
measured (recall + the -32 audit) before any production R8 code is written.

This edits ONLY the VM working tree; it is reverted by the next `crates/` rsync
from the laptop (the git source of truth). Mirrors `termination_strategy` exactly
but calls the existing plan-198 `reachable_spawns_threads` gate: a program with NO
thread spawn reachable from main is sequential => no data race => `true`; otherwise
abstain (`unknown`). Never emits `false`.

Idempotent; verifies both insertions landed.
"""
import sys

PATH = "crates/saf-cli/src/commands.rs"

ARM_ANCHOR = "        saf_svcomp::Property::Termination => Some(termination_strategy),\n        _ => None,\n"
ARM_NEW = ("        saf_svcomp::Property::Termination => Some(termination_strategy),\n"
           "        saf_svcomp::Property::NoDataRace => Some(no_data_race_strategy),\n"
           "        _ => None,\n")

FN_ANCHOR = (
    "fn termination_strategy(ctx: &VerifyCtx) -> VerdictOutcome {\n"
    "    if saf_svcomp::program_structurally_terminates(ctx.module) {\n"
    "        VerdictOutcome {\n"
    "            verdict: saf_svcomp::termination_verdict().to_string(),\n"
    "            witness: None,\n"
    "        }\n"
    "    } else {\n"
    "        unknown_outcome()\n"
    "    }\n"
    "}\n"
)
FN_NEW = FN_ANCHOR + (
    "\n"
    "/// THROWAWAY (R8 Slice-0 de-risk): no-data-race no-threading TRUE.\n"
    "/// A program with NO thread spawn reachable from main is sequential => no data\n"
    "/// race => `true`; otherwise abstain. Never emits `false`. Reuses plan-198 gate.\n"
    "fn no_data_race_strategy(ctx: &VerifyCtx) -> VerdictOutcome {\n"
    "    let callgraph = saf_analysis::callgraph::CallGraph::build(ctx.module);\n"
    "    if saf_svcomp::fast_paths::reachable_spawns_threads(ctx.module, &callgraph) {\n"
    "        unknown_outcome()\n"
    "    } else {\n"
    "        VerdictOutcome {\n"
    "            verdict: \"true\".to_string(),\n"
    "            witness: None,\n"
    "        }\n"
    "    }\n"
    "}\n"
)


def main():
    src = open(PATH, "r").read()
    if "no_data_race_strategy" in src:
        print("already patched (idempotent) — no change")
        return
    if ARM_ANCHOR not in src:
        sys.exit("FATAL: strategy_for arm anchor not found")
    if FN_ANCHOR not in src:
        sys.exit("FATAL: termination_strategy fn anchor not found")
    src = src.replace(ARM_ANCHOR, ARM_NEW, 1)
    src = src.replace(FN_ANCHOR, FN_NEW, 1)
    open(PATH, "w").write(src)
    ok = ("no_data_race_strategy" in src
          and "NoDataRace => Some(no_data_race_strategy)" in src)
    print("patched OK" if ok else "PATCH VERIFY FAILED")
    if not ok:
        sys.exit(1)


if __name__ == "__main__":
    main()
