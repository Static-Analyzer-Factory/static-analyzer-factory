// PURPOSE: Simplest case — address-of creates a points-to edge
#include <stdlib.h>

void test() {
    int x;
    int *p = &x;
    *p = 42;
}
