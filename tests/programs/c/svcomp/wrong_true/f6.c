/* Does not terminate for 0 < x < 100, y == 0: x | 0 == x. */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  int y = __VERIFIER_nondet_int();
  while (x > 0 && x < 100) {
    x = x | y;
  }
  return 0;
}
