// Tests SCCP dead branch elimination.
// The else branch is dead because GLOBAL_CONST == 5 always.
#include <stdlib.h>

static const int GLOBAL_CONST = 5;

int main() {
    int *p = (int *)malloc(sizeof(int));
    if (GLOBAL_CONST == 5) {
        *p = 42;  // This is the only reachable path
        free(p);
    } else {
        // DEAD CODE — SCCP should prove this unreachable
        free(p);
        *p = 99;  // Use-after-free, but in dead code
    }
    return 0;
}
