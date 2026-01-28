// Test: PTA constraint extraction edge cases
// Verifies that SAF extracts all constraint types correctly

#include <stdlib.h>

// Global variable (tests Global → Addr constraint)
int global_x;
int *global_ptr;

// Global initializers with function pointers (tests extract_global_initializers)
typedef void (*func_ptr_t)(void);

void target_func1(void) {}
void target_func2(void) {}

// This tests vtable-like patterns (function pointers in global aggregates)
struct VTable {
    func_ptr_t slot0;
    func_ptr_t slot1;
};

// Test 1: Basic alloca + store + load
void test_basic_store_load(void) {
    int x;           // alloca → Addr
    int *p = &x;     // Copy (of address)
    *p = 42;         // Store
    int y = *p;      // Load
    (void)y;
}

// Test 2: Copy chain propagation
void test_copy_chain(void) {
    int x;
    int *a = &x;
    int *b = a;      // Copy
    int *c = b;      // Copy
    int *d = c;      // Copy → all should point to &x
    (void)d;
}

// Test 3: Phi node (branch merge)
void test_phi_merge(int cond) {
    int x, y;
    int *p;
    if (cond) {
        p = &x;
    } else {
        p = &y;
    }
    // p here has a Phi: p may point to {&x, &y}
    int z = *p;
    (void)z;
}

// Test 4: Select instruction (conditional expression)
void test_select(int cond) {
    int x, y;
    int *p = cond ? &x : &y;  // Select → two Copy constraints
    int z = *p;
    (void)z;
}

// Test 5: GEP (field access)
struct Point {
    int x;
    int y;
};

void test_gep_field(void) {
    struct Point pt;
    int *px = &pt.x;  // GEP at field 0
    int *py = &pt.y;  // GEP at field 1
    *px = 1;
    *py = 2;
}

// Test 6: Array access (GEP with Index)
void test_gep_array(void) {
    int arr[10];
    int *p = &arr[0];  // GEP with Index
    int *q = &arr[5];  // GEP with Index
    *p = 1;
    *q = 2;
}

// Test 7: Heap allocation (malloc → Addr)
void test_heap_alloc(void) {
    int *p = (int*)malloc(sizeof(int));
    *p = 42;
    free(p);
}

// Test 8: Global variable access
void test_global(void) {
    int *p = &global_x;  // Global → Addr
    global_ptr = p;      // Store to global
    int *q = global_ptr; // Load from global
    (void)q;
}

// Test 9: Cast (should propagate as Copy)
void test_cast(void) {
    int x;
    void *v = (void*)&x;  // Cast → Copy
    int *p = (int*)v;     // Cast → Copy
    *p = 42;
}

// Test 10: Interprocedural (arg → param, return → caller)
int* identity(int *p) {
    return p;  // Return constraint
}

void test_interprocedural(void) {
    int x;
    int *a = &x;
    int *b = identity(a);  // arg→param and return→caller
    // b should point to &x
    (void)b;
}

// Test 11: Function pointer in call (indirect call placeholder)
void test_func_ptr(func_ptr_t fp) {
    fp();  // CallIndirect → IndirectPlaceholder
}

int main(void) {
    test_basic_store_load();
    test_copy_chain();
    test_phi_merge(1);
    test_select(1);
    test_gep_field();
    test_gep_array();
    test_heap_alloc();
    test_global();
    test_cast();
    test_interprocedural();
    test_func_ptr(target_func1);
    return 0;
}
