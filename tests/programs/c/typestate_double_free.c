// Typestate E2E: double free → error state (memory_alloc spec).
#include <stdlib.h>

void double_free(void) {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    free(p);
    free(p);  // Bug: double-free → error state.
}
