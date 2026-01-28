// Test: Sparse Flow-Sensitive PTA strong/weak update conditions
// Verifies the three conditions for strong updates:
// 1. Singleton points-to set (single target)
// 2. Non-array target
// 3. Non-recursive store location

#include <stdlib.h>

// =============================================================================
// Strong Update Tests (all conditions satisfied)
// =============================================================================

// Test 1: Basic strong update - singleton, non-array, non-recursive
void test_basic_strong_update(void) {
    int x = 1;
    int y = 2;
    int *p = &x;  // p -> {x}

    // Strong update: p points to exactly x
    *p = 10;  // Should kill old value of x

    int *q = &y;
    *q = 20;  // Strong update on y

    int result = *p + *q;
    (void)result;
}

// Test 2: Strong update through local pointer
void test_local_pointer_strong_update(void) {
    int local;
    int *ptr = &local;  // Singleton: ptr -> {local}

    *ptr = 100;  // Strong update (first store)
    *ptr = 200;  // Strong update (kills previous)

    int val = *ptr;  // Should see 200, not 100
    (void)val;
}

// Test 3: Strong update with struct field (non-array)
typedef struct {
    int field1;
    int field2;
} SimpleStruct;

void test_struct_field_strong_update(void) {
    SimpleStruct s;
    int *p = &s.field1;  // Singleton to field1

    *p = 1;  // Strong update on field1
    *p = 2;  // Strong update (kills previous)

    int v = *p;
    (void)v;
}

// =============================================================================
// Weak Update Tests (at least one condition violated)
// =============================================================================

// Test 4: Weak update due to non-singleton points-to set
void test_weak_update_non_singleton(int cond) {
    int x, y;
    int *p;

    if (cond) {
        p = &x;  // p -> {x} on this path
    } else {
        p = &y;  // p -> {y} on this path
    }
    // After merge: p -> {x, y} (non-singleton)

    *p = 42;  // WEAK update: don't know which target

    // Both x and y may have been modified
    int r1 = x;
    int r2 = y;
    (void)r1;
    (void)r2;
}

// Test 5: Weak update due to array element
void test_weak_update_array(int idx) {
    int arr[10];
    int *p = &arr[idx];  // p -> {arr[?]} - array element

    *p = 100;  // WEAK update: array elements require weak update

    int v = arr[0];  // arr[0] may or may not be modified
    (void)v;
}

// Test 6: Weak update with heap array
void test_weak_update_heap_array(int idx) {
    int *arr = (int*)malloc(10 * sizeof(int));
    int *p = arr + idx;  // Points into heap array

    *p = 50;  // WEAK update: heap array element

    int v = arr[0];
    (void)v;
    free(arr);
}

// =============================================================================
// Flow-Sensitivity Tests (order matters)
// =============================================================================

// Test 7: Kill/gen sequence demonstrates flow sensitivity
void test_kill_gen_sequence(void) {
    int x;
    int *p = &x;

    *p = 1;  // GEN: x = 1
    // At this point: x has value 1

    *p = 2;  // KILL old, GEN: x = 2
    // At this point: x has value 2 (1 is killed)

    *p = 3;  // KILL old, GEN: x = 3
    // At this point: x has value 3

    int final = *p;
    (void)final;
}

// Test 8: Flow sensitivity with branches
void test_flow_with_branches(int cond) {
    int x = 0;
    int *p = &x;

    if (cond) {
        *p = 10;  // Strong update in then-branch
    } else {
        *p = 20;  // Strong update in else-branch
    }

    // After merge: x may be 10 or 20
    // But NOT 0 (killed by both branches)
    int v = *p;
    (void)v;
}

// Test 9: Partial kill with branch (one path modifies)
void test_partial_kill(int cond) {
    int x = 0;
    int *p = &x;

    if (cond) {
        *p = 100;  // Modifies x on this path only
    }
    // else: x unchanged

    // After merge: x may be 0 or 100
    int v = *p;
    (void)v;
}

// =============================================================================
// Complex Scenarios
// =============================================================================

// Test 10: Mixed strong/weak in same function
void test_mixed_updates(int cond) {
    int a, b;
    int arr[5];
    int *p1 = &a;      // Singleton
    int *p2;

    if (cond) {
        p2 = &a;
    } else {
        p2 = &b;
    }
    // p1 -> {a} (singleton), p2 -> {a,b} (non-singleton)

    *p1 = 1;  // Strong update
    *p2 = 2;  // Weak update

    int *parr = &arr[0];
    *parr = 3;  // Weak update (array)

    int r1 = a;
    int r2 = b;
    (void)r1;
    (void)r2;
}

// Test 11: Function call between stores (tests interprocedural)
void modify_through_ptr(int *ptr) {
    *ptr = 999;  // Strong update if ptr is singleton in caller's context
}

void test_interprocedural_update(void) {
    int x = 0;
    int *p = &x;

    *p = 1;  // Strong update
    modify_through_ptr(p);  // Call modifies x
    // After call: x = 999

    int v = *p;
    (void)v;
}

// Test 12: Loop with pointer (iteration affects flow sensitivity)
void test_loop_update(int n) {
    int x = 0;
    int *p = &x;

    for (int i = 0; i < n; i++) {
        *p = i;  // Strong update each iteration
    }
    // After loop: x = n-1 (or 0 if n <= 0)

    int v = *p;
    (void)v;
}

// Test 13: Nested struct fields
typedef struct {
    int val;
} Inner;

typedef struct {
    Inner inner;
    int other;
} Outer;

void test_nested_struct_update(void) {
    Outer o;
    int *p = &o.inner.val;  // Singleton to nested field

    *p = 42;  // Strong update on inner.val

    int v = o.inner.val;
    (void)v;
}

// Test 14: Aliased pointers (same target, different variables)
void test_aliased_pointers(void) {
    int x;
    int *p = &x;
    int *q = &x;  // q aliases p

    *p = 1;  // Strong update via p
    // x = 1

    *q = 2;  // Strong update via q (same target)
    // x = 2 (kills previous)

    int v = *p;  // Should see 2
    (void)v;
}

int main(void) {
    test_basic_strong_update();
    test_local_pointer_strong_update();
    test_struct_field_strong_update();
    test_weak_update_non_singleton(1);
    test_weak_update_array(0);
    test_weak_update_heap_array(0);
    test_kill_gen_sequence();
    test_flow_with_branches(1);
    test_partial_kill(1);
    test_mixed_updates(1);
    test_interprocedural_update();
    test_loop_update(5);
    test_nested_struct_update();
    test_aliased_pointers();
    return 0;
}
