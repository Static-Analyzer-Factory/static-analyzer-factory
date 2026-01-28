/* fspta_interproc.c — interprocedural store/load through parameter
 *
 * A function receives a pointer and stores to it. The caller then
 * loads the result. Flow-sensitive PTA tracks the store/load across
 * the call boundary via SVFG indirect edges.
 */
#include <stdlib.h>

int target_val;

void set_ptr(int **pp) {
    *pp = &target_val;
}

int test_interproc(void) {
    int *p = NULL;
    set_ptr(&p);
    /* After set_ptr: p -> {target_val} */
    return *p;
}

int main(void) {
    return test_interproc();
}
