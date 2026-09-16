// plans/213 §2a, the `!=` spelling. `(ICmpNe, true)` shares the refinement arm with
// `(ICmpEq, false)`, so it must PROVE for the same reason.
extern void abort(void);
void reach_error() {}

int main() {
  int x = 0;
  if (x != 0) {
    reach_error();
    abort();
  }
  return 0;
}
