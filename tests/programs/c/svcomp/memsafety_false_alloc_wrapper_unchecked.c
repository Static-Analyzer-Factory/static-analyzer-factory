/* The wrapper's NULL (allocation failure or overflow guard) reaches an
   unchecked store. Expected: false(valid-deref); the prover must never say true. */
typedef __SIZE_TYPE__ size_t;
extern void *malloc(size_t);
extern void free(void *);
unsigned long __VERIFIER_nondet_ulong();
static void *alloc_array(size_t n, size_t width) {
  if (n > 18446744073709551615UL / width) return (void*)0;
  return malloc(n * width);
}
int main() {
  size_t n = __VERIFIER_nondet_ulong();
  if (n == 0 || n > 64) return 0;
  int *p = alloc_array(n, sizeof(int));
  p[0] = 1;
  free(p);
  return 0;
}
