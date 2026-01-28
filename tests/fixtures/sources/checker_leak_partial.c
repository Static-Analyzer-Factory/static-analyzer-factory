// Partial memory leak: malloc with conditional free.
// Expected: memory-leak checker should find a partial leak finding.
#include <stdlib.h>
extern int cond(void);
int main(void) {
    void *p = malloc(10);
    if (cond()) free(p);
    return 0;
    // PARTIAL LEAK: p is freed on one path only
}
