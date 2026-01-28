/* fspta_branch_merge.c — if/else pointer assignment
 *
 * After the branch, p may point to either a or b depending on the path.
 * Flow-sensitive PTA should track per-path: p->{a} in true branch,
 * p->{b} in false branch, p->{a,b} after merge.
 * Andersen CI always reports p->{a,b} everywhere.
 */
#include <stdlib.h>

int a_val, b_val;

void test_branch_merge(int cond) {
    int *p;
    if (cond) {
        p = &a_val;   /* true branch: p -> {a_val} */
    } else {
        p = &b_val;   /* false branch: p -> {b_val} */
    }
    /* After merge: p -> {a_val, b_val} */
    *p = 42;
}

int main(void) {
    test_branch_merge(1);
    return 0;
}
