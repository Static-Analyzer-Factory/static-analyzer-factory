// Use-after-free vulnerability: memory freed then read through an alias.
//
// The SVFG tracks value flow through memory stores and loads, making this
// pattern visible as a reachable path from free() to the read site.

#include <stdlib.h>

int source(void);
void sink(int);

void test(void) {
    int *buf = (int *)malloc(sizeof(int));
    int *alias = buf;          // alias points to same allocation

    int tainted = source();    // source of taint
    *buf = tainted;            // store tainted value to heap

    free(buf);                 // free the allocation

    int leaked = *alias;       // use-after-free: read via alias
    sink(leaked);              // tainted data reaches sink
}

int main(void) {
    test();
    return 0;
}
