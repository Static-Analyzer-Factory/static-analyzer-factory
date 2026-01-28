// Two pointers that genuinely alias regardless of branch.
// Z3 should confirm aliasing is feasible.
#include <stdlib.h>

int main() {
    int x = 42;
    int *p = &x;
    int *q = &x;
    // Both p and q always point to x — confirmed alias
    return *p + *q;
}
