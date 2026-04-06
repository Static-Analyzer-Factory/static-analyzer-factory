#include <stdlib.h>
extern int cond(void);
int main(void) {
    void *p = malloc(10);
    if (cond()) free(p);
    return 0;
    // Expected: PARTIALLEAK — free on one path only
}
