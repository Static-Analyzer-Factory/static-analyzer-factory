// GUARD for prove_false_alloc_may_fail.c: the program aborts when malloc returns
// NULL, then uses the block. Both allocation outcomes are safe, so the exhaustive
// prover must still prove the error unreachable.
extern void __assert_fail(const char *, const char *, unsigned int, const char *);
void reach_error() { __assert_fail("0", "prove_unreach_alloc_null_guarded.c", 5, "reach_error"); }

#include <stdlib.h>

int main() {
  int *p = malloc(sizeof(int));
  if (!p) {
    abort();
  }
  *p = 5;
  if (*p != 5) {
    reach_error();
  }
  return 0;
}
