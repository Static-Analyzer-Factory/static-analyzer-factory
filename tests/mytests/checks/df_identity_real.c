/**
 * df_identity_real.c — Real double-free through identity function.
 *
 * One allocation passes through id() and is freed twice.
 * The context-insensitive SVFG should still detect this.
 *
 * Expected: 1 double-free finding.
 */

#include <stdlib.h>

int *id(int *p) {
    return p;
}

void process() {
    int *a = (int *)malloc(sizeof(int));

    int *x = id(a);
    int *y = id(a);  /* same allocation */

    free(x);  /* first free */
    free(y);  /* double-free: y == x == a */
}

int main() {
    process();
    return 0;
}
