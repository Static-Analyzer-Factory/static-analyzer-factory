// DDA Cache Reuse Test
// Tests: Multiple related queries should benefit from cache.
// Expected: Second query for same pointer should hit cache.

#include <stdlib.h>

int *get_alloc() {
    return (int *)malloc(sizeof(int));
}

int main() {
    int *p = get_alloc();
    int *q = p;  // q aliases p

    // First query: what does p point to?
    *p = 1;

    // Second query: what does q point to? (should hit cache)
    *q = 2;

    int result = *p + *q;
    free(p);
    return result;
}
