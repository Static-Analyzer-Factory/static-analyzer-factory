/* `~x` is `-x - 1`: once x > y the loop sets y negative and exits, otherwise
 * x decreases, so x + y ranks it (sv-benchmarks termination-bwb/not-03). */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  int y = __VERIFIER_nondet_int();
  while (x > 0 && y > 0) {
    if (x > y) {
      y = ~x;
    } else {
      x = x - 1;
    }
  }
  return 0;
}
