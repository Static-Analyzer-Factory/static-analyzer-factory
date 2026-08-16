# Loop observability design (plan 205)

Concrete instrumentation for the SAF autonomous-improvement loop (`scripts/loop/`) so operators
can diagnose **why arms succeed/fail**, **what workers actually do**, **whether work is wasted or
re-derived**, **per-property / per-lever ROI**, and above all **generalization vs memorization** —
the problem that motivated this: the loop lifts the TRAIN confirmed score while the svcomp26
HOLDOUT stays flat (7→7→8/1770).

Scope: **observability only**. No new gates, no changes to keep/revert semantics, no builds. Every
addition writes under `.loop-state/` (gitignored) — never into the immutable set and never into the
tracked tree in a way that dirties it. Additions to `scripts/loop/*.py` / `*.sh` are allowed but
re-froze the immutable manifest on deploy (operator action).

Design rule followed throughout: **the supervisor already has all the raw material** — `before.json`
and `after.json` carry the scorer's full `per_property` map; the worker `transcript.jsonl` carries a
terminal stream-json `result` event with `num_turns` / `total_cost_usd` / `duration_ms` / `subtype`
and `api_retry` events; `git` knows the diff. Today all of this is thrown away after a one-line
journal entry. We are **capturing what already exists**, not computing anything new.

---

## 0. What data already exists (and is discarded)

At the point `run_arm` finishes an arm, these files sit in `.loop-state/arm-<n>/`:

| File | Contains | Used today | Wasted signal |
|------|----------|-----------|---------------|
| `before.json` | scorer `-o`: `confirmed_score`, `raw_score`, `false_alarms`, `wrong_true`, and **`per_property`** (per family: `n, TP, FP, wrong_true, unknown, timeout, error, raw, confirmed, false_total, confirmed_false, emitted_false`) | only `confirmed_score` read for delta | the entire per-property breakdown |
| `after.json` | same, post-arm | only `confirmed_score` | same |
| `transcript.jsonl` | full stream-json: assistant turns, tool_use events, `api_retry` events, terminal `result` event (`subtype`, `num_turns`, `total_cost_usd`, `duration_ms`, `result` text) | classified to one word by `worker_status.py` | turns, cost, retries, files touched, the worker's own summary |
| `verdict.json` | `verify_arm.py` output: decision, violations, delta, progressed, regressed_families, sound flag | written, never aggregated | already structured — just not collected |
| `session_id` | the worker session uuid | resume key | — |
| `captest.log` | nextest output on the ACCUMULATE probe | debugging | pass/fail signal |

Two more artifacts the supervisor produces but does not correlate:
- `heldout-<n>.json` — the svcomp26 holdout eval (the **real** generalization signal), written every
  `HELDOUT_EVERY_K` kept arms but only ever `tail`-ed by `report.sh` as a flat list.
- `overall-<n>.json` / `overall_checkpoint.json` — full all-property TRAIN evals for the checkpoint.

And one artifact we should start requesting: the scorer's `--per-task` JSONL dump (already supported
by `svcomp_split_eval.py`, currently unused by the loop) — the ONLY way to see *which individual
tasks* an arm flipped, i.e. the difference between "recalled 3 more genuinely-solved tasks" and
"memorized 3 task shapes."

---

## 1. Per-arm structured record — `record.json` + `arms.jsonl`

**What it captures.** One machine-readable row per arm with everything needed to answer "was this arm
worth it, and did it generalize" without re-opening five files. Written to
`.loop-state/arm-<n>/record.json` (full) and appended to `.loop-state/arms.jsonl` (one flat line per
arm, the spine every downstream view reads).

**Operator questions answered.** *Which arms actually moved the score and by how much per property?
How much did each arm cost (turns, dollars, wall)? Did the worker hit max_turns / burn retries? How
big was the change (files, LOC)? What did the worker say it did?*

### 1a. Extend `worker_status.py` to also emit the `result` telemetry

`worker_status.py` already parses the transcript for classification. Add a second, side-effect-free
function that extracts the terminal `result` event's telemetry and the final assistant text. Keep the
existing `classify_transcript` untouched (it is on the hot rate-limit path and unit-tested).

