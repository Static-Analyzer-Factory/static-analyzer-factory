#!/usr/bin/env bash
# Move the 55,690-task re-run onto a user systemd unit so it is properly
# detachable: survives logout (Linger=yes is already set), is manageable with
# `systemctl --user {status,stop} saf-rerun`, and logs to journald.
#
# The previous launch was `nohup ... &` inside an ssh command. It DID survive the
# session being killed, but its wrapper was orphaned, so there was no completion
# marker and no way to stop it cleanly.
#
# Kills are BY PID, never `pkill -f` -- a pattern match on the command line also
# matches the ssh process carrying that same text, which killed my own session.
set -uo pipefail

echo "=== tearing down the orphaned run ==="
# The container owns the python worker; removing it stops the work.
docker ps --filter "name=static-analyzer-factory-dev-run" --format '{{.ID}}' \
  | while read -r id; do echo "  docker rm -f $id"; docker rm -f "$id" >/dev/null; done

# Now the compose clients, by PID, excluding anything that is an ssh carrier.
for pid in $(ps -eo pid,cmd --no-headers \
             | grep 'docker.*compose run' \
             | grep -v tailscaled | grep -v ' grep ' \
             | awk '{print $1}'); do
    echo "  kill $pid"
    kill "$pid" 2>/dev/null || true
done
sleep 3

left=$(ps -eo pid,cmd --no-headers | grep -c 'svcomp_split_eval.py --manifest splits/train' || true)
echo "  remaining eval processes: $left"

cd ~/static-analyzer-factory
rm -f saf-alone-*.jsonl.partial saf-alone-*.jsonl saf-alone-*.json
echo "  cleared partial artifacts"

echo
echo "=== relaunching under systemd --user ==="
systemctl --user reset-failed saf-rerun 2>/dev/null || true
systemd-run --user \
    --unit=saf-rerun \
    --description="SAF SV-COMP 55,690-task re-run (no bundled participants)" \
    --working-directory="$HOME/static-analyzer-factory" \
    --property=Restart=no \
    --collect \
    bash scripts/m5_full_rerun.sh

sleep 20
echo
echo "=== unit status ==="
systemctl --user status saf-rerun --no-pager -n 12 2>&1 | head -25
echo
echo "manage with:"
echo "  systemctl --user status saf-rerun"
echo "  journalctl --user -u saf-rerun -f"
echo "  systemctl --user stop saf-rerun"
