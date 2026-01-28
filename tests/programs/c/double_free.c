// CWE-415: Double Free
// Same pointer is freed twice, causing undefined behavior.
//
// Expected finding: free(p) called twice on same allocation
#include <stdlib.h>

int main(void) {
    int *p = (int *)malloc(sizeof(int));  // SOURCE: heap allocation
    *p = 10;
    free(p);                               // first free (valid)
    free(p);                               // SINK: double free
    return 0;
}