```python
# scripts/loop/lib/worker_status.py  (add below classify_transcript)

def extract_result(path: str) -> dict:
    """Pull worker telemetry from the terminal stream-json `result` event + the last assistant
    message. Pure, deterministic, tolerant of a truncated/missing transcript (returns zeros/None).
    Fields mirror Claude Code's headless `result` event; `summary` is the worker's own final text
    (what it claims it did) — advisory only, NEVER trusted for scoring."""
    out = {"subtype": None, "num_turns": 0, "total_cost_usd": 0.0, "duration_ms": 0,
           "duration_api_ms": 0, "is_error": None, "api_retries": 0, "hit_max_turns": False,
           "summary": ""}
    last_assistant = ""
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    except OSError:
        return out
    for line in lines:
        line = line.strip()
        if not line:
            continue
        try:
            rec = json.loads(line)
        except (ValueError, TypeError):
            continue
        t = rec.get("type")
        if t == "system" and rec.get("subtype") == "api_retry":
            out["api_retries"] += 1
        elif t == "assistant":
            # keep the most recent assistant text block (the worker's running summary)
            for blk in (rec.get("message", {}) or {}).get("content", []) or []:
                if isinstance(blk, dict) and blk.get("type") == "text":
                    last_assistant = blk.get("text", "") or last_assistant
        elif t == "result":
            out["subtype"] = rec.get("subtype")
            out["num_turns"] = rec.get("num_turns", 0) or 0
            out["total_cost_usd"] = rec.get("total_cost_usd", 0.0) or 0.0
            out["duration_ms"] = rec.get("duration_ms", 0) or 0
            out["duration_api_ms"] = rec.get("duration_api_ms", 0) or 0
            out["is_error"] = rec.get("is_error")
            out["hit_max_turns"] = rec.get("subtype") == "error_max_turns"
            # `result` text is present on success; else fall back to last assistant block
            out["summary"] = (rec.get("result") or last_assistant or "").strip()[:1200]
    if not out["summary"]:
        out["summary"] = last_assistant.strip()[:1200]
    return out


if __name__ == "__main__":
    import sys as _s
    # backward-compatible default (classification); `--result` emits the telemetry JSON
    if len(_s.argv) > 2 and _s.argv[1] == "--result":
        print(json.dumps(extract_result(_s.argv[2])))
    else:
        print(classify_transcript(_s.argv[1]))
```

> The `--result` subcommand keeps the file single-purpose and lets `supervisor.sh` shell out the same
> way it already does for classification (`py "$LIB/worker_status.py" ...`). Add a unit test
> `test_worker_status.py` with a fixed transcript fixture asserting the extracted fields (mirrors the
> `test_verify_arm.py` style: a temp file, run the function, assert a dict).

### 1b. Add a `record.py` that assembles the row from files already on disk

A new pure module keeps `supervisor.sh` thin and the logic unit-testable (the project's pattern:
bash orchestrates, python decides/serializes).

