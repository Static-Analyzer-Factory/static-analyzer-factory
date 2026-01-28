// Test: PTA worklist termination with pointer cycles
// Verifies that the solver correctly handles cyclic constraint graphs

#include <stdlib.h>

// Test 1: Simple pointer cycle (p -> q -> p)
void test_simple_cycle(void) {
    void *p, *q;
    p = &q;  // p points to q's address
    q = &p;  // q points to p's address

    void *r = p;  // r should point to {&q}
    void *s = q;  // s should point to {&p}
    (void)r;
    (void)s;
}

// Test 2: Three-way cycle (a -> b -> c -> a)
void test_three_way_cycle(void) {
    void *a, *b, *c;
    a = &b;
    b = &c;
    c = &a;

    // All reach back to each other through chains
    void *x = a;
    (void)x;
}

// Test 3: Cycle through memory (store/load cycle)
void test_memory_cycle(void) {
    void *p;
    void **pp = &p;  // pp points to p

    // Store p's address into *pp (which is p itself conceptually)
    // This creates: p = &p via the pointer
    *pp = (void*)pp;

    void *r = *pp;  // Load from pp
    (void)r;
}

// Test 4: Linked list cycle (head -> node1 -> head)
struct Node {
    struct Node *next;
    int data;
};

void test_linked_list_cycle(void) {
    struct Node head;
    struct Node node1;

    head.next = &node1;
    node1.next = &head;  // Cycle back to head

    struct Node *p = head.next;
    struct Node *q = p->next;  // Should point back to head
    (void)q;
}

// Test 5: Self-referencing structure
struct SelfRef {
    struct SelfRef *self;
    int value;
};

void test_self_reference(void) {
    struct SelfRef obj;
    obj.self = &obj;  // Self-cycle

    struct SelfRef *p = obj.self;  // p points to obj
    struct SelfRef *q = p->self;   // q also points to obj
    (void)q;
}

// Test 6: Convergence test with wide union
void test_convergence_wide(void) {
    int a, b, c, d, e, f, g, h;
    int *p;

    // Many possible targets for p
    p = &a;
    p = &b;
    p = &c;
    p = &d;
    p = &e;
    p = &f;
    p = &g;
    p = &h;

    // q copies from p, should have all 8 targets
    int *q = p;
    (void)q;
}

// Test 7: Deep chain (should converge in O(n) iterations)
void test_deep_chain(void) {
    int x;
    int *p1 = &x;
    int *p2 = p1;
    int *p3 = p2;
    int *p4 = p3;
    int *p5 = p4;
    int *p6 = p5;
    int *p7 = p6;
    int *p8 = p7;
    int *p9 = p8;
    int *p10 = p9;

    // p10 should point to &x
    (void)p10;
}

// Test 8: Diamond pattern (no cycle, but tests convergence)
void test_diamond(int cond) {
    int x;
    int *a = &x;
    int *b, *c, *d;

    // Diamond: a -> b, a -> c, b -> d, c -> d
    b = a;
    c = a;
    if (cond) {
        d = b;
    } else {
        d = c;
    }
    // d should point to &x regardless of path
    (void)d;
}

int main(void) {
    test_simple_cycle();
    test_three_way_cycle();
    test_memory_cycle();
    test_linked_list_cycle();
    test_self_reference();
    test_convergence_wide();
    test_deep_chain();
    test_diamond(1);
    return 0;
}
