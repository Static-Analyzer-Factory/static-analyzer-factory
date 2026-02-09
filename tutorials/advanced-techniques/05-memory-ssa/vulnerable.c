// vulnerable.c -- Memory SSA tutorial: detecting use of stale data.
//
// Two integer variables `a` and `b` are stored through pointers `p` and `q`.
// A function call `modify(p)` overwrites `*p` between the initial store and
// a subsequent load. Without Memory SSA, an analyzer might believe the load
// still reads the value written by `source()`.
//
// Memory SSA can disambiguate:
//   - S1: *p = source()     (Def to loc_a)
//   - S2: *q = 99           (Def to loc_b, does NOT clobber loc_a)
//   - C1: modify(p)         (Def -- callee's mod_ref says it modifies loc_a)
//   - L1: x = *p            (Use -- clobber is C1, NOT S1)
//
// Compile:
//   clang-18 -S -emit-llvm -O0 -g -o vulnerable.ll vulnerable.c

#include <stdlib.h>

extern int source(void);
extern void sink(int x);

void modify(int *p) {
    *p = 42;
}

void test(void) {
    int a, b;
    int *p = &a;
    int *q = &b;

    *p = source();   // S1: tainted store to a
    *q = 99;         // S2: store to b (unrelated)
    modify(p);       // C1: overwrites *p with 42 (kills the taint)
    int x = *p;      // L1: load from a -- clobber is C1, not S1
    sink(x);         // sink receives 42, not source() result
}

int main(void) {
    test();
    return 0;
}
