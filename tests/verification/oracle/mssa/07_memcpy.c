// PURPOSE: memcpy creates an implicit store
#include <string.h>

void test() {
    int src = 42;
    int dst;
    memcpy(&dst, &src, sizeof(int));
    int y = dst;  // reaching def is memcpy
    (void)y;
}
