// PURPOSE: External function calls (libc)
#include <stdlib.h>
#include <string.h>

void test() {
    void *p = malloc(100);
    memset(p, 0, 100);
    free(p);
}
