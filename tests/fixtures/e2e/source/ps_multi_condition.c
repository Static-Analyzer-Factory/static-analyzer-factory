// Path-sensitive test: multi-condition program where the bug IS feasible
// Multiple conditions exist, but the UAF path is genuinely reachable.
// Both modes should report the finding.
#include <stdlib.h>

void use_ptr(int *p);

int main() {
    int *p = (int *)malloc(sizeof(int));
    if (!p) return 1;
    *p = 42;

    int x = *p;
    free(p);

    if (x > 0) {
        // This path IS feasible (x was 42 > 0)
        use_ptr(p);  // UAF: p was freed, and we're using it
    }
    return 0;
}
