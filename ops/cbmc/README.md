# CBMC provisioning for the eval's witness-confirmation panel

`scripts/validate_witness.sh` Stage 4 uses **CBMC** (an official SV-COMP validator)
to confirm violation witnesses that CPAchecker + cpa-witness2test miss:

- **valid-memsafety**: Juliet CWE121 stack/heap overflows that CPAchecker's SMG
  analysis is blind to (confirmed via `--bounds-check --pointer-check`, credited
  only on a genuine memory-safety FAILURE at the witness's target line).
- **unreach-call**: `reach_error()` reachability (confirmed via `--unwind`, credited
  only on a real `reach_error` FAILURE, excluding inconclusive unwinding assertions).

SV-COMP confirms a witness if **any** panel validator agrees, so adding CBMC recovers
real, competition-predictive points without weakening soundness — the confirmer only
runs on SAF's FALSE verdicts (SAF abstains on safe tasks), and each property has a
violation-class + location/target filter that rejects CBMC's whole-program false
positives and modeling artifacts.

## Provisioning

CBMC is staged into `.svtools/cbmc/` (gitignored; bind-mounted into the dev container
at `/workspace/.svtools/cbmc`, which `$SAF_CBMC` points to). The host and dev image are
both Ubuntu 24.04, so the host `cbmc` binary runs in-container as-is with
`LD_LIBRARY_PATH`.

Run once per VM (e.g. after a rebuild):

    bash ops/cbmc/install.sh

The stage degrades gracefully (skips CBMC, no error) if `.svtools/cbmc/cbmc` is absent,
and `SAF_SKIP_CBMC=1` disables it entirely.
