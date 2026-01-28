#include <stdint.h>

// Definite division by zero - Error
int definite_div_zero(int x) {
    return x / 0;
}

// Possible division by zero - Warning
int possible_div_zero(int x, int y) {
    // y is in [0, 10] after this check
    if (y >= 0 && y <= 10) {
        return x / y;  // y could be 0
    }
    return 0;
}

// Safe division - guarded
int safe_div(int x, int y) {
    if (y != 0) {
        return x / y;  // Safe: y != 0
    }
    return 0;
}

// Remainder by zero - Error
int rem_by_zero(int x) {
    return x % 0;
}

// Unsigned division by zero
uint32_t unsigned_div_zero(uint32_t x) {
    return x / 0u;
}

// Safe after narrowing
int safe_after_check(int x, int y) {
    if (y > 0) {
        return x / y;  // Safe: y > 0
    }
    return 1;
}

// Unsigned remainder by zero - Error
uint32_t unsigned_rem_zero(uint32_t x) {
    return x % 0u;
}
