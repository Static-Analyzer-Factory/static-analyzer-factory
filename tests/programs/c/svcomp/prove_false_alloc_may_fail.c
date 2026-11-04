// Reduced from SV-COMP 2027 ldv-regression/rule57_ebda_blast_2.i (expected
// unreach-call FALSE). Since SV-COMP 2027, calloc may return NULL; the error is
// reachable only on that path. SAF's exhaustive prover modelled allocation as
// never failing and answered `true`.
extern void __assert_fail(const char *, const char *, unsigned int, const char *);
void reach_error() { __assert_fail("0", "prove_false_alloc_may_fail.c", 6, "reach_error"); }

#include <stdlib.h>

int used = 0;

int main() {
  int *p = calloc(1, sizeof(int));
  if (p) {
    used = 1;
  }
  if (!used) {
    reach_error();
  }
  return 0;
}
