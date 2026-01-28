/**
 * cspta_nested_wrappers.c — Test CS-PTA with nested wrapper chain.
 *
 * alloc_buffer() calls safe_malloc() which calls malloc().
 * k=1 distinguishes alloc_buffer calls but not safe_malloc calls.
 * k=2 distinguishes both levels.
 */
#include <stdlib.h>

void *safe_malloc(int size) {
    void *p = malloc(size);
    if (!p) {
        return NULL;
    }
    return p;
}

void *alloc_buffer(int size) {
    return safe_malloc(size);
}

int main() {
    void *buf1 = alloc_buffer(64);   // site A
    void *buf2 = alloc_buffer(128);  // site B

    // k=1: buf1 and buf2 may still alias (safe_malloc merged)
    // k=2: buf1 and buf2 are fully separated
    if (buf1) {
        ((char *)buf1)[0] = 'A';
    }
    if (buf2) {
        ((char *)buf2)[0] = 'B';
    }

    free(buf1);
    free(buf2);
    return 0;
}
