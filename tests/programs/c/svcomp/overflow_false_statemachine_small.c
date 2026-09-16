// plans/213 -- the NEGATIVE control for the completeness gate. Identical in shape
// to `overflow_false_statemachine.c` but with small state constants, which the
// interval fixpoint handles correctly. The sentinel must abstain because the
// operand is TOP (`n` is nondet) -- reaching the obligation and rejecting it --
// NOT because the completeness gate fired. If this ever starts reporting the
// gate's reason, the gate has become a blanket abstain and the fix is too coarse.
extern int __VERIFIER_nondet_int(void);

int main(void) {
  int s = 0;
  int n = __VERIFIER_nondet_int();
  while (1) {
    if (s == 0) {
      s = 1;
    } else if (s == 1) {
      s = 2;
    } else if (s == 2) {
      int y = n + 1;
      return y;
    } else {
      return 0;
    }
  }
}
