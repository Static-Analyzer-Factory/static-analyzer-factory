// DDA Budget Fallback Test
// Tests: Complex program structure that may trigger budget exhaustion.
// Expected: DDA should fall back to CI-PTA when budget is exhausted.

#include <stdlib.h>

// Chain of wrapper functions to create deep call context
int *wrap1(int *p) { return p; }
int *wrap2(int *p) { return wrap1(p); }
int *wrap3(int *p) { return wrap2(p); }
int *wrap4(int *p) { return wrap3(p); }
int *wrap5(int *p) { return wrap4(p); }
int *wrap6(int *p) { return wrap5(p); }
int *wrap7(int *p) { return wrap6(p); }
int *wrap8(int *p) { return wrap7(p); }
int *wrap9(int *p) { return wrap8(p); }
int *wrap10(int *p) { return wrap9(p); }

int main() {
    int x = 42;
    int *p = &x;

    // Deep call chain - may exhaust context budget
    int *q = wrap10(p);

    return *q;
}
