// Simple MTA test program
// Tests thread discovery and MHP analysis with pthread_create/pthread_join

#include <pthread.h>
#include <stdio.h>

int shared_var = 0;

void* thread_func(void* arg) {
    // Thread 1 runs here
    shared_var++;
    printf("Thread 1: shared_var = %d\n", shared_var);
    return NULL;
}

int main() {
    pthread_t t1;

    // Before pthread_create: only main thread (thread 0)
    shared_var = 0;

    // Create thread 1
    pthread_create(&t1, NULL, thread_func, NULL);

    // Between create and join: threads 0 and 1 may run concurrently
    shared_var++;
    printf("Main: shared_var = %d\n", shared_var);

    // Wait for thread 1 to complete
    pthread_join(t1, NULL);

    // After join: only main thread (thread 0)
    shared_var++;
    printf("Final: shared_var = %d\n", shared_var);

    return 0;
}
