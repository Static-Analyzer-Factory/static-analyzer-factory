#!/usr/bin/env bash
# Install / refresh the SAF harness+validator reaper (load-runaway guard) into the
# current user's systemd-user. Idempotent — safe to re-run.
#
# The reaper kills leaked SAF sanitizer harnesses, orphaned `saf verify` drivers, and
# orphaned CPAchecker-family witness validators (org.sosy_lab.cpachecker.cmdline.CPAMain,
# which also backs cpa-witness2test) once they exceed MAX_AGE seconds — the failure mode
# that twice drove cd-vm-15 into a load runaway. See ./README.md.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mkdir -p "$HOME/bin" "$HOME/.config/systemd/user"
install -m 0755 "$HERE/saf-harness-reaper.sh"      "$HOME/bin/saf-harness-reaper.sh"
install -m 0644 "$HERE/saf-harness-reaper.service" "$HOME/.config/systemd/user/saf-harness-reaper.service"
install -m 0644 "$HERE/saf-harness-reaper.timer"   "$HOME/.config/systemd/user/saf-harness-reaper.timer"

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
systemctl --user daemon-reload
systemctl --user enable --now saf-harness-reaper.timer

echo "reaper installed; timer status:"
systemctl --user list-timers saf-harness-reaper.timer --no-pager | sed -n '1,3p'
