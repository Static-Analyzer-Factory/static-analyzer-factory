// Phase 6: Complex interprocedural scenarios
// Comprehensive tests for IFDS/IDE algorithm verification

extern char* getenv(const char*);
extern void sink(const char*);

// ============================================
// Recursive functions - ensure termination
// ============================================

// Self-recursive taint propagation
char* recursive_pass(char* p, int depth) {
    if (depth <= 0) {
        return p;
    }
    return recursive_pass(p, depth - 1);
}

void test_recursive_taint(void) {
    char* data = getenv("RECURSIVE");
    char* result = recursive_pass(data, 5);
    sink(result);  // Should be tainted
}

// Mutual recursion
char* ping(char* p, int n);
char* pong(char* p, int n);

char* ping(char* p, int n) {
    if (n <= 0) return p;
    return pong(p, n - 1);
}

char* pong(char* p, int n) {
    if (n <= 0) return p;
    return ping(p, n - 1);
}

void test_mutual_recursion(void) {
    char* data = getenv("MUTUAL");
    char* result = ping(data, 4);
    sink(result);  // Should be tainted
}

// ============================================
// Multiple return paths
// ============================================

char* maybe_taint(int cond) {
    if (cond) {
        return getenv("TAINTED");
    } else {
        return "clean";
    }
}

void test_multiple_returns(void) {
    // With cond=1, result is tainted
    // With cond=0, result is clean
    // Sound analysis must assume tainted
    char* result = maybe_taint(1);
    sink(result);
}

// ============================================
// Aliasing through pointers
// ============================================

void alias_sink(char** pp) {
    sink(*pp);
}

void test_pointer_alias_taint(void) {
    char* data = getenv("ALIAS");
    char** ptr = &data;
    alias_sink(ptr);  // Indirect sink
}

// ============================================
// Struct field propagation
// ============================================

struct Data {
    char* field1;
    char* field2;
};

void set_field(struct Data* d, char* val) {
    d->field1 = val;
}

char* get_field(struct Data* d) {
    return d->field1;
}

void test_struct_taint_flow(void) {
    struct Data d;
    char* tainted = getenv("STRUCT");
    set_field(&d, tainted);
    char* result = get_field(&d);
    sink(result);  // Should be tainted through struct
}

// ============================================
// Call chain with multiple entry points
// ============================================

void inner_sink(char* p) {
    sink(p);
}

void middle(char* p) {
    inner_sink(p);
}

void outer_a(void) {
    char* a = getenv("A");
    middle(a);
}

void outer_b(void) {
    char* b = getenv("B");
    middle(b);
}

// Both outer_a and outer_b should report taint at sink

// ============================================
// Diamond control flow
// ============================================

void test_diamond_flow(int c1, int c2) {
    char* data = getenv("DIAMOND");

    if (c1) {
        // Path 1
        if (c2) {
            // Path 1a
        } else {
            // Path 1b
        }
    } else {
        // Path 2
    }

    // All paths merge here - zero fact must reach
    sink(data);
}

// ============================================
// Early return with taint
// ============================================

char* early_return(int cond) {
    char* data = getenv("EARLY");
    if (cond) {
        return data;  // Early return with taint
    }
    return "default";
}

void test_early_return(void) {
    char* result = early_return(1);
    sink(result);  // Tainted from early return path
}
