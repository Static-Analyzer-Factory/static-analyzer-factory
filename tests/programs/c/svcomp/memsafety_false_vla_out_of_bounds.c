/* Writes one element past a variable-length array. Expected:
   false(valid-deref); never true. */
unsigned int __VERIFIER_nondet_uint();
int main() {
  unsigned int len = __VERIFIER_nondet_uint();
  if (len == 0 || len > 8) return 0;
  unsigned int buf[len];
  for (unsigned int i = 0; i <= len; i++) buf[i] = i;
  return (int)buf[0];
}
