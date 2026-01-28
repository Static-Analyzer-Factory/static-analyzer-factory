// CWE-416: Use After Free
// Pointer is dereferenced after being freed.
//
// Expected finding: free(p) then *p read (use-after-free)
#include <stdlib.h>

int main(void) {
    int *p = (int *)malloc(sizeof(int));  // SOURCE: heap allocation
    *p = 42;
    free(p);                               // free deallocates memory
    return *p;                             // SINK: use after free
}
