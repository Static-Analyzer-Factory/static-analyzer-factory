/**
 * df_interproc_mem.c — Double-free through interprocedural memory flow.
 *
 * The pointer p is stored into *q inside store(), then loaded back
 * as r in process(). Both p and r alias the same malloc, so
 * free(p) + free(r) is a double-free.
 *
 * This tests whether SAF can track value flow through memory
 * across function boundaries (interprocedural MSSA).
 *
 * Expected: 1 double-free finding.
 */

#include <stdlib.h>

void store(int *p, int **q) {
    *q = p;
}

void process() {
    int *p = (int *)malloc(sizeof(int));
    int **q = (int **)malloc(sizeof(int *));
    *p = 42;

    store(p, q);

    free(p);
    int *r = *q;
    free(r);  /* double-free: r == p */
}

int main() {
    process();
    return 0;
}
