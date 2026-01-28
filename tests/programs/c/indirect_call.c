// Test for PTA-absint integration: indirect call resolution
// Demonstrates return interval joining from multiple callees

int return_ten() {
    return 10;
}

int return_twenty() {
    return 20;
}

int return_thirty() {
    return 30;
}

typedef int (*fn_ptr)();

// Test 1: Single target indirect call
void test_single_target() {
    fn_ptr fp = return_ten;
    int result = fp();  // Should resolve to single target
    // result should be [10, 10]
}

// Test 2: Multiple targets - join of return values
void test_multiple_targets(int cond) {
    fn_ptr fp;
    if (cond > 0) {
        fp = return_ten;
    } else {
        fp = return_twenty;
    }

    int result = fp();  // Should join results: [10, 20]
    // result should be [10, 20]
}

// Test 3: Three targets
void test_three_targets(int cond) {
    fn_ptr fp;
    if (cond > 10) {
        fp = return_thirty;
    } else if (cond > 0) {
        fp = return_twenty;
    } else {
        fp = return_ten;
    }

    int result = fp();  // Should join results: [10, 30]
    // result should be [10, 30]
}

int main() {
    test_single_target();
    test_multiple_targets(1);
    test_three_targets(5);
    return 0;
}