```python
# scripts/loop/lib/record.py
"""Assemble ONE per-arm observability record from artifacts the supervisor already produced
(before/after scorer JSON, the worker transcript telemetry, the verdict, and a git diff stat).
Pure + deterministic: same inputs -> byte-identical JSON. Observability only — never gates."""
from __future__ import annotations
import json, sys
from pathlib import Path

def _load(p):  # tolerant: missing/corrupt -> {}
    try: return json.loads(Path(p).read_text())
    except Exception: return {}

def per_property_confirmed(scorer: dict) -> dict[str, int]:
    return {k: (v or {}).get("confirmed", 0) for k, v in (scorer.get("per_property") or {}).items()}

def per_property_recall(scorer: dict) -> dict[str, str]:
    """confirmed_false / false_total per family — the recall the competition actually scores."""
    out = {}
    for k, v in (scorer.get("per_property") or {}).items():
        v = v or {}
        out[k] = f"{v.get('confirmed_false', 0)}/{v.get('false_total', 0)}"
    return out

def build(*, n, lever, mode, family, scope, decision, delta, progressed,
          before, after, verdict, result, diffstat, ts) -> dict:
    b, a = per_property_confirmed(before), per_property_confirmed(after)
    fams = sorted(set(b) | set(a))
    return {
        "arm": n, "ts": ts,
        "lever": lever, "mode": mode, "family": family, "scope": scope,
        "decision": decision, "confirmed_delta": delta, "progressed": bool(progressed),
        # score movement, per property (NOT just the total)
        "per_property_before": b, "per_property_after": a,
        "per_property_delta": {f: a.get(f, 0) - b.get(f, 0) for f in fams},
        "per_property_recall_after": per_property_recall(after),
        "overall_confirmed_before": before.get("confirmed_score"),
        "overall_confirmed_after": after.get("confirmed_score"),
        "false_alarms_after": after.get("false_alarms"),
        "wrong_true_after": after.get("wrong_true"),
        # cross-family safety (populated for crosscut arms via verify_arm's verdict)
        "regressed_families": verdict.get("regressed_families", []),
        # worker cost / effort
        "worker": {
            "subtype": result.get("subtype"),
            "num_turns": result.get("num_turns"),
            "hit_max_turns": result.get("hit_max_turns"),
            "total_cost_usd": round(result.get("total_cost_usd", 0.0), 4),
            "duration_ms": result.get("duration_ms"),
            "api_retries": result.get("api_retries"),
        },
        # what changed (from `git diff --stat`, computed in bash — see 1c)
        "diff": diffstat,            # {"files": N, "insertions": N, "deletions": N, "paths": [...]}
        # the worker's own final message — ADVISORY, never trusted for scoring
        "worker_summary": result.get("summary", ""),
    }

if __name__ == "__main__":
    # args: a json blob of the bash-collected scalars on argv[1]; file paths follow
    scalars = json.loads(sys.argv[1])
    before, after = _load(sys.argv[2]), _load(sys.argv[3])
    verdict, result = _load(sys.argv[4]), _load(sys.argv[5])
    diffstat = json.loads(sys.argv[6]) if len(sys.argv) > 6 else {}
    print(json.dumps(build(before=before, after=after, verdict=verdict,
                           result=result, diffstat=diffstat, **scalars), indent=2))
```

### 1c. Wire it into `supervisor.sh` `run_arm`, right after the decision is known

