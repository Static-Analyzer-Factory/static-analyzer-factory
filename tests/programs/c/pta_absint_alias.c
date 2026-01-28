// Test for PTA-absint integration: alias-aware interval tracking
// Demonstrates weak updates when pointers may-alias

int x, y;
int *p, *q;

void test_alias_aware_store() {
    int a = 10;
    int b = 20;

    p = &a;

    // Conditional assignment creates may-alias between p and q
    if (a > 5) {
        q = &a;  // q may alias p
    } else {
        q = &b;  // q points to different location
    }

    // Store through p - should be strong update (p -> {&a})
    *p = 42;

    // Store through q - should be weak update (q -> {&a, &b})
    *q = 100;

    // Load through p after potential aliasing modification
    // Should reflect both possibilities: 42 or 100
    int result = *p;

    // Oracle: result should be [42, 100] due to may-alias
    MAYALIAS(p, q);  // q may alias p on one path
}

void test_singleton_strong_update() {
    int local = 5;
    int *single_ptr = &local;

    // Store through singleton pointer - strong update
    *single_ptr = 77;

    // Load should get exact value
    int val = *single_ptr;

    // val should be [77, 77]
}

int main() {
    test_alias_aware_store();
    test_singleton_strong_update();
    return 0;
}
