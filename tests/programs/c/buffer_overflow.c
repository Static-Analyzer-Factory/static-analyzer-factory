// CWE-122: Heap Buffer Overflow
// A small heap buffer is written beyond its bounds via a loop.
//
// Expected finding: write past allocated size (buf[i] where i >= 5)
#include <stdlib.h>

int main(void) {
    int *buf = (int *)malloc(5 * sizeof(int));  // SOURCE: allocates 5 ints
    for (int i = 0; i < 10; i++) {
        buf[i] = i;                              // SINK: overflow when i >= 5
    }
    free(buf);
    return 0;
}
