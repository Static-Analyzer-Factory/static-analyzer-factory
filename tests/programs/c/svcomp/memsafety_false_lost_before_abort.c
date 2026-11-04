// valid-memsafety FALSE (valid-memtrack): on the nondet branch the only pointer
// to the block is overwritten before the program aborts, so the block is lost.
// The overwritten SSA value is still bound in the prover's state but dead: it
// must not count as a stack root.
#include <stdlib.h>
extern int __VERIFIER_nondet_int(void);

int main(void) {
  int *a = malloc(sizeof(int));
  if (!a)
    return 0;
  if (__VERIFIER_nondet_int()) {
    a = 0;
    abort();
  }
  free(a);
  return 0;
}
