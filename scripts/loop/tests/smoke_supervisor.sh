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
    *:after.json)                confirmed=100 ;;
  esac
  [ -z "$per" ] && per="{\"valid-memsafety\":{\"confirmed\":$confirmed}}"
  printf "{\"confirmed_score\":%s,\"false_alarms\":%s,\"wrong_true\":%s,\"raw_score\":%s,\"max_score\":1000,\"per_property\":%s}\n" \
    "$confirmed" "$fp" "$wt" "$confirmed" "$per" > "$out"
  [ -n "$pt" ] && : > "$pt"
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
  return 0
}
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
  if [ "$got" = "$expect" ] && [ "$dirty" = "0" ] && [[ "$br" == auto/loop-* ]]; then
    printf '  ok   %-16s scenario=%-14s -> %-13s [clean, on %s]\n' "$name" "$sc" "$got" "$br"; pass=$((pass+1))
  else
    printf '  FAIL %-16s scenario=%-14s expected=%s got=%s dirty=%s branch=%s\n' "$name" "$sc" "$expect" "${got:-<none>}" "$dirty" "$br"; fail=$((fail+1))
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

echo "== supervisor smoke (score-gate + accumulate + crosscut + rotation + anti-cheat + lever-budget) =="
scenario score_gain    tuning_gain      tune-smoke  KEEP           # score improved -> kept (win)
scenario accumulate    accumulate       tune-smoke  ACCUMULATE     # score-neutral + useful (tests pass) -> preserved for future arms
scenario no_progress   tuning_nogain    tune-smoke  REVERT         # score-neutral, tests fail -> revert
scenario fp_lowers     fp_lowers_score  tune-smoke  REVERT         # an FP lowers the score -> revert (soundness is IN the score)
scenario cross_reg     crosscut_regression cross-smoke REVERT      # crosscut arm: total up but a family regressed -> revert (no trades)
scenario tamper        tamper           tune-smoke  REJECT_TAMPER  # tamper an immutable -> reject
scenario gate_sabotage gate_sabotage    tune-smoke  REJECT_TAMPER  # worker rewrites gates.py -> caught via pristine copy
scenario holdout       holdout          tune-smoke  REJECT_HOLDOUT # read the holdout -> reject
multi_arm_budget                                                  # lever repeats, parks after budget, loop breaks
family_rotation                                                   # lever selection round-robins across families

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
