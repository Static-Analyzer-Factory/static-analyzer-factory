// Phase 1: Phi placement in diamond control flow
// Tests: Memory phi placed at merge point after branch

extern int condition(void);
extern void sink(int);

void diamond_phi(void) {
    int x;
    int* p = &x;

    if (condition()) {
        *p = 10;  // Store in true branch
    } else {
        *p = 20;  // Store in false branch
    }

    // Merge point: Memory phi should be placed here
    int result = *p;  // Load at merge - should have phi as reaching def
    sink(result);
}

// Simple diamond with single store per branch
void simple_diamond(int cond) {
    int val;
    if (cond) {
        val = 1;  // def in true branch
    } else {
        val = 2;  // def in false branch
    }
    // phi for val expected here
    sink(val);
}
