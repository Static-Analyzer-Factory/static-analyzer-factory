/* A `goto cleanup` jumps over function-scope declarations (the Hacl-Star
   harness shape), so clang emits no lifetime markers for them; they live
   for the whole function. Expected: true. */
typedef __SIZE_TYPE__ size_t;
extern void *malloc(size_t);
extern void free(void *);
unsigned int __VERIFIER_nondet_uint();
int main() {
  unsigned int *a = 0;
  unsigned int len = __VERIFIER_nondet_uint();
  if (len == 0 || len > 16) goto cleanup;
  size_t bn = (len + 3) / 4;
  a = malloc(bn * sizeof(unsigned int));
  if (!a) goto cleanup;
  unsigned int choice = __VERIFIER_nondet_uint();
  unsigned int *slot = &choice;
  a[0] = *slot;
  a[0] += (unsigned int)bn;
cleanup:
  if (a) free(a);
  return 0;
}
