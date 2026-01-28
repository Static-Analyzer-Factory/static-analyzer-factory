/* mssa_phi_merge.c — Control flow join in Memory SSA.
 *
 * Stores in both if/else branches, then loads at the merge point.
 * Memory SSA should place a Phi at the merge block, with operands
 * from both the then-branch and else-branch stores.
 */
#include <stdlib.h>

extern void sink(int);

void test(int *p, int cond) {
    if (cond) {
        *p = 1;   /* S1: then-branch store */
    } else {
        *p = 2;   /* S2: else-branch store */
    }
    int x = *p;   /* L1: should see Phi(S1, S2) */
    sink(x);
}

int main(void) {
    int v;
    test(&v, 1);
    return 0;
}
