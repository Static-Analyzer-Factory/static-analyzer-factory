// Assertions that are always true given path constraints.
// Expected: Z3 assertion prover should prove them.
#include <assert.h>

int validated_add(int a, int b) {
    if (a < 0 || a > 100) return -1;
    if (b < 0 || b > 100) return -1;
    int sum = a + b;
    // sum is in [0, 200], always < 1000
    assert(sum < 1000);
    return sum;
}

int main() {
    return validated_add(10, 20);
}
