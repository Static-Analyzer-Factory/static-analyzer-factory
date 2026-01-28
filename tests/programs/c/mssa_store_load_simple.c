/* mssa_store_load_simple.c — Basic Memory SSA disambiguation.
 *
 * Two non-aliasing pointers p→a and q→b. Store through p, then store
 * through q, then load from p. The clobber walker should skip S2
 * (q→b doesn't alias p→a) and find S1 as the clobbering def for L1.
 */
#include <stdlib.h>

extern void sink(int);
extern int source(void);

void test(void) {
    int a, b;
    int *p = &a;
    int *q = &b;
    *p = source();  /* S1: store to a */
    *q = 99;        /* S2: store to b */
    int x = *p;     /* L1: load from a — clobber should be S1, not S2 */
    sink(x);
}

int main(void) {
    test();
    return 0;
}
