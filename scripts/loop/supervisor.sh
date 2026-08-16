#!/usr/bin/env bash
# supervisor.sh — the SAF autonomous improvement loop (plan 204 + Addendum A).
#
# This is a PLAIN process, NOT a Claude instance: it owns ALL loop control, ALL rate-limit handling,
# and ALL hard gates deterministically, so a worker `claude` hitting a usage window can never wedge
# the loop or slip a bad change through. Claude is used only for (a) the per-arm improvement attempt
# and (b) an optional diff review — both launched here and subject to this script's rate-limit policy.
#
# Soundness/anti-reward-hacking decisions are delegated to the unit-tested Python in lib/ (gates.py,
# verify_arm.py, ratelimit.py). This file is orchestration: git branches, Docker eval/test, the
# claude worker, and keep/revert/reject bookkeeping. It NEVER pushes and NEVER merges to `svcomp`.
#
# Usage:
#   supervisor.sh --once [--lever <id>]     # Stage 1 watched dry-run: run exactly ONE arm, then stop
#   supervisor.sh --loop                    # Stage 2/3: run arms until a cap / signal
#   supervisor.sh --baseline                # just re-run the immutable scorer on TRAIN and report
#
# Injectable seams for local smoke-testing without Docker/Claude: set SAF_LOOP_STUBS=<file> to a
# script that redefines saf_eval / saf_worker / saf_tests (sourced AFTER the real defaults).
set -euo pipefail

# ----------------------------------------------------------------------------- config
LOOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SAF_REPO_ROOT:-$(cd "$LOOP_DIR/../.." && pwd)}"   # overridable for local smoke tests
LIB="$LOOP_DIR/lib"
: "${LEVERS_FILE:=$LOOP_DIR/levers.tsv}"
: "${ARM_PROMPT_FILE:=$LOOP_DIR/arm_prompt.md}"
: "${WORKER_SETTINGS:=$LOOP_DIR/worker-settings.json}"
STATE_DIR="${SAF_LOOP_STATE:-$REPO_ROOT/.loop-state}"
: "${LOOP_ENV:=$LOOP_DIR/loop.env}"
[ -f "$LOOP_ENV" ] && set -a && . "$LOOP_ENV" && set +a

: "${CLAUDE_ENV_FILE:=$HOME/.claude-code.env}"
: "${MAX_TURNS:=40}"
: "${EVAL_JOBS:=8}"
: "${EVAL_MAX_RSS_MB:=3072}"
: "${EVAL_TIMEOUT:=60}"
: "${HELDOUT_EVERY_K:=3}"          # run the svcomp26 held-out eval every K kept arms (the real signal)
: "${OVERALL_CHECKPOINT_EVERY_KEEP:=3}"  # after every N KEPT arms run a FULL all-property eval: track the
                                   # true C.FalseOverall trajectory + revert any cross-family regression.
                                   # 3 balances safety vs eval cost (the crosscut gate already prevents the
                                   # main regressors per-arm at no extra cost; this backstops local leakage).
: "${MAX_ARMS:=1}"                  # --loop cap; --once forces 1
: "${SCRATCH_PREFIX:=auto/loop}"    # kept tuning arms land here (never pushed, never merged)
: "${CAP_PREFIX:=cap}"              # capability scaffolding lineage: cap-<capability>/<n>
: "${ALLOWED_TOOLS:=Read,Grep,Glob,Edit,Write,Bash,WebSearch,WebFetch}"
# Immutable set — the scorer/splitter/validator/labels/holdout AND the loop's own harness
# (`scripts/loop`, hashed recursively) so a worker can't edit its own gate. The agent may never alter these.
: "${IMMUTABLE_GLOBS:=scripts/svcomp_split_eval.py scripts/svcomp_split.py scripts/validate_witness.sh tests/benchmarks/svcomp-splits/holdout.jsonl tests/benchmarks/svcomp-splits/train.jsonl tests/benchmarks/svcomp-splits/val.jsonl scripts/loop}"
# Forbidden reads audited in the worker transcript (the held-out manifest — the agent may not READ it).
: "${FORBIDDEN_READS:=svcomp-splits/holdout}"
: "${LEVER_BUDGET:=6}"              # park a lever after this many CONSECUTIVE REVERTs (progress resets it)
: "${MAX_LEVER_ARMS:=12}"           # hard cap on total arms per lever (bounds accumulation spend)
TRAIN_MANIFEST="$REPO_ROOT/tests/benchmarks/svcomp-splits/train.jsonl"
HOLDOUT_MANIFEST="$REPO_ROOT/tests/benchmarks/svcomp-splits/holdout.jsonl"
VAL_MANIFEST="$REPO_ROOT/tests/benchmarks/svcomp-splits/val.jsonl"
# LOOP_GEN_MODE=on: the generalization gate (decide_v2) — arms are scored on the reasoning `val` set with
# per-cluster dedup weighting + a deduped-pool no-regression guard, so the loop rewards novel solving, not
# Juliet memorization. off (default): legacy `decide` on the raw train sample. Flip on for a fresh campaign.
: "${LOOP_GEN_MODE:=off}"
# Gate self-protection: run the gate PYTHON and store the immutable manifest in a PRISTINE dir OUTSIDE
# the repo, snapshotted from the clean baseline, so a worker editing scripts/loop/lib on its arm branch
# cannot neuter its own judge — the check itself always runs untampered.
GATE_LIB="${SAF_GATE_LIB:-$HOME/.saf-loop-gate}"
IMMUTABLE_MANIFEST="$GATE_LIB/immutable.sha256.json"
# Runtime journal lives in the gitignored state dir: durable across restarts, never clobbered by an
# arm's reset --hard, and keeps the tracked tree clean. (scripts/loop/progress-journal.md is the seed.)
: "${JOURNAL:=$STATE_DIR/journal.md}"

