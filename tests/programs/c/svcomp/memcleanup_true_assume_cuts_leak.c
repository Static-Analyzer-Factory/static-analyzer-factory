// valid-memcleanup TRUE: the only unfreed block is allocated on a path that a
// false assumption cuts, which is no execution of the program.
extern int __VERIFIER_nondet_int(void);
extern void __VERIFIER_assume(int);
#include <stdlib.h>

int main(void) {
  int x = __VERIFIER_nondet_int();
  int *p = malloc(sizeof(int));
  if (!p) {
    return 0;
  }
  if (x == 0) {
    __VERIFIER_assume(0);
  }
  free(p);
  return 0;
}
