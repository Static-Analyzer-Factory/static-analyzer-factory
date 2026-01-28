// Use-after-free: free then dereference.
// Expected: use-after-free checker should find a finding.
#include <stdlib.h>

int main() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    free(p);
    return *p;  // UAF: use after free
}
