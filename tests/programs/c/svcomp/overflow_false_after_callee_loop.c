/* The callee's loop is free of checks, but its caller overflows after the
 * call returns: the frame's continuation keeps the path alive. */
extern unsigned int __VERIFIER_nondet_uint(void);
extern int __VERIFIER_nondet_int(void);
unsigned int spin(unsigned int n) {
  while (n > 1u) {
    n = n >> 1;
  }
  return n;
}
int main(void) {
  unsigned int n = spin(__VERIFIER_nondet_uint());
  int x = __VERIFIER_nondet_int();
  return x * 2 + (int)n;
}
