// Phase 1: Zero-fact propagation through conditional branches
// The zero fact (Lambda) must reach ALL reachable program points

extern char* getenv(const char*);
extern void sink(const char*);

void test_zero_fact_branches(int cond) {
    char* tainted = getenv("USER");

    if (cond) {
        // Zero fact must reach here
        sink(tainted);  // sink in true branch
    } else {
        // Zero fact must reach here too
        sink(tainted);  // sink in false branch
    }

    // Zero fact must reach merge point
    sink(tainted);  // sink after merge
}

// Test nested conditionals
void test_nested_branches(int a, int b) {
    char* data = getenv("DATA");

    if (a) {
        if (b) {
            sink(data);  // deeply nested - zero must reach
        }
    }
}

// Test loop - zero must reach all iterations conceptually
void test_loop_zero_fact(int n) {
    char* input = getenv("INPUT");

    for (int i = 0; i < n; i++) {
        sink(input);  // zero must reach loop body
    }

    sink(input);  // zero must reach after loop
}
