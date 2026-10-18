// k-induction regression (2026-10-18, btor2c DAIO): a counter that passes one
// more constant bound per Houdini consecution round, so the candidate set is
// still shrinking when the rounds run out. That set is not an invariant; the
// error at x > 24 is reachable after 25 iterations.
extern void __assert_fail(const char *, const char *, unsigned int, const char *);
void reach_error(void) { __assert_fail("0", "slow_counter.c", 5, "reach_error"); }
extern unsigned char __VERIFIER_nondet_uchar(void);
int main(void) {
  unsigned int x = 0, y = 0;
  for (;;) {
    unsigned char in = __VERIFIER_nondet_uchar();
    if (x > 24) reach_error();
    if (x == 2 || x == 3 || x == 4 || x == 5 || x == 6 || x == 7 || x == 8 || x == 9 || x == 10 || x == 11 ||
        x == 12 || x == 13 || x == 14 || x == 15 || x == 16 || x == 17 || x == 18 || x == 19 || x == 20 ||
        x == 21 || x == 22 || x == 23)
      y = in;
    if (in < 2 && x < 200) x = x + 1;
  }
  return 0;
}
