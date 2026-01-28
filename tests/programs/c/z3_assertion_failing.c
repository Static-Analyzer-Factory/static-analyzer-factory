// Assertions that can fail due to unchecked input.
// Expected: Z3 should find counterexamples.
#include <assert.h>
#include <stdlib.h>

int unchecked_index(int *buf, int idx, int size) {
    // No validation on idx — assertion may fail
    assert(idx >= 0 && idx < size);
    return buf[idx];
}

int main() {
    int buf[10];
    int idx = atoi("42");  // External input, could be anything
    return unchecked_index(buf, idx, 10);
}
