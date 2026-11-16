/* A signed overflow AFTER an unbounded unsigned loop must still be found:
 * the loop's continuation reaches the check. */
extern unsigned int __VERIFIER_nondet_uint(void);
extern int __VERIFIER_nondet_int(void);
int main(void) {
  unsigned int n = __VERIFIER_nondet_uint();
  while (n > 1u) {
    n = n >> 1;
  }
  int x = __VERIFIER_nondet_int();
  return x + 1;
}