Plugs in at `supervisor.sh:312` (just after `log "decision: $decision ..."`, before the `case`), so it
records **every** arm regardless of KEEP/REVERT/REJECT. The git diff-stat is taken vs `$base` on the
arm branch **before** the checkpoint mutates branches (for a REVERT the arm branch still holds the
worker's edits at this point).

```bash
  # --- observability: assemble the per-arm record from artifacts already on disk (plan 205) ---
  emit_arm_record "$n" "$id" "$mode" "$family" "$scope" "$decision" "$delta" "$progressed" \
                  "$base" "$branch" "$wk"
```

```bash
# add near journal()/journal_note():
emit_arm_record() {  # n id mode family scope decision delta progressed base branch wk
  local n="$1" id="$2" mode="$3" family="$4" scope="$5" dec="$6" delta="$7" prog="$8"
  local base="$9" branch="${10}" wk="${11}"
  # worker telemetry from the transcript (turns/cost/retries/summary) — reuse the parser
  py "$LIB/worker_status.py" --result "$wk/transcript.jsonl" > "$wk/result.json" 2>/dev/null || echo '{}' > "$wk/result.json"
  # diff stat vs the arm's base, from whatever branch currently holds the edits (numstat is stable/parseable)
  local diffstat; diffstat="$(git_here diff --numstat "$base" -- crates Cargo.toml Cargo.lock benchmark-defs 2>/dev/null \
    | py -c 'import sys,json;
rows=[l.split("\t") for l in sys.stdin if l.strip()];
ins=sum(int(r[0]) for r in rows if r[0].isdigit()); dele=sum(int(r[1]) for r in rows if r[1].isdigit());
print(json.dumps({"files":len(rows),"insertions":ins,"deletions":dele,"paths":[r[2] for r in rows][:40]}))' 2>/dev/null || echo '{}')"
  local scalars; scalars="$(py -c 'import json,sys; print(json.dumps(dict(n=int(sys.argv[1]),lever=sys.argv[2],mode=sys.argv[3],family=sys.argv[4],scope=sys.argv[5],decision=sys.argv[6],delta=int(sys.argv[7]),progressed=int(sys.argv[8]),ts=sys.argv[9])))' \
    "$n" "$id" "$mode" "$family" "$scope" "$dec" "$delta" "$prog" "$(date -u +%Y-%m-%dT%H:%M:%SZ)")"
  py "$LIB/record.py" "$scalars" "$wk/before.json" "$wk/after.json" "$wk/verdict.json" "$wk/result.json" "$diffstat" \
    > "$wk/record.json" 2>/dev/null || true
  # append the flat spine line (one JSON object per arm) that every view reads
  [ -s "$wk/record.json" ] && py -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1]))))' "$wk/record.json" \
    >> "$STATE_DIR/arms.jsonl" 2>/dev/null || true
}
```

Notes on placement / safety:
- Runs **after** `verify_arm.py` wrote `verdict.json` and **before** `keep_arm`/`revert_arm`, so the
  diff and worker edits are still present on the arm branch. For REVERT, this is the only chance to
  record what the (discarded) worker did — exactly the "wasted work" signal we lack.
- `delta` here can be the huge-negative sentinel (`-1e9`) on a broken build; store it as-is, and let
  views clamp/label it "build-broken" when `after.json` is absent (record `after` presence explicitly).
- Purely additive; if any step fails, `|| true` keeps the loop's control flow identical to today.

---

## 2. Journal enrichment — per-property Δ, holdout trend, lever ROI, train-vs-holdout gap

The journal is the operator's at-a-glance narrative. Today each arm is one line with only the TOTAL
delta. Enrich it **without** breaking the `**DECISION**` token the smoke test greps
(`smoke_supervisor.sh:75` does `grep -oE '\*\*[A-Z_]+\*\*' | tail -1`) — so keep the arm line's format
and append detail on **continuation lines** (the existing `journal_note` idiom already does this with a
leading two-space `- `, which never matches the `^- arm` decision-line regex).

**Operator questions answered.** *Which property is each arm moving? Is the holdout following the
train curve or diverging? Which levers earn their keep? What's the current generalization gap?*

### 2a. Per-property Δ on each arm (continuation line under the decision)

```bash
# extend journal() OR add a companion called right after it in run_arm's checkpoint:
journal_perproperty() {  # wk
  local wk="$1"
  py - "$wk/before.json" "$wk/after.json" <<'PY' >> "$JOURNAL" 2>/dev/null || true
import json,sys
def pp(p):
    try: d=json.load(open(p)).get("per_property") or {}
    except Exception: d={}
    return {k:(v or {}).get("confirmed",0) for k,v in d.items()}
b,a=pp(sys.argv[1]),pp(sys.argv[2])
moved={k:a.get(k,0)-b.get(k,0) for k in sorted(set(b)|set(a)) if a.get(k,0)-b.get(k,0)!=0}
if moved:
    print("  - Δ/property: " + " ".join(f"{k}{'+' if v>=0 else ''}{v}" for k,v in moved.items()))
PY
}
```
Produces e.g. `  - Δ/property: valid-memsafety+3 no-overflow-1` under a KEEP line — instantly shows a
crosscut arm that traded one family for another even when the total rose.

### 2b. Holdout trend line (the REAL anti-overfit signal, currently buried)

`maybe_heldout_check` writes `heldout-<n>.json` but journals nothing. Add a trend note that also
records the paired TRAIN score at the same moment, so the **gap** is visible inline:

```bash
# inside maybe_heldout_check, after the saf_eval that writes heldout-$n.json:
    py - "$STATE_DIR/heldout-$n.json" "$STATE_DIR/baseline_ref_score.json" <<'PY' >> "$JOURNAL" 2>/dev/null || true
import json,os,sys
h=json.load(open(sys.argv[1]))
hp={k:(v or {}).get("confirmed_false",0) for k,v in (h.get("per_property") or {}).items()}
line=" ".join(f"{k}={v}" for k,v in sorted(hp.items()) if v)
print(f"  - HOLDOUT confirmed={h.get('confirmed_score')} FP={h.get('false_alarms')} "
      f"wrongTRUE={h.get('wrong_true')} [{line}]  (svcomp26, novel tasks — the generalization signal)")
PY
```
Reading a run's journal now shows the holdout line climbing (or, as observed, NOT climbing) interleaved
with the train KEEPs that preceded it. A flat holdout under a rising train is the memorization
diagnosis, spelled out in the log.

> Optional (cheap, high value): also append `train_confirmed` captured from the most recent
> `overall_checkpoint.json` on the same line, so each holdout note is a self-contained
> `train=… holdout=… gap=…` datapoint.

### 2c. Cumulative per-lever ROI ledger

Aggregate `arms.jsonl` into a running tally per lever: KEEP / ACCUMULATE / REVERT / REJECT counts, net
confirmed points, total cost, total turns. This is a **derived view** (recomputed from `arms.jsonl`),
so it can live entirely in `report.sh` (section 4) — no extra supervisor state, deterministic, and
always consistent with the spine. A one-line journal note per overall-checkpoint keeps a durable
trace:

```bash
# in maybe_overall_checkpoint, alongside the existing journal_note "$summary":
  journal_note "$(py "$LIB/lever_roi.py" "$STATE_DIR/arms.jsonl" --top 3 --oneline 2>/dev/null || echo 'lever ROI n/a')"
```

`lever_roi.py` (new, pure) groups `arms.jsonl` by `lever` and sums `confirmed_delta` on KEEP arms,
counts decisions, sums `worker.total_cost_usd` — see the sketch in §4b (shared with the dashboard).

### 2d. Train-vs-holdout gap indicator

A single scalar the operator watches: `gap = latest_train_confirmed − scaled(latest_holdout)`. Since
train and holdout pools differ in size, report the **per-property recall fraction** on each rather than
raw points (recall is size-normalized and directly comparable). Emitted on every holdout note (2b) and
surfaced as a headline in the dashboard (§4). A widening gap = the loop is fitting the train pool.

---

## 3. Worker cost / turn capture

Already specified mechanically in §1a (`extract_result`) and §1c (write `result.json`, fold into the
record). This section states the **exact extraction contract** and where it is logged, since it is a
named deliverable.

**Source.** The terminal stream-json event on the worker transcript:
```json
{"type":"result","subtype":"success","is_error":false,"num_turns":37,
 "duration_ms":641203,"duration_api_ms":588011,"total_cost_usd":2.14,"result":"<final text>"}
```
`subtype` ∈ `success | error_max_turns | error_during_execution`. `api_retry` events are separate
`{"type":"system","subtype":"api_retry",...}` lines — counted, not in the `result` event.

**Extraction.** `worker_status.py --result <transcript>` (§1a) → JSON with `subtype, num_turns,
hit_max_turns, total_cost_usd, duration_ms, duration_api_ms, api_retries, summary`.

**Where logged.**
1. `.loop-state/arm-<n>/result.json` (raw telemetry).
2. Folded into `record.json` / `arms.jsonl` under `"worker"` (§1b).
3. A compact `log()` line at arm end so it shows in `journalctl`:
   ```bash
   log "arm $n cost=\$$(py -c 'import json,sys;print(f"{json.load(open(sys.argv[1]))[\"total_cost_usd\"]:.2f}")' "$wk/result.json" 2>/dev/null || echo '?') turns=$(py -c 'import json,sys;print(json.load(open(sys.argv[1]))[\"num_turns\"])' "$wk/result.json" 2>/dev/null || echo '?') retries=… subtype=…"
   ```
4. Aggregated cost/turn burn in the dashboard (§4).

**Operator questions answered.** *What is the loop costing per kept point? Are arms hitting max_turns
(under-budgeted lever) or finishing early (over-scoped/stuck)? Is a lever expensive AND unproductive
(park it)?* The key derived metric: **$/confirmed-point** per lever = Σcost / Σ(KEEP deltas).

---

## 4. `report.sh` dashboard

Rebuild `report.sh` around `arms.jsonl` (the spine) + the holdout/checkpoint JSONs, surfacing the five
operator questions. Keep it **read-only and derived** — recomputable at any time, never a source of
truth, so it can't drift from the loop. All heavy grouping goes in one small pure helper
(`lib/report_view.py`) that `report.sh` calls; bash stays glue.

**Operator questions answered.** *Is the loop generalizing? Which levers pay off? What work is being
re-derived? What's stalled? What's it costing?*

### 4a. Sections

```
== SAF loop report ==   repo/branch/state, baseline TRAIN confirmed

-- GENERALIZATION (train vs holdout) ----------------------------  [THE headline]
  train  confirmed=<latest overall_checkpoint>   per-property recall: memsafety=…/… overflow=…/… …
  holdout confirmed=<latest heldout-*>            per-property recall: memsafety=…/… overflow=…/… …
  gap (recall pts): memsafety Δ=… overflow Δ=…    <- widening = memorizing
  holdout trend: 7 → 7 → 8   (last N checks)       <- flat under rising train = overfit

-- LEVER ROI ----------------------------------------------------
  lever                 arms  KEEP ACC REV REJ  netΔconf  cost$  $/pt  status
  overflow-recall         12    3   1   8   0     +21    6.40   0.30  parked
  exec-validator-gate      5    2   0   3   0      +7    2.10   0.30  active
  …

-- WASTE / RE-DERIVATION ----------------------------------------
  levers with ≥3 REVERTs and net 0:  air-slicing (5 rev, 0 pt)
  arms that hit max_turns:            #7 #12 (lever …)     <- under-budgeted
  reverted arms with large diffs:     #9 (14 files, +820)  <- big wasted worker effort
  repeated near-identical diffs:      #4≈#11 (same files, lever …)  <- re-deriving

-- RECENT ARMS (last 12) ----------------------------------------
  #14 KEEP  overflow-recall  Δ+3 [no-overflow+3]  turns=31 $1.9   "widened interval reach…"
  #13 REV   air-slicing      Δ0                    turns=60(max!) $3.1 "slicing pass built, no verdict…"
  …

-- CAPABILITY MILESTONES (cap-* lineage) ------------------------
  conc-shim-m1: 4 arms accumulated, last ACCUMULATE #10, tests green   <- progress on multi-arm feature

-- ALERTS -------------------------------------------------------
  (ALERT_* files, held-out hard-reject notes)
```

### 4b. The one helper it needs — `lib/report_view.py`

```python
# scripts/loop/lib/report_view.py
"""Derived read-only views over .loop-state/arms.jsonl (+ heldout-*.json). Pure; no side effects.
Every number is recomputed from the spine, so a view can never disagree with the recorded history."""
from __future__ import annotations
import json, sys, glob, os
from collections import defaultdict

def load_arms(p):
    rows=[]
    try:
        for l in open(p):
            l=l.strip()
            if l:
                try: rows.append(json.loads(l))
                except Exception: pass
    except OSError: pass
    return rows

def lever_roi(rows):
    g=defaultdict(lambda:{"arms":0,"KEEP":0,"ACCUMULATE":0,"REVERT":0,"REJECT":0,
                          "netDelta":0,"cost":0.0,"turns":0,"maxturns":0})
    for r in rows:
        s=g[r.get("lever","?")]; s["arms"]+=1
        dec=r.get("decision","REVERT")
        s[dec if dec in s else "REJECT"]+=1
        if dec=="KEEP": s["netDelta"]+=r.get("confirmed_delta",0)
        w=r.get("worker",{}) or {}
        s["cost"]+=w.get("total_cost_usd",0) or 0
        s["turns"]+=w.get("num_turns",0) or 0
        s["maxturns"]+= 1 if w.get("hit_max_turns") else 0
    for s in g.values():
        s["cost"]=round(s["cost"],2)
        s["dollars_per_point"]=round(s["cost"]/s["netDelta"],3) if s["netDelta"]>0 else None
    return dict(sorted(g.items(), key=lambda kv:(-kv[1]["netDelta"], kv[1]["cost"])))

def waste(rows):
    """Signals of wasted/re-derived work: reverting levers with net 0, max_turns arms, big reverts,
    and near-identical diffs (same touched paths on different arms)."""
    stuck=[k for k,s in lever_roi(rows).items() if s["REVERT"]>=3 and s["netDelta"]<=0]
    maxturns=[r["arm"] for r in rows if (r.get("worker") or {}).get("hit_max_turns")]
    bigreverts=[(r["arm"],(r.get("diff") or {}).get("files",0),(r.get("diff") or {}).get("insertions",0))
                for r in rows if r.get("decision")=="REVERT" and (r.get("diff") or {}).get("files",0)>=8]
    # cheap "re-derivation" heuristic: same sorted touched-path set appears on >1 arm of the same lever
    seen=defaultdict(list)
    for r in rows:
        key=(r.get("lever"), tuple(sorted((r.get("diff") or {}).get("paths",[]))))
        if key[1]: seen[key].append(r["arm"])
    rederived={k[0]:v for k,v in seen.items() if len(v)>1}
    return {"stuck_levers":stuck,"max_turns_arms":maxturns,"big_reverts":bigreverts,
            "rederived":rederived}

def holdout_trend(state_dir):
    pts=[]
    for f in sorted(glob.glob(os.path.join(state_dir,"heldout-*.json")),
                    key=lambda p:int(p.split("-")[-1].split(".")[0])):
        try:
            d=json.load(open(f)); pts.append((os.path.basename(f), d.get("confirmed_score"),
                d.get("false_alarms"), d.get("wrong_true")))
        except Exception: pass
    return pts

if __name__=="__main__":
    cmd, state = sys.argv[1], sys.argv[2]
    rows=load_arms(os.path.join(state,"arms.jsonl"))
    if cmd=="roi":       print(json.dumps(lever_roi(rows),indent=2))
    elif cmd=="waste":   print(json.dumps(waste(rows),indent=2))
    elif cmd=="holdout": print(json.dumps(holdout_trend(state),indent=2))
    elif cmd=="recent":  print(json.dumps(rows[-int(sys.argv[3] if len(sys.argv)>3 else 12):],indent=2))
```

`report.sh` then becomes: print the baseline, call `report_view.py roi|waste|holdout|recent` and format
each block (bash `printf` tables, exactly the style already in `report.sh`). Generalization headline =
`holdout_trend` + the per-property recall from the latest `overall_checkpoint.json` vs the latest
`heldout-*.json`.

---

## 5. Cheap high-value signals we're missing

Ordered by value/effort. All are additive and gitignored.

### 5a. Per-arm task-flip capture (`--per-task`) — the single most important add for generalization

The scorer already supports `--per-task <jsonl>` (writes per-task `property, expected, kind, outcome,
witness, rel_yml, group, data_model, duration_s, stderr_tail`) but the loop never requests it. Capture
it for **before and after**, diff them, and record **exactly which tasks flipped** unknown→confirmed
(and any confirmed→unknown regressions). This is the difference between "the arm added real recall" and
"the arm learned three task shapes."

```bash
# in run_arm, add --per-task to the family evals:
saf_eval "$TRAIN_MANIFEST" "$wk/before.json"  $prop_arg --per-task "$wk/before.pertask.jsonl"
saf_eval "$TRAIN_MANIFEST" "$wk/after.json"   $prop_arg --per-task "$wk/after.pertask.jsonl"
```
```python
# lib/flips.py — which individual tasks changed outcome (by rel_yml, the stable task id)
def flips(before_jsonl, after_jsonl):
    def idx(p):
        m={}
        for l in open(p):
            r=json.loads(l); m[r["rel_yml"]]=r
        return m
    b,a=idx(before_jsonl),idx(after_jsonl)
    gained=[k for k in a if a[k]["outcome"]=="FalseCorrect" and a[k].get("witness")=="CONFIRMED"
            and not (b.get(k,{}).get("outcome")=="FalseCorrect" and b.get(k,{}).get("witness")=="CONFIRMED")]
    lost=[k for k in b if b[k].get("witness")=="CONFIRMED" and a.get(k,{}).get("witness")!="CONFIRMED"]
    new_fp=[k for k in a if a[k]["outcome"]=="FalseIncorrect" and b.get(k,{}).get("outcome")!="FalseIncorrect"]
    return {"gained_confirmed":sorted(gained),"lost_confirmed":sorted(lost),"new_false_alarms":sorted(new_fp)}
```
Fold `flips` into `record.json`. **Operator question answered directly:** *did this arm solve genuinely
new tasks, or re-confirm ones a prior arm already covered (redundant), or trade a confirmed task for a
new false alarm?* Cross-referencing `gained_confirmed` on TRAIN arms against the holdout's misses is
the concrete memorization test.

> Cost note: `--per-task` is free (same run, extra file). It does add IO; the per-arm eval is
> already the dominant cost, so this is negligible.

### 5b. Worker action fingerprint from the transcript

Beyond the final summary (§1), tally the worker's tool usage cheaply: count `Edit`/`Write`/`Bash`
tool_use events and list distinct files touched (from `tool_use.input.file_path`), plus flag whether it
ran `cargo nextest`/`clippy` (evidence it self-checked). Reuses the `_iter_tool_inputs` walker already
in `gates.py`. Answers *did the worker actually test its change, or just edit and stop?* — a strong
predictor of REVERT.

```python
# extend record.py: fingerprint(transcript) -> {"edits":N,"writes":N,"bash":N,
#   "ran_tests":bool,"ran_clippy":bool,"files":[...]}  (walk tool_use inputs; cheap)
```

### 5c. Capability-milestone progress tracker

Capability levers (mode=`capability`, e.g. `conc-shim-m1`) build a feature over MANY arms and correctly
score 0 for a while — invisible to a score-only view, which makes them look like pure waste. Track their
`cap-*` lineage: number of ACCUMULATE arms, cumulative LOC added, whether `saf-svcomp` tests are green,
and the most recent worker summary. Derivable from `arms.jsonl` filtered to `mode=capability` +
`captest.log`. Surfaced as the "CAPABILITY MILESTONES" dashboard block (§4a). Answers *is the multi-arm
feature actually advancing, or spinning?*

### 5d. Decision-cause tagging (why did this arm REVERT?)

`verify_arm.py` already knows the proximate cause (delta≤0, family_regressed, tamper, holdout). Persist a
single `revert_reason` enum into `verdict.json` (it has the pieces; just name it) and carry it into
`record.json`: `no_gain | regression | build_broken | family_trade | tamper | holdout | worker_fail |
noop`. Then the dashboard's REVERT rows say *why*, and the ROI view can separate "lever produces bad
changes" (regression) from "lever produces no-ops" (no_gain) — different fixes (tighten the lever vs
retire it).

