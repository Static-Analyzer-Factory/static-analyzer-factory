// valid-memsafety TRUE: the allocation wrapper aborts when malloc returns NULL
// (allowed in SV-COMP 2027) while main still holds the first block. The call
// stack still exists at the abort, so the block is not lost (valid-memtrack).
#include <stdlib.h>

void *safe_malloc(size_t n) {
  void *p = malloc(n);
  if (!p)
    abort();
  return p;
}

int main(void) {
  int *a = safe_malloc(sizeof(int));
  int *b = safe_malloc(sizeof(int));
  *a = 1;
  *b = 2;
  free(b);
  free(a);
  return 0;
}
