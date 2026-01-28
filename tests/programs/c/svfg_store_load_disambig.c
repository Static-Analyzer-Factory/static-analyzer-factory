// SVFG E2E test: Two non-aliasing pointers.
// Store through both, load from one.
// Verifies correct store→load indirect edge.

#include <stdlib.h>

int source(void);
void sink(int);

void test(void) {
    int a;
    int b;
    int *p = &a;
    int *q = &b;

    *p = source();  // store tainted value to a
    *q = 42;        // store clean value to b

    int val = *p;   // load from a — should get tainted value
    sink(val);      // sink
}
