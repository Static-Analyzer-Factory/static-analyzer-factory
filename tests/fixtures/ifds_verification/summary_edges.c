// Phase 2: Summary edge computation and memoization
// Tests that summary edges are correctly computed and reused

extern char* getenv(const char*);
extern void sink(const char*);

// Helper that propagates taint through parameter
char* passthrough(char* data) {
    return data;
}

// Helper that kills taint
char* sanitize(char* data) {
    // Returns clean data (constant)
    return "clean";
}

// Test summary edge reuse: same function called multiple times
void test_summary_reuse(void) {
    char* t1 = getenv("VAR1");
    char* t2 = getenv("VAR2");

    // First call creates summary edge
    char* r1 = passthrough(t1);

    // Second call should reuse summary edge
    char* r2 = passthrough(t2);

    sink(r1);  // Should reach (tainted)
    sink(r2);  // Should reach (tainted)
}

// Test summary with multiple callers
void caller1(void) {
    char* data = getenv("C1");
    char* result = passthrough(data);
    sink(result);  // Tainted
}

void caller2(void) {
    char* data = getenv("C2");
    char* result = passthrough(data);
    sink(result);  // Tainted
}

// Test summary edge with sanitizer
void test_sanitizer_summary(void) {
    char* tainted = getenv("TAINTED");
    char* clean = sanitize(tainted);
    sink(clean);  // Should NOT be tainted (sanitized)
}

// Test interprocedural with multiple levels
char* level2(char* p) {
    return p;
}

char* level1(char* p) {
    return level2(p);
}

void test_multi_level_summary(void) {
    char* data = getenv("DATA");
    char* result = level1(data);
    sink(result);  // Tainted via level1 -> level2 chain
}
