// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <limits.h>
extern int __VERIFIER_nondet_int(void);
int i;
void *thr(void *a) { if (i < 100) { int k = i + 2147483600; return (void *)(long)k; } return 0; }
int main(void) { pthread_t t; pthread_create(&t, 0, thr, 0); i = 200; return 0; }
