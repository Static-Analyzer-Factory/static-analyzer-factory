#!/usr/bin/env bash
# report.sh — human dashboard for the SAF loop (plan 205 §4). Read-only + DERIVED: every number is
# recomputed from the .loop-state/arms.jsonl spine (+ heldout/checkpoint JSONs) at run time, so it can
# never drift from the recorded history. Safe to run anytime. Presentation lives here; the pure views
# live in lib/report_view.py. Python is a hard loop dependency, so formatting uses it (not jq/awk).
set -uo pipefail
LOOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SAF_REPO_ROOT:-$(cd "$LOOP_DIR/../.." && pwd)}"
STATE_DIR="${SAF_LOOP_STATE:-$REPO_ROOT/.loop-state}"
LIB="$LOOP_DIR/lib"
ARMS="$STATE_DIR/arms.jsonl"

echo "== SAF loop report =="
echo "repo:   $REPO_ROOT   branch: $(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo ?)"
echo "state:  $STATE_DIR"
if [ -f "$STATE_DIR/baseline.json" ]; then
  python3 - "$STATE_DIR/baseline.json" <<'PY' 2>/dev/null || true
import json,sys; d=json.load(open(sys.argv[1]))
print(f"baseline TRAIN: confirmed={d.get('confirmed_score')} raw={d.get('raw_score','?')} "
      f"FP={d.get('false_alarms')} wrongTRUE={d.get('wrong_true')}")
PY
fi

# ---------------------------------------------------------------- SOUNDNESS (kept-arm anomaly)
# A KEPT arm must never carry a false alarm (-16) or a wrong-TRUE (-32). The gate reverts such arms,
# so this is a belt-and-suspenders check over the recorded history. NOTE: the real arms.jsonl keys are
# false_alarms_after / wrong_true_after (NOT false_alarms / wrong_true).
echo
python3 - "$ARMS" <<'PYSND' 2>/dev/null || true
import json, sys
try:
    rows = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
except OSError:
    rows = []
bad = [r for r in rows if str(r.get("decision", "")).startswith("KEEP")
       and ((r.get("false_alarms_after") or 0) > 0 or (r.get("wrong_true_after") or 0) > 0)]
if bad:
    print("-- SOUNDNESS: *** KEPT ARM WITH FALSE ALARM / WRONG-TRUE -- INVARIANT VIOLATION ***")
    for r in bad:
        print(f"   !!! arm {r.get('arm')} {r.get('decision')} lever={r.get('lever')} "
              f"FP_after={r.get('false_alarms_after')} wrongTRUE_after={r.get('wrong_true_after')}")
else:
    nfp = sum(1 for r in rows if (r.get("false_alarms_after") or 0) > 0 or (r.get("wrong_true_after") or 0) > 0)
    print(f"-- SOUNDNESS: OK -- no KEPT arm has a false alarm or wrong-TRUE "
          f"({nfp} reverted arm(s) had FP/wT, correctly discarded by the gate)")
PYSND

# ---------------------------------------------------------------- GENERALIZATION (the headline)
echo
echo "-- GENERALIZATION (train vs holdout) ----------------------------  [the headline]"
python3 - "$STATE_DIR" "$LIB" <<'PY' 2>/dev/null || echo "  (no checkpoints yet)"
import glob, json, os, sys
state, lib = sys.argv[1], sys.argv[2]
sys.path.insert(0, lib)
import report_view as rv

def recall(d):
    out={}
    for k,v in (d.get("per_property") or {}).items():
        v=v or {}
        ft=v.get("false_total",0); out[k]=(v.get("confirmed_false",0), ft)
    return out

def latest(pat, key):
    fs=glob.glob(os.path.join(state,pat))
    return max(fs, key=key) if fs else None

def load(p):
    try: return json.load(open(p))
    except Exception: return {}

train=load(os.path.join(state,"overall_checkpoint.json"))
hf=latest("heldout-*.json", lambda p:int(os.path.basename(p).split("-")[-1].split(".")[0]) if os.path.basename(p).split("-")[-1].split(".")[0].isdigit() else 0)
hold=load(hf) if hf else {}
tr, hr = recall(train), recall(hold)
def fmt(r): return "  ".join(f"{k}={cf}/{ft}" for k,(cf,ft) in sorted(r.items()) if ft) or "(none)"
if train: print(f"  train   confirmed={train.get('confirmed_score')}   recall: {fmt(tr)}")
else:     print("  train   (no overall_checkpoint.json yet)")
if hold:  print(f"  holdout confirmed={hold.get('confirmed_score')}   recall: {fmt(hr)}")
else:     print("  holdout (no heldout-*.json yet)")
# per-property recall-fraction gap (size-normalized, directly comparable)
gap=[]
for k in sorted(set(tr)|set(hr)):
    tcf,tft=tr.get(k,(0,0)); hcf,hft=hr.get(k,(0,0))
    tf=(tcf/tft) if tft else 0.0; hff=(hcf/hft) if hft else 0.0
    gap.append(f"{k} Δ={tf-hff:+.2f}")
if gap: print("  gap (recall frac, train−holdout): " + "  ".join(gap) + "   <- widening = memorizing")
trend=rv.holdout_trend(state)
if trend: print("  holdout trend: " + " -> ".join(str(t[1]) for t in trend) + "   <- flat under rising train = overfit")
PY

# ---------------------------------------------------------------- LEVER ROI
echo
echo "-- LEVER ROI ----------------------------------------------------"
if [ -s "$ARMS" ]; then
  python3 - "$STATE_DIR" "$LIB" <<'PY' 2>/dev/null || true
