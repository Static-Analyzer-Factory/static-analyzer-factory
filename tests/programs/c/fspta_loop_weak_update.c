/* fspta_loop_weak_update.c — loop store to may-alias pointer
 *
 * Inside the loop, arr[i] may alias any element, so stores through
 * arr[i] cannot be strong-updated. Flow-sensitive PTA should use
 * weak update (union) in the loop body.
 */
#include <stdlib.h>

int g_a, g_b, g_c;

void test_loop_weak_update(void) {
    int *arr[3];
    arr[0] = &g_a;
    arr[1] = &g_b;
    arr[2] = &g_c;

    for (int i = 0; i < 3; i++) {
        /* arr[i] is may-alias (points to multiple locations via array index)
         * so stores through it use weak update */
        *arr[i] = i * 10;
    }
}

int main(void) {
    test_loop_weak_update();
    return 0;
}
