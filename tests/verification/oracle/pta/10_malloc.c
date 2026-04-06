// PURPOSE: Heap allocation — malloc returns a fresh location
#include <stdlib.h>

void test() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    free(p);
}
