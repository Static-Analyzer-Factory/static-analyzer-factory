// Phase 6: Integration tests
// Full abstract interpretation pipeline tests combining all components.

#include <stdlib.h>

void sink(int);
void* malloc_wrapper(size_t);
void free_wrapper(void*);

// Buffer overflow detection test
// Abstract interpretation should detect potential out-of-bounds access
void buffer_overflow_test(int index) {
    int buffer[100];

    // Unknown index - should be flagged as potential overflow
    buffer[index] = 42;  // index in [-inf, +inf] initially

    // Bounded index - should be safe
    if (index >= 0 && index < 100) {
        buffer[index] = 100;  // index in [0, 99], safe
    }
}

// Integer overflow detection test
void integer_overflow_test(int a, int b) {
    // Unknown values - potential overflow
    int sum = a + b;  // could overflow
    sink(sum);

    // Bounded values - safe
    if (a >= 0 && a <= 1000 && b >= 0 && b <= 1000) {
        int safe_sum = a + b;  // max 2000, fits in i32
        sink(safe_sum);
    }
}

// Loop with known bounds - should prove safety
void safe_loop_access(void) {
    int arr[10];
    for (int i = 0; i < 10; i++) {
        // i in [0, 9] from loop condition
        // This access should be provably safe
        arr[i] = i * 2;
    }
}

// Determinism test - same computation multiple times
int deterministic_computation(int n) {
    int sum = 0;
    for (int i = 0; i < n; i++) {
        sum += i;
    }
    return sum;
}

// Complex control flow with arithmetic
int complex_arithmetic(int x, int y, int mode) {
    int result = 0;

    if (mode == 0) {
        // Addition path
        result = x + y;
    } else if (mode == 1) {
        // Multiplication path
        result = x * y;
    } else if (mode == 2) {
        // Division path (guarded)
        if (y != 0) {
            result = x / y;
        }
    } else {
        // Subtraction path
        result = x - y;
    }

    return result;
}

// Cascading bounds refinement
void cascading_bounds(int x) {
    // Start: x in [-inf, +inf]
    if (x > -100) {
        // x in [-99, +inf]
        if (x < 100) {
            // x in [-99, 99]
            if (x >= 0) {
                // x in [0, 99]
                sink(x);
            }
        }
    }
}

// Loop with multiple counters
void multi_counter_loop(int n) {
    int a = 0;
    int b = n;
    int c = 0;

    while (a < n && b > 0) {
        a = a + 1;
        b = b - 1;
        c = a + b;  // c should stay around n
        sink(c);
    }
}

// Entry point for full module test
int main(void) {
    buffer_overflow_test(50);
    integer_overflow_test(100, 200);
    safe_loop_access();

    int d = deterministic_computation(10);
    sink(d);

    int r = complex_arithmetic(10, 5, 0);
    sink(r);

    cascading_bounds(42);
    multi_counter_loop(100);

    return 0;
}
