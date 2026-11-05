/* A pointer into a variable-length array is used after the array's scope
   ended. Expected: false(valid-deref); never true. */
unsigned int __VERIFIER_nondet_uint();
int main() {
  unsigned int len = __VERIFIER_nondet_uint();
  if (len == 0 || len > 8) return 0;
  unsigned int *p;
  {
    unsigned int buf[len];
    buf[0] = 1;
    p = buf;
  }
  return (int)p[0];
}