# Claude Code headless env (Addendum A6): supervisor sees hard blocks fast and owns the sleep policy.
export CLAUDE_CODE_MAX_RETRIES="${CLAUDE_CODE_MAX_RETRIES:-4}"
export API_TIMEOUT_MS="${API_TIMEOUT_MS:-1200000}"
unset CLAUDE_CODE_ENABLE_TELEMETRY DISABLE_TELEMETRY 2>/dev/null || true  # any non-empty value turns ON

mkdir -p "$STATE_DIR"

log() { printf '[%s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2; }
die() { log "FATAL: $*"; exit 1; }
py()  { python3 "$@"; }

# ----------------------------------------------------------------------------- git helpers (never push/merge)
git_here() { git -C "$REPO_ROOT" "$@"; }
current_branch() { git_here rev-parse --abbrev-ref HEAD; }
baseline_ref() { cat "$STATE_DIR/baseline_ref" 2>/dev/null || git_here rev-parse HEAD; }
assert_clean_tree() {
  git_here diff --quiet && git_here diff --cached --quiet \
    || die "working tree not clean — the loop needs a clean baseline (Addendum A8). Commit/stash first."
}

# ----------------------------------------------------------------------------- immutable freeze + check
freeze_immutables() {
  mkdir -p "$(dirname "$IMMUTABLE_MANIFEST")"
  log "freezing immutables -> $IMMUTABLE_MANIFEST"
  py - "$REPO_ROOT" "$IMMUTABLE_MANIFEST" $IMMUTABLE_GLOBS <<'PY'
import hashlib, json, sys
from pathlib import Path
root, out, *rels = sys.argv[1:]
m = {}
def add(rel):
    p = Path(root) / rel
    if p.is_dir():
        for f in sorted(p.rglob("*")):
            if f.is_file() and "__pycache__" not in f.parts:
                m[str(f.relative_to(root))] = hashlib.sha256(f.read_bytes()).hexdigest()
    elif p.is_file():
        m[str(rel)] = hashlib.sha256(p.read_bytes()).hexdigest()
    else:
        print(f"WARN: immutable not found (skipped): {rel}", file=sys.stderr)
for rel in rels:
    add(rel)
Path(out).write_text(json.dumps(m, indent=2))
print(f"froze {len(m)} immutable files")
PY
  # Defense in depth: make plain-file immutables read-only on disk (a-w). Dirs stay hash-gated only
  # (chmod -R would make scripts/loop read-only and can trip git ops; the hash check + pristine gate copy cover it).
  for rel in $IMMUTABLE_GLOBS; do [ -f "$REPO_ROOT/$rel" ] && chmod a-w "$REPO_ROOT/$rel" || true; done
}
snapshot_gate_lib() {
  # Copy the gate python to a pristine dir OUTSIDE the repo and run ALL gate logic from there, so a
  # worker editing scripts/loop/lib on its arm branch cannot fool the check that judges it.
  mkdir -p "$GATE_LIB"
  cp -f "$LOOP_DIR"/lib/*.py "$GATE_LIB"/ 2>/dev/null || true
  LIB="$GATE_LIB"
}

# ----------------------------------------------------------------------------- injectable command seams
# saf_eval <manifest.jsonl> <out.json> [--property P] [--sample N] [--per-task PT]
saf_eval() {
  local manifest="$1" out="$2"; shift 2
  local sample_arg=""; [ "${EVAL_SAMPLE:-0}" -gt 0 ] && sample_arg="--sample ${EVAL_SAMPLE}"
  # The scorer runs INSIDE the container (repo bind-mounted at /workspace). Build the command, then
  # translate any absolute host path under the repo root to its /workspace equivalent so the manifest,
  # -o output, and --per-task paths all resolve in-container (host reads the same files via the mount).
  local inner="cargo build --release -p saf-cli >/dev/null 2>&1 && \
    python3 scripts/svcomp_split_eval.py --manifest '$manifest' --confirm-witness \
      --jobs $EVAL_JOBS --timeout $EVAL_TIMEOUT --max-rss-mb $EVAL_MAX_RSS_MB $sample_arg -o '$out' $*"
  inner="${inner//$REPO_ROOT\//\/workspace\/}"
  docker compose -f "$REPO_ROOT/docker-compose.yml" run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c "$inner"
}
# NOTE: there is NO saf_tests gate (user decision 2026-08-16). build/tests/clippy are the loop AGENT's
# responsibility (the arm_prompt tells it to make them pass), NOT supervisor gates. A build break is
# caught implicitly: saf_eval's `cargo build --release -p saf-cli` fails -> no after.json -> the score
# cannot improve -> the arm reverts. Only anti-cheat + score gate a KEEP.
# saf_captest -> exit 0 iff the arm's code compiles + saf-svcomp tests pass. This is the ACCUMULATE
# progress signal: it decides whether SCORE-NEUTRAL work is USEFUL enough to preserve for future arms
# (delta==0 case only) — it is NOT a gate for scoring arms.
saf_captest() {
  docker compose -f "$REPO_ROOT/docker-compose.yml" run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \
    'cargo nextest run -p saf-svcomp'
}
# saf_worker <prompt_file> <transcript_out> <session_id_out> ; drives claude with rate-limit handling.
# All retries/resumes are keyed by session_id (never --continue). A wall-clock `timeout` guards a
# stalled stream (Addendum A6). Classification is deterministic via lib/worker_status.py.
saf_worker() {
  local prompt="$1" transcript="$2" sid_out="$3"
  local sid; sid="$(py -c 'import uuid;print(uuid.uuid4())')"; echo "$sid" > "$sid_out"
  [ -f "$CLAUDE_ENV_FILE" ] && { set -a; . "$CLAUDE_ENV_FILE"; set +a; }
  export SAF_FORBIDDEN_READS="$FORBIDDEN_READS"   # consumed by the PreToolUse denylist hook
  local attempt=0 max_hard="${WORKER_MAX_ATTEMPTS:-8}" wall="${WORKER_WALL_TIMEOUT:-3600}"
  local common=(-p --output-format stream-json --verbose --max-turns "$MAX_TURNS"
                --allowedTools "$ALLOWED_TOOLS" --dangerously-skip-permissions)
  [ -f "$WORKER_SETTINGS" ] && common+=(--settings "$WORKER_SETTINGS")
  while :; do
    attempt=$((attempt+1))
    log "worker attempt $attempt (session $sid)"
    if [ "$attempt" -eq 1 ]; then
      timeout "$wall" claude "${common[@]}" --session-id "$sid" < "$prompt" \
        > "$transcript" 2>"$transcript.err" || true
    else
      # resume the SAME session and let it continue where it left off
      timeout "$wall" claude "${common[@]}" --resume "$sid" <<<"continue the task" \
        > "$transcript" 2>"$transcript.err" || true
    fi
    local verdict; verdict="$(py "$LIB/worker_status.py" "$transcript" 2>/dev/null || echo error)"
    case "$verdict" in
      success)   return 0 ;;
      max_turns) log "worker hit max_turns; resuming $sid" ;;
      sleep:*)   sleep_to_reset "${verdict#sleep:}" ;;
      transient) local b=$(( (RANDOM % 30) + 15 * attempt )); log "transient; backoff ${b}s"; sleep "$b" ;;
      abort)     log "worker abort (auth/billing/quota-per-request)"; return 2 ;;
      *)         [ "$attempt" -ge "$max_hard" ] && { log "worker error, giving up after $attempt"; return 3; }
                 sleep $(( 30 * attempt )) ;;
    esac
    [ "$attempt" -ge "$max_hard" ] && { log "worker exhausted attempts"; return 3; }
  done
}
sleep_to_reset() {  # arg: unix epoch
  local now target; now="$(date +%s)"; target="$1"
  local dur=$(( target - now )); [ "$dur" -lt 0 ] && dur=0; [ "$dur" -gt 21600 ] && dur=21600
  log "hard usage window: sleeping ${dur}s to reset (epoch $target)"; sleep "$dur"
}

# Load smoke stubs (override the three seams) if provided — for local control-flow testing.
if [ -n "${SAF_LOOP_STUBS:-}" ]; then
  # shellcheck disable=SC1090
  . "$SAF_LOOP_STUBS"
  log "loaded stubs from $SAF_LOOP_STUBS"
fi

# ----------------------------------------------------------------------------- immutable check (real)
immutable_violations() {
  py - "$IMMUTABLE_MANIFEST" "$REPO_ROOT" "$LIB" <<'PY'
import json, sys
manifest_p, root, lib = sys.argv[1:4]
sys.path.insert(0, lib)
import gates
m = json.loads(open(manifest_p).read()) if __import__("os").path.exists(manifest_p) else {}
print("\n".join(gates.verify_immutables(m, root)))
PY
}

# ----------------------------------------------------------------------------- lever selection
pick_lever() {  # prints "id<TAB>mode<TAB>family<TAB>scope<TAB>description"
  local forced="${1:-}"
  if [ -n "$forced" ]; then
    awk -F'\t' -v id="$forced" 'NF>=5 && $1==id {print; exit}' "$LEVERS_FILE"; return
  fi
  # ROUND-ROBIN ACROSS FAMILIES (user 2026-08-16): every property type gets coverage instead of the first
  # productive lever hogging the budget. Families are taken in first-appearance (ROI) order; a persisted
  # cursor makes each pick start AFTER the family worked last. WITHIN a family, the first non-parked lever
  # (ROI order) is picked — so a productive lever still repeats, but only on its family's turn. Repeats are
  # allowed; only PARKED levers (stalled past budget) are skipped.
  local fams; fams="$(awk -F'\t' 'NF>=5 && $1 !~ /^#/ {print $3}' "$LEVERS_FILE" | awk '!seen[$0]++')"
  local last; last="$(cat "$STATE_DIR/family_cursor" 2>/dev/null || echo '')"
  local ordered; ordered="$(printf '%s\n' "$fams" | awk -v last="$last" '
    { all[NR]=$0; if ($0==last) at=NR }
    END { if (at) { for(i=at+1;i<=NR;i++) print all[i]; for(i=1;i<=at;i++) print all[i] }
          else    { for(i=1;i<=NR;i++)   print all[i] } }')"
  local fam id line
  while IFS= read -r fam; do
    [ -z "$fam" ] && continue
    while IFS= read -r line; do
      id="$(printf '%s' "$line" | cut -f1)"
      [ -e "$STATE_DIR/lever.$id.parked" ] && continue
      printf '%s' "$fam" > "$STATE_DIR/family_cursor"
      printf '%s\n' "$line"; return 0
    done < <(awk -F'\t' -v f="$fam" 'NF>=5 && $1 !~ /^#/ && $3==f {print}' "$LEVERS_FILE")
  done <<< "$ordered"
  return 0
}
record_lever_outcome() {  # id decision : PROGRESS (KEEP/ACCUMULATE) resets the stall; park on a stuck
                          # run (LEVER_BUDGET consecutive REVERTs) OR a hard total-arms cap.
  local id="$1" dec="$2" sf="$STATE_DIR/lever.$1.stall" af="$STATE_DIR/lever.$1.attempts"
  local at; at=$(( $(cat "$af" 2>/dev/null || echo 0) + 1 )); echo "$at" > "$af"
  if [ "$at" -ge "$MAX_LEVER_ARMS" ]; then
    touch "$STATE_DIR/lever.$id.parked"
    log "lever $id PARKED after $at total arms (cap $MAX_LEVER_ARMS)"; return
  fi
  # PROGRESS resets the park-stall. In gen mode only a real generalization gain (KEEP) or preserved
  # capability progress (ACCUMULATE_PLUS) counts — a KEEP_POOL (Juliet-only points) or plain ACCUMULATE does
  # NOT, so pure-memorization / neutral levers still park. In legacy mode KEEP|ACCUMULATE reset (as before).
  local reset=0
  case "$dec" in
    KEEP|ACCUMULATE_PLUS) reset=1 ;;
    ACCUMULATE) [ "$LOOP_GEN_MODE" = on ] || reset=1 ;;
  esac
  if [ "$reset" = 1 ]; then
    echo 0 > "$sf"
  else
    local s; s=$(( $(cat "$sf" 2>/dev/null || echo 0) + 1 )); echo "$s" > "$sf"
    if [ "$s" -ge "$LEVER_BUDGET" ]; then
      touch "$STATE_DIR/lever.$id.parked"
      log "lever $id PARKED after $s arms with no progress (budget $LEVER_BUDGET); accumulated work stays on the branch"
    fi
  fi
}

# ----------------------------------------------------------------------------- one arm
run_arm() {
  local lever_line; lever_line="$(pick_lever "${1:-}")"
  [ -z "$lever_line" ] && { log "no available lever (all tried/parked)"; return 10; }
  local id mode family scope desc
  id="$(printf '%s' "$lever_line" | cut -f1)"
  mode="$(printf '%s' "$lever_line" | cut -f2)"
  family="$(printf '%s' "$lever_line" | cut -f3)"
  scope="$(printf '%s' "$lever_line" | cut -f4)"; [ "$scope" = crosscut ] || scope=local
  desc="$(printf '%s' "$lever_line" | cut -f5-)"
  local n; n="$(( $(cat "$STATE_DIR/arm_counter" 2>/dev/null || echo 0) + 1 ))"; echo "$n" > "$STATE_DIR/arm_counter"
  local base="$WORK_BRANCH" branch="arm/${n}"   # uniform: every arm builds on + folds into the work branch

  # A LOCAL lever (change confined to one confirmer/verdict path) is eval'd on its FAMILY only — cheap,
  # and it cannot move another property. A CROSSCUT lever (frontend/PTA/AIR/slicing — shared code that
  # runs for every property) is eval'd on ALL properties and REVERTED if any family regressed, so shared
  # infra can never trade one property's recall away for another's.
  local prop_arg="--property $family" ccflag=""
  [ "$scope" = crosscut ] && { prop_arg=""; ccflag="--check-all-families"; }

  log "=== ARM $n | lever=$id ($mode) | family=$family | scope=$scope ==="
  log "    $desc"
  local wk="$STATE_DIR/arm-$n"; mkdir -p "$wk"

  # ORIENT + BASELINE on the work branch. Soundness is SOFT (baked into the score), so a base carrying
  # some false alarms is noted, not fatal — we do NOT die.
  git_here checkout -q "$base"
  if [ "$LOOP_GEN_MODE" = on ]; then
    # gen gate: score the reasoning VAL set (the metric) + the deduped TRAIN pool (the guard), both weighted.
    # --per-task on the pool eval feeds the observability task-flip diff (plan 205 §5a); free (same run).
    saf_eval "$VAL_MANIFEST"   "$wk/val_before.json"  $prop_arg --group-weight
    saf_eval "$TRAIN_MANIFEST" "$wk/pool_before.json" $prop_arg --group-weight --per-task "$wk/before.pertask.jsonl"
    py -c "import json;v=json.load(open('$wk/val_before.json'));p=json.load(open('$wk/pool_before.json'));print('base: val_w=%s pool_w=%s FP=%s'%(v.get('confirmed_score_weighted'),p.get('confirmed_score_weighted'),p.get('false_alarms')))" >&2 || true
  else
    saf_eval "$TRAIN_MANIFEST" "$wk/before.json" $prop_arg --per-task "$wk/before.pertask.jsonl"
    py -c "import json;d=json.load(open('$wk/before.json'));print('base: confirmed=%s FP=%s wrongTRUE=%s'%(d.get('confirmed_score'),d.get('false_alarms'),d.get('wrong_true')))" >&2 || true
  fi

  # ACT on a throwaway arm branch cut from the work branch
  git_here checkout -q -b "$branch"
  local prompt="$wk/arm_prompt.rendered.md"
  render_arm_prompt "$id" "$mode" "$family" "$desc" > "$prompt"
  if ! saf_worker "$prompt" "$wk/transcript.jsonl" "$wk/session_id"; then
    log "worker did not complete cleanly; reverting arm $n"
    # Record the wasted-work signal BEFORE reverting (worker edits are still on the arm branch): a
    # worker that burned turns/cost then aborted is the single most expensive waste event. before.json
    # exists from ORIENT; there is no after (record.py tolerates it → no-after). Guarded observability.
    local wfb; [ "$LOOP_GEN_MODE" = on ] && wfb="$wk/pool_before.json" || wfb="$wk/before.json"
    emit_arm_record "$n" "$id" "$mode" "$family" "$scope" "WORKER_FAIL" 0 0 "$base" "$wfb" "$wk/after.json" "$wk" || true
    revert_arm "$branch"; journal "$n" "$id" "$mode" "WORKER_FAIL" ""; record_lever_outcome "$id" REVERT; return 1
  fi

  # VERIFY. Anti-cheat + score are the ONLY gates for a WIN. delta>0 -> KEEP (real gain). delta==0 AND
  # the work is USEFUL (compiles + saf-svcomp tests pass + non-empty diff) -> ACCUMULATE: preserve it on
  # the integration branch so FUTURE arms build ON it instead of re-deriving it (user ask). Otherwise
  # (regression / broken build / no-op) -> REVERT. The agent's self-report is never trusted.
  # useful-work probe (ACCUMULATE signal): a real crates/manifest change that compiles + passes saf-svcomp tests.
  local progressed=0
  if ! git_here diff --quiet "$base" -- crates Cargo.toml Cargo.lock 2>/dev/null; then
    saf_captest >"$wk/captest.log" 2>&1 && progressed=1
  fi
  local decision delta="n/a" gen_delta="n/a" pool_delta="n/a" checkpoint_before checkpoint_after jdelta
  local fwd; fwd="$(for f in $FORBIDDEN_READS; do printf ' --forbidden %s' "$f"; done)"
  if [ "$LOOP_GEN_MODE" = on ]; then
    saf_eval "$VAL_MANIFEST"   "$wk/val_after.json"  $prop_arg --group-weight
    saf_eval "$TRAIN_MANIFEST" "$wk/pool_after.json" $prop_arg --group-weight --per-task "$wk/after.pertask.jsonl"
    gen_delta="$(py -c "import json,os;b=json.load(open('$wk/val_before.json'));a=json.load(open('$wk/val_after.json')) if os.path.exists('$wk/val_after.json') else {};c=a.get('confirmed_score_weighted');print((c-b.get('confirmed_score_weighted',0)) if c is not None else -999999999)")"
    pool_delta="$(py -c "import json,os;b=json.load(open('$wk/pool_before.json'));a=json.load(open('$wk/pool_after.json')) if os.path.exists('$wk/pool_after.json') else {};c=a.get('confirmed_score_weighted');print((c-b.get('confirmed_score_weighted',0)) if c is not None else -999999999)")"
    local novel=0; [ "$mode" = capability ] && novel=1
    decision="$(py "$LIB/verify_arm.py" --gen-mode \
        --before "$wk/val_before.json" --after "$wk/val_after.json" \
        --pool-before "$wk/pool_before.json" --pool-after "$wk/pool_after.json" \
        --immutable-manifest "$IMMUTABLE_MANIFEST" --repo-root "$REPO_ROOT" \
        --transcript "$wk/transcript.jsonl" $fwd \
        --progressed "$progressed" --novel-solved "$novel" $ccflag --verdict-out "$wk/verdict.json")" || true
    checkpoint_before="$wk/pool_before.json"; checkpoint_after="$wk/pool_after.json"; jdelta="$gen_delta"
    log "decision: $decision (gen_delta=$gen_delta pool_delta=$pool_delta progressed=$progressed scope=$scope)"
  else
    saf_eval "$TRAIN_MANIFEST" "$wk/after.json" $prop_arg --per-task "$wk/after.pertask.jsonl"
    delta="$(py -c "import json,os;b=json.load(open('$wk/before.json'));a=json.load(open('$wk/after.json')) if os.path.exists('$wk/after.json') else {};c=a.get('confirmed_score');print((c-b.get('confirmed_score',0)) if c is not None else -999999999)")"
    decision="$(py "$LIB/verify_arm.py" \
        --before "$wk/before.json" --after "$wk/after.json" \
        --immutable-manifest "$IMMUTABLE_MANIFEST" --repo-root "$REPO_ROOT" \
        --transcript "$wk/transcript.jsonl" $fwd \
        --progressed "$progressed" $ccflag --verdict-out "$wk/verdict.json")" || true
    checkpoint_before="$wk/before.json"; checkpoint_after="$wk/after.json"; jdelta="$delta"
    log "decision: $decision (delta=$delta progressed=$progressed scope=$scope)"
  fi

  # OBSERVABILITY (plan 205 §1b/§3/§5a): assemble the per-arm record from artifacts already on disk
  # and append the arms.jsonl spine — AFTER the decision, BEFORE the checkpoint mutates branches, so a
  # REVERT still records the discarded worker diff (the wasted-work signal). Purely additive + guarded.
  emit_arm_record "$n" "$id" "$mode" "$family" "$scope" "$decision" "$jdelta" "$progressed" \
                  "$base" "$checkpoint_before" "$checkpoint_after" "$wk" || true
  local costline; costline="$(py -c 'import json,sys
d=json.load(open(sys.argv[1]))
print("cost=$%.2f turns=%s retries=%s subtype=%s"%(d.get("total_cost_usd",0) or 0,d.get("num_turns"),d.get("api_retries"),d.get("subtype")))' "$wk/result.json" 2>/dev/null || true)"
  [ -n "$costline" ] && log "arm $n worker $costline" || true

  # CHECKPOINT
  case "$decision" in
    KEEP|KEEP_POOL)     # both bank real (deduped) points and advance the integration branch
      keep_arm "$branch" "$n" "$id" "$decision (Δ=$jdelta)"
      journal "$n" "$id" "$mode" "$decision" "$jdelta"
      journal_perproperty "$checkpoint_before" "$checkpoint_after"
      maybe_heldout_check "$n"
      maybe_overall_checkpoint "$n" "$scope" "$checkpoint_after" ;;
    ACCUMULATE|ACCUMULATE_PLUS)   # score-neutral useful work preserved for future arms (PLUS = capability progress)
      keep_arm "$branch" "$n" "$id" "$decision (useful; reusable by future arms)"
      journal "$n" "$id" "$mode" "$decision" "0"
      journal_perproperty "$checkpoint_before" "$checkpoint_after" ;;
    REJECT_TAMPER|REJECT_HOLDOUT)
      log "SECURITY: $decision on arm $n — reverting + alerting"
      revert_arm "$branch"; journal "$n" "$id" "$mode" "$decision" "" ; touch "$STATE_DIR/ALERT_$decision" ;;
    *)
      revert_arm "$branch"; journal "$n" "$id" "$mode" "REVERT" "" ;;
  esac
  record_lever_outcome "$id" "$decision"
  return 0
}

keep_arm() {  # branch n id [tag] — fast-forward the work branch onto the arm. Never pushed/merged.
  local branch="$1" n="$2" id="$3" tag="${4:-KEPT}"
  # Stage ONLY real source (crates + manifests + test fixtures/defs) — NOT worker scratch (probe files,
  # temp dirs, garbage-named outputs) that would bloat the integration branch. Each path is added only
  # if it exists (robust for minimal repos). Submodules are never in this list.
  local p
  for p in crates Cargo.toml Cargo.lock benchmark-defs docker Dockerfile tests/programs tests/fixtures; do
    [ -e "$REPO_ROOT/$p" ] && git_here add -A -- "$p" || true
  done
  git_here commit -q -m "loop(arm $n): $id — $tag" || true
  local armcommit; armcommit="$(git_here rev-parse HEAD)"
  git_here checkout -q "$WORK_BRANCH"
  git_here merge -q --ff-only "$armcommit"
  git_here branch -qD "$branch" 2>/dev/null || true
  git_here rev-parse HEAD > "$STATE_DIR/baseline_ref"
  log "arm $n $tag -> $WORK_BRANCH @ $(git_here rev-parse --short HEAD) (never pushed/merged to svcomp)"
}
revert_arm() {  # branch [base(ignored)]
  local branch="$1"
  git_here reset -q --hard HEAD 2>/dev/null || true   # drop the worker's uncommitted tracked edits
  git_here clean -fdq 2>/dev/null || true             # drop the worker's untracked files (ignored kept)
  git_here checkout -q "$WORK_BRANCH" 2>/dev/null || true
  git_here reset -q --hard "$WORK_BRANCH" 2>/dev/null || true
  git_here clean -fdq 2>/dev/null || true
  git_here branch -qD "$branch" 2>/dev/null || true
}
setup_work_branch() {
  # The loop runs on an integration branch (auto/loop-<date>); kept tuning arms fast-forward it so
  # improvements accumulate for the human to cherry-pick to svcomp. Resumable across restarts.
  local integ
  if [ -s "$STATE_DIR/work_branch" ]; then
    integ="$(cat "$STATE_DIR/work_branch")"
  else
    integ="${SCRATCH_PREFIX}-$(date -u +%Y%m%d)"
    echo "$integ" > "$STATE_DIR/work_branch"
  fi
  git_here checkout -q -B "$integ"          # create at (or move to) current HEAD; idempotent on resume
  WORK_BRANCH="$integ"
  [ -f "$JOURNAL" ] || cp "$LOOP_DIR/progress-journal.md" "$JOURNAL" 2>/dev/null || : > "$JOURNAL"
  [ -s "$STATE_DIR/baseline_ref" ] || git_here rev-parse HEAD > "$STATE_DIR/baseline_ref"
  log "work branch: $WORK_BRANCH (baseline $(git_here rev-parse --short HEAD))"
}
maybe_heldout_check() {
  local n="$1"
  local kept; kept="$(( $(cat "$STATE_DIR/kept_counter" 2>/dev/null || echo 0) + 1 ))"; echo "$kept" > "$STATE_DIR/kept_counter"
  if [ $(( kept % HELDOUT_EVERY_K )) -eq 0 ]; then
    log "held-out check (every $HELDOUT_EVERY_K kept): the REAL anti-overfit signal"
    saf_eval "$HOLDOUT_MANIFEST" "$STATE_DIR/heldout-$n.json" || true
    # NOTE: a train gain that does not reproduce on held-out is a HARD reject (Addendum A2);
    # the operator/next iteration deprioritizes that lever. Recorded for review.
    # Journal the holdout confirmed + per-property recall + the paired TRAIN score so the
    # train↔holdout GAP is visible inline (plan 205 §2b/§2d). A flat holdout under a rising
    # train is the memorization diagnosis, spelled out in the log. Observability only, guarded.
    py - "$STATE_DIR/heldout-$n.json" "$STATE_DIR/overall_checkpoint.json" <<'PY' >> "$JOURNAL" 2>/dev/null || true
import json,os,sys
try: h=json.load(open(sys.argv[1]))
except Exception: sys.exit(0)
hp={k:(v or {}).get("confirmed_false",0) for k,v in (h.get("per_property") or {}).items()}
line=" ".join(f"{k}={v}" for k,v in sorted(hp.items()) if v)
train=""
if os.path.exists(sys.argv[2]):
    try: train=" train=%s"%json.load(open(sys.argv[2])).get("confirmed_score")
    except Exception: pass
print(f"  - HOLDOUT confirmed={h.get('confirmed_score')} FP={h.get('false_alarms')} "
      f"wrongTRUE={h.get('wrong_true')} [{line}]{train}  (svcomp26 novel tasks — the generalization signal)")
PY
  fi
}
journal() {  # n id mode outcome delta
  printf -- '- arm %s | %s | %s | **%s** | Δconfirmed=%s | %s\n' \
    "$1" "$2" "$3" "$4" "${5:-n/a}" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$JOURNAL"
}
journal_note() {  # free-form trajectory line (no **OUTCOME** token, so it never shadows an arm decision)
  printf -- '  - %s | %s\n' "$1" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$JOURNAL"
}
# emit_arm_record — assemble ONE per-arm observability record (lib/record.py) from artifacts already
# on disk and append the arms.jsonl spine. Observability ONLY: every step is `|| true`-guarded and
# writes solely under $STATE_DIR, so it can NEVER change the loop's keep/revert control flow. Called
# after the decision, before the checkpoint (so a REVERT still records the discarded worker diff).
emit_arm_record() {  # n id mode family scope decision jdelta progressed base cbefore cafter wk
  local n="$1" id="$2" mode="$3" family="$4" scope="$5" dec="$6" jdelta="$7" prog="$8"
  local base="$9" cb="${10}" ca="${11}" wk="${12}"
  # worker telemetry (turns/cost/retries/summary) from the transcript — reuse the parser
  py "$LIB/worker_status.py" --result "$wk/transcript.jsonl" > "$wk/result.json" 2>/dev/null || echo '{}' > "$wk/result.json"
  # diff stat vs the arm's base (worker edits are still uncommitted on the arm branch at this point)
  local diffstat; diffstat="$(git_here diff --numstat "$base" -- crates Cargo.toml Cargo.lock benchmark-defs 2>/dev/null \
    | py -c 'import sys,json
rows=[l.rstrip("\n").split("\t") for l in sys.stdin if l.strip()]
ins=sum(int(r[0]) for r in rows if r and r[0].isdigit())
dele=sum(int(r[1]) for r in rows if len(r)>1 and r[1].isdigit())
print(json.dumps({"files":len(rows),"insertions":ins,"deletions":dele,"paths":[r[2] for r in rows if len(r)>2][:40]}))' 2>/dev/null || true)"
  [ -n "$diffstat" ] || diffstat='{}'
  local scalars; scalars="$(py -c 'import json,sys
def i(x):
    try: return int(float(x))
    except Exception: return 0
print(json.dumps(dict(n=i(sys.argv[1]),lever=sys.argv[2],mode=sys.argv[3],family=sys.argv[4],scope=sys.argv[5],decision=sys.argv[6],delta=i(sys.argv[7]),progressed=i(sys.argv[8]),ts=sys.argv[9])))' \
    "$n" "$id" "$mode" "$family" "$scope" "$dec" "$jdelta" "$prog" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" 2>/dev/null || true)"
  [ -n "$scalars" ] || return 0
  py "$LIB/record.py" "$scalars" "$cb" "$ca" "$wk/verdict.json" "$wk/result.json" "$diffstat" \
     "$wk/transcript.jsonl" "$wk/before.pertask.jsonl" "$wk/after.pertask.jsonl" > "$wk/record.json" 2>/dev/null || true
  # append the flat spine line (one deterministic JSON object per arm) that every view reads
  [ -s "$wk/record.json" ] && py -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1])),sort_keys=True))' "$wk/record.json" \
    >> "$STATE_DIR/arms.jsonl" 2>/dev/null || true
}
# journal_perproperty — a continuation line under a KEEP/ACCUMULATE decision showing which property
# each arm moved (a crosscut arm that traded one family for another is visible even if the total rose).
journal_perproperty() {  # before_json after_json
  py - "$1" "$2" <<'PY' >> "$JOURNAL" 2>/dev/null || true
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

# init_overall_checkpoint — establish the pre-loop all-property C.FalseOverall reference the periodic
# checkpoint compares against (so even the FIRST KEEP is guarded). Tags the starting HEAD `overall-good`;
# runs one full all-property eval only if we don't already have a checkpoint (resume keeps the prior one).
init_overall_checkpoint() {
  git_here tag -f overall-good "$WORK_BRANCH" >/dev/null 2>&1 || true
  [ -s "$STATE_DIR/overall_checkpoint.json" ] && return 0
  log "establishing initial overall (all-property) checkpoint baseline"
  saf_eval "$TRAIN_MANIFEST" "$STATE_DIR/overall_checkpoint.json" || true
}

# maybe_overall_checkpoint <n> <scope> <after_json> — after every OVERALL_CHECKPOINT_EVERY_KEEP kept arms,
# measure the TRUE all-property C.FalseOverall (reusing a crosscut arm's already-all-property after.json),
# journal the per-property trajectory, and if the total REGRESSED vs the last good checkpoint, ALERT and
# roll the work branch back to the `overall-good` tag (backstop for a "local" change that wasn't). This is
# the safety net behind the per-arm crosscut gate: local arms are only eval'd on their own family, so a
# genuine cross-family leak is caught here.
maybe_overall_checkpoint() {
  local n="$1" scope="$2" after_json="$3"
  local kc; kc="$(( $(cat "$STATE_DIR/ckpt_keep_counter" 2>/dev/null || echo 0) + 1 ))"; echo "$kc" > "$STATE_DIR/ckpt_keep_counter"
  [ $(( kc % OVERALL_CHECKPOINT_EVERY_KEEP )) -eq 0 ] || return 0
  log "overall checkpoint (every $OVERALL_CHECKPOINT_EVERY_KEEP kept): full all-property C.FalseOverall"
  local ov="$STATE_DIR/overall-$n.json"
  if [ "$scope" = crosscut ] && [ -s "$after_json" ]; then
    cp -f "$after_json" "$ov"                              # crosscut arm already eval'd ALL properties
  else
    saf_eval "$TRAIN_MANIFEST" "$ov" || { log "checkpoint eval failed; skipping"; return 0; }
  fi
  local summary rc
  summary="$(py - "$ov" "$STATE_DIR/overall_checkpoint.json" <<'PY'
import json, os, sys
ov = json.load(open(sys.argv[1])); tot = ov.get("confirmed_score", 0)
per = ov.get("per_property", {}) or {}
line = " ".join("%s=%s" % (k, (v or {}).get("confirmed", 0)) for k, v in sorted(per.items()))
prev = sys.argv[2]; ptot = None
if os.path.exists(prev):
    try: ptot = json.load(open(prev)).get("confirmed_score")
    except Exception: ptot = None
print("overall CONFIRMED=%d [%s]%s" % (tot, line, "" if ptot is None else " (prev %d)" % ptot))
sys.exit(2 if (ptot is not None and tot < ptot) else 0)
PY
)"; rc=$?
  journal_note "$summary"; log "$summary"
  # cumulative per-lever ROI ledger (plan 205 §2c): a durable one-line trace, recomputed from arms.jsonl
  journal_note "$(py -c 'import sys,os; sys.path.insert(0,sys.argv[2]); import report_view as rv; print(rv.roi_oneline(rv.load_arms(os.path.join(sys.argv[1],"arms.jsonl")),3))' "$STATE_DIR" "$LIB" 2>/dev/null || echo 'lever ROI: n/a')"
  if [ "$rc" -eq 2 ]; then
    log "ALERT: OVERALL C.FalseOverall REGRESSED — rolling $WORK_BRANCH back to last-good checkpoint (overall-good)"
    touch "$STATE_DIR/ALERT_OVERALL_REGRESSION"; journal_note "**ALERT_OVERALL_REGRESSION** rolled back to overall-good"
    git_here checkout -q "$WORK_BRANCH" 2>/dev/null || true
    git_here reset -q --hard overall-good 2>/dev/null || true
    git_here rev-parse HEAD > "$STATE_DIR/baseline_ref"
  else
    cp -f "$ov" "$STATE_DIR/overall_checkpoint.json"       # advance the reference
    git_here tag -f overall-good "$WORK_BRANCH" >/dev/null 2>&1 || true
  fi
}

render_arm_prompt() {  # id mode family desc -> stdout (fixed prompt + this arm's lever)
  local id="$1" mode="$2" family="$3" desc="$4"
  sed -e "s|{{LEVER_ID}}|$id|g" -e "s|{{MODE}}|$mode|g" -e "s|{{FAMILY}}|$family|g" \
      -e "s|{{DESC}}|$desc|g" "$ARM_PROMPT_FILE"
}

# ----------------------------------------------------------------------------- entrypoints
cmd_baseline() {
  saf_eval "$TRAIN_MANIFEST" "$STATE_DIR/baseline.json"
  py -c "import json;d=json.load(open('$STATE_DIR/baseline.json'));print('CONFIRMED',d['confirmed_score'],'FP',d['false_alarms'],'wrongTRUE',d['wrong_true'])"
}
main() {
  local mode="${1:---once}"; shift || true
  git_here rev-parse --is-inside-work-tree >/dev/null 2>&1 || die "not a git repo: $REPO_ROOT"
  snapshot_gate_lib   # pristine gate copy BEFORE anything runs on an arm branch
  case "$mode" in
    --baseline) freeze_immutables; cmd_baseline ;;
    --once)
      setup_work_branch; freeze_immutables    # freeze from the clean work-branch baseline
      log "DRY-RUN: one arm, foreground, watched, then STOP"
      run_arm "${1:-}" ;;
    --loop)
      setup_work_branch; freeze_immutables; init_overall_checkpoint
      local i=0 rc=0
      while [ "$i" -lt "$MAX_ARMS" ]; do
        [ -e "$STATE_DIR/STOP" ] && { log "STOP file present — halting"; break; }
        rc=0; run_arm || rc=$?
        [ "$rc" -eq 10 ] && { log "no active levers remain — campaign complete"; break; }
        i=$((i+1))
      done
      log "loop finished ($i arms); work branch $WORK_BRANCH left for human review" ;;
    *) die "unknown mode $mode (use --once | --loop | --baseline)" ;;
  esac
}
main "$@"
