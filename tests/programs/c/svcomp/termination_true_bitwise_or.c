/* For y >= 0, `y | 1` is at least 1 and at most y + 1, so x strictly
 * decreases without wrapping. */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  int y = __VERIFIER_nondet_int();
  while (x > 0 && y >= 0) {
    x = x - (y | 1);
  }
  return 0;
}
