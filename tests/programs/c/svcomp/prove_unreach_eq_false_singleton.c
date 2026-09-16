// plans/213 §2a. `if (!(x == 0))` is the canonical SV-COMP error guard and is
// exactly what clang lowers to `(ICmpEq, false)`. With `x` already known to be the
// singleton 0, that edge is infeasible, so the error is unreachable and the sound
// verdict is PROVE. Before the `Interval::refine_eq_false` fix, an equal singleton
// fell through to `self.clone()` and this returned ABSTAIN:error-reachable.
extern void abort(void);
void reach_error() {}

int main() {
  int x = 0;
  if (!(x == 0)) {
    reach_error();
    abort();
  }
  return 0;
}
