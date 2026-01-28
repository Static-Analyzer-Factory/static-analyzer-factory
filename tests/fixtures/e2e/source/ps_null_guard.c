// Path-sensitive test: null guard prevents null dereference
// Path-insensitive checker may report null-deref finding.
// Path-sensitive checker should recognize the null guard filters the null path.
#include <stdlib.h>

void sink(int x);

int main() {
    int *p = (int *)malloc(sizeof(int));
    if (p == NULL) {
        // Null path: return early, no dereference
        return 1;
    }
    // Non-null path: safe dereference
    *p = 42;
    sink(*p);
    free(p);
    return 0;
}
