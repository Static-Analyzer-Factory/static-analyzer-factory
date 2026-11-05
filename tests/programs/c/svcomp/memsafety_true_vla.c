/* Variable-length arrays in a loop and a helper, every access in bounds.
   Expected: true. */
unsigned int __VERIFIER_nondet_uint();
static unsigned int sum(unsigned int len, unsigned int *a) {
  unsigned int s = 0;
  for (unsigned int i = 0; i < len; i++) s += a[i];
  return s;
}
int main() {
  unsigned int len = __VERIFIER_nondet_uint();
  if (len == 0 || len > 8) return 0;
  for (int round = 0; round < 2; round++) {
    unsigned int buf[len];
    for (unsigned int i = 0; i < len; i++) buf[i] = i + round;
    if (sum(len, buf) > 1000) return 1;
  }
  return 0;
}
