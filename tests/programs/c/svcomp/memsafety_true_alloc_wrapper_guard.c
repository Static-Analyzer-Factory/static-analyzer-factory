/* Hacl-Star shape: an allocation wrapper with an overflow guard returns
   NULL or a fresh block; the caller checks it. Merging the two returns lost
   the pointer (ms-store-opaque-pointer). Expected: true. */
typedef __SIZE_TYPE__ size_t;
typedef unsigned char uint8_t;
extern void *malloc(size_t);
extern void free(void *);
unsigned char __VERIFIER_nondet_uchar();
static void *alloc_array(size_t n, size_t width) { if (n > 18446744073709551615UL / width) return (void*)0; return malloc(n * width); }
static void fill(uint8_t *p, size_t n) { for (size_t i = 0; i < n; i = i + 1UL) p[i] = __VERIFIER_nondet_uchar(); }
int main() {
  uint8_t *output = (void*)0;
  output = alloc_array((size_t)16, 1UL);
  if (!output) goto cleanup;
  fill(output, (size_t)16);
  if (output) free(output);
  output = (void*)0;
cleanup:;
  return 0;
}
