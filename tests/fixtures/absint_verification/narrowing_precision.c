// Phase 4: Narrowing iteration tests
// Tests that narrowing recovers precision lost during widening.
//
// After widening: x in [0, +inf]
// After narrowing: x in [0, 100] at loop exit (if x < 100 was condition)

void sink(int);

// Basic narrowing: should recover precise bound after widening
void basic_narrowing(void) {
    int x = 0;
    while (x < 100) {
        x = x + 1;
    }
    // After loop: x >= 100
    // After narrowing: ideally x in [100, 100] or close
    sink(x);
}

// Multiple values to narrow
void multi_value_narrowing(void) {
    int a = 0;
    int b = 10;
    while (a < 50 && b < 60) {
        a = a + 1;
        b = b + 1;
    }
    // Both a and b should be narrowed
    sink(a);
    sink(b);
}

// Nested loop narrowing
void nested_narrowing(void) {
    int outer = 0;
    while (outer < 10) {
        int inner = 0;
        while (inner < 5) {
            inner = inner + 1;
        }
        // inner should be narrowed to [5, 5] ideally
        sink(inner);
        outer = outer + 1;
    }
    sink(outer);
}

// Branch condition refinement + narrowing
void branch_narrowing(int input) {
    int x = input;
    if (x > 0) {
        if (x < 100) {
            // x in [1, 99] from branch conditions
            sink(x);
        }
    }
}

// Loop with condition-based narrowing
void condition_narrowing(void) {
    int i = 0;
    int sum = 0;
    while (i < 10) {
        sum = sum + i;
        i = i + 1;
    }
    // i should narrow to [10, 10]
    // sum should narrow based on accumulated additions
    if (i == 10) {
        sink(sum);  // confirm i was narrowed correctly
    }
}

// Interleaved widening/narrowing across blocks
void interleaved_narrowing(int n) {
    int x = 0;
    int y = 100;

    while (x < n) {
        x = x + 1;
        y = y - 1;
    }

    // Both x and y should benefit from narrowing
    sink(x);  // x >= n
    sink(y);  // y == 100 - x (approximately)
}
