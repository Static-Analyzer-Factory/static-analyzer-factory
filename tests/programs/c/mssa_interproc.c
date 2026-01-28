/* mssa_interproc.c — Interprocedural mod/ref in Memory SSA.
 *
 * Function modify() stores to *p. The caller stores, then calls modify(),
 * then loads. The clobber walker should recognize the call to modify() as
 * a clobber of *p (via the mod/ref summary).
 */
#include <stdlib.h>

extern void sink(int);
extern int source(void);

void modify(int *p) {
    *p = 100;  /* Modifies *p */
}

void test(void) {
    int a;
    int *p = &a;
    *p = source();   /* S1 */
    modify(p);       /* Call: mod_ref says modify() modifies loc_a */
    int x = *p;      /* L1: clobber should be the call, not S1 */
    sink(x);
}

int main(void) {
    test();
    return 0;
}
