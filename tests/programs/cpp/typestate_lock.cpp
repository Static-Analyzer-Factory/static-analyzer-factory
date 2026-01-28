// Typestate E2E: pthread mutex lock without unlock → non-accepting at exit.
#include <pthread.h>

void lock_no_unlock(void) {
    pthread_mutex_t mutex;
    pthread_mutex_init(&mutex, nullptr);
    pthread_mutex_lock(&mutex);
    // Missing unlock — should report non-accepting (locked) at exit.
}
