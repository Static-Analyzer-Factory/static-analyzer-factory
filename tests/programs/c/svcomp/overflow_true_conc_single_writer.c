// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <limits.h>
extern int __VERIFIER_nondet_int(void);
int i, n;
void *thr(void *a) { while (i < n) { i++; } return 0; }
int main(void) { pthread_t t; n = __VERIFIER_nondet_int(); pthread_create(&t, 0, thr, 0); pthread_join(t, 0); return 0; }
