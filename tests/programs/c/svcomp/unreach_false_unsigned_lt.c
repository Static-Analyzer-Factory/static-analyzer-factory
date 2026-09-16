// plans/213 §2b. Reduced from `bitvector-regression/implicitunsignedconversion-1`,
// a ground-truth-FALSE task. The usual arithmetic conversions make `minus_one` into
// 0xFFFFFFFF, so `plus_one < minus_one` is TRUE and reach_error IS reachable.
// The interval domain carries no signedness tag, so before the fix the UNSIGNED
// comparison was refined with the SIGNED refiners: `[1,1] < [-1,-1]` is bottom,
// which declared this reachable branch infeasible and PROVEd a FALSE task.
extern void abort(void);
void reach_error() {}

int main() {
  unsigned int plus_one = 1;
  int minus_one = -1;

  if (plus_one < minus_one) {
    reach_error();
    abort();
  }
  return 0;
}
