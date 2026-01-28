#include <pthread.h>

pthread_mutex_t mtx;
int shared_data = 0;

void process() {
    pthread_mutex_lock(&mtx);
    shared_data++;
    // BUG: lock is never released
}

int main() {
    pthread_mutex_init(&mtx, 0);
    process();
    pthread_mutex_destroy(&mtx);
    return 0;
}
