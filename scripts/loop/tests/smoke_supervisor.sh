#!/usr/bin/env bash
# smoke_supervisor.sh — end-to-end control-flow test of supervisor.sh WITHOUT Docker/Claude.
# Injects stub saf_eval/saf_worker/saf_tests and drives one arm per scenario in a throwaway git
# repo, asserting the supervisor's keep/revert/reject decision. This exercises the real orient ->
# act -> verify -> checkpoint wiring + the gates (a)-(f) end to end (the unit tests cover the pure
# decision logic; this covers the bash orchestration around it).
set -uo pipefail
LOOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUP="$LOOP_DIR/supervisor.sh"
pass=0 fail=0

STUBS_BODY='
saf_eval() {
  local out="$2"; shift 2
  local pt=""; while [ $# -gt 0 ]; do [ "$1" = "--per-task" ] && pt="$2"; shift; done
  local confirmed=100 fp=0 wt=0 per="" b; b="$(basename "$out")"
  case "$SMOKE_SCENARIO:$b" in
    tuning_gain:after.json)      confirmed=105 ;;
    fp_lowers_score:after.json)  confirmed=90; fp=1 ;;   # FP -> -16 baked in -> net lower -> revert
    # crosscut arm scored on ALL properties: total rises +20 but valid-memsafety DROPS 200 to 190 ->
    # family_regressed -> REVERT (never trade one property for another).
    crosscut_regression:before.json) confirmed=300; per="{\"valid-memsafety\":{\"confirmed\":200},\"unreach-call\":{\"confirmed\":100}}" ;;
    crosscut_regression:after.json)  confirmed=320; per="{\"valid-memsafety\":{\"confirmed\":190},\"unreach-call\":{\"confirmed\":130}}" ;;
    # GEN MODE (LOOP_GEN_MODE=on): a val gain (43>40) with a flat deduped pool (200) -> KEEP each arm;
    # the svcomp26 holdout stays FLAT at 7 -> the actionable brake parks the lever after two misses.
    gen_keep:val_before.json)    confirmed=40 ;;
    gen_keep:val_after.json)     confirmed=43 ;;
    gen_keep:pool_before.json)   confirmed=200 ;;
    gen_keep:pool_after.json)    confirmed=200 ;;
    gen_keep:heldout-*)          confirmed=7 ;;          # FLAT across checkpoints -> does not reproduce
    *:after.json)                confirmed=100 ;;
  esac
  [ -z "$per" ] && per="{\"valid-memsafety\":{\"confirmed\":$confirmed,\"confirmed_false\":$confirmed,\"false_total\":200}}"
  # confirmed_score_weighted mirrors confirmed so gen-mode decide_v2 / the holdout brake have a deduped number.
  printf "{\"confirmed_score\":%s,\"confirmed_score_weighted\":%s,\"false_alarms\":%s,\"wrong_true\":%s,\"raw_score\":%s,\"max_score\":1000,\"per_property\":%s}\n" \
    "$confirmed" "$confirmed" "$fp" "$wt" "$confirmed" "$per" > "$out"
  # per-task dump for the observability task-flip diff: before=unknown, after=confirmed (a gain), so
  # record.py exercises the flips path end to end (plan 205 §5a). Keyed on the output filename.
  if [ -n "$pt" ]; then
    case "$(basename "$pt")" in
      before.pertask.jsonl) printf "%s\n" "{\"rel_yml\":\"t/x.yml\",\"property\":\"valid-memsafety\",\"outcome\":\"unknown\",\"witness\":null}" > "$pt" ;;
      after.pertask.jsonl)  printf "%s\n" "{\"rel_yml\":\"t/x.yml\",\"property\":\"valid-memsafety\",\"outcome\":\"FalseCorrect\",\"witness\":\"CONFIRMED\"}" > "$pt" ;;
      *) : > "$pt" ;;
    esac
  fi
  return 0
}
saf_tests() { return 0; }
saf_captest() { [ "$SMOKE_SCENARIO" = accumulate ] && return 0 || return 1; }  # useful-work probe: pass only for the accumulate scenario
saf_worker() {
  local transcript="$2" sid_out="$3"; echo smoke-sid > "$sid_out"
  case "$SMOKE_SCENARIO" in
    tamper)        chmod u+w "$REPO_ROOT/scripts/svcomp_split_eval.py"; echo HACKED >> "$REPO_ROOT/scripts/svcomp_split_eval.py" ;;
    gate_sabotage) echo "def verify_immutables(m, r): return []" > "$REPO_ROOT/scripts/loop/lib/gates.py" ;;  # try to neuter the judge
    holdout)       : ;;
    *)             echo "// arm $SMOKE_SCENARIO" >> "$REPO_ROOT/crates/saf-svcomp/src/lib.rs" ;;
  esac
  if [ "$SMOKE_SCENARIO" = holdout ]; then
    printf "%s\n" "{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"tool_use\",\"name\":\"Read\",\"input\":{\"file_path\":\"tests/benchmarks/svcomp-splits/holdout.jsonl\"}}]}}" > "$transcript"
  else
    printf "%s\n" "{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"tool_use\",\"name\":\"Read\",\"input\":{\"file_path\":\"tests/benchmarks/svcomp-splits/train.jsonl\"}}]}}" > "$transcript"
  fi
  printf "%s\n" "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false}" >> "$transcript"
  [ "$SMOKE_SCENARIO" = worker_fail ] && return 1   # simulate a worker that edited then aborted
  return 0
}
reap_orphan_containers() { echo reaped >> "$STATE_DIR/reaped.log"; }  # BUG-3: prove the sweep is invoked (no docker in smoke)
'

