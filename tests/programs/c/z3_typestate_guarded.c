// File opened and closed under the same guard.
// Path-insensitive analysis may report a leak; Z3 should filter it.
#include <stdio.h>

void process(int flag) {
    FILE *f = NULL;
    if (flag) {
        f = fopen("data.txt", "r");
    }
    // ... do work ...
    if (flag) {
        // Same guard: if f was opened, it's closed here
        fclose(f);
    }
    // Without Z3: may report leak (fopen without matching fclose)
    // With Z3: guards are correlated — no leak on feasible paths
}

int main() {
    process(1);
    process(0);
    return 0;
}
