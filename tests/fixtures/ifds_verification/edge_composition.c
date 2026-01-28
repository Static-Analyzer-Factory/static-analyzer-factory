// Phase 3 & 4: Edge function composition and jump functions
// Tests for IDE solver edge function composition order

extern char* getenv(const char*);
extern void sink(const char*);
extern int open(const char*, int);
extern int close(int);
extern int read(int, void*, int);
extern int write(int, const void*, int);

// Test basic typestate transitions
void test_file_open_close(void) {
    int fd = open("test.txt", 0);  // opened state
    // ... operations ...
    close(fd);  // closed state
}

// Test error state detection: use after close
void test_use_after_close(void) {
    int fd = open("data.txt", 0);
    close(fd);

    // ERROR: reading closed file - edge function should detect
    char buf[100];
    read(fd, buf, 100);  // Error state: use-after-close
}

// Test composition through multiple calls
// call_ef . return_ef composition
char* transform1(char* p) {
    return p;  // Identity
}

char* transform2(char* p) {
    return p;  // Identity
}

void test_composition_chain(void) {
    char* data = getenv("INPUT");
    char* t1 = transform1(data);
    char* t2 = transform2(t1);
    sink(t2);  // Taint should flow through composition
}

// Test composition with kill
char* kill_taint(char* p) {
    (void)p;
    return "constant";  // Kills taint
}

void test_composition_with_kill(void) {
    char* data = getenv("SECRET");
    char* killed = kill_taint(data);
    sink(killed);  // Should NOT be tainted
}

// Test branching composition (join at merge)
void test_branch_composition(int cond) {
    char* data = getenv("DATA");
    char* result;

    if (cond) {
        result = transform1(data);
    } else {
        result = transform2(data);
    }

    // Join of two Identity compositions should still be tainted
    sink(result);
}
