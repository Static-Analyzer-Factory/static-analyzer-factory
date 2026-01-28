// Phase 5: Lattice property tests
// Tests that join/meet/leq satisfy lattice laws through computed intervals.
//
// These tests create scenarios where we can verify lattice properties
// by observing the computed intervals at specific program points.

void sink(int);

// Join at merge point - tests commutativity and upper bound
void join_merge(int cond) {
    int x;
    if (cond) {
        x = 10;  // x = [10, 10]
    } else {
        x = 20;  // x = [20, 20]
    }
    // After merge: join([10,10], [20,20]) = [10, 20]
    // Commutativity: same result regardless of branch order
    sink(x);
}

// Three-way join - tests associativity
void join_three_way(int mode) {
    int x;
    if (mode == 0) {
        x = 5;   // [5, 5]
    } else if (mode == 1) {
        x = 15;  // [15, 15]
    } else {
        x = 25;  // [25, 25]
    }
    // join(join([5,5], [15,15]), [25,25]) should equal
    // join([5,5], join([15,15], [25,25]))
    // Result: [5, 25]
    sink(x);
}

// Join idempotence - same value from both branches
void join_idempotent(int cond) {
    int x;
    if (cond) {
        x = 42;
    } else {
        x = 42;
    }
    // join([42,42], [42,42]) = [42,42] (idempotent)
    sink(x);
}

// Meet via branch condition - intersection
void meet_branch(int x) {
    // x is top initially
    if (x >= 0) {
        if (x <= 100) {
            // meet([-inf, +inf], [0, +inf], [-inf, 100]) = [0, 100]
            sink(x);
        }
    }
}

// Bottom absorption - unreachable code
void bottom_absorption(int x) {
    if (x > 0 && x < 0) {
        // Unreachable: contradictory conditions
        // State should be bottom
        sink(x);  // This code is dead
    }
}

// Leq verification through subset
void leq_subset(int mode) {
    int x;
    if (mode) {
        x = 5;  // [5, 5]
    } else {
        x = 3;  // [3, 3]
    }
    // After merge: [3, 5]

    // [3, 5] ⊑ [0, 10]? Yes
    // [3, 5] ⊑ [4, 6]? No (3 < 4)
    sink(x);
}

// Top and bottom as identity/absorber
void top_bottom_identity(int* ptr) {
    int x;
    if (ptr) {
        x = *ptr;  // x is top (unknown loaded value)
    } else {
        x = 0;     // x = [0, 0]
    }
    // join(top, [0,0]) = top (top absorbs in join)
    sink(x);
}
