// PURPOSE: Conditional assignment creates may-alias via phi node
#include <stdlib.h>

int rand_bool(void);

void test() {
    int x, y;
    int *p;
    if (rand_bool()) {
        p = &x;
    } else {
        p = &y;
    }
    *p = 42;
}
