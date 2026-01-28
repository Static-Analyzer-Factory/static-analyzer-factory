// Test: k-CFA SCC collapse for recursive functions
// Verifies that the solver correctly handles recursion without unbounded context growth

#include <stdlib.h>

// Test 1: Simple self-recursion
// SCC: {factorial}
int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);  // Self-recursive call
}

void test_self_recursion(void) {
    int result = factorial(5);
    (void)result;
}

// Test 2: Mutual recursion (A calls B, B calls A)
// SCC: {is_even, is_odd}
int is_even(int n);
int is_odd(int n);

int is_even(int n) {
    if (n == 0) return 1;
    return is_odd(n - 1);  // Calls is_odd
}

int is_odd(int n) {
    if (n == 0) return 0;
    return is_even(n - 1);  // Calls is_even
}

void test_mutual_recursion(void) {
    int r1 = is_even(4);
    int r2 = is_odd(3);
    (void)r1;
    (void)r2;
}

// Test 3: Recursive with pointer parameter
// Tests context propagation through recursive pointer-passing
void* recursive_identity(void* p, int depth) {
    if (depth <= 0) return p;
    return recursive_identity(p, depth - 1);
}

void test_recursive_pointer(void) {
    int x;
    void* r = recursive_identity(&x, 5);
    (void)r;
}

// Test 4: Three-way mutual recursion
// SCC: {func_a, func_b, func_c}
void* func_a(void* p, int n);
void* func_b(void* p, int n);
void* func_c(void* p, int n);

void* func_a(void* p, int n) {
    if (n <= 0) return p;
    return func_b(p, n - 1);
}

void* func_b(void* p, int n) {
    if (n <= 0) return p;
    return func_c(p, n - 1);
}

void* func_c(void* p, int n) {
    if (n <= 0) return p;
    return func_a(p, n - 1);  // Completes the cycle
}

void test_three_way_recursion(void) {
    int val;
    void* r = func_a(&val, 10);
    (void)r;
}

// Test 5: Recursive allocation (tests heap site in recursive context)
typedef struct Node {
    int value;
    struct Node* next;
} Node;

Node* build_list(int n) {
    if (n <= 0) return NULL;
    Node* node = (Node*)malloc(sizeof(Node));
    node->value = n;
    node->next = build_list(n - 1);  // Recursive call with allocation
    return node;
}

void test_recursive_allocation(void) {
    Node* list = build_list(5);
    (void)list;
}

// Test 6: Indirect recursion through function pointer
typedef void* (*RecFn)(void*, int);

void* indirect_recurse(void* p, int n) {
    if (n <= 0) return p;
    RecFn fn = indirect_recurse;  // Store function pointer
    return fn(p, n - 1);          // Call through pointer
}

void test_indirect_recursion(void) {
    int x;
    void* r = indirect_recurse(&x, 3);
    (void)r;
}

// Test 7: Recursive wrapper pattern
// Non-recursive entry calls recursive helper
void* helper_recursive(void* p, int depth);

void* wrapper_entry(void* p) {
    return helper_recursive(p, 5);  // Non-recursive entry
}

void* helper_recursive(void* p, int depth) {
    if (depth <= 0) return p;
    return helper_recursive(p, depth - 1);  // Recursive
}

void test_recursive_wrapper(void) {
    int a, b;
    // Two calls to non-recursive entry
    // With proper SCC handling, these should still get distinct contexts
    void* r1 = wrapper_entry(&a);
    void* r2 = wrapper_entry(&b);
    (void)r1;
    (void)r2;
}

// Test 8: Diamond with recursion
// A -> B -> D (recursive)
// A -> C -> D (recursive)
void* diamond_d(void* p, int n);

void* diamond_b(void* p, int n) {
    return diamond_d(p, n);
}

void* diamond_c(void* p, int n) {
    return diamond_d(p, n);
}

void* diamond_d(void* p, int n) {
    if (n <= 0) return p;
    return diamond_d(p, n - 1);  // Self-recursive
}

void* diamond_a(void* p, int cond, int n) {
    if (cond) {
        return diamond_b(p, n);
    } else {
        return diamond_c(p, n);
    }
}

void test_diamond_recursive(void) {
    int x;
    void* r = diamond_a(&x, 1, 5);
    (void)r;
}

int main(void) {
    test_self_recursion();
    test_mutual_recursion();
    test_recursive_pointer();
    test_three_way_recursion();
    test_recursive_allocation();
    test_indirect_recursion();
    test_recursive_wrapper();
    test_diamond_recursive();
    return 0;
}
