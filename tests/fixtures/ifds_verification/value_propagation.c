// Phase 5: IDE value propagation (Phase 2 of IDE algorithm)
// Tests for correct value propagation through jump functions

extern int open(const char*, int);
extern int close(int);
extern int read(int, void*, int);
extern int write(int, const void*, int);

// Typestate test: file operations with multiple states
// States: Uninit -> Opened -> Closed
void test_basic_typestate(void) {
    int fd = open("file.txt", 0);  // Uninit -> Opened
    char buf[64];
    read(fd, buf, 64);             // Opened -> Opened (valid)
    close(fd);                     // Opened -> Closed
    // File is now in Closed state
}

// Test value join at merge points
void test_value_join_at_merge(int cond) {
    int fd = open("test.txt", 0);  // Opened

    if (cond) {
        close(fd);  // Closed in this branch
    }
    // else: fd still Opened

    // At merge: join(Opened, Closed) should be error or top
    // Reading here is potentially invalid
}

// Test value propagation through call chain
void helper_close(int fd) {
    close(fd);
}

void test_interproc_value_propagation(void) {
    int fd = open("data.txt", 0);  // Opened
    helper_close(fd);              // Calls close -> Closed
    // fd should be Closed after helper_close returns
}

// Test double close detection
void test_double_close(void) {
    int fd = open("important.txt", 0);
    close(fd);  // First close -> Closed
    close(fd);  // Second close -> Error (double close)
}

// Test leak detection (no close on any path)
void test_file_leak(int cond) {
    int fd = open("leaked.txt", 0);

    if (cond) {
        return;  // Leak: fd not closed
    }

    close(fd);  // Only closed on one path
}

// Test correct usage (open, use, close on all paths)
void test_correct_usage(int cond) {
    int fd = open("correct.txt", 0);

    if (cond) {
        char buf[32];
        read(fd, buf, 32);
    } else {
        write(fd, "data", 4);
    }

    close(fd);  // Closed on all paths - correct
}
