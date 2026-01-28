// Phase 3: Wrapped vs signed interval semantics
// Tests that interval arithmetic respects LLVM IR's modular semantics.
//
// LLVM `add` wraps (modular), `add nsw`/`nuw` have UB on overflow.
// SAF should conservatively handle wraparound.

#include <stdint.h>
#include <limits.h>

void sink_i8(int8_t);
void sink_i16(int16_t);
void sink_i32(int32_t);
void sink_u8(uint8_t);
void sink_u32(uint32_t);

// 8-bit overflow: 127 + 1 should wrap to -128 (or become top)
void i8_overflow(void) {
    int8_t x = 127;
    int8_t y = x + 1;  // wraps to -128 in 2's complement
    sink_i8(y);
}

// Unsigned wrap: 255 + 1 wraps to 0
void u8_wrap(void) {
    uint8_t x = 255;
    uint8_t y = x + 1;  // wraps to 0
    sink_u8(y);
}

// 32-bit overflow
void i32_overflow(void) {
    int32_t x = INT_MAX;
    int32_t y = x + 1;  // wraps or UB
    sink_i32(y);
}

// Multiplication overflow
void mul_overflow(void) {
    int32_t x = 100000;
    int32_t y = x * 100000;  // overflows i32
    sink_i32(y);
}

// Subtraction underflow
void sub_underflow(void) {
    int8_t x = -128;
    int8_t y = x - 1;  // wraps to 127
    sink_i8(y);
}

// Shift overflow
void shift_overflow(void) {
    int32_t x = 1;
    int32_t y = x << 31;  // may overflow
    sink_i32(y);
}

// Division edge case - division by potential zero
int division_by_range(int x) {
    if (x > 0 && x < 10) {
        return 100 / x;  // x in [1,9], safe
    }
    return 0;
}

// Mixing signed and unsigned
void signed_unsigned_mix(void) {
    int32_t s = -1;
    uint32_t u = (uint32_t)s;  // becomes 0xFFFFFFFF
    sink_u32(u);
}