import os, sys
state, lib = sys.argv[1], sys.argv[2]; sys.path.insert(0, lib)
import report_view as rv
roi=rv.lever_roi(rv.load_arms(os.path.join(state,"arms.jsonl")))
print(f"  {'lever':<22}{'arms':>5}{'KEEP':>5}{'ACC':>4}{'REV':>4}{'REJ':>4}{'netΔ':>7}{'cost$':>8}{'$/pt':>7}")
for k,s in roi.items():
    dpp = f"{s['dollars_per_point']:.2f}" if s['dollars_per_point'] else "-"
    print(f"  {k[:22]:<22}{s['arms']:>5}{s['KEEP']:>5}{s['ACCUMULATE']:>4}{s['REVERT']:>4}{s['REJECT']:>4}"
          f"{s['netDelta']:>+7}{s['cost']:>8.2f}{dpp:>7}")
PY
else
  echo "  (no arms recorded yet — arms.jsonl empty)"
fi

# ---------------------------------------------------------------- WASTE / RE-DERIVATION
echo
echo "-- WASTE / RE-DERIVATION ----------------------------------------"
if [ -s "$ARMS" ]; then
  python3 - "$STATE_DIR" "$LIB" <<'PY' 2>/dev/null || true
import os, sys
state, lib = sys.argv[1], sys.argv[2]; sys.path.insert(0, lib)
import report_view as rv
w=rv.waste(rv.load_arms(os.path.join(state,"arms.jsonl")))
print("  stuck levers (>=3 REVERTs, net<=0): " + (", ".join(w["stuck_levers"]) or "none"))
print("  arms that hit max_turns (under-budgeted): " + (", ".join("#%s"%a for a in w["max_turns_arms"]) or "none"))
print("  big reverts (wasted worker effort): " + (", ".join("#%s(%sf,+%s)"%(a,f,i) for a,f,i in w["big_reverts"]) or "none"))
print("  re-derived (same paths, >1 arm/lever): " + (", ".join("%s:%s"%(k,v) for k,v in w["rederived"].items()) or "none"))
PY
else
  echo "  (no arms recorded yet)"
fi

# ---------------------------------------------------------------- CAPABILITY MILESTONES
echo
echo "-- CAPABILITY MILESTONES (mode=capability lineage) --------------"
if [ -s "$ARMS" ]; then
  python3 - "$STATE_DIR" "$LIB" <<'PY' 2>/dev/null || true
import os, sys
state, lib = sys.argv[1], sys.argv[2]; sys.path.insert(0, lib)
import report_view as rv
caps=rv.capability_milestones(rv.load_arms(os.path.join(state,"arms.jsonl")))
if not caps: print("  (none)")
for k,s in caps.items():
    print(f"  {k}: {s['arms']} arms, {s['accumulate_arms']} accumulated, {s['keeps']} kept, "
          f"+{s['loc_added']} LOC  \"{(s['last_summary'] or '')[:60]}\"")
PY
else
  echo "  (none)"
fi

# ---------------------------------------------------------------- RECENT ARMS
echo
echo "-- RECENT ARMS (last 12) ----------------------------------------"
if [ -s "$ARMS" ]; then
  python3 - "$STATE_DIR" "$LIB" <<'PY' 2>/dev/null || true
import os, sys
state, lib = sys.argv[1], sys.argv[2]; sys.path.insert(0, lib)
import report_view as rv
for r in rv.recent(rv.load_arms(os.path.join(state,"arms.jsonl")), 12):
    w=r.get("worker") or {}
    moved={k:v for k,v in (r.get("per_property_delta") or {}).items() if v}
    mv=" ".join(f"{k}{v:+d}" for k,v in sorted(moved.items()))
    rr=r.get("revert_reason"); rr=f" ({rr})" if rr else ""
    fl=r.get("flips") or {}; g=len(fl.get("gained_confirmed",[])); l=len(fl.get("lost_confirmed",[]))
    flip=f" flips+{g}-{l}" if (g or l) else ""
    cost=w.get("total_cost_usd"); cost=f"${cost:.1f}" if isinstance(cost,(int,float)) else "$?"
    mt="(max!)" if w.get("hit_max_turns") else ""
    print(f"  #{r.get('arm')} {r.get('decision','?'):<15}{rr} {str(r.get('lever'))[:20]:<20} "
          f"Δ{r.get('confirmed_delta'):+} [{mv}]{flip} turns={w.get('num_turns')}{mt} {cost} "
          f"\"{(r.get('worker_summary') or '')[:44]}\"")
PY
else
  echo "  (no arms recorded yet)"
fi

# ---------------------------------------------------------------- kept branches + ALERTS
echo
echo "-- kept branches (auto/loop-*, cap-*) ---------------------------"
git -C "$REPO_ROOT" for-each-ref --format='  %(refname:short)  %(objectname:short)  %(contents:subject)' \
  'refs/heads/auto/loop-*' 'refs/heads/cap-*' 2>/dev/null || true

echo
echo "-- journal (last 20) --------------------------------------------"
tail -20 "$STATE_DIR/journal.md" 2>/dev/null || tail -20 "$LOOP_DIR/progress-journal.md" 2>/dev/null || true

for a in ALERT_REJECT_TAMPER ALERT_REJECT_HOLDOUT ALERT_OVERALL_REGRESSION; do
  [ -e "$STATE_DIR/$a" ] && { echo; echo "!! ALERT: $a present ($STATE_DIR/$a) — investigate"; }
done
exit 0   # this read-only dashboard always succeeds; the ALERT loop's last `[ -e ] && {}` must not leak a non-zero exit
