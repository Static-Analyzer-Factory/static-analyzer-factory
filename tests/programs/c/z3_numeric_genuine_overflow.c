// Unchecked multiplication that can genuinely overflow.
// Z3 should confirm the integer overflow is feasible.
#include <stdlib.h>

int compute(int a, int b) {
    // No bounds check — multiplication can overflow
    int result = a * b;
    return result;
}

int main() {
    int a = atoi("1000000");
    int b = atoi("1000000");
    return compute(a, b);
}
