/**
 * cspta_wrapper_dispatch.c — Test CS-PTA with a wrapper function.
 *
 * my_alloc() wrapper is called from two different sites.
 * With k=1, the two allocations are distinguished.
 * Without context sensitivity, they merge to the same points-to set.
 */
#include <stdlib.h>

void *my_alloc(int size) {
    return malloc(size);
}

void use_ptr(void *p) {
    // Sink: use the pointer
    volatile char *cp = (char *)p;
    *cp = 'x';
}

int main() {
    void *a = my_alloc(16);   // call site 1
    void *b = my_alloc(32);   // call site 2

    // With CS-PTA (k=1): a and b point to distinct heap objects
    // Without CS-PTA: a and b may alias (same my_alloc merged)
    use_ptr(a);
    use_ptr(b);

    free(a);
    free(b);
    return 0;
}
