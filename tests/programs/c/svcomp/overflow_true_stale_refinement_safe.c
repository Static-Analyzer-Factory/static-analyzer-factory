// plans/213 M3 -- the PRECISION CONTROL for the stale-refinement fix.
//
// Structurally identical to `overflow_false_stale_refinement.c`, but the final
// state is 100 instead of INT_MAX, so `s + 1` genuinely cannot overflow and the
// program is ground-truth TRUE. SAF proves it today and must KEEP proving it.
//
// This is the test that stops the M3 fix from degenerating into "stop refining
// altogether". Removing refinement persistence entirely is trivially sound and
// would cost real recall; this pins that the fix preserves the precision that
// comes from the branch condition itself.
extern void abort(void);

int main(void) {
  int s = 8466;
  int r = 0;
  while (1) {
    if (s == 8466) {
      s = 8496;
    } else if (s == 8496) {
      s = 100;
    } else {
      r = s + 1; /* s == 100 -> no overflow possible */
      return r;
    }
  }
}
