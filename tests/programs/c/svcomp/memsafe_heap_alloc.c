/* plans/214 Movement 2 -- pins OBLIGATION 1 (heap-free).
 *
 * The prover discharges `valid-free` and `valid-memtrack` VACUOUSLY: it proves
 * only programs that never allocate, so there is nothing to free twice and
 * nothing to leak. That argument holds exactly as long as no allocator is
 * admitted into the reachable universe.
 *
 * Here the object is leaked and the store through `p` is to memory the prover
 * models not at all. Admitting `malloc` would leave both undetected. */
extern void *malloc(unsigned long);

int *p;

int main(void) {
  p = malloc(sizeof(int) * 4);
  *p = 1;
  return 0;
}
