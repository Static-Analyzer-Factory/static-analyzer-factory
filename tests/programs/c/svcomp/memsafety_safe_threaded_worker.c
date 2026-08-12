// valid-memsafety SAFE with a GENUINE, reachable thread spawn: main creates one
// worker that performs a safe write to a valid heap cell and joins it. The
// reachability-refined thread gate (plan 198) sees pthread_create reachable from
// main and ABSTAINS -> unknown. Soundness guard: a threaded program's memory safety
// can be schedule-dependent, so the confirmer must never emit a verdict here (a
// schedule-specific ASan trap on a safe task would be a -16 false alarm).
#include <pthread.h>
#include <stdlib.h>

static void *worker(void *a) {
    int *p = (int *)a;
    if (p) {
        *p = 42; // safe write to a valid heap cell
    }
    return (void *)0;
}

int main(void) {
    int *x = (int *)malloc(sizeof(int));
    if (!x) return 0;
    pthread_t t;
    if (pthread_create(&t, (const pthread_attr_t *)0, worker, (void *)x) == 0) {
        pthread_join(t, (void **)0);
    }
    int r = *x;
    free(x);
    return r;
}
