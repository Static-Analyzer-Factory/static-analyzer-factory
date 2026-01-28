#include <stdint.h>

// Definite negative shift - Error
int negative_shift(int x) {
    return x << -1;
}

// Definite overflow shift (>= 32 for i32) - Error
int overflow_shift(int x) {
    return x << 32;
}

// Possible negative shift - Warning
int maybe_negative_shift(int x, int y) {
    // y is in [-5, 5] after this check
    if (y >= -5 && y <= 5) {
        return x << y;  // y could be negative
    }
    return 0;
}

// Possible overflow shift - Warning
int maybe_overflow_shift(int x, int y) {
    // y is in [28, 35] after this check
    if (y >= 28 && y <= 35) {
        return x << y;  // y could be >= 32
    }
    return 0;
}

// Safe shift - guarded
int safe_shift(int x, int y) {
    if (y >= 0 && y < 32) {
        return x << y;  // Safe: 0 <= y < 32
    }
    return 0;
}

// Unsigned right shift with large count - Error
uint32_t unsigned_right_shift_overflow(uint32_t x) {
    return x >> 64;  // 64 >= 32
}

// Arithmetic right shift with negative - Error
int32_t arith_shift_negative(int32_t x) {
    return x >> -2;
}

// Safe after narrowing
int safe_after_check(int x, int y) {
    if (y > 0 && y < 31) {
        return x << y;  // Safe: 1 <= y <= 30
    }
    return 1;
}
