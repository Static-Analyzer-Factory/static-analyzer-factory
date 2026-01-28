// Phase 3: Mod/Ref Summary with SCC handling
// Tests: Mutual recursion, self-recursion, transitive effects

extern void sink(int);

// Global for testing mod/ref
int global_x;
int global_y;

// Self-recursive function
// mod: {global_x}, ref: {global_x}
void self_recursive(int n, int* p) {
    if (n <= 0) {
        *p = 0;
        return;
    }
    *p = n;
    self_recursive(n - 1, p);  // Recursive call
}

// Mutual recursion: ping <-> pong
// Both should have mod/ref for global_x after fixed point
void pong(int n);

void ping(int n) {
    global_x = n;  // mod: global_x
    if (n > 0) {
        pong(n - 1);  // call pong
    }
}

void pong(int n) {
    int tmp = global_x;  // ref: global_x
    global_y = tmp;      // mod: global_y
    if (n > 0) {
        ping(n - 1);  // call ping
    }
}

// Three-way mutual recursion cycle
void func_b(int n);
void func_c(int n);

void func_a(int n) {
    global_x = n;
    if (n > 0) func_b(n - 1);
}

void func_b(int n) {
    global_y = n;
    if (n > 0) func_c(n - 1);
}

void func_c(int n) {
    int sum = global_x + global_y;
    if (n > 0) func_a(n - 1);
    sink(sum);
}

// Non-recursive helper (leaf in call graph)
// mod: {*p}, ref: {}
void leaf_modifier(int* p) {
    *p = 42;
}

// Caller that transitively modifies via leaf
// mod: transitive from leaf_modifier
void transitive_caller(int* p) {
    leaf_modifier(p);
}

// Test entry point
void modref_test_entry(void) {
    int local = 0;
    int* p = &local;

    self_recursive(5, p);
    sink(*p);

    ping(3);
    sink(global_x);
    sink(global_y);

    transitive_caller(p);
    sink(*p);
}
