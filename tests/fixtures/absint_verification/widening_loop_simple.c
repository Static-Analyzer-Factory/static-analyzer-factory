// Phase 1: Simple loop widening test
// Tests that widening is applied at loop headers to ensure convergence.
//
// Without widening: x iterates [0], [0,1], [0,2], ... infinitely
// With widening: x jumps to [0, +inf] after bound detection

void sink(int);

void simple_loop(int n) {
    int x = 0;
    while (x < n) {
        // At this point, x should have been widened at loop header
        sink(x);
        x = x + 1;
    }
    // After loop: x >= n (widening + narrowing should refine)
    sink(x);
}

// Nested loops - both loop headers should receive widening
void nested_loops(int m, int n) {
    int outer = 0;
    while (outer < m) {
        int inner = 0;
        while (inner < n) {
            // Both outer and inner should have widened intervals
            sink(outer + inner);
            inner = inner + 1;
        }
        outer = outer + 1;
    }
}

// Multiple exit loop - widening still needed at header
int multiple_exits(int* arr, int len) {
    int sum = 0;
    int i = 0;
    while (i < len) {
        if (arr[i] < 0) {
            return sum;  // early exit
        }
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}

// Counter that would never converge without widening
void unbounded_counter(void) {
    int count = 0;
    while (1) {
        sink(count);
        count = count + 1;
        if (count > 1000000) break;
    }
}
