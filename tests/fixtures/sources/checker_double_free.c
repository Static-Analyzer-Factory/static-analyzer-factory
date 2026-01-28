// Double free: free called twice.
// Expected: double-free checker should find a finding.
#include <stdlib.h>

int main() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    free(p);
    free(p);  // DOUBLE FREE
    return 0;
}
