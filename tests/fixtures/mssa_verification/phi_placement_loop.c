// Phase 1: Phi placement in loop structures
// Tests: Memory phi placed at loop headers

extern void sink(int);

void loop_phi(int n) {
    int x = 0;
    int* p = &x;

    // Loop header: phi should merge initial value and loop-back value
    for (int i = 0; i < n; i++) {
        int prev = *p;  // Load before store - sees phi
        *p = prev + 1;  // Store updates memory
    }

    // After loop: should see final value
    int result = *p;
    sink(result);
}

// Nested loops test
void nested_loop_phi(int m, int n) {
    int arr[10];
    int* p = &arr[0];

    for (int i = 0; i < m; i++) {
        // Outer loop header: phi for p's target
        for (int j = 0; j < n; j++) {
            // Inner loop header: another phi
            *p = i * n + j;
        }
    }

    int result = *p;
    sink(result);
}

// While loop with multiple exits
void while_loop_phi(int cond, int limit) {
    int counter;
    int* p = &counter;
    *p = 0;  // Initial store

    while (cond && *p < limit) {
        *p = *p + 1;  // Store in loop body
    }

    sink(*p);  // Load after loop
}
