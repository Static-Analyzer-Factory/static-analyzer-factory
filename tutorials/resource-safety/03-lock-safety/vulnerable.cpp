/**
 * Threading code with pthread mutex typestate bugs.
 *
 * Bugs detected by typestate analysis:
 * 1. acquire_no_release(): lock acquired but never released — held lock at exit
 * 2. acquire_release_correct(): proper lock/unlock — no bug
 */
#include <pthread.h>

/* BUG 1: Lock acquired but never released */
void acquire_no_release(void) {
    pthread_mutex_t mtx;
    pthread_mutex_init(&mtx, nullptr);
    pthread_mutex_lock(&mtx);
    /* Missing pthread_mutex_unlock — held lock at exit */
}

/* CORRECT: Proper lock/unlock pair */
void acquire_release_correct(void) {
    pthread_mutex_t mtx;
    pthread_mutex_init(&mtx, nullptr);
    pthread_mutex_lock(&mtx);
    pthread_mutex_unlock(&mtx);
}
