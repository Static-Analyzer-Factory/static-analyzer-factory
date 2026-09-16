// plans/213 M2 -- SCCP arithmetic must be normalised to the RESULT WIDTH.
//
// `2147483648u + 2147483648u` is `0` in 32-bit unsigned arithmetic, so `v == 0`
// holds and reach_error IS reachable. Ground-truth FALSE.
//
// `evaluate_binary` folds in `i128` with `wrapping_*`, which does not wrap at
// the VALUE's width: LLVM stores the operand as `i32 -2147483648`, and
// `-2147483648 + -2147483648` in i128 is `-4294967296` -- a value no `i32` can
// hold, whose true value is `0`. The mis-folded constant then decides
// `icmp eq` as false, SCCP marks the live block dead, and the pre-seeded bottom
// is read as a proof.
//
// Same lattice invariant `evaluate_cast` protects: the `i128` in a
// `SccpValue::Constant` is the SIGNED interpretation at the value's OWN width.
extern void abort(void);
void reach_error() {}

int main(void) {
  unsigned int u = 2147483648u;
  unsigned int v = u + u; /* wraps to 0 */
  if (v == 0 && v == 0) {
    reach_error();
    abort();
  }
  return 0;
}
