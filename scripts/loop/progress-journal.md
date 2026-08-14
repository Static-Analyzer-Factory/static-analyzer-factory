# SAF loop — progress journal (durable shift-handoff)

The supervisor appends one line per arm here (arm N | lever | mode | outcome | Δconfirmed | UTC). On a
crash/reboot the loop reconstructs state from `git log` + this journal. Do not hand-edit while running.

Baseline to beat (plan 203, svcomp25): CONFIRMED C.FalseOverall ≈ 3960 + ~396 termination; FP=0,
wrong-TRUE=0, deterministic.

## Arms
