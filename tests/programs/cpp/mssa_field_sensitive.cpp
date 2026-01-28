/* mssa_field_sensitive.cpp — Struct field disambiguation in Memory SSA.
 *
 * With field-sensitive PTA, stores to s.a and s.b go to different
 * abstract locations. The load from s.a should only see S1 (the store
 * to s.a), not S2 (the store to s.b).
 */
#include <cstdlib>

extern "C" void sink(int);
extern "C" int source();

struct Pair {
    int a;
    int b;
};

void test() {
    Pair s;
    s.a = source();  /* S1: GEP field 0 + store */
    s.b = 20;        /* S2: GEP field 1 + store */
    int x = s.a;     /* L1: GEP field 0 + load — clobber should be S1 */
    sink(x);
}

int main() {
    test();
    return 0;
}
