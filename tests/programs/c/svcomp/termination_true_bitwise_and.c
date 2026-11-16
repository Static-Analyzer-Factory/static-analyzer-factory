/* `x & (x - 1)` clears the lowest set bit: for x > 0 both operands are
 * non-negative, so 0 <= x & (x - 1) <= x - 1 and x ranks the positive phase;
 * the negative phase counts up to 0 (sv-benchmarks termination-bwb/and-03). */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  while (x != 0) {
    if (x > 0) {
      x = x & (x - 1);
    } else {
      x++;
    }
  }
  return 0;
}
