// Test: Context-sensitive PTA (k-CFA)
// Verifies that different call sites produce different contexts
// and that the solver correctly distinguishes between them.

#include <stdlib.h>

// Test 1: Simple identity wrapper - classic k-CFA test
// With k>=1, r1 and r2 should NOT alias (different contexts)
void* identity(void* p) {
    return p;
}

void test_identity_wrapper(void) {
    int a, b;
    void* r1 = identity(&a);  // Call site 1: context [call1]
    void* r2 = identity(&b);  // Call site 2: context [call2]

    // With k=0 (CI): r1 may alias r2 (both point to {&a, &b})
    // With k>=1: r1 points to {&a}, r2 points to {&b} (no alias)
    (void)r1;
    (void)r2;
}

// Test 2: Nested wrapper calls - tests k=2
void* wrapper2(void* p) {
    return identity(p);  // Nested call
}

void test_nested_wrappers(void) {
    int x, y;
    void* r1 = wrapper2(&x);  // [call1, call_identity]
    void* r2 = wrapper2(&y);  // [call2, call_identity]

    // With k>=2: r1 and r2 should NOT alias
    (void)r1;
    (void)r2;
}

// Test 3: Triple nesting - tests k=3
void* wrapper3(void* p) {
    return wrapper2(p);
}

void test_triple_nesting(void) {
    int a, b;
    void* r1 = wrapper3(&a);
    void* r2 = wrapper3(&b);

    // With k>=3: r1 and r2 should NOT alias
    (void)r1;
    (void)r2;
}

// Test 4: Allocation wrapper (malloc wrapper pattern)
void* my_alloc(size_t size) {
    return malloc(size);
}

void test_alloc_wrapper(void) {
    void* p1 = my_alloc(10);  // Allocsite in context [call1]
    void* p2 = my_alloc(20);  // Allocsite in context [call2]

    // With k>=1: p1 and p2 should point to different abstract objects
    // because the malloc inside my_alloc has different contexts
    (void)p1;
    (void)p2;
}

// Test 5: Factory pattern (returns newly allocated object)
typedef struct { int data; } Object;

Object* make_object(int value) {
    Object* obj = (Object*)malloc(sizeof(Object));
    obj->data = value;
    return obj;
}

void test_factory_pattern(void) {
    Object* o1 = make_object(1);  // Context [call1]
    Object* o2 = make_object(2);  // Context [call2]

    // With k>=1: o1 and o2 point to different abstract objects
    (void)o1;
    (void)o2;
}

// Test 6: Same caller, different callees - tests interprocedural precision
void* pass_through_a(void* p) {
    return identity(p);
}

void* pass_through_b(void* p) {
    return identity(p);
}

void test_different_callers(void) {
    int x;
    // Both call identity, but through different intermediate functions
    void* r1 = pass_through_a(&x);
    void* r2 = pass_through_b(&x);

    // r1 and r2 should both point to &x (same target, but different paths)
    (void)r1;
    (void)r2;
}

// Test 7: Context with branching (tests context propagation through phi)
void* maybe_wrap(int cond, void* p) {
    if (cond) {
        return identity(p);
    }
    return p;
}

void test_context_with_branch(void) {
    int a;
    void* r = maybe_wrap(1, &a);
    (void)r;
}

int main(void) {
    test_identity_wrapper();
    test_nested_wrappers();
    test_triple_nesting();
    test_alloc_wrapper();
    test_factory_pattern();
    test_different_callers();
    test_context_with_branch();
    return 0;
}
