# SAF harness + validator reaper (load-runaway guard)

A systemd-user timer that kills leaked SAF child processes before they pile up and
drive the box into a load runaway. Two incidents motivated it:

1. **Sanitizer replay harnesses** (`saf_*_harness`, `saf verify`, `saf_captest`) that
   hang past their timeout and orphan (the inner timeout killed only the direct child).
2. **CPAchecker-family witness validators** (`org.sosy_lab.cpachecker.cmdline.CPAMain`,
   which also backs `cpa-witness2test`), spawned by `scripts/validate_witness.sh`. Their
   internal `--timelimit` is not hard-enforced by the JVM, so a wedged validation orphans
   to init and pegs a core for hours.

The durable source-side fix for (2) is the hard `timeout -k` wrappers in
`scripts/validate_witness.sh`; this reaper is the belt-and-suspenders backstop that also
covers (1) and anything future that leaks under these process names.

## Files
- `saf-harness-reaper.sh` — the reaper. Kills matching processes older than
  `SAF_REAPER_MAX_AGE` (default 300s) via a process-group kill (leader + JVM children).
- `saf-harness-reaper.service` — oneshot that runs the script.
- `saf-harness-reaper.timer` — fires every 2 minutes.
- `install.sh` — idempotent installer into `~/bin` + `~/.config/systemd/user`.

## Install / refresh

    bash ops/reaper/install.sh

Re-run after editing any file here to push the change to systemd-user.
