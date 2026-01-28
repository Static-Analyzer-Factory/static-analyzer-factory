/**
 * cspta_identity_function.c — Test CS-PTA with identity function.
 *
 * id(p) simply returns its argument. Called with different pointers,
 * CI analysis merges the results; CS separates them.
 */
#include <stdlib.h>

void *id(void *p) {
    return p;
}

int main() {
    int x = 10;
    int y = 20;

    void *px = id(&x);  // call site 1: should point to x
    void *py = id(&y);  // call site 2: should point to y

    // With CS-PTA: px points to {x}, py points to {y}
    // Without CS-PTA: px and py both point to {x, y}
    *(int *)px = 100;
    *(int *)py = 200;

    return x + y;
}
