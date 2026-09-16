// plans/213 M3 -- STALE REFINEMENT regression. `no-overflow` FALSE.
//
// The dispatch chain drives `s` to INT_MAX, and the final `else` computes
// `s + 1`, which overflows. The overflow IS reachable, so the only correct
// sentinel answers are ABSTAIN or a FALSE verdict -- never PROVE.
//
// SAF used to PROVE it, and NOT because the block was unvisited (the
// completeness gate does not catch this one). `collect_refinements` cached the
// INTERVAL the predicate `s != 8496` produced on iteration 1, and
// `apply_refinements` re-imposed it by MEET on every later iteration. The true
// interval `[8467, 2147483647]` contains INT_MAX; the stale `[8467, 8495]`
// clamps it, and `s + 1` then "proves" in-bounds. An under-approximated state
// is a wrong TRUE: -32, uncapped by dedup.
//
// A branch predicate is an edge invariant. The interval it produced GIVEN one
// iteration's pre-state is NOT.
extern void abort(void);

int main(void) {
  int s = 8466;
  int r = 0;
  while (1) {
    if (s == 8466) {
      s = 8496;
    } else if (s == 8496) {
      s = 2147483647;
    } else {
      r = s + 1; /* reachable with s == INT_MAX -> signed overflow */
      return r;
    }
  }
}
