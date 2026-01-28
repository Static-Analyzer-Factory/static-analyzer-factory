// CWE-476: NULL Pointer Dereference
// malloc return value is used without checking for NULL.
//
// Expected finding: malloc() may return NULL -> dereference without check
#include <stdlib.h>

int main(void) {
    int *p = (int *)malloc(sizeof(int));  // SOURCE: may return NULL
    *p = 7;                                // SINK: dereference without null check
    free(p);
    return 0;
}
