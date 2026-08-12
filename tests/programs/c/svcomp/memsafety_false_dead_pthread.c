// valid-memsafety FALSE with DEAD pthread scaffolding (the sv-benchmarks Juliet
// shape, plan 198): `pthread_create` is present in the module but UNREACHABLE from
// `main` (main runs the faulting sink directly; the thread wrapper is never called).
// The reachability-refined thread gate proves the execution sequential, so the ASan
// confirmer reproduces the unconditional fault -> false(valid-deref). The old
// symbol-presence gate (`program_spawns_threads`) wrongly abstained here.
#include <pthread.h>
#include <stdlib.h>

static void *worker(void *a) {
    (void)a;
    return (void *)0;
}

// External linkage -> always emitted at -O0; references pthread_create so the symbol
// is linked into the module. NOTHING reachable from main calls this (dead scaffolding).
int dead_thread_wrapper(void) {
    pthread_t t;
    if (pthread_create(&t, (const pthread_attr_t *)0, worker, (void *)0) == 0) {
        pthread_join(t, (void **)0);
    }
    return 0;
}

int main(void) {
    int *a = (int *)malloc(4 * sizeof(int));
    if (!a) return 0;
    a[100] = 1; // unconditional heap-buffer-overflow in main (thread T0)
    return a[0];
}
