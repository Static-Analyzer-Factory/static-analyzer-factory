// Simple memory leak: malloc without free.
// Expected: memory-leak checker should find a finding.
#include <stdlib.h>

int main() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    return *p;
    // LEAK: p is never freed
}
