// Test for interprocedural abstract interpretation.
//
// Tests SAF's ability to track value ranges across function boundaries.

// Simple function that returns a constant
int return_constant(void) {
    return 42;
}

// Function with parameter that returns it
int identity(int x) {
    return x;
}

// Function that adds 1 to parameter
int add_one(int x) {
    return x + 1;
}

// Function that clamps to range [0, 100]
int clamp_to_100(int x) {
    if (x < 0) return 0;
    if (x > 100) return 100;
    return x;
}

// Test: return value should be [42, 42]
int test_constant_return(void) {
    int v = return_constant();
    // v should be exactly 42
    return v;
}

// Test: return value should be [10, 10] when passing 10
int test_identity_call(void) {
    int v = identity(10);
    // v should be exactly 10
    return v;
}

// Test: return value should be [11, 11] when passing 10
int test_add_one_call(void) {
    int v = add_one(10);
    // v should be exactly 11
    return v;
}

// Test: return value should be [0, 100]
int test_clamp_call(int input) {
    int v = clamp_to_100(input);
    // v should be in [0, 100] regardless of input
    return v;
}

// Test: chained calls
int test_chained_calls(void) {
    int a = return_constant();  // [42, 42]
    int b = add_one(a);         // [43, 43]
    int c = identity(b);        // [43, 43]
    return c;
}

// Test: multiple callers (context sensitivity)
int multi_caller_callee(int x) {
    return x * 2;
}

int test_multi_caller_1(void) {
    return multi_caller_callee(5);  // Should be [10, 10]
}

int test_multi_caller_2(void) {
    return multi_caller_callee(10); // Should be [20, 20]
}

int main(void) {
    test_constant_return();
    test_identity_call();
    test_add_one_call();
    test_clamp_call(50);
    test_chained_calls();
    test_multi_caller_1();
    test_multi_caller_2();
    return 0;
}
