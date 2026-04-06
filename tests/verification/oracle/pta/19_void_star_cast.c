// PURPOSE: void* casting — pointer identity preserved through cast
#include <stdlib.h>

void test() {
    int x;
    void *v = &x;
    int *p = (int *)v;
    *p = 42;
}
