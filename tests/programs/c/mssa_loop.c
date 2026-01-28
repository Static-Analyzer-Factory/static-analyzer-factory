/* mssa_loop.c — Loop memory Phi in Memory SSA.
 *
 * A loop that reads and writes through the same pointer. The loop header
 * should have a Phi node merging the initial store (before the loop) with
 * the loop-body store (back edge).
 */
#include <stdlib.h>

extern void sink(int);

void test(int n) {
    int acc;
    int *p = &acc;
    *p = 0;                 /* S1: initial */
    for (int i = 0; i < n; i++) {
        int x = *p;         /* L1: Phi(S1, S2) at loop header */
        *p = x + 1;         /* S2: loop body */
    }
    int result = *p;        /* L2: Phi(S1, S2) at loop exit */
    sink(result);
}

int main(void) {
    test(10);
    return 0;
}
