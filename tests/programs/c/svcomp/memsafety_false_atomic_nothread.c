// valid-memsafety FALSE, sequential: uses __VERIFIER_atomic_* (which are NOT
// thread-spawn — a sequential program may use them as no-ops) around an
// unconditional heap OOB. Regression guard: the threading abstain must key on
// actual thread creation (pthread_create/thrd_create), NOT atomics/mutex/fork,
// or it false-abstains on sequential programs and craters memsafety recall.
#include <stdlib.h>
extern void __VERIFIER_atomic_begin(void);
extern void __VERIFIER_atomic_end(void);

int main(void) {
    int *a = (int *)malloc(4 * sizeof(int));
    if (!a) return 0;
    __VERIFIER_atomic_begin();
    a[100] = 1; // heap-buffer-overflow (no threads involved)
    __VERIFIER_atomic_end();
    return a[0];
}
