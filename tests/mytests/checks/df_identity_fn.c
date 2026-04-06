/**
 * df_identity_fn.c — False-positive test for context-insensitive SVFG.
 *
 * Two independent allocations pass through the same identity function id().
 * Each is freed exactly once. A context-insensitive SVFG merges the flows
 * through id()'s parameter, potentially causing the checker to see spurious
 * aliasing and report a false-positive double-free.
 *
 * Expected: 0 double-free findings (no bug exists).
 * Context-insensitive bug: may report 1 false-positive double-free.
 */

#include <stdlib.h>

int *id(int *p) {
    return p;
}

void process() {
    int *a = (int *)malloc(sizeof(int));
    int *b = (int *)malloc(sizeof(int));

    int *x = id(a);
    int *y = id(b);

    free(x);  /* frees a — correct */
    free(y);  /* frees b — correct */
}

int main() {
    process();
    return 0;
}
