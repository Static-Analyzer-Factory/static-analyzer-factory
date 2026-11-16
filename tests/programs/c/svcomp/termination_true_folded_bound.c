/* At -O0 the bound `m - 1` is recomputed in the loop header as
 * `sub 0xffffffff, 1`: a constant-operand fold, exact as the unsigned value
 * 0xfffffffe, so the counter loop is ranked (sv-benchmarks loop-simple/deep-nested). */
int main(void) {
  unsigned a, m;
  m = 0xffffffffu;
  for (a = 0; a < m - 1; ++a) {
  }
  return 0;
}
