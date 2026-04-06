// PURPOSE: Function returns a pointer to local-ish allocation
#include <stdlib.h>

int *create() {
    int *p = (int *)malloc(sizeof(int));
    return p;
}

void test() {
    int *q = create();
    *q = 42;
    free(q);
}
