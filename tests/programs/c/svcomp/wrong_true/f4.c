/* No stay condition bounds y: with y == 0 the loop never ends. The guard's
 * `2147483647 - y` may wrap for y < 0, so it must not be read as an integer bound. */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  int y = __VERIFIER_nondet_int();
  int z = __VERIFIER_nondet_int();
  while (x >= z && z <= 2147483647 - y) {
    z = z + y;
  }
  return 0;
}
