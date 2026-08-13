// no-overflow FALSE: an UNCONDITIONAL signed addition overflow (INT_MAX + 1). The
// `volatile` read forces a runtime add, so UBSan instruments and traps it regardless
// of input -> false(no-overflow) at the first mini-fuzz constant (0).
#include <limits.h>
int main(void) {
    volatile int x = INT_MAX;
    return x + 1;
}
