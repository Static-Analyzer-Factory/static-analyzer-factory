/* Does not terminate for x > 0, y == -1: x & -1 == x. */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  int y = __VERIFIER_nondet_int();
  while (x > 0) {
    x = x & y;
  }
  return 0;
}
