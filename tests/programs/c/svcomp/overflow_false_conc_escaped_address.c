// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <limits.h>
extern int __VERIFIER_nondet_int(void);
int cnt = 0;
void *thr(void *a) { *(int *)a = INT_MAX; return 0; }
int main(void) { pthread_t t; pthread_create(&t, 0, thr, &cnt); pthread_join(t, 0); return cnt + 1; }
