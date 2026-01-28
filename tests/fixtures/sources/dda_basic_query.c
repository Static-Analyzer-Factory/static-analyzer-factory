// DDA Basic Query Test
// Tests: Single function pointer query with DDA.
// Expected: DDA should find the allocation site for p.

#include <stdlib.h>

int main() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    int x = *p;
    free(p);
    return x;
}
