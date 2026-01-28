/* fspta_strong_update.c — sequential stores to the same pointer
 *
 * Flow-sensitive PTA should track that after the second store,
 * p only points to b (not a). Andersen CI cannot distinguish
 * program points and reports {a, b}.
 */
#include <stdlib.h>

int a_val, b_val;

void test_strong_update(void) {
    int *p = &a_val;   /* p -> {a_val} */
    *p = 10;
    p = &b_val;        /* p -> {b_val} (kills a_val via strong update) */
    *p = 20;
    /* At this point: flow-sensitive says p -> {b_val}
     *                Andersen CI says    p -> {a_val, b_val} */
}

int main(void) {
    test_strong_update();
    return 0;
}