scenario() {  # name  scenario  lever  expected-outcome-word
  local name="$1" sc="$2" lever="$3" expect="$4"
  local tmp; tmp="$(mktemp -d)"
  mkdir -p "$tmp/scripts" "$tmp/tests/benchmarks/svcomp-splits" "$tmp/crates/saf-svcomp/src"
  echo "IMMUTABLE SCORER v1" > "$tmp/scripts/svcomp_split_eval.py"
  echo '{"t":1}' > "$tmp/tests/benchmarks/svcomp-splits/train.jsonl"
  echo '{"t":2}' > "$tmp/tests/benchmarks/svcomp-splits/holdout.jsonl"
  echo 'pub fn foo() {}' > "$tmp/crates/saf-svcomp/src/lib.rs"
  mkdir -p "$tmp/scripts/loop/lib"; cp "$LOOP_DIR"/lib/*.py "$tmp/scripts/loop/lib/"  # so scripts/loop is hashed
  printf 'state/\n.loop-state/\nlevers.tsv\nstubs.sh\ngate/\n' > "$tmp/.gitignore"  # test cfg is untracked; real harness is tracked+outside-repo (survives git clean)   # mirror the real repo: loop state is ignored
  git -C "$tmp" init -q
  git -C "$tmp" -c user.email=a@b -c user.name=t add -A >/dev/null
  git -C "$tmp" -c user.email=a@b -c user.name=t commit -qm init
  echo "$STUBS_BODY" > "$tmp/stubs.sh"
  printf 'tune-smoke\ttuning\tvalid-memsafety\tlocal\tsmoke tuning\n' > "$tmp/levers.tsv"
  printf 'cap-smoke\tcapability\tvalid-memsafety\tlocal\tsmoke capability\n' >> "$tmp/levers.tsv"
  printf 'cross-smoke\tcapability\tunreach-call\tcrosscut\tsmoke crosscut (all-property gated)\n' >> "$tmp/levers.tsv"

  local log; log="$(SAF_REPO_ROOT="$tmp" SAF_LOOP_STATE="$tmp/state" SAF_GATE_LIB="$tmp/gate" SAF_LOOP_STUBS="$tmp/stubs.sh" \
      LEVERS_FILE="$tmp/levers.tsv" ARM_PROMPT_FILE="$LOOP_DIR/arm_prompt.md" \
      JOURNAL="$tmp/state/journal.md" LOOP_ENV=/dev/null SMOKE_SCENARIO="$sc" \
      GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=a@b GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=a@b \
      bash "$SUP" --once "$lever" 2>&1)"
  local got; got="$(grep -oE '\*\*[A-Z_]+\*\*' "$tmp/state/journal.md" 2>/dev/null | tail -1 | tr -d '*')"
  # After any arm the loop must sit on the integration branch with a CLEAN tree (no detached HEAD,
  # no leftover worker files) — the multi-arm-safety property the dry-run surfaced.
  local dirty br; dirty="$(git -C "$tmp" status --porcelain --ignore-submodules=all 2>/dev/null | wc -l | tr -d ' ')"
  br="$(git -C "$tmp" rev-parse --abbrev-ref HEAD 2>/dev/null)"
  # OBSERVABILITY (plan 205 §1b): every arm — KEEP or REVERT or REJECT — must append exactly one
  # arms.jsonl spine row for arm 1 whose recorded decision matches the journal decision.
  local rec; rec="$(python3 -c 'import json,os,sys
p=sys.argv[1]
rows=[json.loads(l) for l in open(p)] if os.path.exists(p) else []
r=next((r for r in rows if r.get("arm")==1), None)
print(r.get("decision") if r else "MISSING")' "$tmp/state/arms.jsonl" 2>/dev/null || echo ERR)"
  if [ "$got" = "$expect" ] && [ "$dirty" = "0" ] && [[ "$br" == auto/loop-* ]] && [ "$rec" = "$expect" ]; then
    printf '  ok   %-16s scenario=%-14s -> %-13s [clean, on %s, rec=%s]\n' "$name" "$sc" "$got" "$br" "$rec"; pass=$((pass+1))
  else
    printf '  FAIL %-16s scenario=%-14s expected=%s got=%s dirty=%s branch=%s rec=%s\n' "$name" "$sc" "$expect" "${got:-<none>}" "$dirty" "$br" "$rec"; fail=$((fail+1))
    printf '       --- supervisor output ---\n%s\n' "$log" | sed 's/^/       /' | tail -20
  fi
  rm -rf "$tmp" 2>/dev/null || true
}

multi_arm_budget() {  # --loop must allow repeated attempts, PARK a stalled lever after LEVER_BUDGET, then break
  local tmp; tmp="$(mktemp -d)"
  mkdir -p "$tmp/scripts/loop/lib" "$tmp/tests/benchmarks/svcomp-splits" "$tmp/crates/saf-svcomp/src"
  echo "SCORER" > "$tmp/scripts/svcomp_split_eval.py"
  cp "$LOOP_DIR"/lib/*.py "$tmp/scripts/loop/lib/"
  echo '{"t":1}' > "$tmp/tests/benchmarks/svcomp-splits/train.jsonl"
  echo '{"t":2}' > "$tmp/tests/benchmarks/svcomp-splits/holdout.jsonl"
  echo 'pub fn foo(){}' > "$tmp/crates/saf-svcomp/src/lib.rs"
  printf 'state/\n.loop-state/\nlevers.tsv\nstubs.sh\ngate/\n' > "$tmp/.gitignore"  # test cfg is untracked; real harness is tracked+outside-repo (survives git clean)
  git -C "$tmp" init -q
  git -C "$tmp" -c user.email=a@b -c user.name=t add -A >/dev/null
  git -C "$tmp" -c user.email=a@b -c user.name=t commit -qm init
  echo "$STUBS_BODY" > "$tmp/stubs.sh"
  printf 'only\ttuning\tvalid-memsafety\tlocal\tonly lever\n' > "$tmp/levers.tsv"   # single lever, always reverts
  SAF_REPO_ROOT="$tmp" SAF_LOOP_STATE="$tmp/state" SAF_GATE_LIB="$tmp/gate" SAF_LOOP_STUBS="$tmp/stubs.sh" \
    LEVERS_FILE="$tmp/levers.tsv" ARM_PROMPT_FILE="$LOOP_DIR/arm_prompt.md" JOURNAL="$tmp/state/journal.md" \
    LOOP_ENV=/dev/null SMOKE_SCENARIO=tuning_nogain MAX_ARMS=8 LEVER_BUDGET=3 \
    GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=a@b GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=a@b \
    bash "$SUP" --loop >"$tmp.log" 2>&1
  local reverts parked broke
  reverts="$(grep -c 'REVERT' "$tmp/state/journal.md" 2>/dev/null || echo 0)"
  [ -e "$tmp/state/lever.only.parked" ] && parked=yes || parked=no
  grep -q 'no active levers remain' "$tmp.log" && broke=yes || broke=no
  if [ "$reverts" = 3 ] && [ "$parked" = yes ] && [ "$broke" = yes ]; then
    printf '  ok   %-16s reverts=%s parked=%s broke-early=%s (budget 3 < cap 8)\n' multi_arm_budget "$reverts" "$parked" "$broke"; pass=$((pass+1))
  else
    printf '  FAIL %-16s reverts=%s(want 3) parked=%s broke-early=%s\n' multi_arm_budget "$reverts" "$parked" "$broke"; fail=$((fail+1))
    tail -18 "$tmp.log" | sed 's/^/       /'
  fi
  rm -rf "$tmp" 2>/dev/null || true
}

family_rotation() {  # lever selection must ROUND-ROBIN across families, not repeat the first lever 3x
  local tmp; tmp="$(mktemp -d)"
  mkdir -p "$tmp/scripts/loop/lib" "$tmp/tests/benchmarks/svcomp-splits" "$tmp/crates/saf-svcomp/src"
  echo "SCORER" > "$tmp/scripts/svcomp_split_eval.py"
  cp "$LOOP_DIR"/lib/*.py "$tmp/scripts/loop/lib/"
  echo '{"t":1}' > "$tmp/tests/benchmarks/svcomp-splits/train.jsonl"
  echo '{"t":2}' > "$tmp/tests/benchmarks/svcomp-splits/holdout.jsonl"
  echo 'pub fn foo(){}' > "$tmp/crates/saf-svcomp/src/lib.rs"
  printf 'state/\n.loop-state/\nlevers.tsv\nstubs.sh\ngate/\n' > "$tmp/.gitignore"
  git -C "$tmp" init -q
  git -C "$tmp" -c user.email=a@b -c user.name=t add -A >/dev/null
  git -C "$tmp" -c user.email=a@b -c user.name=t commit -qm init
  echo "$STUBS_BODY" > "$tmp/stubs.sh"
  # three families, one always-reverting lever each -> a pure test of the selection ORDER (rotation)
  printf 'la\ttuning\tfamA\tlocal\tlever A\n'  > "$tmp/levers.tsv"
  printf 'lb\ttuning\tfamB\tlocal\tlever B\n' >> "$tmp/levers.tsv"
  printf 'lc\ttuning\tfamC\tlocal\tlever C\n' >> "$tmp/levers.tsv"
  SAF_REPO_ROOT="$tmp" SAF_LOOP_STATE="$tmp/state" SAF_GATE_LIB="$tmp/gate" SAF_LOOP_STUBS="$tmp/stubs.sh" \
    LEVERS_FILE="$tmp/levers.tsv" ARM_PROMPT_FILE="$LOOP_DIR/arm_prompt.md" JOURNAL="$tmp/state/journal.md" \
    LOOP_ENV=/dev/null SMOKE_SCENARIO=tuning_nogain MAX_ARMS=3 LEVER_BUDGET=9 \
    GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=a@b GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=a@b \
    bash "$SUP" --loop >"$tmp.log" 2>&1
  local seq; seq="$(awk -F' \\| ' '/^- arm/ {print $2}' "$tmp/state/journal.md" 2>/dev/null | tr -d ' ' | tr '\n' ',')"
  if [ "$seq" = "la,lb,lc," ]; then
    printf '  ok   %-16s picked=%s (round-robins families, not the first lever 3x)\n' family_rotation "$seq"; pass=$((pass+1))
  else
    printf '  FAIL %-16s picked=%s want=la,lb,lc,\n' family_rotation "$seq"; fail=$((fail+1))
    tail -18 "$tmp.log" | sed 's/^/       /'
  fi
  rm -rf "$tmp" 2>/dev/null || true
}

accumulate_park() {  # BUG-4: a lever that only ever ACCUMULATEs (0 KEEPs) must park on ACCUMULATE_BUDGET,
                     # not run to MAX_LEVER_ARMS. Single capability lever, useful-but-score-neutral every arm.
  local tmp; tmp="$(mktemp -d)"
  mkdir -p "$tmp/scripts/loop/lib" "$tmp/tests/benchmarks/svcomp-splits" "$tmp/crates/saf-svcomp/src"
  echo "SCORER" > "$tmp/scripts/svcomp_split_eval.py"
  cp "$LOOP_DIR"/lib/*.py "$tmp/scripts/loop/lib/"
  echo '{"t":1}' > "$tmp/tests/benchmarks/svcomp-splits/train.jsonl"
  echo '{"t":2}' > "$tmp/tests/benchmarks/svcomp-splits/holdout.jsonl"
  echo 'pub fn foo(){}' > "$tmp/crates/saf-svcomp/src/lib.rs"
  printf 'state/\n.loop-state/\nlevers.tsv\nstubs.sh\ngate/\n' > "$tmp/.gitignore"
  git -C "$tmp" init -q
  git -C "$tmp" -c user.email=a@b -c user.name=t add -A >/dev/null
  git -C "$tmp" -c user.email=a@b -c user.name=t commit -qm init
  echo "$STUBS_BODY" > "$tmp/stubs.sh"
  printf 'accjunk\tcapability\tvalid-memsafety\tlocal\tan unscoreable capability that only accumulates\n' > "$tmp/levers.tsv"
  SAF_REPO_ROOT="$tmp" SAF_LOOP_STATE="$tmp/state" SAF_GATE_LIB="$tmp/gate" SAF_LOOP_STUBS="$tmp/stubs.sh" \
    LEVERS_FILE="$tmp/levers.tsv" ARM_PROMPT_FILE="$LOOP_DIR/arm_prompt.md" JOURNAL="$tmp/state/journal.md" \
    LOOP_ENV=/dev/null SMOKE_SCENARIO=accumulate MAX_ARMS=8 LEVER_BUDGET=9 ACCUMULATE_BUDGET=3 \
    GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=a@b GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=a@b \
    bash "$SUP" --loop >"$tmp.log" 2>&1
  local accs parked broke reason
  accs="$(grep -c 'ACCUMULATE' "$tmp/state/journal.md" 2>/dev/null || echo 0)"
  [ -e "$tmp/state/lever.accjunk.parked" ] && parked=yes || parked=no
  grep -q 'no active levers remain' "$tmp.log" && broke=yes || broke=no
  reason="$(grep -oE 'PARKED \(accumulate' "$tmp.log" | head -1)"
  if [ "$accs" = 3 ] && [ "$parked" = yes ] && [ "$broke" = yes ] && [ -n "$reason" ]; then
    printf '  ok   %-16s accumulates=%s parked=%s(accumulate) broke-early=%s (budget 3 < cap 8)\n' accumulate_park "$accs" "$parked" "$broke"; pass=$((pass+1))
  else
    printf '  FAIL %-16s accumulates=%s(want 3) parked=%s broke-early=%s reason=%s\n' accumulate_park "$accs" "$parked" "$broke" "${reason:-<none>}"; fail=$((fail+1))
    tail -20 "$tmp.log" | sed 's/^/       /'
  fi
  rm -rf "$tmp" 2>/dev/null || true
}

cap_wip_reprime() {  # BUG-2: a reverted capability arm's crates diff is preserved to lever.<id>.wip.patch
                     # and the NEXT same-lever arm's rendered prompt is primed with it (no re-derivation).
  local tmp; tmp="$(mktemp -d)"
  mkdir -p "$tmp/scripts/loop/lib" "$tmp/tests/benchmarks/svcomp-splits" "$tmp/crates/saf-svcomp/src"
  echo "SCORER" > "$tmp/scripts/svcomp_split_eval.py"
  cp "$LOOP_DIR"/lib/*.py "$tmp/scripts/loop/lib/"
  echo '{"t":1}' > "$tmp/tests/benchmarks/svcomp-splits/train.jsonl"
  echo '{"t":2}' > "$tmp/tests/benchmarks/svcomp-splits/holdout.jsonl"
  echo 'pub fn foo(){}' > "$tmp/crates/saf-svcomp/src/lib.rs"
  printf 'state/\n.loop-state/\nlevers.tsv\nstubs.sh\ngate/\n' > "$tmp/.gitignore"
  git -C "$tmp" init -q
  git -C "$tmp" -c user.email=a@b -c user.name=t add -A >/dev/null
  git -C "$tmp" -c user.email=a@b -c user.name=t commit -qm init
  echo "$STUBS_BODY" > "$tmp/stubs.sh"
  printf 'capwip\tcapability\tvalid-memsafety\tlocal\tmulti-arm capability that reverts then continues\n' > "$tmp/levers.tsv"
  SAF_REPO_ROOT="$tmp" SAF_LOOP_STATE="$tmp/state" SAF_GATE_LIB="$tmp/gate" SAF_LOOP_STUBS="$tmp/stubs.sh" \
    LEVERS_FILE="$tmp/levers.tsv" ARM_PROMPT_FILE="$LOOP_DIR/arm_prompt.md" JOURNAL="$tmp/state/journal.md" \
    LOOP_ENV=/dev/null SMOKE_SCENARIO=tuning_nogain MAX_ARMS=2 LEVER_BUDGET=9 \
    GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=a@b GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=a@b \
    bash "$SUP" --loop >"$tmp.log" 2>&1
  local saved primed
  [ -s "$tmp/state/lever.capwip.wip.patch" ] && saved=yes || saved=no
  grep -q 'lever.capwip.wip.patch' "$tmp/state/arm-2/arm_prompt.rendered.md" 2>/dev/null && primed=yes || primed=no
  if [ "$saved" = yes ] && [ "$primed" = yes ]; then
    printf '  ok   %-16s wip-saved=%s arm2-primed=%s (capability continues, not re-derives)\n' cap_wip_reprime "$saved" "$primed"; pass=$((pass+1))
  else
    printf '  FAIL %-16s wip-saved=%s arm2-primed=%s\n' cap_wip_reprime "$saved" "$primed"; fail=$((fail+1))
    tail -20 "$tmp.log" | sed 's/^/       /'
  fi
  rm -rf "$tmp" 2>/dev/null || true
}

reap_swept() {  # BUG-3: reap_orphan_containers must be invoked on BOTH the keep and the revert checkpoint path
  local ok=yes sc
  for sc in tuning_gain tuning_nogain; do
    local tmp; tmp="$(mktemp -d)"
    mkdir -p "$tmp/scripts/loop/lib" "$tmp/tests/benchmarks/svcomp-splits" "$tmp/crates/saf-svcomp/src"
    echo SCORER > "$tmp/scripts/svcomp_split_eval.py"
    cp "$LOOP_DIR"/lib/*.py "$tmp/scripts/loop/lib/"
    echo '{"t":1}' > "$tmp/tests/benchmarks/svcomp-splits/train.jsonl"
    echo '{"t":2}' > "$tmp/tests/benchmarks/svcomp-splits/holdout.jsonl"
    echo 'pub fn foo(){}' > "$tmp/crates/saf-svcomp/src/lib.rs"
    printf 'state/\n.loop-state/\nlevers.tsv\nstubs.sh\ngate/\n' > "$tmp/.gitignore"
    git -C "$tmp" init -q
    git -C "$tmp" -c user.email=a@b -c user.name=t add -A >/dev/null
    git -C "$tmp" -c user.email=a@b -c user.name=t commit -qm init
    echo "$STUBS_BODY" > "$tmp/stubs.sh"
    printf 'l\ttuning\tvalid-memsafety\tlocal\treap probe\n' > "$tmp/levers.tsv"
    SAF_REPO_ROOT="$tmp" SAF_LOOP_STATE="$tmp/state" SAF_GATE_LIB="$tmp/gate" SAF_LOOP_STUBS="$tmp/stubs.sh" \
      LEVERS_FILE="$tmp/levers.tsv" ARM_PROMPT_FILE="$LOOP_DIR/arm_prompt.md" JOURNAL="$tmp/state/journal.md" \
      LOOP_ENV=/dev/null SMOKE_SCENARIO="$sc" \
      GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=a@b GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=a@b \
      bash "$SUP" --once l >/dev/null 2>&1
    [ -s "$tmp/state/reaped.log" ] || ok=no
    rm -rf "$tmp" 2>/dev/null || true
  done
  if [ "$ok" = yes ]; then
    printf '  ok   %-16s reap invoked on keep + revert checkpoints\n' reap_swept; pass=$((pass+1))
  else
    printf '  FAIL %-16s reap NOT invoked on some checkpoint path\n' reap_swept; fail=$((fail+1))
  fi
}

gen_credit_priority() {  # §2.5: within a family, pick_lever prefers the highest earned gen_credit (holdout
                         # boosts) over file order; ties fall back to file order.
  local tmp; tmp="$(mktemp -d)"
  mkdir -p "$tmp/scripts/loop/lib" "$tmp/tests/benchmarks/svcomp-splits" "$tmp/crates/saf-svcomp/src" "$tmp/state"
  echo SCORER > "$tmp/scripts/svcomp_split_eval.py"
  cp "$LOOP_DIR"/lib/*.py "$tmp/scripts/loop/lib/"
  echo '{"t":1}' > "$tmp/tests/benchmarks/svcomp-splits/train.jsonl"
  echo '{"t":2}' > "$tmp/tests/benchmarks/svcomp-splits/holdout.jsonl"
  echo 'pub fn foo(){}' > "$tmp/crates/saf-svcomp/src/lib.rs"
  printf 'state/\n.loop-state/\nlevers.tsv\nstubs.sh\ngate/\n' > "$tmp/.gitignore"
  git -C "$tmp" init -q
  git -C "$tmp" -c user.email=a@b -c user.name=t add -A >/dev/null
  git -C "$tmp" -c user.email=a@b -c user.name=t commit -qm init
  echo "$STUBS_BODY" > "$tmp/stubs.sh"
  # la is file-first; lb has earned a holdout boost -> lb must be picked despite coming second.
  printf 'la\ttuning\tvalid-memsafety\tlocal\tfile-first lever\n'  > "$tmp/levers.tsv"
  printf 'lb\ttuning\tvalid-memsafety\tlocal\thigher gen_credit lever\n' >> "$tmp/levers.tsv"
  echo 5 > "$tmp/state/lever.lb.gen_credit"
  SAF_REPO_ROOT="$tmp" SAF_LOOP_STATE="$tmp/state" SAF_GATE_LIB="$tmp/gate" SAF_LOOP_STUBS="$tmp/stubs.sh" \
    LEVERS_FILE="$tmp/levers.tsv" ARM_PROMPT_FILE="$LOOP_DIR/arm_prompt.md" JOURNAL="$tmp/state/journal.md" \
    LOOP_ENV=/dev/null SMOKE_SCENARIO=tuning_nogain \
    GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=a@b GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=a@b \
    bash "$SUP" --once >"$tmp.log" 2>&1
  local picked; picked="$(awk -F' \\| ' '/^- arm/ {print $2}' "$tmp/state/journal.md" 2>/dev/null | tr -d ' ')"
  if [ "$picked" = lb ]; then
    printf '  ok   %-16s picked=%s (higher gen_credit beats file order)\n' gen_credit_priority "$picked"; pass=$((pass+1))
  else
    printf '  FAIL %-16s picked=%s want=lb\n' gen_credit_priority "$picked"; fail=$((fail+1))
    tail -20 "$tmp.log" | sed 's/^/       /'
  fi
  rm -rf "$tmp" 2>/dev/null || true
}

heldout_brake_park() {  # §1a/§2.4 END-TO-END (gen mode): a lever KEEPs on val each arm but the svcomp26
                        # holdout stays flat -> the actionable brake PARKS it as overfit after two misses.
  local tmp; tmp="$(mktemp -d)"
  mkdir -p "$tmp/scripts/loop/lib" "$tmp/tests/benchmarks/svcomp-splits" "$tmp/crates/saf-svcomp/src"
  echo SCORER > "$tmp/scripts/svcomp_split_eval.py"
  cp "$LOOP_DIR"/lib/*.py "$tmp/scripts/loop/lib/"
  echo '{"t":1}' > "$tmp/tests/benchmarks/svcomp-splits/train.jsonl"
  echo '{"t":2}' > "$tmp/tests/benchmarks/svcomp-splits/holdout.jsonl"
  echo '{"t":3}' > "$tmp/tests/benchmarks/svcomp-splits/val.jsonl"
  echo 'pub fn foo(){}' > "$tmp/crates/saf-svcomp/src/lib.rs"
  printf 'state/\n.loop-state/\nlevers.tsv\nstubs.sh\ngate/\n' > "$tmp/.gitignore"
  git -C "$tmp" init -q
  git -C "$tmp" -c user.email=a@b -c user.name=t add -A >/dev/null
  git -C "$tmp" -c user.email=a@b -c user.name=t commit -qm init
  echo "$STUBS_BODY" > "$tmp/stubs.sh"
  printf 'gk\ttuning\tvalid-memsafety\tlocal\tgen-mode lever: val gains that do not reproduce on holdout\n' > "$tmp/levers.tsv"
  SAF_REPO_ROOT="$tmp" SAF_LOOP_STATE="$tmp/state" SAF_GATE_LIB="$tmp/gate" SAF_LOOP_STUBS="$tmp/stubs.sh" \
    LEVERS_FILE="$tmp/levers.tsv" ARM_PROMPT_FILE="$LOOP_DIR/arm_prompt.md" JOURNAL="$tmp/state/journal.md" \
    LOOP_ENV=/dev/null SMOKE_SCENARIO=gen_keep LOOP_GEN_MODE=on MAX_ARMS=6 \
    HELDOUT_EVERY_K=1 HELDOUT_MIN_LIFT=2 HELDOUT_STALL_TO_PARK=2 LEVER_BUDGET=9 ACCUMULATE_BUDGET=9 \
    GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=a@b GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=a@b \
    bash "$SUP" --loop >"$tmp.log" 2>&1
  local keeps parked alerted broke overfit
  keeps="$(grep -c '\*\*KEEP\*\*' "$tmp/state/journal.md" 2>/dev/null || echo 0)"
  [ -e "$tmp/state/lever.gk.parked" ] && parked=yes || parked=no
  [ -e "$tmp/state/ALERT_OVERFIT_gk" ] && alerted=yes || alerted=no
  grep -q 'no active levers remain' "$tmp.log" && broke=yes || broke=no
  grep -q 'PARK-OVERFIT gk' "$tmp/state/journal.md" && overfit=yes || overfit=no
  if [ "$keeps" = 3 ] && [ "$parked" = yes ] && [ "$alerted" = yes ] && [ "$broke" = yes ] && [ "$overfit" = yes ]; then
    printf '  ok   %-16s keeps=%s parked=%s alert=%s overfit-note=%s broke=%s (holdout brake works)\n' \
      heldout_brake_park "$keeps" "$parked" "$alerted" "$overfit" "$broke"; pass=$((pass+1))
  else
    printf '  FAIL %-16s keeps=%s(want 3) parked=%s alert=%s overfit=%s broke=%s\n' \
      heldout_brake_park "$keeps" "$parked" "$alerted" "$overfit" "$broke"; fail=$((fail+1))
    tail -30 "$tmp.log" | sed 's/^/       /'
  fi
  rm -rf "$tmp" 2>/dev/null || true
}

echo "== supervisor smoke (score-gate + accumulate + crosscut + rotation + anti-cheat + lever-budget) =="
scenario score_gain    tuning_gain      tune-smoke  KEEP           # score improved -> kept (win)
scenario accumulate    accumulate       tune-smoke  ACCUMULATE     # score-neutral + useful (tests pass) -> preserved for future arms
scenario no_progress   tuning_nogain    tune-smoke  REVERT         # score-neutral, tests fail -> revert
scenario fp_lowers     fp_lowers_score  tune-smoke  REVERT         # an FP lowers the score -> revert (soundness is IN the score)
scenario cross_reg     crosscut_regression cross-smoke REVERT      # crosscut arm: total up but a family regressed -> revert (no trades)
scenario tamper        tamper           tune-smoke  REJECT_TAMPER  # tamper an immutable -> reject
scenario gate_sabotage gate_sabotage    tune-smoke  REJECT_TAMPER  # worker rewrites gates.py -> caught via pristine copy
scenario holdout       holdout          tune-smoke  REJECT_HOLDOUT # read the holdout -> reject
scenario worker_fail   worker_fail      tune-smoke  WORKER_FAIL    # worker aborted after editing -> wasted-work record still emitted (§1c)
multi_arm_budget                                                  # lever repeats, parks after budget, loop breaks
family_rotation                                                   # lever selection round-robins across families
accumulate_park                                                   # BUG-4: an ACCUMULATE-only lever parks on its own budget
cap_wip_reprime                                                   # BUG-2: reverted capability WIP preserved + next arm primed
reap_swept                                                        # BUG-3: orphan-container sweep invoked on keep + revert
gen_credit_priority                                               # §2.5: holdout-boosted lever wins within its family
heldout_brake_park                                                # §1a/§2.4: gen-mode holdout brake parks a non-reproducing lever

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
