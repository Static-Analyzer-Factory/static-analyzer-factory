/* Unsigned shift wraps (defined): u stays nonzero forever. */
extern unsigned __VERIFIER_nondet_uint(void);
int main(void) {
  unsigned u = __VERIFIER_nondet_uint();
  while (u != 0) {
    u = (u << 1) | 1u;
  }
  return 0;
}
