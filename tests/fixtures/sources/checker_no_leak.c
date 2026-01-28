// No leak: malloc + free properly paired.
// Expected: memory-leak checker should NOT find a finding (negative test).
#include <stdlib.h>

int main() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    int val = *p;
    free(p);
    return val;
}
