// Phase 2: Threshold widening tests
// Tests that threshold widening improves precision over standard widening.
//
// With standard widening: x < 100 → x in [0, +inf]
// With threshold widening: x < 100 → x in [0, 100] (if 100 is a threshold)

void sink(int);

// Explicit loop bound - threshold should be extracted from ICmp
void bounded_loop(void) {
    int x = 0;
    while (x < 100) {
        // Threshold widening: x in [0, 99] inside loop
        sink(x);
        x = x + 1;
    }
    // After loop: x >= 100, ideally x == 100
    sink(x);
}

// Buffer size constant - should be a threshold
void buffer_iteration(void) {
    int buffer[256];
    int i = 0;
    while (i < 256) {
        // Threshold widening: i in [0, 255]
        buffer[i] = 0;
        i = i + 1;
    }
}

// Multiple thresholds in same function
void multi_threshold(int mode) {
    int x = 0;
    if (mode == 0) {
        while (x < 10) {
            sink(x);
            x = x + 1;
        }
    } else if (mode == 1) {
        while (x < 100) {
            sink(x);
            x = x + 1;
        }
    } else {
        while (x < 1000) {
            sink(x);
            x = x + 1;
        }
    }
}

// Threshold from global constant
#define MAX_ITEMS 1024

void global_threshold(void) {
    int items = 0;
    while (items < MAX_ITEMS) {
        sink(items);
        items = items + 1;
    }
}

// Off-by-one variants - threshold ± 1 should also be thresholds
void off_by_one(void) {
    int x = 0;
    while (x <= 99) {  // uses 99, threshold extraction should include 100
        sink(x);
        x = x + 1;
    }
}
