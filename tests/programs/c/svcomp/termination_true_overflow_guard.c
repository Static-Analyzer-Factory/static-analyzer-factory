/* `z <= INT_MAX - y` is computed in the loop; `y > 0` (another stay condition)
 * proves the subtraction never wraps, so the guard bounds z and `INT_MAX - y - z`
 * ranks the loop (sv-benchmarks termination-restricted-15/a.09_assume). */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  int y = __VERIFIER_nondet_int();
  int z = __VERIFIER_nondet_int();
  while (y > 0 && x >= z && z <= 2147483647 - y) {
    z = z + y;
  }
  return 0;
}
