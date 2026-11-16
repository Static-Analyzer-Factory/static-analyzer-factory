/* The only signed arithmetic runs before an unbounded unsigned loop: once a
 * path enters the loop no overflow check can execute again, so the engine ends
 * it as safe instead of unrolling (sv-benchmarks bitvector/modulus-2). */
extern unsigned int __VERIFIER_nondet_uint(void);
extern void abort(void);
int main(void) {
  unsigned int n = __VERIFIER_nondet_uint();
  unsigned int s = __VERIFIER_nondet_uint();
  if (!(s < 31)) abort();
  unsigned int d = (1 << s) - 1;
  unsigned int m = 0;
  while (n > d) {
    m += n & d;
    n = n >> 1;
  }
  return (int)(m & 1u);
}
