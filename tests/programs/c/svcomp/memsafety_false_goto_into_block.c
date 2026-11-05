/* A `goto` into a nested block skips the declaration of `x` (no lifetime
   markers); `x` still dies at the end of the block, and the read through
   `p` after it is a dangling access. Expected: false(valid-deref); never true. */
int __VERIFIER_nondet_int();
int main() {
  int *p = 0;
  if (__VERIFIER_nondet_int()) goto inside;
  return 0;
  {
    int x;
  inside:
    x = 1;
    p = &x;
    *p = 2;
  }
  return *p;
}
