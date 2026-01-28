// Double free on mutually exclusive branches — NOT a double-free.
// Expected: joint feasibility filter should classify this as infeasible (false positive).
#include <stdlib.h>

void exclusive_free(int cond) {
    int *p = (int *)malloc(sizeof(int));
    if (cond)
        free(p);   // sink1: guarded by cond != 0
    else
        free(p);   // sink2: guarded by cond == 0
    // NOT a double-free — mutually exclusive branches
}

int main() {
    exclusive_free(1);
    return 0;
}