```python
# in verify_arm.py, when writing verdict-out, add:
def revert_reason(delta, after_present, regressed, decision):
    if decision!="REVERT": return None
    if not after_present:  return "build_broken"
    if regressed:          return "family_trade"
    if delta<0:            return "regression"
    return "no_gain"       # delta==0 and not progressed
```
(`worker_fail`/`tamper`/`holdout` are set at their existing branch points in `run_arm`/`decide`.)

---

## 6. Where each piece plugs in (summary map)

| Addition | New/changed file | Plugs into | Deliverable |
|----------|------------------|-----------|-------------|
| `extract_result()` telemetry | `lib/worker_status.py` (+`--result`) | called by `emit_arm_record` | §1a, §3 |
| Per-arm record assembler | `lib/record.py` (new) | `emit_arm_record` in `supervisor.sh` | §1 |
| `emit_arm_record()` + `arms.jsonl` | `supervisor.sh` (after :312) | end of `run_arm` verify phase | §1c |
| Per-property Δ journal line | `supervisor.sh` `journal_perproperty()` | KEEP/ACCUMULATE checkpoint | §2a |
| Holdout trend + gap note | `supervisor.sh` `maybe_heldout_check` | after holdout eval | §2b, §2d |
| Lever ROI ledger note | `supervisor.sh` `maybe_overall_checkpoint` | uses `lib/report_view.py` | §2c |
| Cost/turn log line | `supervisor.sh` end of `run_arm` | reads `result.json` | §3 |
| Dashboard | `report.sh` (rewrite) + `lib/report_view.py` (new) | read-only, anytime | §4 |
| Task-flip capture | `supervisor.sh` (`--per-task` on family evals) + `lib/flips.py` (new) | folded into `record.json` | §5a |
| Worker fingerprint | `lib/record.py` `fingerprint()` | `record.json` | §5b |
| Capability tracker | `lib/report_view.py` filter | dashboard block | §5c |
| Revert-reason tag | `lib/verify_arm.py` + `verdict.json` | `record.json`, dashboard | §5d |

**Tests to add (mirroring `tests/` conventions):**
`test_worker_status.py` (fixture transcript → asserted telemetry), `test_record.py` (before/after +
verdict + result blobs → asserted record incl. per-property delta and diff passthrough), `test_flips.py`
(before/after per-task JSONL → asserted gained/lost/new_fp), and one `report_view.py` roundtrip case in
the existing smoke or a new `test_report_view.py`. All pure-python, no Docker/Claude — same as the
current suite. The bash smoke (`smoke_supervisor.sh`) needs its stub `saf_eval` extended to also honor
`--per-task` (it already parses `--per-task`, line 15) and to write `arms.jsonl`; assert a record row is
appended per scenario.

**Determinism / freeze compliance.** Every writer emits sorted keys / stable ordering (BTree-equivalent
in Python via `sorted()`), writes only under `.loop-state/`, and is `|| true`-guarded so it can never
change the loop's keep/revert outcome. All new code lives in `scripts/loop/` — inside the immutable set,
so deploying it requires the operator to **re-freeze** (`freeze_immutables` / re-snapshot the gate lib),
which is expected and documented in the constraints.
