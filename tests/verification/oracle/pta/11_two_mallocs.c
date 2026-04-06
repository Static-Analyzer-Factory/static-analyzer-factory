// PURPOSE: Two mallocs produce distinct heap locations
#include <stdlib.h>

void test() {
    int *p = (int *)malloc(sizeof(int));
    int *q = (int *)malloc(sizeof(int));
}
