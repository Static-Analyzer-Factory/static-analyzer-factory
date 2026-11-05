/* One branch frees the block, the join then writes to it. The two sides
   disagree on the block's status; the prover must never say true.
   Expected: false(valid-deref). */
typedef unsigned long size_t;
extern void *malloc(size_t);
extern void free(void *);
int __VERIFIER_nondet_int();
int main() {
  int *p = malloc(8);
  if (!p) return 0;
  int *q = p;
  if (__VERIFIER_nondet_int()) { free(p); q = (void*)0; }
  if (__VERIFIER_nondet_int()) q = p;
  if (q) q[1] = 2;
  if (q == p && q) free(q);
  return 0;
}
